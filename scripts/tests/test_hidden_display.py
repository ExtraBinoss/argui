"""Teardown must preserve child failures and tolerate a portal unmount race."""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest


class HiddenDisplay(unittest.TestCase):
    def test_command_status_and_concurrent_unmount(self):
        script = Path(__file__).resolve().parents[1] / "linux-hidden-display.sh"
        for status, persistent, expected in [(0, False, 0), (7, False, 7), (0, True, 1)]:
            with self.subTest(status=status, persistent=persistent), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                programs = {
                    "mutter": "exit 0",
                    "dbus-run-session": f"exit {status}",
                    "fusermount3": "exit 1",
                    "mountpoint": 'if [ -e "$COUNTER" ]; then exit "$PERSISTENT"; fi; touch "$COUNTER"; exit 0',
                }
                for name, body in programs.items():
                    path = root / name
                    path.write_text("#!/bin/sh\n" + body + "\n")
                    path.chmod(0o755)
                result = subprocess.run([str(script), "true"], capture_output=True, text=True,
                    env={**os.environ, "PATH": str(root) + os.pathsep + os.environ["PATH"],
                         "TMPDIR": directory, "COUNTER": str(root / "counter"),
                         "PERSISTENT": "0" if persistent else "1"})
                self.assertEqual(result.returncode, expected, result.stderr)
                self.assertEqual(list(root.glob("argui-display.*")), [])


if __name__ == "__main__":
    unittest.main()
