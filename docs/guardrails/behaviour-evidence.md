# Evidence of behaviour

This page explains what each check can show about the engine.

- **Independent expected values.** A separate source can prove a claim wrong.
- **Published perft.** Published values check the engine independently.
- **Comparison with plain minimax.** The search is compared with fisher's own plain minimax, kept in the tests of `search`, on the published positions, seeded walks and examples reasoned by hand. Seeded walks exercise cases nobody selected by hand.
- **UCI transcripts.** The wired binary has to satisfy the GUI contract.
- **Bench signature.** A refactor has a stable observation the owner can check.
- **Strength record.** The owner can trace an improvement claim to a match.

The first ten published mates run in `scripts/check`. The weekly published suite runs all 100 in release mode. Run [`scripts/check-published-suite`](../../scripts/check-published-suite) to repeat the suite locally.

The weekly published suite is independent of the merge gates. A failed comparison means `main` disagrees with a published value. GitHub notifies the account that last changed the workflow schedule if that account has Actions notifications enabled. No other action is taken on failure.

When fisher-4yn.2 adds pruning, retire the engine's four-ply node count comparison, which assumes the search visits every move. Keep the four-ply node count of the plain minimax in the tests of `search`: it shows that the expected scores come from a search that skips nothing.

## Real-clock match

The weekly [real-clock match](../../.github/workflows/clock-match.yml) plays 1,000 release games with one fisher binary as both engines at `10/1`. It pairs colors from the checked-in openings in order, uses half the runner's available cores rounded down (at least one), and gives fastchess a time margin of zero. It requires the exact 1,000 game IDs in the PGN and finish events, normal endings, legal replay with matching ply counts and results, and a completed `tournament_finished` event. The script accepts JSON events, the threefold repetition warning and the listed console formats. All other output fails the match. Review the listed formats whenever the fastchess pin changes.

The job runs Thursdays at 07:17 UTC and on manual dispatch, independently of the merge gates. GitHub notifies the account that last changed the schedule if that account has Actions notifications enabled. The match has a 3,600-second deadline within a 75-minute job. At the deadline the script interrupts fastchess to let it save the partial evidence before reporting failure. The artifact includes the PGN, log, command, fastchess version, runner details and summary.

To repeat it locally, install the [pinned fastchess release](../../scripts/install-fastchess) and install the replay library in a virtual environment with the same hash checks and build isolation settings as [the workflow](../../.github/workflows/clock-match.yml). Run [the match script](../../scripts/check-clock-match) with the absolute or relative path to fastchess and a new evidence directory.
