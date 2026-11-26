# Clock schedules for a forecast factor of 15

Staged by fisher-mhy on 2026-11-26, before the forecast factor changes, by
an author who will not change it. Sources: fisher-r6y D49 (start the next
depth only if the factor times the time the last finished depth took fits
in what is left), D50 (the time for the move and the margin), D11, D20,
D22, D31 and D52; the proposal in fisher-4yn.2 (PR #86) to change the
factor from 30 to 15; and the clock schedules for a search with cutoffs,
staged by fisher-4yn.1 as `uci-cutoff/uci-go-clocks.md` and moved by
fisher-4yn.2 to [`uci-go-clocks/README.md`](../uci-go-clocks/README.md).
Every value below was worked out by hand from D49 and D50 with a factor
of 15. No value came from running fisher.

No test reads this directory. The document
[`uci-go-clocks.md`](uci-go-clocks.md) here is a full copy of the schedules
for a search with cutoffs, with only the values listed here changed. It
replaces the document at the path `expectations/uci-go-clocks/README.md`
once fisher-4yn.2 has put the cutoff schedules there (see
[Activation](#activation)). Until then the factor stays 30 and the tests
keep its schedules, so nothing here can fail.

These values presume a search with cutoffs (fisher-4yn.2): the starting
position reaches a total of 79 nodes at depth 2 and 80 with one node of
depth 3.

## The rule with a factor of 15

D50 gives T, the time the search may use, from the clock of the side to
move. Let C be that clock, floored at zero; I its increment, or zero when
absent; and M the moves to go when positive, or 30 when absent or zero.
Divide C by M in whole milliseconds, rounding down, and add I. The time
for the move is the smaller of that sum and C. T is the time for the move
less the 10 ms margin, floored at zero.

Elapsed time is measured from the acceptance of `go`. When a depth
finishes, let d be its duration: the elapsed time when it finished less
the elapsed time when it started. Before the next depth starts, the
search reads the elapsed time again; call that reading s. The next depth
starts only if

    15 × d ≤ T − s

In the controlled schedules no time passes between the end of one depth
and that reading, so s equals the time at which the last depth finished.
The comparison is exact: equality starts the depth. Only the clock's T
enters it. A time given by `go movetime` does not follow D49 (D50), and
beside a clock it sets its own deadline (D52). The default of 30 moves
to go is a separate rule (D50) and does not change.

## Choosing the cases

A case that checks a boundary of D49 has to sit on that boundary for the
new factor. Each reworked case keeps the property its old version checked
and, where the property allows it, fails with a factor of 30 so the
implementer can see the new check fail before changing the constant. A
refusal cannot fail with a larger factor, because 30 × d refuses whenever
15 × d does. Those cases instead name the faulty rule they catch.

The new clocks divide by 30 exactly, so T is a whole number of
milliseconds:

| Clock | Time for the move | T |
|---|---:|---:|
| `wtime 780` | 780 ÷ 30 = 26 ms | 16 ms |
| `wtime 1050` | 1050 ÷ 30 = 35 ms | 25 ms |
| `wtime 1260` | 1260 ÷ 30 = 42 ms | 32 ms |

## Reworked cases

"Check N" is the Nth call of the test's node hook, made before node N is
entered. A schedule entry (N, t) sets the elapsed time to t at check N.
Each reading of the deadline before a node sees the elapsed time then.

### The starting-depth boundary: `wtime 1230` becomes `wtime 780`

Old: T = 31 ms; depth 1 finishes at 1 ms; 30 × 1 = 30 = 31 − 1, so depth 2
starts at equality. With 15, 15 × 1 = 15 < 30 is no longer a boundary, and
16 × d = 31 has no whole-millisecond answer. Keep depth 1 at 1 ms and
choose T so that 15 × 1 = T − 1: T = 16 ms, a time for the move of 26 ms,
and a clock of 26 × 30 = 780 ms.

- **Equality.** Schedule (20, 1 ms), (22, 16 ms). Depth 1 starts at 0 and
  finishes at 1 ms: d = 1, f = 1. 15 × 1 = 15 ≤ 16 − 1 = 15, so depth 2
  starts. Check 21 at 1 ms enters node 21. Check 22 reads 16 ms ≥ T and
  enters nothing. **21 nodes entered**, depth 1 reported at 1 ms, the
  depth-1 move. With a factor of 30: 30 > 15 refuses depth 2 and enters
  20 nodes, so the case fails. A rule that starts only when the forecast
  is strictly less than what is left also fails it.
- **One microsecond later.** Start at 1 µs. Schedule (20, 1 ms + 1 µs),
  (22, 16 ms). d = 1 ms, f = 1.001 ms. 15 > 16 − 1.001 = 14.999, so depth
  2 does not start. **20 nodes entered**, depth 1 reported at 1 ms + 1 µs.
  Under the factor of 15 the old clock fails this case: 15 ≤ 29.999 would
  start depth 2.

### The equality beside movetime: `wtime 1230 movetime 50` becomes `wtime 780 movetime 50`

T from the clock is 16 ms; movetime 50 gives 40 ms (D22/D31). Depth 1
finishes at 1 ms. 15 × 1 = 15 ≤ 16 − 1 = 15, so depth 2 starts at
equality. Schedule (20, 1 ms), (22, 15 ms), (23, 16 ms): check 21 enters at
1 ms, check 22 enters at 15 ms, check 23 reads 16 ms and the clock wins,
ahead of the 40 ms movetime deadline. **22 nodes entered**, depth 1 at
1 ms, the depth-1 move. With a factor of 30, 30 > 15 refuses depth 2 and
enters 20 nodes, so the case fails.

### The last depth, not the first: `wtime 2790` becomes `wtime 1050`

The case shows that D49 forecasts from the depth that just finished.
Keep depth 1 at 1 ms and depth 2 at 5 ms (d₁ = 1, d₂ = 4). With 15 the
conditions are:

- depth 2 starts: 15 × 1 ≤ T − 1, so T ≥ 16;
- depth 3 is refused: 15 × 4 = 60 > T − 5, so T ≤ 64;
- depth 1's duration would admit depth 3: 15 × 1 ≤ T − 5, so T ≥ 20;
- a factor of 30 refuses depth 2, so the case can fail: 30 > T − 1, so
  T ≤ 30.

T = 25 ms meets all four with room on each side: a time for the move of
35 ms and a clock of 1050 ms. Schedule (20, 1 ms), (79, 5 ms), (81, 25 ms).
15 ≤ 24 starts depth 2; check 79 at 5 ms enters its last node; 60 > 20
refuses depth 3. **79 nodes entered**, depths 1 and 2 at 1 and 5 ms, the
depth-2 move. If depth 3 were admitted, check 80 would enter node 80 and
check 81 would read 25 ms and stop, so the entry fails the check at once.
With a factor of 30, 30 > 24 refuses depth 2 and enters 20 nodes.

Old: T = 83 ms, 30 ≤ 82 and 120 > 78. Under the factor of 15 that clock
admits depth 3 (60 ≤ 78), and the case fails.

### The last depth, not the longest: `wtime 2160` becomes `wtime 1260`

The case shows that a shorter second depth admits the third. Keep depth 1
at 2 ms and depth 2 at 3 ms (d₁ = 2, d₂ = 1). With 15:

- depth 2 starts: 15 × 2 = 30 ≤ T − 2, so T ≥ 32;
- depth 3 starts: 15 × 1 ≤ T − 3, so T ≥ 18;
- the longest duration would refuse depth 3: 15 × 2 = 30 > T − 3, so
  T ≤ 32.

T = 32 ms is the only value, and it keeps the old case's equality at the
start of depth 2: a time for the move of 42 ms and a clock of 1260 ms.
Schedule (20, 2 ms), (79, 3 ms), (81, 32 ms). 30 ≤ 30 starts depth 2;
15 ≤ 29 starts depth 3; check 80 enters at 3 ms; check 81 reads 32 ms and
stops. **80 nodes entered**, depths 1 and 2 at 2 and 3 ms, the depth-2
move. The longest duration, or the first, forecasts 30 > 29 and enters 79.
With a factor of 30, 60 > 30 refuses depth 2 and enters 20.

### The forecast beside another limit: `wtime 1500` with `movetime 80`, `depth 2` or `nodes 79`, both orders

T is 40 ms. Old: depth 1 finishes at 2 ms; 30 × 2 = 60 > 38 refuses depth 2
before the other limit ends the search. With 15, 15 × 2 = 30 ≤ 38 would
start depth 2, so the case no longer shows the forecast. The smallest
whole-millisecond d with 15 × d > 40 − d is 3 (16 × d > 40). Finish depth 1
at 3 ms: 15 × 3 = 45 > 40 − 3 = 37 refuses depth 2. Schedule (20, 3 ms),
(22, 80 ms). **20 nodes entered**, depth 1 at 3 ms, the depth-1 move, for
each of the three limits in each order. A rule that drops the forecast
beside another limit starts depth 2, enters node 21, then reads 80 ms at
check 22 and stops, so it fails the case.

### The forecast uses the clock alone: `wtime 3300 movetime 50`

Clock T is 100 ms; the movetime deadline is 40 ms. Old: depth 1 at 2 ms;
30 × 2 = 60 ≤ 98 admits depth 2, while 60 > 38 against the movetime
deadline would have refused it. With 15, 30 ≤ 38 admits either way, so
the case no longer tells the two apart. Finish depth 1 at d = 4 ms, so
s = 4 ms and the clock leaves 96 ms:

- factor 15 admits depth 2: 15 × 4 = 60 ≤ 96;
- factor 30 refuses it: 30 × 4 = 120 > 96;
- forecasting against the 36 ms left before the movetime deadline would
  refuse it: 60 > 36.

(At 3 ms both factors admit depth 2, 45 ≤ 97 and 90 ≤ 97, so a return to
30 would pass; at 4 ms it fails.) Schedule (20, 4 ms), (22, 39 ms),
(23, 40 ms). Check 21 enters at 4 ms, check 22 enters at 39 ms, check 23
reads 40 ms and movetime wins, ahead of the clock's 100 ms. **22 nodes
entered**, depth 1 at 4 ms, the depth-1 move. With a factor of 30, or
with the forecast measured against movetime, depth 2 is refused and 20
nodes are entered.

### Movetime alone: `go movetime 50`

Kept as the check that D49 does not apply to a time given by movetime.
Old: depth 1 at 2 ms; movetime admits depth 2 although 30 × 2 = 60 > 38.
With 15, 30 ≤ 38, so a faulty rule that applied the forecast to movetime
would also admit depth 2 and pass. Finish depth 1 at 3 ms, as in the
cases beside a clock: 15 × 3 = 45 > 37, yet movetime admits depth 2.
Schedule (20, 3 ms), (22, 39 ms), (23, 40 ms). **22 nodes entered**, depth
1 at 3 ms, the depth-1 move.

## Rechecked and unchanged

Each still holds with a factor of 15; only the arithmetic in the text
changes.

| Case | Factor 30 | Factor 15 | Values |
|---|---|---|---|
| `wtime 3300`, one depth-3 node | 30 ≤ 99 starts depth 2; 90 ≤ 96 starts depth 3 | 15 ≤ 99; 45 ≤ 96 | Unchanged: (20, 1 ms), (79, 4 ms), (81, 100 ms); 80 entered |
| Puzzle `001om`, `go btime 1500` and `go infinite btime 1500` | 30 ≤ 39 starts depth 2 | 15 ≤ 39 | Unchanged: (30, 1 ms), (60, 40 ms); 59 entered; `g5g2` |
| `wtime 1500 depth 1`, depth wins | 30 ≤ 39 would permit depth 2 | 15 ≤ 39 would permit depth 2 | Unchanged: (20, 1 ms); 20 entered |
| `wtime 1500`, `wtime 1500 depth 1` and `wtime 1500 nodes 19`, ending inside depth 1 | No depth finishes, so no forecast | Same | Unchanged |
| Only the other color's clock; the default depth | No clock for the side to move, so no forecast | Same | Unchanged |

The time-for-the-move table keeps every row except the three clocks it
had only for these schedules: `wtime 1230` (41, 31), `wtime 2790` (93, 83)
and `wtime 2160` (72, 62) become `wtime 780` (26, 16), `wtime 1050`
(35, 25) and `wtime 1260` (42, 32).

The scripted `uci` cases are unchanged. A scripted search emits fixed
reports and does not apply D49, so its commands (`go wtime 1230`,
`go wtime 2790`, `go wtime 2160`, `go wtime 1230 movetime 50`) and its
reports at 2 ms stay as they are, as the scripted `nodes 440` case did in
fisher-4yn.1. `crates/uci` needs no change.

## Activation

The implementer of fisher-4yn.2 activates these values in PR #86, in the
commit that sets `DEPTH_GROWTH_FACTOR` to 15, after this document is on
main:

1. Before changing the constant, copy the Factor 15 column below into
   `crates/deepening/src/tests/clocks.rs`, run those tests with the factor
   of 30, and record that the five cases marked "fails at 30" do fail. The
   author of these values did so in throwaway copies of d15ccc2 and, for
   the `wtime 3300 movetime 50` case, of 765d594, with the factor left at
   30: all five entered 20 nodes and failed, and the other reworked cases
   passed. fisher was not run with
   a factor of 15.
2. Change the constant to 15 and run the tests again.
3. Move the document with a pure rename, changing no line in it:
   `git mv expectations/uci-forecast-15/uci-go-clocks.md expectations/uci-go-clocks/README.md`.
   The document it replaces is the one at that path in PR #86 (765d594),
   which holds the clock schedules for a search with cutoffs and the
   factor of 30. The rename touches `expectations/`, so the commit
   carries a `Guard-Change:` line. Its links are relative to
   `expectations/` one level down, so they hold at the new path.
   This README stays where it is as the record of the derivation.

Three test comments name the path `expectations/uci-go-clocks/README.md`
in PR #86: `crates/deepening/src/tests/clocks.rs` lines 115 and 276, and
`crates/uci/tests/threaded_session/clocks.rs` line 1. After the rename
the path holds the factor-15 document, so none of them needs a new path.
The comment at line 115 should add `expectations/uci-forecast-15/README.md`
as the derivation of the three new clocks. If the factor changes before
fisher-4yn.2 has moved the cutoff schedules into place, stop: these values
assume the search with cutoffs.

Line numbers are at 765d594, the head of PR #86, and match d15ccc2.

| Test (line at 765d594) | Factor 30 | Factor 15 | Fails at 30 |
|---|---|---|---|
| `time_for_the_move_uses_the_side_to_move_and_keeps_the_margin` (163, 185, 186) | `white(1230, …, 41, 31)`, `white(2790, …, 93, 83)`, `white(2160, …, 72, 62)` | `white(780, …, 26, 16)`, `white(1050, …, 35, 25)`, `white(1260, …, 42, 32)` | No; D50 only |
| `starts_next_depth_at_equality_but_stops_one_microsecond_later` (393–406) | `white_clock(1230)`; schedule (20, 1 ms + start), (22, 31 ms); 21 and 20 entered | `white_clock(780)`; schedule (20, 1 ms + start), (22, 16 ms); 21 and 20 entered | Yes, at equality: 20 entered |
| `forecasts_with_the_duration_of_the_last_depth` (448–457) | `white_clock(3300)`; (20, 1 ms), (79, 4 ms), (81, 100 ms); 80 entered | Unchanged | No |
| `longer_second_depth_refuses_depth_three` (459–468) | `white_clock(2790)`; (20, 1 ms), (79, 5 ms), (81, 83 ms); 79 entered; reports at 1 and 5 ms | `white_clock(1050)`; (20, 1 ms), (79, 5 ms), (81, 25 ms); 79 entered; reports at 1 and 5 ms | Yes: 20 entered |
| `shorter_second_depth_admits_depth_three` (470–479) | `white_clock(2160)`; (20, 2 ms), (79, 3 ms), (81, 62 ms); 80 entered; reports at 2 and 3 ms | `white_clock(1260)`; (20, 2 ms), (79, 3 ms), (81, 32 ms); 80 entered; reports at 2 and 3 ms | Yes: 20 entered |
| `clock_forecast_applies_beside_each_finite_limit` (481–508) | `wtime 1500` with `movetime 80`, `depth 2`, `nodes 79`; (20, 2 ms), (22, 80 ms); 20 entered; report at 2 ms | Same limits; (20, 3 ms), (22, 80 ms); 20 entered; report at 3 ms | No; a rule without the forecast enters 21 |
| `clock_and_movetime_can_each_end_the_search` (536–554) | `(3300, 2, 40)` and `(1230, 1, 31)` as (clock, depth 1 ends, deadline); 22 entered each | `(3300, 4, 40)` and `(780, 1, 16)`; 22 entered each | Yes, both: 20 entered |
| `movetime_alone_does_not_forecast_the_next_depth` (556–568) | (20, 2 ms), (22, 39 ms), (23, 40 ms); 22 entered; report at 2 ms | (20, 3 ms), (22, 39 ms), (23, 40 ms); 22 entered; report at 3 ms | No; a rule that forecasts movetime enters 20 |

`running_out_of_search_time_inside_depth_two_keeps_the_puzzle_move`,
`clock_and_depth_can_each_end_the_search`,
`clock_and_nodes_can_each_end_the_search`,
`running_out_of_search_time_inside_depth_one_keeps_the_fallback` and the
tests of a missing clock or the default depth keep their values.

Never replace a value here with one from running fisher. If the code with
a factor of 15 disagrees with a value, stop and say which.
