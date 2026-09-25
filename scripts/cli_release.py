#!/usr/bin/env python3
"""Version-gate and package the Argui CLI for a coordinated GitHub release."""

import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tarfile
import tomllib
import zipfile

from scripts.release import version_key

ROOT = Path(__file__).resolve().parent.parent
TARGETS = {
    'x86_64-unknown-linux-gnu',
    'aarch64-unknown-linux-gnu',
    'x86_64-apple-darwin',
    'aarch64-apple-darwin',
    'x86_64-pc-windows-msvc',
}


def run(*args):
    """Return stdout for a checked command in the repository root."""
    return subprocess.run(args, cwd=ROOT, check=True, capture_output=True,
                          text=True).stdout.strip()


def current_version():
    """Return the shared Rust version after verifying the CLI inherits it."""
    workspace = tomllib.loads((ROOT / 'Cargo.toml').read_text())
    cli = tomllib.loads((ROOT / 'crates/argui-cli/Cargo.toml').read_text())
    version = workspace['workspace']['package']['version']
    version_key(version)
    if cli['package'].get('version') != {'workspace': True}:
        raise ValueError('argui-cli must inherit workspace.package.version')
    return version


def check_newer(version, previous):
    """Reject `version` unless it exceeds every previous release version."""
    current = version_key(version)
    for earlier in previous:
        if version_key(earlier) >= current:
            raise ValueError(f'CLI version {version} must exceed released {earlier}')


def asset_name(tag, target):
    """Return the release archive name for a verified `tag` and `target`."""
    if target not in TARGETS:
        raise ValueError(f'Unsupported CLI release target: {target}')
    if tag != f'v{current_version()}':
        raise ValueError(f'CLI tag {tag} must match workspace v{current_version()}')
    suffix = 'zip' if 'windows' in target else 'tar.gz'
    return f'argui-cli-{tag}-{target}.{suffix}'


def verify_release(tag, target):
    """Require a new, correctly tagged release with no existing target asset."""
    name = asset_name(tag, target)
    head = run('git', 'rev-parse', 'HEAD')
    tagged = run('git', 'rev-parse', f'refs/tags/{tag}^{{commit}}')
    if tagged != head:
        raise ValueError(f'{tag} does not point to this release commit')
    previous = []
    for value in run('git', 'tag', '--list', 'v*').splitlines():
        if value != tag:
            try:
                version_key(value.removeprefix('v'))
            except ValueError:
                continue
            previous.append(value[1:])
    check_newer(current_version(), previous)
    release = json.loads(run('gh', 'release', 'view', tag, '--json', 'assets'))
    existing = {asset['name'] for asset in release['assets']}
    if name in existing or f'{name}.sha256' in existing:
        raise ValueError(f'CLI asset already published for {tag}: {name}')
    return name


def make_archive(tag, target, binary, output):
    """Package `binary` and write its SHA-256 beside the target archive."""
    name = asset_name(tag, target)
    if not binary.is_file():
        raise FileNotFoundError(binary)
    output.mkdir(parents=True, exist_ok=True)
    archive = output / name
    executable = 'argui.exe' if 'windows' in target else 'argui'
    if name.endswith('.zip'):
        with zipfile.ZipFile(archive, 'w', zipfile.ZIP_DEFLATED) as package:
            info = zipfile.ZipInfo(executable)
            info.external_attr = 0o755 << 16
            package.writestr(info, binary.read_bytes(), compress_type=zipfile.ZIP_DEFLATED)
    else:
        with tarfile.open(archive, 'w:gz') as package:
            info = package.gettarinfo(binary, executable)
            info.mode = 0o755
            with binary.open('rb') as source:
                package.addfile(info, source)
    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    checksum = output / f'{name}.sha256'
    checksum.write_text(f'{digest}  {name}\n')
    return archive, checksum


def main():
    """Dispatch the version check, archive generation, or GitHub upload."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=('check', 'package', 'upload'))
    parser.add_argument('--tag', default=None)
    parser.add_argument('--target', required=True)
    args = parser.parse_args()
    tag = args.tag or f'v{current_version()}'
    name = verify_release(tag, args.target) if args.command != 'upload' else asset_name(tag, args.target)
    if args.command == 'check':
        print(f'CLI release verified: {name}')
    elif args.command == 'package':
        binary = ROOT / 'target' / args.target / 'release' / ('argui.exe' if 'windows' in args.target else 'argui')
        archive, checksum = make_archive(tag, args.target, binary, ROOT / 'target/cli-assets')
        print(f'Packaged {archive} and {checksum}')
    else:
        output = ROOT / 'target/cli-assets'
        run('gh', 'release', 'upload', tag, str(output / name), str(output / f'{name}.sha256'))
        print(f'Uploaded CLI asset {name}')


if __name__ == '__main__':
    main()
