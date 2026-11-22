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

## Wayfinding operations

- Create the map as an epic with `--labels wayfinder:map` and its body supplied through `--body-file`.
- Create each decision ticket with `--parent <map-id> --no-inherit-labels --labels wayfinder:<type>`. Use `research`, `prototype`, `grilling`, or `task` for `<type>`. Add blocking dependencies after creating the tickets.
- Find the frontier with `bd ready --parent <map-id> --unassigned --sort oldest` and claim a ticket with `bd update <id> --claim` before working on it.
- Record the answer as a comment and close the ticket. Fetch the latest map body and update its Decisions so far with `bd update <map-id> --body-file <path>`, referring to the resolved ticket by title and Beads ID.
