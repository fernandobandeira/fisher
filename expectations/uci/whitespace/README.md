# Whitespace between the tokens of a command

Staged by fisher-82i on 2026-11-26 for fisher-b0b. No test reads these
sessions yet: the transcript runner selects sessions by name directly in
`expectations/uci/`, and this folder sits below it. fisher-b0b activates them.

| Session | What it covers |
|---|---|
| [`no-break-space-command.uci`](no-break-space-command.uci) | U+00A0 stays inside the command token, so the command is unknown and fisher stays silent. |
| [`tab-command.uci`](tab-command.uci) | Horizontal tab separates tokens: one `readyok`. |
| [`vertical-tab-command.uci`](vertical-tab-command.uci) | Vertical tab separates tokens: one `readyok`. |
| [`no-break-space-in-fen.uci`](no-break-space-in-fen.uci) | U+00A0 joins the en passant field to the halfmove clock. fisher rejects the en passant field and keeps the previous position. |

Each command session runs in a fresh process and ends with a strict exit,
so a missing `readyok` fails at the read and an extra one fails at the exit.

## Source

The separators come from
[Stockfish 19 (sf_19)](https://github.com/official-stockfish/Stockfish/releases/tag/sf_19),
revision `edb0d9db6731067ec50ce619ff372b463bc4dd5d`. The binary was the
Homebrew arm64 Stockfish 19 bottle at `/opt/homebrew/bin/stockfish`
(`/opt/homebrew/Cellar/stockfish/19/bin/stockfish`, SHA-256
`186aaee236524939aeae9d0f0019fbecbfc954c6ed3bdedfb641075b1045d5c0`).

- [`src/uci.cpp`, lines 94–113](https://github.com/official-stockfish/Stockfish/blob/edb0d9db6731067ec50ce619ff372b463bc4dd5d/src/uci.cpp#L94-L113)
  (`UCIEngine::loop`) reads the command token with `operator>>`. Under the C
  locale it separates tokens at space, horizontal tab, LF, vertical tab, form
  feed and CR, and not at the UTF-8 bytes `C2 A0` of U+00A0.
- [`src/uci.cpp`, lines 498–522](https://github.com/official-stockfish/Stockfish/blob/edb0d9db6731067ec50ce619ff372b463bc4dd5d/src/uci.cpp#L498-L522)
  (`UCIEngine::position`) reads the FEN tokens the same way.
- [`src/position.cpp`, lines 420–433](https://github.com/official-stockfish/Stockfish/blob/edb0d9db6731067ec50ce619ff372b463bc4dd5d/src/position.cpp#L420-L433)
  (`Position::set`) reads the halfmove clock and fullmove number without
  requiring the extraction to succeed.

What fisher answers where Stockfish's spelling differs comes from fisher's
existing rules, as fisher-b0b requires:

- An unknown command is ignored without output
  ([`unknown-input.uci`](../unknown-input.uci), fisher-l7f). Stockfish prints
  `Unknown command: ...` instead.
- A rejected position gives one `info string` diagnostic and leaves the
  previous position in place ([`rejected-position.uci`](../rejected-position.uci)).
  The diagnostic must contain `invalid en passant square`, the message of
  fisher's existing FEN validation for an en passant field that is neither
  `-` nor a square (`rules::Position::from_fen`). Stockfish accepts the
  malformed FEN instead: its clock display shows `0 1` whatever follows the
  no-break space, because the clock extraction fails silently. That display is
  an observation of how Stockfish handles this malformed FEN, not a value
  fisher has to match.
- A checkmated position answers `go` with `info depth 0 score mate 0` and
  `bestmove 0000` ([`terminal-positions.uci`](../terminal-positions.uci),
  owner approval on fisher-2aj.2).

## Stockfish 19 probes

Reproduced on 2026-11-26 with the binary above. They match the record on
fisher-82i and fisher-b0b, except that the record trims the trailing space
Stockfish writes after `Checkers:`; it is shown here as `\x20`. Each probe
started a fresh process. Inputs are JSON strings: decode the escapes and send
the UTF-8 bytes, including the final LF. In the output the no-break space
Stockfish echoes back is shown as `\xc2\xa0`.

Input

```json
"isready x\nisready\tx\nisready\u000bx\nquit\n"
```

Stdout

```text
Stockfish 19 by the Stockfish developers (see AUTHORS file)
Unknown command: 'isready\xc2\xa0x'. Type help for more information.
readyok
readyok
```

Input

```json
"position fen 4k3/8/8/8/8/8/8/4K2R w K - 0 1\nd\nquit\n"
```

and

```json
"position fen 4k3/8/8/8/8/8/8/4K2R w K - 7 42\nd\nquit\n"
```

Stdout, identical for both

```text
Stockfish 19 by the Stockfish developers (see AUTHORS file)

 +---+---+---+---+---+---+---+---+
 |   |   |   |   | k |   |   |   | 8
 +---+---+---+---+---+---+---+---+
 |   |   |   |   |   |   |   |   | 7
 +---+---+---+---+---+---+---+---+
 |   |   |   |   |   |   |   |   | 6
 +---+---+---+---+---+---+---+---+
 |   |   |   |   |   |   |   |   | 5
 +---+---+---+---+---+---+---+---+
 |   |   |   |   |   |   |   |   | 4
 +---+---+---+---+---+---+---+---+
 |   |   |   |   |   |   |   |   | 3
 +---+---+---+---+---+---+---+---+
 |   |   |   |   |   |   |   |   | 2
 +---+---+---+---+---+---+---+---+
 |   |   |   |   | K |   |   | R | 1
 +---+---+---+---+---+---+---+---+
   a   b   c   d   e   f   g   h

Fen: 4k3/8/8/8/8/8/8/4K2R w K - 0 1
Key: 32F04C409787442D
Checkers:\x20
```

Control with an ordinary space before the clocks:

```json
"position fen 4k3/8/8/8/8/8/8/4K2R w K - 7 42\nd\nquit\n"
```

gives the same output except `Fen: 4k3/8/8/8/8/8/8/4K2R w K - 7 42`.

## fisher's existing diagnostic

fisher 0.1.0 at `e088fec` cannot yet be given the joined field, because it
splits at U+00A0. The same FEN with the stand-in field `-x0` (and `-x7`)
reaches the same check in `rules::Position::from_fen`:

```text
> position fen 4k3/8/8/8/8/8/8/4K2R w K -x0 1
< info string position rejected: invalid FEN: invalid en passant square
> position fen 4k3/8/8/8/8/8/8/4K2R w K -x7 42
< info string position rejected: invalid FEN: invalid en passant square
```

## The bytes

The session files are plain ASCII. Each special byte is written with the
escapes of the [send record](../../README.md#uci-transcripts): `\xc2\xa0` for
U+00A0, `\t` for horizontal tab and `\x0b` for vertical tab. No editor or
formatter can normalise them, and `LC_ALL=C grep -c $'[\x80-\xff\x09\x0b]' *.uci`
counts 0 in every file. Decoded, the sends that carry them are these bytes,
which are the same bytes the Stockfish probes sent:

```text
no-break-space-command.uci   > isready\xc2\xa0x
69 73 72 65 61 64 79 c2 a0 78 0a

tab-command.uci              > isready\tx
69 73 72 65 61 64 79 09 78 0a

vertical-tab-command.uci     > isready\x0bx
69 73 72 65 61 64 79 0b 78 0a

no-break-space-in-fen.uci    > position fen 4k3/8/8/8/8/8/8/4K2R w K -\xc2\xa00 1
70 6f 73 69 74 69 6f 6e 20 66 65 6e 20 34 6b 33 2f 38 2f 38 2f 38 2f 38
2f 38 2f 38 2f 34 4b 32 52 20 77 20 4b 20 2d c2 a0 30 20 31 0a

no-break-space-in-fen.uci    > position fen 4k3/8/8/8/8/8/8/4K2R w K -\xc2\xa07 42
70 6f 73 69 74 69 6f 6e 20 66 65 6e 20 34 6b 33 2f 38 2f 38 2f 38 2f 38
2f 38 2f 38 2f 34 4b 32 52 20 77 20 4b 20 2d c2 a0 37 20 34 32 0a
```

## What the sessions tell apart

Checked on 2026-11-26 with a one-off player and scripted engines that were
not kept.

| Engine | No-break space command | Tab | Vertical tab | FEN |
|---|---|---|---|---|
| fisher 0.1.0 at `e088fec` | Fails: extra `readyok` before exit | Passes | Passes | Fails: no diagnostic |
| Scripted, Stockfish's separators and fisher's rules | Passes | Passes | Passes | Passes |
| Scripted, splits at U+00A0 | Fails | Passes | Passes | Fails |
| Scripted, tab not a separator | Passes | Fails: no `readyok` | Passes | Passes |
| Scripted, vertical tab not a separator | Passes | Passes | Fails: no `readyok` | Passes |
| Scripted, accepts the joined field | Passes | Passes | Passes | Fails: no diagnostic |
| Scripted, rejects but replaces the position | Passes | Passes | Passes | Fails: searches the new position |
