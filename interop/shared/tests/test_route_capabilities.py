"""KMIPKIT-0092-FR-001/002/009: source-backed construction routes."""
from __future__ import annotations

import copy
import importlib.util
import json
from pathlib import Path
import unittest
import sys
import subprocess

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
            values = {v['target_id'] for v in self.data.get('value_symbols', []) if v['language'] == language and v['availability'] == 'named-constant'}
            self.assertEqual(target_ids, values)

    def test_installed_named_values_cannot_be_missing_or_have_the_wrong_wire_value(self):
        self.assertTrue(hasattr(self.module, 'validate_installed_values'), 'installed named-value validation is missing')
        with self.assertRaisesRegex(ValueError, 'named value'):
            self.module.validate_installed_values(self.data, 'python', {})
        values = {row['public_api_symbol']: row['wire_value'] for row in self.data['value_symbols']
                  if row['language'] == 'python' and row['availability'] == 'named-constant'}
        first = next(iter(values))
        values[first] ^= 1
        with self.assertRaisesRegex(ValueError, 'named value'):
            self.module.validate_installed_values(self.data, 'python', values)

    def test_generic_value_constructors_have_distinct_data_type_routes(self):
        symbols = {r['public_api_symbol'] for r in self.data['routes']}
        for symbol in ('kmipkit_ttlv::Value::enumeration', 'kmipkit_ttlv_value_enumeration',
                       'org.kmipkit.ttlv.TtlvValue.Enumeration', 'kmipkit.ttlv.ttlv_value_enumeration'):
            self.assertTrue(symbol in symbols, symbol)

    def test_known_but_false_api_level_cannot_relabel_a_constructor(self):
        self.data['routes'][0]['api_level'] = 'high-level'
        self.reject(self.data, 'api_level')

    def test_case_category_and_protocol_coverage_cannot_be_silently_removed(self):
        self.data['routes'][0]['case_categories'] = []
        self.reject(self.data, 'case_categories')
        self.data = json.loads((SHARED / 'coverage/route-capabilities.json').read_text())
        self.data['routes'][0]['protocol_versions'] = ['2.1']
        self.reject(self.data, 'protocol_versions')

    def test_transparent_key_constructor_is_bound_to_its_owned_key_material_table(self):
        route = next(r for r in self.data['routes'] if r['public_api_symbol'] ==
                     'kmipkit_protocol::TransparentRsaPrivateKey::try_from_components')
        ids = route['normative_target_ids']
        self.assertTrue(any(target.startswith('KMIPKIT-ELEM-STRUCTURE-MEMBER-3-7-KEY-MATERIAL-') for target in ids))
        self.assertFalse(any(target.startswith('KMIPKIT-ELEM-STRUCTURE-MEMBER-3-8-KEY-MATERIAL-') for target in ids))

    def test_referenced_pkcs11_values_remain_covered_when_named_constants_are_absent(self):
        catalog = json.loads((SHARED / 'catalog/kmipkit-1.0-test-catalog.json').read_text())
        expected = {target['target_id'] for target in catalog['normative_targets']
                    if target['kind'] == 'referenced_enum_value'}
        for language in ('rust', 'c', 'java', 'python'):
            rows = [row for row in self.data['value_symbols'] if row['language'] == language
                    and row['target_id'] in expected]
            self.assertEqual(expected, {row['target_id'] for row in rows})
            constructors = {route['public_api_symbol'] for route in self.data['routes'] if route['language'] == language}
            for row in rows:
                if row['availability'] == 'numeric-only':
                    self.assertIsNone(row['public_api_symbol'])
                    self.assertIn(row['numeric_constructor'], constructors)

    def test_check_cli_is_read_only_and_accepts_current_generated_metadata(self):
        manifest = SHARED / 'coverage/route-capabilities.json'
        before = manifest.read_bytes()
        result = subprocess.run([sys.executable, str(SHARED / 'tools/route_capabilities.py'), '--check'],
                                capture_output=True, text=True, check=False)
        self.assertEqual(0, result.returncode, result.stderr)
        self.assertEqual(before, manifest.read_bytes())

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


class ModelApplicabilityTests(unittest.TestCase):
    def test_operation_scope_also_filters_descendant_payload_type_variants(self):
        sys.path.insert(0, str(SHARED / 'tools'))
        from route_models import target_ids
        targets = []
        for operation in ('activate', 'destroy'):
            field_id = operation + '-identifier'
            targets.append({'target_id': field_id, 'kind': 'operation_payload_field',
                'name': 'Unique Identifier', 'operation_id': operation, 'role': 'request'})
            targets.append({'target_id': field_id + '-text', 'kind': 'operation_payload_type_variant',
                'name': 'Unique Identifier as Text String', 'operation_id': operation,
                'role': 'request', 'parent_element_ids': [field_id]})
        self.assertEqual(['activate-identifier', 'activate-identifier-text'],
            target_ids(targets, ['Unique Identifier'], operation_id='activate'))
        self.assertEqual([], target_ids(targets, ['Unique Identifier']))


class ManifestSerializationTests(unittest.TestCase):
    def test_generated_records_are_complete_json_before_atomic_replacement(self):
        sys.path.insert(0, str(SHARED / 'tools'))
        import route_capabilities
        data = {'schema_version': '1.0', 'routes': [{'route_id': 'a'}], 'value_symbols': []}
        serialized = route_capabilities.serialize_manifest(data)
        self.assertTrue(serialized.endswith('}\n'), 'serialized manifest must close its root object')
        self.assertEqual(data, json.loads(serialized))


if __name__ == '__main__':
    unittest.main()
