# Evidence

A change is not done when its author says it works. It is done when each claim it makes has survived an observation that could have proved it wrong.

## The rule

For every claim that matters, write three things down before the code exists:

1. The claim, in one sentence.
2. The observation that would prove it wrong.
3. Where the expected value comes from. It must not come from the agent that writes the code.

The human decides which claims matter. Agents do most of the work of testing them.

## Where an expected value can come from

Best first:

1. **A published result.** Perft counts for the standard positions; the solutions to a mate or tactics suite.
2. **An independent implementation.** Another engine or library is asked the same question: which moves are legal here, what is the perft count at this depth.
3. **An invariant the human approved.** "Unmaking a move restores the position exactly", checked over many generated positions.
4. **A statistical test whose bounds were fixed beforehand.** SPRT, for "this plays stronger".
5. **An example the human approved**, written from the requirement by someone who has not seen the patch.

These are not evidence:

- A test whose expected value the implementer got by running its own code.
- A second model reading the same patch and agreeing with it.
- A check that has never been seen to fail. Show a new check failing once, against the old code or against a deliberately broken copy, before relying on it.

Values of kinds 1 and 5 live in [`expectations/`](../expectations/README.md), are committed before the code that has to meet them, and are not edited by the implementer.

## When a reference disagrees

An independent implementation is a witness, not the judge. When fisher and the reference give different answers:

1. The test fails and work stops. Do not change fisher to match the reference.
2. Settle the position against the rules of chess, with a third implementation as the tie-breaker.
3. If the reference is wrong, the position goes into `expectations/` as an example the human approved, and the comparison skips it by name.

Two implementations can also be wrong in the same way, and a comparison cannot see that. Published results cover it, because they come from neither.

## What this looks like for a chess engine

These are the starting examples. The grilling for each feature settles the real list.

| Claim | Proved wrong by | Expected value from |
|---|---|---|
| Move generation is correct | A perft count differs from the published count for a standard position | Published tables |
| It stays correct on positions nobody picked | The legal moves differ from a reference engine's on a random reachable position | Independent implementation |
| Making then unmaking a move loses nothing | The position after unmake differs from the one before make | Invariant |
| The incremental hash is right | It differs from the hash computed from scratch | Invariant |
| A canonical FEN survives a round trip | Parsing then formatting changes the canonical string | Invariant |
| This change does not alter behaviour (a refactor, a speedup) | The bench signature changes: the node count of a fixed search over fixed positions | The previous revision |
| This change plays stronger | SPRT against the previous revision does not accept the improvement | Statistical test |
| It never plays an illegal move, crashes or loses on time | A match of many games reports one | The match runner |
| It speaks UCI | A scripted session differs from the expected transcript, or a match runner cannot finish a game | The protocol specification |

Choosing the right observation is the part to argue about. A test that passes under the wrong conditions answers a different question: a perft run that never reaches an en passant capture says nothing about en passant.

## The claims table

Every ticket lists its claims when it is written. The report or pull request that closes it fills in the last two columns.

```markdown
| Claim | Proved wrong by | Expected value from | Result | Revision |
|---|---|---|---|---|
| Legal moves are generated correctly from the six standard positions | Any perft count to depth 5 differs | expectations/perft.epd | 6 of 6 match | 3f2a91c |
```

Below the table, list anything still undecided: a claim nobody could test, a finding the human has to accept or reject. An empty list is stated as "Nothing open".

## What is worth its cost here

- **Always:** perft against published counts, comparison with a reference engine on random positions, invariants as property tests.
- **For every change to search, evaluation or time management:** the [bench signature](process.md#recording-the-bench-signature), and an SPRT result when the claim is strength.
- **Pull requests:** mutation-test the whole workspace on the merge commit. Require per-crate region coverage at or above the [reviewed floors](../.cargo/coverage-floors.json).
- **Weekly:** mutation-test the whole workspace. Recheck every reviewed equivalent mutant and require a current proof for each survivor. Run the [published suite](../scripts/check-published-suite) in release mode at the approved deep perft depths, with all 100 published mates and both four-ply node tests.

Mutation and coverage judge each crate using only its unit and integration tests.

`scripts/check` has to stay fast enough to run on every commit and at the end of every agent turn. Slow evidence (deep perft, matches, SPRT) runs on its own and is recorded in the claims table with the revision it ran against.
