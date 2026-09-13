#!/usr/bin/env python3
"""Validate coordinated crate versions and publish an explicit release commit."""

import argparse
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import time
import tomllib
import urllib.error
import urllib.request

ROOT = Path(__file__).resolve().parent.parent
REGISTRY = 'https://crates.io/api/v1/crates/'
VERSION = re.compile(r'(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-([0-9A-Za-z.-]+))?')


def version_key(value):
    match = VERSION.fullmatch(value)
    if not match:
        raise ValueError(f'Invalid release version: {value}')
    prerelease = match[4]
    identifiers = prerelease.split('.') if prerelease else []
    if any(not part or (part.isdigit() and len(part) > 1 and part[0] == '0') for part in identifiers):
        raise ValueError(f'Invalid prerelease: {value}')
    return (*map(int, match.group(1, 2, 3)), not prerelease,
            tuple((0, int(part)) if part.isdigit() else (1, part) for part in identifiers))


def run(*args, capture=False):
    result = subprocess.run(args, cwd=ROOT, check=True, text=True,
                            stdout=subprocess.PIPE if capture else None)
    return result.stdout.strip() if capture else None


def publication_order(packages):
    """Return publishable packages after their internal dependencies."""
    by_name = {package['name']: package for package in packages}
    dependencies = {}
    for name, package in by_name.items():
        dependencies[name] = {
            dependency['name']
            for dependency in package['dependencies']
            if dependency['kind'] != 'dev'
            and dependency.get('path') is not None
            and dependency['name'] in by_name
        }
    if any('argui' in required for name, required in dependencies.items() if name != 'argui'):
        raise ValueError('argui must remain the final facade in the publication graph')

    remaining = set(by_name)
    ordered = []
    while remaining:
        ready = [name for name in remaining if not dependencies[name] & remaining]
        if not ready:
            cycle = ', '.join(sorted(remaining))
            raise ValueError(f'Internal dependency cycle: {cycle}')
        # The root facade is a leaf. Holding it until every internal crate has
        # been packaged makes the release contract explicit and deterministic.
        name = min(ready, key=lambda candidate: (candidate == 'argui', candidate))
        ordered.append(name)
        remaining.remove(name)
    return ordered


def workspace():
    manifest = tomllib.loads((ROOT / 'Cargo.toml').read_text())
    current = manifest['workspace']['package']['version']
    version_key(current)
    metadata = json.loads(run('cargo', 'metadata', '--locked', '--no-deps',
                              '--format-version', '1', capture=True))
    packages = [p for p in metadata['packages'] if p['id'] in metadata['workspace_members']]
    publishable = {p['name']: p for p in packages if p['publish'] != []}
    for name, package in publishable.items():
        if package['version'] != current:
            raise ValueError(f'{name}: version must match workspace {current}')
        for field in ('description', 'license', 'repository', 'homepage', 'readme'):
            if not package[field]:
                raise ValueError(f'{name}: missing {field}')
        for dep in package['dependencies']:
            if dep['kind'] == 'dev' or dep.get('path') is None:
                continue
            if dep['name'] not in publishable:
                raise ValueError(f'{name}: unpublished dependency {dep["name"]}')
            if dep['req'] != f'^{current}':
                raise ValueError(f'{name}: {dep["name"]} must require version {current}')
    return current, publication_order(publishable.values())


def package_archives():
    current, names = workspace()
    args = ['cargo', 'package', '--locked', '--all-features', '--allow-dirty']
    for name in names:
        args.extend(['--package', name])
    run(*args)
    print(f'Verified {len(names)} crates.io archives at version {current}')


def registry_versions(name):
    headers = {
        'User-Agent': 'Argui release automation (https://github.com/ExtraBinoss/argui)',
    }
    request = urllib.request.Request(REGISTRY + name, headers=headers)
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            versions = json.load(response)['versions']
    except urllib.error.HTTPError as error:
        if error.code == 404:
            return []
        raise
    owners = urllib.request.Request(REGISTRY + name + '/owners', headers=headers)
    with urllib.request.urlopen(owners, timeout=30) as response:
        users = json.load(response)['users']
    if not any(user['login'].lower() == 'extrabinoss' for user in users):
        raise ValueError(f'{name}: the existing crate is not owned by ExtraBinoss')
    return versions


def pending_packages(names, current, fetch=registry_versions):
    pending = []
    for name in names:
        versions = fetch(name)
        exact = [entry for entry in versions if entry['num'] == current]
        if exact:
            if exact[0]['yanked']:
                raise ValueError(f'{name} {current} is yanked; choose a new version')
            continue
        if any(version_key(entry['num'].split('+')[0]) >= version_key(current) for entry in versions):
            raise ValueError(f'{name}: release {current} must exceed published versions')
        pending.append(name)
    return pending


def release_plan(base, fetch=registry_versions):
    current, names = workspace()
    sha = run('git', 'rev-parse', 'HEAD', capture=True)
    result = {'release': False, 'version': current, 'sha': sha, 'tag': f'v{current}',
              'packages': [], 'all_packages': names}
    if '[PUBLISH]' not in run('git', 'show', '-s', '--format=%B', 'HEAD', capture=True):
        return result | {'reason': 'The latest commit has no [PUBLISH] marker'}
    if not re.fullmatch(r'[0-9a-f]{40}', base) or base == '0' * 40:
        raise ValueError('A real pre-push commit SHA is required to compare versions')
    run('git', 'merge-base', '--is-ancestor', base, sha)
    previous = tomllib.loads(run('git', 'show', f'{base}:Cargo.toml', capture=True))
    old = previous['workspace']['package']['version']
    if version_key(current) < version_key(old):
        raise ValueError(f'Version decreased from {old} to {current}')
    if current == old:
        return result | {'reason': f'Workspace version {current} did not change'}
    tag = subprocess.run(['git', 'rev-parse', '--verify', f'refs/tags/v{current}^{{commit}}'],
                         cwd=ROOT, text=True, capture_output=True)
    if tag.returncode == 0 and tag.stdout.strip() != sha:
        raise ValueError(f'Tag v{current} already points to another commit')
    pending = pending_packages(names, current, fetch)
    return result | {'release': True, 'packages': pending, 'previous': old,
                     'reason': f'Version increased from {old} to {current}'}


def bump(value):
    current, _ = workspace()
    if version_key(value) <= version_key(current):
        raise ValueError(f'New version must exceed {current}')
    path = ROOT / 'Cargo.toml'
    text = path.read_text()
    text = text.replace(f'version = "{current}"', f'version = "{value}"')
    path.write_text(text)
    run('cargo', 'update', '--workspace', '--offline')
    workspace()
    print(f'Updated workspace and internal dependencies: {current} → {value}')


def publish(plan):
    if not plan['release']:
        print(plan['reason'])
        return
    if plan['packages']:
        if not os.environ.get('CARGO_REGISTRY_TOKEN'):
            raise ValueError('CARGO_REGISTRY_TOKEN is required to publish')
        args = ['cargo', 'publish', '--registry', 'crates-io', '--locked', '--all-features']
        for name in plan['packages']:
            args.extend(['--package', name])
        run(*args, '--dry-run')
        for name in plan['packages']:
            run('cargo', 'publish', '--registry', 'crates-io', '--locked',
                '--all-features', '--package', name)
    # GitHub must never announce a release whose crates are not available yet.
    for attempt in range(12):
        pending = pending_packages(plan['all_packages'], plan['version'])
        if not pending:
            break
        if attempt == 11:
            raise ValueError(f'Registry still missing published crates: {pending}')
        print(f'Waiting for the registry: {", ".join(pending)}', flush=True)
        time.sleep(5)
    existing = subprocess.run(['gh', 'release', 'view', plan['tag'], '--json', 'tagName'],
                              cwd=ROOT, capture_output=True)
    if existing.returncode == 0:
        print(f'GitHub release {plan["tag"]} already exists')
        return
    notes = (f'Argui {plan["version"]} is available on crates.io.\n\n'
             f'```toml\n[dependencies]\nargui = "{plan["version"]}"\n```\n\n'
             '[Website and live widget gallery](https://extrabinoss.github.io/argui/)\n\n'
             'Published crates:\n\n' + '\n'.join(
                 f'- [{name}](https://crates.io/crates/{name}/{plan["version"]})'
                 for name in plan['all_packages']))
    with tempfile.NamedTemporaryFile(mode='w', suffix='.md') as body:
        body.write(notes)
        body.flush()
        args = ['gh', 'release', 'create', plan['tag'], '--target', plan['sha'],
                '--title', f'Argui {plan["version"]}', '--generate-notes', '--notes-file', body.name]
        if '-' in plan['version']:
            args.append('--prerelease')
        run(*args)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    commands.add_parser('check')
    commands.add_parser('package')
    commands.add_parser('bump').add_argument('version')
    for name in ('plan', 'publish'):
        command = commands.add_parser(name)
        command.add_argument('--base', required=True)
    args = parser.parse_args()
    if args.command == 'check':
        current, names = workspace()
        print(f'Validated {len(names)} publishable crates at version {current}')
        print('Publication order: ' + ' -> '.join(names))
    elif args.command == 'package':
        package_archives()
    elif args.command == 'bump':
        bump(args.version)
    else:
        plan = release_plan(args.base)
        print(json.dumps(plan, indent=2))
        if args.command == 'publish':
            publish(plan)


if __name__ == '__main__':
    try:
        main()
    except (ValueError, subprocess.CalledProcessError, urllib.error.URLError) as error:
        print(f'Release stopped: {error}', file=sys.stderr)
        sys.exit(1)
