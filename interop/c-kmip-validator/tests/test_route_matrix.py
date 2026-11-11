"""Matrix coverage must be backed by actual installed ABI calls (0092 FR-002/009)."""
from pathlib import Path
import importlib.util
import json
import sys
import unittest

TOOLS = Path(__file__).resolve().parents[1] / 'tools'
sys.path.insert(0, str(TOOLS))


class RouteMatrixTests(unittest.TestCase):
    def setUp(self):
        self.assertIsNotNone(importlib.util.find_spec("route_matrix"),
                             "C matrix executor is missing")
        import route_matrix
        self.runner = route_matrix

    def test_nonexistent_recipe_is_pending_and_never_library_unavailable(self):
        self.assertEqual(self.runner.pending_reason({'public_api_symbol': 'kmipkit_client_batch_item_sign'}),
                         'validator_recipe_pending')

    def test_known_enum_fixture_preserves_exact_named_target_value(self):
        target = {'kind': 'enumeration_value', 'wire_value': '00000003',
                  'parent_element_ids': ['KMIPKIT-ELEM-ENUMERATION-CRYPTOGRAPHIC-ALGORITHM']}
        fixture = self.runner.scalar_fixture(target, {'Cryptographic Algorithm': 0x420028})
        self.assertEqual(fixture, (0x420028, 5, (3).to_bytes(4, 'big')))

    def test_unmapped_structure_branch_is_pending_not_arbitrary_scalar(self):
        target = {'kind': 'operation_payload_field', 'name': 'Unique Identifier'}
        self.assertIsNone(self.runner.scalar_fixture(target, {}))

    def test_pending_cells_cannot_pass_completeness_gate(self):
        with self.assertRaisesRegex(ValueError, 'pending'):
            self.runner.require_complete([{'status': 'passed'}], [{'reason': 'validator_recipe_pending'}])

    def test_duplicate_events_rejected(self):
        event = {'case_id': 'case', 'route_id': 'route', 'target_id': 'target',
                 'protocol_version': '2.1', 'encoding': 'TTLV', 'transport': None,
                 'action': 'construct', 'execution_route_id': None, 'status': 'passed'}
        with self.assertRaisesRegex(ValueError, 'duplicate'):
            self.runner.require_complete([event, event], [])

    def test_ci_executes_matrix_and_shared_evidence_gate(self):
        root = Path(__file__).resolve().parents[3]
        workflow = (root / '.github/workflows/interop-c.yml').read_text()
        self.assertIn('route_matrix.py', workflow)
        self.assertIn('validate_route_evidence.py', workflow)


    def test_installed_scalar_roundtrip_checks_exact_value(self):
        from artifact_support import INSTALL
        abi = self.runner.InstalledAbi(INSTALL / 'lib/libkmipkit_ffi.so')
        fixture = (0x420028, 5, bytes.fromhex('00000003'))
        wire = abi.execute('kmipkit_ttlv_decode', fixture)
        self.assertEqual(wire[8:12], fixture[2])
        self.assertIn('kmipkit_ttlv_decode', abi.called)

    def test_named_route_is_not_credited_when_another_symbol_was_called(self):
        from artifact_support import INSTALL
        abi = self.runner.InstalledAbi(INSTALL / 'lib/libkmipkit_ffi.so')
        with self.assertRaisesRegex(ValueError, 'named_route_not_invoked'):
            abi.execute('kmipkit_ttlv_value_boolean', (0x420028, 5, bytes.fromhex('00000003')))

    def test_codec_roundtrips_controlled_response_in_all_encodings(self):
        from route_peer import Codec, parse, response
        from artifact_support import ROOT
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        codec = Codec(catalog)
        wire = response((1, 10, 20), 1)
        for encoding in ('TTLV', 'JSON', 'XML'):
            self.assertEqual(codec.decode(codec.encode(wire, encoding), encoding), parse(wire))

    def test_positive_case_cannot_pass_by_operation_name_alone(self):
        from route_peer import assert_recipe, parse, tlv
        # This contains the correct Query operation but omits Query Function.
        integer = lambda tag, value: tlv(tag, 2, value.to_bytes(4, 'big'))
        request = tlv(0x420078, 1, integer(0x42006A, 2) + integer(0x42006B, 1)
                      + integer(0x42000D, 1) + tlv(0x42005C, 5, (24).to_bytes(4, 'big')))
        with self.assertRaisesRegex(ValueError, 'fixture_query_function_mismatch'):
            assert_recipe(parse(request), 'test-op-query', '2.1')


    def test_lifecycle_peer_requires_cleanup_and_checks_illegal_destroy(self):
        import route_peer
        self.assertTrue(hasattr(route_peer, 'verify_lifecycle'), 'stateful lifecycle verification is missing')
        from route_peer import parse, response
        effects = route_peer.verify_lifecycle((1, 18, 20, 19, 20), parse(response((1, 18, 20, 19, 20), 1)))
        self.assertIn('illegal_destroy_rejected', effects)
        self.assertEqual(effects[-1], 'destroyed')
        with self.assertRaisesRegex(ValueError, 'fixture_cleanup_missing'):
            route_peer.verify_lifecycle((1,), parse(response((1,), 1)))

    def test_reserved_tag_decoder_is_checked_as_rejection(self):
        from artifact_support import INSTALL
        abi = self.runner.InstalledAbi(INSTALL / 'lib/libkmipkit_ffi.so')
        self.assertTrue(hasattr(abi, 'execute_reserved_tag'), 'reserved tag negative recipe is missing')
        wire = abi.execute_reserved_tag('kmipkit_ttlv_decode', 0x420009)
        self.assertEqual(wire[:3], bytes.fromhex('420009'))
        self.assertIn('kmipkit_ttlv_decode', abi.called)


    def test_compiled_consumer_provenance_cannot_be_replaced_by_coordinator(self):
        import route_peer
        self.assertTrue(hasattr(route_peer, 'verify_consumer_provenance'), 'consumer provenance check is missing')
        from artifact_support import PIN
        good = {'execution': {'library_commit': PIN, 'artifact_sha256': 'a' * 64},
                'catalog_sha256': 'b' * 64}
        route_peer.verify_consumer_provenance(good, 'a' * 64, 'b' * 64)
        with self.assertRaisesRegex(ValueError, 'consumer_artifact_provenance_mismatch'):
            route_peer.verify_consumer_provenance(good, 'c' * 64, 'b' * 64)


    def tree_abi(self):
        self.assertIsNotNone(importlib.util.find_spec('route_ttlv'), 'recursive public C TTLV executor is missing')
        from route_ttlv import TreeAbi
        from artifact_support import INSTALL
        return TreeAbi(INSTALL / 'lib/libkmipkit_ffi.so')

    def test_recursive_ttlv_preserves_nested_order_and_values(self):
        abi = self.tree_abi()
        wire = bytes.fromhex('42000101000000204200010200000004000000010000000042000205000000040000000200000000')
        result = abi.roundtrip(wire)
        self.assertEqual(result['decoded_wire'], wire)
        self.assertEqual(result['constructed_wire'], wire)
        self.assertIn('kmipkit_ttlv_value_structure', result['invoked_symbols'])

    def test_every_normative_ttlv_boundary_vector_is_executed(self):
        from artifact_support import ROOT
        vectors = json.loads((ROOT / 'interop/shared/vectors/ttlv-boundaries.json').read_text())
        for vector in vectors['vectors']:
            with self.subTest(data_type=vector['data_type']):
                result = self.tree_abi().boundary_case(vector)
                self.assertEqual(result['sample_count'], len(vector['samples']))
                self.assertEqual(result['failed_samples'], 0)
                self.assertIn('kmipkit_ttlv_decode', result['invoked_symbols'])

    def test_boolean_public_constructor_rejects_non_boolean_value(self):
        abi = self.tree_abi()
        self.assertTrue(abi.reject_invalid_boolean())


    def value_executor(self):
        self.assertIsNotNone(importlib.util.find_spec('route_values'), 'named-value C case executor is missing')
        from route_values import ValueCases
        from artifact_support import INSTALL, ROOT
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        return ValueCases(INSTALL / 'lib/libkmipkit_ffi.so', catalog)

    def test_enum_case_uses_its_named_type_and_preserves_unknown_value(self):
        result = self.value_executor().execute({'requires': {'enum_type': 'Cryptographic Algorithm'}}, '2.1')
        self.assertEqual(result['tag'], 0x420028)
        self.assertGreater(result['known_values_checked'], 20)
        self.assertTrue(result['unknown_value_preserved'])
        self.assertIn('kmipkit_ttlv_value_enumeration', result['invoked_symbols'])

    def test_bitmask_case_executes_every_flag_combination(self):
        result = self.value_executor().execute({'requires': {'bitmask_type': 'Cryptographic Usage Mask',
            'exhaustive_bitmask_combinations': {'flag_wire_values': ['00000001', '00000002', '00000004'],
                                              'combination_count': 8}}}, '2.1')
        self.assertEqual(result['combination_count'], 8)
        self.assertTrue(result['unknown_value_preserved'])
        self.assertEqual(result['tag'], 0x42002C)


    def test_attribute_request_routes_are_independently_executed(self):
        from artifact_support import ROOT
        from route_peer import execute_network_cells
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        case_ids = {'test-op-get-attributes', 'test-op-get-attribute-list'}
        selected = {**catalog, 'test_cases': [case for case in catalog['test_cases'] if case['case_id'] in case_ids]}
        events, _, checks = execute_network_cells(ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest)
        self.assertTrue(checks and all(check['status'] == 'passed' for check in checks))
        for symbol in ('kmipkit_client_batch_item_get_attributes_from_request',
                       'kmipkit_client_batch_item_get_attribute_list_from_request'):
            self.assertTrue(any(event['public_api_symbol'] == symbol and event['action'] == 'send' for event in events), symbol)


    def test_hash_and_rng_cases_call_their_public_routes_and_check_responses(self):
        from artifact_support import ROOT
        from route_peer import RECIPES, execute_network_cells
        case_ids = {'test-op-hash', 'test-op-rng-retrieve', 'test-op-rng-seed'}
        self.assertTrue(case_ids <= RECIPES.keys(), 'Hash/RNG case-specific peer recipes are missing')
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        manifest = json.loads((ROOT / 'interop/shared/coverage/route-capabilities.json').read_text())
        selected = {**catalog, 'test_cases': [case for case in catalog['test_cases'] if case['case_id'] in case_ids]}
        events, _, checks = execute_network_cells(ROOT / 'interop/c-kmip-validator/build/kmip_validator_c', selected, manifest)
        self.assertEqual(len(checks), 36)
        self.assertTrue(all(check['status'] == 'passed' for check in checks))
        for name in ('hash', 'rng_retrieve', 'rng_seed'):
            self.assertTrue(any(event['public_api_symbol'] == 'kmipkit_client_batch_item_' + name and event['action'] == 'decode' for event in events))


    def attribute_executor(self):
        self.assertIsNotNone(importlib.util.find_spec('route_attributes'), 'typed attribute route executor is missing')
        from route_attributes import AttributeCases
        from artifact_support import INSTALL, ROOT
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        return AttributeCases(INSTALL / 'lib/libkmipkit_ffi.so', catalog)

    def test_all_typed_attribute_paths_preserve_value_and_reject_wrong_type(self):
        executor = self.attribute_executor()
        target = next(target for target in executor.catalog['normative_targets']
                      if target['target_id'] == 'KMIPKIT-ELEM-ATTRIBUTE-CRYPTOGRAPHIC-ALGORITHM')
        result = executor.execute(target, '2.1')
        expected = {'kmipkit_typed_attribute_from_item', 'kmipkit_typed_attribute_from_value',
                    'kmipkit_attribute_set_create', 'kmipkit_attribute_set_try_push',
                    'kmipkit_attribute_set_try_push_typed'}
        self.assertTrue(expected <= result['invoked_symbols'])
        self.assertEqual(result['negative_checks'], 5)
        self.assertGreater(result['sample_count'], 1)

    def test_structured_name_attribute_has_both_required_fields(self):
        executor = self.attribute_executor()
        target = next(target for target in executor.catalog['normative_targets']
                      if target['target_id'] == 'KMIPKIT-ELEM-ATTRIBUTE-NAME')
        trees = executor.fixtures(target, '2.1')
        self.assertEqual([child[0] for child in trees[0][2]], [0x420055, 0x420054])
        result = executor.execute(target, '2.1')
        self.assertTrue(result['invoked_symbols'])

    def test_vendor_attribute_uses_attribute_container_and_required_members(self):
        executor = self.attribute_executor()
        target = next(target for target in executor.catalog['normative_targets']
                      if target['target_id'] == 'KMIPKIT-ELEM-ATTRIBUTE-VENDOR-ATTRIBUTE')
        trees = executor.fixtures(target, '2.1')
        self.assertIsNotNone(trees)
        self.assertEqual(len(trees), 1)
        self.assertEqual(trees[0][0], 0x420008)
        self.assertEqual([child[0] for child in trees[0][2]], [0x42009D, 0x42000A, 0x42000B])
        self.assertEqual([child[2] for child in trees[0][2]], [b'fixture_vendor', b'fixture_attribute', b'fixture-value'])
        result = executor.execute(target, '2.1')
        self.assertTrue({'kmipkit_typed_attribute_from_item', 'kmipkit_attribute_set_create',
                         'kmipkit_ttlv_decode'} <= result['invoked_symbols'])
        self.assertEqual(result['negative_checks'], 5)


if __name__ == '__main__':
    unittest.main()
