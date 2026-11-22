# Rust, with `unsafe` forbidden

The engine is written in Rust on a pinned toolchain, with `unsafe_code` forbidden for the whole workspace and a strict lint set that fails the build. Agents write most of the code, and a compiler that rejects a class of mistake is cheaper than a reviewer who has to spot it. The alternatives were C++, which most strong engines use and which rejects far less, and TypeScript, which the owner's other projects use and which is too slow for this.

## Consequences

- The compiler says nothing about whether a move is legal or a search is sound. Those claims are settled by the evidence in [docs/evidence.md](../evidence.md), not by the language.
- Strong engines use `unsafe` for unchecked table lookups and SIMD. If a measurement shows the engine needs it, that is a new ADR and a change the human approves, confined to one small crate.
- "Agents write better Rust" is a hypothesis this project is testing, not a fact it relies on.
