# Evidence of behaviour

This page explains what each check can show about the engine.

- **Independent expected values.** A separate source can prove a claim wrong.
- **Published perft.** Published values check the engine independently.
- **Seeded reference comparison.** Reachable positions exercise cases nobody selected by hand.
- **UCI transcripts.** The wired binary has to satisfy the GUI contract.
- **Bench signature.** A refactor has a stable observation the owner can check.
- **Strength record.** The owner can trace an improvement claim to a match.

The first ten published mates run in `scripts/check`. The weekly published suite runs all 100 in release mode. Run [`scripts/check-published-suite`](../../scripts/check-published-suite) to repeat the suite locally.

The weekly published suite is independent of the merge gates. A failed comparison means `main` disagrees with a published value. GitHub notifies the account that last changed the workflow schedule if that account has Actions notifications enabled. No other action is taken on failure.

When fisher-4yn.2 adds pruning, retire the engine's four-ply node count comparison, which assumes the search visits every move, and keep the reference minimax comparison.

## Real-clock match

The weekly [real-clock match](../../.github/workflows/clock-match.yml) plays 1,000 release games with one fisher binary as both engines at `10/1`. It pairs colors from the checked-in openings in order, uses half the runner's available cores rounded down (at least one), and gives fastchess a time margin of zero. It requires 1,000 normal endings and a legal PGN replay. It counts all warnings and accepts only `PV continues after threefold repetition`.

The job runs Thursdays at 07:17 UTC and on manual dispatch, independently of the merge gates. GitHub notifies the account that last changed the schedule if that account has Actions notifications enabled. The PGN and fastchess log are attached to the run.

To repeat it locally, install the [pinned fastchess release](../../scripts/install-fastchess) and the [pinned replay library](../../scripts/clock-match-requirements.txt). Run [the match script](../../scripts/check-clock-match) with the absolute or relative path to fastchess and a new evidence directory.
