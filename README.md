# fisher

A chess engine in Rust, built almost entirely by coding agents, and set up so that their work is checked by experiments instead of by reading every line.

Fisher plays chess over UCI using iterative deepening, alpha-beta pruning and material evaluation. The [acceptance record](docs/uci-acceptance.md) documents the earlier random-play release's protocol sessions, perft counts and timed games.

## Set up a fresh clone

```sh
git config core.hooksPath .githooks                           # run the check before every commit
cargo install cargo-deny --version 0.22.0 --locked             # check dependency advisories and sources
scripts/check                                                 # run the local checks
```

The crate boundary check uses Python 3.8 or newer. The pre-commit hook runs the local gate. If the RustSec refresh fails, the gate reports the date of the cached snapshot and uses it if its checkout is intact and its commit is at most seven days old. Set `CARGO_NET_OFFLINE=true` to check with a usable snapshot and cached Cargo dependencies. CI runs the local checks, mutation and coverage on pull requests, pushes to `main`, and weekly. The weekly run requires a successful RustSec refresh. CI also checks commit messages for `Guard-Change:` on pull requests and pushes to `main`. Rust comes from [rustup](https://rustup.rs). The version is pinned in `rust-toolchain.toml`.

A separate weekly workflow runs the approved deep perft, all 100 published mates, comparisons against plain minimax and one plain-minimax four-ply node-count test. Run `scripts/check-published-suite` locally to run the same suite.

## Start work

Start by sizing the change you want to make. Follow the steps in [docs/process.md](docs/process.md).

## Read

- [AGENTS.md](AGENTS.md): the rules every agent works under.
- [docs/evidence.md](docs/evidence.md): what counts as evidence that a change works.
- [docs/process.md](docs/process.md): the steps, the tiers, and what the human reads.
- [CODING_STANDARDS.md](CODING_STANDARDS.md): how a review comment becomes a check.
- [GLOSSARY.md](GLOSSARY.md): the vocabulary the build enforces.
