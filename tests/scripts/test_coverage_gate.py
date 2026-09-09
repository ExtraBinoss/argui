"""Coverage gate contracts; run with python3 -m unittest discover -s tests/scripts."""
import importlib.util
from pathlib import Path
import unittest
from unittest.mock import patch
from tempfile import TemporaryDirectory
from contextlib import redirect_stdout, redirect_stderr
from io import StringIO
import json

spec = importlib.util.spec_from_file_location(
    "coverage_gate", Path(__file__).resolve().parents[2] / "scripts/coverage-gate.py"
)
gate = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gate)


def entry(branches, count):
    return {
        "filename": "generic.rs",
        "branches": branches,
        "summary": {
            "branches": {"count": count, "covered": 0},
            **{metric: {"count": 10, "covered": 9}
               for metric in ("functions", "lines", "regions")},
        },
    }


class SourceCoverage(unittest.TestCase):
    def test_threshold_is_enforced_for_workspace_and_changed_crate(self):
        for false_count, failed in [(0, True), (1, False)]:
            file = entry([[10, 4, 10, 20, 1, false_count]], 2)
            file["filename"] = "/repo/crates/example/src/lib.rs"
            with TemporaryDirectory() as directory:
                report = Path(directory) / "report.json"
                report.write_text(json.dumps({"data": [{"files": [file]}]}))
                with patch.object(gate.subprocess, "check_output", side_effect=["crates/example/src/lib.rs\n", ""]), redirect_stdout(StringIO()), redirect_stderr(StringIO()):
                    self.assertEqual(gate.run(report, 85, "baseline"), failed)
                result = json.loads(report.with_name("coverage-gate.json").read_text())
                self.assertEqual(len(result["errors"]), 2 if failed else 0)
                self.assertEqual(result["results"]["example"]["branches"]["count"], 2)

    def test_complementary_generic_instantiations_cover_both_outcomes(self):
        result = gate.source_summary(entry([
            [10, 4, 10, 20, 7, 0], [10, 4, 10, 20, 0, 3],
            [10, 4, 10, 20, 0, 0],
        ], 2))
        self.assertEqual(result["branches"], {"count": 2, "covered": 2})
        self.assertEqual(result["regions"], {"count": 10, "covered": 9})

    def test_repeated_true_outcomes_do_not_cover_false_or_other_locations(self):
        result = gate.source_summary(entry([
            [10, 4, 10, 20, 7, 0], [10, 4, 10, 20, 3, 0],
            [10, 24, 10, 30, 0, 0],
        ], 4))
        self.assertEqual(result["branches"], {"count": 4, "covered": 1})

    def test_partial_branch_maps_keep_llvm_counts_and_summary_only_is_rejected(self):
        self.assertEqual(gate.source_summary(entry([], 2))["branches"],
                         {"count": 2, "covered": 0})
        report = entry([], 0)
        del report["branches"]
        with self.assertRaises(KeyError):
            gate.source_summary(report)

    def test_crate_aggregation_preserves_counts_and_separate_files(self):
        result = gate.aggregate([
            entry([[10, 4, 10, 20, 1, 0]], 2),
            entry([[10, 4, 10, 20, 0, 1]], 2),
        ])
        self.assertEqual(result["branches"], {"count": 4, "covered": 2})
        self.assertEqual(result["lines"], {"count": 20, "covered": 18})
