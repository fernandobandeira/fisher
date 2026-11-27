# A promoted-piece marker and nine pawns in a FEN board

Staged on 2026-11-27 by fisher-2ou. No test reads these sessions yet: they
sit outside `uci/`, as the [cutoff sessions](../uci-cutoff/README.md) did
before fisher-4yn.2 moved them, because every document under `uci/` must
be played or listed as staged (fisher-my3).

## The two boards

fisher-2ou found two boards on which the engine and the UCI history
checker (`crates/uci/tests/support/search_history.rs`) disagree. The
transcript runner agreed with the engine on both.

| Board | Engine at `9772649` | History checker | Stockfish 19 | fisher must |
|---|---|---|---|---|
| `4k3/8/8/8/8/8/8/4K2R~ w K - 0 1`, a `~` after a piece | Rejects: `each rank must have eight files` | Accepts | Rejects: `Invalid FEN. Invalid file reached.` | Reject |
| `4k3/8/8/8/8/P7/PPPPPPPP/4K2R w K - 0 1`, nine white pawns | Accepts | Rejects | Rejects: `Unsupported position. WHITE has more than 8 pawns.` | Reject |

## Source

[Stockfish 19 (sf_19)](https://github.com/official-stockfish/Stockfish/releases/tag/sf_19),
revision `edb0d9db6731067ec50ce619ff372b463bc4dd5d`. The binary was the
Homebrew arm64 Stockfish 19 bottle at `/opt/homebrew/bin/stockfish`, selected
from PATH as `strength/measure` selects it. Its first line of output is
`Stockfish 19 by the Stockfish developers (see AUTHORS file)`. Probed on
2026-11-27.

- [`src/position.cpp`, lines 219–268](https://github.com/official-stockfish/Stockfish/blob/edb0d9db6731067ec50ce619ff372b463bc4dd5d/src/position.cpp#L219-L268)
  read the piece placement. A character other than a digit, `/` or one of
  the twelve piece letters fails: after the eighth file with
  `Invalid file reached`, inside a rank with `Invalid piece: ~`. Stockfish
  has no promoted-piece marker.
- [Lines 279–283](https://github.com/official-stockfish/Stockfish/blob/edb0d9db6731067ec50ce619ff372b463bc4dd5d/src/position.cpp#L279-L283)
  reject a board on which either color has more than eight pawns.
- [`src/uci.cpp`, lines 697–702](https://github.com/official-stockfish/Stockfish/blob/edb0d9db6731067ec50ce619ff372b463bc4dd5d/src/uci.cpp#L697-L702)
  print the `CRITICAL ERROR` line and exit with status 1.

## Decisions that apply

- **Follow Stockfish** for a protocol detail the UCI specification leaves
  open, as the owner decided on fisher-og6 (2026-11-25) and fisher-4h7.
  Stockfish rejects both boards, so fisher rejects both.
- **A rejected position** gets one `info string` diagnostic and leaves the
  earlier position in place, under the owner decision in
  [`rejected-position.uci`](../uci/rejected-position.uci) and fisher-9vk.
  Stockfish exits instead; fisher keeps running, as the
  [castling-letter sessions](../uci/castling-letters/README.md) record.
- **The boards fisher-l7f.5 requires `rules::Position::from_fen` to reject**
  (a color without exactly one king, a pawn on rank 1 or 8, adjacent kings,
  castling rights without their king and rook) are not an exhaustive list.
  Rejecting nine pawns adds to it and contradicts no earlier decision. No
  decision accepts a promoted-piece marker.

## The sessions

Both start from `7k/8/5K1Q/8/8/8/8/8 b - - 0 1`, where Black's only legal
move is `h8g8`, as in `rejected-position.uci`. Each rejected board must
send one diagnostic before `readyok`. The final `go depth 1` must then
search the earlier position: `cp -900`, `nodes 1` and `bestmove h8g8`. On
any of the rejected boards the side to move has other moves, so a fisher
that keeps such a board fails there. The wording of the diagnostic is open
(`{message}`); no outside source chose it.

| Session | Boards it sends |
|---|---|
| [`promoted-marker.uci`](promoted-marker.uci) | `4k3/8/8/8/8/8/8/4K2R~ w K - 0 1` (the ticket's board, `~` after the eighth file) and `4k3/8/8/8/8/8/8/R~3K3 w Q - 0 1` (`~` inside a rank that adds up to eight files without it) |
| [`nine-pawns.uci`](nine-pawns.uci) | `4k3/8/8/8/8/P7/PPPPPPPP/4K2R w K - 0 1` (the ticket's board), `4k3/pppppppp/p7/8/8/8/8/4K3 b - - 0 1` (nine black pawns), then the control `4k3/8/8/8/8/8/PPPPPPPP/4K2R w K - 0 1` |

The control has eight pawns and must be accepted. Stockfish 19 accepts it
and `go perft 1` counts 21 legal moves, so its depth-1 line is exact under
rule 7: `nodes 21`, and `cp 1300` under rule 2 (eight pawns and a rook, no
capture available). A fisher that rejects every board with many pawns fails
there.

## Runs against fisher at 9772649

The current runner played both sessions through a test added for the run
and then removed, each copied to `uci/fen-boards/`:

| Fisher | `promoted-marker.uci` | `nine-pawns.uci` |
|---|---|---|
| `9772649` unchanged | Passed | Failed at line 14: timed out waiting for the diagnostic (fisher accepts nine pawns) |
| `9772649` with a throwaway check in `rules` that rejects more than eight pawns of a color | Passed | Failed at line 30: the runner's reference position had accepted the black nine-pawn board, so it called `h8g8` illegal |
| `9772649` unchanged, on a copy holding only the control's records | | Passed. With seven pawns in place of eight it failed: `cp 1200 nodes 26` |

The second row shows that the runner needs its own change.

## Activation in fisher-2ou

The implementer changes three things, then moves the two sessions
unchanged into `expectations/uci/fen-boards/` and registers them as
transcript tests named `fen-boards/promoted-marker` and
`fen-boards/nine-pawns`.

1. **Engine.** Reject a board on which either color has more than eight
   pawns. Today `rules::Position::from_fen` accepts it. The `~` board is
   already rejected; keep it so.
2. **Transcript runner** (`crates/fisher/tests/transcripts.rs`). `Tracked::from_fen`
   calls `ignore_too_much_material`, so its reference accepts nine pawns.
   It must reject them as fisher will. Shakmaty's `TOO_MUCH_MATERIAL`
   also counts promoted pieces by bishop color, which is not Stockfish's
   rule and not part of this ticket, so the runner cannot simply stop
   ignoring it.
3. **History checker** (`crates/uci/tests/support/search_history.rs`).
   `fen_position` passes the board to shakmaty, which reads `~` as a
   Crazyhouse promoted piece and accepts it. It must reject any `~` in the
   board. It already rejects nine pawns. The expected verdict for every
   board above is rejection, and acceptance for the control.

Other boards on which Stockfish, shakmaty and fisher may differ, such as
Stockfish's `Too many pieces for WHITE.` (more extra pieces than missing
pawns) and shakmaty's impossible check, are out of scope here.

<details>
<summary>Stockfish 19 input and output</summary>

In the capture, `>` marks input and `<` marks standard output. A trailing
\x20 denotes one output space. Each blank unprefixed line separates fresh
processes, and the exit status follows each. The four `info string` lines
Stockfish printed about processors, threads and the network before the
perft count are left out. The rejected boards printed none: Stockfish
exited before `go`.

```text
> position fen 4k3/8/8/8/8/8/8/4K2R~ w K - 0 1
> d
> go depth 1
> quit
< Stockfish 19 by the Stockfish developers (see AUTHORS file)
< info string CRITICAL ERROR: Command `position fen 4k3/8/8/8/8/8/8/4K2R~ w K - 0 1` failed. Reason: Invalid FEN. Invalid file reached.
<
(exit status 1)

> position fen 4k3/8/8/8/8/8/8/R~3K3 w Q - 0 1
> d
> go depth 1
> quit
< Stockfish 19 by the Stockfish developers (see AUTHORS file)
< info string CRITICAL ERROR: Command `position fen 4k3/8/8/8/8/8/8/R~3K3 w Q - 0 1` failed. Reason: Invalid FEN. Invalid piece: ~
<
(exit status 1)

> position fen 4k3/8/8/8/8/P7/PPPPPPPP/4K2R w K - 0 1
> d
> go depth 1
> quit
< Stockfish 19 by the Stockfish developers (see AUTHORS file)
< info string CRITICAL ERROR: Command `position fen 4k3/8/8/8/8/P7/PPPPPPPP/4K2R w K - 0 1` failed. Reason: Unsupported position. WHITE has more than 8 pawns.
<
(exit status 1)

> position fen 4k3/pppppppp/p7/8/8/8/8/4K3 b - - 0 1
> d
> go depth 1
> quit
< Stockfish 19 by the Stockfish developers (see AUTHORS file)
< info string CRITICAL ERROR: Command `position fen 4k3/pppppppp/p7/8/8/8/8/4K3 b - - 0 1` failed. Reason: Unsupported position. BLACK has more than 8 pawns.
<
(exit status 1)

> position fen 4k3/8/8/8/8/8/PPPPPPPP/4K2R w K - 0 1
> d
> go perft 1
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
<  | P | P | P | P | P | P | P | P | 2
<  +---+---+---+---+---+---+---+---+
<  |   |   |   |   | K |   |   | R | 1
<  +---+---+---+---+---+---+---+---+
<    a   b   c   d   e   f   g   h
<
< Fen: 4k3/8/8/8/8/8/PPPPPPPP/4K2R w K - 0 1
< Key: A40A38FF3AA93896
< Checkers:\x20
< a2a3: 1
< b2b3: 1
< c2c3: 1
< d2d3: 1
< e2e3: 1
< f2f3: 1
< g2g3: 1
< h2h3: 1
< a2a4: 1
< b2b4: 1
< c2c4: 1
< d2d4: 1
< e2e4: 1
< f2f4: 1
< g2g4: 1
< h2h4: 1
< h1f1: 1
< h1g1: 1
< e1d1: 1
< e1f1: 1
< e1g1: 1
<
< Nodes searched: 21
<
(exit status 0)
```

</details>
