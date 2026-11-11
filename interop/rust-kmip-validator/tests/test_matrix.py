"""Executable KMIPKIT-0092 route evidence regression contracts."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

import jsonschema

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / 'interop/shared/tools'))
from coverage_matrix import iter_cells
from validate_route_evidence import validate_report

VALIDATOR = Path(os.environ.get('KMIPKIT_VALIDATOR_BIN', ROOT / 'interop/rust-kmip-validator/target/debug/rust-kmip-validator'))
CATALOG_PATH = ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json'
ROUTES_PATH = ROOT / 'interop/shared/coverage/route-capabilities.json'
CATALOG = json.loads(CATALOG_PATH.read_bytes())
ROUTES = json.loads(ROUTES_PATH.read_bytes())
SCHEMA = json.loads((ROOT / 'interop/shared/schemas/route-execution-evidence.schema.json').read_bytes())


class MatrixContract(unittest.TestCase):
    def cell(self, symbol, target='KMIPKIT-ELEM-ATTRIBUTE-CRYPTOGRAPHIC-LENGTH'):
        return next(cell for cell in iter_cells(CATALOG, ROUTES, languages=('rust',))
                    if cell['public_api_symbol'] == symbol and (target is None or cell['target_id'] == target))

    def run_matrix(self, cells):
        with tempfile.TemporaryDirectory() as work:
            matrix = Path(work) / 'matrix.jsonl'
            output = Path(work) / 'evidence.json'
            matrix.write_text(''.join(json.dumps(cell) + '\n' for cell in cells))
            process = subprocess.run([str(VALIDATOR), 'matrix-local', '--matrix', str(matrix),
                '--output', str(output)], capture_output=True, text=True, timeout=30)
            return process, json.loads(output.read_bytes()) if output.exists() else None

    def test_named_typed_constructor_emits_actual_cell_evidence(self):
        cell = self.cell('kmipkit_protocol::attribute::Attribute::<kmipkit_protocol::attribute::typed::CryptographicLength>::new')
        process, report = self.run_matrix([cell])
        self.assertEqual(process.returncode, 0, process.stderr)
        jsonschema.Draft202012Validator(SCHEMA).validate(report)
        event = report['events'][0]
        self.assertEqual(event['public_api_symbol'], cell['public_api_symbol'])
        self.assertEqual(event['invocation_count'], 1)
        self.assertEqual(event['status'], 'passed')
        self.assertEqual(event['request_tree_sha256'], hashlib.sha256(b'[[4325418,2,256]]').hexdigest())
        self.assertNotIn('wire_bytes_sha256', event, 'wire bytes are not public model evidence')

    def test_generic_constructor_roundtrip_preserves_case_value(self):
        integer = self.cell('kmipkit_ttlv::Item::new')
        enumeration = self.cell('kmipkit_ttlv::Item::new', 'KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-AES-00000003')
        process, report = self.run_matrix([integer, enumeration])
        self.assertEqual(process.returncode, 0, process.stderr)
        hashes = [event['request_tree_sha256'] for event in report['events']]
        self.assertNotEqual(*hashes, 'a fixed unrelated fixture cannot satisfy different target values')

    def test_forged_symbol_is_rejected_without_false_positive(self):
        cell = self.cell('kmipkit_ttlv::Item::new')
        cell['public_api_symbol'] = 'kmipkit_ttlv::Value::boolean'
        process, report = self.run_matrix([cell])
        self.assertNotEqual(process.returncode, 0)
        self.assertIsNone(report)

    def test_duplicate_cells_fail_before_invocation(self):
        cell = self.cell('kmipkit_ttlv::Item::new')
        process, report = self.run_matrix([cell, cell])
        self.assertNotEqual(process.returncode, 0)
        self.assertIsNone(report)

    def test_unfinished_recipe_cannot_be_reported_as_api_unavailable(self):
        cell = self.cell('kmipkit_client::ClientBatchOutcome::Activate', None)
        process, report = self.run_matrix([cell])
        self.assertNotEqual(process.returncode, 0)
        self.assertIsNotNone(report, process.stderr)
        self.assertEqual(report['events'], [])
        self.assertIn('pending_recipe', process.stderr)
        self.assertNotIn('api_unavailable', process.stderr)


if __name__ == '__main__':
    unittest.main()
