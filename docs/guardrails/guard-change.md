# The `Guard-Change:` line

This page explains how the `Guard-Change:` line makes changes to the checks findable.

The goal is to make every change to what counts as passing easy for the
owner to find and read.

The local hook runs only where `git config core.hooksPath .githooks` has
been set. It refuses a commit touching a path in `.githooks/protected-paths`
unless the commit body has a `Guard-Change: <why>` line of its own.
The reason is blank if it is empty or contains only spaces, tabs and
carriage returns. Put a blank line after the subject before writing the body. GitHub
prefixes each subject with `* ` when it joins commit messages for a
squash. An agent can write the reason. The owner can find it with
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

CI checks each commit. For a two-parent merge, it checks paths where
the recorded tree differs from Git's automatic merge of the parents
or Git reports the path as conflicted. A protected conflict needs a
reason even when Git writes no conflict markers and the resolution
takes one parent's version. Clean merges
can carry forward changes the parents already marked. For a commit
with more than two parents, CI checks paths whose result differs from
every parent. The checker also requires at least one
`Guard-Change:` line in the body of an ordinary commit in the range
if the net change touches a protected path. An ordinary commit has
one parent. For a pull request the checker compares the base tip
with the tree Git would land by merging the head into it. If the
merge has conflicts, the checker reports that it skipped the
net-change rule and still checks each commit. GitHub blocks merging
until the conflicts are resolved, and pushing the resolution runs the
check again. For a push the checker compares the previous and pushed
revisions.

A reason on one ordinary commit satisfies the net-change rule for
the whole range. The owner accepts that a reason for one protected
path can also cover another protected path restored by a merge. The
squash still appears in the log search and its diff shows the reset.
A clean merge can also carry an unmarked edit into a protected path.
If `main` moves `docs/x` to `scripts/x` in a marked commit and a
branch edits `docs/x` while it is unprotected, Git can land that edit
at `scripts/x` without a new reason. The net-change check catches it
only when the range has no marked ordinary commit.

The `main gates` ruleset requires `guard-change` to pass on the pull
request head and requires the branch to be up to date with `main`.
A pull request cannot remove the check or satisfy it with its own
passing job of the same name. An ordinary pull request was mergeable
([#71](https://github.com/fernandobandeira/fisher/pull/71)). GitHub blocked
an unmarked protected change ([#72](https://github.com/fernandobandeira/fisher/pull/72)),
including when the pull request deleted the workflow, checker and path
list ([#73](https://github.com/fernandobandeira/fisher/pull/73)). GitHub
also blocked the pull request when a passing job of the same name
finished after the required job failed, with the old step disabled
([#75](https://github.com/fernandobandeira/fisher/pull/75)).

Repository Actions policy 17732 allows `pull_request_target` and `push`
for `.github/workflows/guard-change.yml`. GitHub enforces its default
block of `pull_request_target` for affected public repositories from
2 December 2026. Deleting the policy would stop the required check from
reporting and block every pull request once enforcement starts.
[GitHub's default policy](https://docs.github.com/en/actions/reference/security/securely-using-pull_request_target#default-policy-for-pull_request_target)

- GitHub may skip this trigger for a head branch whose name looks like a commit SHA. If the required check never reports, rename the branch. [GitHub trigger reference](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows#pull_request_target)
- Deleting `.githooks/protected-paths` at the head makes the check fail even with a reason. Restore the list to pass the check.

The local hook reads the working-tree list. `git commit --no-verify`
skips it, and it exempts merge commits. The hook cannot see the cleanup
mode, so it requires a subject and a body reason that survive Git's
cleanup modes. It honors `core.commentChar` and treats any of Git's
possible comment characters as a comment when the setting is `auto`.
Start the subject with a character that Git will keep during editor
cleanup, put a blank line after it, and put the reason above the scissors
line. CI reads the recorded message.

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
