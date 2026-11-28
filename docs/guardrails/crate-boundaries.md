# Crate boundaries

This page lists the permitted workspace dependencies and the interface checks that need human review.

The [boundary table](../../scripts/check-crate-boundaries) permits these dependencies.

- `fisher` → `bench`, `deepening`, `uci`
- `bench` → `deepening`, `rules`
- `deepening` → `rules`, `search`
- `uci` → `deepening`, `rules`
- `search` → `rules`
- `rules` and `xtask` have no workspace dependencies.

[`scripts/check-source-paths`](../../scripts/check-source-paths) checks rustc dep-info for Cargo workspace targets on the host with default features and with all features enabled. It resolves symlinks and assigns each input to its deepest enclosing package. A target can use inputs from its own package and its package's Cargo build-script output directory. The repository's `test-data/` and `expectations/` allow shared data reads. If either the recorded or resolved path is in a shared directory, neither path may end in `.rs`. Missing or invalid dep-info fails the check. The `--self-test` fixtures run in `scripts/check`.

Rustdoc has no stable dep-info output for doc examples. [`scripts/check-doctests`](../../scripts/check-doctests) requires doctests to be disabled for every workspace library target. CI builds from a clean target directory, so rustc regenerates dep-info even when a source edit preserved its original mtime.

A person reviews macro-generated exports and re-exports. The [crate interface rules](../../CODING_STANDARDS.md#crate-interfaces) explain what to look for.

- **Crate dependencies.** Each capability keeps a clear owner.
- **Deep modules.** Callers use a small, deliberate interface.
- **Development-only reference.** The reference can check engine behaviour independently of the engine.
