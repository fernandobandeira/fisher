# Guardrails

A guardrail catches a failure or makes it visible to the owner. Before merging, the change must pass the required checks and receive the review required by its tier.

- [Checks before handoff and merge](#checks-before-handoff-and-merge) — Catch failures before work is committed, handed back or merged.
- [Code shape](#code-shape) — Make Rust code easier to read and check.
- [Crate boundaries](#crate-boundaries) — Give each capability a clear owner behind a small, deliberate interface.
- [Vocabulary](#vocabulary) — Use names that mean the same thing across crates.
- [Evidence of behaviour](#evidence-of-behaviour) — Test claims about the engine against independent evidence.
- [Strength of the tests](#strength-of-the-tests) — Make each crate’s unit and integration tests defend its own code.
- [Dependencies](#dependencies) — Check third-party dependencies even when an implementation change is small.
- [Guarding the guards](#guarding-the-guards) — Make changes to what passes easy for the owner to find and review.
- [Review](#review) — Focus the owner’s review on claims, interfaces and guards.
- [Deferred guardrails](#deferred-guardrails) — Revisit checks when their adoption triggers occur.

## Checks before handoff and merge

Catch failures before work is committed, handed back or merged.

The `main gates` ruleset requires the Actions `check`, `mutation`, and `coverage` jobs to pass and the pull request to be up to date with `main`.

| Guardrail | What it stops | Where enforced | When |
|---|---|---|---|
| CI | Missing shared check results | [Check workflow](../../.github/workflows/check.yml) | All PRs, pushes to `main`, weekly |
| Pre-commit hook | Committing after a failed local check | [Pre-commit hook](../../.githooks/pre-commit) | Every commit with the hooks installed |
| Claude Code stop hook | Ending a turn with unchecked changes | [Stop hook](../../.claude/hooks/stop-check) | End of a dirty Claude Code turn |

See [merge gates](merge-gates.md) for the branch protections.

## Code shape

Make Rust code easier to read and check.

| Guardrail | What it stops | Where enforced | When |
|---|---|---|---|
| Formatter | Layout drift | [`cargo fmt --all -- --check`](../../scripts/check) | Each commit and CI run |
| Workspace lint inheritance | A crate omitting the workspace lints | [`scripts/check`](../../scripts/check) checks each manifest | Each commit and CI run |
| Compiler and Clippy lints | Discarded results, panic shortcuts, unchecked conversions and catch-all match arms | [Workspace lints](../../Cargo.toml) via [`scripts/check`](../../scripts/check) | Each commit and CI run |
| Forbidden `unsafe` | Code that bypasses Rust's safety checks | [Workspace lint](../../Cargo.toml) and [ADR](../adr/0001-rust-with-unsafe-forbidden.md) | Each commit and CI run |
| Reasoned lint exceptions | Silent suppression with `#[allow]` | [Workspace lints](../../Cargo.toml), [AGENTS.md](../../AGENTS.md#guards) | Each commit and CI run |
| Cognitive complexity | Functions that exceed a complexity of 8 | [Workspace lint](../../Cargo.toml), [threshold](../../clippy.toml) | Each commit and CI run |
| `unreachable_pub` | Unnecessarily public items inside private modules | [Workspace lint](../../Cargo.toml) | Each commit and CI run |

See [code shape](code-shape.md) for what each check contributes.

## Crate boundaries

Give each capability a clear owner behind a small, deliberate interface.

| Guardrail | What it stops | Where enforced | When |
|---|---|---|---|
| Crate dependencies | Edges outside the allowed directions | [Boundary table](../../scripts/check-crate-boundaries) | Each commit and CI run |
| Deep modules | Direct public module declarations at the library root and alternate library entry paths | [`cargo xtask interfaces`](../../crates/xtask/src/interfaces.rs), [`lib.rs` path check](../../scripts/check-crate-boundaries), [interface rules](../../CODING_STANDARDS.md#crate-interfaces) | Each commit and CI run. Macro-generated exports and re-exports in review |
| Development-only reference | Shipping the independent chess reference as an engine dependency | [`shakmaty` dependency check](../../scripts/check-crate-boundaries) | Each commit and CI run |

See [crate boundaries](crate-boundaries.md) for the permitted dependencies and interface review.

## Vocabulary

Use names that mean the same thing across crates.

| Guardrail | What it stops | Where enforced | When |
|---|---|---|---|
| Glossary | Banned wording in identifiers | [Glossary](../../GLOSSARY.md) and [`cargo xtask glossary`](../../crates/xtask/src/glossary.rs) | Each commit and CI run |

## Evidence of behaviour

Give each claim about what the engine does an observation that could prove it wrong, from a source other than the implementer.

| Guardrail | What it stops | Where enforced | When |
|---|---|---|---|
| Independent expected values | Tests that attest to the implementer's own output | [`expectations/`](../../expectations/README.md), [evidence rules](../evidence.md) | Tests each commit and CI run. Provenance in review |
| Published perft | Incorrect legal move counts | [Expected counts](../../expectations/perft.epd), [rules](../../crates/rules/tests/positions.rs), [search](../../crates/search/tests/search.rs) | Shallow in `scripts/check`. Deep perft and both four-ply node tests run in the weekly published suite |
| Weekly published suite | Differences from published counts or mates going unnoticed | [Published suite workflow](../../.github/workflows/published-suite.yml) | Weekly and on manual dispatch |
| Weekly real-clock match | Losses on time, abnormal endings, illegal or incomplete PGN records, missing match completion and unrecognized fastchess output | [Real-clock match workflow](../../.github/workflows/clock-match.yml) | Weekly and on manual dispatch |
| Seeded reference comparison | Differences in legal moves, resulting positions, check detection or search results | [Rules comparison](../../crates/rules/tests/positions.rs), [search comparison](../../crates/search/tests/search.rs) | In `scripts/check`, including the first ten published mates. All 100 run in the weekly published suite |
| UCI transcripts | Incorrect responses, process exits or search reports | [Transcript runner](../../crates/fisher/tests/transcripts.rs), [expected transcripts](../../expectations/uci) | In `scripts/check` |
| Bench signature | A changed fixed-search node count | [Signature test](../../crates/bench/tests/signature.rs), [`fisher bench` and commit record](../process.md#recording-the-bench-signature) | Signature test each commit and CI run. Commit record in review |
| Strength record | Unsupported claims about playing strength | [`strength/measure` and records](../../strength/README.md) | When claiming a strength change |

See [evidence of behaviour](behaviour-evidence.md) for what each check shows and when to run the full mate suite.

See [independent expected values](expected-values.md) for how those values are chosen and where they stop applying.

## Strength of the tests

Make each crate’s unit and integration tests defend its own code.

| Guardrail | What it stops | Where enforced | When |
|---|---|---|---|
| Test ownership | A caller’s tests supplying the evidence for a crate | [Mutation](../../scripts/check-mutants) and [coverage](../../scripts/check-coverage). The [dependency check](../../scripts/check-crate-boundaries) enforces allowed edges | Mutation and coverage on each CI run. Dependencies in `scripts/check` |
| Mutation testing | A crate’s own tests passing despite a behaviour change | [`scripts/check-mutants`](../../scripts/check-mutants), [reviewed equivalents](../../.cargo/mutation-gate.json) | Whole workspace on each CI run |
| Per-crate region coverage | Losing coverage supplied by a crate’s own tests | [`scripts/check-coverage`](../../scripts/check-coverage), [reviewed floors](../../.cargo/coverage-floors.json) | Each CI run |

See [mutation testing](mutation.md) for what a surviving mutant shows and why the job runs across the workspace. See [coverage](coverage.md) for floors and uncovered regions.

## Dependencies

Check third-party dependencies even when an implementation change is small.

| Guardrail | What it stops | Where enforced | When |
|---|---|---|---|
| Dependency advisories and sources | Known vulnerable dependencies and unauthorized sources | [`cargo deny`](../../scripts/check), [configuration](../../deny.toml) | Each commit and CI run |

## Guarding the guards

Make changes to what passes easy for the owner to find and review.

| Guardrail | What it stops | Where enforced | When |
|---|---|---|---|
| Protected paths and `Guard-Change:` | Changes to checking policy without a stated reason | [Protected paths](../../.githooks/protected-paths), [commit-message hook](../../.githooks/commit-msg), [Guard-Change workflow](../../.github/workflows/guard-change.yml) | Commits with the hooks installed, pull requests into `main` (including retargeting), and pushes to `main` |

See [the `Guard-Change:` line](guard-change.md) for how the hook works and where it stops.

## Review

Focus the owner’s review on claims, interfaces and guards.

| Guardrail | What it stops | Where enforced | When |
|---|---|---|---|
| Review and tiers | Merging behaviour or commitments without the required review | [Process tiers](../process.md#tiers) and [review](../process.md#what-the-human-reads-in-order) | Prose only |
| Ticketed TODO | Deferred work with no owner-visible record | [Deferred work rules](../../CODING_STANDARDS.md#deferred-work) | Prose only |

See [review](review.md) for how review tiers and ticketed work help the owner.

## Deferred guardrails

The [deferred checks](deferred.md) pair each guardrail fisher has considered with the change that would make it worth adopting, drawing on the [fuzzing and formal verification research](research/fuzzing-and-formal-verification.md).
