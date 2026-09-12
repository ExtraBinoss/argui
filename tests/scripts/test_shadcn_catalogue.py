"""Keep the public catalogue linked to real exported widgets and feature gates."""

import re
import tomllib
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


class ShadcnCatalogueTests(unittest.TestCase):
    def test_all_64_entries_link_to_exported_apis_and_forwarded_features(self):
        document = ROOT / "docs/shadcn-lib.md"
        rows = re.findall(
            r"^\| ([^|]+) \| \[([^]]+)\]\(([^)]+)\) \| `([^`]+)` \|",
            document.read_text(),
            re.MULTILINE,
        )
        self.assertEqual(len(rows), 64)
        self.assertEqual(len({row[0] for row in rows}), 64)
        widgets = tomllib.loads((ROOT / "crates/argui-widgets/Cargo.toml").read_text())
        facade = tomllib.loads((ROOT / "crates/argui/Cargo.toml").read_text())
        exports = (ROOT / "crates/argui-widgets/src/lib.rs").read_text()
        for name, api, link, feature in rows:
            with self.subTest(component=name):
                source = document.parent / link
                self.assertTrue(source.is_file(), link)
                self.assertRegex(exports, rf"\b{api}\b")
                self.assertIn(feature, widgets["features"])
                self.assertIn(f"widget-{feature}", facade["features"])
                self.assertIn(f"argui-widgets/{feature}", facade["features"][f"widget-{feature}"])
        for link in re.findall(r"\]\(([^)]+)\)", document.read_text()):
            if not link.startswith("https:"):
                self.assertTrue((document.parent / link).exists(), link)


if __name__ == "__main__":
    unittest.main()
