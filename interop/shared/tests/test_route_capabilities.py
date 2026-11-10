"""KMIPKIT-0092-FR-001/002/009: source-backed construction routes."""
from __future__ import annotations

import copy
import importlib.util
import json
from pathlib import Path
import unittest
import sys

SHARED = Path(__file__).resolve().parents[1]
ROOT = SHARED.parents[1]


class RouteCapabilitiesTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        sys.path.insert(0, str(SHARED / 'tools'))
        spec = importlib.util.spec_from_file_location('route_capabilities', SHARED / 'tools/route_capabilities.py')
        cls.module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(cls.module)

    def setUp(self):
        self.data = json.loads((SHARED / 'coverage/route-capabilities.json').read_text())

    def test_normative_inventory_requires_local_construct_and_decode_in_every_language(self):
        for language in ('rust', 'c', 'java', 'python'):
            routes = [r for r in self.data['routes'] if r['language'] == language
                      and 'normative_inventory' in r['case_categories'] and r['api_level'] == 'generic-ttlv']
            self.assertEqual({'construct', 'decode'}, {a for r in routes for a in r['actions']})
            self.assertTrue(all(r['mode'] == 'local' for r in routes))
            self.assertTrue(all(r['combinations'] == [{'encoding': 'TTLV', 'transport': None}]
                                for r in routes))

    def test_local_typed_model_routes_cover_objects_attributes_and_enum_wrappers(self):
        symbols = {r['public_api_symbol']: r for r in self.data['routes']}
        required = {
            'rust': ['kmipkit_protocol::Certificate::try_new', 'kmipkit_protocol::ObjectType::from_raw'],
            'c': ['kmipkit_managed_object_from_fields', 'kmipkit_typed_attribute_from_value'],
            'java': ['org.kmipkit.ttlv.ManagedObject.fromFields', 'org.kmipkit.ttlv.TypedAttribute.create'],
            'python': ['kmipkit.managed_objects.ManagedObject.from_fields', 'kmipkit.typed_attributes.TypedAttribute.create']}
        for language, names in required.items():
            for name in names:
                self.assertTrue(name in symbols, name)
                route = symbols[name]
                self.assertEqual(language, route['language'])
                self.assertEqual('local', route['mode'])
                self.assertEqual('typed-protocol', route['api_level'])
                self.assertTrue(route['normative_target_ids'])

    def test_typed_attribute_specializations_have_exact_normative_targets(self):
        routes = [r for r in self.data['routes'] if r['language'] == 'rust'
                  and 'Attribute::<' in r['public_api_symbol']]
        self.assertTrue(routes)
        self.assertTrue(any('KMIPKIT-ELEM-ATTRIBUTE-ACTIVATION-DATE' in r['normative_target_ids'] for r in routes))

    def test_every_named_enum_and_mask_target_has_an_exact_public_value_symbol(self):
        catalog = json.loads((SHARED / 'catalog/kmipkit-1.0-test-catalog.json').read_text())
        target_ids = {t['target_id'] for t in catalog['normative_targets']
                      if t['kind'] in {'enumeration_value', 'bitmask_value'} and t.get('allocation') == 'assigned'}
        for language in ('rust', 'c', 'java', 'python'):
            values = {v['target_id'] for v in self.data.get('value_symbols', []) if v['language'] == language}
            self.assertEqual(target_ids, values)

    def test_installed_named_values_cannot_be_missing_or_have_the_wrong_wire_value(self):
        self.assertTrue(hasattr(self.module, 'validate_installed_values'), 'installed named-value validation is missing')
        with self.assertRaisesRegex(ValueError, 'named value'):
            self.module.validate_installed_values(self.data, 'python', {})
        values = {row['public_api_symbol']: row['wire_value'] for row in self.data['value_symbols']
                  if row['language'] == 'python'}
        first = next(iter(values))
        values[first] ^= 1
        with self.assertRaisesRegex(ValueError, 'named value'):
            self.module.validate_installed_values(self.data, 'python', values)

    def test_generic_value_constructors_have_distinct_data_type_routes(self):
        symbols = {r['public_api_symbol'] for r in self.data['routes']}
        for symbol in ('kmipkit_ttlv::Value::enumeration', 'kmipkit_ttlv_value_enumeration',
                       'org.kmipkit.ttlv.TtlvValue.Enumeration', 'kmipkit.ttlv.ttlv_value_enumeration'):
            self.assertTrue(symbol in symbols, symbol)

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
