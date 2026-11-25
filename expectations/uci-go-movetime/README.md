# Controlled schedules for go movetime

Derived 2026-11-25. The [April 2006 UCI specification](https://backscattering.de/chess/uci/)
supplies the command and reply order. fisher-8ie rules 1–9 and fisher-r6y
D9–D11, D20, D22–D25, D28, D30–D31 and D41 supply fisher's session rules,
margin, fallback, report spelling and release bounds. The diagnostic
text comes from the approved sessions and the session at `d80f30d`.
fisher-5sg decisions 2–4, 8 and 10 supply the depth cap, first limit and
cumulative node count.

## Derivation

Enumerate legal moves with [python-chess 1.11.2](https://pypi.org/project/chess/1.11.2/).
Count each played move, score the horizon by material (pawn 100,
knight and bishop 300, rook 500, queen 900), and choose the highest
negated child score under fisher-2aj rules 1–7. At the starting position,
all 20 moves score 0 at depth 1. Per-ply counts of 20, 400, 8,902 and
197,281 match [published perft](../perft.epd), making cumulative totals
20, 440, 9,762 and 216,365 across the first four finished depths.
A limit of 19 nodes leaves one node of depth 1 unentered.
For ties and the fallback, the existing order at `d80f30d` scans rank 1
first, from a1 to h1. The first square with a legal move is b1 and
the knight offsets try a3 before c3, so the starting fallback is
`b1a3`. Python-chess confirms both moves are legal and that `h8h7` is
the sole legal move of the replacement FEN in the transcript.

For the later interruption use puzzle `001om` from
[mate-in-two.tsv](../mate-in-two.tsv), FEN
`5r1k/pp4pp/5p2/1BbQp1r1/7K/7P/1PP3P1/3R3R b - - 3 26`.
The first legal move in fisher's order is `c5b4`. The c5 bishop is the
first black piece with a legal move in rank order and its first
diagonal leads to b4. Python-chess enumerates 30 root moves and 1,248
replies. At depth 1 only `g5g2` scores −600 cp and the next score is
−700 cp. Depth 2 would finish at 30 + 30 + 1,248 = 1,308 cumulative
nodes, so the poll at 1,024 cumulative nodes is inside depth 2.
The 50 ms limit gives a deadline of 50 − 10 = 40 ms after acceptance.
Below the margin the elapsed budget is zero. The maximum parser input
is the decimal value of `u64::MAX`, 18446744073709551615 ms.

## Movetime alone

Run each row in a fresh session. Set the origin at acceptance of the
`go` and keep the clock monotonic. Hold and release work with signals at
the boundaries named below. Assert silence at those boundaries using an
acknowledged command or a report from the controlled search. All waits
are hang timeouts. Keep the clock and node observations in private
deepening tests. For session ordering use a search under the session's
existing callback interface, with reports and input ordered by signals.
The `fisher-5sg.8` comment from 2026-11-24 records this split.

| Schedule | Required observations |
|---|---|
| Starting position, `go movetime 50`. Let the worker start at elapsed 40 ms and let it read time before entering depth 1. | On the first time reading, exactly `bestmove b1a3`. Zero nodes entered and no `info`. |
| Puzzle, `go movetime 50`. Start the worker at elapsed 39 ms and hold time there through the depth-1 report. Hold the worker immediately before the time reading at 1,024 cumulative nodes. Process `isready`, then set elapsed time to 40 ms and release the worker for that read. | At 39 ms, `info depth 1 multipv 1 score cp -600 nodes 30 nps 769 time 39 pv g5g2` followed by `readyok`. No answer while held. The first reading at 40 ms ends the search with exactly `bestmove g5g2`, no depth-2 report and no further node entered after the expired reading. The rate is floor(30,000 / 39). |
| Starting position, `go movetime 0` and `go movetime 10` in fresh sessions. Hold elapsed time at zero. | Each sends exactly `bestmove b1a3` with zero nodes and no `info`. |
| Starting position, `go movetime 18446744073709551615`. Freeze at 39 ms, allow depth 1 to finish, then hold before depth 2 enters a node and handle `stop`. | Accept the value without overflow or premature interruption. One depth-1 report (cp 0, 20 nodes, pv b1a3, time 39, nps 512), then precisely `bestmove b1a3`. The rate is floor(20,000 / 39). |
| Starting position, `go infinite movetime 50`. Start the worker at 40 ms, observe it finish with zero nodes and handle `isready` before `stop`. | Only `readyok` before `stop`. Then exactly `bestmove b1a3`, with no `info`. |

## Command ordering at the deadline

Use the puzzle's later-interruption schedule above and hold at the same
poll, with depth 1 already reported. For every row verify that no depth-2
report or second answer for the original `go` appears. Fresh sessions
keep each ordering independent.

| Command | Process it first at 39 ms | Process the deadline first and observe `bestmove g5g2` |
|---|---|---|
| `stop` | Acknowledge the interrupt before releasing work, keeping the clock at 39 ms until the answer. Exactly `bestmove g5g2`. Advance to 40 ms, then `isready` gets only `readyok`. | Send `stop` then `isready`. Only `readyok` follows the move. |
| `quit` or end of input (separate sessions) | Acknowledge the interrupt and release the worker at 39 ms. Exactly `bestmove g5g2`, then clean exit with no further output. | Send `quit` or close input. Clean exit with no further output. |
| `position` and `go` | Send the replacement FEN from the transcript and `go movetime 0` before releasing the worker. Read one rejection each, in input order. Advance to 40 ms, release and read `bestmove g5g2`. Send `go movetime 0` to verify the old root still gives `bestmove c5b4`. | Send the replacement `position` and `go movetime 0`. Exactly `bestmove h8h7` follows, with no diagnostic or `info`. |

Also queue `stop`, the replacement `position` and `go movetime 0` in
that order while the search is held at 39 ms. After `stop` interrupts
the worker, require `bestmove g5g2` before handling the replacement
position, followed by `bestmove h8h7` from the new search. The two answers arrive without any diagnostic or additional report (D24).

## Several limits

Adopt these examples when both fisher-5sg.6 and fisher-5sg.8 have landed.
Use the starting position and elapsed time since the accepted command.
Every row names both command orders. No additional depth or second
answer is allowed.

| Commands | Schedule | Required observations |
|---|---|---|
| `go depth 1 movetime 50` / `go movetime 50 depth 1` | Freeze at 2 ms from the first time reading. | Finish depth 1 at 20 nodes with `info depth 1 multipv 1 score cp 0 nodes 20 nps 10000 time 2 pv b1a3`, then `bestmove b1a3`. No node from depth 2. |
| `go nodes 19 movetime 50` / `go movetime 50 nodes 19` | Freeze at 39 ms from the first time reading. | Enter 19 nodes, no `info`, exactly `bestmove b1a3`. |
| `go nodes 19 movetime 50` / `go movetime 50 nodes 19` | Accept at elapsed zero. Start work at 40 ms so the first reading precedes any node. | Enter zero nodes, no `info`, exactly `bestmove b1a3`. The next periodic poll would be past the node limit, so this case orders time first at the initial reading. |

## Release measurements

Outside `scripts/check`, run a real release process many times with a
position that has legal moves and `go movetime 50` as the only limit.
Measure from flushing the command to receipt of the complete
`bestmove`. Every run must arrive by 50 ms (D31), including the cost of
polling, scheduling and sending the reply. Record the build, machine,
revision and measured times. Measure `stop` separately during a search
that would otherwise run for seconds, from flushing `stop` to receipt
of the complete `bestmove`, with D10's 250 ms bound per D30.
