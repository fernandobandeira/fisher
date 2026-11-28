# Coding standards

Most of the standards are not written here. They are the formatter, the lints in `Cargo.toml` and `clippy.toml`, the glossary lint, the tests and the crate boundaries, and `scripts/check` runs all of them. A reviewer does not comment on anything those already enforce.

## Deferred-work comments

When deferring work, create a ticket and leave `// TODO(fisher-<id>): <what is missing>` where the work belongs. See [tracker commands](docs/agents/issue-tracker.md). If the work has no place in the code yet, create the ticket alone.

`scripts/check` runs `cargo xtask todos`. It requires that form with a nonblank description in Rust comments under `crates/` and the equivalent `# TODO(fisher-<id>): <what is missing>` in shell and Python comments under `scripts/` and `strength/`. Dotted child IDs are accepted. Bare or malformed `TODO` markers and `FIXME` markers fail.

## Standards that are still prose

A rule lands here only while it waits for a check, or because it cannot be one.

### Rust compilation and lints

`scripts/check` uses `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings` for the compile and lint gate. It checks libraries, binaries and test targets with optional features enabled. The test step also exercises the default feature configuration. Workspace crates deny compiler warnings in both configurations. The pinned Clippy toolchain also denies cognitive complexity above 8. Split a function at a meaningful boundary and keep its behavior pinned by the existing tests.

### Crate interfaces

A deep module gives callers a small interface for a complete job. In Fisher, `Position::legal_moves` lets a caller ask what it can play. The rules crate owns board storage, castling rights and checks for an attacked king. A change to any of those details belongs together in `rules`. [Matt Pocock on deep modules](https://www.youtube.com/watch?v=uC44zFz7JSM)

- Use `src/lib.rs` as the single public entry point for each library crate. Declare implementation modules with `mod` and explicitly re-export the operations and types callers need. Other crates import those names from the crate root.
- Start types, fields and helpers private. Use the narrowest visibility that reaches the caller. When proposing an export, name the external caller and the job it needs to do, or the agreed ticket that will need it. Revisit an export when its last caller disappears.
- Keep each capability with its owner. Before adding a crate dependency, identify the operation the caller needs. Keep shared work behind that operation so callers can use it without assembling the same internal steps themselves.
- Exercise public behavior through the same interface the application uses. An integration test alone is insufficient reason to export an implementation detail. Private tests can live beside the code. Keep tests of a public library interface in the crate that owns it. Tests that exercise the wired binary through UCI stay in `fisher`.

`scripts/check-crate-boundaries` checks declared workspace dependencies, including build, test, optional and target-specific ones. Its dependency table is the one list of permitted directions. The independent reference, `shakmaty`, is a development dependency only. The same dependency table applies to integration tests. When proposing a new direction, change the dependency table and its `--self-test` fixtures in the same script.

`cargo xtask interfaces` checks library roots for public module declarations. Rust also checks unnecessarily public items inside private modules with `unreachable_pub`. Review macro-generated exports and re-exports for exposed implementation modules. A reviewer checks that each public item has a caller.

### The binary

`fisher` is the binary, and it holds no logic. Its `main.rs` wires the library crates together: it hands standard input, standard output and the search to the crate that runs the session, and returns what that crate returns. Whatever parses, decides, searches or formats lives in a library crate, where the build enforces its boundary and its tests reach it through the interface the binary uses.

Since fisher-5sg.11, `uci` runs the session and `deepening` runs the search for a `go`. This rule waits for a check (fisher-u8o).

### Dependency sources and advisories

`cargo deny --locked --offline check -D warnings advisories sources` checks dependencies with all workspace features enabled, including development dependencies. Warnings fail the dependency check. Resolve a new advisory by updating or removing the affected dependency. Any exception needs human review of the reason and the protected `deny.toml` change.

`scripts/check` refreshes the RustSec advisory database separately. It verifies the checkout against its Git tree and checks its origin against `deny.toml`. If the refresh fails, it reports the snapshot date and requires a commit at most seven days old. Set `CARGO_NET_OFFLINE=true` to use cached dependencies and a usable snapshot without attempting a refresh. Run `cargo deny --locked fetch db` while online to populate the database. CI restores the database from a daily cache and only `main` saves it after a successful refresh. The weekly Check run requires a successful refresh.

### Closing deferred-work tickets

Close a ticket only when no `TODO` still names it. Automating this rule is tracked by fisher-xcl.

### Flaky tests

A flaky test is no test. When a check fails intermittently, fix its cause in the next change, even if that change did not cause it. Never rerun to green and move on.

Every wait in a test is a hang timeout, never an expected duration. Use one shared hang timeout constant per test crate. Keep expected durations on a controlled clock. Order events with a signal or a controlled clock, never a sleep. Claim speed with measurements outside `scripts/check` (fisher-5sg.4, D30).

A check could flag literal wait durations, sleeps in test code, and assertions on wall-clock elapsed time. Review still has to judge what orders events and whether an intermittent failure was fixed at its cause.

### Mutation testing and region coverage

Run [`scripts/check-mutants`](scripts/check-mutants) for the whole workspace. Each mutant runs only the unit and integration tests of its own crate. Pull requests test the merge commit, pushes to `main` test the pushed commit, and the weekly run tests the scheduled revision. Every run rejects stale entries in the [reviewed registry](.cargo/mutation-gate.json) and runs the listed mutants. Add a test for a surviving mutant or propose a proof of equivalence. Each proof records a digest of its enclosing function and needs review when that function changes.

Run [`scripts/check-coverage`](scripts/check-coverage) to measure each crate in fresh, separate build and profile directories. Compare that crate's source regions with its [coverage floor](.cargo/coverage-floors.json) using only its unit and integration tests. Put dedicated test-only sources in `tests.rs` or beneath `tests/` inside `src/` and start each with `#![cfg(test)]`. Declare out-of-line test modules as `mod tests` with their submodules beneath them. Coverage excludes these dedicated files. Inline test code in mixed source files still contributes regions. Restore lost coverage with tests. The two full tools run outside `scripts/check`.

## From review comment to check

A comment becomes a rule on its second independent occurrence, or on the first defect that mattered. The owner can also approve a check as baseline policy. Either route requires a bad example and a legitimate near miss.

Get both examples from the reviewer or derive them from the owner's approved policy. Pick the cheapest check that tells them apart.

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
