"""Draw openings, play fastchess ranges, and read the kept games."""

from collections import Counter
from hashlib import sha256
from io import BytesIO
from pathlib import Path
import re
import shutil
import subprocess
import sys
from tempfile import TemporaryDirectory
from urllib.request import urlopen
from zipfile import ZipFile

from sprt_statistic import MAX_PAIRS, decide, trace


ROOT = Path(__file__).resolve().parents[1]
BOOK_URL = (
    "https://raw.githubusercontent.com/official-stockfish/books/"
    "37751b08db745aa02babfa74056dd1168b9da803/noob_3moves.epd.zip"
)
ARCHIVE_SHA256 = "68fe2691e0876d0de54fffb68c797e752138d5ed363beab2ac6ca34be68ee9f3"
BOOK_SHA256 = "2011193b4854e9a8cfdc05312ca2dbaffa6ceae3abbdee20e2ead2a18a603347"
TIME_CONTROL = "10+0.1"
ENDINGS = {
    "normal": "normal",
    "time forfeit": "loss on time",
    "abandoned": "crash or stall",
    "illegal move": "illegal move",
}
WHITE_POINTS = {"1-0": 2, "1/2-1/2": 1, "0-1": 0}
TAG = re.compile(r'\[(\w+) "((?:[^"\\]|\\.)*)"\]')
COMMENTS = re.compile(r"\{[^{}]*\}")
UCI_MOVE = re.compile(r"[a-h][1-8][a-h][1-8][qrbn]?")
MOVE_NUMBER = re.compile(r"\d+\.(?:\.\.)?")


def opening_bytes(openings):
    return ("\n".join(openings) + "\n").encode("utf-8")


def draw_openings(book, series, seed):
    return sorted(
        set(book) - set(series),
        key=lambda fen: (sha256(f"{seed}:{fen}".encode("utf-8")).digest(), fen),
    )


def fetch_openings(seed):
    with urlopen(BOOK_URL, timeout=30) as response:
        archive = response.read()
    if sha256(archive).hexdigest() != ARCHIVE_SHA256:
        raise ValueError("Opening book archive SHA-256 mismatch")
    with ZipFile(BytesIO(archive)) as book:
        epd = book.read("noob_3moves.epd")
    if sha256(epd).hexdigest() != BOOK_SHA256:
        raise ValueError("Opening book SHA-256 mismatch")
    series = (ROOT / "strength/series-openings.epd").read_text(encoding="utf-8").splitlines()
    return draw_openings(epd.decode("utf-8").splitlines(), series, seed)[:MAX_PAIRS]


def revision(ref):
    return subprocess.check_output(
        ["git", "rev-parse", "--verify", "--end-of-options", f"{ref}^{{commit}}"],
        cwd=ROOT, text=True,
    ).strip()


def build(commit, engine):
    # Build an exported snapshot so the checkout and its toolchain stay independent.
    with TemporaryDirectory(dir=engine.parent) as temporary:
        source = Path(temporary)
        archive = source / "source.tar"
        subprocess.run(["git", "archive", "-o", archive, commit], cwd=ROOT, check=True)
        subprocess.run(["tar", "-xf", archive], cwd=source, check=True)
        target = source / "target"
        subprocess.run(
            ["cargo", "build", "--locked", "--release", "-p", "fisher", "--target-dir", target],
            cwd=source, check=True, stdout=sys.stderr,
        )
        shutil.copy2(target / "release/fisher", engine)


def engine_names(measured, opponent):
    return f"measured-{measured}", f"opponent-{opponent}"


def play(out, measured, opponent, openings, concurrency):
    runner = shutil.which("fastchess")
    if runner is None:
        raise ValueError("Install fastchess with scripts/install-fastchess and put it on PATH")
    measured_name, opponent_name = engine_names(measured, opponent)
    build(measured, out / "measured")
    build(opponent, out / "opponent")
    (out / "openings.epd").write_bytes(opening_bytes(openings))
    with (out / "fastchess.log").open("w", encoding="utf-8") as log:
        subprocess.run(
            [
                runner,
                "-engine", "cmd=./measured", f"name={measured_name}",
                "-engine", "cmd=./opponent", f"name={opponent_name}",
                "-each", "proto=uci", f"tc={TIME_CONTROL}", "timemargin=0",
                "-openings", "file=openings.epd", "format=epd", "order=sequential",
                "-rounds", str(len(openings)), "-games", "2", "-concurrency", str(concurrency),
                "-pgnout", "file=games.pgn", "notation=uci", "append=false",
                "-recover", "-autosaveinterval", "0", "-ratinginterval", "0",
            ],
            cwd=out, stdout=log, stderr=subprocess.STDOUT, check=True,
        )


def read_record(record):
    """Read fastchess's UCI-notation PGN, including its final result marker."""
    header, separator, moves = record.strip().partition("\n\n")
    if not separator:
        raise ValueError("PGN game has no movetext")
    tags = {}
    for line in header.splitlines():
        match = TAG.fullmatch(line)
        if not match or match[1] in tags:
            raise ValueError(f"Invalid or duplicate PGN tag: {line}")
        tags[match[1]] = match[2]
    tokens = COMMENTS.sub(" ", moves).split()
    result = tags.get("Result")
    if result not in WHITE_POINTS or not tokens or tokens.pop() != result:
        raise ValueError("PGN game has a missing or inconsistent result")
    plies = [token for token in tokens if not MOVE_NUMBER.fullmatch(token)]
    if tags.get("PlyCount") != str(len(plies)) or not all(map(UCI_MOVE.fullmatch, plies)):
        raise ValueError("PGN game has incomplete or invalid UCI movetext")
    return tags


def pgn_records(stream):
    record = []
    for line in stream:
        if line.startswith("[Event "):
            if record:
                yield "".join(record)
            record = [line]
        elif record:
            record.append(line)
        elif line.strip():
            raise ValueError("Unexpected content before the first PGN game")
    if record:
        yield "".join(record)


def read_pairs(stream, openings, first, measured, opponent):
    """Validate all games in a range before returning scores in drawn order."""
    numbers = {fen: number for number, fen in enumerate(openings, first)}
    measured_name, opponent_name = engine_names(measured, opponent)
    games = {}
    for record in pgn_records(stream):
        tags = read_record(record)
        number = numbers.get(tags.get("FEN"))
        if number is None:
            raise ValueError(f"PGN game outside the opening range: {tags.get('FEN')!r}")
        pairing = (tags.get("White"), tags.get("Black"))
        if pairing not in ((measured_name, opponent_name), (opponent_name, measured_name)):
            raise ValueError(f"Opening {number}: unexpected engines {pairing}")
        measured_white = pairing[0] == measured_name
        key = (number, measured_white)
        if key in games:
            raise ValueError(f"Opening {number}: duplicate game with White {pairing[0]}")
        ending = tags.get("Termination")
        points = WHITE_POINTS[tags["Result"]]
        if ending not in ENDINGS or (ending != "normal" and points == 1):
            raise ValueError(f"Opening {number}: invalid termination {ending!r}")
        if tags.get("TimeControl") != TIME_CONTROL:
            raise ValueError(f"Opening {number}: unexpected time control {tags.get('TimeControl')!r}")
        games[key] = (points if measured_white else 2 - points, ENDINGS[ending])

    scores = []
    endings = Counter()
    for number in range(first, first + len(openings)):
        for measured_white in (True, False):
            if (number, measured_white) not in games:
                color = "White" if measured_white else "Black"
                raise ValueError(f"Missing game at opening {number} with the measured revision as {color}")
        white, black = (games[number, color] for color in (True, False))
        scores.append(white[0] + black[0])
        endings.update((white[1], black[1]))
    return scores, endings


def judge(scores, test):
    observation = decide(trace(scores, test))
    result = observation.verdict
    if result is None:
        result = "inconclusive" if observation.number == MAX_PAIRS else "no verdict yet"
    return observation, result
