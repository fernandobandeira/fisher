# Fisher issue tracker

Install Beads with `brew install beads`. In a fresh clone, run `bd bootstrap`
from the repository root, then check the queue with `bd ready`.

See the [issue tracker workflow](../docs/agents/issue-tracker.md) for publishing
specs, claiming tickets, and recording evidence.

Fisher uses embedded Dolt. The issue database lives under
`.beads/embeddeddolt/` and is ignored by Git. Sharing issue history across
clones requires a Dolt remote. See the
[Beads documentation](https://github.com/gastownhall/beads) for sync setup.
