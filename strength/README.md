# Playing strength

A record of how strongly each revision of fisher plays. Compare matches played under the same numbered conditions.

The node-limited series is a repeatable record and judges no claim of
strength. The clocked SPRT uses SPRT bounds fixed before play to settle
claims of strength. Both run separately from `scripts/check`. See
[Evidence](../docs/evidence.md).

## What is here

| Path | What it holds |
|---|---|
| `records.tsv` | One line per recorded series or match played by hand. |
| `measure` | Plays a clocked match by hand under conditions 2. |
| `summarise.py` | Counts results and queen-ahead games in a PGN. |
| `openings.epd` | The 200 positions for conditions 1 and 2. |
| `sprt-records.tsv` | One row per recorded clocked SPRT dispatch. |
| `series` | Plays and validates ranges, measures speed, and joins the games for a record. |
| `series-openings.epd` | The 2,000 positions for conditions 3 and 4, in draw order. |
| `ordering-sample.tsv` | 300 positions from real play and the finished depths used to judge move order under D64. |
| `ordering` | Draws the sample, compares two revisions, and calibrates the time threshold. |
| `ordering-threshold.tsv` | The time threshold and its runner, CPU model, date and run link. A header alone means calibration is pending. |

## Measuring move order (D64)

The committed sample judges whether a move order saves work. A series in
`records.tsv` records games. For D64, compare an ordering against the revision
immediately before it. Keep it if it searches fewer nodes on the sample and its
median time increase is at most the threshold. A time saving smaller than the
threshold counts as no gain in time.

Dispatch **Ordering sample** by hand from `main` on GitHub Actions with the
base and measured commits. The commits must be available in the repository. One
`ubuntu-latest` runner installs each pinned Rust channel in a fresh Rustup
directory before building both commits in release. The script warms up each
build on the full sample, then times six pairs of full-sample runs, switching
which build goes first in each pair. Each build goes first three times.
Each position starts a new UCI game, and the engine searches it with
`go depth N` using the depth in the sample. The script times the whole sample
using the runner's monotonic clock, excluding builds, warm-up, and UCI
process startup and shutdown. It reports each build's total cumulative nodes
at the specified depths, the six times and their median, and the measured
median's percentage change relative to the base. Node counts must repeat
exactly. The job log and summary include the results and the runner's CPU
model. The script also prints the CPU model associated with the recorded
threshold.

To repeat the node measurement locally, run
`strength/ordering measure <base-commit> <measured-commit>` from the checkout
that contains the sample. Use the hosted run's times for D64.

### Time threshold

The owner's agent settled the verdict rule on 2026-11-27. The threshold in
[`ordering-threshold.tsv`](ordering-threshold.tsv) always governs a D64 verdict.
A verdict requires the CPU model in that file. The script prints the CPU model
before timing. If it differs, discard the run for the verdict and dispatch again
based on the CPU model alone.

Dispatch **Ordering sample** with the measured revision blank to calibrate.
The workflow builds the base twice and runs five self-comparisons. Divide each
absolute median gap by the base median and round the largest up to a whole
percent. Calibration takes 70 full-sample runs, including 10 warm-ups, and
allows up to 300 minutes.

Set `calibrate_first` with a measured revision for a calibration check before
measurement on the same runner. The recorded threshold still governs. If the
paired calibration exceeds the recorded threshold, withhold the verdict and
bring the run to the owner.

### Hosted qualification

The owner's agent accepted `950e06e6aa78e4ceeaa51914fc71087d3efa4e82` on
2026-11-27 as the calibration base. It is the baseline immediately before
fisher-707.4. It has the same search and deepening as the previously named
`9772649`. FEN parsing, UCI command handling and the bench signature changed
between those revisions.

All three calibrations rounded to **1%**. The largest self-comparison gap
observed on AMD EPYC 7763 was **0.3931%** (0.39%). Both hosted base builds
reported **114,642,076 nodes**, matching the local result.

| Hosted run | CPU model | Result |
|---|---|---|
| [46035834751](https://github.com/fernandobandeira/fisher/actions/runs/46035834751) | AMD EPYC 9V74 | Initial calibration on main. Largest gap 0.3249%, rounded to 1%. |
| [46043088284](https://github.com/fernandobandeira/fisher/actions/runs/46043088284) | Intel Xeon 6975P-C | Fixed delay measured +2.1966% with equal nodes. The CPU differed from the initial calibration. |
| [46044717437](https://github.com/fernandobandeira/fisher/actions/runs/46044717437) | AMD EPYC 7763 | Largest calibration gap 0.3931%, rounded to 1%. The fixed delay of 2,843 µs per search was about 1.67% of the sample time. It measured +0.6521% with equal nodes and failed the control by staying within 1%. |
| [46056818707](https://github.com/fernandobandeira/fisher/actions/runs/46056818707) | AMD EPYC 7763 | Recorded calibration. Largest gap 0.1845%, rounded to 1%. The proportional delay raised the median from 51.249 s to 52.828 s (+3.0818%) with equal nodes. |
| [46043091865](https://github.com/fernandobandeira/fisher/actions/runs/46043091865) | AMD EPYC 7763 | Reversing legal move traversal below the root changed 114,642,076 nodes to 611,209,419. |

The fixed delay took a smaller share of the sample time on the slower CPU.
The difference between the nominal delay and the measured change was about
one percentage point, consistent with build-layout effects on timing. The
revised copy adds 2.5% of each search's elapsed time before reporting the best
move, so the delay scales with the work the runner did.

Lightweight tags keep the three control revisions available to rebuild.

| Tag | Commit |
|---|---|
| `ordering-qualification/fixed-delay` | `8816e9e48e78d8d6b129377eccd944f4c0dcf1ed` |
| `ordering-qualification/proportional-delay` | `e8eaffda235a0189b21543afc4ee740df026cf83` |
| `ordering-qualification/reversed-order` | `5ae5a5882fddf0ee824aee9857dad19ea05aa639` |

```sh
git fetch origin 'refs/tags/ordering-qualification/*:refs/tags/ordering-qualification/*'
```

Run `python3 strength/test-ordering.py` to check the timing comparison with
a planted first-run cost.

### Reproduce the draw

The sample header names the source, digest, seed, generator and depth
revision. Reuse the joined `games.jsonl` from the conditions-3 series. To
rejoin the original run, install the pinned Python dependencies as in
[Play a range by hand](#play-a-range-by-hand), then run:

```sh
gh run download 45876884742 -p 'series-*' -D /tmp/fisher-ordering-ranges
strength/series join -- d15ccc2 8b9c254 3 2026-11-26 success \
  /tmp/fisher-ordering-ranges /tmp/fisher-ordering-joined
strength/ordering draw /tmp/fisher-ordering-joined/games.jsonl 9772649 \
  > /tmp/fisher-ordering-sample.tsv
cmp strength/ordering-sample.tsv /tmp/fisher-ordering-sample.tsv
```

The join uses the played commit `d15ccc2` in the PGN tags, as described below.
Fetch that commit from GitHub if the checkout lacks it. Replay with CPython
3.9.6 and `chess==1.11.2` to reproduce the sample byte for byte. Each row
records its one-based game number in the joined JSONL and its ply after the
opening. The first row is from the first third, the next from the middle
third, the next from the last third, and the rotation repeats.

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
Actions while the first runs. GitHub starts them one at a time, usually in
queue-entry order, but does not guarantee that order. Up to 100 runs can
wait. When the queue is full, additional dispatches show as canceled in
the Actions run list.
[GitHub Actions concurrency](https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/control-workflow-concurrency)

Allow about **45 minutes for conditions 3** and **2 hours for conditions
4** after the series starts, plus any wait for runners. These estimates
schedule the measured playing-job durations in matrix order onto eight
runners and add the plan and record times. Conditions 3 took an estimated
44.0 and 41.5 minutes using runs
[45713248835](https://github.com/fernandobandeira/fisher/actions/runs/45713248835)
and [45698177264](https://github.com/fernandobandeira/fisher/actions/runs/45698177264).
Conditions 4 took an estimated 109.9 and 116.7 minutes using runs
[45713145494](https://github.com/fernandobandeira/fisher/actions/runs/45713145494)
and [45698173508](https://github.com/fernandobandeira/fisher/actions/runs/45698173508).

Before building, each playing job installs the two revisions' pinned Rust
channels with the minimal profile in a fresh Rustup directory on the hosted runner.
The release builds explicitly use those installations.

For a clocked match by hand, check out the revision to measure and run
`strength/measure stockfish /tmp/fisher-against-stockfish` with fastchess,
Stockfish and Python 3 with `chess==1.11.2`. Give a new directory for the
games. Record clocked matches played by hand under conditions 2.

## What a record says

Each line of `records.tsv` describes one match, from the measured revision's point of view.

The four alpha-beta records name the squash commit `5c5d6cf`. The conditions 4
series and both conditions 2 matches were played on `1a2d03d`, which has the
same tree. The conditions 3 series was played on `d15ccc2`, whose only engine
difference was a clock forecast factor that has no effect under a node limit.
Its record was joined locally from the artifacts of
[run 45876884742](https://github.com/fernandobandeira/fisher/actions/runs/45876884742)
after the played revision became unreachable to the record job.

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
| `elo_difference`, `elo_interval` | The Elo difference and its 95% interval. A positive difference means the measured revision scored above 50%. Conditions 1 and 2 use the values printed by fastchess. The series computes them from the opening pairs and can print `inf` or `nan` for a one-sided score. |
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

### 2

Both engines get 10 seconds plus 0.1 seconds a move. The other conditions are the same as in conditions 1.

The last recorded revision under conditions 1, `dbe9113`, is measured again under conditions 2 so both sets of records include it. It ignores its clock and may lose games on time. Those games count toward `abnormal_endings`.

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
variance. It converts the score and the two endpoints of the score interval
to Elo. The `elo_interval` field stores half the width of the Elo interval.
A mirror match scores exactly half on every pair, so its observed pair
variance is zero.
[fastchess paired Elo](https://github.com/Disservin/fastchess/blob/d7b47e8dec6b564aaf4cbb2922ab37e6c6761e75/app/src/matchmaking/elo/elo_pentanomial.cpp)

A node limit gives no credit for speed. The ratio compares the two builds
on each machine to catch large slowdowns. Treat values within about 1%
of 1 as noise. The first series had per-range ratios of 0.9813–1.0175 under
conditions 3 and 0.9557–1.0826 under conditions 4.
Process startup dominates the alpha-beta bench's wall time because it searches
94,178 nodes, down from 11,024,485 with plain minimax, which explains both ratios
near 0.19. With `go nodes 3000000`, both builds search about 7–11 million nodes
per second (Beads ticket `fisher-b5i` tracks the fix).

For the Stockfish records under conditions 1 and 2, scores
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

## Clocked SPRT

The gain test settles "this change plays stronger" with SPRT bounds of
0 and 10 Elo. The no-loss test settles "this change does not play weaker"
with SPRT bounds of -10 and 0 Elo. Both tests set a 5% error rate in each
direction.

Both engines get 10 seconds plus 0.1 seconds per move. The two revisions
seed the draw from the published opening positions. The draw leaves out
the positions of the node-limited series. Each opening is played once with
each color. The points from both games are one observation. See
[how the openings are drawn](../scripts/clocked-sprt.md#draw-the-openings).

The judge counts pairs in draw order. The first log-likelihood ratio at
or beyond either threshold decides the verdict. The upper threshold is
log(19) for a pass and the lower threshold is -log(19) for a fail. The
jobs finish their requested ranges and keep games beyond the deciding
pair. The result is inconclusive if neither threshold is reached in
10,000 pairs.

Dispatch **Clocked SPRT** from the repository's default branch. Give the
measured revision and the gain or no-loss test. Use a commit ID or
`origin/<branch>` for the measured revision and opponent. A bare branch
name does not resolve in the checkout. A commit on a throwaway branch must
stay reachable until all replays of its games are done. A blank opponent
uses the merge base with `main` (the first parent for a revision already
on `main`). Set the first opening to 1 for a new test. The default request
is 1,000 pairs.

Each playing job runs two games at a time over at most 50 pairs. Up to
eight playing jobs run at once here and eight run at once in a strength
series. The workflows have separate queues. They share the account's
concurrent hosted-job limit with other workflows. For standard runners
GitHub lists 20 concurrent jobs on Free and 40 on Pro.
[GitHub Actions job concurrency limits](https://docs.github.com/en/actions/reference/limits#job-concurrency-limits-for-github-hosted-runners)

The qualification played 256 pairs of `5c5d6cf` against `8b9c254` in
4 minutes 59 seconds of elapsed workflow time. The gain test counted 104
pairs. The last self-match dispatch played 376 pairs in 7 minutes 34
seconds and the gain test decided at cumulative pair 167. The last
self-match dispatch, the predecessor match and the 1,180- and 130-iteration
matches used about 7.3 to 7.7 runner-seconds per pair at two games at a
time. This includes building both engines. Treat it as a floor for later
revisions that use more of their clock. In these matches `5c5d6cf` spent
about 0.12 seconds per move from a starting allowance of about 0.43 seconds.
The hosted replays took 28 to 32 seconds each.
See the [predecessor run](https://github.com/fernandobandeira/fisher/actions/runs/46001457732),
the [self-match continuation](https://github.com/fernandobandeira/fisher/actions/runs/46000161044)
and the [qualification rows](sprt-records.tsv).

For a revision that spends its clock, estimate 20 to 30 runner-seconds
per pair, or about 17 to 25 minutes of play per 50-pair range. The playing
step has a 75-minute timeout and the job has a 90-minute timeout.

After **no verdict yet**, keep the same revisions and test, set the first
opening to the next unplayed one, and enter the last run's numeric Actions
ID in `previous_run`. The plan replays the cumulative prefix from opening
1 and refuses a continuation if it already has a verdict or is incomplete.
The first hosted continuation added openings 9 through 24 to the first
eight pairs and named both runs in its record. A local replay of the
joined games reproduced the judge output byte for byte.
[First dispatch](https://github.com/fernandobandeira/fisher/actions/runs/45999763856)
and [first continuation](https://github.com/fernandobandeira/fisher/actions/runs/45999957928).

To replay, give the Actions ID in `replay_run`, keep the original
revisions in their original order, set the first opening to 1, and
request the full cumulative range originally asked for, even if
some games are missing. Select the test to apply. Selecting
`opponent_view` scores the points taken by the original opponent. Both
inputs for earlier run IDs require a manual dispatch of this workflow
from the default branch, and use its first attempt. Retain the run
artifacts before they expire. To compute the result locally, pass the
cumulative `games.pgn` from the `sprt-record` artifact to
`scripts/clocked-sprt --pgn`. Use the original revision order, the test,
and the same cumulative range starting at 1. For an opponent reading,
add `--opponent-view`. See the
[local replay instructions](../scripts/clocked-sprt.md).

### Qualification on hosted runners

The self-match showed no detectable advantage for either side. Its 95%
intervals still allow a small advantage.

All slow copies add the same loop once per searched node. Local timing
divides the median wall time of five `go nodes 3000000` searches from the
starting position by the baseline median. Hosted timing sums the nodes and
milliseconds reported in the engine-attributed UCI `info` entries in each
fastchess log. Divide the baseline's nodes per millisecond by the copy's.
Fastchess logs these entries when a principal variation continues past a
threefold repetition. They sample those searches, and a search can
contribute more than one entry. There were 35 to 218 entries per job for
the two new measured copies, and 35 to 279 per engine per job across all
four runs. Grouping by engine, starting position and move history gives
22 to 129 distinct searches per engine per job. Using the median of the
last logged rate per search gave slowdown ratios within about 10% of
the ratios from summed nodes and time.

| Loop iterations | Revision | Local slowdown | Hosted slowdown overall (per job) | No-loss result |
|---:|---|---:|---:|---|
| 1,180 | `a30d6ba` | 16.26× | 5.09× (4.09–5.56×) | Fail at pair 104, -76.37 Elo ([run 46002462277](https://github.com/fernandobandeira/fisher/actions/runs/46002462277)) |
| 130 | `998f9a4` | 2.17× | 1.34× (1.23–1.41×) | Fail at pair 135 ([run 46003154219](https://github.com/fernandobandeira/fisher/actions/runs/46003154219)) |
| 6,400 | `bcd0b83` | 88.41× | 22.40× (17.27–26.57×) | Fail at pair 104 ([run 46013121250](https://github.com/fernandobandeira/fisher/actions/runs/46013121250)) |
| 380 | `057bf56` | 5.80× | 2.35× (2.02–3.75×) | Fail at pair 104 ([run 46013930062](https://github.com/fernandobandeira/fisher/actions/runs/46013930062)) |

The 1,180- and 130-iteration copies ran 5.09 and 1.34 times slower on the
hosted runners. The 6,400-iteration copy met D82's requirement of more
than fifteen times slower in every job and failed the no-loss test.

Each overall ratio pools the jobs in that run. It depends on the mix of
processors that played. The lowest ratios in the 6,400-, 380- and
1,180-iteration runs were 17.27×, 2.02× and 4.09× on the Xeon 8573C.
The 3.75× job was the only one on a Xeon 8370C. Across its sampled
searches, the 380-iteration copy ran at about half its usual node rate
while the baseline ran about 15% slower. The other 350 pairs gave
-39.88 Elo (95% interval [-49.76, -30.07]). All 400 gave -41.45 Elo.

<details>
<summary>Slowdown and processor by playing job</summary>

| Loop iterations | First opening | Processor | Slowdown |
|---:|---:|---|---:|
| 1,180 | 1 | EPYC 9V74 | 5.14× |
| 1,180 | 33 | EPYC 9V45 | 5.46× |
| 1,180 | 65 | EPYC 9V45 | 5.08× |
| 1,180 | 97 | EPYC 7763 | 4.97× |
| 1,180 | 129 | EPYC 7763 | 4.92× |
| 1,180 | 161 | EPYC 9V74 | 5.56× |
| 1,180 | 193 | EPYC 9V45 | 5.39× |
| 1,180 | 225 | Xeon 8573C | 4.09× |
| 130 | 1 | EPYC 9V45 | 1.37× |
| 130 | 51 | EPYC 9V74 | 1.38× |
| 130 | 101 | EPYC 9V74 | 1.31× |
| 130 | 151 | EPYC 9V74 | 1.30× |
| 130 | 201 | EPYC 9V74 | 1.39× |
| 130 | 251 | EPYC 7763 | 1.39× |
| 130 | 301 | EPYC 9V74 | 1.41× |
| 130 | 351 | Xeon 8573C | 1.23× |
| 6,400 | 1 | EPYC 9V74 | 23.08× |
| 6,400 | 33 | Xeon 8573C | 17.27× |
| 6,400 | 65 | EPYC 9V45 | 26.57× |
| 6,400 | 97 | EPYC 9V74 | 23.12× |
| 6,400 | 129 | EPYC 7763 | 22.81× |
| 6,400 | 161 | EPYC 9V74 | 20.75× |
| 6,400 | 193 | EPYC 9V74 | 22.54× |
| 6,400 | 225 | EPYC 9V74 | 22.44× |
| 380 | 1 | EPYC 9V45 | 2.37× |
| 380 | 51 | EPYC 9V45 | 2.35× |
| 380 | 101 | EPYC 9V74 | 2.35× |
| 380 | 151 | Xeon 8573C | 2.02× |
| 380 | 201 | EPYC 9V74 | 2.28× |
| 380 | 251 | Xeon 8370C | 3.75× |
| 380 | 301 | EPYC 7763 | 2.19× |
| 380 | 351 | EPYC 9V74 | 2.35× |

</details>

Each of the four copies scored exactly half under
a node limit and played the same moves in both games of all 25 opening pairs.

For measuring the cost of a slowdown, use all the pairs. Stopping at a
threshold can pull the estimate away from zero.

| Loop iterations | Elo difference at the deciding pair (95% interval) | Elo difference over all played pairs (95% interval) |
|---:|---|---|
| 1,180 | -76.37 [-96.53, -56.71] | -72.98 [-86.43, -59.75] over 256 pairs |
| 130 | -27.08 [-43.93, -10.35] | -23.92 [-33.45, -14.44] over 400 pairs |
| 6,400 | -114.18 [-138.95, -90.49] | -111.58 [-126.52, -97.02] over 256 pairs |
| 380 | -33.51 [-51.14, -16.05] | -41.45 [-50.64, -32.33] over 400 pairs |

Calibrate a planted slowdown on the machines that play, from the engines'
own node rates in the fastchess logs, and expect the machines to differ.

The lightweight tags `sprt-qualification/slow-1180`,
`sprt-qualification/slow-130`, `sprt-qualification/slow-6400` and
`sprt-qualification/slow-380` keep the four recorded commits reachable
after their branches are deleted.

The subjects of `sprt-qualification/slow-1180` and
`sprt-qualification/slow-130` give the local slowdowns. The hosted
runners measured 5.09× and 1.34×, respectively.

To fetch the commits in a checkout for local replay, run:

```sh
git fetch origin 'refs/tags/sprt-qualification/*:refs/tags/sprt-qualification/*'
```

### Record and artifacts

The record job replays the joined games with the judge and prints the row
for [`sprt-records.tsv`](sprt-records.tsv). Save every printed row in that
file in a pull request. A verdict stands. Play the same test again only
when a job failure voided it. The `sprt-record` artifact
has the row in `record.tsv`, the judge output, the plan, the current range
in `range.pgn`, and the cumulative games in `games.pgn`. The two initial
rows are the fisher-mwa.4 trials played from a throwaway branch before
this workflow merged.

When the record job finishes handling a judge result, it prints a row
for pass, fail, no verdict yet, inconclusive, or void. Missing or unreadable games give a void row.
An inconsistent GitHub artifact inventory or a failed transfer of a
listed artifact fails the job without a row.
An invalid request, judge refusal, or unforeseen error also fails the job.

If the record job failed or was cancelled before printing, dispatch a
replay of the original run. The replay can reconstruct the cumulative
games from the saved plan, the uploaded ranges, and any saved earlier
games. “Re-run failed jobs” looks for playing artifacts from its new
attempt. If it finds none it stops without a row and directs you to
dispatch a replay.

| Fields | Meaning |
|---|---|
| `date`, `run` | UTC date the run was planned and its Actions URL. |
| `measured`, `opponent`, `view` | Full commits in original playing order and the side scored by this reading. |
| `test`, `sprt_low_elo`, `sprt_high_elo`, `time_control` | Gain or no-loss, its SPRT bounds in Elo, and the clock for both engines. |
| `seed`, `draw` | The two full commits separated by a colon and the [SHA-256 method that orders positions](../scripts/clocked-sprt.md#draw-the-openings). |
| `first`, `pairs_requested` | First opening and length of the range requested in this dispatch. |
| `pairs_counted`, `pair_counts` | Pairs used by the judge and their counts in order of 0, 0.5, 1, 1.5, and 2 points. |
| `elo_difference_and_95_interval`, `llr` | Logistic Elo difference with its 95% interval, when estimable, and the log-likelihood ratio at the last pair counted. |
| `losses_on_time`, `crashes_or_stalls`, `illegal_moves` | Counts across all games the judge read, including games beyond the deciding pair. |
| `result`, `detail` | Pass, fail, no verdict yet, inconclusive at the pair cap, or void. A void row gives the judge's reason and leaves its observation and ending counts blank. A replay gives its source run in the detail. |
| `prior_runs` | Space-separated Actions IDs of the runs from which this run obtained its earlier games or its replay. Open those IDs in this repository's Actions runs. |
