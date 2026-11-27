# Play or replay a clocked SPRT

Put fastchess v1.8.3-alpha (`d7b47e8`) on `PATH` with
[`install-fastchess`](install-fastchess). The command checks the version
before building either engine. Use a checkout containing both commits, the
Rust toolchains requested by those commits, and Python 3.9 or later.

```sh
scripts/clocked-sprt --concurrency 2 -- 5c5d6cf 8b9c254 gain 1 20 /tmp/fisher-sprt
```

Give the measured revision, the opponent revision, `gain` or `no-loss`, the
first opening (starting at 1), the number of pairs, and a new directory.
Concurrency gives the number of games at a time and defaults to one. The
command builds each revision in an exported snapshot. Both engines get 10
seconds plus 0.1 a move, with no score or move-count adjudication, tablebase
adjudication or resignation. The directory keeps the binaries, opening
order, requested openings, fastchess log and PGN.

The command plays and validates every game in the requested range. A loss on
time, crash or stall, or illegal move counts as the loss fastchess gives. It
prints counts for the pairs used by the judge and for all games read, including
any supplied earlier games.

| Status | Meaning | Output |
|---|---|---|
| 0 | Pass, fail, no verdict yet, or inconclusive | `Result: …` on stdout |
| 1 | A game is missing or unreadable after play starts or while reading kept games | `Result: void (…)` on stdout |
| 2 | Play was refused before asking fastchess for games, or preflight failed | `Refused: …` on stderr |
| 3 | An unforeseen error | `Unexpected error (…): …` on stderr |

An interruption before play is a refusal. An interruption after fastchess has
started play gives `Result: void (interrupted)`. Only statuses 0 and 1 report
a result for a play or replay request. `--help` prints usage information.

Replay a range by supplying its original fastchess PGN and a fresh directory.
Replay downloads the same pinned published openings and reads the PGN results
without starting fastchess or either engine.

```sh
scripts/clocked-sprt --pgn /tmp/fisher-sprt/games.pgn -- \
  5c5d6cf 8b9c254 gain 1 20 /tmp/fisher-sprt-replay
```

Add `--opponent-view` when replaying to score the points the original opponent
took. Keep the revision arguments in the order used to play the games. The
output identifies this reading and names both original roles. This option
requires `--pgn` and applies to `--previous-pgn` when supplied.

The judge counts pairs in drawn order from opening 1. It stops counting at
the first threshold reached and reports the pair counts, logistic Elo with
its 95% interval when the pair scores vary, and log-likelihood ratio at that
observation. The games after it are kept. With neither threshold reached, a prefix shorter than
10,000 pairs gives no verdict yet. A full prefix gives inconclusive.

For a later range, use `--previous-pgn` with all the games before its first
opening. PGNs from successive ranges can be concatenated with blank lines
between records. A standalone later range can collect games, but counts zero
pairs until the prefix is supplied. A prefix with a verdict can be replayed.

## Draw the openings

Download [`noob_3moves.epd.zip` at `37751b08db745aa02babfa74056dd1168b9da803`](https://github.com/official-stockfish/books/blob/37751b08db745aa02babfa74056dd1168b9da803/noob_3moves.epd.zip).
Verify archive SHA-256
`68fe2691e0876d0de54fffb68c797e752138d5ed363beab2ac6ca34be68ee9f3`
and extracted `noob_3moves.epd` SHA-256
`2011193b4854e9a8cfdc05312ca2dbaffa6ceae3abbdee20e2ead2a18a603347`.
Verify [`series-openings.epd`](../strength/series-openings.epd) against
`df2c41c8b88e220050d5de4462b4838f6eaff1a809f4d6f159860624fee7555a`.
Remove its lines to leave 148,932 positions.

Use the full lowercase measured commit, a colon, and the full lowercase
opponent commit as the seed. For each remaining line, remove its line feed
and hash the UTF-8 bytes of the seed, a colon, and the line with SHA-256.
Sort by ascending lowercase hexadecimal hash, using the line text to break
ties. Keep the first 10,000 lines, each followed by one line feed, including
the last. Their bytes are the opening order. A range takes the requested
lines from that order by their one-based line numbers.

The time control and pins for fastchess and the opening positions live in
[`clocked_sprt.py`](clocked_sprt.py).
The SPRT bounds, error rates and pair cap live in [`sprt_statistic.py`](sprt_statistic.py).
