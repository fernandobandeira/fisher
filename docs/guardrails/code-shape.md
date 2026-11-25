# Code shape

This page explains what each code shape check contributes.

- **Formatter.** Reviews can focus on behaviour.
- **Workspace lint inheritance.** New crates start with the same checks.
- **Compiler and Clippy lints.** Errors and new cases need explicit handling.
- **Forbidden `unsafe`.** The owner can rely on Rust’s safety guarantees.
- **Reasoned lint exceptions.** The owner can review the reason and Clippy flags obsolete expectations.
- **Cognitive complexity.** Important branches stay readable when a defect needs review.
- **`unreachable_pub`.** Internal code stays free to change within its crate.
