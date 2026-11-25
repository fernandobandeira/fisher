# Playing strength

A record of how strongly each revision of fisher plays. Compare matches played under the same numbered conditions.

This is a record, not a gate. `scripts/check` does not run it, and nothing here is an expected value. A claim that a change plays stronger is still settled by SPRT with bounds fixed beforehand, as [docs/evidence.md](../docs/evidence.md) says. The ticket for that is `fisher-op4`.

## What is here

| Path | What it holds |
|---|---|
| `records.tsv` | One line per match: the two revisions that played, and the result. |
| `measure` | Plays one match under the conditions below and prints its line for `records.tsv`. |
| `summarise.py` | Counts the results in a match's PGN. `measure` runs it. |
| `openings.epd` | The 200 positions for conditions 1 and 2. |
| `series` | Plays a range of openings under conditions 3 or 4 and prints its SHA-256. |
| `series-openings.epd` | The 2,000 positions for conditions 3 and 4, in draw order. |

## Taking a measurement

Check out the commit to measure. Then play it against Stockfish, against an earlier revision of fisher, or both:

```sh
strength/measure stockfish /tmp/fisher-against-stockfish
strength/measure dbe9113 /tmp/fisher-against-dbe9113
```

The second argument is a new directory for the games. `measure` plays 400 games, prints the result, and ends with one tab-separated line. Add that line to `records.tsv` and commit it.

`measure` builds each fisher from its commit, not from the working files, so the hashes in the line name exactly the engines that played. Uncommitted changes are not measured. It refuses to print a line if the match did not produce 400 games.

To see whether a change helped, measure the new revision against the last revision in `records.tsv`.

`measure` needs [fastchess](https://github.com/Disservin/fastchess), Python 3 with `pip install chess==1.11.2`, and Stockfish for a match against it. Set `PYTHON` to use another interpreter.

## What a record says

Each line of `records.tsv` describes one match, from the measured revision's point of view.

| Field | Meaning |
|---|---|
| `date` | The day the match was played. |
| `revision` | The commit of fisher that was measured. |
| `opponent` | The commit of fisher it played, or the name Stockfish gives itself over UCI and its `UCI_Elo`. |
| `conditions` | Which set of conditions below the match was played under. Only records with the same number can be compared. |
| `games` | Games played: 400. |
| `won`, `drawn`, `lost` | Results for the measured revision. |
| `score` | The share of the points it took, in percent. A draw is half a point. |
| `queen_ahead_won`, `queen_ahead_drawn`, `queen_ahead_lost` | Results of the games in which it was a queen ahead. |
| `abnormal_endings` | Games whose PGN `Termination` tag is not `normal`: an engine crashed, played an illegal move or lost on time. Either engine may be at fault; the PGN says which. |
| `elo_difference`, `elo_interval` | The Elo difference fastchess computes from the score, and the margin it prints after `+/-`. Positive means the measured revision scored more than half the points. fastchess prints `inf` or `nan` when the score is too one-sided to compute them. |
| `runner` | The version of fastchess that ran the match. |

A revision was a queen ahead in a game if its material minus the opponent's was at least 900 in two positions in a row, so that the lead outlasted a reply. Material is counted as fisher's evaluation counts it: pawn 100, knight 300, bishop 300, rook 500, queen 900. The queen-ahead games it drew or lost are the games it failed to finish. The count does not say why: it may have been unable to give checkmate, or it may have blundered later.

`summarise.py` replays the moves in the PGN with python-chess and stops if one is illegal. That checks the PGN, not the engines: when an engine plays an illegal move, fastchess ends the game, leaves the move out of the PGN, and the game is counted in `abnormal_endings`.

## Conditions

### 1

- 400 games: each of the 200 positions in `openings.epd` once with each color, in the order of the document.
- A game ends by checkmate, stalemate, threefold repetition, the fifty-move rule or insufficient material, or abnormally when an engine crashes, plays an illegal move or runs out of time. No game is adjudicated on its score, and no engine resigns.
- fisher gets a clock of 600 seconds plus 10 a move and no other limit. Revisions before `fisher-5sg.10` ignore the clock and search to their own fixed depth. The clock lets the runner end a game if fisher hangs.
- Stockfish runs with `UCI_LimitStrength` on and `UCI_Elo` 1320, the lowest it allows, on one thread with a 16 MB hash and a clock of 10 seconds plus 0.1 a move.
- Eight games run at a time. Stockfish is on a clock, so a busy machine weakens it; run nothing else heavy during a match against it.

The conditions do not fix the Stockfish version or the machine. Compare two records against Stockfish only if `opponent` reads the same in both, and treat records from different machines with care.

### 2

Both engines get 10 seconds plus 0.1 seconds a move. The other conditions are the same as in conditions 1.

The last recorded revision under conditions 1, `dbe9113`, is measured again under conditions 2 to connect the series. It ignores its clock and may lose games on time. Those games count toward `abnormal_endings`.

### 3 and 4

| Conditions | Nodes for each move | Openings | Games |
|---|---:|---:|---:|
| 3 | 1,000,000 | 2,000 | 4,000 |
| 4 | 10,000,000 | The first 500 of the same list | 1,000 |

Each opening is played once with each color between two revisions of fisher.
Every `go` carries only `nodes N`. The games have no time control, score or
move-count adjudication, tablebase adjudication, or resignation.

### Play a range by hand

Put fastchess v1.8.3-alpha (`d7b47e8`) on `PATH`, then install the pinned
Python dependency and run the series from a checkout with both revisions
and their history. Both revisions must include `4fd3002`, where fisher
began to obey `go nodes`.

```sh
python3 -m venv /tmp/fisher-series-venv
. /tmp/fisher-series-venv/bin/activate
python3 -m pip install --require-hashes -r strength/series-requirements.txt
strength/series a5dac71 4fd3002 3 1 100 /tmp/fisher-series-3 --concurrency 4
```

Give the measured revision, opponent revision, conditions, first opening
(starting at 1), opening count, and a new directory. The script builds each
engine from its commit. Set concurrency to the number of games to play at
once, up to the machine's core count. The default is one.

The directory holds the PGN, fastchess log and ordered game data. The digest
is the SHA-256 of `games.jsonl`. Each line holds the opening number and FEN,
White's full commit, the UCI moves, the result and the termination. For each
opening, the measured revision's game as White comes first. To validate an
existing PGN and print its digest, pass `--pgn path/to/games.pgn` with the same
revisions and range and a fresh output directory. An incomplete range or
abnormal ending produces no digest.

The manual Strength series workflow plays the first 100 openings under
conditions 3 or the first 25 under conditions 4 on each of two runners. It
checks their digests and retains the games.

## How far to trust the numbers

- **Between two revisions of fisher that search**, a match under conditions 1 repeats game for game because each revision searches to a fixed depth and plays the same move from the same position. Under conditions 2 the match no longer repeats game for game. The depths the engines finish and the resulting scores depend on the machine and its load. The score describes these 200 openings. Revisions before `71a4608` choose legal moves at random.
- **Against Stockfish** the result varies from match to match, because Stockfish at limited strength chooses weaker moves on purpose and not always the same ones. The margin fastchess prints gives a rough idea of how much; it is not a promise about the next match.
- **An Elo for fisher** is not something this record establishes. Adding the difference against Stockfish to 1320 gives a rough placement only: it leans on one opponent, and for a fisher far below 1320 it is an extrapolation. Stockfish's documentation says `UCI_Elo` was calibrated at 120 seconds plus 1 a move and anchored to the CCRL 40/4 list; here it gets 10 seconds plus 0.1.
- **A score near 0% or 100%** says little about the size of the gap. Measure against a closer opponent instead.

## Where the openings come from

`openings.epd` holds 200 positions from `noob_3moves.epd` in [official-stockfish/books](https://github.com/official-stockfish/books) at commit `37751b0`, which that repository publishes under CC0. Each is a position after three moves by each color. fastchess drew them at random with `-srand 20261123` for the first match, before any result was known, and they are kept in the order it drew them.

The source for `series-openings.epd` is
[`noob_3moves.epd.zip` at `37751b08db745aa02babfa74056dd1168b9da803`](https://github.com/official-stockfish/books/blob/37751b08db745aa02babfa74056dd1168b9da803/noob_3moves.epd.zip),
published under CC0. The archive SHA-256 is
`68fe2691e0876d0de54fffb68c797e752138d5ed363beab2ac6ca34be68ee9f3`.
Verify it before extracting `noob_3moves.epd`, whose SHA-256 is
`2011193b4854e9a8cfdc05312ca2dbaffa6ceae3abbdee20e2ead2a18a603347`.
The source has 150,932 distinct lines, each ending in a line feed.

For each line, remove its line feed and compute the lowercase hexadecimal
SHA-256 of the UTF-8 text `20261125:` followed by that line. Sort the lines
by that hash in ascending order. Write the first 2,000, with one line feed
after each, in that order. The seed `20261125` and draw were fixed before
any games were played.

| Openings | SHA-256 |
|---|---|
| First 100 | `e3b9fbc279c52b03e8c2f6ac6a6687f1d1d3fd63a0250ed362b08b5535ee0c42` |
| First 500 | `4c35b0ae9ae0783d1e344e896666a28a8006f84cfb8947ce4c22145019130130` |
| All 2,000 | `df2c41c8b88e220050d5de4462b4838f6eaff1a809f4d6f159860624fee7555a` |
