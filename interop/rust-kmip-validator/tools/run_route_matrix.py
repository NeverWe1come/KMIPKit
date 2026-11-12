#!/usr/bin/env python3
"""Run canonical local routes and enforce the shared full-matrix evidence gate.

The gate deliberately fails while any required recipe or network evidence is
missing. Emitted events are actual runner calls, never copied planned rows.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys

import jsonschema

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / 'interop/shared/tools'))
from coverage_matrix import iter_cells
from validate_route_evidence import validate_report

RESPONSE_EVIDENCE_FILES = (
    'rust-response-routes-2.0.json',
    'rust-response-routes-2.1.json',
)
RESPONSE_ACCESSOR_SYMBOLS = {
    'kmipkit_client::ClientBatchOutcome::asynchronous_correlation_value',
    'kmipkit_client::ClientBatchItemResponse::outcome',
    'kmipkit_client::ClientBatchItemResponse::unique_batch_item_id',
    'kmipkit_client::ClientBatchItemResponse::extensions',
    'kmipkit_client::ClientBatchItemResponse::discover_versions_supported_versions',
    'kmipkit_client::ClientBatchResponse::get',
    'kmipkit_client::ClientBatchResponse::len',
    'kmipkit_client::ClientMessageExtension::with_ttlv',
    'kmipkit_client::ClientResponseView::ping',
}


def load_response_evidence(evidence_dir: Path, schema: dict) -> list[dict]:
    reports = []
    for filename in RESPONSE_EVIDENCE_FILES:
        path = evidence_dir / filename
        if not path.is_file():
            continue
        try:
            report = json.loads(path.read_bytes())
        except (OSError, json.JSONDecodeError) as error:
            raise ValueError('response accessor evidence is unreadable') from error
        jsonschema.Draft202012Validator(schema).validate(report)
        version = filename.removesuffix('.json').rsplit('-', 1)[-1]
        if (report['language'] != 'rust'
                or report['server']['id'] != 'controlled-fixture'
                or report['server']['product'] != 'controlled-fixture'
                or report['protocol_versions'] != [version]
                or report['network_combinations'] != [{'encoding': 'TTLV', 'transport': 'raw_tls'}]
                or not report['events']
                or any(event['public_api_symbol'] not in RESPONSE_ACCESSOR_SYMBOLS
                       for event in report['events'])):
            raise ValueError('response accessor evidence is not from the controlled Rust fixture')
        reports.append(report)
    return reports


def route_cell_key(record: dict) -> tuple:
    return (
        record['case_id'], record['route_id'], record['target_id'],
        record.get('workflow_step_index'), record.get('workflow_item_index'),
        record.get('execution_route_id'), record['action'], record['protocol_version'],
        record['encoding'], record['transport'],
    )


def executable_local_cells(cells: list[dict]) -> list[dict]:
    """Select only local calls; response accessors use the controlled peer runner."""
    return [cell for cell in cells
            if cell.get('mode') == 'local'
            and cell.get('public_api_symbol') not in RESPONSE_ACCESSOR_SYMBOLS]


def validate_scope(report: dict, cells: list[dict], catalog: dict, routes: dict,
                   hashes: dict, artifact_sha256: str) -> dict:
    return validate_report(
        report, catalog, routes,
        catalog_sha256=hashes['catalog'],
        route_manifest_sha256=hashes['routes'],
        expected_artifact_sha256=artifact_sha256,
        scope_cells=cells,
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--matrix', type=Path)
    parser.add_argument('--output-dir', type=Path, default=ROOT / 'interop/rust-kmip-validator/results/routes')
    parser.add_argument('--response-evidence-dir', type=Path,
                        default=ROOT / 'interop/rust-kmip-validator/results')
    args = parser.parse_args()
    catalog_bytes = (ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_bytes()
    route_bytes = (ROOT / 'interop/shared/coverage/route-capabilities.json').read_bytes()
    catalog, routes = json.loads(catalog_bytes), json.loads(route_bytes)
    canonical_cells = list(iter_cells(catalog, routes, languages=('rust',)))
    canonical_local_cells = [cell for cell in canonical_cells if cell['mode'] == 'local']
    canonical_network_cells = [cell for cell in canonical_cells if cell['mode'] == 'network']
    provenance = json.loads((ROOT / 'interop/.local/rust-live/provenance.json').read_bytes())
    args.output_dir.mkdir(parents=True, exist_ok=True)
    matrix = args.matrix or args.output_dir / 'rust-local-matrix.jsonl'
    if args.matrix is None:
        with matrix.open('w') as stream:
            for cell in canonical_local_cells:
                stream.write(json.dumps(cell, sort_keys=True, separators=(',', ':')) + '\n')
    selected_cells = [json.loads(line) for line in matrix.read_text().splitlines() if line.strip()]
    selected_local_cells = executable_local_cells(selected_cells)
    selected_response_cells = [
        cell for cell in selected_cells
        if cell.get('mode') == 'local' and cell.get('public_api_symbol') in RESPONSE_ACCESSOR_SYMBOLS
    ]
    evidence_path = args.output_dir / 'rust-local-route-evidence.json'
    evidence_path.unlink(missing_ok=True)
    validator = ROOT / 'interop/rust-kmip-validator/target/debug/rust-kmip-validator'
    runner_matrix = args.output_dir / 'rust-executable-local-matrix.jsonl'
    runner_matrix.write_text(''.join(
        json.dumps(cell, sort_keys=True, separators=(',', ':')) + '\n'
        for cell in selected_local_cells
    ))
    process = subprocess.run([str(validator), 'matrix-local', '--matrix', str(runner_matrix),
                              '--output', str(evidence_path)], capture_output=True, text=True)
    summary = {'schema_valid': False, 'complete_matrix': False, 'invoked_local_cells': 0,
               'failed_local_cells': 0, 'invoked_response_cells': 0,
               'failed_response_cells': 0,
               'gate_failure_code': 'runner_did_not_emit_evidence'}
    summary['required_local_cells'] = len(canonical_local_cells)
    summary['required_network_cells'] = len(canonical_network_cells)
    summary['invoked_network_cells'] = 0
    summary['failed_network_cells'] = 0
    if evidence_path.exists():
        report = json.loads(evidence_path.read_bytes())
        schema = json.loads((ROOT / 'interop/shared/schemas/route-execution-evidence.schema.json').read_bytes())
        try:
            jsonschema.Draft202012Validator(schema).validate(report)
            summary['schema_valid'] = True
        except jsonschema.ValidationError:
            summary['gate_failure_code'] = 'route_evidence_schema_invalid'
        else:
            selected_local_keys = {route_cell_key(cell) for cell in selected_local_cells}
            local_events = [event for event in report['events'] if route_cell_key(event) in selected_local_keys]
            summary['invoked_local_cells'] = len(local_events)
            summary['failed_local_cells'] = sum(event['status'] == 'failed' for event in local_events)
            hashes = {
                'catalog': hashlib.sha256(catalog_bytes).hexdigest(),
                'routes': hashlib.sha256(route_bytes).hexdigest(),
            }
            try:
                if selected_local_cells:
                    validate_scope(report, selected_local_cells, catalog, routes, hashes,
                                   provenance['artifact_sha256'])
                response_reports = load_response_evidence(args.response_evidence_dir, schema)
            except ValueError:
                summary['gate_failure_code'] = 'route_evidence_contract_failed'
                response_reports = None
            except jsonschema.ValidationError:
                summary['gate_failure_code'] = 'route_evidence_schema_invalid'
                summary['schema_valid'] = False
                response_reports = None
            if response_reports is not None:
                selected_response_keys = {route_cell_key(cell) for cell in selected_response_cells}
                response_events = [
                    event for response_report in response_reports
                    for event in response_report['events']
                    if route_cell_key(event) in selected_response_keys
                ]
                summary['invoked_response_cells'] = len(response_events)
                summary['failed_response_cells'] = sum(event['status'] == 'failed' for event in response_events)
                try:
                    for response_report in response_reports:
                        version = response_report['protocol_versions'][0]
                        scope = [cell for cell in selected_response_cells
                                 if cell['protocol_version'] == version]
                        if scope:
                            validate_scope(response_report, scope, catalog, routes, hashes,
                                           provenance['artifact_sha256'])
                except ValueError:
                    summary['gate_failure_code'] = 'response_route_evidence_invalid'
                else:
                    summary['gate_failure_code'] = None
    summary['pending_local_cells'] = max(0, summary['required_local_cells']
        - summary['invoked_local_cells'] - summary['invoked_response_cells'])
    summary['pending_network_cells'] = max(0, summary['required_network_cells']
        - summary['invoked_network_cells'])
    if summary['gate_failure_code'] is None:
        if summary['pending_local_cells']:
            summary['gate_failure_code'] = 'missing_required_matrix_cells'
        elif summary['failed_local_cells'] or summary['failed_response_cells']:
            summary['gate_failure_code'] = 'required_matrix_cells_failed'
        elif summary['pending_network_cells']:
            summary['gate_failure_code'] = 'missing_network_route_cells'
        elif summary['failed_network_cells']:
            summary['gate_failure_code'] = 'required_matrix_cells_failed'
        else:
            summary['complete_matrix'] = True
    (args.output_dir / 'matrix-summary.json').write_text(json.dumps(summary, sort_keys=True, indent=2) + '\n')
    print(json.dumps(summary, sort_keys=True))
    return 0 if process.returncode == 0 and summary['complete_matrix'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
