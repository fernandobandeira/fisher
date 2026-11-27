"""Exercise the timing comparison with a fixed cost for going first in each pair."""

from contextlib import redirect_stdout
from fractions import Fraction
from io import StringIO
from pathlib import Path
import runpy
import unittest
from unittest.mock import patch

compare = runpy.run_path(str(Path(__file__).with_name("ordering")))["compare"]


class ComparisonTest(unittest.TestCase):
    def test_first_run_cost_affects_both_medians_equally(self):
        for durations, medians, expected_gap in (
            ((100, 100), (101, 101), Fraction(0)),
            ((100, 103), (101, 104), Fraction(300, 101)),
            ((103, 100), (104, 101), Fraction(-300, 104)),
        ):
            with self.subTest(durations=durations):
                order = []

                def run_sample(binary, _):
                    order.append(binary)
                    run_first = len(order) % 2 == 1
                    seconds = durations[binary] + 2 * run_first
                    return [79], seconds * 10**9

                output = StringIO()
                with patch.dict(compare.__globals__, run_sample=run_sample), redirect_stdout(output):
                    gap = compare(("base", "measured"), (0, 1), [])

                for label, median in zip(("Base base", "Measured measured"), medians):
                    self.assertIn(f"{label}: 79 nodes, {median:.6f} s median", output.getvalue())
                self.assertEqual(gap, expected_gap)
                self.assertEqual(order, [0, 1] + [0, 1, 1, 0] * 3)


if __name__ == "__main__":
    unittest.main()
