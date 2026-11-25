# The `Guard-Change:` line

This page explains how the `Guard-Change:` line makes changes to the checks findable.

The goal is to make every change to what counts as passing easy for the
owner to find and read.

The local hook runs only where `git config core.hooksPath .githooks` has
been set. It refuses a commit touching a path in `.githooks/protected-paths`
unless the message has a `Guard-Change:` line giving a reason. An agent
can write the line itself. The owner can find it with
`git log --grep '^Guard-Change:'`.

`git commit --no-verify` skips the hook, and the hook exempts merge
commits. No CI job checks for the line. Commits reach `main` through
GitHub squash merges, where the hook never runs. By default, the squash
message keeps the original commit messages. Whoever merges can edit it
and remove a line without the hook seeing the edit.
See the [merge process](../process.md#pull-requests).

See [where the ideas came from](origin.md).
