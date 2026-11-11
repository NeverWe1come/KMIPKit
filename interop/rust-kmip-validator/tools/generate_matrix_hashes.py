#!/usr/bin/env python3
"""Bind the Rust consumer to canonical shared matrix rows at build time."""
import hashlib
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / 'interop/shared/tools'))
from coverage_matrix import iter_cells

catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_bytes())
manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_bytes())
hashes = sorted(hashlib.sha256(json.dumps(row, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()).hexdigest()
                for row in iter_cells(catalog, manifest, languages=('rust',)))
if len(hashes) != len(set(hashes)):
    raise SystemExit('canonical Rust matrix has duplicate rows')
Path(sys.argv[1]).write_text('\n'.join(hashes) + '\n')
