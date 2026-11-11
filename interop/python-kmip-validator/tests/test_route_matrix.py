"""KMIPKIT-0092: executable route evidence must fail closed on missing recipes."""
import copy
import importlib
import json
from pathlib import Path
import sys
import unittest

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / 'interop/shared/tools'))
from coverage_matrix import iter_cells
from validate_route_evidence import validate_report


class RouteMatrixTests(unittest.TestCase):
    def setUp(self):
        self.assertIsNotNone(
            importlib.util.find_spec('kmipkit_validator.route_matrix'),
            'the installed-package consumer needs an executable route-matrix runner',
        )
        self.runner = importlib.import_module('kmipkit_validator.route_matrix')
        self.catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        self.manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        self.artifact = {'library_commit': self.manifest['library_commit'], 'artifact_sha256': 'a' * 64}

    def integer_fixture(self):
        catalog = copy.deepcopy(self.catalog)
        catalog['test_cases'] = [next(case for case in catalog['test_cases']
                                     if case['requires'].get('target_kind') == 'data_type'
                                     and case['requires'].get('normative_target_id', '').endswith('INTEGER'))]
        manifest = copy.deepcopy(self.manifest)
        wanted = {'kmipkit.ttlv.ttlv_value_integer', 'kmipkit.ttlv.ttlv_item_create',
                  'kmipkit.ttlv.ttlv_decode', 'kmipkit.ttlv.ttlv_item_view_from_item'}
        manifest['routes'] = [route for route in manifest['routes']
                              if route['language'] == 'python' and route['public_api_symbol'] in wanted]
        manifest['routes'].append(next(route for route in self.manifest['routes']
                                       if route['public_api_symbol'] == 'kmipkit.extensions.client_batch_item_ping'))
        return catalog, manifest

    def test_generic_routes_call_named_apis_and_compare_constructed_and_decoded_values(self):
        catalog, manifest = self.integer_fixture()
        report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact)
        self.assertEqual([], gaps)
        self.assertEqual(len(list(iter_cells(catalog, manifest, languages=('python',)))), len(report['events']))
        summary = validate_report(report, catalog, manifest,
                                  catalog_sha256=report['catalog_sha256'],
                                  route_manifest_sha256=report['route_manifest_sha256'],
                                  expected_artifact_sha256=self.artifact['artifact_sha256'])
        self.assertTrue(summary['all_invocations_passed'])
        self.assertTrue(all(event['invocation_count'] == 1 for event in report['events']))
        jsonschema = importlib.import_module('jsonschema')
        schema = json.loads((ROOT / 'interop/shared/schemas/route-execution-evidence.schema.json').read_text())
        jsonschema.validate(report, schema)

    def test_missing_recipe_is_pending_coverage_and_never_becomes_api_unavailability(self):
        catalog, manifest = self.integer_fixture()
        route = copy.deepcopy(manifest['routes'][0])
        route['route_id'] = 'python.test.unimplemented.public.route'
        route['public_api_symbol'] = 'kmipkit.ttlv.codec_limits_defaults'
        manifest['routes'] = [route, manifest['routes'][-1]]
        report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact)
        self.assertEqual([], report['events'])
        self.assertTrue(gaps)
        self.assertEqual({'recipe_pending'}, {gap['failure_code'] for gap in gaps})
        with self.assertRaisesRegex(ValueError, 'missing route execution cells'):
            validate_report(report, catalog, manifest, catalog_sha256=report['catalog_sha256'],
                            route_manifest_sha256=report['route_manifest_sha256'],
                            expected_artifact_sha256=self.artifact['artifact_sha256'])

    def test_duplicate_or_forged_positive_cells_are_rejected(self):
        catalog, manifest = self.integer_fixture()
        report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact)
        self.assertFalse(gaps)
        for mutation, message in (
            (lambda value: value['events'].append(copy.deepcopy(value['events'][0])), 'duplicate'),
            (lambda value: value['events'][0].update(public_api_symbol='kmipkit.fake'), 'public API symbol'),
            (lambda value: value['events'][0].update(invocation_count=0), 'not actually invoked'),
        ):
            altered = copy.deepcopy(report)
            mutation(altered)
            with self.assertRaisesRegex(ValueError, message):
                validate_report(altered, catalog, manifest, catalog_sha256=report['catalog_sha256'],
                                route_manifest_sha256=report['route_manifest_sha256'],
                                expected_artifact_sha256=self.artifact['artifact_sha256'])

    def test_exception_text_and_values_never_escape_into_report_or_pending_coverage(self):
        catalog, manifest = self.integer_fixture()
        from unittest.mock import patch
        with patch.object(self.runner, 'execute_cell', side_effect=RuntimeError('private-key-sentinel')):
            report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact)
        self.assertNotIn('private-key-sentinel', json.dumps([report, gaps]))
        self.assertEqual({'recipe_execution_failed'}, {gap['failure_code'] for gap in gaps})

    def test_call_observation_does_not_confuse_shared_synchronization_wrappers(self):
        from kmipkit import ttlv
        with self.runner.CallTrace(['kmipkit.ttlv.ttlv_decode']) as trace:
            with ttlv.codec_limits_defaults():
                pass
        self.assertNotIn('kmipkit.ttlv.ttlv_decode', trace.observed)

    def test_ping_network_cells_execute_public_client_over_all_supported_combinations(self):
        catalog = copy.deepcopy(self.catalog)
        catalog['test_cases'] = [next(case for case in catalog['test_cases'] if case['case_id'] == 'test-op-ping')]
        manifest = copy.deepcopy(self.manifest)
        manifest['routes'] = [route for route in manifest['routes']
                              if route['public_api_symbol'] == 'kmipkit.extensions.client_batch_item_ping']
        report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact)
        self.assertFalse(gaps)
        self.assertEqual(18, len(report['events']))
        for event in report['events']:
            self.assertEqual('passed', event['status'])
        self.assertEqual(1, len({event['request_tree_sha256'] for event in report['events']}))
        self.assertEqual(1, len({event['response_tree_sha256'] for event in report['events']}))

    def test_construct_and_decode_are_distinct_invocations(self):
        catalog, manifest = self.integer_fixture()
        report, gaps = self.runner.run_matrix(catalog, manifest, self.artifact)
        self.assertFalse(gaps)
        self.assertEqual(len(report['events']), len({event['invocation_id'] for event in report['events']}))
        for event in report['events']:
            field = 'response_tree_sha256' if event['action'] == 'decode' else 'request_tree_sha256'
            self.assertRegex(event[field], '^[0-9a-f]{64}$')


if __name__ == '__main__':
    unittest.main()
