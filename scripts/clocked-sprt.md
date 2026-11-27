# Play or replay a clocked SPRT

Put fastchess v1.8.3-alpha (`d7b47e8`) on `PATH` with
[`install-fastchess`](install-fastchess). Use a checkout containing both commits,
the Rust toolchains requested by those commits, and Python 3.9 or later.

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

The command plays and validates every game in the requested range. It reports
losses on time, crashes or stalls, and illegal moves over those games and any
earlier games supplied. Each such ending counts as the loss fastchess gives.
An incomplete range is void and exits with status 1.

Replay a range by supplying its original fastchess PGN and a fresh directory.
Replay downloads the same pinned opening book and reads the PGN results
without starting fastchess or either engine.

```sh
scripts/clocked-sprt --pgn /tmp/fisher-sprt/games.pgn -- \
  5c5d6cf 8b9c254 gain 1 20 /tmp/fisher-sprt-replay
```

The judge counts pairs in drawn order from opening 1. It stops counting at
the first threshold reached and reports the pair counts, logistic Elo with
its 95% interval, and log-likelihood ratio at that observation. The games
after it are kept. With neither threshold reached, a prefix shorter than
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
Remove the lines in [`series-openings.epd`](../strength/series-openings.epd)
to leave 148,932 positions.

Use the full lowercase measured commit, a colon, and the full lowercase
opponent commit as the seed. For each remaining line, remove its line feed
and hash the UTF-8 bytes of the seed, a colon, and the line with SHA-256.
Sort by ascending lowercase hexadecimal hash, using the line text to break
ties. Keep the first 10,000 lines, each followed by one line feed, including
the last. Their bytes are the opening order. A range takes the requested
lines from that order by their one-based line numbers.

The time control and opening-book pins live in [`clocked_sprt.py`](clocked_sprt.py).
The SPRT bounds, error rates and pair cap live in [`sprt_statistic.py`](sprt_statistic.py).
