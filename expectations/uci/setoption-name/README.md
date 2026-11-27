# Any word in the name position of setoption

Staged by fisher-38w on 2026-11-27, before the engine changes, by an author
who will not change it. The owner decided on fisher-38w (2026-11-27) that
fisher follows Stockfish 19: the token in the name position of `setoption`
is consumed whatever it is, and the option name is read after it.

fisher-38w activated the four sessions in pull request #125, moving them
unchanged into `expectations/uci/setoption-name/`. Transcript tests in
`crates/fisher/tests/transcripts.rs` play each one as `setoption-name/<name>`.

| Session | What it covers |
|---|---|
| [`any-word.uci`](any-word.uci) | `setoption foo MultiPV value 2` sets MultiPV to 2, silently. |
| [`no-break-space-name.uci`](no-break-space-name.uci) | `setoption <U+00A0>name MultiPV value 2` sets MultiPV to 2, silently. U+00A0 stays inside the token, so the word in the name position is not `name`. |
| [`value-in-name-position.uci`](value-in-name-position.uci) | `setoption value MultiPV value 2` sets MultiPV to 2, silently. A `value` in the name position is consumed like any other word and does not end the name. |
| [`missing-name-word.uci`](missing-name-word.uci) | `setoption MultiPV value 2` consumes `MultiPV`, reads an empty name, and leaves MultiPV at 1, silently. |

Each session runs in a fresh process, so MultiPV starts at its default of 1.
Each sends `isready` straight after `setoption` and requires one `readyok`
and nothing else, then searches `startpos` at depth 1 and requires exactly
one searched `info` line per variation, then a strict exit.

## Source

[Stockfish 19 (sf_19)](https://github.com/official-stockfish/Stockfish/releases/tag/sf_19),
revision `edb0d9db6731067ec50ce619ff372b463bc4dd5d`. The binary was the
Homebrew arm64 Stockfish 19 bottle at `/opt/homebrew/bin/stockfish`
(`/opt/homebrew/Cellar/stockfish/19/bin/stockfish`, SHA-256
`186aaee236524939aeae9d0f0019fbecbfc954c6ed3bdedfb641075b1045d5c0`). Its
`uci` reply begins `id name Stockfish 19`.

[`src/ucioption.cpp`, lines 43–60](https://github.com/official-stockfish/Stockfish/blob/edb0d9db6731067ec50ce619ff372b463bc4dd5d/src/ucioption.cpp#L43-L60)
(`OptionsMap::setoption`):

```cpp
is >> token;  // Consume the "name" token

// Read the option name (can contain spaces)
while (is >> token && token != "value")
    name += (name.empty() ? "" : " ") + token;
```

Where Stockfish's output differs, fisher keeps its existing rules:

- Stockfish prints `No such option: ` for the empty name of
  `missing-name-word.uci`. fisher ignores `setoption` for an undeclared name
  without output (fisher-8hc answer 4,
  [`multipv/option-values.uci`](../multipv/option-values.uci)).
- Stockfish's searched lines carry `seldepth`, `hashfull` and `tbhits` and
  real scores. The sessions use fisher's spelling (fisher-r6y D41) and its
  depth-1 scores from `startpos`, all `cp 0` by material, as
  [`multipv/settings.uci`](../multipv/settings.uci) does. What the
  sessions take from Stockfish is the number of variations: two after the
  three accepted commands, one after the missing name word.

## Stockfish 19 probes

Run on 2026-11-27 with the binary above. Each probe started a fresh process
and sent the command, then `isready`, `position startpos` and `go depth 1`,
waited one second and sent `quit`. The hex is the bytes sent before `quit`.
Stockfish's `info string` lines about processors, threads and the network
are left out below; they came after `readyok` in every probe.

```text
setoption foo MultiPV value 2
73 65 74 6f 70 74 69 6f 6e 20 66 6f 6f 20 4d 75 6c 74 69 50 56 20 76 61
6c 75 65 20 32 0a 69 73 72 65 61 64 79 0a 70 6f 73 69 74 69 6f 6e 20 73
74 61 72 74 70 6f 73 0a 67 6f 20 64 65 70 74 68 20 31 0a

Stockfish 19 by the Stockfish developers (see AUTHORS file)
readyok
info depth 1 seldepth 2 multipv 1 score cp 30 nodes 39 nps 39000 hashfull 0 tbhits 0 time 1 pv d2d4
info depth 1 seldepth 2 multipv 2 score cp 25 nodes 39 nps 39000 hashfull 0 tbhits 0 time 1 pv c2c4
bestmove d2d4
```

```text
setoption \xc2\xa0name MultiPV value 2
73 65 74 6f 70 74 69 6f 6e 20 c2 a0 6e 61 6d 65 20 4d 75 6c 74 69 50 56
20 76 61 6c 75 65 20 32 0a 69 73 72 65 61 64 79 0a 70 6f 73 69 74 69 6f
6e 20 73 74 61 72 74 70 6f 73 0a 67 6f 20 64 65 70 74 68 20 31 0a

Stockfish 19 by the Stockfish developers (see AUTHORS file)
readyok
info depth 1 seldepth 2 multipv 1 score cp 30 nodes 39 nps 39000 hashfull 0 tbhits 0 time 1 pv d2d4
info depth 1 seldepth 2 multipv 2 score cp 25 nodes 39 nps 39000 hashfull 0 tbhits 0 time 1 pv c2c4
bestmove d2d4
```

```text
setoption value MultiPV value 2

Stockfish 19 by the Stockfish developers (see AUTHORS file)
readyok
info depth 1 seldepth 2 multipv 1 score cp 30 nodes 39 nps 39000 hashfull 0 tbhits 0 time 1 pv d2d4
info depth 1 seldepth 2 multipv 2 score cp 25 nodes 39 nps 39000 hashfull 0 tbhits 0 time 1 pv c2c4
bestmove d2d4
```

```text
setoption MultiPV value 2

Stockfish 19 by the Stockfish developers (see AUTHORS file)
No such option:\x20
readyok
info depth 1 seldepth 2 multipv 1 score cp 30 nodes 20 nps 20000 hashfull 0 tbhits 0 time 1 pv d2d4
bestmove d2d4
```

`No such option: ` ends in a space, shown as `\x20`. Control: `setoption
name MultiPV value 2` gave the same output as the first probe.

## What the sessions tell apart

Checked on 2026-11-27 by copying the sessions into `uci/setoption-name/`
of a throwaway checkout of `9772649`, registering them, and running the
transcript tests. Nothing from that checkout was kept.

| Engine | `any-word` | `no-break-space-name` | `value-in-name-position` | `missing-name-word` |
|---|---|---|---|---|
| fisher at `9772649` (requires `name`) | Fails: `bestmove` where `multipv 2` is expected | Fails: same | Fails: same | Passes |
| `9772649` with any word consumed in the name position | Passes | Passes | Passes | Passes |
| As above, but a `value` in the name position ends the name | Passes | Passes | Fails: `bestmove` where `multipv 2` is expected | Passes |

With any word consumed, all 87 transcript tests passed, the four
sessions included.

## What the sessions do not show

- **The checkers.** fisher-38w asks the engine, the transcript runner and
  the UCI history checker to agree. These sessions drive the engine through
  the transcript runner. The history test
  `setoption_consumes_any_token_in_the_name_position` checks the same
  commands during a search, when a recognized option requires a busy
  diagnostic.
- **Names of more than one word.** MultiPV is fisher's only declared option
  and has a one-word name, so no session reads a name across several tokens.
