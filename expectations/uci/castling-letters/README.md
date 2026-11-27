# Kingside castling with KK and H

Staged on 2026-11-26 by fisher-vls for fisher-4h7, and given the bare
`position` command on 2026-11-27 after review. No test reads these
sessions yet: the runner selects sessions by name directly in `uci/`, so
the `castling-letters/` directory is not played. fisher-4h7 activates them
after fisher-b0b.

## Owner decision

Recorded on fisher-4h7: over UCI, the castling field `KK` (a duplicate
letter) and `H` (a file letter) each grant the same kingside castling right
as `K` on the orthodox position `4k3/8/8/8/8/8/8/4K2R w`. This applies at the
UCI boundary only. `rules::Position::from_fen` stays strict. Chess960 and
Stockfish's other repairs of malformed FENs are out of scope, and nothing
here covers them.

## Source

[Stockfish 19 (sf_19)](https://github.com/official-stockfish/Stockfish/releases/tag/sf_19),
revision `edb0d9db6731067ec50ce619ff372b463bc4dd5d`. The binary was the
Homebrew arm64 Stockfish 19 bottle at `/opt/homebrew/bin/stockfish`, selected
from PATH as `strength/measure` selects it. Probed on 2026-11-26.

- [`src/position.cpp`, lines 302–373](https://github.com/official-stockfish/Stockfish/blob/edb0d9db6731067ec50ce619ff372b463bc4dd5d/src/position.cpp#L302-L373)
  reads the castling field. For `K` it takes the outermost rook on the
  king's rank, and for a file letter the rook on that file. A second `K`
  sets the same right again.
- [`src/uci.cpp`, lines 498–522](https://github.com/official-stockfish/Stockfish/blob/edb0d9db6731067ec50ce619ff372b463bc4dd5d/src/uci.cpp#L498-L522)
  ends the FEN at `moves` and applies the moves.

With `K`, `KK` and `H`, Stockfish prints the same position, the same FEN
(`4k3/8/8/8/8/8/8/4K2R w K - 0 1`) and the same key before a move, and the
same position, FEN (`4k3/8/8/8/8/8/8/5RK1 b - - 1 1`) and key after `e1g1`.
The captures below are byte for byte equal across the three spellings.
With `-` in place of the field, Stockfish reports `e1g1` as an illegal
move in an `info string CRITICAL ERROR` line and exits with status 1
before it reads `d`. fisher instead keeps running and leaves the position
in place, under the owner decision for a rejected position.

## The sessions

| Session | Castling field |
|---|---|
| [`kingside-k.uci`](kingside-k.uci) | `K`, the control. fisher accepts it today. |
| [`kingside-kk.uci`](kingside-kk.uci) | `KK` |
| [`kingside-h.uci`](kingside-h.uci) | `H` |

Each session sends the same three commands with its own field.

1. `position fen 4k3/8/8/8/8/8/8/4K2R w <field> - 0 1`, the bare command a
   GUI sends to set the position up and the first Stockfish probe, then
   `go depth 1`. The expected line is
   `info depth 1 multipv 1 score cp 500 nodes 15 nps {nps} time {time} pv {pv}`,
   then `bestmove {legal}`.
   - **Nodes 15.** Rule 7 of fisher-2aj counts the positions reached by
     playing a move during the search. A depth-1 search plays each legal
     move at the root once, so its total is the number of legal moves there,
     and [`expectations/README.md`](../../README.md#iterative-deepening-transcripts)
     keeps every depth-1 total exact for that reason (fisher-4yn.1, as in
     `equal-scores.uci` with 20 at the starting position). With the
     kingside right this position has 15 legal moves, `e1g1` among them.
     Without the right it has 14. python-chess 1.11.2 counts 15 for `K`,
     `KK` and `H` and 14 for `-`, and Stockfish 19 `go perft 1` reports
     `Nodes searched: 15` for `K`, `KK` and `H` and `Nodes searched: 14`
     for `-` (captures below). So 15 is the only total a correct engine can
     report, and an engine that accepts the field but drops the right
     reports 14.
   - **Score cp 500.** Rules 2 and 3: the horizon is scored by material. No
     legal move captures, so every move leaves White a rook up.
   - **The move stays open.** Rule 6 plays the first of the tied moves in the
     order of legal moves, which no expectation fixes (D48). `{pv}` and
     `{legal}` check only that the move is legal in the position.
2. `position fen 4k3/8/8/8/8/8/8/4K2R w <field> - 0 1 moves e1g1`, the
   exact Stockfish probe, then `isready`. The next line must be `readyok`.
   Without the kingside right `e1g1` is illegal, and fisher's rejection
   sends an `info string` diagnostic first (the owner decision for a
   rejected position in [`rejected-position.uci`](../rejected-position.uci)).
3. The same FEN with the moves
   `e1g1 e8d8 g1f2 d8c8 f2e3 c8b7 e3d4 b7a6 d4c5 a6a5 f1a1`, then
   `go depth 1`. After `e1g1` the king on g1 and the rook on f1 move on,
   and `f1a1` checkmates Black in `8/8/8/k1K5/8/8/8/R7 b - - 11 6`. A
   checkmated side to move gets the owner-approved terminal response
   `info depth 0 score mate 0` and `bestmove 0000`, as in
   [`terminal-positions.uci`](../terminal-positions.uci) and
   [`position-fen-four-fields.uci`](../position-fen-four-fields.uci).
   The search plays no move here, so the response does not depend on what
   the search would choose.

The first command covers the field without a move list, so an engine that
reads `KK` or `H` only when `moves` follows fails there. The second and third
cover the field with one. The third command's lines are exact. The first
uses the `{nps}`, `{time}`, `{pv}` and `{legal}` allowances, which need the
runner's reference position (see [Activation](#activation-in-fisher-4h7)).

### Why a session fails when the right is lost

python-chess 1.11.2 checked the move list from three positions after White's
first move:

| After White's first move | Result of the move list |
|---|---|
| King on g1, rook on f1 (castled) | Checkmate |
| King on g1, rook left on h1 | `f1a1` is illegal |
| Nothing moved (right lost, `e1g1` rejected) | `e1g1` itself is illegal |

A fisher that rejects the field sends a diagnostic where the session expects
the depth-1 line. One that accepts the field but drops the right reports 14
nodes there, and rejects `e1g1` later. One that castles without moving the
rook sends a diagnostic where the session expects the terminal response.

On 2026-11-27 the current transcript runner played the sessions against
fisher at `ce3dd17` (`origin/main` at `064a672` merged into this branch)
through a test added for the run and then removed:

| Session | Result |
|---|---|
| `kingside-k.uci` | Passed |
| `kingside-kk.uci` | Failed at line 17: `info string position rejected: invalid FEN: castling rights must not contain duplicates` |
| `kingside-h.uci` | Failed at line 17: `info string position rejected: invalid FEN: castling rights must use K, Q, k, q, or -` |
| A copy of `kingside-k.uci` whose bare command has the field `-` | Failed at line 17: `info depth 1 multipv 1 score cp 500 nodes 14 nps 14000 time 1 pv e1d2` |

The last row stands for an engine that accepts the field but grants no right.
An engine that reads `KK` or `H` only before a move list rejects the bare
command, as fisher does today, and fails at line 17 like the second and third
rows.

## Activation in fisher-4h7

- Move the three sessions into `uci/` and select them by name in the runner.
- Change the runner's own castling-field check in
  `crates/fisher/tests/transcripts.rs` (about line 488), which rejects `KK`
  and `H` today. The runner then sets up the reference position from the
  same field, which the first command's `{pv}` and `{legal}` need.
- Keep the history checker's behaviour. `position` in
  `crates/uci/tests/support/search_history.rs` reads the field with
  shakmaty in standard castling mode and already accepts `K`, `KK` and `H`
  with 15 legal moves before `e1g1` (a review of this staging compiled it
  unchanged).
- Add an explicit check that `rules::Position::from_fen` still rejects both
  `KK` and `H`. The owner decision keeps that parser strict, and nothing in
  these sessions would fail if `H` were accepted there instead of at the UCI
  boundary. `crates/rules/tests/positions.rs` (about line 132) rejects `X`
  and `KK` today but has no `H` case. This staging adds no test.

<details>
<summary>Stockfish 19 input and output</summary>

Each probe ran in a fresh process. `>` marks input and `<` marks standard
output. A trailing \x20 denotes one output space. Each blank unprefixed line
separates fresh processes. The first six probes are the ones recorded on
fisher-4h7; the output here was reproduced on 2026-11-26 and matched it.

```text
> position fen 4k3/8/8/8/8/8/8/4K2R w K - 0 1
> d
> quit
< Stockfish 19 by the Stockfish developers (see AUTHORS file)
<
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   | k |   |   |   | 8
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 7
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 6
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 5
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 4
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 3
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 2
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   | K |   |   | R | 1
<  +---+---+---+---+---+---+---+---+
<    a   b   c   d   e   f   g   h
<
< Fen: 4k3/8/8/8/8/8/8/4K2R w K - 0 1
< Key: 32F04C409787442D
< Checkers:\x20

> position fen 4k3/8/8/8/8/8/8/4K2R w KK - 0 1
> d
> quit
< Stockfish 19 by the Stockfish developers (see AUTHORS file)
<
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   | k |   |   |   | 8
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 7
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 6
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 5
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 4
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 3
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 2
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   | K |   |   | R | 1
<  +---+---+---+---+---+---+---+---+
<    a   b   c   d   e   f   g   h
<
< Fen: 4k3/8/8/8/8/8/8/4K2R w K - 0 1
< Key: 32F04C409787442D
< Checkers:\x20

> position fen 4k3/8/8/8/8/8/8/4K2R w H - 0 1
> d
> quit
< Stockfish 19 by the Stockfish developers (see AUTHORS file)
<
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   | k |   |   |   | 8
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 7
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 6
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 5
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 4
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 3
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 2
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   | K |   |   | R | 1
<  +---+---+---+---+---+---+---+---+
<    a   b   c   d   e   f   g   h
<
< Fen: 4k3/8/8/8/8/8/8/4K2R w K - 0 1
< Key: 32F04C409787442D
< Checkers:\x20

> position fen 4k3/8/8/8/8/8/8/4K2R w K - 0 1 moves e1g1
> d
> quit
< Stockfish 19 by the Stockfish developers (see AUTHORS file)
<
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   | k |   |   |   | 8
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 7
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 6
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 5
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 4
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 3
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 2
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   | R | K |   | 1
<  +---+---+---+---+---+---+---+---+
<    a   b   c   d   e   f   g   h
<
< Fen: 4k3/8/8/8/8/8/8/5RK1 b - - 1 1
< Key: EC78854614628876
< Checkers:\x20

> position fen 4k3/8/8/8/8/8/8/4K2R w KK - 0 1 moves e1g1
> d
> quit
< Stockfish 19 by the Stockfish developers (see AUTHORS file)
<
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   | k |   |   |   | 8
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 7
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 6
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 5
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 4
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 3
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 2
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   | R | K |   | 1
<  +---+---+---+---+---+---+---+---+
<    a   b   c   d   e   f   g   h
<
< Fen: 4k3/8/8/8/8/8/8/5RK1 b - - 1 1
< Key: EC78854614628876
< Checkers:\x20

> position fen 4k3/8/8/8/8/8/8/4K2R w H - 0 1 moves e1g1
> d
> quit
< Stockfish 19 by the Stockfish developers (see AUTHORS file)
<
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   | k |   |   |   | 8
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 7
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 6
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 5
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 4
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 3
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 2
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   | R | K |   | 1
<  +---+---+---+---+---+---+---+---+
<    a   b   c   d   e   f   g   h
<
< Fen: 4k3/8/8/8/8/8/8/5RK1 b - - 1 1
< Key: EC78854614628876
< Checkers:\x20

> position fen 4k3/8/8/8/8/8/8/4K2R w - - 0 1 moves e1g1
> d
> quit
< Stockfish 19 by the Stockfish developers (see AUTHORS file)
< info string CRITICAL ERROR: Command `position fen 4k3/8/8/8/8/8/8/4K2R w - - 0 1 moves e1g1` failed. Reason: Illegal move: e1g1
<

```

</details>

<details>
<summary>Stockfish 19 on the move list of the sessions</summary>

`quit` was sent one second after `go depth 1` so the search could finish.

```text
> position fen 4k3/8/8/8/8/8/8/4K2R w K - 0 1 moves e1g1 e8d8 g1f2 d8c8 f2e3 c8b7 e3d4 b7a6 d4c5 a6a5 f1a1
> d
> go depth 1
< Stockfish 19 by the Stockfish developers (see AUTHORS file)
<
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 8
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 7
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 6
<  +---+---+---+---+---+---+---+---+
<  | k |   | K |   |   |   |   |   | 5
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 4
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 3
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 2
<  +---+---+---+---+---+---+---+---+
<  | R |   |   |   |   |   |   |   | 1
<  +---+---+---+---+---+---+---+---+
<    a   b   c   d   e   f   g   h
<
< Fen: 8/8/8/k1K5/8/8/8/R7 b - - 11 6
< Key: 228A3A92925F281E
< Checkers: a1\x20
< info string Available processors: 0-17
< info string Using 1 thread
< info string NNUE evaluation using nn-1a298aa575a0.nnue (109MiB, (86896, 1024, 32, 32, 1))
< info string Network replica 1: Shared memory.
< info depth 0 score mate 0
< bestmove (none)
> quit

> position fen 4k3/8/8/8/8/8/8/4K2R w KK - 0 1 moves e1g1 e8d8 g1f2 d8c8 f2e3 c8b7 e3d4 b7a6 d4c5 a6a5 f1a1
> d
> go depth 1
< Stockfish 19 by the Stockfish developers (see AUTHORS file)
<
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 8
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 7
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 6
<  +---+---+---+---+---+---+---+---+
<  | k |   | K |   |   |   |   |   | 5
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 4
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 3
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 2
<  +---+---+---+---+---+---+---+---+
<  | R |   |   |   |   |   |   |   | 1
<  +---+---+---+---+---+---+---+---+
<    a   b   c   d   e   f   g   h
<
< Fen: 8/8/8/k1K5/8/8/8/R7 b - - 11 6
< Key: 228A3A92925F281E
< Checkers: a1\x20
< info string Available processors: 0-17
< info string Using 1 thread
< info string NNUE evaluation using nn-1a298aa575a0.nnue (109MiB, (86896, 1024, 32, 32, 1))
< info string Network replica 1: Shared memory.
< info depth 0 score mate 0
< bestmove (none)
> quit

> position fen 4k3/8/8/8/8/8/8/4K2R w H - 0 1 moves e1g1 e8d8 g1f2 d8c8 f2e3 c8b7 e3d4 b7a6 d4c5 a6a5 f1a1
> d
> go depth 1
< Stockfish 19 by the Stockfish developers (see AUTHORS file)
<
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 8
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 7
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 6
<  +---+---+---+---+---+---+---+---+
<  | k |   | K |   |   |   |   |   | 5
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 4
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 3
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 2
<  +---+---+---+---+---+---+---+---+
<  | R |   |   |   |   |   |   |   | 1
<  +---+---+---+---+---+---+---+---+
<    a   b   c   d   e   f   g   h
<
< Fen: 8/8/8/k1K5/8/8/8/R7 b - - 11 6
< Key: 228A3A92925F281E
< Checkers: a1\x20
< info string Available processors: 0-17
< info string Using 1 thread
< info string NNUE evaluation using nn-1a298aa575a0.nnue (109MiB, (86896, 1024, 32, 32, 1))
< info string Network replica 1: Shared memory.
< info depth 0 score mate 0
< bestmove (none)
> quit

```

</details>

<details>
<summary>Stockfish 19 go perft 1 on the bare position</summary>

Probed on 2026-11-27, one fresh process for each field.

```text
> position fen 4k3/8/8/8/8/8/8/4K2R w K - 0 1
> go perft 1
> quit
< Stockfish 19 by the Stockfish developers (see AUTHORS file)
< info string Available processors: 0-17
< info string Using 1 thread
< info string NNUE evaluation using nn-1a298aa575a0.nnue (109MiB, (86896, 1024, 32, 32, 1))
< info string Network replica 1: Shared memory.
< h1f1: 1
< h1g1: 1
< h1h2: 1
< h1h3: 1
< h1h4: 1
< h1h5: 1
< h1h6: 1
< h1h7: 1
< h1h8: 1
< e1d1: 1
< e1f1: 1
< e1d2: 1
< e1e2: 1
< e1f2: 1
< e1g1: 1
<
< Nodes searched: 15
<

> position fen 4k3/8/8/8/8/8/8/4K2R w KK - 0 1
> go perft 1
> quit
< Stockfish 19 by the Stockfish developers (see AUTHORS file)
< info string Available processors: 0-17
< info string Using 1 thread
< info string NNUE evaluation using nn-1a298aa575a0.nnue (109MiB, (86896, 1024, 32, 32, 1))
< info string Network replica 1: Shared memory.
< h1f1: 1
< h1g1: 1
< h1h2: 1
< h1h3: 1
< h1h4: 1
< h1h5: 1
< h1h6: 1
< h1h7: 1
< h1h8: 1
< e1d1: 1
< e1f1: 1
< e1d2: 1
< e1e2: 1
< e1f2: 1
< e1g1: 1
<
< Nodes searched: 15
<

> position fen 4k3/8/8/8/8/8/8/4K2R w H - 0 1
> go perft 1
> quit
< Stockfish 19 by the Stockfish developers (see AUTHORS file)
< info string Available processors: 0-17
< info string Using 1 thread
< info string NNUE evaluation using nn-1a298aa575a0.nnue (109MiB, (86896, 1024, 32, 32, 1))
< info string Network replica 1: Shared memory.
< h1f1: 1
< h1g1: 1
< h1h2: 1
< h1h3: 1
< h1h4: 1
< h1h5: 1
< h1h6: 1
< h1h7: 1
< h1h8: 1
< e1d1: 1
< e1f1: 1
< e1d2: 1
< e1e2: 1
< e1f2: 1
< e1g1: 1
<
< Nodes searched: 15
<

> position fen 4k3/8/8/8/8/8/8/4K2R w - - 0 1
> go perft 1
> quit
< Stockfish 19 by the Stockfish developers (see AUTHORS file)
< info string Available processors: 0-17
< info string Using 1 thread
< info string NNUE evaluation using nn-1a298aa575a0.nnue (109MiB, (86896, 1024, 32, 32, 1))
< info string Network replica 1: Shared memory.
< h1f1: 1
< h1g1: 1
< h1h2: 1
< h1h3: 1
< h1h4: 1
< h1h5: 1
< h1h6: 1
< h1h7: 1
< h1h8: 1
< e1d1: 1
< e1f1: 1
< e1d2: 1
< e1e2: 1
< e1f2: 1
<
< Nodes searched: 14
<

```

</details>
