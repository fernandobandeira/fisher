# Crate boundaries

This page lists the permitted workspace dependencies and the interface checks that need human review.

The [boundary table](../../scripts/check-crate-boundaries) permits these dependencies.

- `fisher` → `bench`, `deepening`, `uci`
- `bench` and `deepening` → `rules`, `search`
- `uci` → `deepening`, `rules`
- `search` → `rules`
- `rules` and `xtask` have no workspace dependencies.

A person reviews macro-generated exports and re-exports. The [crate interface rules](../../CODING_STANDARDS.md#crate-interfaces) explain what to look for.

- **Crate dependencies.** Each capability keeps a clear owner.
- **Deep modules.** Callers use a small, deliberate interface.
- **Development-only reference.** The reference can check engine behaviour independently of the engine.
