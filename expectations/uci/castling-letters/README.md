# Kingside castling with KK and H

Staged on 2026-11-26 by fisher-vls for fisher-4h7. No test reads these
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

Each session sends the same two commands with its own field.

1. `position fen 4k3/8/8/8/8/8/8/4K2R w <field> - 0 1 moves e1g1`, the
   exact Stockfish probe, then `isready`. The next line must be `readyok`.
   Without the kingside right `e1g1` is illegal, and fisher's rejection
   sends an `info string` diagnostic first (the owner decision for a
   rejected position in [`rejected-position.uci`](../rejected-position.uci)).
2. The same FEN with the moves
   `e1g1 e8d8 g1f2 d8c8 f2e3 c8b7 e3d4 b7a6 d4c5 a6a5 f1a1`, then
   `go depth 1`. After `e1g1` the king on g1 and the rook on f1 move on,
   and `f1a1` checkmates Black in `8/8/8/k1K5/8/8/8/R7 b - - 11 6`. A
   checkmated side to move gets the owner-approved terminal response
   `info depth 0 score mate 0` and `bestmove 0000`, as in
   [`terminal-positions.uci`](../terminal-positions.uci) and
   [`position-fen-four-fields.uci`](../position-fen-four-fields.uci).
   The search plays no move here, so the response does not depend on what
   the search would choose.

Every expected line is exact, so the runner needs no reference replay of
these positions and no `{legal}` or `{score}` allowance.

### Why a session fails when the right is lost

python-chess 1.11.2 checked the move list from three positions after White's
first move:

| After White's first move | Result of the move list |
|---|---|
| King on g1, rook on f1 (castled) | Checkmate |
| King on g1, rook left on h1 | `f1a1` is illegal |
| Nothing moved (right lost, `e1g1` rejected) | `e1g1` itself is illegal |

A fisher that rejects the field, reads it as no right, or castles without
moving the rook therefore sends a diagnostic where the session expects
`readyok` or the terminal response.

On 2026-11-26 the current transcript runner played the three sessions
against fisher at `e088fec` through a test added for the run and then
removed. `kingside-k.uci` passed. `kingside-kk.uci` failed at line 13 with
`info string position rejected: invalid FEN: castling rights must not contain duplicates`,
and `kingside-h.uci` failed at line 13 with
`info string position rejected: invalid FEN: castling rights must use K, Q, k, q, or -`.
A copy of `kingside-k.uci` with the field `-` failed at line 12 with
`info string position rejected: illegal UCI move e1g1`.

## Activation in fisher-4h7

Move the three sessions into `uci/` and select them by name in the runner.
The runner's reference parser in `crates/fisher/tests/transcripts.rs` and
the history checker in `crates/uci/tests/support/search_history.rs` reject
`KK` and `H` today. fisher-4h7 owns those changes.

<details>
<summary>Stockfish 19 input and output</summary>

Each probe ran in a fresh process. `>` marks input and `<` marks standard
output. A trailing   denotes one output space. Each blank unprefixed line
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
