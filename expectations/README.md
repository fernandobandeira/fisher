# Expectations

Expected values that came from outside the implementer: published perft counts, solved test positions, examples the owner approved. Tests read these files. See [docs/evidence.md](../docs/evidence.md).

## Rules

- Every file starts with where its values came from (a link, a tool and its version) and the date they were taken.
- A value is committed before the code that has to meet it, in a commit of its own.
- The agent implementing a ticket does not edit this directory. If a value looks wrong, it stops and says why.
- This directory is a protected path: a commit that touches it needs a `Guard-Change:` line, and the human reads it.

For fisher-l7f, fisher-l7f.1 supplies the UCI transcripts before protocol code, and fisher-l7f.2 supplies the published perft counts before move generation.
