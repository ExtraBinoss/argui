"""Coverage gate contracts; run with python3 -m unittest discover -s tests/scripts."""
import importlib.util
from pathlib import Path
import unittest
from unittest.mock import patch
from tempfile import TemporaryDirectory
from contextlib import redirect_stdout, redirect_stderr
from io import StringIO
import json
import os

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
    def test_nested_packages_are_individually_gated(self):
        with TemporaryDirectory() as directory:
            root = Path(directory)
            for crate in ("argui-core", "extensions/runtime"):
                manifest = root / "crates" / crate / "Cargo.toml"
                manifest.parent.mkdir(parents=True)
                manifest.write_text("[package]\nname = \"example\"\n")
            previous = Path.cwd()
            try:
                os.chdir(root)
                self.assertEqual(gate.workspace_crates(),
                                 ["argui-core", "extensions/runtime"])
            finally:
                os.chdir(previous)

    def test_nested_prelude_with_only_reexports_has_no_coverable_code(self):
        with TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "crates" / "facade" / "src"
            source.mkdir(parents=True)
            (source / "lib.rs").write_text(
                "//! Facade.\n"
                "pub mod prelude {\n"
                "    #[cfg(feature = \"widgets\")]\n"
                "    pub use crate::widgets::{self, Button};\n"
                "}\n"
                "pub use dependency as widgets;\n"
            )
            previous = Path.cwd()
            try:
                os.chdir(root)
                self.assertTrue(gate.only_reexports("facade"))
            finally:
                os.chdir(previous)

    def test_threshold_is_enforced_for_workspace_and_every_crate(self):
        for false_count, failed in [(0, True), (1, False)]:
            file = entry([[10, 4, 10, 20, 1, false_count]], 2)
            file["filename"] = "/repo/crates/example/src/lib.rs"
            with TemporaryDirectory() as directory:
                report = Path(directory) / "report.json"
                report.write_text(json.dumps({"data": [{"files": [file]}]}))
                with patch.object(gate, "workspace_crates", return_value=["example"]), redirect_stdout(StringIO()), redirect_stderr(StringIO()):
                    self.assertEqual(gate.run(report, 85), failed)
                result = json.loads(report.with_name("coverage-gate.json").read_text())
                self.assertEqual(len(result["errors"]), 2 if failed else 0)
                self.assertEqual(result["results"]["example"]["branches"]["count"], 2)

    def test_workspace_average_cannot_hide_a_crate_below_any_threshold(self):
        for metric in gate.METRICS:
            with self.subTest(metric=metric), TemporaryDirectory() as directory:
                good, bad = entry([], 200), entry([], 10)
                good["filename"] = "/repo/crates/good/src/lib.rs"
                bad["filename"] = "/repo/crates/bad/src/lib.rs"
                good["summary"] = {m: {"count": 200, "covered": 200} for m in gate.METRICS}
                bad["summary"] = {m: {"count": 10, "covered": 10} for m in gate.METRICS}
                bad["summary"][metric]["covered"] = 8
                report = Path(directory) / "report.json"
                report.write_text(json.dumps({"data": [{"files": [good, bad]}]}))
                with patch.object(gate, "workspace_crates", return_value=["bad", "good"]), redirect_stdout(StringIO()), redirect_stderr(StringIO()):
                    self.assertTrue(gate.run(report, 85))
                result = json.loads(report.with_name("coverage-gate.json").read_text())
                self.assertGreater(result["results"]["workspace"][metric]["percent"], 85)
                self.assertEqual(result["errors"], [f"bad {metric}: 80.00% below 85%"])

    def test_missing_crate_fails_and_pure_reexports_are_not_reported_as_covered(self):
        with TemporaryDirectory() as directory:
            report = Path(directory) / "report.json"
            report.write_text(json.dumps({"data": [{"files": []}]}))
            with patch.object(gate, "workspace_crates", return_value=["facade", "missing"]), patch.object(gate, "only_reexports", side_effect=[True, False]), redirect_stdout(StringIO()), redirect_stderr(StringIO()):
                self.assertTrue(gate.run(report, 85))
            result = json.loads(report.with_name("coverage-gate.json").read_text())
            self.assertEqual(result["errors"], ["missing: missing coverage report"])
            self.assertTrue(all(v["percent"] is None for v in result["results"]["facade"].values()))

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
