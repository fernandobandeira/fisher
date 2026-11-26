# Beads issue tracker

Beads stores Fisher issues in a local Dolt database. IDs start with `fisher-`. Sharing issues across clones requires a Dolt remote. See [setup](../../.beads/README.md).

## Specs and tickets

- Publish a spec with `bd create "<title>" --type epic --body-file <path> --labels ready-for-agent`.
- Publish each implementation slice with `bd create "<title>" --type task --body-file <path> --labels ready-for-agent`. Pass `--parent <spec-id>` when slicing an existing spec. Keep the [claims table](../evidence.md) in the ticket body.
- Record dependencies with `bd dep add <ticket-id> --blocked-by <blocker-id>`.
- Fetch a ticket by ID with `bd show <id>`.
- Find implementation work with `bd ready --type task --label ready-for-agent --unassigned`. Claim it with `bd update <id> --claim` before starting work.
- Record updates and evidence with `bd comments add <id> -f <path>`. Close the ticket with `bd close <id>` once its acceptance criteria and the checks pass.

Use `bd prime` for the general Beads command reference.

## Deferred work

- Track deferred work with `bd create "<title>" --type feature --labels needs-triage --body-file <path>`. Say in the body what was deferred, when, and why.
- Put the new ID in the code as `// TODO(fisher-<id>): <what is missing>`. [CODING_STANDARDS.md](../../CODING_STANDARDS.md) has the rule.
- Find the comments for a ticket with `grep -rn 'TODO(<id>)' crates/`.

## Decisions across sessions

- Create an epic with `bd create "<title>" --type epic --body-file <path>`. Include a `## Decisions so far` section in the body to record resolved decisions.
- Create a ticket for each decision with `--parent <epic-id>`. Add blocking dependencies after creating the tickets.
- Find ready decisions with `bd ready --parent <epic-id> --unassigned --sort oldest` and claim a ticket with `bd update <id> --claim` before working on it.
- Record the answer as a comment and close the ticket. Fetch the latest epic body and update its Decisions so far with `bd update <epic-id> --body-file <path>`, referring to the resolved ticket by title and Beads ID.
