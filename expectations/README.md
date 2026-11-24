# Expectations

Expected values that came from outside the implementer: published perft counts, solved test positions, examples the owner approved. Tests read these files. See [docs/evidence.md](../docs/evidence.md).

## Rules

- Every file starts with where its values came from (a link, a tool and its version) and the date they were taken.
- A value is committed before the code that has to meet it, in a commit of its own.
- The agent implementing a ticket does not edit this directory. If a value looks wrong, it stops and says why.
- This directory is a protected path: a commit that touches it needs a `Guard-Change:` line, and the human reads it.

fisher-l7f.1 supplied the original UCI transcripts before protocol code, and
fisher-l7f.2 supplied the published perft counts before move generation.
The 15 searched transcripts that expect an `info` line come from fisher-2aj.2.

## UCI transcripts

The current UCI transcripts live in `uci/`. Each `uci/*.uci` path describes one
session with a fresh fisher process. Read the records top to bottom. Blank
lines and lines starting with `#` are comments. The header gives the sources
and date. A comment above a command or reply names the passage or owner
decision that supplies its expectation.

| Record | Meaning |
|---|---|
| `> text` | Send the text and flush standard input. Append LF. Preserve whitespace after the prefix. |
| `>CR text` | Send and flush as above. Append CR. |
| `< text` | Read the next standard-output line and compare its text exactly, excluding the line ending. |
| `<* text` | Consume zero or more consecutive matching searched `info` lines before the next reply or clean exit. See [Repeated searched info](#repeated-searched-info). |
| `! eof` | Close standard input. |
| `! exit` | Require clean process exit and no further standard output. If input is still open, keep it open until exit. |

In both send records, decode `\t` as TAB, `\r` as CR, `\\` as a backslash,
and `\xNN` as one byte written with two hexadecimal digits.

The following expected lines have special meanings. All other responses are exact.

| Expected line | Match |
|---|---|
| `< id name fisher {version}` | Substitute the version of the fisher binary under test. |
| `< info string {message}` | Match one `info string` line with any message. |
| `< info string {message:fragment}` | Match one `info string` line whose message contains the case-sensitive fragment. |
| `< bestmove {legal}` | Match one `bestmove` carrying a UCI move that an independent implementation lists as legal in the current position. |
| `< info depth 4 multipv 1 score {score} nodes {nodes} nps {nps} time {time} pv {pv}` | Match one searched `info` line against the [allowance pattern](#searched-moves). |
| `< info depth 4 score {score} nodes {nodes}` | Used only by the scripted runner fixtures. Match one short searched `info` line with the S and N forms of [Repeated searched info](#repeated-searched-info). |

For `{legal}`, the reference starts at the starting position and follows
accepted `position` commands and their move lists. A `bestmove` reply does
not itself advance the position. For `go infinite`, check `readyok` before
sending `stop` so an early `bestmove` fails by output order. A runner may
bound reads and process exit to detect a hang.

### Searched moves

Under rule 9 of fisher-2aj, an accepted `go infinite` produces its `info`
line before the reply to `isready` and holds `bestmove` until `stop`. The
owner approved both terminal `info` lines on 2026-11-23 in a comment on
fisher-2aj.2.

Run [equal-scores.uci](uci/equal-scores.uci) twice in fresh
processes. Require each `bestmove` to match `{legal}` and compare the two
actual UCI moves for equality, as required by rule 6. This comparison is
part of running that transcript and uses the existing `{legal}` record.
Run [material-zero.uci](uci/material-zero.uci) once to check the
exact score for a position with no captures within four plies.

The depth-four searched lines in `uci/` use fisher-r6y D41 and Stockfish sf_19
(edb0d9d), read on 2026-11-24.
See [`UCIEngine::on_update_full`](https://github.com/official-stockfish/Stockfish/blob/edb0d9d/src/uci.cpp#L654)
and [`SearchManager::output_pv`](https://github.com/official-stockfish/Stockfish/blob/edb0d9d/src/search.cpp#L2273).

```text
< info depth 4 multipv 1 score {score} nodes {nodes} nps {nps} time {time} pv {pv}
```

Match the entire output line after removing its line ending. With every
allowance in place, use this anchored regular expression. Require exact
text for any literal score or node count. Only `position-moves.uci` uses
the score and node allowances together.

```regex
^info depth 4 multipv 1 score (?:cp (?:0|-?[1-9][0-9]*)|mate -?[1-9][0-9]*) nodes (?:0|[1-9][0-9]*) nps (?:0|[1-9][0-9]*) time [1-9][0-9]* pv [a-h][1-8][a-h][1-8][qrbn]?(?: [a-h][1-8][a-h][1-8][qrbn]?){0,3}$
```

`multipv` is the literal 1. Time is a whole number of milliseconds of
at least 1 with no leading zero. The principal variation contains one to
four UCI moves separated by single spaces. Each move contains two squares
and an optional lowercase promotion suffix. The move pattern excludes
`0000`. Digits in every allowance are ASCII. The runner checks the nodes
per second from the reported nodes and time and checks the principal
variation under D5.

### Staged iterative deepening transcripts

fisher-5sg.13 staged the [successors in `uci-iterative-deepening/`](uci-iterative-deepening/)
on 2026-11-24 for fisher-5sg.4. The current runner reads only `uci/`.
Each staged file with the same name succeeds its active file. The `go depth`,
`go nodes` and `go movetime` cases of `go-parameters.uci` each start a fresh
session in `go-depth.uci`, `go-nodes.uci` and `go-movetime.uci`. They use
the default depth of four until their limits are implemented.

Each completed depth has its own required reply with a literal depth.
For depth D, the node total adds the counts of every search from depth 1
through D. Each individual search counts the nodes at plies 1 through its
depth. For example, the starting position reports 20, 440, 9762 and 216365
nodes. The three score allowances in `position-moves.uci` retain `{score}`
at every depth and use independently enumerated node totals. The derivation
script and its results are on fisher-5sg.13.

Keep the two fresh processes and compare their moves when adopting
`equal-scores.uci`. The `terminal-positions.uci` successor retains one
depth-zero line for each position. The `go-infinite.uci` successor stages
the bare searches in that session and retains the held searches for
fisher-5sg.4 to handle under the separate `go infinite` rules.

### Repeated searched info

Source: fisher-r6y D15 and fisher-5sg decision 1, approved by the owner on
2026-11-24. See the [runner fixtures](uci-runner/repeated-searched-info.md).

After the literal prefix `<* `, use the full spelling under D41. The
scripted fixtures also use a short spelling. The spelling in the record
selects the grammar for every match.

```text
info depth D multipv 1 score S nodes N nps R time T pv P
info depth D score S nodes N
```

The fixtures also use the short spelling in ordinary `< ` responses,
where it accepts allowances at literal depth 4.

fisher-8hc.6 chose the verdicts of every fixture, and fisher-8hc.7
implements the checks. Five fixtures reject one fault of the full
spelling each: `time-zero`, `missing-multipv`, `multipv-two`, `old-order`
(nodes, time and nps out of Stockfish's order), and `nps-mismatch` (an nps
that is not 1000 times the nodes divided by the time, rounded down).

Match the entire line, excluding its ending, with single ASCII spaces.
Each slot accepts its corresponding allowance or a literal of the same
form. A literal requires an exact text match. Transcript authors supply
no regular expressions.

| Slot | Allowance | Form of literal and text matched by the allowance |
|---|---|---|
| D | `{depth}` | Positive ASCII decimal, `[1-9][0-9]*`. |
| S | `{score}` | `cp 0`, `cp` followed by a signed decimal, or `mate` followed by a signed decimal. Here a signed decimal is `-?[1-9][0-9]*`. |
| N | `{nodes}` | Nonnegative ASCII decimal, `0\|[1-9][0-9]*`. |
| T | `{time}` | Positive ASCII decimal, `[1-9][0-9]*`. |
| R | `{nps}` | Same decimal form as N. |
| P | `{pv}` | One or more UCI moves separated by single spaces, each spelled `[a-h][1-8][a-h][1-8][qrbn]?`. |

Under fisher-r6y D5, the principal variation is as long as the depth
searched, shorter only when it ends in checkmate or stalemate. The
repetition checks that P contains at most D moves and checks the spelling
of each move. With literal D and P, check the bound at load time. With
`{depth}`, check it against each output line.

Reject unknown or misplaced allowances and malformed patterns at load
time, including when stdout would supply zero matches. For example,
`<* info depth {depth} score mate -1 nodes {nodes}` is valid.
`<* info depth 0 score cp 0` is invalid because it reports a terminal
position at depth zero, outside either positive-depth searched spelling.
`<* info depth 1 score banana nodes {nodes}` is invalid because the score
literal has neither score form.

The next record must be an ordinary `< ` reply or `! exit`, ignoring
comments and blank lines. A send (`> ` or `>CR `), `! eof` (even if
followed by `! exit`), end of transcript, or another `<* ` at that
position is a load-time error. The fixtures show each placement.

Execute all preceding sends before starting the observation. Consume
consecutive matches until the first nonmatch, then hand that same line
to the ordinary reply comparison. EOF before a required reply fails.
For `! exit`, the first nonmatch must be stdout EOF and the process must
exit cleanly. The allowance ends at its delimiter. An early `bestmove`,
malformed `info`, or unsolicited diagnostic still reaches the ordinary
comparison or exit check.

Matching is greedy. If the next ordinary reply can also match the
repetition, the repetition consumes that line. To require a depth report
that can match the repetition, put the mandatory reply before the
repetition. For example, require depth 1, allow depths 2 through 4 to
match a following `{depth}` repetition, then require `readyok`. A stream
with depths 1 through 4 followed by `readyok` passes in that order.

Start one absolute five-second read bound when each observation starts.
It covers all matches and the delimiter read, including stdout EOF for
`! exit`. Matching lines do not renew it. The process exit-status wait
keeps its own separate five-second bound, which starts at stdout EOF,
as it does for `! exit` today.

To check silence after a command, synchronize with a search or held move
and use a strict exit. A repetition before exit alone cannot establish
when buffered output was written.

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
