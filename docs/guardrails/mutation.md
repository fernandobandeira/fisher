# Mutation testing

The Check workflow mutation-tests the whole workspace on pull requests, pushes to `main`, and the weekly schedule. Four jobs divide the workspace mutants into round-robin shards and upload their reports. The required `mutation` job requires successful parts, checks that the finished reports contain every mutant from the unsplit list exactly once, and reconciles surviving mutants against the reviewed equivalents over the combined results.

Diff scoping was removed because changes to shared tests, manifests, expected data and the gate itself can affect mutants outside the diff. `scripts/check-mutants` without arguments tests and reconciles the whole workspace locally.

- **Mutation testing.** A surviving mutant identifies an assertion or proof the owner needs.
