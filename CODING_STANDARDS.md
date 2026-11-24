# Coding standards

Most of the standards are not written here. They are the formatter, the lints in `Cargo.toml` and `clippy.toml`, the glossary lint, the tests and the crate boundaries, and `scripts/check` runs all of them. A reviewer does not comment on anything those already enforce.

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

`cargo deny --locked check -D warnings advisories sources` checks dependencies with all workspace features enabled, including development dependencies. Warnings fail the dependency check. Resolve a new advisory by updating or removing the affected dependency. Any exception needs human review of the reason and the protected `deny.toml` change.

### Deferred work

A `TODO` and a ticket always come as a pair, so the code shows what is missing and the tracker shows where it is followed.

- When you defer something, create a ticket for it and leave `// TODO(fisher-<id>): <what is missing>` at the place in the code where the work would go. See [tracker commands](docs/agents/issue-tracker.md). Deferred work that has no place in the code yet has the ticket alone.
- Every `TODO` in the code names its ticket in that form. A bare `// TODO` is not allowed.
- A ticket is closed only when no `TODO` still names it.

The first line cannot be a check. The other two wait for one.

### Mutation testing and region coverage

Run [`scripts/check-mutants --in-diff origin/main`](scripts/check-mutants) to exercise changes from the merge base to HEAD, or run [`scripts/check-mutants`](scripts/check-mutants) for the whole workspace. Tests run across the workspace. Add a test for a surviving mutant or propose a proof of equivalence in the [reviewed registry](.cargo/mutation-gate.json). On pull requests, cargo-mutants selects mutants whose spans intersect the diff. This includes function-wide replacements that cover a changed line. Changes beneath a crate’s `tests/` directory, in dedicated test-only source files, or in Rust source containing test attributes in either revision make the gate mutate the whole crate. The weekly run rejects stale entries and checks the reviewed mutants again. Each proof records a digest of its enclosing function and needs review when that function changes.

Run [`scripts/check-coverage`](scripts/check-coverage) to compare each crate's source regions with its [coverage floor](.cargo/coverage-floors.json). Put dedicated test-only sources in `tests.rs` or beneath `tests/` inside `src/` and start each with `#![cfg(test)]`. Declare out-of-line test modules as `mod tests` with their submodules beneath them. Coverage excludes these dedicated files. Inline test code in mixed source files still contributes regions. Restore lost coverage with tests. The two full tools run outside `scripts/check`.

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
