"""Release policy tests never upload crates or create GitHub releases."""
import importlib.util
import io
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import Mock, patch
import urllib.error

SPEC = importlib.util.spec_from_file_location('release', Path(__file__).parents[2] / 'scripts/release.py')
release = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(release)


class ReleasePolicyTests(unittest.TestCase):
    def test_semver_ordering_and_invalid_versions(self):
        values = ['0.1.0-alpha.1', '0.1.0-alpha.2', '0.1.0-alpha.10', '0.1.0-beta', '0.1.0', '0.2.0']
        self.assertEqual(sorted(reversed(values), key=release.version_key), values)
        for value in ['v0.1.0', '0.1', '01.1.0', '0.1.0-alpha..1', '0.1.0-01', '0.1.0+build']:
            with self.subTest(value=value), self.assertRaises(ValueError):
                release.version_key(value)

    def test_partial_release_only_selects_missing_crates(self):
        fetch = lambda name: [{'num': '0.2.0', 'yanked': False}] if name == 'argui-core' else []
        self.assertEqual(release.pending_packages(['argui-core', 'argui'], '0.2.0', fetch), ['argui'])

    def test_yanked_and_older_versions_stop_publication(self):
        for entry in [{'num': '0.2.0', 'yanked': True}, {'num': '0.3.0', 'yanked': False}]:
            with self.subTest(entry=entry), self.assertRaises(ValueError):
                release.pending_packages(['argui'], '0.2.0', lambda _: [entry])

    def test_registry_errors_fail_closed(self):
        for status in [403, 429, 500]:
            error = urllib.error.HTTPError('https://crates.io', status, 'unavailable', {}, None)
            with patch.object(release.urllib.request, 'urlopen', side_effect=error):
                with self.assertRaises(urllib.error.HTTPError):
                    release.registry_versions('argui')
            error.close()
        error = urllib.error.HTTPError('https://crates.io', 404, 'missing', {}, None)
        with patch.object(release.urllib.request, 'urlopen', side_effect=error):
            self.assertEqual(release.registry_versions('argui'), [])
        error.close()

    def test_foreign_crate_cannot_be_skipped_as_our_published_version(self):
        for owner, accepted in [('ExtraBinoss', True), ('another-account', False)]:
            versions = [{'num': '0.1.1', 'yanked': False}]
            replies = [io.BytesIO(json.dumps({'versions': versions}).encode()),
                       io.BytesIO(json.dumps({'users': [{'login': owner}]}).encode())]
            with self.subTest(owner=owner), patch.object(release.urllib.request, 'urlopen', side_effect=replies):
                if accepted:
                    self.assertEqual(release.registry_versions('argui'), versions)
                else:
                    with self.assertRaises(ValueError):
                        release.registry_versions('argui')

    def make_history(self, root, version, message):
        def git(*args):
            return subprocess.check_output(['git', *args], cwd=root, text=True, stderr=subprocess.DEVNULL).strip()
        git('init', '-q')
        git('config', 'user.name', 'Release tests')
        git('config', 'user.email', 'release-tests@example.invalid')
        manifest = root / 'Cargo.toml'
        manifest.write_text('[workspace.package]\nversion = "0.1.0"\n')
        git('add', '.')
        git('commit', '-qm', 'Initial version')
        base = git('rev-parse', 'HEAD')
        manifest.write_text(f'[workspace.package]\nversion = "{version}"\n')
        git('add', '.')
        git('commit', '--allow-empty', '-qm', message)
        return base, git

    def test_marker_and_version_increase_are_both_required(self):
        for version, message, expected in [
            ('0.1.1', 'Normal change', False),
            ('0.1.0', '[RELEASE] unchanged', False),
            ('0.1.1', '[RELEASE] Argui 0.1.1', True),
        ]:
            with self.subTest(version=version, message=message), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                base, _ = self.make_history(root, version, message)
                fetch = Mock(return_value=[])
                with patch.object(release, 'ROOT', root), patch.object(release, 'workspace', return_value=(version, ['argui'])):
                    plan = release.release_plan(base, fetch)
                self.assertEqual(plan['release'], expected)
                self.assertEqual(fetch.call_count, int(expected))

    def test_downgrade_and_existing_tag_on_other_commit_are_rejected(self):
        for version, tagged in [('0.0.9', False), ('0.1.1', True)]:
            with self.subTest(version=version), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                base, git = self.make_history(root, version, '[RELEASE] version')
                if tagged:
                    git('tag', f'v{version}', base)
                with patch.object(release, 'ROOT', root), patch.object(release, 'workspace', return_value=(version, ['argui'])):
                    with self.assertRaises(ValueError):
                        release.release_plan(base, lambda _: [])

    def test_invalid_push_base_cannot_be_treated_as_first_release(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.make_history(root, '0.1.1', '[RELEASE] version')
            with patch.object(release, 'ROOT', root), patch.object(release, 'workspace', return_value=('0.1.1', ['argui'])):
                for base in ['0' * 40, 'HEAD~1', '--help']:
                    with self.subTest(base=base), self.assertRaises(ValueError):
                        release.release_plan(base, lambda _: [])

    def test_manifest_validation_rejects_unversioned_and_unpublished_dependencies(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'Cargo.toml').write_text('[workspace.package]\nversion = "0.1.0"\n')
            package = dict(id='argui', name='argui', version='0.1.0', publish=None,
                           description='Argui', license='MIT', repository='repo', homepage='site', readme='README.md')
            for dependency in [dict(name='argui', req='*'), dict(name='private-demo', req='^0.1.0')]:
                metadata = {'workspace_members': ['argui'], 'packages': [package | {
                    'dependencies': [dependency | {'path': '../argui', 'kind': None}]}]}
                with patch.object(release, 'ROOT', root), patch.object(release, 'run', return_value=json.dumps(metadata)):
                    with self.assertRaises(ValueError):
                        release.workspace()

    def test_unchanged_version_never_runs_a_publisher(self):
        with patch.object(release, 'run') as run:
            release.publish({'release': False, 'reason': 'unchanged'})
            run.assert_not_called()

    def test_failed_cargo_verification_never_publishes_or_announces(self):
        plan = {'release': True, 'packages': ['argui'], 'version': '0.1.1'}
        with patch.dict(release.os.environ, {'CARGO_REGISTRY_TOKEN': 'test-only'}):
            with patch.object(release, 'run', side_effect=subprocess.CalledProcessError(1, 'cargo')) as run:
                with self.assertRaises(subprocess.CalledProcessError):
                    release.publish(plan)
                self.assertEqual(run.call_count, 1)
                self.assertIn('--dry-run', run.call_args.args)

    def test_missing_registry_token_stops_before_cargo(self):
        with patch.dict(release.os.environ, {}, clear=True), patch.object(release, 'run') as run:
            with self.assertRaises(ValueError):
                release.publish({'release': True, 'packages': ['argui']})
            run.assert_not_called()

    def test_github_release_follows_successful_cargo_and_registry_verification(self):
        plan = {'release': True, 'packages': ['argui'], 'all_packages': ['argui'],
                'version': '0.1.1-beta.1', 'tag': 'v0.1.1-beta.1', 'sha': 'a' * 40}
        with patch.dict(release.os.environ, {'CARGO_REGISTRY_TOKEN': 'test-only'}), \
                patch.object(release, 'run') as run, \
                patch.object(release, 'pending_packages', return_value=[]), \
                patch.object(release.subprocess, 'run', return_value=Mock(returncode=1)):
            release.publish(plan)
        calls = [call.args for call in run.call_args_list]
        self.assertIn('--dry-run', calls[0])
        self.assertEqual(calls[1][0:2], ('cargo', 'publish'))
        self.assertNotIn('--dry-run', calls[1])
        self.assertEqual(calls[2][0:4], ('gh', 'release', 'create', plan['tag']))
        self.assertIn(plan['sha'], calls[2])
        self.assertIn('--prerelease', calls[2])

    def test_completed_release_does_not_upload_or_create_a_duplicate(self):
        plan = {'release': True, 'packages': [], 'all_packages': ['argui'],
                'version': '0.1.1', 'tag': 'v0.1.1'}
        with patch.object(release, 'run') as run, \
                patch.object(release, 'pending_packages', return_value=[]), \
                patch.object(release.subprocess, 'run', return_value=Mock(returncode=0)):
            release.publish(plan)
            run.assert_not_called()

    def test_bump_updates_internal_requirements_and_preserves_external_pins(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest = root / 'Cargo.toml'
            manifest.write_text('[workspace.package]\nversion = "0.1.0"\n[workspace.dependencies]\n'
                                'argui = { path = "crates/argui", version = "0.1.0" }\nexternal = "=0.1.0"\n')
            with patch.object(release, 'ROOT', root), patch.object(release, 'workspace', return_value=('0.1.0', [])), patch.object(release, 'run') as run:
                release.bump('0.1.1')
                run.assert_called_once_with('cargo', 'update', '--workspace', '--offline')
            self.assertEqual(manifest.read_text().count('version = "0.1.1"'), 2)
            self.assertIn('external = "=0.1.0"', manifest.read_text())


if __name__ == '__main__':
    unittest.main()
