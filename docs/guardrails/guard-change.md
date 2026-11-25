# The `Guard-Change:` line

This page explains how the `Guard-Change:` line makes changes to the checks findable.

The goal is to make every change to what counts as passing easy for the
owner to find and read.

The local hook runs only where `git config core.hooksPath .githooks` has
been set. It refuses a commit touching a path in `.githooks/protected-paths`
unless the message has a `Guard-Change:` line giving a reason. An agent
can write the line itself. The owner can find it with
`git log --grep '^Guard-Change:'`.

The required CI check reads each commit in a pull request and each new
commit in a push to `main`. It uses the protected paths from both the base
and head revisions, so removing a path from the list does not lift its
protection for that change. It runs the base revision's checker when
available and uses the head revision's checker during bootstrap. The
weekly run skips the commit-message check.

The local hook reads the working-tree list. `git commit --no-verify`
skips it, and it exempts merge commits. CI checks paths whose result
differs from every merge parent. The hook can accept a reason below
Git's scissors line even if Git then strips it. CI reads the recorded
message.

A pull request that edits the workflow can remove the check. Retargeting
a pull request to `main` does not rerun the workflow, so a green result
from another base can carry over. The squash message normally keeps the
original commit messages, but the person merging can edit it. If a
retargeted pull request brings unchecked changes and its squash message
has no reason, or someone removes the reason while merging, the push
check catches the missing line after the squash lands on `main`.
See the [merge process](../process.md#pull-requests).

See [where the ideas came from](origin.md).
