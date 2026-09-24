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

    def test_rate_limited_publish_waits_until_the_registry_retry_time(self):
        limited = Mock(
            returncode=101,
            stdout='',
            stderr=('status 429 Too Many Requests: try again after '
                    'Thu, 01 Jan 1970 00:01:40 GMT and see the rate limits'),
        )
        published = Mock(returncode=0, stdout='published\n', stderr='')
        with patch.object(release.subprocess, 'run', side_effect=[limited, published]) as run, \
                patch.object(release.time, 'time', return_value=90), \
                patch.object(release.time, 'sleep') as sleep:
            release.publish_package('argui-core')
        self.assertEqual(run.call_count, 2)
        sleep.assert_called_once_with(15)

    def test_publish_does_not_retry_non_rate_limit_failures(self):
        failed = Mock(returncode=101, stdout='', stderr='authentication failed')
        with patch.object(release.subprocess, 'run', return_value=failed), \
                patch.object(release.time, 'sleep') as sleep:
            with self.assertRaises(subprocess.CalledProcessError):
                release.publish_package('argui-core')
        sleep.assert_not_called()

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

    def test_marker_publishes_current_or_increased_version(self):
        for version, message, expected in [
            ('0.1.1', 'Normal change', False),
            ('0.1.0', '[PUBLISH] initial 0.1.0', True),
            ('0.1.1', '[PUBLISH] Argui 0.1.1', True),
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
                base, git = self.make_history(root, version, '[PUBLISH] version')
                if tagged:
                    git('tag', f'v{version}', base)
                with patch.object(release, 'ROOT', root), patch.object(release, 'workspace', return_value=(version, ['argui'])):
                    with self.assertRaises(ValueError):
                        release.release_plan(base, lambda _: [])

    def test_invalid_push_base_cannot_be_treated_as_first_release(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.make_history(root, '0.1.1', '[PUBLISH] version')
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

    def test_manifest_validation_rejects_versioned_internal_dev_dependencies(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'Cargo.toml').write_text('[workspace.package]\nversion = "0.1.0"\n')
            package = dict(version='0.1.0', publish=None, description='Argui', license='MIT',
                           repository='repo', homepage='site', readme='README.md')
            metadata = {
                'workspace_members': ['argui-core', 'argui-i18n'],
                'packages': [
                    package | {'id': 'argui-core', 'name': 'argui-core', 'dependencies': []},
                    package | {
                        'id': 'argui-i18n',
                        'name': 'argui-i18n',
                        'dependencies': [{
                            'name': 'argui-core',
                            'req': '^0.1.0',
                            'path': '../argui-core',
                            'kind': 'dev',
                        }],
                    },
                ],
            }
            with patch.object(release, 'ROOT', root), \
                    patch.object(release, 'run', return_value=json.dumps(metadata)):
                with self.assertRaisesRegex(ValueError, 'must be path-only'):
                    release.workspace()

    def test_publication_order_follows_dependencies(self):
        def package(name, *dependencies):
            return {
                'name': name,
                'dependencies': [
                    {'name': dependency, 'path': f'../{dependency}', 'kind': None}
                    for dependency in dependencies
                ],
            }

        packages = [
            package('argui-host', 'argui-render', 'argui-ui'),
            package('argui-ui', 'argui-core'),
            package('argui-media', 'argui-core'),
            package('argui-render', 'argui-core'),
            package('argui-core'),
        ]
        order = release.publication_order(packages)
        self.assertLess(order.index('argui-core'), order.index('argui-render'))
        self.assertLess(order.index('argui-core'), order.index('argui-ui'))
        self.assertLess(order.index('argui-ui'), order.index('argui-host'))
        self.assertLess(order.index('argui-render'), order.index('argui-host'))

    def test_publication_order_rejects_cycles_and_allows_dependents(self):
        dependency = lambda name: {'name': name, 'path': f'../{name}', 'kind': None}
        with self.assertRaises(ValueError):
            release.publication_order([
                {'name': 'argui-a', 'dependencies': [dependency('argui-b')]},
                {'name': 'argui-b', 'dependencies': [dependency('argui-a')]},
            ])
        self.assertEqual(
            release.publication_order([
                {'name': 'argui-ui', 'dependencies': []},
                {'name': 'argui-extension', 'dependencies': [dependency('argui-ui')]},
            ]),
            ['argui-ui', 'argui-extension'],
        )

    def test_package_verifies_every_archive_in_publication_order(self):
        names = ['argui-core', 'argui-render', 'argui']
        targets = []

        def verify(current, actual_names, target):
            self.assertEqual((current, actual_names), ('0.1.0', names))
            self.assertTrue(target.is_dir())
            targets.append(target)

        with tempfile.TemporaryDirectory() as directory:
            shared_package_directory = Path(directory) / 'package'
            shared_package_directory.mkdir()
            (shared_package_directory / 'stale-registry').write_text('old archive')
            with patch.object(release, 'workspace', return_value=('0.1.0', names)), \
                    patch.object(release, 'run') as run, \
                    patch.object(release, 'verify_staged_archives', side_effect=verify):
                release.package_archives()
            self.assertTrue((shared_package_directory / 'stale-registry').exists())
        self.assertEqual(run.call_count, 1)
        self.assertEqual(run.call_args.args, (
            'cargo', 'package', '--locked', '--all-features', '--allow-dirty',
            '--no-verify',
            '--package', 'argui-core', '--package', 'argui-render', '--package', 'argui',
        ))
        target = targets[0]
        self.assertFalse(target.exists())
        self.assertEqual(Path(run.call_args.kwargs['env']['CARGO_TARGET_DIR']), target)

    def test_archive_verification_fetches_before_the_offline_locked_check(self):
        names = ['argui-core', 'argui']

        class Archive:
            def __init__(self, name):
                self.name = name

            def __enter__(self):
                return self

            def __exit__(self, *_):
                return False

            def extractall(self, destination, filter):
                if filter != 'data':
                    raise AssertionError(f'unexpected archive filter: {filter}')
                source = destination / self.name.removesuffix('.crate')
                source.mkdir()
                (source / 'Cargo.toml').write_text('[package]\nname = "staged"\n')

        with tempfile.TemporaryDirectory() as directory:
            target = Path(directory)
            package_directory = target / 'package'
            package_directory.mkdir()
            for name in names:
                (package_directory / f'{name}-0.3.0.crate').touch()

            def archive(path, mode):
                self.assertEqual(mode, 'r:gz')
                return Archive(Path(path).name)

            with patch.object(release.tarfile, 'open', side_effect=archive), \
                    patch.object(release.subprocess, 'run') as run:
                release.verify_staged_archives('0.3.0', names, target)

        self.assertEqual(run.call_count, 2)
        self.assertEqual(run.call_args_list[0].args[0], ['cargo', 'fetch'])
        self.assertEqual(
            run.call_args_list[1].args[0],
            ['cargo', 'check', '--offline', '--locked', '--workspace', '--all-features'],
        )
        self.assertNotEqual(
            Path(run.call_args_list[1].kwargs['env']['CARGO_TARGET_DIR']), target,
        )

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
                patch.object(release, 'publish_package') as publish_package, \
                patch.object(release, 'pending_packages', return_value=[]), \
                patch.object(release.subprocess, 'run', return_value=Mock(returncode=1)):
            release.publish(plan)
        calls = [call.args for call in run.call_args_list]
        self.assertIn('--dry-run', calls[0])
        publish_package.assert_called_once_with('argui')
        self.assertEqual(calls[1][0:4], ('gh', 'release', 'create', plan['tag']))
        self.assertIn(plan['sha'], calls[1])
        self.assertIn('--prerelease', calls[1])

    def test_cargo_publishes_one_crate_at_a_time_in_planned_order(self):
        plan = {'release': True, 'packages': ['argui-core', 'argui-render', 'argui'],
                'all_packages': ['argui-core', 'argui-render', 'argui'],
                'version': '0.1.1', 'tag': 'v0.1.1', 'sha': 'a' * 40}
        with patch.dict(release.os.environ, {'CARGO_REGISTRY_TOKEN': 'test-only'}), \
                patch.object(release, 'run') as run, \
                patch.object(release, 'publish_package') as publish_package, \
                patch.object(release, 'pending_packages', return_value=[]), \
                patch.object(release.subprocess, 'run', return_value=Mock(returncode=0)):
            release.publish(plan)
        self.assertIn('--dry-run', run.call_args_list[0].args)
        self.assertEqual(
            [call.args[0] for call in publish_package.call_args_list],
            plan['packages'],
        )

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
