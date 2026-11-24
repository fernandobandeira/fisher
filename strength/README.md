# Playing strength

A record of how strongly each revision of fisher plays. Every match is played under the same conditions, so a change to search or evaluation shows up as a change in the numbers.

This is a record, not a gate. `scripts/check` does not run it, and nothing here is an expected value. A claim that a change plays stronger is still settled by SPRT with bounds fixed beforehand, as [docs/evidence.md](../docs/evidence.md) says. The ticket for that is `fisher-op4`.

## What is here

| Path | What it holds |
|---|---|
| `records.tsv` | One line per match: the two revisions that played, and the result. |
| `measure` | Plays one match under the conditions below and prints its line for `records.tsv`. |
| `summarise.py` | Counts the results in a match's PGN. `measure` runs it. |
| `openings.epd` | The 200 positions every match starts from. |

## Taking a measurement

Check out the commit to measure. Then play it against Stockfish, against an earlier revision of fisher, or both:

```sh
strength/measure stockfish /tmp/fisher-against-stockfish
strength/measure dbe9113 /tmp/fisher-against-dbe9113
```

The second argument is a new directory for the games. `measure` plays 400 games, prints the result, and ends with one tab-separated line. Add that line to `records.tsv` and commit it.

`measure` builds each fisher from its commit, not from the working files, so the hashes in the line name exactly the engines that played. Uncommitted changes are not measured. It refuses to print a line if the match did not produce 400 games.

To see whether a change helped, measure the new revision against the last revision in `records.tsv`.

`measure` needs [fastchess](https://github.com/Disservin/fastchess), Python 3 with `pip install chess==1.11.2`, and Stockfish for a match against it. Set `PYTHON` to use another interpreter.

## What a record says

Each line of `records.tsv` describes one match, from the measured revision's point of view.

| Field | Meaning |
|---|---|
| `date` | The day the match was played. |
| `revision` | The commit of fisher that was measured. |
| `opponent` | The commit of fisher it played, or the name Stockfish gives itself over UCI and its `UCI_Elo`. |
| `conditions` | Which set of conditions below the match was played under. Only records with the same number can be compared. |
| `games` | Games played: 400. |
| `won`, `drawn`, `lost` | Results for the measured revision. |
| `score` | The share of the points it took, in percent. A draw is half a point. |
| `queen_ahead_won`, `queen_ahead_drawn`, `queen_ahead_lost` | Results of the games in which it was a queen ahead. |
| `abnormal_endings` | Games whose PGN `Termination` tag is not `normal`: an engine crashed, played an illegal move or lost on time. Either engine may be at fault; the PGN says which. |
| `elo_difference`, `elo_interval` | The Elo difference fastchess computes from the score, and the margin it prints after `+/-`. Positive means the measured revision scored more than half the points. fastchess prints `inf` or `nan` when the score is too one-sided to compute them. |
| `runner` | The version of fastchess that ran the match. |

A revision was a queen ahead in a game if its material minus the opponent's was at least 900 in two positions in a row, so that the lead outlasted a reply. Material is counted as fisher's evaluation counts it: pawn 100, knight 300, bishop 300, rook 500, queen 900. The queen-ahead games it drew or lost are the games it failed to finish. The count does not say why: it may have been unable to give checkmate, or it may have blundered later.

`summarise.py` replays the moves in the PGN with python-chess and stops if one is illegal. That checks the PGN, not the engines: when an engine plays an illegal move, fastchess ends the game, leaves the move out of the PGN, and the game is counted in `abnormal_endings`.

## Conditions

### 1

- 400 games: each of the 200 positions in `openings.epd` once with each color, in the order of the document.
- A game ends by checkmate, stalemate, threefold repetition, the fifty-move rule or insufficient material, or abnormally when an engine crashes, plays an illegal move or runs out of time. No game is adjudicated on its score, and no engine resigns.
- fisher gets a clock of 600 seconds plus 10 a move and no other limit. It ignores the clock (`fisher-7vo`) and searches to its own fixed depth. The clock is there so that a fisher that hangs loses the game instead of stalling the match.
- Stockfish runs with `UCI_LimitStrength` on and `UCI_Elo` 1320, the lowest it allows, on one thread with a 16 MB hash and a clock of 10 seconds plus 0.1 a move.
- Eight games run at a time. Stockfish is on a clock, so a busy machine weakens it; run nothing else heavy during a match against it.

The conditions do not fix the Stockfish version or the machine. Compare two records against Stockfish only if `opponent` reads the same in both, and treat records from different machines with care.

When fisher obeys the clocks these conditions stop making sense, because a build would then use ten minutes a game. Write the new conditions here as 2, change `conditions` in `measure`, and measure the last revision again under them so the two series join.

## How far to trust the numbers

- **Between two revisions of fisher that search**, a match should repeat: from `71a4608` on, fisher plays the same move from the same position every time, as the `equal-scores` transcript checks for one position. Nobody has yet played a whole match twice to confirm it. The score is then a fact about these 200 openings, and says nothing about others. Revisions before `71a4608` choose a legal move at random, so a match against one of them varies.
- **Against Stockfish** the result varies from match to match, because Stockfish at limited strength chooses weaker moves on purpose and not always the same ones. The margin fastchess prints gives a rough idea of how much; it is not a promise about the next match.
- **An Elo for fisher** is not something this record establishes. Adding the difference against Stockfish to 1320 gives a rough placement only: it leans on one opponent, and for a fisher far below 1320 it is an extrapolation. Stockfish's documentation says `UCI_Elo` was calibrated at 120 seconds plus 1 a move and anchored to the CCRL 40/4 list; here it gets 10 seconds plus 0.1.
- **A score near 0% or 100%** says little about the size of the gap. Measure against a closer opponent instead.

## Where the openings come from

`openings.epd` holds 200 positions from `noob_3moves.epd` in [official-stockfish/books](https://github.com/official-stockfish/books) at commit `37751b0`, which that repository publishes under CC0. Each is a position after three moves by each color. fastchess drew them at random with `-srand 20261123` for the first match, before any result was known, and they are kept in the order it drew them.
