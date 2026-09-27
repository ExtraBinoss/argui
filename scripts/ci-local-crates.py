#!/usr/bin/env python3
"""Emit Cargo patches for pre-publication CLI app checks in CI.

Generated apps resolve exact Argui releases from crates.io. Before that release
is published, CI points those dependencies at this checkout's publishable crates.
"""

import json
from pathlib import Path
import subprocess


ROOT = Path(__file__).resolve().parent.parent
metadata = json.loads(subprocess.check_output(
    ['cargo', 'metadata', '--locked', '--no-deps', '--format-version', '1'],
    cwd=ROOT,
    text=True,
))
members = set(metadata['workspace_members'])
packages = sorted(
    (package for package in metadata['packages']
     if package['id'] in members and package['publish'] != []),
    key=lambda package: package['name'],
)
print('[patch.crates-io]')
for package in packages:
    directory = str(Path(package['manifest_path']).parent)
    print(f'{package["name"]} = {{ path = {json.dumps(directory)} }}')
