# Evidence of behaviour

This page explains what each check can show about the engine.

- **Independent expected values.** A separate source can prove a claim wrong.
- **Published perft.** Published values check the engine independently.
- **Seeded reference comparison.** Reachable positions exercise cases nobody selected by hand.
- **UCI transcripts.** The wired binary has to satisfy the GUI contract.
- **Bench signature.** A refactor has a stable observation the owner can check.
- **Strength record.** The owner can trace an improvement claim to a match.

The first ten published mates run in `scripts/check`. The weekly published suite runs all 100 in release mode. Run [`scripts/check-published-suite`](../../scripts/check-published-suite) to repeat the suite locally.

The weekly job blocks nothing. A failed comparison means `main` disagrees with a published value. GitHub notifies the account that last changed the workflow schedule if that account has Actions notifications enabled. No other action is taken on failure.

When fisher-4yn.2 adds pruning, retire the engine's four-ply node count comparison, which assumes the search visits every move, and keep the reference minimax comparison.
