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
[`uci/`](../uci/), and these sessions do the same, with three kinds of
exception. Six more sessions already matched node totals only through
`{nodes}` in repeated records and are unchanged: end-of-input-while-deepening,
go-healthy-clock, go-movetime, go-then-end-of-input, go-then-quit and
quit-while-deepening.

Pruning cannot change a total at depth 1: the root's bound never causes a
cutoff, so depth 1 plays every legal move at the root and the total is the
number of legal moves there. Nor can it change the response at a root with
no legal move, or `go nodes 20` from the starting position. Those stay
exact in `uci/` and here.

Nor can it change a depth-2 total below a root with one legal move. Nothing
bounds the root, so its sole move leaves the replies unbounded and depth 2
plays every one of them. A depth-1 total of 1 marks such a root. The owner
decided on 2026-11-25 (PR #74) to keep these totals exact: 23 and 27 for
the forced positions `7k/8/5K1Q/8/8/8/8/8 b` and `7k/8/5K2/6Q1/8/8/8/8 b`,
and, by the same rule, 66, 42 and 41 for the three other roots with one
legal move in `position-moves.uci`. They keep the values they had before
fisher-4yn.1.

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
it is a node total at depth 2 or more, a node limit that sits on one, a
node boundary inside depth 2 or later, or a node count or node-per-second
figure that follows from one.

Every new number comes from six facts:

1. Depth 1 plays every legal move at the root, because nothing bounds the
   root from above. The same holds at every depth: each search plays all
   the root moves.
2. For the same reason a root with one legal move plays every reply at
   depth 2, and in every deeper search plies 1 and 2 keep their counts:
   1 and 21 for h8g8, 1 and 25 for h8h7. Their totals through depth 2 stay
   23 and 27.
3. From the starting position depth 2 enters 20 + 39 = 59 nodes, for a
   total of 79 (fisher-4yn). One node of depth 3 makes 80.
4. A search with cutoffs plays a subset of the moves the search without
   cutoffs plays at each depth (Knuth and Moore 1975), so every exhaustive
   total is an upper bound.
5. Depth 2 on puzzle 001om plays all 30 root moves, so after 29 of its
   nodes (59 in all) at least one root move is unplayed and depth 2 has not
   finished. That is the latest node boundary inside depth 2 that does not
   depend on the order of moves.
6. Nodes per second follow D41: 1000 × 79 ÷ 5 = 15,800.

Where no outside value exists, the staged file says "no outside value"
and gives the exhaustive total as an upper bound. The implementer of
fisher-4yn.2 must not replace it with a count from fisher. A test of such a
total keeps the exact prefix (depth 1, and depth 2 where facts 2 or 3 give
it), then checks that each later total is greater than the one before, no
greater than the exhaustive total at that depth, and no greater than any
node limit.

### What fisher-4yn.2 switches to

Test paths are under `crates/deepening/`, at revision 06de6a6. "Check N"
is the Nth call of the test's node hook, made before node N is entered.

| File | Schedule or value | Without cutoffs | With cutoffs | Test that copies it today |
|---|---|---|---|---|
| uci-depth-and-nodes | Starting position, depth-2 search by ply, and totals | 20, 400; 20, 440 | 20, 39; 20, 79 | `src/tests.rs` 229–230, 257–258 |
| uci-depth-and-nodes | Node limits from startpos (entered at depth 2; totals) | 439 (419; 20), 440 (420; 20, 440) | 78 (58; 20), 79 (59; 20, 79) | `src/tests.rs` 222–244 |
| uci-depth-and-nodes | Combined limits with depth 1 | `nodes 440` | `nodes 79` | `src/tests.rs` 263–279 |
| uci-depth-and-nodes | Private depth cap 2 with `go depth 3`, 256, u64 max | 440 entered; totals 20, 440 | 79 entered; totals 20, 79 | `src/tests.rs` 246–261 |
| uci-depth-and-nodes | Forced position h8g8, `go depth 5` and the default depth 4 | Totals 1, 23, 75, 857, 3138 | Exactly depths 1–5 (or 1–4). Totals 1, 23, then rising and at most 75, 857, 3138 | `tests/search.rs` 81–106 (857), 152–193 |
| uci-depth-and-nodes | Forced position, `nodes 3137` and `nodes 3138` | Depths 1–4; depths 1–5 | 3137 dropped. 3138: consecutive depths from 1 through at least 5, further depths allowed; totals 1, 23, then rising, at most 75, 857, 3138 at depths 3–5, never above 3138 | `tests/search.rs` 152–193, whose helper `reports_with_limits` (21–44) rejects any depth above `expected_depth` and must allow more for 3138 |
| uci-depth-and-nodes, uci-deepen-until-stop | Puzzle 001om, `stop`, `quit` or end of input inside depth 2 | `go nodes 1308`, after node 639 of depth 2, 669 entered | `go depth 2`, after node 29 of depth 2, 59 entered, only the depth-1 report, `bestmove g5g2` | `src/tests.rs` 120–156: the limit `nodes: Some(1308)` becomes `depth: Some(2)`; `Some(30 + 640)` and 670 at line 122 become `Some(30 + 30)` and 60; 669 at line 148 becomes 59 |
| uci-depth-and-nodes, uci-deepen-until-stop | Puzzle 001om, search ends on its own after depth 2, and the derivation tables | `go nodes 1308`, 1308 entered, depth 2 reports 1308; depth 2 by ply 30, 1248 | `go depth 2`. Node checks and nodes entered both equal the depth-2 total: no outside value (at least 60 by fact 5, at most 1308). cp −700, PV c5f2 g2g3, `bestmove c5f2` | `src/tests.rs` 120–164 (the 1308 at lines 122 and 148) |
| uci-deepen-until-stop | Starting position, cap 2, depths at 2 and 5 ms | Totals 20, 440; nps 10,000, 88,000; 440 checks | Totals 20, 79; nps 10,000, 15,800; 79 checks | `src/tests.rs` 70–118 |
| uci-go-movetime | Puzzle, `go movetime 50`, expiry or interrupt at the first reading inside depth 2 | Production interval: check 1024; readings at checks 0, 30, 30, 1024 | `run_search::<60>`: the event at check 60; readings 0, 30, 30, 60 (or 0, 30, 30 when the interrupt comes first); 60 checks; 59 entered; depth 1 only; `g5g2` | `src/tests/movetime.rs` 78–123 |
| uci-go-movetime | Starting position, `go movetime 50`, depths at 2 and 5 ms, expiry at the first reading in depth 3 | Production interval: check 1024; totals 20, 440 | `run_search::<81>`: expiry at check 81; 81 checks; totals 20, 79; 80 entered; depth-2 move | `src/tests/movetime.rs` 126–162 |
| uci-go-movetime | Starting position, `go nodes 1308 movetime 50`, held at 39 ms | Expiry at check 1024; 1023 entered; totals 20, 440 | `run_search::<81>`: expiry at check 81; 80 entered; totals 20, 79; movetime wins before the node limit. The rows with `nodes 19` are unchanged | `src/tests/movetime.rs` 309–339 (no README row before this commit) |
| uci-go-movetime | Production interval of 1,024 nodes | Starting position, cap 3, 40 ms from check 1025, expect 2048 checks | Starting position, `movetime 50`, cap at least 103, 39 ms until check 2048, then 40 ms: readings at checks 1024 and 2048, 2048 checks, 2047 entered | `src/tests/movetime.rs` 212–230 |
| uci-go-movetime | Starting-position totals through depth 4 | 20, 440, 9,762, 216,365 | 20, 79, then no outside value | None directly |
| uci-go-movetime, uci-go-clocks | Puzzle 001om, the total at which depth 2 finishes, in the derivation | 1,308 | No outside value (at least 60, at most 1,308) | None directly |
| uci-go-clocks | Search values for the three roots in the sessions | 20, 440, 9,762, 216,365; 1, 23, 75, 857; 1, 27, 98, 1,397 | 20, 79; 1, 23; 1, 27; then no outside value | `src/tests/clocks.rs` 225–261 (`btime`, `winc`, `binc`, `movestogo` alone) |
| uci-go-clocks | `go btime 60000` from startpos, private | Totals 20, 440, 9,762, 216,365 | 20, 79, then no outside value | `src/tests/clocks.rs` 263–276 |
| uci-go-clocks | `go wtime 60000` on h8g8, private | Totals 1, 23, 75, 857 | 1, 23, then rising, at most 75 and 857 | `src/tests/clocks.rs` 278–305 |
| uci-go-clocks | Negative values on other parameters, private default search | Totals 20, 440, 9,762, 216,365 | 20, 79, then no outside value | `src/tests/clocks.rs` 307–314 |
| uci-go-clocks | Puzzle 001om, `go btime 1500` and `go infinite btime 1500`, expiry inside depth 2 | Schedule `(30, 1 ms), (1308, 40 ms)`; 1,307 entered | Schedule `(30, 1 ms), (60, 40 ms)`; 59 entered; `g5g2`. The last node boundary of depth 2 has no outside value | `src/tests/clocks.rs` 395–422 |
| uci-go-clocks | `go wtime 3300`, one depth-3 node | Schedule `(20, 1 ms), (440, 4 ms), (442, 100 ms)`; 441 entered | `(20, 1 ms), (79, 4 ms), (81, 100 ms)`; 80 entered | `src/tests/clocks.rs` 424–433 |
| uci-go-clocks | `go wtime 2790`, end at 5 ms | `(20, 1 ms), (440, 5 ms), (442, 83 ms)`; 440 entered | `(20, 1 ms), (79, 5 ms), (81, 83 ms)`; 79 entered | `src/tests/clocks.rs` 435–444 |
| uci-go-clocks | `go wtime 2160`, one depth-3 node | `(20, 2 ms), (440, 3 ms), (442, 62 ms)`; 441 entered | `(20, 2 ms), (79, 3 ms), (81, 62 ms)`; 80 entered | `src/tests/clocks.rs` 446–455 |
| uci-go-clocks | `go wtime 1500 nodes …`, both orders, private | `nodes 440` | `nodes 79` | `src/tests/clocks.rs` 457–484 (line 468) |

### What stays exact

- Every count inside depth 1, every interruption before depth 2, and every
  limit that ends the search at depth 1 or earlier (0, 1, 14, 19, 20, 21).
- The scores, principal variations, best moves, times and decisions about
  starting a depth. Cutoffs change none of them (fisher-4yn).
- The scripted `uci` observations in `uci-go-clocks`, including the command
  of the scripted `go wtime 1500 nodes 440` case
  (`crates/uci/tests/threaded_session/clocks.rs` 342–344). A scripted search
  emits fixed reports, so cutoffs cannot change them. They keep 440, 9,762,
  216,365 and their nps (110000, 88000, 146666 and the 2,000 ms figures).
  The scripted tests in `crates/uci` need no change.

### Corrections to the first version of this table (80c81e4)

The first version said every changed value had an entry here. It did not.
It dropped the h8g8 counts at plies 1 and 2 without an entry, and moved the
puzzle interruptions to node 1 of depth 2 when node 29 was available. It
changed the command of the scripted `nodes 440` case while saying
`crates/uci` needed no change. It missed the production-interval test at
`src/tests/movetime.rs` 212–230, and it left the movetime schedules tied to
the production interval. Each of these now has an entry above.
