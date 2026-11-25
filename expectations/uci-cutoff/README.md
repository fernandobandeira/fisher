# Sessions and schedules for a search that makes cutoffs

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

The runner does not select these sessions, and no test reads the staged
schedules. fisher-4yn.2 moves them into place when the search makes
cutoffs. The two successor sessions replace the sessions of the same name
in [`uci/`](../uci/), and `startpos-depth-2.uci` gets a test of its own.
Each staged schedule file replaces the README of the directory it is named
after (see [Controlled schedules](#controlled-schedules)).

| Staged session | Replaces | What changes |
|---|---|---|
| [`startpos-depth-2.uci`](startpos-depth-2.uci) | Nothing | New. From the starting position, `go depth 2` reports depth 1 with cp 0 and 20 nodes, then depth 2 with cp 0 and 79 nodes. |
| [`go-nodes.uci`](go-nodes.uci) | [`uci/go-nodes.uci`](../uci/go-nodes.uci) | From the starting position, the limits 439 and 440 become 78 and 79, and depth 2 reports exactly 79 nodes. For the forced position, the 3137 case is dropped and the 3138 case accepts depths beyond 5. |
| [`go-depth.uci`](go-depth.uci) | [`uci/go-depth.uci`](../uci/go-depth.uci) | The node limit beside depth 1 becomes 79 instead of 440. |
| [`uci-depth-and-nodes.md`](uci-depth-and-nodes.md) | [`uci-depth-and-nodes/README.md`](../uci-depth-and-nodes/README.md) | See [Controlled schedules](#controlled-schedules). |
| [`uci-deepen-until-stop.md`](uci-deepen-until-stop.md) | [`uci-deepen-until-stop/README.md`](../uci-deepen-until-stop/README.md) | See [Controlled schedules](#controlled-schedules). |
| [`uci-go-movetime.md`](uci-go-movetime.md) | [`uci-go-movetime/README.md`](../uci-go-movetime/README.md) | See [Controlled schedules](#controlled-schedules). |
| [`uci-go-clocks.md`](uci-go-clocks.md) | [`uci-go-clocks/README.md`](../uci-go-clocks/README.md) | See [Controlled schedules](#controlled-schedules). |

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

## Controlled schedules

The owner asked on 2026-11-25 (PR #74) for the four READMEs of controlled
schedules to be staged the same way. The private `deepening` tests copy
their values. Each staged file is a full copy of its README with only the
values below changed, and a note under its title. A value is changed when
it is a node total at depth 2 or more, a node limit that sits on one, or a
node count or node-per-second figure that follows from one.

Every new number comes from five facts. Depth 1 plays every legal move at
the root, because nothing bounds the root from above. For the same reason
a root with one legal move plays every reply at depth 2, so the forced
positions keep 1 and 23 (h8g8) and 1 and 27 (h8h7). From the starting
position depth 2 enters 20 + 39 = 59 nodes, for a total of 79 (fisher-4yn).
A search with cutoffs plays a subset of the moves the search without cutoffs
plays at each depth (Knuth and Moore 1975), so every exhaustive total is an
upper bound. Depth 2 on puzzle 001om plays all 30 root moves, so it
cannot finish after its first node. Nodes per second follow D41:
1000 × 79 ÷ 5 = 15,800.

Where no outside value exists, the staged file says "no outside value"
and gives the exhaustive total as an upper bound. The implementer of
fisher-4yn.2 must not replace it with a count from fisher.

### What fisher-4yn.2 switches to

| File | Schedule or value | Without cutoffs | With cutoffs | Test that copies it today |
|---|---|---|---|---|
| uci-depth-and-nodes | Starting position, depth-2 search by ply, and totals | 20, 400; 20, 440 | 20, 39; 20, 79 | `src/tests.rs` 229–230, 257–258 |
| uci-depth-and-nodes | Node limits from startpos (entered at depth 2; totals) | 439 (419; 20), 440 (420; 20, 440) | 78 (58; 20), 79 (59; 20, 79) | `src/tests.rs` 229–230 |
| uci-depth-and-nodes | Combined limits with depth 1 | `nodes 440` | `nodes 79` | `src/tests.rs` 264 |
| uci-depth-and-nodes | Private depth cap 2 with `go depth 3`, 256, u64 max | 440 entered; totals 20, 440 | 79 entered; totals 20, 79 | `src/tests.rs` 257–258 |
| uci-depth-and-nodes | Forced position h8g8, totals at depths 1–5 | 1, 23, 75, 857, 3138 | 1, 23, then no outside value | `tests/search.rs` 95, 162–178 |
| uci-depth-and-nodes | Forced position, `nodes 3137` and `nodes 3138` | Depths 1–4; depths 1–5 | 3137 dropped; 3138 finishes depth 5 or deeper (owner decision) | `tests/search.rs` 162–178 |
| uci-depth-and-nodes | Puzzle 001om, interrupt inside depth 2 | `go nodes 1308`, after node 639 of depth 2, 669 entered | `go depth 2`, after node 1 of depth 2, 31 entered | `src/tests.rs` 122–148 |
| uci-depth-and-nodes | Puzzle 001om, search ends on its own | `go nodes 1308`, 1308 entered, depth 2 reports 1308 | `go depth 2`, entered equals the depth-2 total: no outside value (at most 1308) | `src/tests.rs` 148, 164 |
| uci-deepen-until-stop | Puzzle 001om, `stop` inside depth 2 | After node 639 of 1,278 | After node 1 of depth 2 (31 entered) | `src/tests.rs` 122–148 |
| uci-deepen-until-stop | Puzzle 001om, private depth cap 2 | Depth 2 total 1,308 | No outside value (at most 1,308) | `src/tests.rs` 164 |
| uci-deepen-until-stop | Starting position, cap 2, depths at 2 and 5 ms | Totals 20, 440; nps 10,000, 88,000 | Totals 20, 79; nps 10,000, 15,800 | `src/tests.rs` 108–109 |
| uci-go-movetime | Starting-position totals through depth 4 | 20, 440, 9,762, 216,365 | 20, 79, then no outside value | None directly |
| uci-go-movetime | Starting position, `go movetime 50`, depths at 2 and 5 ms | Totals 20, 440; nps 10000, 88000 | Totals 20, 79; nps 10000, 15800 | `src/tests/movetime.rs` 159 |
| uci-go-clocks | Search values for the three roots in the sessions | 20, 440, 9,762, 216,365; 1, 23, 75, 857; 1, 27, 98, 1,397 | 20, 79; 1, 23; 1, 27; then no outside value | `src/tests/clocks.rs` 247–248 (`btime`, `winc`, `binc`, `movestogo` alone) |
| uci-go-clocks | `go btime 60000` from startpos, private | Totals 20, 440, 9,762, 216,365 | 20, 79, then no outside value | `src/tests/clocks.rs` 273–274 |
| uci-go-clocks | `go wtime 60000` on h8g8, private | Totals 1, 23, 75, 857 | 1, 23, then no outside value | `src/tests/clocks.rs` 282, 291 |
| uci-go-clocks | Puzzle 001om, `go btime 1500`, expiry inside depth 2 | After 1,277 of 1,278 nodes of depth 2; 1,307 entered | After node 1 of depth 2; 31 entered. The last node boundary has no outside value | `src/tests/clocks.rs` 407–408 |
| uci-go-clocks | Puzzle 001om, `go infinite btime 1500` | 1,307 entered | 31 entered | `src/tests/clocks.rs` 407–408 |
| uci-go-clocks | `go wtime 3300`, one depth-3 node | 441 entered | 80 entered | `src/tests/clocks.rs` 428–430 |
| uci-go-clocks | `go wtime 2790`, end at 5 ms | 440 entered | 79 entered | `src/tests/clocks.rs` 439–441 |
| uci-go-clocks | `go wtime 2160`, one depth-3 node | 441 entered | 80 entered | `src/tests/clocks.rs` 450–452 |
| uci-go-clocks | `go wtime 1500 nodes …`, both orders | `nodes 440` | `nodes 79` | `src/tests/clocks.rs` 468 |
| uci-go-clocks | Negative values on other parameters, private default search | Totals 20, 440, 9,762, 216,365 | 20, 79, then no outside value | `src/tests/clocks.rs` 311–312 |

Test paths are under `crates/deepening/`, at revision 06de6a6.

### What stays exact

- Every count inside depth 1, every interruption before depth 2, and every
  limit that ends the search at depth 1 or earlier (0, 1, 14, 19, 20, 21).
- The scores, principal variations, best moves, times and decisions about
  starting a depth. Cutoffs change none of them (fisher-4yn).
- The scripted `uci` observations in `uci-go-clocks`. A scripted search emits
  those reports, so cutoffs cannot change them. They keep 440, 9,762,
  216,365 and their nps (110000, 88000, 146666 and the 2,000 ms figures).
  The scripted tests in `crates/uci` need no change.

### What the staged values cannot settle

- The private `deepening` tests time their expiry by node checks: a time
  reading every 1024 nodes (`NODES_BETWEEN_READINGS`). `src/tests/movetime.rs`
  95–109 and 141–147 expect the reading at check 1024 to fall inside depth 2
  of puzzle 001om and inside depth 3 of the starting position. With cutoffs,
  where check 1024 falls has no outside value. The schedules name the
  reading by its depth ("the first time reading inside depth 2"), so the
  implementer must reach that reading without a fixed node count.
- `src/tests/movetime.rs` 314 runs the starting position with `nodes 1308`
  and expects totals 20 and 440, with 1023 nodes entered. No README row
  supplies it. With cutoffs its outcome has no outside value.
