# Evidence of behaviour

This page explains what each check can show about the engine.

- **Independent expected values.** A separate source can prove a claim wrong.
- **Published perft.** Published values check the engine independently.
- **Comparison with plain minimax.** The search is compared with fisher's own plain minimax, kept in the tests of `search`, on the published positions, seeded walks and examples reasoned by hand. Seeded walks exercise cases nobody selected by hand.
- **UCI transcripts.** The wired binary has to satisfy the GUI contract.
- **Bench signature.** A refactor has a stable observation the owner can check.
- **Strength record.** The owner can trace an improvement claim to a match.

The first ten published mates run in `scripts/check`. The weekly published suite runs in release mode and checks deep perft counts, all 100 published mates, and plain minimax's four-ply node counts against sums of published perft counts. Run [`scripts/check-published-suite`](../../scripts/check-published-suite) to repeat the suite locally.

Alpha-beta pruning retired the production search's exhaustive four-ply node-count comparison.

The suite also compares fisher's search with its test-only plain minimax on the six perft positions, the last position of each seeded walk, and all 100 published mate positions. It compares scores and best moves and checks bounds on the node count. Plain minimax checks that the score from the root's perspective holds after each move of the principal variation. A disagreement with plain minimax can occur even when the published checks pass. Independently, shakmaty replays the principal variation to verify that each move is legal and that the ending supports the reported score. On the mate positions, the suite also checks that the best move is the published move.

On the same positions, the suite checks that the interruptible search returns the same result as the ordinary search, including the node count, when its callback always returns `false`. This comparison can fail even when the ordinary search agrees with both the published values and plain minimax.

The weekly published suite is independent of the merge gates. GitHub notifies the account that last changed the workflow schedule if that account has Actions notifications enabled. No other action is taken on failure.

## Real-clock match

The weekly [real-clock match](../../.github/workflows/clock-match.yml) plays 1,000 release games with one fisher binary as both engines at `10/1`. It pairs colors from the checked-in openings in order, uses half the runner's available cores rounded down (at least one), and gives fastchess a time margin of zero. A pass requires the exact 1,000 game IDs in the PGN and finish events, normal endings, and legal replay with matching ply counts and results. Each PGN record needs an explicit `Result` header and accepts the pinned writer's movetext. This consists of move numbers, standard algebraic notation for legal moves, brace comments and one matching final result. A console finish must agree with the JSON finish on the result. A loss on time, illegal move or abnormal reason fails the match even if the JSON termination says `normal`.

### Recognized JSON events

`tournament_started`, `game_started`, `move`, `game_finished`, `tournament_finished`, and `game_aborted`.

A `game_aborted` event fails the match and counts as an interruption. If its reason records a loss on time, it also counts as a loss on time. The other five kinds can contribute to a pass. There must be exactly one tournament finish, with reason `completed`.

### Diagnostics and evidence

The two admitted warnings cover a principal variation that continues after a draw by threefold repetition or the fifty-move rule. Each warning must match the pinned formatter and be followed immediately by `Info;`, `Position;`, and `Moves;`, in order. The position must be a valid ASCII FEN that round-trips through the reference parser. The pinned logger emits the group without colour escapes. An empty history is `Moves;` followed by one ASCII space. Unknown output fails the match. Review the listed formats whenever the fastchess pin changes. [Pinned fastchess formatter](https://github.com/Disservin/fastchess/blob/d7b47e8dec6b564aaf4cbb2922ab37e6c6761e75/app/src/matchmaking/match/match.cpp#L785-L809)

The job runs Thursdays at 07:17 UTC and on manual dispatch, independently of the merge gates. GitHub notifies the account that last changed the schedule if that account has Actions notifications enabled. The match has a 3,600-second play deadline. At the deadline the script interrupts fastchess, gives it ten seconds to save partial evidence, and kills any processes still in its group, including when fastchess has already exited. A local Ctrl-C also kills the group. The 100-minute job allows 15 minutes for setup, 75 for build, play, cleanup and replay, and three for upload even after failure, with seven minutes to spare. The artifact includes the PGN, log, command, fastchess version, runner details and summary.

After a loss on time, fastchess sends `stop` to a started engine and makes a second read of up to ten seconds if the first read yielded no usable `bestmove`. What consumed the full allowance in individual games remains unknown. [Pinned timeout handler](https://github.com/Disservin/fastchess/blob/d7b47e8dec6b564aaf4cbb2922ab37e6c6761e75/app/src/matchmaking/match/match.cpp#L738-L763) · [Delayed run](https://github.com/fernandobandeira/fisher/actions/runs/45683651168)

To repeat it locally, install the [pinned fastchess release](../../scripts/install-fastchess) and install the replay library in a virtual environment with the same hash checks and build isolation settings as [the workflow](../../.github/workflows/clock-match.yml). Run [the match script](../../scripts/check-clock-match) with the absolute or relative path to fastchess and a new evidence directory.
