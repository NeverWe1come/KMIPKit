#!/usr/bin/env python3
"""Bind the Rust consumer to canonical shared matrix rows at build time."""
import hashlib
import json
import os
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / 'interop/shared/tools'))
from coverage_matrix import iter_cells
from matrix_hashes import contained_output_path


def main():
    if len(sys.argv) != 2 or not os.environ.get('OUT_DIR'):
        raise SystemExit('Cargo OUT_DIR and one generated output path are required')
    try:
        output = contained_output_path(sys.argv[1], os.environ['OUT_DIR'])
    except ValueError as error:
        raise SystemExit(str(error)) from error
    catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_bytes())
    manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_bytes())
    hashes = sorted(hashlib.sha256(json.dumps(
        row, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()).hexdigest()
        for row in iter_cells(catalog, manifest, languages=('rust',)))
    if len(hashes) != len(set(hashes)):
        raise SystemExit('canonical Rust matrix has duplicate rows')
    output.write_text('\n'.join(hashes) + '\n')


if __name__ == '__main__':
    main()
