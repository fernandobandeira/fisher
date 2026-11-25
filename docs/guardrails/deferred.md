## Deferred guardrails

Proposed for the owner's decision at merge. The [research and pilot criteria](research/fuzzing-and-formal-verification.md) give the evidence required before spending CI time.

| Gap | Why defer | Observable trigger | First experiment |
|---|---|---|---|
| Ordering of stop and worker completion | Existing threaded tests check release on both events | A missing or duplicate best move or hang depends on scheduling | Explore the release path with Loom and replay a broken release-once case. |
| Unexpected FEN input | Examples already exercise parser rejection | A malformed FEN escapes the examples | Run libFuzzer on FEN parsing and detect a deliberately broken length check. |
| Square conversion boundaries | Constructors restrict the domain | Square representation or indexing changes | Check the coordinate formula over symbolic inputs with Kani. |
| Ownership of overlapping searches | UCI currently tracks one accepted search at a time | A protocol proposal allows overlapping searches | Check two searches with TLC and reproduce a wrong release in a transcript. |
| Correctness of a cutoff | Material minimax already supplies a reference | Two searches disagree and the intended bound is disputed | Prove score preservation in Lean and refute a reversed bound. |
| Bench and UCI error paths | Some require a full bench or a forced OS or writer failure | The writer, reader or thread creation boundary changes | Inject a final flush, formatting, disconnect or spawn failure at the changed boundary. |
| Defensive and test-only coverage | Current data and public constructors exclude several paths | Fixed perft data, constructor guarantees, token or slice preconditions, or the inline test changes | Recheck reachability and exercise any newly reachable path with an independent expected result. |
