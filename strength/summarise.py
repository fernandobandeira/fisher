#!/usr/bin/env python3
"""Summarise the games of a match from one engine's point of view.

Usage: summarise.py [--record] GAMES.pgn ENGINE_NAME

Prints how many games the engine won, drew and lost, how many ended
abnormally, and what became of the games in which it was a queen ahead.
With --record it prints the same numbers as the tab-separated fields of
strength/records.tsv, from games to abnormal_endings.
strength/README.md defines each number. Needs python-chess:
pip install chess==1.11.2
"""

import sys

try:
    import chess
    import chess.pgn
except ImportError:
    sys.exit("python-chess is missing. Install it with: pip install chess==1.11.2")

# The values fisher's own evaluation gives each piece kind.
MATERIAL = {
    chess.PAWN: 100,
    chess.KNIGHT: 300,
    chess.BISHOP: 300,
    chess.ROOK: 500,
    chess.QUEEN: 900,
}
QUEEN_AHEAD = MATERIAL[chess.QUEEN]
OUTCOMES = ("won", "drawn", "lost")


def material_lead(position, color):
    return sum(
        value
        * (len(position.pieces(kind, color)) - len(position.pieces(kind, not color)))
        for kind, value in MATERIAL.items()
    )


def outcome(result, color):
    if result == "1/2-1/2":
        return "drawn"
    if result not in ("1-0", "0-1"):
        sys.exit(f"Unfinished game: result {result!r}")
    return "won" if (result == "1-0") == (color == chess.WHITE) else "lost"


def was_a_queen_ahead(leads):
    """True when the lead held through a reply: two positions in a row."""
    return any(
        min(before, after) >= QUEEN_AHEAD for before, after in zip(leads, leads[1:])
    )


def summarise(pgn, engine):
    outcomes = dict.fromkeys(OUTCOMES, 0)
    queen_ahead = dict.fromkeys(OUTCOMES, 0)
    abnormal = 0
    game_number = 0
    while (game := chess.pgn.read_game(pgn)) is not None:
        game_number += 1
        if game.errors:
            sys.exit(f"Game {game_number}: PGN parser errors: {game.errors}")
        names = {chess.WHITE: game.headers["White"], chess.BLACK: game.headers["Black"]}
        colors = [color for color, name in names.items() if name == engine]
        if len(colors) != 1:
            sys.exit(f"Game {game_number}: {engine!r} must play exactly one color")
        color = colors[0]
        if game.headers.get("Termination") != "normal":
            abnormal += 1
        position = game.board()
        leads = [material_lead(position, color)]
        for ply, move in enumerate(game.mainline_moves(), start=1):
            if not position.is_legal(move):
                sys.exit(f"Game {game_number}, ply {ply}: illegal move {move}")
            position.push(move)
            leads.append(material_lead(position, color))
        result = outcome(game.headers["Result"], color)
        outcomes[result] += 1
        if was_a_queen_ahead(leads):
            queen_ahead[result] += 1
    return game_number, outcomes, queen_ahead, abnormal


def main():
    arguments = sys.argv[1:]
    as_record = arguments[:1] == ["--record"]
    if as_record:
        arguments = arguments[1:]
    if len(arguments) != 2:
        sys.exit(__doc__)
    with open(arguments[0], encoding="utf-8") as pgn:
        games, outcomes, queen_ahead, abnormal = summarise(pgn, arguments[1])
    if games == 0:
        sys.exit("No games in the PGN")
    points = outcomes["won"] + outcomes["drawn"] / 2
    score = f"{100 * points / games:.1f}"
    if as_record:
        fields = [games, *outcomes.values(), score, *queen_ahead.values(), abnormal]
        print("\t".join(str(field) for field in fields))
        return
    print(f"Games: {games}")
    print("Won, drawn, lost: {won}, {drawn}, {lost}".format(**outcomes))
    print(f"Score: {score}%")
    print(
        "Queen-ahead games: {} (won {won}, drew {drawn}, lost {lost})".format(
            sum(queen_ahead.values()), **queen_ahead
        )
    )
    print(f"Abnormal endings: {abnormal}")


if __name__ == "__main__":
    main()
