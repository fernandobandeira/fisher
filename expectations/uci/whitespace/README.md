# Whitespace between the tokens of a command

Staged by fisher-82i on 2026-11-26 for fisher-b0b. No test reads these
sessions yet: the transcript runner selects sessions by name directly in
`expectations/uci/`, and this folder sits below it. fisher-b0b activates them.

| Session | What it covers |
|---|---|
| [`no-break-space-command.uci`](no-break-space-command.uci) | U+00A0 stays inside the command token, so the command is unknown and fisher stays silent. |
| [`tab-command.uci`](tab-command.uci) | Horizontal tab separates tokens: `isready<TAB>x` gives one `readyok`. |
| [`tab-then-isready.uci`](tab-then-isready.uci) | Horizontal tab separates tokens and does not end the command: `isready<TAB>isready` gives one `readyok`, not two. |
| [`vertical-tab-command.uci`](vertical-tab-command.uci) | Vertical tab separates tokens: `isready<VT>x` gives one `readyok`. |
| [`vertical-tab-then-isready.uci`](vertical-tab-then-isready.uci) | Vertical tab separates tokens and does not end the command: `isready<VT>isready` gives one `readyok`, not two. |
| [`no-break-space-in-fen.uci`](no-break-space-in-fen.uci) | U+00A0 joins the en passant field to the halfmove clock. fisher rejects the en passant field and keeps the previous position. |

Each command session runs in a fresh process and ends with a strict exit,
so a missing `readyok` fails at the read and an extra one fails at the exit.
The two probes of each separator sit in separate sessions, so a missing reply
to one cannot make up for an extra reply to the other.

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
- A rejected position gives one `info string` diagnostic that describes the
  cause and leaves the previous position in place
  ([`rejected-position.uci`](../rejected-position.uci)). The cause here is
  the en passant field, which fisher's existing FEN validation rejects when it
  is neither `-` nor a square. No committed expectation or recorded owner
  decision fixes the words of the diagnostic for this cause, so the session
  uses the plain matcher `info string {message}`, as `rejected-position.uci`
  does for a cause with no fixed fragment. Stockfish accepts the malformed FEN
  instead: its clock display shows `0 1` whatever follows the
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

Input, each separator inside one command, reproduced on 2026-11-27 with the
same binary:

```json
"isready\tisready\nquit\n"
```

```text
69 73 72 65 61 64 79 09 69 73 72 65 61 64 79 0a 71 75 69 74 0a
```

and

```json
"isready\u000bisready\nquit\n"
```

```text
69 73 72 65 61 64 79 0b 69 73 72 65 61 64 79 0a 71 75 69 74 0a
```

Stdout, identical for both: one `readyok`, so the second `isready` is an
argument of the first command.

```text
Stockfish 19 by the Stockfish developers (see AUTHORS file)
readyok
```

Control with an ordinary space before the clocks:

```json
"position fen 4k3/8/8/8/8/8/8/4K2R w K - 7 42\nd\nquit\n"
```

gives the same output except `Fen: 4k3/8/8/8/8/8/8/4K2R w K - 7 42`.

## The bytes

The session files are plain ASCII. Each special byte is written with the
escapes of the [send record](../../README.md#uci-transcripts): `\xc2\xa0` for
U+00A0, `\t` for horizontal tab and `\x0b` for vertical tab. No editor or
formatter can normalise them, and `LC_ALL=C grep -c $'[\x80-\xff\x09\x0b]' *.uci`
counts 0 in every file. Decoded, the sends that hold a special byte are these
bytes, which are the same bytes the Stockfish probes sent. The other sends
are plain ASCII text followed by LF.

```text
no-break-space-command.uci   > isready\xc2\xa0x
69 73 72 65 61 64 79 c2 a0 78 0a

tab-command.uci              > isready\tx
69 73 72 65 61 64 79 09 78 0a

tab-then-isready.uci         > isready\tisready
69 73 72 65 61 64 79 09 69 73 72 65 61 64 79 0a

vertical-tab-command.uci     > isready\x0bx
69 73 72 65 61 64 79 0b 78 0a

vertical-tab-then-isready.uci > isready\x0bisready
69 73 72 65 61 64 79 0b 69 73 72 65 61 64 79 0a

no-break-space-in-fen.uci    > position fen 4k3/8/8/8/8/8/8/4K2R w K -\xc2\xa00 1
70 6f 73 69 74 69 6f 6e 20 66 65 6e 20 34 6b 33 2f 38 2f 38 2f 38 2f 38
2f 38 2f 38 2f 34 4b 32 52 20 77 20 4b 20 2d c2 a0 30 20 31 0a

no-break-space-in-fen.uci    > position fen 4k3/8/8/8/8/8/8/4K2R w K -\xc2\xa07 42
70 6f 73 69 74 69 6f 6e 20 66 65 6e 20 34 6b 33 2f 38 2f 38 2f 38 2f 38
2f 38 2f 38 2f 34 4b 32 52 20 77 20 4b 20 2d c2 a0 37 20 34 32 0a
```

## What the sessions tell apart

Checked on 2026-11-27 with a one-off player and scripted engines that were
not kept. A session either passes, or fails at the reply or exit named.

| Engine | No-break space command | `isready<TAB>x` | `isready<TAB>isready` | `isready<VT>x` | `isready<VT>isready` | FEN |
|---|---|---|---|---|---|---|
| fisher 0.1.0 at `064a672` | Fails: extra `readyok` | Passes | Passes | Passes | Passes | Fails: no diagnostic |
| Scripted, Stockfish's separators and fisher's rules | Passes | Passes | Passes | Passes | Passes | Passes |
| Scripted, as above with other diagnostic words | Passes | Passes | Passes | Passes | Passes | Passes |
| Scripted, splits at U+00A0 | Fails | Passes | Passes | Passes | Passes | Fails |
| Scripted, tab not a separator | Passes | Fails: no `readyok` | Fails: no `readyok` | Passes | Passes | Passes |
| Scripted, tab ends the command | Passes | Passes | Fails: extra `readyok` | Passes | Passes | Passes |
| Scripted, vertical tab not a separator | Passes | Passes | Passes | Fails: no `readyok` | Fails: no `readyok` | Passes |
| Scripted, vertical tab ends the command | Passes | Passes | Passes | Passes | Fails: extra `readyok` | Passes |
| Scripted, accepts the joined field | Passes | Passes | Passes | Passes | Passes | Fails: no diagnostic |
| Scripted, rejects but replaces the position | Passes | Passes | Passes | Passes | Passes | Fails: searches the new position |

## What the sessions do not show

- **The checkers.** The sessions drive the engine. All six can pass while the
  transcript runner still splits at U+00A0 and tracks the joined command as
  the valid FEN `4k3/8/8/8/8/8/8/4K2R w K - 7 42`: the expected replies are
  literal terminal replies, so the runner never consults its tracked
  position. The history checker in `crates/uci/tests/` is outside these
  sessions altogether. fisher-b0b is about the engine and both checkers
  agreeing, so on activation it must add a test that fails if either checker
  is left on its old splitting, using the same bytes as these sessions: the
  no-break space in a command and in the FEN, and the tab and vertical-tab
  controls. These sessions do not supply that test.
- **The exact position kept.** Checkmate shows that the previous position is
  still searched, not that it is unchanged. An engine that rejects the
  command but writes the new counters (`7 42`, or `0 1`) into the previous
  position gives the same replies. Under the committed rules no UCI reply
  depends on the halfmove clock or the fullmove number: at `064a672` only
  `rules` and unit tests read them. fisher-b0b should therefore compare the
  full position passed to the search, counters included, with the previous
  position after each rejection.
