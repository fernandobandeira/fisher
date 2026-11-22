# Coding standards

Most of the standards are not written here. They are the formatter, the lints in `Cargo.toml` and `clippy.toml`, the glossary lint, the tests and the crate boundaries, and `scripts/check` runs all of them. A reviewer does not comment on anything those already enforce.

## Standards that are still prose

None yet. A rule lands here only while it waits for a check, or because it cannot be one.

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
