# Searches on the clock

Staged by fisher-mhy on 2026-11-26 for a forecast factor of 15: the
successor of [the schedules for a search with cutoffs](../uci-cutoff/uci-go-clocks.md),
which is itself the successor of [the active file](../uci-go-clocks/README.md).
It applies once the search makes cutoffs (fisher-4yn.2) and D49 multiplies
the time of the last finished depth by 15 instead of 30. [This directory's
README](README.md) lists every value this file changes and derives each new
number by hand from D49 and D50. No value came from running fisher.

The note that follows is the one fisher-4yn.1 wrote for the file this one
replaces. It still holds for the node totals.

Staged by fisher-4yn.1 on 2026-11-25 for a search that makes cutoffs: the
successor of [the active file](../uci-go-clocks/README.md). fisher-4yn.2 moves it
into place. [The staged sessions' README](../uci-cutoff/README.md#controlled-schedules)
lists every value this file changes and derives each new number by hand
from fisher-4yn. No value came from running fisher. Values that a search
with cutoffs reaches by an order of moves nobody chose are marked as
having no outside value.

Derived on 2026-11-25 from the owner's answers to the clocks interview on
fisher-5sg.10 (D17, recorded as fisher-r6y D49–D56), the [April 2006 UCI
specification](https://backscattering.de/chess/uci/#gui-go),
[python-chess 1.11.2](https://pypi.org/project/chess/1.11.2/), and
[published perft](../perft.epd).

## Adoption in fisher-5sg.10

The real-process transcript tests select these sessions from
`expectations/uci/`.

| Session | Coverage |
|---|---|
| [go-white-clock-exhausted.uci](../uci/go-white-clock-exhausted.uci) | White to move with a clock at zero and below zero, with or without Black's healthy clock. |
| [go-black-clock-exhausted.uci](../uci/go-black-clock-exhausted.uci) | Black to move with a clock at zero and below zero, with or without White's healthy clock. |
| [go-healthy-clock.uci](../uci/go-healthy-clock.uci) | White with an increment and Black with one move to go. |
| [go-clock-terminal.uci](../uci/go-clock-terminal.uci) | A terminal position with an exhausted clock. |

The successor [go-parameters.uci](../uci/go-parameters.uci) replaces the
previous session in full. The clocks change
only the responses to `go wtime 60000`. The other four clock commands
(`go btime 60000`, `go winc 1000`, `go binc 1000`, `go movestogo 40`)
retain exactly depths 1–4 by D51 and D28. The successor also includes
`go depth 1` and `go nodes 20` from the epic's go-parameters table
(fisher-5sg.5 and fisher-5sg.6), and `go movetime 50` (fisher-5sg.7
and fisher-5sg.8). The dedicated depth, nodes and movetime sessions are
also active in `expectations/uci/`. The `mate`, `searchmoves`, `ponder`, and four bare
searches retain their responses from the active session.

For a healthy clock, the `<*` record accepts zero or more consecutive
matching searched `info` responses before one legal `bestmove`. The
session sends no `stop`. The controlled schedules verify that the
reported depths start at 1, increase by one for each finished depth,
and precede the best-move response (D49, D56, UCI go and bestmove).
An exhausted clock at a root with legal moves requires an immediate
`bestmove` with no `info` (D20, D51). Clean exit detects any extra output
after the final search.

D56 refers to a terminal zero-time case in fisher-5sg.7. The active
[movetime](../uci/go-movetime.uci) and [nodes and movetime](../uci/go-nodes-movetime.uci)
sessions have no terminal case.
Use the zero-limit precedence from [fisher-5sg.5](../uci-depth-and-nodes/README.md#derivation):
send the terminal score before `bestmove 0000`. D41 fixes the terminal
score, D42 fixes the null move, and Stockfish sf_19 at edb0d9d checks
the empty root before iterative deepening, emitting its terminal response
and best move. [Stockfish search](https://github.com/official-stockfish/Stockfish/blob/edb0d9d/src/search.cpp#L207-L217)
and [terminal response](https://github.com/official-stockfish/Stockfish/blob/edb0d9d/src/uci.cpp#L650-L652)
were read on 2026-11-25.

## Independent search values

Enumerate legal moves with python-chess 1.11.2. For each depth, count
each legal move played as one node and exclude the root. At the horizon
score material (100 for a pawn, 300 for a knight or bishop, 500 for a
rook, 900 for a queen and 0 for a king). Before the horizon score
checkmate by distance and stalemate as zero. At each position take the
maximum of the negated child scores (fisher-2aj rules 1–7). Repeat
from the root for each depth and add each search's count to the total
from the preceding depth (fisher-5sg decision 10).

The first table below gives a search with cutoffs, and the second the
search without cutoffs. A search with cutoffs plays a subset of the
moves counted in the second table at each depth (Knuth and Moore, "An
analysis of alpha-beta pruning", *Artificial Intelligence* 6, 1975), so
its totals are at most those. Depth 1 keeps its count, because nothing bounds the
root from above, and so does depth 2 in the two forced positions, whose
root has one legal move. From the starting position depth 2 enters
20 + 39 = 59 nodes (fisher-4yn): the first root move costs 1 + 20, and
each of the other 19 costs 1 + 1, because no capture is possible within
two plies. Every other total has no outside value with cutoffs.

| Root in the sessions | Nodes in each search with cutoffs | Totals at depths 1–4 with cutoffs |
|---|---|---|
| Starting position | 20, 59, then no outside value | 20, 79, then no outside value (at most 9,762 and 216,365) |
| Sole legal move `h8g8` | 1, 22, then no outside value; every search plays 1 node at ply 1 and 21 at ply 2 | 1, 23, then no outside value (at most 75 and 857) |
| Sole legal move `h8h7` | 1, 26, then no outside value; every search plays 1 node at ply 1 and 25 at ply 2 | 1, 27, then no outside value (at most 98 and 1,397) |

| Root in the sessions | Nodes by ply | Nodes in each search | Totals at depths 1–4 | Scores at depths 1–4 |
|---|---|---|---|---|
| Starting position | 20, 400, 8,902, 197,281 | 20, 420, 9,322, 206,603 | 20, 440, 9,762, 216,365 | cp 0 at each |
| Sole legal move `h8g8` | 1, 21, 30, 730 | 1, 22, 52, 782 | 1, 23, 75, 857 | cp −900, cp −900, mate −1, mate −1 |
| Sole legal move `h8h7` | 1, 25, 45, 1,228 | 1, 26, 71, 1,299 | 1, 27, 98, 1,397 | cp −900, cp −900, mate −1, mate −1 |

The starting-position counts match published perft at every ply shown.
In the two forced positions, White checkmates on ply 2 with `h6g7` or
`g5g7` respectively. At depth 2 the checkmate is on the horizon and still
scores by material. The terminal FEN in the transcript is checkmate
with no legal move.

Use puzzle `001om` from [mate-in-two.tsv](../mate-in-two.tsv) in the
later-depth interruption schedules. Its FEN is
`5r1k/pp4pp/5p2/1BbQp1r1/7K/7P/1PP3P1/3R3R b - - 3 26`.
Independent enumeration gives 30 root moves and 1,248 replies.
The sole best move at depth 1 is `g5g2`, scoring cp −600.
Without cutoffs depth 2 would finish at 30 + 30 + 1,248 = 1,308
cumulative nodes with `c5f2`, scoring cp −700. With cutoffs it finishes
at no more than that, at a total with no outside value. It still plays
all 30 root moves, so it has not finished after 29 of its nodes.

## Time for the move

From D50 and the 2026-11-25 wording clarification on fisher-5sg.10,
let C be the clock of the side to move floored at zero, I its increment
(zero if absent), and M the positive moves to go (30 if absent or zero).
Divide C by M in whole milliseconds, round down and add I. The time for
the move is the smaller of that sum and C. T is the time the search may
use after the 10 ms margin (D22), floored at zero. Both the rule for
starting a depth and the interruption deadline use T.

| Side to move and parameters after `go` | Working in ms | Time for the move (ms) | Time the search may use, T (ms) | Source |
|---|---|---:|---:|---|
| White, `wtime 60000` | 60000 ÷ 30 = 2000 | 2,000 | 1,990 | D17 worked value, D50 |
| White, `wtime 10100 winc 100` | 10100 ÷ 30 rounds down to 336, plus 100 | 436 | 426 | D17 worked value, D50 |
| White, `wtime 2000 winc 50` | 2000 ÷ 30 rounds down to 66, plus 50 | 116 | 106 | D17 worked value, D50 |
| White, `wtime 60000 movestogo 40` | 60000 ÷ 40 = 1500 | 1,500 | 1,490 | D17 worked value, D50 |
| White, `wtime 60000 movestogo 1` | 60000 ÷ 1 = 60000 | 60,000 | 59,990 | D17 worked value, D50 |
| White, `wtime 12 winc 100000 movestogo 1` | 12 ÷ 1 + 100000 = 100012, limited to 12 | 12 | 2 | D17 worked value, D50 |
| White, `wtime 330` | 330 ÷ 30 = 11 | 11 | 1 | D17 worked value, D50 |
| White, `wtime 300` | 300 ÷ 30 = 10 | 10 | 0 | D17 worked value, D50 |
| White, `wtime 200` | 200 ÷ 30 rounds down to 6 | 6 | 0 | D17 worked value, D50 |
| White, `wtime 0` and `wtime -500`, separately | C is 0, so 0 ÷ 30 = 0 | 0 each | 0 each | D17 worked values, D50/D51 |
| Black, `wtime 5 btime 30000 binc 1000` | 30000 ÷ 30 = 1000, plus 1000 | 2,000 | 1,990 | D17 worked value, D50 |
| White, `wtime 60000 movestogo 0` | 60000 ÷ 30 = 2000 | 2,000 | 1,990 | D17 worked value, D50 |
| White, `wtime 1531 winc 26` | 1531 ÷ 30 rounds down to 51, plus 26 | 77 | 67 | Additional D50 case |
| Black, `wtime 17 winc 77 btime 9007 binc 28 movestogo 7` | 9007 ÷ 7 rounds down to 1286, plus 28 | 1,314 | 1,304 | Additional D50 case |
| Black, `wtime 80000 winc 9000 btime 9007 binc 28 movestogo 7` | Same Black values give 1286 + 28 | 1,314 | 1,304 | D50, change the other color's clock and increment |
| Black, `btime 510 movestogo 1` | 510 ÷ 1 = 510 | 510 | 500 | Additional D50 case |
| Black, `btime 0` and `btime -500`, separately | C is 0, so 0 ÷ 30 = 0 | 0 each | 0 each | D20, D50/D51 |
| Black, `wtime 60000 btime 0` and `wtime 60000 btime -500`, separately | Black's C is 0, so 0 ÷ 30 = 0 | 0 each | 0 each | D20, D50/D51 |
| White, `wtime 0 btime 60000` and `wtime -500 btime 60000`, separately | White's C is 0, so 0 ÷ 30 = 0 | 0 each | 0 each | D20, D50/D51 |
| White, `wtime 1500 btime 0` and `wtime 1500 btime -500`, separately | White's 1500 ÷ 30 = 50. T = 50 − 10 | 50 each | 40 each | D50/D51, other color exhausted |
| Black, `wtime 0 btime 1830` and `wtime -500 btime 1830`, separately | Black's 1830 ÷ 30 = 61. T = 61 − 10 | 61 each | 51 each | D50/D51, other color exhausted |
| White, `wtime 780` | 780 ÷ 30 = 26 | 26 | 16 | D50, starting-depth boundary |
| White, `wtime 1500` or Black, `btime 1500` | 1500 ÷ 30 = 50 | 50 | 40 | D50, expiry and combined limits |
| White, `wtime 3300` | 3300 ÷ 30 = 110 | 110 | 100 | D50, combined limits and depth duration |
| White, `wtime 1531 btime 2579 binc 47` | 1531 ÷ 30 rounds down to 51, plus 0 | 51 | 41 | D50, White's increment absent |
| Black, `wtime 1531 btime 2579 winc 23` | 2579 ÷ 30 rounds down to 85, plus 0 | 85 | 75 | D50, Black's increment absent |
| White, `wtime 1531 btime 2579 winc 23 binc 47` | 51 + 23 = 74 | 74 | 64 | D50, use White's increment |
| Black, `wtime 1531 btime 2579 winc 23 binc 47` | 85 + 47 = 132 | 132 | 122 | D50, use Black's increment |
| White, `wtime 1050` | 1050 ÷ 30 = 35 | 35 | 25 | D50, longer depth 2 |
| White, `wtime 1260` | 1260 ÷ 30 = 42 | 42 | 32 | D50, shorter depth 2 |

Run the arithmetic cases with a nonterminal root for the named color.
Assert both the time for the move and T in private `deepening` tests.
For each positive T, in fresh searches observe entry to depth 1 with
elapsed time held 1 ms below T, and no node entry when the first time
reading reaches T. For T of zero, observe zero nodes even at elapsed 0.
The scripted `uci` seam verifies both colors' clocks and increments
and the moves to go. For the negative clock of either color it expects
an accepted `go` and that clock present as zero (D51).

For White, `go btime 60000`, `go winc 1000`, `go binc 1000` and
`go movestogo 40` each supply no clock limit. With time held at zero and
no other limit, observe exactly the four starting-position depths above.
Adding `depth 1` to `go btime 60000` gives just depth 1, 20 nodes and
one legal `bestmove` (D51 and UCI go depth).

### Only the other color's clock is supplied

Use fresh searches. In private `deepening`, advance elapsed time to
2,000 ms before the first node and hold it there. If the other clock
were incorrectly used, 60,000 ÷ 30 = 2,000 ms and the 10 ms margin would
give a deadline at 1,990 ms. D51 gives no clock limit in either case.
Both searches finish depths 1–4 under D28. In scripted `uci`, require
one accepted request carrying only the clock sent, then emit the listed
reports and exactly one best-move response.

| Root and command | Private deepening observations | Scripted uci observations |
|---|---|---|
| Starting position, White to move, `go btime 60000` ([active session](../uci/go-parameters.uci), D51/D28). | No White clock. Finish depths 1–4, with cumulative nodes 20 and 79, then totals at depths 3 and 4 that have no outside value, then return the depth-4 move. | Only `btime` is present. Emit depths 1–4, all cp 0, with the totals 20, 440, 9,762 and 216,365 of the search without cutoffs and nps 10, 220, 4881, and 108182 at 2,000 ms. Emit a legal `bestmove` matching the depth-4 principal variation. |
| `7k/8/5K1Q/8/8/8/8/8 b - - 0 1`, Black to move, `go wtime 60000` (D51/D28). | No Black clock. Finish depths 1–4, with cumulative nodes 1 and 23, then totals at depths 3 and 4 that have no outside value, then return `h8g8`. | Only `wtime` is present. Emit depths 1–4 with scores cp −900, cp −900, mate −1, and mate −1, with the totals 1, 23, 75 and 857 of the search without cutoffs and nps 0, 11, 37, and 428 at 2,000 ms. Emit `bestmove h8g8`. |

The scores and node totals come from the independent search values
above. A scripted search emits the scripted reports, so cutoffs cannot
change them: they keep the totals of the search without cutoffs. D41 gives the displayed nps as the cumulative node count
divided by two here (floor of 1,000 × nodes ÷ 2,000 ms).

## Controlled schedules

Set elapsed time to zero when `go` is accepted. Use fresh searches and
advance controlled time at the stated boundaries. In a schedule that
interrupts a depth, take the expired reading inside that depth before
the specified next node. The schedule fixes when the reading observes
expiry. Production polling cadence is outside the expected values.
At a nonterminal root, compare a fallback with the first legal move
in the root's legal-move order (D20). Compare a finished depth's best
move with the first move of its principal variation (D5, D11).

In scripted `uci` sessions, send the listed finished-depth reports in
order with legal principal variations. The starting-position root moves
can tie. Use the first legal move for a scripted fallback and the first
move of the last reported principal variation otherwise. Use the
existing session interface and synchronize through acknowledged commands
and report delivery. After each finite search, send an idle `isready`
and require only `readyok`. Then require clean exit under `quit`.
Under D41 the displayed time is at least 1 ms and nps is the whole
number obtained by dividing 1000 times cumulative nodes by that
displayed time. Timing and node entry are observations of the private
search schedules.

| Command and schedule | Private deepening observations | Scripted uci observations |
|---|---|---|
| Starting position, `go wtime 780`. Start depth 1 at acceptance and finish it at 1 ms. Keep elapsed at 1 ms until depth 2 enters one node, then read 16 ms before its next node. | D49 starts depth 2 because 15 × 1 ms = 16 − 1 ms. Enter 21 nodes overall. D11 discards depth 2 and returns the depth-1 move. | Exactly depth 1, cp 0, 20 nodes at 1 ms and nps 20000, followed by one matching `bestmove`. The scripted case keeps `go wtime 1230`: a scripted search does not apply D49. |
| Same command. Start depth 1 after a dispatch delay of 1 µs. Finish it at 1 ms + 1 µs, preserving the 1 ms depth duration. | Under D49 the forecast exceeds the remaining time by 1 µs (15 ms > 14.999 ms). Enter only 20 nodes and return the depth-1 move at that decision. | Same sole finished-depth response and best-move order as the equality case. The displayed time is 1 ms. |
| Starting position, `go wtime 1500`. Enter one depth-1 node at elapsed 0. Hold elapsed at 39 ms for a reading before the second node, enter it, then read 40 ms before the third. | The D17 interruption rule permits the second node with 1 ms still available. At the deadline, enter no third node, report no finished depth and return the first legal move (D20, D11). | One matching `bestmove` with no `info`. |
| Puzzle `001om`, `go btime 1500`. Finish depth 1 at 1 ms. Let depth 2 enter 29 nodes before reading 40 ms at the next node boundary. With cutoffs the last node boundary of depth 2 has no outside value. | 15 × 1 ms = 15 ms fits in the remaining 39 ms (D49). Enter 59 nodes overall, discard depth 2 (D11), and return `g5g2`. | Exactly depth 1, cp −600, 30 nodes at 1 ms and nps 30000, PV `g5g2`, then `bestmove g5g2`. |
| Starting position, `go wtime 3300`. Finish depths 1 and 2 at 1 and 4 ms. Start depth 3 at 4 ms, then read 100 ms after one node of depth 3. | Depth 1 took 1 ms, so 15 ms fits in the remaining 99 ms. Depth 2 took 3 ms. Its forecast is 15 × 3 = 45 ms, which fits in the remaining 96 ms (D49). Enter 80 nodes overall, discard depth 3, and return the depth-2 move (D11). | Exactly depths 1 and 2 with cp 0, cumulative nodes 20 and 440, and nps 20000 and 110000 at 1 and 4 ms. Then one `bestmove` matching depth 2. |
| Starting position, `go wtime 1050`. Start depth 1 at acceptance and finish depths 1 and 2 at 1 and 5 ms. | With T = 25 ms, depth 1 took 1 ms, so 15 ms fits in 24 ms and depth 2 starts. Depth 2 took 4 ms, so 15 × 4 = 60 ms exceeds the remaining 20 ms (D49). Reusing depth 1's 15 ms forecast would admit depth 3. End at 5 ms with 79 cumulative nodes, zero depth-3 nodes, and the move of depth 2 (D5/D11). | Exactly depths 1 and 2, cp 0, 20 and 440 cumulative nodes, and nps 20000 and 88000 at 1 and 5 ms, then one `bestmove` matching depth 2. The scripted case keeps `go wtime 2790`: a scripted search does not apply D49. |
| Starting position, `go wtime 1260`. Start depth 1 at acceptance and finish depths 1 and 2 at 2 and 3 ms. Admit one depth-3 node at 3 ms and read 32 ms before the next. | With T = 32 ms, depth 1 took 2 ms, so 15 × 2 = 30 ms fits the remaining 30 ms exactly. Depth 2 took 1 ms, so 15 ms fits in the remaining 29 ms (D49). Retaining the longest duration would forecast 30 ms and refuse depth 3. Enter 80 nodes overall, discard depth 3 at the deadline and return the depth-2 move (D11). | Exactly depths 1 and 2, cp 0, 20 and 440 cumulative nodes, and nps 10000 and 146666 at 2 and 3 ms, then one `bestmove` matching depth 2. The scripted case keeps `go wtime 2160`: a scripted search does not apply D49. |

Without cutoffs the starting-position counts give 20 + (20 + 400) = 440
nodes through depth 2. With cutoffs the private search enters
20 + (20 + 39) = 79 (fisher-4yn), and one depth-3 node makes 80. The
scripted reports keep 440, because a scripted search emits them, and
the scripted nps uses D41. In the `wtime 1050` case, whose T is 25 ms,
if depth 3 enters its first node, advance elapsed time to 25 ms before
its next node so the unexpected entry fails the check without
depending on a hang timeout.

## Several limits

Repeat the finite pairs with both parameter orders. D23/D52 take the
first limit reached. D22/D31 give the search 40 ms from a `movetime 50`
and 70 ms from a `movetime 80`, after the 10 ms margin.
For each case the scripted `uci` request carries both limits.
The last case exercises the distinction in D50 between a time given by
`movetime` and a time worked out from a clock.

| Command and schedule | Private deepening observations | Scripted uci observations |
|---|---|---|
| Starting position, `go wtime 1500 movetime 80` and `go movetime 80 wtime 1500`. Finish depth 1 at 3 ms. | The clock's time for the move is 50 ms and T is 40 ms (D50). The forecast is 15 × 3 = 45 ms, exceeding the clock's remaining 37 ms. D49 and the D52 clarification end the search before the 70 ms movetime deadline. Enter 20 nodes, all at depth 1, and return its move at 3 ms. | Exactly depth 1, cp 0, 20 nodes at 2 ms and nps 10000, then one matching `bestmove`. The scripted reports are fixed inputs and keep 2 ms. |
| Starting position, `go wtime 1500 depth 2` and `go depth 2 wtime 1500`. Finish depth 1 at 3 ms. | The same 50 ms and 40 ms give a remaining 37 ms. A forecast of 45 ms ends the search under D49/D52 before the depth limit. Enter 20 nodes, all at depth 1, and return its move at 3 ms. | Exactly depth 1, cp 0, 20 nodes at 2 ms and nps 10000, then one matching `bestmove`. The scripted reports keep 2 ms. |
| Starting position, `go wtime 1500 nodes 79` and `go nodes 79 wtime 1500` in private `deepening`. The scripted `uci` case keeps `nodes 440` in both orders, because its reports are fixed inputs and it checks only that the request carries both limits. Finish depth 1 at 3 ms. | The same 45 ms forecast exceeds the remaining 37 ms (D49/D52). End at 3 ms with 20 nodes of the 79 allowed, the total at which depth 2 would finish. Enter no depth-2 node and return the depth-1 move. | Exactly depth 1, cp 0, 20 nodes at 2 ms and nps 10000, then one matching `bestmove`. The scripted reports keep 2 ms. |
| Starting position, `go wtime 1500 depth 1`. Finish depth 1 at 1 ms. | Depth wins with 39 ms left. The forecast of 15 × 1 = 15 ms would permit depth 2. Enter exactly 20 nodes and return the depth-1 move. | Exactly depth 1, cp 0, 20 nodes at 1 ms and nps 20000, then one matching `bestmove`. |
| Same command. After one node at 0, read 40 ms before entering the next node. | Clock wins before the depth limit. Enter one node, discard depth 1 and return the first legal move (D20). | One matching `bestmove` with no `info`. |
| Starting position, `go wtime 1500 nodes 19`. Hold elapsed at 1 ms. | Nodes win. Exactly 19 of the 20 nodes needed for depth 1 are entered. Return the first legal move (D20). | One matching `bestmove` with no `info`. |
| Same command. After one node at 0, read 40 ms before entering the next node. | Clock wins before node 19. Enter one node, discard depth 1 and return the first legal move (D20). | One matching `bestmove` with no `info`. |
| Starting position, `go wtime 3300 movetime 50`. Finish depth 1 at 3 ms. After one depth-2 node, read 39 ms before the next, enter it, then read 40 ms before the third. | D49 admits depth 2 against the clock's remaining 97 ms (15 × 3 = 45 ≤ 97). Measured against the 37 ms left before the movetime deadline, the same forecast would refuse depth 2 (45 > 37), so the case shows that D49 uses the clock's T alone. D52 applies movetime independently. It wins at 40 ms, ahead of the clock's 100 ms deadline. Enter 22 nodes overall and return the depth-1 move. | Exactly depth 1, cp 0, 20 nodes at 2 ms and nps 10000, then one matching `bestmove`. The scripted reports keep 2 ms. |
| Starting position, `go wtime 780 movetime 50`. Finish depth 1 at 1 ms. After one depth-2 node, read 15 ms before the next, enter it, then read 16 ms before the third. | T is 16 ms (D50). The forecast permits depth 2 at equality: 15 × 1 = 15 = 16 − 1 ms. Clock wins at 16 ms, ahead of the 40 ms movetime deadline. Enter 22 nodes overall and return the depth-1 move. | Exactly depth 1, cp 0, 20 nodes at 1 ms and nps 20000, then one matching `bestmove`. The scripted case keeps `go wtime 1230 movetime 50` in both orders: a scripted search does not apply D49. |
| Puzzle `001om`, `go infinite btime 1500`. Use the same controlled expiry inside depth 2 as above. | The clock ends the work at 40 ms with 59 nodes entered. Only depth 1 finishes and the result is `g5g2`. | Send the same depth-1 report and deliver the best-move report. Require `readyok` for `isready` before sending `stop`. Under D52 and UCI go infinite, only then require one `bestmove g5g2`. |
| Starting position, `go movetime 50` alone. Finish depth 1 at 3 ms. Enter one node of depth 2, read 39 ms before the next node, enter it, then read 40 ms before the third. | Movetime permits depth 2 even though 15 × 3 = 45 ms exceeds the remaining 37 ms. At 39 ms there is still work and no final report. At 40 ms enter no further node, discard depth 2 and return the depth-1 move (D11, D31, D50 and UCI go movetime). | Exactly depth 1, cp 0, 20 nodes at 2 ms and nps 10000. Hold the scripted final report through `isready` and `readyok`, then release it and require one matching `bestmove`. The scripted reports keep 2 ms. |

## Negative values on other go parameters

The April 2006 UCI specification describes [ignoring unrecognized
tokens](https://backscattering.de/chess/uci/#unknown)
and preserving the effect of the remaining [go parameters](https://backscattering.de/chess/uci/#gui-go).
The committed [`go depth infinite` case](../uci/go-infinite.uci) shows
how fisher treats an unreadable numeric value: the depth is absent and
`infinite` still applies. D51 changes the reading of signed values
for `wtime` and `btime`. The five cases below preserve the treatment of
unreadable numeric limits for the other parameters. The protocol
explicitly uses positive increments and moves to go.

Run each command in a fresh session at the starting position. Scripted
`uci` observes one accepted search request with the named parameter
absent and no diagnostic, then the responses for the default search.
Assert absence directly at this seam, since a value of zero could
have the same effect as absence for an increment or moves to go.
With the expected request, private `deepening` observes the D28 default
depth of 4 and cumulative nodes 20 and 79, then totals at depths 3 and 4
that have no outside value. Script four cp 0 reports with the totals
20, 440, 9,762 and 216,365 of the search without cutoffs and one legal
`bestmove` matching depth 4.

| Command | Expected parameter handling | Source |
|---|---|---|
| `go binc -1` | Discard the unreadable increment and continue the search with Black's increment absent. | UCI `binc` positive increment, D50/D51 and the unreadable-value precedent above. |
| `go movestogo -1` | Discard the unreadable moves to go and continue with moves to go absent. With no clock supplied, there is no clock limit. | UCI `movestogo` positive count, D50/D51 and the unreadable-value precedent above. |
| `go depth -1` | Discard the unreadable depth. The default depth applies. | D51, D28 and the unreadable depth in `go depth infinite`. |
| `go nodes -1` | Discard the unreadable node count. The default depth applies. | UCI `nodes` counts searched nodes, D51, D28 and the unreadable-value precedent above. |
| `go movetime -1` | Discard the unreadable time for the move. The default depth applies. | UCI `movetime` gives a search duration, D51, D28 and the unreadable-value precedent above. |
