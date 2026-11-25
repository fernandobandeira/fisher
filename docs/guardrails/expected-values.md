# Independent expected values

This page explains how fisher uses independent expected values and where they stop applying.

The goal is to check the engine against results chosen by someone other
than the implementer.

Otherwise an implementer could write a test that accepts whatever the code
does, or edit the expected value to make a failing check pass. Either can
hide a change in behaviour from the owner. Published results also catch
some mistakes shared by fisher and the `shakmaty` crate used as the
reference, because the results come from neither. See the
[rules comparison](../../crates/rules/tests/positions.rs).

Expected data records its source and date. The values go in a commit of
their own before the code that must meet them. For staged UCI transcripts,
a separate ticket puts them in their own directory. The implementing
commit takes the staged transcripts as written into `expectations/uci/`,
updates the expectations README and carries a `Guard-Change:` line. The
implementer does not choose or change the values.

The implementer can read the values. Code can pass every listed case and
still be wrong elsewhere. The seeded comparison with a reference checks
positions nobody picked by hand.

Exact UCI transcripts need edits whenever the output they check is
redesigned. At `f709017`, 24 of the 49 commits on `main` touched
`expectations/`, including moves of staged transcripts and edits to the
README. The published `perft.epd` and `mate-in-two.tsv` each have one commit
and have never changed. Individual transcripts have up to five commits,
and `expectations/README.md` has 20.

Python-chess supplied the scores and per-ply counts for
`position-moves.uci`, with starting-position counts that match published
perft. The bench test sums published perft counts over plies one to four,
and the search comparison checks scores and counts against a reference
minimax over `shakmaty` moves.

The material scores apply to the current evaluation, and the exhaustive
node totals apply to the current unpruned search. A change to evaluation
or the addition of pruning can legitimately change the corresponding
value, so the old expectation stops applying. The bench signature uses
the previous revision's node count to detect changes. Fisher-op4 plans
SPRT, a statistical test over games against the previous revision with
decision bounds fixed before play. See [Evidence](../evidence.md).

In fisher-j07, deleting the line that checks a king's attacks along its
rank and file from `is_attacked` passed every test in `scripts/check` at
the time. Only the deep perft tests caught it. These now run in release mode
in the weekly published suite. At depth 7, Position 3 in
`expectations/perft.epd` gave 178633737 against a published count of
178633661. The widened reference comparison now catches that deliberate
break in `scripts/check`. The history on `main` has no record of a published
value catching an implementer's real mistake. Git history alone cannot
show how often a published value caught a mistake before a commit.

See [where the ideas came from](origin.md).
