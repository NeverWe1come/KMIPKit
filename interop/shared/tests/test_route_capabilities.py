"""KMIPKIT-0092-FR-001/002/009: source-backed construction routes."""
from __future__ import annotations

import copy
import importlib.util
import json
from pathlib import Path
import unittest

SHARED = Path(__file__).resolve().parents[1]
ROOT = SHARED.parents[1]


class RouteCapabilitiesTests(unittest.TestCase):
    def setUp(self):
        self.assertTrue((SHARED / 'coverage/route-capabilities.json').is_file(),
                        'the public construction route capability source is missing')
        spec = importlib.util.spec_from_file_location('route_capabilities', SHARED / 'tools/route_capabilities.py')
        self.assertIsNotNone(spec)
        self.module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(self.module)
        self.data = json.loads((SHARED / 'coverage/route-capabilities.json').read_text())

    def test_checked_in_manifest_validates_against_public_api(self):
        self.module.validate(self.data, ROOT)
        self.assertEqual({'rust', 'c', 'java', 'python'}, {r['language'] for r in self.data['routes']})

    def reject(self, data, message):
        with self.assertRaisesRegex(ValueError, message):
            self.module.validate(data, ROOT)

    def test_unknown_language_is_rejected(self):
        self.data['routes'][0]['language'] = 'ruby'
        self.reject(self.data, 'language')

    def test_unknown_route_family_is_rejected(self):
        self.data['routes'][0]['api_level'] = 'imaginary'
        self.reject(self.data, 'api_level')

    def test_duplicate_ids_are_rejected(self):
        self.data['routes'].append(copy.deepcopy(self.data['routes'][0]))
        self.reject(self.data, 'duplicate route_id')

    def test_missing_public_symbol_is_rejected(self):
        self.data['routes'][0]['public_api_symbol'] = 'not_a_public_symbol'
        self.reject(self.data, 'public API symbol')

    def test_impossible_encoding_transport_is_rejected(self):
        self.data['routes'][0]['combinations'].append({'encoding': 'DER', 'transport': 'udp'})
        self.reject(self.data, 'encoding|transport')

    def test_local_route_cannot_claim_network_transport(self):
        route = next(r for r in self.data['routes'] if r['mode'] == 'local')
        route['combinations'][0]['transport'] = 'raw_tls'
        self.reject(self.data, 'local')

    def test_public_constructor_cannot_disappear(self):
        route = next(r for r in self.data['routes'] if r['mode'] == 'network')
        self.data['routes'].remove(route)
        self.reject(self.data, 'missing public construction route')

    def test_false_unavailable_route_is_rejected(self):
        route = next(r for r in self.data['routes'] if r['mode'] == 'network')
        self.data['unavailable'].append({'language': route['language'], 'capability': 'constructor',
            'public_api_symbol': route['public_api_symbol'], 'reason': 'validator recipe pending',
            'source_paths': ['AGENTS.md']})
        self.reject(self.data, 'public constructor.*unavailable')

    def test_same_constructor_cannot_be_counted_twice(self):
        route = copy.deepcopy(self.data['routes'][0])
        route['route_id'] += '.pretend-independent'
        self.data['routes'].append(route)
        self.reject(self.data, 'same constructor')

    def test_unknown_route_id_is_rejected(self):
        self.data['routes'][0]['route_id'] += '.invented'
        self.reject(self.data, 'unknown route_id')

    def test_constructor_cannot_claim_another_operation(self):
        route = next(r for r in self.data['routes'] if r['mode'] == 'network')
        route['operation_ids'] = ['KMIPKIT-ELEM-OP-C2S-NONEXISTENT']
        self.reject(self.data, 'operation mapping')

    def test_missing_supported_pair_is_rejected(self):
        route = next(r for r in self.data['routes'] if r['mode'] == 'network')
        route['combinations'].pop()
        self.reject(self.data, 'supported encoding/transport')

    def test_missing_execution_route_is_rejected(self):
        self.data['execution_routes'].pop()
        self.reject(self.data, 'execution route')

    def test_regeneration_matches_checked_in_capabilities(self):
        self.assertEqual(self.data, self.module.build_manifest(ROOT, self.data['library_commit']))

    def test_artifact_probe_requires_the_public_sender(self):
        symbols = {r['public_api_symbol'] for r in self.data['routes'] if r['language'] == 'python'}
        with self.assertRaisesRegex(ValueError, 'installed artifact.*missing'):
            self.module.validate_installed_symbols(self.data, 'python', symbols)

    def test_artifact_probe_rejects_wrong_commit(self):
        with self.assertRaisesRegex(ValueError, 'artifact commit'):
            self.module.validate_artifact_identity(self.data, {'library_commit': 'a' * 40})

    def test_artifact_probe_requires_every_declared_symbol(self):
        with self.assertRaisesRegex(ValueError, 'installed artifact.*missing'):
            self.module.validate_installed_symbols(self.data, 'python', set())


if __name__ == '__main__':
    unittest.main()
