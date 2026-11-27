"""The generalized SPRT over opening pairs, with logistic Elo."""

from dataclasses import dataclass
from math import inf, log, log10, log1p, sqrt


SPRT_BOUNDS = {"gain": (0, 10), "no-loss": (-10, 0)}
ALPHA = BETA = 0.05
LOWER_THRESHOLD = log(BETA / (1 - ALPHA))
UPPER_THRESHOLD = log((1 - BETA) / ALPHA)
MAX_PAIRS = 10_000
PSEUDOCOUNT = 0.001
SCORES = (0, 0.25, 0.5, 0.75, 1)


def constrained_log_likelihood(counts, score):
    # Maximize sum(count_i * log(q_i)) with sum(q_i) = 1 and
    # sum(q_i * (score_i - score)) = 0. Lagrange multipliers give
    # q_i = empirical_share_i / (1 + multiplier * (score_i - score)).
    # Solve the mean constraint by bisection: its left side decreases
    # strictly between the two poles that enclose positive denominators.
    offsets = tuple(value - score for value in SCORES)
    low, high = -1 / offsets[-1], -1 / offsets[0]
    for _ in range(48):
        midpoint = (low + high) / 2
        mean_constraint = sum(
            count * offset / (1 + midpoint * offset)
            for count, offset in zip(counts, offsets)
        )
        if mean_constraint > 0:
            low = midpoint
        else:
            high = midpoint
    multiplier = (low + high) / 2
    # The empirical log-likelihood is common to both hypotheses and cancels.
    # Using counts here retains the total mass of the pseudocounts.
    return -sum(
        count * log1p(multiplier * offset) for count, offset in zip(counts, offsets)
    )


def log_likelihood_ratio(counts, test):
    if not any(counts):
        return 0.0
    regularized = tuple(count or PSEUDOCOUNT for count in counts)
    score0, score1 = (1 / (1 + 10 ** (-elo / 400)) for elo in SPRT_BOUNDS[test])
    return (
        constrained_log_likelihood(regularized, score1)
        - constrained_log_likelihood(regularized, score0)
    )


def verdict(llr):
    if llr >= UPPER_THRESHOLD:
        return "pass"
    if llr <= LOWER_THRESHOLD:
        return "fail"
    return None


@dataclass(frozen=True)
class Observation:
    number: int
    counts: tuple
    llr: float
    verdict: str = None


def trace(pair_results, test):
    """Yield observations, including the continued trace if the caller reads it."""
    counts = [0] * len(SCORES)
    for number, points_in_halves in enumerate(pair_results, 1):
        counts[points_in_halves] += 1
        llr = log_likelihood_ratio(counts, test)
        yield Observation(number, tuple(counts), llr, verdict(llr))


def decide(observations):
    """Return the first crossing or the last observation of the range in hand."""
    last = Observation(0, (0,) * len(SCORES), 0.0)
    for last in observations:
        if last.verdict:
            break
    return last


def elo(score):
    if score <= 0:
        return -inf
    if score >= 1:
        return inf
    return 400 * log10(score / (1 - score))


def elo_interval(counts):
    """Return logistic Elo and a 95% interval when the pair scores vary."""
    total = sum(counts)
    mean = sum(count * score for count, score in zip(counts, SCORES)) / total
    variance = sum(count * (score - mean) ** 2 for count, score in zip(counts, SCORES)) / total
    if variance == 0:
        return elo(mean), None
    margin = 1.959963984540054 * sqrt(variance / total)
    return elo(mean), (elo(mean - margin), elo(mean + margin))
