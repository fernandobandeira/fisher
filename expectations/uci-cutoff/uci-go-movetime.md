# Controlled schedules for go movetime

Staged by fisher-4yn.1 on 2026-11-25 for a search that makes cutoffs: the
successor of [the active file](../uci-go-movetime/README.md). fisher-4yn.2 moves it
into place. [The staged sessions' README](../uci-cutoff/README.md#controlled-schedules)
lists every value this file changes and derives each new number by hand
from fisher-4yn. No value came from running fisher. Values that a search
with cutoffs reaches by an order of moves nobody chose are marked as
having no outside value.

Derived on 2026-11-25 with [python-chess 1.11.2](https://pypi.org/project/chess/1.11.2/).
The [April 2006 UCI specification](https://backscattering.de/chess/uci/),
fisher-8ie rules 1–9 and fisher-r6y D5, D9–D11, D20, D22–D25, D28,
D30–D31 and D41 supply the response order, margin, fallback, report
spelling and release bounds. The diagnostics follow the approved sessions
and the session at `d80f30d`. fisher-5sg decisions 2–4, 8 and 10 supply
the depth cap, first limit and cumulative node count.

## Derivation

Enumerate legal moves, count each played move, score the horizon by
material (pawn 100, knight and bishop 300, rook 500, queen 900), and take
the highest negated child score under fisher-2aj rules 1–7. At the starting
position all 20 moves score 0 at depths 1 and 2. The per-ply counts of 20,
400, 8,902 and 197,281 match [published perft](../perft.epd). The cumulative
totals across the first four finished depths are 20, 440, 9,762 and
216,365 without cutoffs. With cutoffs (fisher-4yn) they are 20 and 79
through depth 2: depth 2 enters 20 + 39 nodes, because the first root
move costs 1 + 20 and each of the other 19 costs 1 + 1. A search with
cutoffs plays a subset of the moves at each depth (Knuth and Moore,
"An analysis of alpha-beta pruning", *Artificial Intelligence* 6,
1975), so the totals at depths 3 and 4 are at most 9,762 and 216,365
and otherwise have no outside value. A limit of 19 nodes leaves depth 1 unfinished. Enumeration also
finds `h8h7` as the sole legal move of the replacement position in the transcript.

For interruption inside depth 2 use puzzle `001om` from
[mate-in-two.tsv](../mate-in-two.tsv), FEN
`5r1k/pp4pp/5p2/1BbQp1r1/7K/7P/1PP3P1/3R3R b - - 3 26`.
There are 30 root moves and 1,248 replies. Only `g5g2` scores −600 cp
at depth 1. The other moves score −700 cp. Without cutoffs depth 2 would finish at
30 + 30 + 1,248 = 1,308 cumulative nodes. With cutoffs it finishes at
no more than that, at a total with no outside value.

D22 gives a 40 ms deadline for `movetime 50`. Values of 0 and 10 ms
have zero time after the margin. For 11 ms the limit expires at elapsed
1 ms. The largest parser input is `18446744073709551615`, the decimal
value of `u64::MAX`.

## How to run the schedules

Use fresh searches and sessions. In private deepening tests, set the
clock origin at acceptance of `go` and hold the clock monotonic. Compare
every fallback with the root's `legal_moves().next()`, including the
puzzle fallback (`c5b4` in the current order). For a finished depth, check
the principal variation under D5 and compare the final best move with
its first move. Starting-position variations can tie.

In uci tests use the existing scripted search interface and compute the
move from `request.position.legal_moves().next()` for each search. Use
that move as the first move in the scripted principal variation and in
the best-move report. Extend the variation with legal replies up to its
reported depth. Compare the session's output with the scripted move.
Synchronize command processing and report delivery with signals. Observe silence at a boundary
using an acknowledged command, a report, or clean exit. All waits are
hang timeouts. The split recorded on fisher-5sg.8 on 2026-11-24 uses
private search controls and the existing session interface.

## Movetime alone

| Schedule | Private deepening observations | Scripted uci observations |
|---|---|---|
| Starting position, `go movetime 50`, first time reading at 40 ms before depth 1. | Zero nodes, no finished-depth report, then the first legal move. | Hold the best-move report. After `go`, require `readyok` for `isready`. Release the report and require one matching `bestmove` without `info`. |
| Puzzle, `go movetime 50`. Hold at 39 ms through depth 1, then advance to 40 ms for the first time reading inside depth 2. | Depth 1 reports cp −600, 30 nodes, time 39, nps 769 and variation `g5g2`. Observe the nodes entered before the reading inside depth 2. After the expired reading, no further node is entered and no depth-2 report is sent. Return `g5g2`. | Send the depth-1 report and hold the best-move report. Require `readyok` for `isready` after the depth-1 output and before releasing the best-move report. Require one matching `bestmove` with no depth-2 output. |
| Starting position, `go movetime 50`. Finish depths 1 and 2 at 2 and 5 ms, then read 40 ms at the first time reading in depth 3. | Report cp 0 at both depths, with 20 and 79 cumulative nodes and nps 10000 and 15800, respectively. No further node after the expired reading and no depth-3 report. Return the first move of the depth-2 variation. | Send two finished-depth reports in order, then hold the best-move report through `isready` and `readyok`. Release it and require one matching `bestmove` with no depth-3 output. |
| Starting position, `go movetime 0` and `go movetime 10` in separate sessions, elapsed 0. | For each limit, zero nodes, no finished-depth report and the first legal move. | For each command, release the best-move report without a depth report and require one matching `bestmove`. Subsequent idle `stop` followed by `isready` yields only `readyok`. |
| Starting position, `go movetime 11`, elapsed held at 0. | Enter at least one node, then interrupt at the next node boundary before depth 1 finishes. No finished-depth report and the first legal move. | Hold the search, send `stop`, observe its interrupt signal and release the best-move report. Require one matching `bestmove` with no `info`, then `readyok` for `isready`. |
| Starting position, `go movetime 18446744073709551615`. Hold at 39 ms through depth 1, then interrupt before entering a depth-2 node. | No overflow or premature interruption. Report cp 0, 20 nodes, time 39 and nps 512 at depth 1. Enter no depth-2 node and return the first move of the depth-1 variation. | Require the depth-1 output after `go`, send `stop` and observe its interrupt signal before releasing the best-move report. Require one matching `bestmove` and no depth-2 output. |
| Starting position, `go infinite movetime 50`, first time reading at 40 ms before depth 1. | Zero nodes, no finished-depth report and the first legal move in the final report. | Send the best-move report with no depth report. Require only `readyok` for `isready` before `stop`, then one matching `bestmove`. |

## Command ordering at the deadline

Use fresh sessions for each order. In the private deepening test the
puzzle starts at 39 ms and reports depth 1 as above. In the scripted uci
tests send a depth-1 report and control when its best-move report arrives.

| Commands and order | Private deepening observations | Scripted uci observations |
|---|---|---|
| `stop` first | Signal the interrupt before the first time reading inside depth 2 while elapsed stays at 39 ms. Enter no further node after observing the interrupt, report no depth 2, and return `g5g2`. | Observe the interrupt signal from `stop` before releasing the best-move report. Require one matching `bestmove`, then only `readyok` for `isready`. |
| Deadline first, then `stop` | Advance to 40 ms for the first time reading inside depth 2. Enter no further node, report no depth 2, and return `g5g2`. | Observe the matching `bestmove` before sending `stop`. Send `isready` and require only `readyok`. |
| `quit` or end of input first, in separate sessions | Use the 39 ms interruption from the stop-first case, with the same node, report and move observations. | Observe the interrupt signal before releasing the best-move report. Require one matching `bestmove` followed by clean exit with no further output. |
| Deadline first, then `quit` or end of input, in separate sessions | Use the 40 ms expiry from the deadline-first stop case, with the same node, report and move observations. | Observe the matching `bestmove` before sending `quit` or closing input. Require clean exit with no further output. |
| Replacement `position` and `go movetime 0` first | First use the 40 ms expiry from the deadline-first stop case. Separately check zero-time search of the puzzle for zero nodes, no depth report and its first legal move. | While the original best-move report is held, send the replacement FEN from the transcript and `go movetime 0`. Require a rejection for each, in order. Release the report and require one matching `bestmove` with no depth-2 output. Send `go movetime 0` again and compare its output with the scripted move computed from the original root. |
| Deadline first, then replacement `position` and `go movetime 0` | First use the 40 ms expiry from the deadline-first stop case. Separately check the replacement position with zero time for zero nodes, no depth report and its sole legal move. | After the first matching `bestmove`, send the replacement `position` and `go movetime 0`. Require a second `bestmove` from the replacement root without a diagnostic or another depth report. |
| Queue `stop`, replacement `position` and `go movetime 0` (D24) | Use the 39 ms interruption from the stop-first case and the zero-time check of the replacement position from the preceding case. | Queue the commands while holding the first best-move report. Observe the interrupt and release the report. Require its matching `bestmove` before the replacement search begins. That search reports its move from the replacement root and the session sends one matching `bestmove` without a diagnostic or extra depth report. |

## Several limits

Adopt after both fisher-5sg.6 and fisher-5sg.8 land. Use the starting
position and run both command orders. Each scripted uci case ends with
one best move and an idle `isready` yielding only `readyok`.

| Commands and schedule | Private deepening observations | Scripted uci observations |
|---|---|---|
| `go depth 1 movetime 50` / `go movetime 50 depth 1`, elapsed held at 2 ms. | Depth 1 reports cp 0, 20 nodes, time 2 and nps 10000. No depth-2 node. Return the first move of the reported variation. | After either `go`, send one depth report followed by the best-move report. Require the depth-1 output before the matching `bestmove`. |
| `go nodes 19 movetime 50` / `go movetime 50 nodes 19`, elapsed held at 39 ms. | Enter exactly 19 nodes, report no finished depth and return the first legal move. | After either `go`, release only the best-move report and require one matching `bestmove` without `info`. |
| `go nodes 19 movetime 50` / `go movetime 50 nodes 19`, first reading at 40 ms. | Enter zero nodes, report no finished depth and return the first legal move. | After either `go`, release only the best-move report and require one matching `bestmove` without `info`. |

## Release measurements

Outside `scripts/check`, run a real release process many times with a
position that has legal moves and `go movetime 50` as the only limit.
Measure from flushing the command to receipt of the complete
`bestmove`. Every run must arrive by 50 ms (D31), including polling,
scheduling and sending the response. Record the build, machine,
revision and measured times. Measure `stop` separately during a search
that would otherwise run for seconds, from flushing `stop` to receipt
of the complete `bestmove`, with D10's 250 ms bound per D30.
