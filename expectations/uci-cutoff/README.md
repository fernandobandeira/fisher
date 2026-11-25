# Sessions for a search that makes cutoffs

Staged by fisher-4yn.1 on 2026-11-25, before fisher-4yn.2 implements
cutoffs, by an author who will not implement them. Sources: fisher-4yn
("Nodes" and "Expected sessions"), grilled with the owner on 2026-11-24 and
recorded as fisher-r6y D34 to D40; rule 7 of fisher-2aj; published perft in
[`perft.epd`](../perft.epd); the [April 2006 UCI
specification](https://backscattering.de/chess/uci/); and Knuth and Moore,
"An analysis of alpha-beta pruning", *Artificial Intelligence* 6 (1975),
for the fact that a search with cutoffs plays a subset of the moves the
search without cutoffs plays at each depth. No value here came from running
fisher.

The runner does not select these sessions. fisher-4yn.2 moves them into
[`uci/`](../uci/) when the search makes cutoffs: the two successors replace
the sessions of the same name, and `startpos-depth-2.uci` gets a test of its
own.

| Staged session | Replaces | What changes |
|---|---|---|
| [`startpos-depth-2.uci`](startpos-depth-2.uci) | Nothing | New. From the starting position, `go depth 2` reports depth 1 with cp 0 and 20 nodes, then depth 2 with cp 0 and 79 nodes. |
| [`go-nodes.uci`](go-nodes.uci) | [`uci/go-nodes.uci`](../uci/go-nodes.uci) | From the starting position, the limits 439 and 440 become 78 and 79, and depth 2 reports exactly 79 nodes. For the forced position, the 3137 case is dropped and the 3138 case accepts depths beyond 5. |
| [`go-depth.uci`](go-depth.uci) | [`uci/go-depth.uci`](../uci/go-depth.uci) | The node limit beside depth 1 becomes 79 instead of 440. |

## The node totals

Once the search makes cutoffs, a node total at depth 2 or more depends on
the order `Position::legal_moves` returns moves in (D36), which nobody
chose. fisher-4yn.1 therefore wrote `{nodes}` for every such total in
[`uci/`](../uci/), and these sessions do the same, with two kinds of
exception.

Pruning cannot change a total at depth 1: the root's bound never causes a
cutoff, so depth 1 plays every legal move at the root and the total is the
number of legal moves there. Nor can it change the response at a root with
no legal move, or `go nodes 20` from the starting position. Those stay
exact in `uci/` and here.

The starting position at depth 2 has an exact total of 79, derived in the
header of [`startpos-depth-2.uci`](startpos-depth-2.uci) and in fisher-4yn:
20 nodes at depth 1, then 1 + 20 for the first root move and 1 + 1 for each
of the other 19, because no capture is possible within two plies and the
first reply to each later root move already reaches the bound. The count
does not depend on the order of moves. A search without cutoffs reports
440 there and fails every staged line that reads 79.

## The node limits

A limit that sat on a total of the search without cutoffs at depth 2 or
more no longer marks a boundary once the search makes cutoffs.

| Session | Limit without cutoffs | Limit with cutoffs | Why |
|---|---|---|---|
| `go-nodes.uci`, starting position | 439, 440 | 78, 79 | One node short of depth 2, then the exact total at which depth 2 finishes (fisher-4yn). |
| `go-depth.uci`, starting position, beside `depth 1` | 440 | 79 | The node limit sits where depth 2 would finish, so only the depth limit ends the search after depth 1. |
| `go-nodes.uci`, forced position `7k/8/5K1Q/8/8/8/8/8 b` | 3137, 3138 | 3138 only | No outside value fixes the total at depth 5 with cutoffs. Since a search with cutoffs plays a subset of the moves at each depth, a limit of 3138 still finishes depth 5, and the session still shows an explicit node limit passing the default depth of 4 (D28). Depth 6 or more may finish too, so a repeated record follows depth 5. No limit one node short of a known total exists here, so the 3137 case goes. |

The limits 0, 1, 19, 20 and 21 in these sessions, 19 in
[`uci/go-nodes-movetime.uci`](../uci/go-nodes-movetime.uci) and 20 in
[`uci/go-parameters.uci`](../uci/go-parameters.uci) end the search at
depth 1 or before it, so cutoffs leave them as they are.
