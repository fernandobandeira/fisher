# Random legal play over UCI

Acceptance evidence for `fisher-l7f`, recorded on 2026-11-22. The release
engine tested in the match came from `aaf0218`. Rebuilding `e16f1da` after strengthening the transcripts produced the identical executable.

| Claim | Proved wrong by | Expected value from | Result | Revision |
|---|---|---|---|---|
| Legal moves are generated correctly | Any perft count differs for a published position and depth | [Chess Programming Wiki perft results](https://www.chessprogramming.org/Perft_Results), recorded in [perft.epd](../expectations/perft.epd) | All six quick counts and all six approved deep counts matched. | `d9f346c` |
| It stays correct on positions nobody picked | Legal move lists or resulting positions differ in seeded games | shakmaty 0.30.3 | All 81,884 legal moves and their resulting FENs matched across 2,688 positions in 21 fixed-seed games, including one played en passant capture, two castles, and 12 promotions. | `d9f346c` |
| Canonical FEN survives a round trip and a halfmove clock of `00` is accepted as zero | Reading and writing changes canonical FEN, rejects `00`, or fails to restore `0` | Owner-approved invariant, published positions, and seeded shakmaty games | All four FEN tests passed, including raw en passant targets and normalization of `00`. The UCI clock transcript passed. | `d8934d7`, `e16f1da` |
| Fisher speaks UCI | A process reply or exit differs from a committed transcript | [April 2006 UCI specification](https://backscattering.de/chess/uci/) and [UCI transcripts](../expectations/uci) | All 16 transcripts ran and passed, including both approved empty move-list forms. Committed parser tests checked all 12 typed `go` parameters and four position syntax errors. | `e16f1da` |
| It listens while there is a held move | `readyok` is missing, `bestmove` precedes `readyok`, or the stop or exit behavior differs from the transcript | April 2006 UCI specification and the held move transcripts | All four sessions returned `readyok` before any `bestmove` and matched the expected stop or exit behavior. | `e16f1da` |
| Bad input changes nothing | Rejected input changes the sole legal move of the current position | Owner-approved examples in `fisher-l7f.1` | Each of the four rejection cases emitted one diagnostic naming its cause, answered readiness, and preserved `h8g8`. | `e16f1da` |
| A position with no legal move is answered | A checkmate or stalemate gives anything other than `bestmove 0000` | Owner-approved examples in `fisher-l7f.1` | Both immediate replies and the held null reply matched. | `e16f1da` |
| It survives real games | A PGN termination is abnormal or the fastchess warning log contains a warning | fastchess 1.8.3-alpha `20261107-d7b47e8` | All 1,000 games ended normally and the fastchess warning log was empty. The independent PGN replay verified 338,072 legal plies. | `aaf0218` tested, `e16f1da` rebuilt byte-identically |

## Deep perft

The owner approved the depths after measuring a release build on this Mac.
The acceptance run took 33.46 seconds. The weekly published suite uses
these depths. Run [`scripts/check-published-suite`](../scripts/check-published-suite)
to repeat the full suite locally.

| Published position | Depth | Matching count |
|---|---:|---:|
| 1 | 6 | 119,060,324 |
| 2 | 5 | 193,690,690 |
| 3 | 7 | 178,633,661 |
| 4 | 5 | 15,833,292 |
| 5 | 5 | 89,941,194 |
| 6 | 5 | 164,075,551 |

```sh
FISHER_PERFT_DEPTHS=6,5,7,5,5,5 cargo test --locked --release -p rules \
    --test positions perft_at_selected_depths -- --exact --ignored --nocapture
```

## Timed self-play

Installed fastchess 1.8.3-alpha and BanksiaGUI 0.58 with the
owner's approval. The trace from a 20-game trial with the same release binary showed fastchess sent
`uci`, `isready`, `ucinewgame`, `position`, `go`, `stop`, and `quit`.
Fisher emitted no `info string` diagnostics in the trial.

The full match took 19.72 seconds. All 1,000 PGN termination tags read
`normal`, and the warning log was empty. python-chess 1.11.2 independently
replayed the PGN. The longest game lasted 680 plies.

```sh
fisher_match_dir=$(mktemp -d)
(
    set -eu
    cargo build --locked --release -p fisher
    fisher_executable="$PWD/target/release/fisher"
    shasum -a 256 "$fisher_executable"
    cd "${fisher_match_dir:?}"
    fastchess \
        -engine "cmd=$fisher_executable" name=fisher-a \
        -engine "cmd=$fisher_executable" name=fisher-b \
        -each proto=uci tc=2+0.05 -rounds 500 -games 2 -concurrency 8 -strict \
        -pgnout file=games.pgn append=false \
        -log file=match.warn.log level=warn append=false
)
```

Run the replay in the same shell as the match with `chess==1.11.2`
installed. From a fresh shell, replace the argument with the path to the
saved `games.pgn`. Keep `match.warn.log` beside it.

```sh
python3 - "$fisher_match_dir/games.pgn" <<'PY'
import sys
from pathlib import Path

import chess.pgn

pgn_path = Path(sys.argv[1])
warning_log = pgn_path.with_name("match.warn.log")
if not warning_log.is_file():
    sys.exit(f"Missing fastchess warning log at {warning_log}")
warnings = warning_log.read_bytes()
if warnings:
    sys.stderr.buffer.write(warnings)
    sys.exit(1)

games = plies = longest = 0
with pgn_path.open() as pgn:
    while (game := chess.pgn.read_game(pgn)) is not None:
        game_number = games + 1
        if game.errors:
            sys.exit(f"Game {game_number}: PGN parser errors: {game.errors}")
        termination = game.headers.get("Termination")
        if termination is None:
            sys.exit(f"Game {game_number}: missing Termination tag")
        if termination != "normal":
            sys.exit(f"Game {game_number}: expected normal termination, got {termination!r}")
        position = game.board()
        length = 0
        for length, move in enumerate(game.mainline_moves(), start=1):
            if not position.is_legal(move):
                sys.exit(f"Game {game_number}, ply {length}: illegal move {move}")
            position.push(move)
        plies += length
        longest = max(longest, length)
        games += 1

expected_games = 1000
if games != expected_games:
    sys.exit(f"Expected {expected_games} games, replayed {games}")
print(f"{games} games, {plies:,} legal plies, longest game {longest} plies")
PY
```

The release binary SHA-256 was
`d2cbd85d402143519e0fe054b20d4e40daa851617f2f9a7be35a64e97b10ca80`.
The match PGN SHA-256 was
`6f8eb358f5005f7da6149a899852d1625d53a3b03dfb74dbd302ae816201ae46`.

The gate `scripts/check` passed. All 16 committed transcript paths matched
the inventory in [the transcript tests](../crates/fisher/tests/transcripts.rs). Every code TODO names a Beads ticket. The five
deferrals in the spec are `fisher-2aj`, `fisher-7vo`, `fisher-a2q`,
`fisher-2ha`, and `fisher-p25`.

## Review verification

The revised rules at `9bc18e8` matched all six approved deep perft counts in 35.55 seconds. The reference comparison checked every legal move lookup. On `feccd3b`, all 16 sessions passed, including seven rejected position commands and the held move after `go depth infinite`.

A separate 1,000-game match of the revised engine finished in 15.74 seconds with normal terminations and an empty warning log. Optimized Python replay verified 339,864 legal plies. The longest game lasted 684 plies. The release binary SHA-256 was `c6b8a3039b7610f1cd2764a6c19104b4e1604b600f9cf2c37f1c62f48aafa2b0` and the PGN SHA-256 was `ad07dbab88113f2a79d888b8113a1abf854360081d8816747756e41a1add337e`.

## Desktop acceptance

Fernando Bandeira watched Fisher play both White and Black in BanksiaGUI 0.58 with the release from `aaf0218`. The release rebuilt from `e16f1da` has the same SHA-256 as both the desktop executable and the historical match binary.
