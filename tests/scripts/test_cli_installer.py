"""Offline Linux installer checks with fixture release assets."""

import hashlib
import os
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]


@unittest.skipUnless(sys.platform.startswith('linux'), 'Linux shell installer fixture')
class CliInstallerTests(unittest.TestCase):
    """Verify archive installation and checksum rejection without network."""

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.asset = self.root / 'argui-cli-v9.9.9-x86_64-unknown-linux-gnu.tar.gz'
        executable = self.root / 'argui'
        executable.write_text('#!/bin/sh\necho argui\n')
        with tarfile.open(self.asset, 'w:gz') as package:
            package.add(executable, arcname='argui')
        self.checksum = self.root / (self.asset.name + '.sha256')
        self.checksum.write_text(hashlib.sha256(self.asset.read_bytes()).hexdigest() + '  ' + self.asset.name + '\n')
        bin_dir = self.root / 'bin'
        bin_dir.mkdir()
        curl = bin_dir / 'curl'
        curl.write_text('#!/bin/sh\nfor value do\n  case "$value" in\n    *.sha256) source="$TEST_CHECKSUM" ;;\n    *.tar.gz) source="$TEST_ARCHIVE" ;;\n  esac\ndone\nwhile [ "$#" -gt 0 ]; do\n  if [ "$1" = -o ]; then shift; cp "$source" "$1"; exit 0; fi\n  shift\ndone\nexit 1\n')
        curl.chmod(0o755)
        self.environment = os.environ | {
            'PATH': str(bin_dir) + os.pathsep + os.environ['PATH'],
            'ARGUI_VERSION': 'v9.9.9',
            'ARGUI_INSTALL_DIR': str(self.root / 'install'),
            'TEST_ARCHIVE': str(self.asset),
            'TEST_CHECKSUM': str(self.checksum),
        }

    def install(self):
        """Run the installer with fixture inputs and return its process result."""
        return subprocess.run(['sh', str(ROOT / 'scripts/install-cli.sh')],
                              env=self.environment, capture_output=True, text=True)

    def test_installs_verified_binary(self):
        result = self.install()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual((self.root / 'install/argui').read_text(), '#!/bin/sh\necho argui\n')

    def test_rejects_bad_checksum_and_invalid_version(self):
        self.checksum.write_text('0' * 64 + '  ' + self.asset.name + '\n')
        result = self.install()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('checksum mismatch', result.stderr)
        self.assertFalse((self.root / 'install/argui').exists())
        self.environment['ARGUI_VERSION'] = 'v9.9.9/../../bad'
        result = self.install()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('invalid release version', result.stderr)


if __name__ == '__main__':
    unittest.main()
