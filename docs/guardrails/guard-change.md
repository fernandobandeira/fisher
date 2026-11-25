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
The check runs when the pull request opens, reopens, receives new
commits, or is edited, including when its base changes to `main`.
It checks out the base revision, fetches the head as Git objects, and
runs the base revision's checker on the commits in `base..head`.
The checker reads the union of the base and head protected paths.
Pull request content is only read as data.

On a push to `main` the workflow checks the new commits with the
previous revision's checker, using the pushed revision's checker
during bootstrap if the previous revision has none.

CI checks each commit. For merge commits it checks paths whose result
differs from every merge parent. The checker also requires at least one
`Guard-Change:` line in the range if the net change touches a protected
path. For a pull request it compares the merge base to the head. For a
push it compares the previous and pushed revisions.

The `guard-change` job becomes required once the event policy permits
it, GitHub reports the result on the pull request head, and throwaway
pull requests prove that removing the guard or adding a passing job
with the same name cannot satisfy the requirement. Squash must also be
the only enabled merge method. An already-open pull request needs a
push or an edit to get a run. The existing step in the
[Check workflow](../../.github/workflows/check.yml) stays until the
`guard-change` job is required and has been seen failing from `main`.
The weekly run of the Check workflow skips the commit-message check.

- GitHub may skip this trigger for a head branch whose name looks like a commit SHA. If the required check never reports, rename the branch. [GitHub trigger reference](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows#pull_request_target)
- Deleting `.githooks/protected-paths` at the head makes the check fail even with a reason. Restore the list to pass the check.

The local hook reads the working-tree list. `git commit --no-verify`
skips it, and it exempts merge commits. The hook can accept a reason
below Git's scissors line even if Git then strips it. CI reads the
recorded message.

The pull request check cannot see an edit made to the squash message at
merge. Keep the default squash message so the original
`Guard-Change:` lines survive. The owner decided to leave this gap.
Dropping a line takes a hand edit to the proposed message, and the push
check normally reports the loss after the squash lands on `main`. If
the edited message also contains a GitHub skip instruction, the push
workflows do not run and there is no failed run. Dropping squash merges
or adding a merge queue costs more than closing this gap.
See the [merge process](../process.md#pull-requests) and
[GitHub's workflow skip instructions](https://docs.github.com/en/actions/how-tos/manage-workflow-runs/skip-workflow-runs).

See [where the ideas came from](origin.md).
