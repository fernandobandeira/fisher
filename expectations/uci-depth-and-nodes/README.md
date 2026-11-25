# Limits on depth and nodes

Derived on 2026-11-25 using [python-chess 1.11.2](https://pypi.org/project/chess/1.11.2/)
with Python 3.9.6, [published perft](../perft.epd), the [April 2006 UCI
specification](https://backscattering.de/chess/uci/), and the owner decisions
in fisher-5sg, fisher-r6y and fisher-8ie. Full searched responses use D41 and
Stockfish sf_19 (edb0d9d), as cited in the [active grammar](../README.md).

| Staged session | Active session it succeeds |
|---|---|
| [go-depth.uci](go-depth.uci) | [uci/go-depth.uci](../uci/go-depth.uci) |
| [go-nodes.uci](go-nodes.uci) | [uci/go-nodes.uci](../uci/go-nodes.uci) |

The active `go depth infinite` case stays in [go-infinite.uci](../uci/go-infinite.uci).
Its depth argument is nonnumeric and leaves `infinite` in force.
[go-parameters.uci](../uci/go-parameters.uci) tests bare searches and
parameters deferred beyond fisher-5sg.6.

## Derivation

Enumerate legal moves with python-chess. For each search, count one node
per legal move entered at each ply, excluding the root. Score the horizon
by material and earlier terminal positions by checkmate or stalemate under
fisher-2aj rules 1–7. Minimax negates the score on return from the
position reached by each move and takes the maximum. Add each search's node count to those of all
earlier finished depths (fisher-5sg decision 10 and D28).

| Position | Legal moves at root | Nodes by ply through depth 2 | Totals at finished depths 1 and 2 | Scores at depths 1 and 2 |
|---|---|---|---|---|
| Starting position | 20 | 20, 400 | 20, 440 | cp 0, cp 0 |
| First forced position in go-nodes | h8g8 | 1, 21 | 1, 23 | cp -900, cp -900 |
| Replacement position in go-nodes | h8h7 | 1, 25 | 1, 27 | cp -900, cp -900 |
| Puzzle 001om below | 30 | 30, 1248 | 30, 1308 | cp -600, cp -700 |

Starting-position counts agree with published perft. Depth 2 alone
enters 20 + 400 nodes. The starting position has no captures within two plies,
so every move ties at the horizon. Its best move must be legal and agree
with the last reported principal variation when a depth finishes. D20
supplies the first legal move when no depth finishes. The forced positions
pin this move without relying on the order of equal moves.

For the h8g8 position, exhaustive enumeration gives the following deeper
results. The per-ply counts are successive entries of the same exhaustive
tree. Add each depth's sum to the previous finished-depth total.

| Depth | Nodes at that ply | Nodes in that search | Total since go | Score | Principal variation |
|---:|---:|---:|---:|---|---|
| 1 | 1 | 1 | 1 | cp -900 | h8g8 |
| 2 | 21 | 22 | 23 | cp -900 | h8g8 f6e5 (one of the tied variations) |
| 3 | 30 | 52 | 75 | mate -1 | h8g8 h6g7 |
| 4 | 730 | 782 | 857 | mate -1 | h8g8 h6g7 |
| 5 | 1499 | 2281 | 3138 | mate -1 | h8g8 h6g7 |

At depth 2, the mate after `h6g7` lies at the horizon and scores by material.
From depth 3, that same position ends the variation with Black checkmated
in two plies. D5 leaves tied depth-2 variations open. The depth-5 line pins
depth, score, nodes and the unique principal variation. The runner checks
the full UCI spelling, variation length and nodes-per-second arithmetic at
depth 5. Its independent variation replay ends at depth 4, and its
independent minimax comparison for literal scores ends at depth 2.

The terminal FEN has zero legal moves and the side to move is in check.
The owner-approved responses in fisher-2aj.2 (2026-11-23) supply the
depth-zero mate score and UCI's null move spelling supplies `0000`.
Stockfish sf_19 at edb0d9d handles the empty root move list before entering
iterative deepening and immediately sends the terminal score and best move.
See [search.cpp, lines 207–217](https://github.com/official-stockfish/Stockfish/blob/edb0d9d/src/search.cpp#L207-L217)
and [uci.cpp, lines 650–652](https://github.com/official-stockfish/Stockfish/blob/edb0d9d/src/uci.cpp#L650-L652).
Under D41 and D42 fisher sends `info depth 0 score mate 0` and
`bestmove 0000` for both zero limits.
Rule 8 supplies the diagnostic for each rejection, with the text from
the active go-infinite session. Decision 5 holds the best move under
`infinite`. The exact depth-1 response, then `readyok`, then both
diagnostics establish the output order before `stop`. D24 orders the
stopped search's best move before the next go. The two different forced
moves distinguish the retained position from the rejected replacement.

## Controlled schedules for fisher-5sg.6

The deepening `before_node` hook counts permission checks. When the next
node is refused, there is one more permission check than nodes entered.
The node limit counts nodes entered. Keep a search at the specified
boundary with a signal until the session handles the command. The UCI
records observe finished-depth reports and output order. Actual node
entry, a private depth cap, and the order of a command against a search
ending are observations of these schedules.

| Node limit from startpos | Nodes entered at depth 1 | Nodes entered at depth 2 | Required finished-depth totals |
|---|---:|---:|---|
| 0 | 0 | 0 | None |
| 19 | 19 | 0 | None |
| 20 | 20 | 0 | 20 |
| 21 | 20 | 1 | 20 |
| 439 | 20 | 419 | 20 |
| 440 | 20 | 420 | 20, 440 |

No row enters depth 3. Each expected count is the limit applied to the
independently enumerated searches, including nodes of an interrupted
depth. The final node at an exact boundary still permits the depth to
finish. For the two combined-limit cases that use depth 1 and 440 nodes,
depth ends the search at 20 nodes. For the two cases that use depth 2 and
20 nodes, nodes end the search at 20. For the forced position, depth 0
and nodes 0 enter zero nodes and return h8g8. With `infinite nodes 1`,
exactly one node is entered, and the move is held. With `infinite depth 1`,
the sole root move finishes depth 1 with cp -900, one node and PV h8g8.
The session receives the report and `readyok` before sending `stop`
to release the held move. On the same position `nodes 3137` enters 2280
of the 2281 nodes of depth 5 and reports through depth 4.
`nodes 3138` finishes depth 5 and reports it.

Use puzzle 001om from [mate-in-two.tsv](../mate-in-two.tsv) for the
interrupt schedules. Its FEN is
`5r1k/pp4pp/5p2/1BbQp1r1/7K/7P/1PP3P1/3R3R b - - 3 26`.
The [fisher-5sg.3 schedule](../uci-deepen-until-stop/README.md#controlled-schedules)
records the rules-order probe establishing c5b4 as the first legal move
and g5g2 as the unique best move so far at node 14. A separate throwaway
program against `rules` at af1cd01 confirmed c5b4 at entry 1 and g5g2 at
entry 14 of the 30 legal moves. Independent horizon evaluation gave cp -700
for each of the first 13 moves and cp -600 for g5g2. The independent
python-chess minimax gives g5g2 at depth 1 and c5f2 g2g3 at depth 2.

| Schedule | Required observations |
|---|---|
| Use startpos with a private depth cap of 2. Send `go depth 3`, `go depth 256`, and `go depth 18446744073709551615` in fresh sessions. | Exactly depths 1 and 2 with cp 0 and totals 20 and 440, followed by one best move from the depth-2 principal variation. Exactly 440 nodes entered. Narrowing 256 before clamping would produce zero. The largest depth is the parser's u64 maximum. |
| Send `go depth 2` on puzzle 001om. Handle `stop` before entering node 15. | Exactly 14 nodes entered, no searched report, one `bestmove c5b4` under D20. A later idle `stop` followed by `isready` produces only `readyok`. |
| Send `go nodes 14` on puzzle 001om. | Exactly 14 nodes entered, no `info` line, one `bestmove c5b4` under D20. |
| Send `go nodes 1308` on puzzle 001om. After depth 1 and 639 nodes of depth 2, handle `stop`, `quit` or end of input in separate sessions (D11, D25). | Exactly 669 nodes entered. The only searched report is depth 1, cp -600, 30 nodes, PV g5g2. Exactly `bestmove g5g2`. Quit or end of input gives clean exit with no further output. |
| Send the same go on puzzle 001om. Let the session handle the search ending before `stop`, `quit` or end of input, in separate sessions. | Exactly 1308 nodes entered. Depth 1 as above, then depth 2, cp -700, 1308 nodes, PV c5f2 g2g3. Exactly `bestmove c5f2` before the command takes effect. Idle stop adds no output. Quit or end of input gives clean exit. |
| On the h8g8 position send `go depth 1`. Hold before entering the first node. Send the h8h7 position and `go nodes 1`. After both diagnostics, handle `stop` and send `go depth 0` immediately after it. | One diagnostic per rejected command in input order. Zero nodes entered and no searched report for the interrupted search. Exactly `bestmove h8g8` for the stopped search before exactly `bestmove h8g8` for the following go. |
