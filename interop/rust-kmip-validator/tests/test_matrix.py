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

    def test_all_generic_value_constructors_roundtrip_their_own_data_type(self):
        kinds = {'big_integer': 'BIG-INTEGER', 'boolean': 'BOOLEAN', 'byte_string': 'BYTE-STRING',
                 'date_time': 'DATE-TIME', 'date_time_extended': 'DATE-TIME-EXTENDED',
                 'enumeration': 'ENUMERATION', 'integer': 'INTEGER', 'interval': 'INTERVAL',
                 'long_integer': 'LONG-INTEGER', 'structure': 'STRUCTURE', 'text_string': 'TEXT-STRING'}
        cells = [self.cell('kmipkit_ttlv::Value::' + name, 'KMIPKIT-ELEM-DATA-TYPE-' + target)
                 for name, target in kinds.items()]
        process, report = self.run_matrix(cells)
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(len(report['events']), 11)
        self.assertTrue(all(event['status'] == 'passed' for event in report['events']))

    def test_unknown_case_and_wrong_version_cannot_generate_evidence(self):
        for mutation in ({'case_id': 'not-a-real-case'}, {'protocol_version': '1.4'}):
            cell = self.cell('kmipkit_ttlv::Item::new')
            cell.update(mutation)
            process, report = self.run_matrix([cell])
            self.assertNotEqual(process.returncode, 0)
            self.assertIsNone(report)

    def test_route_cannot_claim_an_unrelated_normative_target(self):
        cell = self.cell('kmipkit_protocol::attribute::Attribute::<kmipkit_protocol::attribute::typed::CryptographicLength>::new')
        cell['target_id'] = 'KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-AES-00000003'
        process, report = self.run_matrix([cell])
        self.assertNotEqual(process.returncode, 0)
        self.assertIsNone(report)

    def test_constructor_cannot_pass_when_it_changes_the_normative_tag(self):
        cell = self.cell('kmipkit_protocol::attribute::Attribute::<kmipkit_protocol::attribute::typed::Comment>::new',
                         'KMIPKIT-ELEM-ATTRIBUTE-COMMENT')
        cell['target_id'] = 'KMIPKIT-ELEM-ATTRIBUTE-DESCRIPTION'
        process, report = self.run_matrix([cell])
        self.assertNotEqual(process.returncode, 0)
        self.assertIsNone(report)

    def test_shared_gate_rejects_incomplete_and_duplicate_actual_evidence(self):
        process, report = self.run_matrix([self.cell('kmipkit_ttlv::Item::new')])
        self.assertEqual(process.returncode, 0, process.stderr)
        arguments = {'catalog_sha256': hashlib.sha256(CATALOG_PATH.read_bytes()).hexdigest(),
                     'route_manifest_sha256': hashlib.sha256(ROUTES_PATH.read_bytes()).hexdigest(),
                     'expected_artifact_sha256': report['artifact_sha256']}
        with self.assertRaisesRegex(ValueError, 'missing route execution cells'):
            validate_report(report, CATALOG, ROUTES, **arguments)
        report['events'].append(dict(report['events'][0]))
        with self.assertRaisesRegex(ValueError, 'duplicate route execution cell'):
            validate_report(report, CATALOG, ROUTES, **arguments)

    def test_named_open_wrappers_preserve_exact_numeric_codes(self):
        cells = [self.cell('kmipkit_protocol::ResultStatus::from_raw', 'KMIPKIT-ELEM-ENUM-VALUE-RESULT-STATUS-SUCCESS-00000000'),
                 self.cell('kmipkit_protocol::ObjectType::from_raw', 'KMIPKIT-ELEM-OBJECT-TYPE-CERTIFICATE')]
        process, report = self.run_matrix(cells)
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual([event['request_tree_sha256'] for event in report['events']],
                         [hashlib.sha256(value).hexdigest() for value in (b'[[4325503,5,0]]', b'[[4325463,5,1]]')])

    def test_named_tag_and_structure_paths_execute_roundtrips(self):
        cells = [self.cell('kmipkit_ttlv::RawTag::' + method, 'KMIPKIT-ELEM-TAG-42002A')
                 for method in ('new', 'try_checked')]
        cells.extend(self.cell('kmipkit_ttlv::Structure::' + method, 'KMIPKIT-ELEM-DATA-TYPE-STRUCTURE')
                     for method in ('new', 'try_push'))
        process, report = self.run_matrix(cells)
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(len(report['events']), 4)
        self.assertEqual(report['events'][2]['request_tree_sha256'], report['events'][3]['request_tree_sha256'])

    def test_ci_gate_refuses_a_partial_real_route_run(self):
        with tempfile.TemporaryDirectory() as directory:
            matrix = Path(directory) / 'one-cell.jsonl'
            matrix.write_text(json.dumps(self.cell('kmipkit_ttlv::Item::new')) + '\n')
            process = subprocess.run([sys.executable, str(ROOT / 'interop/rust-kmip-validator/tools/run_route_matrix.py'),
                '--matrix', str(matrix), '--output-dir', directory, '--validator', str(VALIDATOR)],
                capture_output=True, text=True, timeout=30)
            self.assertNotEqual(process.returncode, 0)
            summary_path = Path(directory) / 'matrix-summary.json'
            self.assertTrue(summary_path.exists(), process.stderr)
            summary = json.loads(summary_path.read_bytes())
            self.assertTrue(summary['schema_valid'])
            self.assertFalse(summary['complete_matrix'])
            self.assertEqual(summary['invoked_local_cells'], 1)
            self.assertEqual(summary['gate_failure_code'], 'missing_required_matrix_cells')

    def test_missing_request_recipe_cannot_pass_an_empty_structure(self):
        cell = self.cell('kmipkit_ttlv::Item::new', 'KMIPKIT-ELEM-OP-C2S-QUERY')
        process, report = self.run_matrix([cell])
        self.assertNotEqual(process.returncode, 0)
        self.assertEqual(report['events'], [])
        self.assertIn('pending_recipe', process.stderr)

    def test_parameter_modifier_projects_the_exact_constructed_target(self):
        cell = self.cell('kmipkit_protocol::CryptographicParametersBuilder::with_padding_method',
                         'KMIPKIT-ELEM-ENUM-VALUE-PADDING-METHOD-NONE-00000001')
        process, report = self.run_matrix([cell])
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(report['events'][0]['request_tree_sha256'], hashlib.sha256(b'[[4325471,5,1]]').hexdigest())

    def test_parameter_constructor_preserves_source_permitted_empty_model(self):
        cell = self.cell('kmipkit_protocol::CryptographicParametersBuilder::new', 'KMIPKIT-ELEM-ATTRIBUTE-CRYPTOGRAPHIC-PARAMETERS')
        process, report = self.run_matrix([cell])
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(report['events'][0]['request_tree_sha256'], hashlib.sha256(b'[[4325419,1,[]]]').hexdigest())

    def test_every_parameter_setter_projects_its_own_normative_field(self):
        symbols = {route['public_api_symbol'] for route in ROUTES['routes']
                   if route['language'] == 'rust' and route['public_api_symbol'].startswith('kmipkit_protocol::CryptographicParametersBuilder::with_')}
        targets = {target['target_id']: target for target in CATALOG['normative_targets']}
        selected = {}
        for cell in iter_cells(CATALOG, ROUTES, languages=('rust',)):
            if cell['public_api_symbol'] not in symbols or cell['public_api_symbol'] in selected:
                continue
            target = targets[cell['target_id']]
            if target['kind'] == 'enumeration_value' and target.get('allocation') != 'assigned':
                continue
            if target['kind'] in ('enumeration_value', 'structure_member'):
                selected[cell['public_api_symbol']] = cell
        self.assertEqual(set(selected), symbols)
        process, report = self.run_matrix(list(selected.values()))
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(len(report['events']), len(symbols))
        self.assertTrue(all(event['status'] == 'passed' for event in report['events']))

    def test_credential_variants_and_nonce_invoke_their_real_model_routes(self):
        kinds = {'UsernameAndPasswordCredential': 'USERNAME-AND-PASSWORD', 'DeviceCredential': 'DEVICE',
                 'AttestationCredential': 'ATTESTATION', 'HashedPasswordCredential': 'HASHED-PASSWORD'}
        cells = [self.cell('kmipkit_protocol::' + owner + '::try_from_ttlv', 'KMIPKIT-ELEM-CREDENTIAL-' + target)
                 for owner, target in kinds.items()]
        cells.append(self.cell('kmipkit_protocol::Nonce::try_from_ttlv', 'KMIPKIT-ELEM-MESSAGE-FIELD-9-14-NONCE'))
        process, report = self.run_matrix(cells)
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(len(report['events']), 5)
        self.assertTrue(all(event['status'] == 'passed' for event in report['events']))
        encoded = json.dumps(report)
        self.assertNotIn('KMIPKit', encoded, 'fixture credentials must stay out of evidence')

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
