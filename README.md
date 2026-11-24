# fisher

A chess engine in Rust, built almost entirely by coding agents, and set up so that their work is checked by experiments instead of by reading every line.

Fisher plays random legal moves over UCI. The [acceptance record](docs/uci-acceptance.md) covers the protocol sessions, perft counts and timed games.

## Set up a fresh clone

```sh
git config core.hooksPath .githooks                           # run the check before every commit
npx skills@latest experimental_install -a claude-code codex   # restore the agent skills
cargo install cargo-deny --version 0.22.0 --locked             # check dependency advisories and sources
scripts/check                                                 # run the checks from the CI check job
```

The skills are Matt Pocock's. They are not committed: `skills-lock.json` pins the commit, and the second command copies them into `.agents/skills/`. A new git worktree needs the same command.

The crate boundary check uses Python 3.8 or newer. The pre-commit hook runs the local gate. The pull-request jobs also report mutation and coverage results. It needs network access to refresh RustSec and Cargo registry data. If the refresh fails, restore connectivity and rerun `scripts/check`. Rust comes from [rustup](https://rustup.rs). The version is pinned in `rust-toolchain.toml`.

## Start work

Open the project in your agent and run `/grill-with-docs` on what you want to build first. [docs/process.md](docs/process.md) describes the path from there to a merge.

## Read

- [AGENTS.md](AGENTS.md): the rules every agent works under.
- [docs/evidence.md](docs/evidence.md): what counts as evidence that a change works.
- [docs/process.md](docs/process.md): the steps, the tiers, and what the human reads.
- [CODING_STANDARDS.md](CODING_STANDARDS.md): how a review comment becomes a check.
- [GLOSSARY.md](GLOSSARY.md): the vocabulary the build enforces.
