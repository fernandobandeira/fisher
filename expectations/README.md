# Expectations

Expected values that came from outside the implementer: published perft counts, solved test positions, examples the owner approved. Tests read these files. See [docs/evidence.md](../docs/evidence.md).

## Rules

- Every file starts with where its values came from (a link, a tool and its version) and the date they were taken.
- A value is committed before the code that has to meet it, in a commit of its own.
- The agent implementing a ticket does not edit this directory. If a value looks wrong, it stops and says why.
- This directory is a protected path: a commit that touches it needs a `Guard-Change:` line, and the human reads it.

For fisher-l7f, fisher-l7f.1 supplies the UCI transcripts before protocol code, and fisher-l7f.2 supplies the published perft counts before move generation.

## UCI transcripts

Each `uci/*.uci` path is one session with a fresh fisher process. Read
the records top to bottom. Blank lines and lines starting with `#` are
comments. The header gives the sources and date. A comment above a command
or reply names the passage or owner decision that supplies its expectation.

| Record | Meaning |
|---|---|
| `> text` | Send the text and flush standard input. Append LF. Preserve whitespace after the prefix. |
| `>CR text` | Send and flush as above. Append CR. |
| `< text` | Read the next standard-output line and compare its text exactly, excluding the line ending. |
| `! eof` | Close standard input. |
| `! exit` | Require clean process exit and no further standard output. If input is still open, keep it open until exit. |

In both send records, decode `\t` as TAB, `\r` as CR, `\\` as a backslash,
and `\xNN` as one byte written with two hexadecimal digits.

The following expected lines have special meanings. All other replies are exact.

| Expected line | Match |
|---|---|
| `< id name fisher {version}` | Substitute the version of the fisher binary under test. |
| `< info string {message}` | Match one `info string` line with any message. |
| `< info string {message:fragment}` | Match one `info string` line whose message contains the case-sensitive fragment. |
| `< bestmove {legal}` | Match one `bestmove` carrying a UCI move that an independent implementation lists as legal in the current position. |

For `{legal}`, the reference starts at the starting position and follows
accepted `position` commands and their move lists. A `bestmove` reply does
not itself advance the position. For `go infinite`, check `readyok` before
sending `stop` so an early `bestmove` fails by output order. A runner may
bound reads and process exit to detect a hang.

### Searched moves

`uci-search/*.uci` holds the expected sessions for fisher-2aj. The runner
will play them when fisher-2aj.4 adds search. Under rule 9, an accepted
`go infinite` produces its `info` line before the reply to `isready` and
holds `bestmove` until `stop`. The terminal `info` lines are examples
pending owner approval.

| Expected line | Match |
|---|---|
| `< info depth 4 score {score} nodes {nodes}` | Match one line with these fields in order and no extra fields. The score is `cp` followed by a decimal integer or `mate` followed by a nonzero decimal integer. The node count is a nonnegative decimal integer. |

Use this allowance only when the score and node count have no outside
expected values. The exact lines use the published starting-position
perft counts and positions checked independently with python-chess.

Run [equal-scores.uci](uci-search/equal-scores.uci) twice in fresh
processes. Require each `bestmove` to match `{legal}` and compare the two
actual UCI moves for equality, as required by rule 6. This comparison is
part of running that transcript and uses the existing `{legal}` record.

## Published perft

[`perft.epd`](perft.epd) holds one position per data line. Split at semicolons.
The first field is the full six-field FEN. Each remaining field has the form
`D<depth> <count>`, with both values written in decimal. Skip blank lines and
lines starting with `#`. Read counts as `u128` to accommodate depth 15.
Keep the records in published order, from the starting position through
Position 6, because the tests assign depths in that order.
The comments record the first depths at which the source reports en passant
captures, castling, and promotions.

## Published mate-in-two positions

[`mate-in-two.tsv`](mate-in-two.tsv) holds one puzzle per data line. Skip
blank lines and lines starting with `#`. Split each remaining line at
literal tabs into three fields in this order.

1. The case-sensitive Lichess puzzle ID.
2. The full six-field FEN after the first published move. The spaces inside
   the FEN are part of this field. The en passant square is written after
   a two-square pawn move even when no en passant capture is legal.
3. The published first move of the solution, written as a UCI move.

The records retain the order of the first 100 puzzles tagged `mateIn2`
in the export. In each position, exactly one legal move forces checkmate
within three plies, and no legal move gives checkmate in one ply.
