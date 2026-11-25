# Per-crate region coverage

This page explains how coverage floors are set and which regions the tests leave uncovered.

Each crate has a separate report and floor. The check excludes dedicated test-only files. Source paths escaping `src/` and build scripts fail the layout check.

Coverage floors use the measured regions, rounded down to a half percentage point. The bench tests exercise a failed position write. The final total write and the final flush can also fail, but testing both would add two complete four-ply benches to `scripts/check`.

The fixed perft data also leaves the invalid-FEN path uncovered. The rules constructors require kings and valid square coordinates, leaving defensive paths for a missing king or an out-of-bounds square outside public tests.

Deepening’s remaining gap is in its inline test code.

UCI’s uncovered regions include error propagation in response and move formatting, reader exit after the receiver disconnects, OS thread-spawn failure, and inline test code. Two defensive UCI branches are unreachable. A successful `peek` guarantees [the next token](../../crates/uci/src/command.rs#L204), and a valid slice through byte four guarantees [the suffix slice](../../crates/uci/src/lib.rs#L144).

The binary’s own bench and transcript tests cover both paths through `main`, including a failed bench write.
