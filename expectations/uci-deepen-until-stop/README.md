# Search that deepens until stop

Source and derivation date: 2026-11-24. Protocol order follows the April
2006 UCI specification, fisher-8ie and fisher-r6y D9–D11, D15, D20, D24,
D25 and D28. Scores, legal moves and node counts come from exhaustive
enumeration with shakmaty 0.30.3 and the reference minimax for fisher-2aj
rules 1–7. Starting-position counts also come from [published perft](../perft.epd).

## Controlled schedules

The ordinary transcripts describe what crosses standard input and output.
The schedules here specify when to interrupt or hold search in a controlled
test. Synchronize on the search boundary and on handling the command.
The production depth cap is 255 plies under fisher-5sg decision 2. These
checks supply a private cap of two under D11 and fisher-5sg.4.

| Schedule | Required observations |
|---|---|
| Use position `5r1k/pp4pp/5p2/1BbQp1r1/7K/7P/1PP3P1/3R3R b - - 3 26` (puzzle `001om` in [mate-in-two.tsv](../mate-in-two.tsv)). During depth 1, handle `stop` after node 14 has been scored and before node 15 is entered. | No `info` line. Exactly `bestmove c5b4`, the first legal move under D20. At node 14 the best move so far is `g5g2`. An idle `stop` followed by `isready` produces only `readyok`. |
| Use the same puzzle position. Let depth 1 finish. At depth 2, after 639 of its 1,278 nodes, handle `stop` before the next node is entered. Repeat with `quit` and end of input, in fresh sessions. | Exactly one searched `info` at depth 1, `cp -600`, 30 nodes, principal variation `g5g2`. Exactly `bestmove g5g2`. No report for depth 2. On `quit` or end of input, clean exit follows the move with no further output. |
| Run the same puzzle position with a private depth cap of 2, then handle `isready` after the cap is reached and send `stop` after `readyok`. Repeat with `quit` and end of input in fresh sessions after the same `readyok`. | Depth 1 as above. Depth 2 reports `cp -700`, 1,308 total nodes and principal variation `c5f2 g2g3`. No node from depth 3. Hold the move until `stop`, `quit` or end of input, then send exactly `bestmove c5f2`. On `quit` or end of input, clean exit follows the move with no further output (D25). |
| Use the starting position and a private depth cap of 2. Fix elapsed time at 2 ms when depth 1 finishes and 5 ms when depth 2 finishes, measured from the same accepted `go infinite`. Handle `isready` at the cap, then `stop`. | One report each at depths 1 and 2, both `cp 0`, with 20 and 440 total nodes, time 2 and 5, and nps 10,000 and 88,000. No node from depth 3 and no `bestmove` before `stop`. Return the legal first move of the depth-2 principal variation. |

Shakmaty enumeration scored every legal root move of puzzle `001om`
at both depths. At depth 1 only `g5g2` scores −600 cp and the next
score is −700 cp. At depth 2 only `c5f2` scores −700 cp and the next
score is −800 cp. Depth 1 enters 30 nodes. Depth 2 enters 1,278 nodes
including its first ply, so the depth-2 report totals 30 + 1,278.

On 2026-11-24 a standalone probe using the rules crate at `98324fa`
with rustc 1.99.1 enumerated the legal moves in fisher's order. Python-chess
1.11.2 independently matched the set of all 30 moves and scored the first
14 positions reached. At depth 1 each root move enters exactly one node.
The first 13, starting with `c5b4`, score −700 cp and node 14, after
`g5g2`, scores −600 cp. Full-width minimax with the same tool independently
found `c5f2` at depth 2 with `g2g3` as its sole best reply.

D10's 250 ms bound is measured by fisher-5sg.4 on a release process per
D30. The transcript grammar expresses the output order. The cap, time
source, node boundary and whether `quit` or end of input interrupted
an unfinished depth are observations of the controlled schedules.
The repeated record accepts the spelling of additional reports. The
controlled schedules check that finished depths are reported once in
order and that an interrupted depth gives no report.
