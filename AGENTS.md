# fisher

A chess engine in Rust. Agents write most of the code. This file says how that work is checked, and where to find the rest.

Keep this file short. A rule that can be a check belongs in the check, not here.

## The one rule

Every claim that matters names the observation that would prove it wrong, and someone other than the implementer chose the expected value.

"Move generation is correct" is a claim. "Perft from the standard positions matches the published counts" is the observation. A ticket, a commit or a pull request that makes a claim carries the claims table from [docs/evidence.md](docs/evidence.md).

## Green

`scripts/check` is the only definition of green. Run it before you say a change is done. Git runs it before every commit, and Claude Code runs it before an agent may end its turn.

If it is red and you cannot fix it, say so plainly with the output. Do not weaken a check to make it pass.

## Where things live

| Path | What it holds |
|---|---|
| `crates/` | The code. One crate per capability; a crate's `lib.rs` exports are its whole public interface. |
| `crates/xtask/` | The project's own checks (the glossary lint). |
| `GLOSSARY.md` | The vocabulary. The build enforces the banned words in it. |
| `docs/evidence.md` | What counts as evidence, and the claims table. |
| `docs/process.md` | How a change goes from idea to merge, and what the human reads. |
| `CODING_STANDARDS.md` | How a review comment becomes a check, and the few rules that are still prose. |
| `docs/adr/` | Decisions that are hard to reverse, and why. |
| `expectations/` | Expected values that came from outside the implementer. |
| `.scratch/` | Specs and tickets (the issue tracker). |

## Boundaries

- Stay inside the crates your ticket names. If the work needs a change outside them, say so in your report instead of making it quietly.
- Another crate is used only through what it exports. Do not widen an export to reach something; raise it.
- A change to a crate's public interface is a decision for the human. Propose it, do not slip it in.

## Vocabulary

Use the terms in `GLOSSARY.md` in code, tickets, commit messages and reports. If the concept you need is not there, stop and raise it. Do not invent a word, and do not reach for a synonym the glossary avoids.

## Guards

The human reads every change to the things that do the checking. Two mechanisms make those changes findable:

- A commit that touches a path listed in `.githooks/protected-paths` needs a `Guard-Change: <why>` line in its message. Find them with `git log --grep '^Guard-Change:'`.
- A lint is suppressed only with `#[expect(<lint>, reason = "...")]`, never `#[allow]`. Each one is a request for the human to agree.

Never edit a file in `expectations/` to make a test pass. If an expected value looks wrong, stop and say why.

## Agent skills

Matt Pocock's skills live in `.agents/skills/`. They are not committed: `skills-lock.json` pins them and `npx skills@latest experimental_install -a claude-code codex` restores them. [docs/process.md](docs/process.md) says which one to use when.

### Issue tracker

Specs and tickets are markdown files under `.scratch/<feature>/`. See `docs/agents/issue-tracker.md`.

### Triage labels

The five default labels, written as a `Status:` line in each ticket. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: one `GLOSSARY.md` and `docs/adr/` at the repo root. See `docs/agents/domain.md`.
