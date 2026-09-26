#!/usr/bin/env python3
"""Regenerate the optional TSX Tabler catalogue from pinned icondata_tb 0.1.0.

Run explicitly after the Rust dependency changes; normal gallery builds do not
scan Cargo's registry or rewrite this checked-in development-only module.
"""

import argparse
import glob
import json
import os
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUTPUT = ROOT / 'src/tabler-catalog.generated.ts'
VERSION = '0.1.0'
EXPECTED = 5944


def icondata_source():
    cargo_home = Path(os.environ.get('CARGO_HOME', Path.home() / '.cargo'))
    paths = sorted(glob.glob(str(cargo_home / f'registry/src/*/icondata_tb-{VERSION}/src/lib.rs')))
    if not paths:
        raise SystemExit(f'icondata_tb {VERSION} missing; run cargo fetch')
    return Path(paths[0]).read_text()


def field(body, name):
    match = re.search(r'\b' + name + r': (?:Some\("([^\"]*)"\)|None)', body)
    if match is None:
        raise ValueError(f'missing {name}')
    return match.group(1)


def catalogue(source):
    rows = re.findall(
        r'pub static (Tb\w+): &icondata_core::IconData = &icondata_core::IconData \{(.*?)\n\};',
        source, re.S,
    )
    if len(rows) != EXPECTED:
        raise ValueError(f'expected {EXPECTED} Tabler definitions, found {len(rows)}')
    result = {}
    attributes = [
        ('width', 'width'), ('height', 'height'), ('view_box', 'viewBox'),
        ('stroke_linecap', 'stroke-linecap'), ('stroke_linejoin', 'stroke-linejoin'),
        ('stroke_width', 'stroke-width'), ('stroke', 'stroke'), ('fill', 'fill'),
    ]
    for rust_name, body in rows:
        short = rust_name[2:]
        filled = short.endswith('Filled')
        if not filled and not short.endswith('Outline'):
            raise ValueError(f'unexpected icon variant {rust_name}')
        short = short[:-6] if filled else short[:-7]
        short = re.sub(r'([A-Z])([A-Z][a-z])', r'\1-\2', short)
        short = re.sub(r'([a-z0-9])([A-Z])', r'\1-\2', short)
        name = short.lower() + ('-filled' if filled else '')
        if name in result:
            raise ValueError(f'duplicate icon name {name}')
        data = re.search(r'\bdata: r###"(.*?)"###', body, re.S)
        if data is None:
            raise ValueError(f'missing SVG path data in {rust_name}')
        xml_attributes = ' '.join(
            f'{html}="{value}"'
            for rust, html in attributes
            if (value := field(body, rust)) is not None
        )
        svg = f'<svg xmlns="http://www.w3.org/2000/svg" {xml_attributes}>{data.group(1)}</svg>'
        hash_value = 0xCBF29CE484222325
        for byte in ('dev-tabler:' + name + ':' + svg).encode():
            hash_value = ((hash_value ^ byte) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
        asset_id = hash_value & 0x1FFFFFFFFFFFFF
        if not asset_id:
            raise ValueError(f'zero asset ID for {name}')
        result[name] = (asset_id, svg)
    if len({asset_id for asset_id, _ in result.values()}) != EXPECTED:
        raise ValueError('development icon asset ID collision')
    return result


def render(icons):
    lines = [
        '// Generated from icondata_tb 0.1.0 (Tabler Icons, MIT). Development only.',
        'export const tablerCatalogue: Record<string, { id: number; svg: string }> = {',
    ]
    for name, (asset_id, svg) in sorted(icons.items()):
        lines.append(
            f'  {json.dumps(name)}: {{ id: {asset_id}, svg: {json.dumps(svg, ensure_ascii=False)} }},'
        )
    return '\n'.join([*lines, '}', ''])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true', help='verify checked-in output')
    options = parser.parse_args()
    icons = catalogue(icondata_source())
    output = render(icons)
    if options.check:
        if not OUTPUT.exists() or OUTPUT.read_text() != output:
            raise SystemExit(f'{OUTPUT} is stale; rerun {Path(__file__).name}')
    else:
        OUTPUT.write_text(output)
    print(f'{len(icons)} Tabler icons, {len(output.encode())} bytes; {"verified" if options.check else "written"}')


if __name__ == '__main__':
    main()
