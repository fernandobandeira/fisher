# The `Guard-Change:` line

This page explains how the `Guard-Change:` line makes changes to the checks findable.

The goal is to make every change to what counts as passing easy for the
owner to find and read.

The local hook runs only where `git config core.hooksPath .githooks` has
been set. It refuses a commit touching a path in `.githooks/protected-paths`
unless the message has a `Guard-Change:` line giving a reason. An agent
can write the line itself. The owner can find it with
`git log --grep '^Guard-Change:'`.

The [Guard-Change workflow](../../.github/workflows/guard-change.yml)
uses `pull_request_target` so GitHub runs the workflow from `main`.
A pull request into `main` cannot remove its own check by editing a
workflow. The check runs when the pull request opens, reopens, receives
new commits, or is edited, including when its base changes to `main`.
It checks out the base revision, fetches the head as Git objects, and
runs the base revision's checker on the commits in `base..head`.
The checker reads the union of the base and head protected paths.
Pull request content is only read as data.

On a push to `main` the workflow checks the new commits with the
previous revision's checker, using the pushed revision's checker
during bootstrap if the previous revision has none. The existing
step in the [Check workflow](../../.github/workflows/check.yml) stays
until the `guard-change` job is required and has been seen failing
from `main`.

The local hook reads the working-tree list. `git commit --no-verify`
skips it, and it exempts merge commits. CI checks paths whose result
differs from every merge parent. The hook can accept a reason below
Git's scissors line even if Git then strips it. CI reads the recorded
message.

The pull request check cannot see an edit made to the squash message at
merge. Keep the default squash message so the original
`Guard-Change:` lines survive. The owner decided to keep squash merges
without a merge queue. Losing the line requires a hand edit, the push
check reports the missing line in a failed run after the squash lands
on `main`, and both dropping squash merges and adding a merge queue
cost more than closing this gap. See the
[merge process](../process.md#pull-requests).

See [where the ideas came from](origin.md).
