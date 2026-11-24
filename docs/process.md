# From idea to merge

The human is in the loop at the start (what to build, which claims matter) and at the end (reading what only a human can judge). The middle runs unattended.

## The steps

| # | Step | Who | Skill | What it leaves behind |
|---|---|---|---|---|
| 1 | Size the change | Human | `/ask-matt` if unsure | Small: go to step 3 with one ticket. Larger: step 2. Too big for one session: `/wayfinder`. |
| 2 | Grill | Human and agent | `/grill-with-docs` | Glossary edits, ADRs, and the claims with the source of each expected value. |
| 3 | Spec and slice | Agent, human approves the slices | `/to-spec`, `/to-tickets` | A Beads spec and one ticket per slice, each with its claims table. |
| 4 | Commit the expected values | Human, or an agent that will not implement | | Files in `expectations/`, in their own commit, before any code that has to meet them. |
| 5 | Implement | Agent, unattended | `/implement-spec` or `/implement`, with `/tdd` | Code, a green `scripts/check`, the claims table filled in. |
| 6 | Review once | A different agent | `/code-review`; a second provider for a commitment | Concrete failure scenarios, each with a disposition. |
| 7 | Read | Human | | A merge, or a decision. |
| 8 | Retro | Agent, human reads the list | `/retro` | Review comments turned into checks. See [CODING_STANDARDS.md](../CODING_STANDARDS.md). |

## Slices

A slice is thin and goes end to end. "The engine answers `go` with a legal move from the starting position" is a slice: it touches the position, move generation, search and the protocol, and it can be run. "All of move generation" is a layer, not a slice.

Merge each slice into `main` after `scripts/check`, the pull-request mutation and coverage jobs, and the review required by its tier pass. See [guardrails](guardrails.md). Do not build a tall stack of dependent branches and review it at the end.

### Pull requests

GitHub enforces the `main gates` ruleset on `main`. A pull request can merge only when its branch is up to date with `main` and the Actions `check`, `mutation`, and `coverage` jobs pass. The ruleset blocks force-pushes and deletion of `main` and has no bypass list. The project also requires the review for the change’s tier. All three jobs run on pull requests, including stacked layers, pushes to `main`, and the weekly schedule.

- **Squash-merge.** Each pull request becomes one commit on `main`. With the repository's settings the squash commit keeps the messages of the commits it replaces, so their `Guard-Change:` lines survive.
- **Stack a slice that builds on one not merged yet.** Use GitHub's stacked pull requests through the `gh stack` extension (`gh extension install github/gh-stack`), not a base branch set by hand. `gh stack init` adopts existing branches, `gh stack link` joins pull requests that are already open, and `gh stack sync` rebases the layers above one that changed.
- **Title each layer by its place and its ticket.** A layer's title starts with its position from the bottom of the stack, its size and its ticket ID, as in `[1/4] (fisher-5sg.1) Add a transcript record for any number of info lines`. When a layer is added or removed, renumber the others with `gh pr edit <number> --title`.
- **Run it without prompts.** An agent passes `--auto` to `gh stack submit` and `--yes` to `gh stack merge`. The interactive editors wait forever in an agent's shell.
- **Merge from the bottom.** `gh stack merge <pr> --squash --yes` merges every layer up to that pull request in one step, one squash commit per layer, and none of them if any layer cannot merge.

## Tiers

Tier a change by how far it reaches, how easily a mistake in it would be noticed, and how hard it is to undo.

| Tier | Examples | Gate |
|---|---|---|
| **Local** | The inside of one crate, where existing evidence pins the behaviour: perft still matches, the bench signature is unchanged | Merge when the local and pull-request checks pass. The human does not read the code. |
| **Behaviour** | Search, evaluation, time management | The human reads the claims table and plays or watches a game. Not the code. |
| **Commitment** | A crate's public interface; how a position or a move is represented; what the engine says over UCI; a file format; a new dependency; `unsafe`; anything under a protected path | The human approves before merge. An ADR if it is hard to reverse. |

## Recording the bench signature

Run `fisher bench` for a change to search, evaluation or time management. Put `Bench: <total>` in the commit message and record the total and the revision in the Result column of the [claims table](evidence.md#the-claims-table). No hook enforces this yet; a hook is a guard and would be proposed on its own.

A change that claims to preserve behaviour must keep the signature unchanged. A change intended to alter the search reports the old and new totals and explains why they differ.

## What the human reads, in order

1. **Intent and expected outcomes.** The claims, and where each expected value comes from. Only the owner can say what was meant.
2. **Commitments.** Whatever is hard to undo.
3. **Disputed claims and their experiments.** Whether the experiment tested the claim, not only whether it passed.
4. **Public interfaces and visible behaviour.** The exports of a crate, and a game played against the engine.

The human stops reading: implementation lines of a local change, formatter output, mechanical renames, generated code, and repeated review commentary.

## When review stops

One independent pass. It names concrete failure scenarios, not preferences.

Every serious finding gets exactly one of:

- an experiment that settles it,
- a design guarantee that makes it impossible (a type, a constructor, a crate boundary),
- the human's written acceptance of the risk.

Then only the fixes are checked again. Review is over when every serious finding has one of those and `scripts/check` is green.

Style and naming comments do not get a second round. Each one either climbs the ladder in [CODING_STANDARDS.md](../CODING_STANDARDS.md) or is dropped.
