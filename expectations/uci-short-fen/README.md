# FEN with four or five fields

Recorded on 2026-11-25 for fisher-og6, following the owner's decision that day
to follow Stockfish at the UCI edge. The source and binary version are
[Stockfish 19 (sf_19)](https://github.com/official-stockfish/Stockfish/releases/tag/sf_19),
revision `edb0d9db6731067ec50ce619ff372b463bc4dd5d`. The binary was the
Homebrew arm64 Stockfish 19 bottle at `/opt/homebrew/bin/stockfish`, selected
from PATH as `strength/measure` selects it.

- [`src/uci.cpp`, lines 498–522](https://github.com/official-stockfish/Stockfish/blob/edb0d9db6731067ec50ce619ff372b463bc4dd5d/src/uci.cpp#L498-L522)
  ends the FEN at `moves` and passes the remaining tokens as moves.
- [`src/position.cpp`, lines 206–211](https://github.com/official-stockfish/Stockfish/blob/edb0d9db6731067ec50ce619ff372b463bc4dd5d/src/position.cpp#L206-L211)
  zeros the position and its state.
  [Lines 420–433](https://github.com/official-stockfish/Stockfish/blob/edb0d9db6731067ec50ce619ff372b463bc4dd5d/src/position.cpp#L420-L433)
  read the halfmove clock and fullmove number without requiring the extraction
  to succeed. The missing halfmove clock stays at 0. A missing fullmove number
  becomes the same game ply as fullmove number 1 for either color.
- [`src/engine.cpp`, lines 195–214](https://github.com/official-stockfish/Stockfish/blob/edb0d9db6731067ec50ce619ff372b463bc4dd5d/src/engine.cpp#L195-L214)
  applies the moves after parsing the FEN.

Each probe ran in a fresh process. The quiet White move `g5g7` increments
the halfmove clock, leaves the fullmove number unchanged and checkmates Black.

| FEN fields after the en passant square | Stockfish clocks after the move |
|---|---|
| Omitted, or explicit `0 1` | Halfmove clock 1, fullmove number 1 |
| `17`, or explicit `17 1` | Halfmove clock 18, fullmove number 1 |

## Adoption in fisher-og6

The transcript runner selects
[the four-field session](../uci/position-fen-four-fields.uci) and
[the five-field session](../uci/position-fen-five-fields.uci) in
`expectations/uci/`. Each uses exact terminal responses so
the observation needs no reference position replay or elapsed-time reading.
The sessions check acceptance and application of the move. The Stockfish
probe observes the clock values via `d`.

The expected responses came from running
[fisher 0.1.0 at 44bfe4c](https://github.com/fernandobandeira/fisher/commit/44bfe4caec69e3b4c4a86fb1b062b9723310b46b),
built with `cargo build -p fisher`, with the equivalent six-field FENs.
Both short forms on that revision instead produced an
`info string position rejected: invalid FEN: FEN must have six fields`
where the staged sessions expect the terminal response.

<details>
<summary>Stockfish 19 input and output</summary>

In the capture, `>` marks input and `<` marks standard output.
A trailing \x20 denotes one output space. Each blank unprefixed
line separates fresh processes.

```text
> position fen 7k/8/5K2/6Q1/8/8/8/8 w - - moves g5g7
> d
> go depth 1
< Stockfish 19 by the Stockfish developers (see AUTHORS file)
<
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   | k | 8
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   | Q |   | 7
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   | K |   |   | 6
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 5
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 4
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 3
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 2
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 1
<  +---+---+---+---+---+---+---+---+
<    a   b   c   d   e   f   g   h
<
< Fen: 7k/6Q1/5K2/8/8/8/8/8 b - - 1 1
< Key: 7A0A8C4593FCCE35
< Checkers: g7\x20
< info string Available processors: 0-17
< info string Using 1 thread
< info string NNUE evaluation using nn-1a298aa575a0.nnue (109MiB, (86896, 1024, 32, 32, 1))
< info string Network replica 1: Shared memory.
< info depth 0 score mate 0
< bestmove (none)
> quit

> position fen 7k/8/5K2/6Q1/8/8/8/8 w - - 17 moves g5g7
> d
> go depth 1
< Stockfish 19 by the Stockfish developers (see AUTHORS file)
<
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   | k | 8
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   | Q |   | 7
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   | K |   |   | 6
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 5
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 4
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 3
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 2
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 1
<  +---+---+---+---+---+---+---+---+
<    a   b   c   d   e   f   g   h
<
< Fen: 7k/6Q1/5K2/8/8/8/8/8 b - - 18 1
< Key: 6E0FF73B649B4F7A
< Checkers: g7\x20
< info string Available processors: 0-17
< info string Using 1 thread
< info string NNUE evaluation using nn-1a298aa575a0.nnue (109MiB, (86896, 1024, 32, 32, 1))
< info string Network replica 1: Shared memory.
< info depth 0 score mate 0
< bestmove (none)
> quit

> position fen 7k/8/5K2/6Q1/8/8/8/8 w - - 0 1 moves g5g7
> d
> go depth 1
< Stockfish 19 by the Stockfish developers (see AUTHORS file)
<
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   | k | 8
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   | Q |   | 7
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   | K |   |   | 6
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 5
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 4
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 3
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 2
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 1
<  +---+---+---+---+---+---+---+---+
<    a   b   c   d   e   f   g   h
<
< Fen: 7k/6Q1/5K2/8/8/8/8/8 b - - 1 1
< Key: 7A0A8C4593FCCE35
< Checkers: g7\x20
< info string Available processors: 0-17
< info string Using 1 thread
< info string NNUE evaluation using nn-1a298aa575a0.nnue (109MiB, (86896, 1024, 32, 32, 1))
< info string Network replica 1: Shared memory.
< info depth 0 score mate 0
< bestmove (none)
> quit

> position fen 7k/8/5K2/6Q1/8/8/8/8 w - - 17 1 moves g5g7
> d
> go depth 1
< Stockfish 19 by the Stockfish developers (see AUTHORS file)
<
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   | k | 8
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   | Q |   | 7
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   | K |   |   | 6
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 5
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 4
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 3
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 2
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   |   |   |   |   | 1
<  +---+---+---+---+---+---+---+---+
<    a   b   c   d   e   f   g   h
<
< Fen: 7k/6Q1/5K2/8/8/8/8/8 b - - 18 1
< Key: 6E0FF73B649B4F7A
< Checkers: g7\x20
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
<summary>fisher input and output</summary>

```text
> position fen 7k/8/5K2/6Q1/8/8/8/8 w - - 0 1 moves g5g7
> go depth 1
< info depth 0 score mate 0
< bestmove 0000
> quit

> position fen 7k/8/5K2/6Q1/8/8/8/8 w - - 17 1 moves g5g7
> go depth 1
< info depth 0 score mate 0
< bestmove 0000
> quit

> position fen 7k/8/5K2/6Q1/8/8/8/8 w - - moves g5g7
> go depth 1
< info string position rejected: invalid FEN: FEN must have six fields
< info depth 1 multipv 1 score cp 0 nodes 20 nps 20000 time 1 pv b1a3
< bestmove b1a3
> quit

> position fen 7k/8/5K2/6Q1/8/8/8/8 w - - 17 moves g5g7
> go depth 1
< info string position rejected: invalid FEN: FEN must have six fields
< info depth 1 multipv 1 score cp 0 nodes 20 nps 20000 time 1 pv b1a3
< bestmove b1a3
> quit

```

</details>
