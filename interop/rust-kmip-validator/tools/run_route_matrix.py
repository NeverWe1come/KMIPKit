#!/usr/bin/env python3
"""Run canonical local routes and enforce the shared full-matrix evidence gate.

The gate deliberately fails while any required recipe or network evidence is
missing. Emitted events are actual runner calls, never copied planned rows.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

import jsonschema

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / 'interop/shared/tools'))
from coverage_matrix import iter_cells
from validate_route_evidence import validate_collection


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--matrix', type=Path)
    parser.add_argument('--output-dir', type=Path, default=ROOT / 'interop/rust-kmip-validator/results/routes')
    parser.add_argument('--validator', type=Path, default=Path(os.environ.get('KMIPKIT_VALIDATOR_BIN', ROOT / 'interop/rust-kmip-validator/target/debug/rust-kmip-validator')))
    args = parser.parse_args()
    catalog_bytes = (ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_bytes()
    route_bytes = (ROOT / 'interop/shared/coverage/route-capabilities.json').read_bytes()
    catalog, routes = json.loads(catalog_bytes), json.loads(route_bytes)
    provenance = json.loads((ROOT / 'interop/.local/rust-live/provenance.json').read_bytes())
    args.output_dir.mkdir(parents=True, exist_ok=True)
    matrix = args.matrix or args.output_dir / 'rust-local-matrix.jsonl'
    if args.matrix is None:
        with matrix.open('w') as stream:
            for cell in iter_cells(catalog, routes, languages=('rust',)):
                if cell['mode'] == 'local':
                    stream.write(json.dumps(cell, sort_keys=True, separators=(',', ':')) + '\n')
    evidence_path = args.output_dir / 'rust-local-route-evidence.json'
    evidence_path.unlink(missing_ok=True)
    process = subprocess.run([str(args.validator), 'matrix-local', '--matrix', str(matrix),
                              '--output', str(evidence_path)], capture_output=True, text=True)
    summary = {'schema_valid': False, 'complete_matrix': False, 'invoked_local_cells': 0,
               'failed_local_cells': 0, 'gate_failure_code': 'runner_did_not_emit_evidence'}
    summary['required_local_cells'] = sum(1 for line in matrix.read_text().splitlines() if line.strip())
    if evidence_path.exists():
        report = json.loads(evidence_path.read_bytes())
        schema = json.loads((ROOT / 'interop/shared/schemas/route-execution-evidence.schema.json').read_bytes())
        try:
            jsonschema.Draft202012Validator(schema).validate(report)
            summary['schema_valid'] = True
        except jsonschema.ValidationError:
            summary['gate_failure_code'] = 'route_evidence_schema_invalid'
        else:
            summary['invoked_local_cells'] = len(report['events'])
            summary['failed_local_cells'] = sum(event['status'] == 'failed' for event in report['events'])
            try:
                checked = validate_collection([report], catalog, routes,
                    catalog_sha256=hashlib.sha256(catalog_bytes).hexdigest(),
                    route_manifest_sha256=hashlib.sha256(route_bytes).hexdigest(),
                    expected_artifact_sha256=provenance['artifact_sha256'])
                summary['complete_matrix'] = checked['complete_matrix'] and checked['all_invocations_passed']
                summary['gate_failure_code'] = None if summary['complete_matrix'] else 'required_matrix_cells_failed'
            except ValueError as error:
                summary['gate_failure_code'] = ('missing_required_matrix_cells'
                    if str(error).startswith(('missing route execution cells:', 'route evidence collection misses full matrix cells:'))
                    else 'route_evidence_contract_failed')
    summary['pending_local_cells'] = max(0, summary['required_local_cells'] - summary['invoked_local_cells'])
    (args.output_dir / 'matrix-summary.json').write_text(json.dumps(summary, sort_keys=True, indent=2) + '\n')
    print(json.dumps(summary, sort_keys=True))
    return 0 if process.returncode == 0 and summary['complete_matrix'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
