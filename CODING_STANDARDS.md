# Coding standards

Most of the standards are not written here. They are the formatter, the lints in `Cargo.toml` and `clippy.toml`, the glossary lint, the tests and the crate boundaries, and `scripts/check` runs all of them. A reviewer does not comment on anything those already enforce.

## Standards that are still prose

A rule lands here only while it waits for a check, or because it cannot be one.

### Rust compilation and lints

`scripts/check` uses `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings` for the compile and lint gate. It checks libraries, binaries and test targets with optional features enabled. The test step also exercises the default feature configuration.

### Crate interfaces

A deep module gives callers a small interface for a complete job. In Fisher, `Position::legal_moves` lets a caller ask what it can play. The rules crate owns board storage, castling rights and checks for an attacked king. A change to any of those details belongs together in `rules`. [Matt Pocock on deep modules](https://www.youtube.com/watch?v=uC44zFz7JSM)

- Use `src/lib.rs` as the single public entry point for each library crate. Declare implementation modules with `mod` and explicitly re-export the operations and types callers need. Other crates import those names from the crate root.
- Start types, fields and helpers private. Use the narrowest visibility that reaches the caller. When proposing an export, name the external caller and the job it needs to do, or the agreed ticket that will need it. Revisit an export when its last caller disappears.
- Keep each capability with its owner. Before adding a crate dependency, identify the operation the caller needs. Keep shared work behind that operation so callers can use it without assembling the same internal steps themselves.
- Exercise public behavior through the same interface the application uses. An integration test alone is insufficient reason to export an implementation detail. Private tests can live beside the code.

`scripts/check-crate-boundaries` checks declared workspace dependencies, including build, test, optional and target-specific ones. The permitted directions are `fisher → uci`, `fisher → rules` and `uci → rules`. The independent reference, `shakmaty`, is a development dependency only. When proposing a new direction, change the dependency table and its examples together.

`cargo xtask interfaces` checks library roots for public module declarations. Rust also checks unnecessarily public items inside private modules with `unreachable_pub`. Review macro-generated exports and re-exports for exposed implementation modules. A reviewer checks that each public item has a caller.

### Dependency sources and advisories

`cargo deny --locked check advisories sources` checks dependencies with all workspace features enabled, including development dependencies. Review changes to `deny.toml` alongside the dependency that needs them.

### Deferred work

A `TODO` and a ticket always come as a pair, so the code shows what is missing and the tracker shows where it is followed.

- When you defer something, create a ticket for it and leave `// TODO(fisher-<id>): <what is missing>` at the place in the code where the work would go. See [tracker commands](docs/agents/issue-tracker.md). Deferred work that has no place in the code yet has the ticket alone.
- Every `TODO` in the code names its ticket in that form. A bare `// TODO` is not allowed.
- A ticket is closed only when no `TODO` still names it.

The first line cannot be a check. The other two wait for one.

## From review comment to check

A comment becomes a rule on its second independent occurrence, or on the first defect that mattered. Before that it is one person's remark about one change.

To promote one, get two examples from the reviewer: a bad one, and a legitimate near miss that must keep passing. Then pick the cheapest check that tells them apart:

| The comment is about | It becomes |
|---|---|
| Layout | A formatter setting |
| A local pattern in the code | A clippy lint, or a small lint in `crates/xtask` |
| A value or combination that should not exist | A type or a constructor that cannot build it |
| What the code does | A test, with its expected value chosen as [docs/evidence.md](docs/evidence.md) says |
| Which code may use which | A crate boundary: the dependency is absent from `Cargo.toml`, or the item is not exported |
| A word | A `_Banned in code_` line in `GLOSSARY.md` |
| A trade-off that is right here and wrong elsewhere | An ADR in `docs/adr/` |
| A preference nobody would block a merge over | Nothing. Drop it. |

A check's failure message says what to do instead. An agent reads the message, not this file.

## Keeping the rule set small

- Every custom rule has both examples as fixtures, so it can be changed safely.
- Retire a rule the toolchain now covers, and one that has not fired in a long time.
- A rule that keeps collecting `#[expect]` or `glossary-allow` exceptions is wrong. Fix the rule or delete it.
- Adding, loosening or removing a check is a change to a protected path. The human reads it.
