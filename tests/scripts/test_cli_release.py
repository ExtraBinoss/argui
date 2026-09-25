"""CLI release version and archive regression tests."""

import hashlib
from pathlib import Path
import tarfile
import tempfile
import unittest
from unittest.mock import patch
import zipfile

from scripts import cli_release


class CliReleaseTests(unittest.TestCase):
    """Exercise version ordering and cross-platform archive contents."""

    def test_version_must_exceed_every_prior_release(self):
        cli_release.check_newer('0.3.3', ['0.3.2', '0.2.1'])
        for version in ('0.3.2', '0.3.1'):
            with self.assertRaisesRegex(ValueError, 'must exceed'):
                cli_release.check_newer(version, ['0.3.2'])

    def test_packages_binary_with_matching_checksum(self):
        version = cli_release.current_version()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            binary = root / 'argui'
            binary.write_bytes(b'Argui CLI test binary')
            for target in ('x86_64-unknown-linux-gnu', 'x86_64-pc-windows-msvc'):
                archive, checksum = cli_release.make_archive(
                    f'v{version}', target, binary, root / 'out')
                expected = checksum.read_text().split()[0]
                self.assertEqual(expected, hashlib.sha256(archive.read_bytes()).hexdigest())
                if archive.suffix == '.zip':
                    with zipfile.ZipFile(archive) as package:
                        self.assertEqual(package.namelist(), ['argui.exe'])
                        self.assertEqual(package.read('argui.exe'), binary.read_bytes())
                else:
                    with tarfile.open(archive, 'r:gz') as package:
                        self.assertEqual(package.getnames(), ['argui'])
                        self.assertEqual(package.extractfile('argui').read(), binary.read_bytes())

    def test_verify_release_rejects_existing_cli_asset(self):
        version = cli_release.current_version()
        tag = f'v{version}'
        target = 'x86_64-unknown-linux-gnu'
        name = cli_release.asset_name(tag, target)

        def fake_run(*args):
            if args[:2] == ('git', 'rev-parse'):
                return 'a' * 40
            if args[:3] == ('git', 'tag', '--list'):
                return 'v0.2.1\nv0.3.1\n' + tag
            return '{"assets":[{"name":"' + name + '"}]}'

        with patch.object(cli_release, 'run', fake_run):
            with self.assertRaisesRegex(ValueError, 'already published'):
                cli_release.verify_release(tag, target)


if __name__ == '__main__':
    unittest.main()
