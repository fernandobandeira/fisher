# Merge gates

This page explains what GitHub blocks on `main`.

GitHub blocks red and out-of-date pull requests, force-pushes to `main`, and deletion of `main`. Nobody is on a bypass list. See the [merge process](../process.md#pull-requests).

- **CI.** The owner can inspect check results from a fresh checkout.
- **Pre-commit hook.** A failed local check catches a broken intermediate commit before Git records it.
- **Claude Code stop hook.** The agent gets a chance to fix a failed check before handing the work back.
