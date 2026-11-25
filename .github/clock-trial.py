"""Throwaway harness for fisher-1w7."""
import collections
import json
import os
from pathlib import Path
import re
import shlex
import signal
import subprocess
import sys
import time
from decimal import Decimal

import chess
import chess.pgn


EXPECTED_WARNING = "PV continues after threefold repetition"


def milliseconds(text):
    seconds = Decimal(0)
    negative = text.startswith("-")
    for component in text.lstrip("-").split(":"):
        seconds = 60 * seconds + Decimal(component)
    return int(seconds * (-1000 if negative else 1000))


def measure(directory, control):
    warnings = collections.Counter()
    terminations = collections.Counter()
    aborted = collections.Counter()
    clocks = {}
    boards = {}
    last_move = {}
    remaining = []
    losses = []
    finished = set()
    reached_tenth = set()
    with (directory / "fastchess.log").open() as log:
        for line in log:
            if not line.startswith("{"):
                warning = re.search(r"Warning; (.*)|\bWARN\b[:\]]?\s*(.*)", line, re.I)
                if warning:
                    message = next(group for group in warning.groups() if group is not None)
                    kind = re.split(r" - (?:move|from) |:\s", message, maxsplit=1)[0]
                    warnings[kind] += 1
                continue
            try:
                event = json.loads(line)
            except json.JSONDecodeError:
                continue
            game_id = event.get("game")
            if event["event"] == "game_started":
                clocks[game_id] = {side: event[side] for side in ("wtime", "btime")}
                board = chess.Board(event["start_fen"])
                for move in event["opening"]:
                    board.push_uci(move)
                boards[game_id] = board
            elif event["event"] == "move":
                board = boards[game_id]
                side = "wtime" if event["side"] == "white" else "btime"
                fullmove = board.fullmove_number
                if side == "wtime" and fullmove == 10:
                    reached_tenth.add(game_id)
                remaining.append(clocks[game_id][side] - event["time_ms"])
                clocks[game_id][side] = event[side]
                last_move[game_id] = (event["side"], fullmove, "fen" in event)
                if "fen" in event:
                    boards[game_id] = chess.Board(event["fen"])
            elif event["event"] == "game_finished":
                finished.add(game_id)
                terminations[event["termination"]] += 1
                if event["termination"] == "time_forfeit":
                    side = "white" if event["result"] == "0-1" else "black"
                    board = boards[game_id]
                    move_number = board.fullmove_number
                    previous = last_move.get(game_id)
                    if previous and previous[0] == side and not previous[2]:
                        move_number = previous[1]
                    losses.append({"game": game_id, "side": side, "move": move_number,
                                   "reason": event["reason"]})
                    if side == "white" and move_number == 10:
                        reached_tenth.add(game_id)
            elif event["event"] == "game_aborted":
                aborted[event["reason"]] += 1

    pgn_terminations = collections.Counter()
    replay_errors = []
    pgn_ids = set()
    latencies = []
    pgn_path = directory / "games.pgn"
    if pgn_path.exists():
        with pgn_path.open() as stream:
            while (game := chess.pgn.read_game(stream)) is not None:
                game_id = int(game.headers["GameId"])
                if game_id in pgn_ids:
                    replay_errors.append(f"Duplicate game {game_id}")
                pgn_ids.add(game_id)
                pgn_terminations[game.headers.get("Termination", "missing")] += 1
                if game.errors or game.headers["Result"] not in ("1-0", "0-1", "1/2-1/2"):
                    replay_errors.append(f"Game {game_id}: {game.errors or 'unfinished'}")
                board = game.board()
                for node in game.mainline():
                    if not board.is_legal(node.move):
                        replay_errors.append(f"Game {game_id}: illegal move {node.move}")
                        break
                    board.push(node.move)
                    latency = re.search(r"\blatency=(-?[\d:.]+)", node.comment)
                    if latency:
                        latencies.append(milliseconds(latency[1]))
    return {
        "control": control,
        "games": len(finished),
        "terminations": terminations,
        "aborted": aborted,
        "losses_on_time": losses,
        "least_time_left_ms": min(remaining, default=None),
        "moves_at_or_under_ms": {limit: sum(ms <= limit for ms in remaining) for limit in (10, 20, 50)},
        "max_latency_ms": max(latencies, default=None),
        "warnings": warnings,
        "pgn_games": len(pgn_ids),
        "pgn_terminations": pgn_terminations,
        "replay_errors": replay_errors,
        "pgn_matches_events": pgn_ids == finished,
        "white_tenth_move_games": len(reached_tenth),
    }


def play(arm):
    root = Path.cwd()
    evidence = root / "evidence"
    fastchess = root / "fastchess-linux-x86-64/fastchess"
    fisher = root / "engine/target/release/fisher"
    openings = root / "engine/strength/openings.epd"
    controls = ("2+0.05", "2", "10/1") if arm == "A" else ("10/1",)
    concurrency = 8 if arm == "A" else max(1, int(subprocess.check_output(["nproc"])) // 2)
    if os.environ.get("CLOCK_WIRING") == "1":
        controls = ("2",) if arm == "A" else ("10/1",)
    started = time.monotonic()
    deadline = started + (1800 if arm == "A" else 3600)
    runs = []
    for control in controls:
        directory = evidence / control.replace("/", "-")
        directory.mkdir()
        command = [
            str(fastchess), "-engine", f"cmd={fisher}", "name=fisher-a",
            "-engine", f"cmd={fisher}", "name=fisher-b",
            "-each", "proto=uci", f"tc={control}", "timemargin=0",
            "-openings", f"file={openings}", "format=epd", "order=sequential",
            "-rounds", "500", "-games", "2", "-concurrency", str(concurrency),
            "-pgnout", "file=games.pgn", "append=false", "timeleft=true", "latency=true",
            "-events", "file=-", "info=false", "-autosaveinterval", "0", "-ratinginterval", "0",
        ]
        (directory / "command.sh").write_text(shlex.join(command) + "\n")
        timed_out = False
        match_started = time.monotonic()
        with (directory / "fastchess.log").open("w") as log:
            with subprocess.Popen(command, cwd=directory, stdout=log, stderr=subprocess.STDOUT) as process:
                try:
                    process.wait(timeout=max(0, deadline - match_started))
                except subprocess.TimeoutExpired:
                    timed_out = True
                    process.send_signal(signal.SIGINT)
                    try:
                        process.wait(timeout=10)
                    except subprocess.TimeoutExpired:
                        process.kill()
                        process.wait()
        runs.append((directory, control, round(time.monotonic() - match_started, 2), process.returncode, timed_out))
        if timed_out:
            break
    total_seconds = time.monotonic() - started
    summaries = []
    for directory, control, seconds, status, timed_out in runs:
        summary = measure(directory, control)
        summary.update(seconds=seconds, status=status, timed_out=timed_out, concurrency=concurrency)
        summary["passed"] = (
            status == 0 and not timed_out and summary["terminations"] == {"normal": 1000}
            and not summary["losses_on_time"] and not summary["aborted"]
            and summary["pgn_terminations"] == {"normal": 1000}
            and summary["pgn_matches_events"] and not summary["replay_errors"]
            and not (summary["warnings"].keys() - {EXPECTED_WARNING})
        )
        summaries.append(summary)
    passed = time.monotonic() > started and total_seconds <= deadline - started and len(runs) == len(controls) and all(
        summary["passed"] for summary in summaries
    )
    report = json.dumps({"seconds": round(total_seconds, 2), "matches": summaries, "passed": passed}, indent=2)
    (evidence / "summary.json").write_text(report + "\n")
    print(report)
    return 0 if passed else 1


if __name__ == "__main__":
    sys.exit(play(sys.argv[1]))
