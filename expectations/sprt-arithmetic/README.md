# The arithmetic of the SPRT

Staged by fisher-mwa.2 on 2026-11-26, before the judge exists (fisher-mwa.3), by
an author who will not implement it. Sources of the test: the spec fisher-mwa
("The verdict" and "The arithmetic") and fisher-r6y D76 and D78. Every
log-likelihood ratio (LLR) and every result below was printed by two published
implementations that fisher did not write. No script of this repository
computed one, and no value came from running fisher.

No check reads this directory yet. fisher-mwa.3 adds the check and shows it
failing.

## The test

One observation is the points the measured revision takes from the two games of
one opening: 0, 0.5, 1, 1.5 or 2. The test statistic is the LLR of the
generalized SPRT over opening pairs, with the SPRT bounds in logistic Elo. There
are two tests:

| Test | SPRT bounds |
|---|---|
| Gain | 0 and 10 |
| No-loss | -10 and 0 |

Each test has a 5% chance of error either way (alpha = beta = 0.05), so its
thresholds are ln 19 = 2.9444389791664403 and -ln 19. The test ends at the first
observation where the LLR reaches ln 19, with the verdict pass, or falls to
-ln 19, with the verdict fail. A sequence that reaches neither threshold by its
last observation has no verdict, written `none`.

## The files

All three are tab-separated, with a header line of field names. Lines that
start with `#` are comments: the sources, the date and the meaning of the
fields. Python's `csv` module reads them once the comment lines are skipped.

| File | What it holds |
|---|---|
| [`counts.tsv`](counts.tsv) | 16 cases. Each gives the number of pairs with each of the five results and the LLR of each test over them. |
| [`sequences.tsv`](sequences.tsv) | 6 sequences. For each test: the observation at which it ends, and its result. |
| [`trace.tsv`](trace.tsv) | The observations of each sequence in order, and after each one the LLR and the result of each test over the observations so far. |

In `counts.tsv`, `points_0` to `points_2` count the pairs that gave the measured
revision 0, 0.5, 1, 1.5 and 2 points. `gain_llr` and `no_loss_llr` are the
expected values, printed by fastchess. `gain_llr_fishtest` and
`no_loss_llr_fishtest` are the same LLRs printed by Fishtest, the witness. An LLR
matches when it differs from the expected value by at most the tolerance in
`gain_tolerance` or `no_loss_tolerance`. The tolerance is 0.001 times the larger
of 1 and the absolute expected value. See [The tolerance](#the-tolerance).

In `sequences.tsv`, `gain_ends_at` and `no_loss_ends_at` are the number, from 1,
of the observation at which each test ends, or `-` when it does not end.
`gain_result` and `no_loss_result` are `pass`, `fail` or `none`.

In `trace.tsv`, each line is one observation of one sequence: the sequence, the
observation's number from 1, and its points. The rest of the line describes the
counts of all observations up to and including this one: fastchess's LLR and
`SPRT::getResult` (`H1` passes, `H0` fails, `CONTINUE` is neither), then
Fishtest's LLR and the state `update_SPRT` gives a fresh SPRT object
(`accepted` passes, `rejected` fails, `-` is neither). The trace goes on past
the observation at which a test ends. Those later values show what a judge that
does not stop would see, and they never change the result.

## What the cases cover

| Sequence | Observations | Gain test | No-loss test | What it shows |
|---|---:|---|---|---|
| `decisive-pass` | 160 | pass at 122 | pass at 117 | A revision about 90 Elo stronger. |
| `decisive-fail` | 180 | fail at 139 | fail at 149 | Its mirror, about 90 Elo weaker. |
| `no-verdict` | 60 | none | none | Too few pairs for either test. |
| `equal` | 2600 | fail at 2535 | pass at 827 | Two equal revisions, the result D79 expects of 5c5d6cf against itself. |
| `all-shared` | 200 | fail at 102 | pass at 102 | Every pair shared 1 to 1, so the variance of the sample is zero. |
| `crosses-and-returns` | 530 | pass at 122 | pass at 117 | The LLR crosses a threshold and later returns inside. |

In `crosses-and-returns` the gain test's LLR reaches ln 19 at observation 122,
returns inside at 125, and falls past -ln 19 at 359. It ends there with fail.
The no-loss test's LLR reaches ln 19 at 117, returns inside at 126, reaches it
again at 131, returns at 133, and falls past -ln 19 at 407. So the first
crossing gives pass for both tests, and a judge that reads only the counts of
the last observation would give fail for both. The first 130 observations are
the first 130 of `decisive-pass`.

The cases in `counts.tsv` are: one pair of each result; small decisive samples
and their mirror; balanced and slightly favourable samples of 1,000 pairs; a
sample with no pair of 0 or 2 points; 1, 10, 100 and 1,000 pairs all shared
1 to 1; 10 pairs all won and 10 all lost; the two pentanomial test cases of
fastchess's `app/tests/sprt_test.cpp` (scored here with fisher's SPRT bounds);
and the two pentanomial examples in Fishtest's `stat_util.py`.

Every verdict here holds for any LLR within the tolerance. Before the
observation at which a test ends, the absolute LLR is below ln 19 by more than
the tolerance at every observation. At that observation it is past the
threshold by more than the tolerance. In a sequence with no verdict, every
observation stays inside by more than the tolerance. The generator below
asserts this.

## When every pair is shared 1 to 1

Both sources replace a count of zero with 0.001 before they compute anything
(fastchess's `regularize`, Fishtest's `LLRcalc.regularize`). With every pair
shared 1 to 1, the empirical distribution keeps a mass of 0.001 on each of the
other four results. Its variance is small but not zero, and the maximum
likelihood estimate is defined. The LLR then moves by about 0.029 for each
shared pair, down for the gain test and up for the no-loss test: -0.2858 after
10 pairs, -2.9111 after 100. In `all-shared` the gain test fails and the
no-loss test passes at observation 102. Neither source returns zero or reports
an error.

The approximation that divides by the variance of the sample does not survive
this case. Fishtest's own `LLRcalc.LLR_alt2`, given the same regularized
counts, gives -16.57 after 10 shared pairs and -1656.06 after 100. Cases
`shared-10`, `shared-100`, `shared-1000` and the sequence `all-shared` tell
the exact LLR from it.

A pair shared 1 to 1 may be one win and one loss, or two draws. Both are 1
point. Both sources add those two counts before computing, so the files carry
only the total.

## The sources

**fastchess** alpha 1.8.3, https://github.com/Disservin/fastchess at
`d7b47e8dec6b564aaf4cbb2922ab37e6c6761e75` (tag `v1.8.3-alpha`), the release
`scripts/install-fastchess` pins. The file is
`app/src/matchmaking/sprt/sprt.cpp`. `SPRT::getLLR(stats, true)` computes the
pentanomial LLR. With the model `logistic` it converts both SPRT bounds
from logistic Elo to scores and computes the exact generalized LLR from the maximum
likelihood estimates (`getLLR_logistic`, after Van den Bergh). `SPRT::getResult`
returns `H1` when the LLR is at least `log((1 - beta) / alpha)` = ln 19, `H0`
when it is at most `log(beta / (1 - alpha))` = -ln 19, and `CONTINUE`
otherwise. That is the rule of D78. The expected values are fastchess's.

**Fishtest**, https://github.com/official-stockfish/fishtest at
`ea59d0c793613882480198bf5402401054046ad7`, with the files
`server/fishtest/stats/LLRcalc.py` and `server/fishtest/stats/stat_util.py`
(last changed in `93fe81eb8256b870ab759ec4470f6251fe985827`).
`LLRcalc.LLR_logistic(elo0, elo1, results)` computes the same generalized LLR
from the five pentanomial counts in logistic Elo. It uses its own root finder,
SciPy's `brentq`. `stat_util.update_SPRT` computes the LLR the same way and sets
the state `accepted` or `rejected`. It is the witness for every value.

Two differences in Fishtest's stopping rule do not change any value here:

- In operation, Fishtest calls `update_SPRT` on one SPRT object after every
  batch, and that object corrects the thresholds for overshoot as it goes. D78
  has no such correction, and fastchess has none. The driver below gives
  `update_SPRT` a fresh SPRT object for each set of counts, so the correction
  stays zero and the state depends on the counts alone. Fed one pair at a time
  into one object, Fishtest would end `decisive-fail` at 138 and 148 and
  `all-shared` at 101 instead of 139, 149 and 102. This is reported, not
  committed.
- `update_SPRT` compares with strict inequalities, and fastchess includes
  equality. No LLR here lies within the tolerance of a threshold, so the
  difference never shows.

### The tolerance

Both sources compute the same formula and differ only in how precisely they
find the root of one equation. fastchess stops its ITP search when the root,
theta, lies within an interval of width 0.002. Fishtest's `brentq` stops at
about 2e-12. The two LLRs agree to within 1e-7 on small samples and drift apart
slowly with the number of pairs. The difference is largest when every pair is
shared: 0.0100 on `shared-1000`, where the LLR is -29.18. The tolerance,
0.001 times the larger of 1 and the absolute LLR, is at least twice the
difference between the two sources on every line of `counts.tsv` and
`trace.tsv` (at most 34% of it). So a judge that agrees with either source
within half the tolerance matches.

## How to produce the values again

Commands are for macOS or Linux with git, a C++17 compiler, `uv` and Python 3.
`$WORK` is any empty directory outside this repository. The values here were
taken with Apple clang 21.0.0, Python 3.15.1 with SciPy 1.18.1 and NumPy 2.5.5
for Fishtest (Fishtest's `uv.lock` pins SciPy 1.18.1), and Python 3.9.6 for the
generator.

Fetch the two sources at their commits:

```sh
mkdir -p "$WORK/src" "$WORK/build" "$WORK/drivers" && cd "$WORK/src"
git clone https://github.com/Disservin/fastchess.git
git -C fastchess checkout d7b47e8dec6b564aaf4cbb2922ab37e6c6761e75
git clone https://github.com/official-stockfish/fishtest.git
git -C fishtest checkout ea59d0c793613882480198bf5402401054046ad7
```

As a check on the build, fastchess's own tests pass on this machine:
`make -C fastchess/app tests CXX=c++ && fastchess/app/fastchess-tests -ts=SPRT`
reports 12 of 12 SPRT test cases passed.

Save the three drivers below into `$WORK/drivers`. Then build the fastchess
driver against fastchess's own source with its release flags, make the Python
environment for Fishtest, and run the generator. It writes the three files
into its argument:

```sh
F="$WORK/src/fastchess/app"
c++ -O3 -std=c++17 -DNDEBUG -I"$F/src" -I"$F/third_party" -I"$F/third_party/fmt/include" \
    -o "$WORK/build/fastchess_sprt" "$WORK/drivers/fastchess_sprt.cpp" \
    "$F/src/matchmaking/sprt/sprt.cpp" "$F/src/core/logger/logger.cpp" \
    "$F/src/core/globals/globals.cpp" "$F/src/core/time/time.cpp" \
    "$F/third_party/fmt/src/format.cc" -pthread
uv venv --python 3.15 "$WORK/venv-fishtest"
VIRTUAL_ENV="$WORK/venv-fishtest" uv pip install scipy==1.18.1
mkdir -p "$WORK/out" && cd "$WORK/drivers"
WORK="$WORK" python3 -I generate.py "$WORK/out"
```

The three files in `$WORK/out` are then identical to the ones here.

### `fastchess_sprt.cpp`

It hands each set of counts to fastchess's `Stats` and `SPRT` classes and
prints what they return.

```cpp
// Feeds pentanomial counts to fastchess's own SPRT class and prints what it returns.
// Input, one line per set of counts: LL LD WL DD WD WW (fastchess's Stats order).
// Output, one line per input: the two tests' LLR (%.17g) and SPRT::getResult.
#include <cstdio>
#include <iostream>
#include <matchmaking/sprt/sprt.hpp>
#include <matchmaking/stats.hpp>

using namespace fastchess;

static const char* name(SPRTResult r) {
    return r == SPRT_H1 ? "H1" : r == SPRT_H0 ? "H0" : "CONTINUE";
}

int main() {
    const SPRT gain(0.05, 0.05, 0.0, 10.0, "logistic", true);
    const SPRT no_loss(0.05, 0.05, -10.0, 0.0, "logistic", true);
    std::fprintf(stderr, "gain lower %.17g upper %.17g; no-loss lower %.17g upper %.17g\n", gain.getLowerBound(),
                 gain.getUpperBound(), no_loss.getLowerBound(), no_loss.getUpperBound());
    int ll, ld, wl, dd, wd, ww;
    while (std::cin >> ll >> ld >> wl >> dd >> wd >> ww) {
        const Stats stats(ll, ld, wl, dd, wd, ww);
        const double g = gain.getLLR(stats, true);
        const double n = no_loss.getLLR(stats, true);
        std::printf("%.17g\t%s\t%.17g\t%s\n", g, name(gain.getResult(g)), n, name(no_loss.getResult(n)));
    }
}
```

### `fishtest_sprt.py`

It hands each set of counts to Fishtest's `LLRcalc.LLR_logistic` and
`stat_util.update_SPRT` and prints what they return.

```python
"""Feeds pentanomial counts to Fishtest's own SPRT functions and prints what they return.

Input, one line per set of counts: LL LD WL DD WD WW (the same order as the fastchess driver).
Fishtest takes the five frequencies LL, LD+DL, LW+DD+WL, DW+WD, WW, so WL and DD are added.
Output, one line per input: for each test, LLRcalc.LLR_logistic and the state that
stat_util.update_SPRT sets on a fresh SPRT object given those counts.
"""
import sys

from fishtest.stats import LLRcalc, stat_util

TESTS = ((0.0, 10.0), (-10.0, 0.0))


def main():
    for line in sys.stdin:
        if not line.strip():
            continue
        ll, ld, wl, dd, wd, ww = (int(x) for x in line.split())
        results = [ll, ld, wl + dd, wd, ww]
        out = []
        for elo0, elo1 in TESTS:
            llr = LLRcalc.LLR_logistic(elo0, elo1, list(results))
            sprt = stat_util.SPRT(alpha=0.05, beta=0.05, elo0=elo0, elo1=elo1, elo_model="logistic")
            stat_util.update_SPRT({"pentanomial": list(results)}, sprt)
            assert sprt["llr"] == llr
            out += ["%.17g" % llr, sprt["state"] or "-"]
        print("\t".join(out))


if __name__ == "__main__":
    main()
```

### `generate.py`

It chooses the inputs: the cases of `counts.tsv`, and the sequences, which
are drawn from fixed pentanomial weights with Python's `random.Random` and a
fixed seed. It builds the counts after each observation and passes them to
both drivers. It writes their output unchanged, and takes as the observation
at which a test ends the first one where fastchess's `getResult` is not
`CONTINUE`. It asserts that the two sources agree and that every verdict holds
by more than the tolerance. It computes no LLR and no result of its own.

```python
"""Chooses the inputs, feeds them to both sources' drivers, and writes what the sources printed.

This script computes no log-likelihood ratio and no verdict. It builds the counts of each prefix,
copies the drivers' output verbatim, finds the first prefix whose source result is not "continue",
and checks that the two sources agree.
"""
import math
import os
import random
import subprocess
import sys

WORK = os.environ["WORK"]
OUT = sys.argv[1]
POINTS = ["0", "0.5", "1", "1.5", "2"]
# Index into the drivers' input order LL LD WL DD WD WW. A pair shared 1 to 1 is counted as DD.
SLOT = {"0": 0, "0.5": 1, "1": 3, "1.5": 4, "2": 5}
LN19 = math.log(19)


def run(lines):
    inp = "".join(lines)
    fc = subprocess.run([f"{WORK}/build/fastchess_sprt"], input=inp, capture_output=True,
                        text=True, check=True).stdout.splitlines()
    ft = subprocess.run([f"{WORK}/venv-fishtest/bin/python", f"{WORK}/drivers/fishtest_sprt.py"],
                        input=inp, capture_output=True, text=True, check=True,
                        env={"PYTHONPATH": f"{WORK}/src/fishtest/server"}).stdout.splitlines()
    assert len(fc) == len(ft) == len(lines)
    return [l.split("\t") for l in fc], [l.split("\t") for l in ft]


def driver_line(five):
    ll, ld, one, wd, ww = five
    return f"{ll} {ld} 0 {one} {wd} {ww}\n"


def tolerance(llr):
    return 1e-3 * max(1.0, abs(llr))


def agree(fc, ft):
    """fastchess and Fishtest give the same result, and their LLRs differ by at most half the tolerance."""
    for col_llr, col_res in ((0, 1), (2, 3)):
        a, b = float(fc[col_llr]), float(ft[col_llr])
        assert abs(a - b) <= tolerance(a) / 2, (fc, ft)
        assert {"H1": "accepted", "H0": "rejected", "CONTINUE": "-"}[fc[col_res]] == ft[col_res], (fc, ft)


def sample(weights, n, seed):
    return random.Random(seed).choices(POINTS, weights=weights, k=n)


COUNTS = [
    ("one-of-each", [1, 1, 1, 1, 1]),
    ("strong-60", [1, 6, 24, 18, 11]),
    ("weak-60", [11, 18, 24, 6, 1]),
    ("balanced-1000", [50, 200, 500, 200, 50]),
    ("slight-gain-1000", [45, 195, 500, 205, 55]),
    ("no-extremes-30", [0, 5, 20, 5, 0]),
    ("shared-1", [0, 0, 1, 0, 0]),
    ("shared-10", [0, 0, 10, 0, 0]),
    ("shared-100", [0, 0, 100, 0, 0]),
    ("shared-1000", [0, 0, 1000, 0, 0]),
    ("wins-10", [0, 0, 0, 0, 10]),
    ("losses-10", [10, 0, 0, 0, 0]),
    ("fastchess-test-1", [223, 9863, 20279 + 1000, 10037, 246]),
    ("fastchess-test-2", [871, 26175, 55003 + 980, 26678, 821]),
    ("fishtest-example-1", [10789, 19328, 33806, 19402, 10543]),
    ("fishtest-example-2", [39, 2226, 31451, 2412, 40]),
]

strong = sample([2, 10, 40, 30, 18], 200, 1)
weak = sample([18, 30, 40, 10, 2], 400, 2)
SEQUENCES = [
    ("decisive-pass", strong[:160]),
    ("decisive-fail", weak[:180]),
    ("no-verdict", sample([5, 20, 50, 20, 5], 60, 3)),
    ("equal", sample([5, 20, 50, 20, 5], 2600, 4)),
    ("all-shared", ["1"] * 200),
    ("crosses-and-returns", strong[:130] + weak[:400]),
]

HEADER = """# Source: fastchess alpha 1.8.3, https://github.com/Disservin/fastchess at d7b47e8dec6b564aaf4cbb2922ab37e6c6761e75, app/src/matchmaking/sprt/sprt.cpp
# Witness: Fishtest, https://github.com/official-stockfish/fishtest at ea59d0c793613882480198bf5402401054046ad7, server/fishtest/stats/LLRcalc.py and stat_util.py
# Taken: 2026-11-26, by the commands in README.md
# Test: generalized SPRT over opening pairs, SPRT bounds in logistic Elo, alpha = beta = 0.05; gain test 0 and 10, no-loss test -10 and 0
# Thresholds: a test ends at the first observation where the LLR reaches ln 19 (pass) or falls to -ln 19 (fail)
"""

with open(f"{OUT}/counts.tsv", "w") as f:
    fc, ft = run([driver_line(c) for _, c in COUNTS])
    f.write(HEADER)
    f.write("# Fields: case; pairs that gave the measured revision 0, 0.5, 1, 1.5 and 2 points; then for the gain test and the no-loss test: fastchess's LLR, Fishtest's LLR, tolerance\n")
    f.write("case\tpoints_0\tpoints_0.5\tpoints_1\tpoints_1.5\tpoints_2\tgain_llr\tgain_llr_fishtest\tgain_tolerance\tno_loss_llr\tno_loss_llr_fishtest\tno_loss_tolerance\n")
    for (name, c), a, b in zip(COUNTS, fc, ft):
        agree(a, b)
        f.write("\t".join([name] + [str(x) for x in c] + [
            a[0], b[0], "%.6g" % tolerance(float(a[0])), a[2], b[2], "%.6g" % tolerance(float(a[2]))]) + "\n")

summary = []
with open(f"{OUT}/trace.tsv", "w") as f:
    f.write(HEADER)
    f.write("# Fields: sequence; observation number from 1; points of that observation; then for the gain test and the no-loss test, over the observations up to and including this one: fastchess's LLR and SPRT::getResult (H1 pass, H0 fail, CONTINUE neither), Fishtest's LLR and update_SPRT state on a fresh SPRT object (accepted, rejected, - for neither)\n")
    f.write("sequence\tobservation\tpoints\tgain_llr\tgain_result_fastchess\tgain_llr_fishtest\tgain_state_fishtest\tno_loss_llr\tno_loss_result_fastchess\tno_loss_llr_fishtest\tno_loss_state_fishtest\n")
    for name, obs in SEQUENCES:
        counts = [0] * 6
        lines = []
        for p in obs:
            counts[SLOT[p]] += 1
            lines.append(" ".join(map(str, counts)) + "\n")
        fc, ft = run(lines)
        ends = []
        for col_llr, col_res in ((0, 1), (2, 3)):
            end = next((i for i, r in enumerate(fc) if r[col_res] != "CONTINUE"), None)
            # The verdict must hold for any LLR within the tolerance: every earlier prefix
            # inside the thresholds by more than it, and the deciding one past a threshold by more than it.
            for i, r in enumerate(fc[: len(fc) if end is None else end + 1]):
                llr = float(r[col_llr])
                if end is None or i < end:
                    assert abs(llr) < LN19 - tolerance(llr), (name, i + 1, llr)
                else:
                    assert abs(llr) >= LN19 + tolerance(llr), (name, i + 1, llr)
            if end is None:
                ends += ["-", "none"]
            else:
                ends += [str(end + 1), {"H1": "pass", "H0": "fail"}[fc[end][col_res]]]
        for i, (p, a, b) in enumerate(zip(obs, fc, ft)):
            agree(a, b)
            f.write("\t".join([name, str(i + 1), p, a[0], a[1], b[0], b[1], a[2], a[3], b[2], b[3]]) + "\n")
        summary.append([name, str(len(obs))] + ends)

with open(f"{OUT}/sequences.tsv", "w") as f:
    f.write(HEADER)
    f.write("# Fields: sequence (its observations are in trace.tsv); number of observations; for the gain test and the no-loss test: the observation at which the test ends, or - if it does not end; its result: the verdict pass or fail, or none when no threshold is reached by the last observation\n")
    f.write("sequence\tobservations\tgain_ends_at\tgain_result\tno_loss_ends_at\tno_loss_result\n")
    for s in summary:
        f.write("\t".join(s) + "\n")
print(open(f"{OUT}/sequences.tsv").read())
```
