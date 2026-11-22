# fisher

A chess engine in Rust, built almost entirely by coding agents, and set up so that their work is checked by experiments instead of by reading every line.

There is no engine yet. This is the ground it will be built on.

## Set up a fresh clone

```sh
git config core.hooksPath .githooks                           # run the check before every commit
npx skills@latest experimental_install -a claude-code codex   # restore the agent skills
scripts/check                                                 # format, lints, tests, glossary
```

The skills are Matt Pocock's. They are not committed: `skills-lock.json` pins the commit, and the second command copies them into `.agents/skills/`. A new git worktree needs the same command.

Rust comes from [rustup](https://rustup.rs); `rust-toolchain.toml` pins the version.

## Start work

Open the project in your agent and run `/grill-with-docs` on what you want to build first. [docs/process.md](docs/process.md) describes the path from there to a merge.

## Read

- [AGENTS.md](AGENTS.md): the rules every agent works under.
- [docs/evidence.md](docs/evidence.md): what counts as evidence that a change works.
- [docs/process.md](docs/process.md): the steps, the tiers, and what the human reads.
- [CODING_STANDARDS.md](CODING_STANDARDS.md): how a review comment becomes a check.
- [GLOSSARY.md](GLOSSARY.md): the vocabulary the build enforces.
