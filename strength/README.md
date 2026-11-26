# Playing strength

A record of how strongly each revision of fisher plays. Compare matches played under the same numbered conditions.

This is a record, not a gate. `scripts/check` does not run it, and nothing here is an expected value. A claim that a change plays stronger is still settled by SPRT with bounds fixed beforehand, as [docs/evidence.md](../docs/evidence.md) says.

## What is here

| Path | What it holds |
|---|---|
| `records.tsv` | One line per recorded series or historical match. |
| `measure` | Plays a match by hand using the historical conditions. |
| `summarise.py` | Counts results and queen-ahead games in a PGN. |
| `openings.epd` | The 200 positions for conditions 1 and 2. |
| `series` | Plays and validates ranges, measures speed, and joins the games for a record. |
| `series-openings.epd` | The 2,000 positions for conditions 3 and 4, in draw order. |

## Taking a measurement

Dispatch the **Strength series** workflow by hand when cutting a release
or when a revision's strength claim warrants a record. Skip it as proof on
everyday pull requests. Give the measured revision, conditions 3 or 4, and
an opponent revision. Leaving the opponent blank uses the measured revision
from the last record under those conditions.

Each of `range-01` through `range-20` plays a contiguous range in opening
order, with four games at a time. The ranges have 100 openings under
conditions 3 and 25 under conditions 4. `repeat-01` plays the first range
on a separate hosted runner. The last job checks the expected games and
compares the repeat. It prints a tab-separated record and saves the
record and joined games in the `strength-record` artifact. Add the
printed line to `records.tsv` in a pull request.

Artifacts expire after 90 days. A new dispatch reproduces the games. “Re-run failed jobs” joins ranges from different attempts and keeps the first attempt's date.

A series uses up to eight hosted runners at once. The `plan` and `record`
jobs run on either side of the playing jobs. All branches using this
workflow share one queue. A second and third dispatch remain pending in
Actions while the first runs, and GitHub starts them one at a time in the
order they entered the queue. Up to 100 runs can wait. Once the queue is
full, additional dispatches show as canceled in the Actions run list.
[GitHub Actions concurrency](https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/control-workflow-concurrency)

Allow about **40 minutes for conditions 3** and **1 hour 50 minutes for
conditions 4** after the series starts, plus any wait for runners. These
estimates schedule the measured playing-job durations onto eight runners
and add the plan and record times. Conditions 3 used runs
[45713248835](https://github.com/fernandobandeira/fisher/actions/runs/45713248835)
and [45698177264](https://github.com/fernandobandeira/fisher/actions/runs/45698177264).
Conditions 4 used runs
[45713145494](https://github.com/fernandobandeira/fisher/actions/runs/45713145494)
and [45698173508](https://github.com/fernandobandeira/fisher/actions/runs/45698173508).
Conditions 1 and 2 are closed.

Before building, each playing job installs the two revisions' pinned Rust
channels with the minimal profile in a fresh Rustup directory on the hosted runner.
The release builds explicitly use those installations.

For a clocked match by hand, check out the revision to measure and run
`strength/measure stockfish /tmp/fisher-against-stockfish` with fastchess,
Stockfish and Python 3 with `chess==1.11.2`. Give a new directory for the
games. Conditions 1 and 2 are closed and their historical records remain
in the table.

## What a record says

Each line of `records.tsv` describes one match, from the measured revision's point of view.

| Field | Meaning |
|---|---|
| `date` | The day the match was played. |
| `revision` | The commit of fisher that was measured. |
| `opponent` | The commit of fisher it played, or the name Stockfish gives itself over UCI and its `UCI_Elo`. |
| `conditions` | Which set of conditions below the match was played under. Only records with the same number can be compared. |
| `games` | Games in the record, excluding the repeat. |
| `won`, `drawn`, `lost` | Results for the measured revision. |
| `score` | The share of the points it took, in percent. A draw is half a point. |
| `queen_ahead_won`, `queen_ahead_drawn`, `queen_ahead_lost` | Results of the games in which it was a queen ahead. |
| `abnormal_endings` | Games whose PGN `Termination` tag is not `normal`: an engine crashed, played an illegal move or lost on time. Either engine may be at fault; the PGN says which. |
| `elo_difference`, `elo_interval` | The Elo difference and its 95% interval. A positive difference means the measured revision scored above 50%. Historical records use the values printed by fastchess. The series computes them from the opening pairs and can print `inf` or `nan` for a one-sided score. |
| `runner` | The version of fastchess that ran the games. |
| `speed_ratio` | The median over the 20 playing ranges of measured to opponent bench speed on the same machine. Empty for conditions 1 and 2. |

A revision was a queen ahead in a game if its material minus the opponent's was at least 900 in two positions in a row, so that the lead outlasted a reply. Material is counted as fisher's evaluation counts it: pawn 100, knight 300, bishop 300, rook 500, queen 900. The queen-ahead games it drew or lost are the games it failed to finish. The count does not say why: it may have been unable to give checkmate, or it may have blundered later.

`summarise.py` replays the moves in the PGN with python-chess and stops if one is illegal. That checks the PGN, not the engines: when an engine plays an illegal move, fastchess ends the game, leaves the move out of the PGN, and the game is counted in `abnormal_endings`.

## Conditions

### 1 (closed)

- 400 games: each of the 200 positions in `openings.epd` once with each color, in the order of the document.
- A game ends by checkmate, stalemate, threefold repetition, the fifty-move rule or insufficient material, or abnormally when an engine crashes, plays an illegal move or runs out of time. No game is adjudicated on its score, and no engine resigns.
- fisher gets a clock of 600 seconds plus 10 a move and no other limit. Revisions before `fisher-5sg.10` ignore the clock and search to their own fixed depth. The clock lets fastchess end a game if fisher hangs.
- Stockfish runs with `UCI_LimitStrength` on and `UCI_Elo` 1320, the lowest it allows, on one thread with a 16 MB hash and a clock of 10 seconds plus 0.1 a move.
- Eight games run at a time. Stockfish is on a clock, so a busy machine weakens it; run nothing else heavy during a match against it.

The conditions do not fix the Stockfish version or the machine. Compare two records against Stockfish only if `opponent` reads the same in both, and treat records from different machines with care.

### 2 (closed)

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
Python dependencies and run the series from a checkout with two distinct
revisions and their history. Both revisions must include `4fd3002`, where fisher
began to obey `go nodes`.

```sh
python3 -m venv /tmp/fisher-series-venv
. /tmp/fisher-series-venv/bin/activate
python3 -m pip install --only-binary :all: --require-hashes -r strength/series-build-requirements.txt
python3 -m pip install --no-build-isolation --no-deps --require-hashes -r strength/series-requirements.txt
strength/series --concurrency 4 -- a5dac71 4fd3002 3 1 100 /tmp/fisher-series-3
```

Give the measured revision, opponent revision, conditions, first opening
(starting at 1), opening count, and a new directory. The script builds each
engine from its commit. Set concurrency to the number of games to play at
once, up to the machine's core count. The default is one.

The directory holds the PGN, fastchess log and ordered game data. To compute
the digest from the PGN, encode each game as a compact JSON array containing,
in order, the opening's one-based line number in `series-openings.epd`, the opening FEN, White's
full commit, an array of UCI move strings, the result, and the termination.
Use ASCII escaping for JSON strings, no spaces around separators, and one
line feed (byte `0x0a`) after each array, including the last. Write the games
in ascending opening number, with the measured revision's game as White first
for each opening. The SHA-256 covers these concatenated bytes, which the
script writes to `games.jsonl`.

To validate an existing PGN and print its digest, pass `--pgn path/to/games.pgn`
before `--` with the same revisions and range and a fresh output directory.
An incomplete range, abnormal ending, or result unsupported by the final
position produces no digest.

Pass `--bench` to time both builds before playing. The workflow always does
this. It runs `fisher bench` on each build once to warm up, then measures
five samples per build, alternating which build goes first in each pair.
Fishtest uses engine-reported milliseconds. Fisher’s bench reports only
nodes, so each sample divides the node count by wall time. The job
divides the measured median nodes per second by the opponent median.
The final record uses the median of the 20 range ratios. The repeated
range does not contribute. [Fishtest worker bench](https://github.com/official-stockfish/fishtest/blob/ea59d0c793613882480198bf5402401054046ad7/worker/games.py#L521-L538)

## How far to trust the numbers

Under a node limit, repeating the series gives the same games. The Elo
interval estimates how far the result might move with other openings.
Each opening contributes the mean score of its two games. The series
uses the mean of these pair scores and the 95% normal interval from their
variance, then converts the score and both bounds to Elo. The `elo_interval` field stores
half the width between the converted bounds. A mirror match scores
exactly half on every pair, so its observed pair variance is zero.
[fastchess paired Elo](https://github.com/Disservin/fastchess/blob/d7b47e8dec6b564aaf4cbb2922ab37e6c6761e75/app/src/matchmaking/elo/elo_pentanomial.cpp)

A node limit gives no credit for speed. The ratio compares the two builds
on each machine to catch large slowdowns. Treat values within about 1%
of 1 as noise. The first series had per-range ratios of 0.9813–1.0175 under
conditions 3 and 0.9557–1.0826 under conditions 4.

For the historical Stockfish records under conditions 1 and 2, scores
vary from match to match. These matches do not establish an Elo rating
for fisher. A score near 0% or 100% says little about how large the
strength difference is.

## Where the openings come from

`openings.epd` holds 200 positions from `noob_3moves.epd` in [official-stockfish/books](https://github.com/official-stockfish/books) at commit `37751b0`, which that repository publishes under CC0. Each is a position after three moves by each color. fastchess drew them at random with `-srand 20261123` for the first match, before any result was known, and they are kept in the order it drew them.

The source for `series-openings.epd` is
[`noob_3moves.epd.zip` at `37751b08db745aa02babfa74056dd1168b9da803`](https://github.com/official-stockfish/books/blob/37751b08db745aa02babfa74056dd1168b9da803/noob_3moves.epd.zip)
([raw download](https://raw.githubusercontent.com/official-stockfish/books/37751b08db745aa02babfa74056dd1168b9da803/noob_3moves.epd.zip)),
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
