# Guardrails

A guardrail catches a failure or makes it visible to the owner. The `main gates` ruleset requires the Actions `check`, `mutation`, and `coverage` jobs to pass and the pull request to be up to date with `main`. The change also needs the review required by its tier.

## Merge gates

Catch failed checks before a commit or handoff so the author can fix them. Give the owner shared results from a fresh checkout before merge.

| Guardrail | What it stops | Where enforced | When |
|---|---|---|---|
| CI | Missing shared check results | [Check workflow](../../.github/workflows/check.yml) | All PRs, pushes to `main`, weekly |
| Pre-commit hook | Committing after a failed local check | [Pre-commit hook](../../.githooks/pre-commit) | Every commit with the hooks installed |
| Claude Code stop hook | Ending a turn with unchecked changes | [Stop hook](../../.claude/hooks/stop-check) | End of a dirty Claude Code turn |

See [merge gates](merge-gates.md) for the branch protections.

## Code shape

Apply the same checks to every crate so reviews can focus on behaviour, Rust’s safety guarantees hold, and important branches stay readable. Handle errors and new cases explicitly. Give the owner a reason for each lint exception and let Clippy flag obsolete expectations.

| Guardrail | What it stops | Where enforced | When |
|---|---|---|---|
| Formatter | Layout drift | [`cargo fmt --all -- --check`](../../scripts/check) | Each commit and CI run |
| Workspace lint inheritance | A crate omitting the workspace lints | [`scripts/check`](../../scripts/check) checks each manifest | Each commit and CI run |
| Compiler and Clippy lints | Discarded results, panic shortcuts, unchecked conversions and catch-all match arms | [Workspace lints](../../Cargo.toml) via [`scripts/check`](../../scripts/check) | Each commit and CI run |
| Forbidden `unsafe` | Code that bypasses Rust's safety checks | [Workspace lint](../../Cargo.toml) and [ADR](../adr/0001-rust-with-unsafe-forbidden.md) | Each commit and CI run |
| Reasoned lint exceptions | Silent suppression with `#[allow]` | [Workspace lints](../../Cargo.toml), [AGENTS.md](../../AGENTS.md#guards) | Each commit and CI run |
| Cognitive complexity | Functions that exceed a complexity of 8 | [Workspace lint](../../Cargo.toml), [threshold](../../clippy.toml) | Each commit and CI run |
| `unreachable_pub` | Unnecessarily public items inside private modules | [Workspace lint](../../Cargo.toml) | Each commit and CI run |

## Crate boundaries

Give every capability a clear owner and callers a small interface so internal code can change within its crate. Each crate’s own tests must defend its code, and the chess reference stays independent of the engine.

| Guardrail | What it stops | Where enforced | When |
|---|---|---|---|
| Crate dependencies | Edges outside the allowed directions | [Boundary table](../../scripts/check-crate-boundaries) | Each commit and CI run |
| Deep modules | Direct public module declarations at the library root and alternate library entry paths | [`cargo xtask interfaces`](../../crates/xtask/src/interfaces.rs), [`lib.rs` path check](../../scripts/check-crate-boundaries), [interface rules](../../CODING_STANDARDS.md#crate-interfaces) | Each commit and CI run. Macro-generated exports and re-exports in review |
| Development-only reference | Shipping the independent chess reference as an engine dependency | [`shakmaty` dependency check](../../scripts/check-crate-boundaries) | Each commit and CI run |
| Test ownership | A caller’s tests supplying the evidence for a crate | [Mutation](../../scripts/check-mutants) and [coverage](../../scripts/check-coverage). The [dependency check](../../scripts/check-crate-boundaries) enforces allowed edges | Mutation and coverage on each CI run. Dependencies in `scripts/check` |

See [crate boundaries](crate-boundaries.md) for the permitted dependencies and interface review.

## Vocabulary

Use the same names across crates so a reader can tell when code refers to the same chess or engine concept.

| Guardrail | What it stops | Where enforced | When |
|---|---|---|---|
| Glossary | Banned wording in identifiers | [Glossary](../../GLOSSARY.md) and [`cargo xtask glossary`](../../crates/xtask/src/glossary.rs) | Each commit and CI run |

## Evidence of behaviour

Use independent expected values, published perft and reachable positions nobody selected by hand to catch errors. Check the wired binary against the GUI contract, give the owner a stable bench signature to check after a refactor, and trace strength claims to matches.

| Guardrail | What it stops | Where enforced | When |
|---|---|---|---|
| Independent expected values | Tests that attest to the implementer's own output | [`expectations/`](../../expectations/README.md), [evidence rules](../evidence.md) | Tests each commit and CI run. Provenance in review |
| Published perft | Incorrect legal move counts | [Expected counts](../../expectations/perft.epd), [rules](../../crates/rules/tests/positions.rs), [search](../../crates/search/tests/search.rs) | Shallow in `scripts/check`. Deep perft uses the ignored release-mode tests on request |
| Seeded reference comparison | Differences in legal moves, resulting positions, check detection or search results | [Rules comparison](../../crates/rules/tests/positions.rs), [search comparison](../../crates/search/tests/search.rs) | In `scripts/check`. The complete published mate suite is ignored by default and run in release mode on request |
| UCI transcripts | Incorrect responses, process exits or search reports | [Transcript runner](../../crates/fisher/tests/transcripts.rs), [expected transcripts](../../expectations/uci) | In `scripts/check` |
| Bench signature | A changed fixed-search node count | [Signature test](../../crates/bench/tests/signature.rs), [`fisher bench` and commit record](../process.md#recording-the-bench-signature) | Signature test each commit and CI run. Commit record in review |
| Strength record | Unsupported claims about playing strength | [`strength/measure` and records](../../strength/README.md) | When claiming a strength change |

## Strength of the tests

Use each crate’s own tests to catch changes in behaviour and retain region coverage. A surviving mutant points to an assertion the tests need or a proof the owner can review.

| Guardrail | What it stops | Where enforced | When |
|---|---|---|---|
| Mutation testing | A crate’s own tests passing despite a behaviour change | [`scripts/check-mutants`](../../scripts/check-mutants), [reviewed equivalents](../../.cargo/mutation-gate.json) | Whole workspace on each CI run |
| Per-crate region coverage | Losing coverage supplied by a crate’s own tests | [`scripts/check-coverage`](../../scripts/check-coverage), [reviewed floors](../../.cargo/coverage-floors.json) | Each CI run |

See [mutation testing](mutation.md) for why the job runs across the workspace and [coverage](coverage.md) for floors and uncovered regions.

## Guarding the guards

Make changes to what passes easy for the owner to find and review. Check dependencies even when an implementation change is small.

| Guardrail | What it stops | Where enforced | When |
|---|---|---|---|
| Protected paths and `Guard-Change:` | Changes to checking policy without a stated reason | [Protected paths](../../.githooks/protected-paths), [commit-message hook](../../.githooks/commit-msg) | Every commit with the hooks installed |
| Dependency advisories and sources | Known vulnerable dependencies and unauthorized sources | [`cargo deny`](../../scripts/check), [configuration](../../deny.toml) | Each commit and CI run |

## Review

Spend the owner’s attention on claims, interfaces and guards. Keep deferred behaviour visible in the tracker after the agent hands back the work.

| Guardrail | What it stops | Where enforced | When |
|---|---|---|---|
| Review and tiers | Merging behaviour or commitments without the required review | [Process tiers](../process.md#tiers) and [review](../process.md#what-the-human-reads-in-order) | Prose only |
| Ticketed TODO | Deferred work with no owner-visible record | [Deferred work rules](../../CODING_STANDARDS.md#deferred-work) | Prose only |
