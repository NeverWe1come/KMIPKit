"""Matrix coverage must be backed by actual installed ABI calls (0092 FR-002/009)."""
from pathlib import Path
import importlib.util
import hashlib
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

    def test_operation_and_object_type_fixtures_use_exact_normative_values(self):
        from artifact_support import ROOT
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        targets = {target['target_id']: target for target in catalog['normative_targets']}
        tags = {target['name']: int(target['wire_value'], 16) for target in catalog['normative_targets']
                if target['kind'] == 'tag'}
        operation = targets['KMIPKIT-ELEM-OP-C2S-ACTIVATE']
        object_type = targets['KMIPKIT-ELEM-OBJECT-TYPE-SYMMETRIC-KEY']
        self.assertEqual(self.runner.scalar_fixture(operation, tags, catalog, '2.1'),
                         (tags['Operation'], 5, bytes.fromhex('00000012')))
        self.assertEqual(self.runner.scalar_fixture(object_type, tags, catalog, '2.0'),
                         (tags['Object Type'], 5, bytes.fromhex(object_type['wire_value'])))

    def test_version_boundary_operation_item_uses_its_supported_catalog_value(self):
        from artifact_support import ROOT
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        targets = {target['target_id']: target for target in catalog['normative_targets']}
        tags = {target['name']: int(target['wire_value'], 16) for target in catalog['normative_targets']
                if target['kind'] == 'tag'}
        operation = targets['KMIPKIT-ELEM-OP-C2S-GET-CONSTRAINTS']
        value = next(target for target in catalog['normative_targets']
                     if target['kind'] == 'enumeration_value' and target['name'] == 'Get Constraints'
                     and target['parent_element_ids'] == ['KMIPKIT-ELEM-ENUMERATION-OPERATION'])
        self.assertEqual(self.runner.scalar_fixture(operation, tags, catalog, '2.0'),
                         (tags['Operation'], 5, bytes.fromhex(value['wire_value'])))

    def test_generic_ttlv_fixture_uses_structure_member_tag_and_declared_type(self):
        from artifact_support import ROOT, INSTALL
        from route_ttlv import TreeAbi, encode
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        target = next(item for item in catalog['normative_targets']
                      if item['target_id'] == 'KMIPKIT-ELEM-STRUCTURE-MEMBER-2-1-CERTIFICATE-CERTIFICATE-TYPE')
        tags = {item['name']: int(item['wire_value'], 16) for item in catalog['normative_targets']
                if item['kind'] == 'tag'}
        trees = self.runner.generic_tree_fixtures(target, catalog, tags, '2.1')
        self.assertEqual(trees, [(tags['Certificate Type'], 5, bytes.fromhex('00000001'))])
        result = TreeAbi(INSTALL / 'lib/libkmipkit_ffi.so').roundtrip(encode(trees[0]))
        self.assertEqual(result['constructed_wire'], result['decoded_wire'])
        self.assertIn('kmipkit_ttlv_item_create', result['invoked_symbols'])

    def test_reserved_tag_is_not_constructed_as_an_assigned_ttlv_item(self):
        from artifact_support import ROOT
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        target = next(item for item in catalog['normative_targets']
                      if item['target_id'] == 'KMIPKIT-ELEM-TAG-420009')
        tags = {item['name']: int(item['wire_value'], 16) for item in catalog['normative_targets']
                if item['kind'] == 'tag'}
        self.assertIsNone(self.runner.generic_tree_fixtures(target, catalog, tags, '2.1'))

    def test_enum_extension_and_reserved_ranges_roundtrip_both_boundaries(self):
        from artifact_support import ROOT
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        targets = {target['target_id']: target for target in catalog['normative_targets']}
        tags = {target['name']: int(target['wire_value'], 16) for target in catalog['normative_targets']
                if target['kind'] == 'tag'}
        enum_extension = targets['KMIPKIT-ELEM-ENUM-VALUE-ADJUSTMENT-TYPE-EXTENSIONS-8XXXXXXX']
        mask_extension = targets['KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-EXTENSIONS-XXX00000']
        reserved_range = targets['KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-RESERVED-00000001-00000103']
        self.assertEqual([tree[2] for tree in self.runner.generic_tree_fixtures(
            enum_extension, catalog, tags, '2.1')], [bytes.fromhex('80000000'), bytes.fromhex('8fffffff')])
        self.assertEqual([tree[2] for tree in self.runner.generic_tree_fixtures(
            mask_extension, catalog, tags, '2.1')], [bytes.fromhex('00000000'), bytes.fromhex('fff00000')])
        self.assertEqual([tree[2] for tree in self.runner.generic_tree_fixtures(
            reserved_range, catalog, tags, '2.1')], [bytes.fromhex('00000001'), bytes.fromhex('00000103')])

    def test_tag_allocation_ranges_check_each_boundary_through_the_decoder(self):
        from artifact_support import ROOT, INSTALL
        from route_ttlv import TreeAbi, encode
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        abi = TreeAbi(INSTALL / 'lib/libkmipkit_ffi.so')
        targets = [target for target in catalog['normative_targets'] if target['kind'] == 'tag_range']
        self.assertEqual(len(targets), 5)
        expected_rejections = {
            'KMIPKIT-RANGE-001': [12, 12],
            'KMIPKIT-RANGE-002': [12, 11],
            'KMIPKIT-RANGE-003': [12, 12],
            'KMIPKIT-RANGE-005': [12, 12],
        }
        for target in targets:
            with self.subTest(target=target['target_id']):
                result = self.runner.execute_tag_range_case(abi, target, catalog)
                self.assertIn('kmipkit_ttlv_decode', result['invoked_symbols'])
                if target['allocation'] == 'extension':
                    self.assertIn('kmipkit_ttlv_item_create', result['invoked_symbols'])
                else:
                    wires = [encode((tag, 5, (7).to_bytes(4, 'big')))
                             for tag in self.runner.range_boundary_tags(target)]
                    self.assertEqual(result['rejection_codes'], expected_rejections[target['target_id']])
                    self.assertEqual(result['decode_input_sha256'], hashlib.sha256(b''.join(wires)).hexdigest())
                self.assertEqual(result['sample_count'], 2)

    def test_ttlv_encoding_parser_keeps_compound_types_distinct(self):
        self.assertEqual(self.runner._type_codes('Long Integer'), [3])
        self.assertEqual(self.runner._type_codes('Big Integer'), [4])
        self.assertEqual(self.runner._type_codes('Date Time Extended'), [11])
        self.assertEqual(self.runner._type_codes('Text String, Enumeration or Integer'), [7, 5, 2])

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

    def test_reserved_tag_decode_event_reports_rejection_without_tree_hash(self):
        cell = {'case_id': 'test-norm-tag-tag-420009', 'route_id': 'c.generic-ttlv.kmipkit_ttlv_decode',
                'target_id': 'KMIPKIT-ELEM-TAG-420009', 'public_api_symbol': 'kmipkit_ttlv_decode',
                'execution_route_id': None, 'execution_api_symbol': None, 'action': 'decode',
                'protocol_version': '2.0', 'encoding': 'TTLV', 'transport': None,
                'workflow_step_index': None, 'workflow_operation': None, 'workflow_item_index': None}
        wire = bytes.fromhex('42000905000000040000000700000000')
        event = self.runner.rejected_tag_event(cell, wire, 4)
        self.assertEqual(event['decode_outcome'], 'rejected')
        self.assertEqual(event['decode_rejection_code'], 'tag_not_allocated')
        self.assertEqual(event['decode_input_sha256'], hashlib.sha256(wire).hexdigest())
        self.assertNotIn('response_tree_sha256', event)

    def test_certificate_model_recipe_invokes_typed_and_generic_field_routes(self):
        from artifact_support import INSTALL, ROOT
        from c_route_models import ModelCases
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        target = next(item for item in catalog['normative_targets']
                      if item['target_id'] == 'KMIPKIT-ELEM-OBJECT-STRUCTURE-CERTIFICATE')
        result = ModelCases(INSTALL / 'lib/libkmipkit_ffi.so', catalog).execute(target, '2.1')
        self.assertEqual(result['sample_count'], 1)
        self.assertEqual(result['normalized_tree'], [0x420013, 1, [
            [0x42001d, 5, '00000001'], [0x42001e, 8, '010203']]])
        self.assertTrue({
            'kmipkit_certificate_from_fields', 'kmipkit_managed_object_from_fields',
            'kmipkit_managed_object_from_item',
        }.issubset(result['invoked_symbols']))

    def test_symmetric_key_model_recipe_invokes_nested_key_block_routes(self):
        from artifact_support import INSTALL, ROOT
        from c_route_models import ModelCases
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        target = next(item for item in catalog['normative_targets']
                      if item['target_id'] == 'KMIPKIT-ELEM-OBJECT-STRUCTURE-SYMMETRIC-KEY')
        result = ModelCases(INSTALL / 'lib/libkmipkit_ffi.so', catalog).execute(target, '2.1')
        self.assertEqual(result['normalized_tree'], [0x42008f, 1, [
            [0x420040, 1, [[0x420042, 5, '00000001']]]]])
        self.assertTrue({
            'kmipkit_key_block_from_fields', 'kmipkit_key_block_from_item',
            'kmipkit_symmetric_key_from_fields', 'kmipkit_managed_object_from_fields',
            'kmipkit_managed_object_from_item',
        }.issubset(result['invoked_symbols']))

    def test_named_managed_object_recipes_cover_catalogued_object_structures(self):
        from artifact_support import INSTALL, ROOT
        from c_route_models import ModelCases
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        cases = {
            'KMIPKIT-ELEM-OBJECT-STRUCTURE-CERTIFICATE-REQUEST': (
                'kmipkit_certificate_request_from_fields',
                [0x420018, 1, [[0x420019, 5, '00000002'], [0x420140, 8, '72657175657374']]],
            ),
            'KMIPKIT-ELEM-OBJECT-STRUCTURE-PGP-KEY': (
                'kmipkit_pgp_key_from_fields',
                [0x4200bd, 1, [[0x4200be, 2, '00000007'],
                               [0x420040, 1, [[0x420042, 5, '00000001']]]]],
            ),
            'KMIPKIT-ELEM-OBJECT-STRUCTURE-PRIVATE-KEY': (
                'kmipkit_private_key_from_fields',
                [0x420064, 1, [[0x420040, 1, [[0x420042, 5, '00000001']]]]],
            ),
            'KMIPKIT-ELEM-OBJECT-STRUCTURE-PUBLIC-KEY': (
                'kmipkit_public_key_from_fields',
                [0x42006d, 1, [[0x420040, 1, [[0x420042, 5, '00000001']]]]],
            ),
            'KMIPKIT-ELEM-OBJECT-STRUCTURE-OPAQUE-OBJECT': (
                'kmipkit_opaque_object_from_fields',
                [0x42005b, 1, [[0x420059, 5, '80000000'], [0x42005a, 8, '6f7061717565']]],
            ),
        }
        recipes = ModelCases(INSTALL / 'lib/libkmipkit_ffi.so', catalog)
        for target_id, (constructor, expected_tree) in cases.items():
            with self.subTest(target=target_id):
                target = next(item for item in catalog['normative_targets'] if item['target_id'] == target_id)
                result = recipes.execute(target, '2.1')
                self.assertEqual(result['normalized_tree'], expected_tree)
                self.assertIn(constructor, result['invoked_symbols'])

    def test_key_block_model_recipe_invokes_field_and_item_converters(self):
        from artifact_support import INSTALL, ROOT
        from c_route_models import ModelCases
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        target = next(item for item in catalog['normative_targets']
                      if item['target_id'] == 'KMIPKIT-ELEM-OBJECT-STRUCTURE-KEY-BLOCK')
        result = ModelCases(INSTALL / 'lib/libkmipkit_ffi.so', catalog).execute(target, '2.1')
        self.assertEqual(result['normalized_tree'], [0x420040, 1, [[0x420042, 5, '00000001']]])
        self.assertTrue({'kmipkit_key_block_from_fields', 'kmipkit_key_block_from_item'}
                        .issubset(result['invoked_symbols']))

    def test_key_value_model_recipe_invokes_format_scoped_converters(self):
        from artifact_support import INSTALL, ROOT
        from c_route_models import ModelCases
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        target = next(item for item in catalog['normative_targets']
                      if item['target_id'] == 'KMIPKIT-ELEM-OBJECT-STRUCTURE-KEY-VALUE')
        result = ModelCases(INSTALL / 'lib/libkmipkit_ffi.so', catalog).execute(target, '2.1')
        self.assertEqual(result['normalized_tree'], [0x420045, 1, [[0x420043, 8, 'a1']]])
        self.assertTrue({'kmipkit_key_value_from_fields', 'kmipkit_key_value_from_item'}
                        .issubset(result['invoked_symbols']))

    def test_secret_data_model_recipe_covers_required_key_material(self):
        from artifact_support import INSTALL, ROOT
        from c_route_models import ModelCases
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        target = next(item for item in catalog['normative_targets']
                      if item['target_id'] == 'KMIPKIT-ELEM-OBJECT-STRUCTURE-SECRET-DATA')
        result = ModelCases(INSTALL / 'lib/libkmipkit_ffi.so', catalog).execute(target, '2.1')
        self.assertEqual(result['normalized_tree'], [0x420085, 1, [
            [0x420086, 5, '00000001'],
            [0x420040, 1, [[0x420042, 5, '00000001'],
                           [0x420045, 1, [[0x420043, 8, 'a1']]]]],
        ]])
        self.assertTrue({'kmipkit_secret_data_from_fields', 'kmipkit_key_block_from_fields',
                         'kmipkit_key_block_from_item'}.issubset(result['invoked_symbols']))

    def test_split_key_model_recipe_preserves_source_order_and_method(self):
        from artifact_support import INSTALL, ROOT
        from c_route_models import ModelCases
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        target = next(item for item in catalog['normative_targets']
                      if item['target_id'] == 'KMIPKIT-ELEM-OBJECT-STRUCTURE-SPLIT-KEY')
        result = ModelCases(INSTALL / 'lib/libkmipkit_ffi.so', catalog).execute(target, '2.1')
        self.assertEqual(result['normalized_tree'], [0x420089, 1, [
            [0x42008b, 2, '00000002'], [0x420044, 2, '00000002'],
            [0x42008c, 2, '00000001'], [0x42008a, 5, '00000001'],
            [0x420040, 1, [[0x420042, 5, '00000001']]],
        ]])
        self.assertTrue({'kmipkit_split_key_from_fields', 'kmipkit_key_block_from_fields',
                         'kmipkit_key_block_from_item'}.issubset(result['invoked_symbols']))

    def test_cryptographic_parameters_recipe_preserves_the_exact_catalog_enum(self):
        from artifact_support import INSTALL, ROOT
        from c_route_models import ModelCases
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        target = next(item for item in catalog['normative_targets']
                      if item['target_id'] == 'KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-GCM-00000009')
        result = ModelCases(INSTALL / 'lib/libkmipkit_ffi.so', catalog).execute(target, '2.1')
        self.assertEqual(result['sample_count'], 1)
        self.assertEqual(result['normalized_tree'], [
            [0x420011, 5, '00000009'], [0x4200cd, 2, '0000000c'],
            [0x4200ce, 2, '00000010'],
        ])
        self.assertTrue({
            'kmipkit_cryptographic_parameters_from_fields_v1',
            'kmipkit_cryptographic_parameters_read_fields_v1',
            'kmipkit_cryptographic_parameters_into_ttlv',
            'kmipkit_ttlv_structure_view',
        }.issubset(result['invoked_symbols']))

    def test_cryptographic_parameters_recipe_checks_boolean_structure_members(self):
        from artifact_support import INSTALL, ROOT
        from c_route_models import ModelCases
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        target = next(item for item in catalog['normative_targets']
                      if item['target_id'] == 'KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-RANDOM-IV')
        result = ModelCases(INSTALL / 'lib/libkmipkit_ffi.so', catalog).execute(target, '2.1')
        self.assertEqual(result['normalized_tree'], [[0x4200c5, 6, '0000000000000000']])
        self.assertIn('kmipkit_cryptographic_parameters_from_fields_v1', result['invoked_symbols'])

    def test_cryptographic_parameters_recipe_roundtrips_enum_extension_boundaries(self):
        from artifact_support import INSTALL, ROOT
        from c_route_models import ModelCases
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        target = next(item for item in catalog['normative_targets']
                      if item['target_id'] == 'KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-EXTENSIONS-8XXXXXXX')
        result = ModelCases(INSTALL / 'lib/libkmipkit_ffi.so', catalog).execute(target, '2.1')
        self.assertEqual(result['sample_count'], 2)
        self.assertEqual(result['normalized_tree'], [
            [[0x420011, 5, '80000000']], [[0x420011, 5, '8fffffff']],
        ])
        self.assertIn('kmipkit_cryptographic_parameters_from_fields_v1', result['invoked_symbols'])

    def test_query_peer_driver_checks_typed_response_ttlv_accessor(self):
        root = Path(__file__).resolve().parents[3]
        source = (root / 'interop/c-kmip-validator/src/live_driver.c').read_text()
        self.assertTrue('kmipkit_client_batch_item_response_query_ttlv' in source,
                        'Query driver does not inspect the typed TTLV response accessor')
        self.assertTrue('kmipkit_ttlv_structure_view_item_count' in source,
                        'Query driver does not compare the typed response field count')

    def test_key_block_route_preserves_exact_cryptographic_algorithm(self):
        from artifact_support import INSTALL, ROOT
        from c_route_models import ModelCases
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        target = next(item for item in catalog['normative_targets']
                      if item['target_id'] == 'KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-AES-00000003')
        result = ModelCases(INSTALL / 'lib/libkmipkit_ffi.so', catalog).execute(
            target, '2.1', 'kmipkit_key_block_from_fields')
        self.assertEqual(result['normalized_tree'], [0x420040, 1, [
            [0x420042, 5, '00000001'], [0x420028, 5, '00000003'],
            [0x42002a, 2, '00000080'],
        ]])
        self.assertIn('kmipkit_key_block_from_fields', result['invoked_symbols'])

    def test_key_block_route_roundtrips_transparent_rsa_public_key_material(self):
        from artifact_support import INSTALL, ROOT
        from c_route_models import ModelCases
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        target = next(item for item in catalog['normative_targets']
                      if item['target_id'] == 'KMIPKIT-ELEM-OBJECT-STRUCTURE-3-8-KEY-MATERIAL')
        result = ModelCases(INSTALL / 'lib/libkmipkit_ffi.so', catalog).execute(
            target, '2.1', 'kmipkit_key_block_from_fields')
        tags = {item['name']: int(item['wire_value'], 16) for item in catalog['normative_targets']
                if item['kind'] == 'tag'}
        self.assertIsNotNone(result)
        self.assertEqual(result['normalized_tree'], [tags['Key Block'], 1, [
            [tags['Key Format Type'], 5, '0000000b'],
            [tags['Key Value'], 1, [[tags['Key Material'], 1, [
                [tags['Modulus'], 4, '01'], [tags['Public Exponent'], 4, '010001'],
            ]]]],
        ]])
        self.assertTrue({'kmipkit_key_block_from_fields', 'kmipkit_key_block_from_item'}
                        .issubset(result['invoked_symbols']))

    def test_key_block_route_roundtrips_each_transparent_key_material_format(self):
        from artifact_support import INSTALL, ROOT
        from c_route_models import ModelCases
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        targets = {item['target_id']: item for item in catalog['normative_targets']}
        tags = {item['name']: int(item['wire_value'], 16) for item in catalog['normative_targets']
                if item['kind'] == 'tag'}
        cases = {
            4: (7, [('Key', 8)]),
            5: (8, [('P', 4), ('Q', 4), ('G', 4), ('X', 4)]),
            6: (9, [('P', 4), ('Q', 4), ('G', 4), ('Y', 4)]),
            7: (10, [('Modulus', 4), ('Public Exponent', 4), ('Private Exponent', 4),
                     ('P', 4), ('Q', 4), ('Prime Exponent P', 4),
                     ('Prime Exponent Q', 4), ('CRT Coefficient', 4)]),
            8: (11, [('Modulus', 4), ('Public Exponent', 4)]),
            9: (12, [('P', 4), ('Q', 4), ('G', 4), ('J', 4), ('X', 4)]),
            10: (13, [('P', 4), ('Q', 4), ('G', 4), ('J', 4), ('Y', 4)]),
            11: (20, [('Recommended Curve', 5), ('D', 4)]),
            12: (21, [('Recommended Curve', 5), ('Q String', 8)]),
        }
        models = ModelCases(INSTALL / 'lib/libkmipkit_ffi.so', catalog)
        for section, (format_value, members) in cases.items():
            target = targets[f'KMIPKIT-ELEM-OBJECT-STRUCTURE-3-{section}-KEY-MATERIAL']
            for version in ('2.0', '2.1'):
                with self.subTest(section=section, version=version):
                    result = models.execute(target, version, 'kmipkit_key_block_from_fields')
                    self.assertIsNotNone(result)
                    key_block_fields = result['normalized_tree'][2]
                    self.assertEqual(key_block_fields[0], [tags['Key Format Type'], 5,
                                                           f'{format_value:08x}'])
                    material_fields = key_block_fields[1][2][0][2]
                    self.assertEqual([(field[0], field[1]) for field in material_fields],
                                     [(tags[name], item_type) for name, item_type in members])
                    self.assertTrue({'kmipkit_key_block_from_fields', 'kmipkit_key_block_from_item'}
                                    .issubset(result['invoked_symbols']))

    def test_key_block_route_roundtrips_optional_key_value_attributes(self):
        from artifact_support import INSTALL, ROOT
        from c_route_models import ModelCases
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        targets = {item['target_id']: item for item in catalog['normative_targets']}
        tags = {item['name']: int(item['wire_value'], 16) for item in catalog['normative_targets']
                if item['kind'] == 'tag'}
        target = targets['KMIPKIT-ELEM-STRUCTURE-MEMBER-3-2-KEY-VALUE-ATTRIBUTES']
        result = ModelCases(INSTALL / 'lib/libkmipkit_ffi.so', catalog).execute(
            target, '2.1', 'kmipkit_key_block_from_fields')
        self.assertIsNotNone(result)
        self.assertEqual(result['normalized_tree'], [tags['Key Block'], 1, [
            [tags['Key Format Type'], 5, '00000001'],
            [tags['Key Value'], 1, [[tags['Key Material'], 8, 'a1'],
                                    [tags['Attributes'], 1, []]]],
        ]])
        self.assertIn('kmipkit_key_block_from_fields', result['invoked_symbols'])

    def test_secret_ttlv_value_routes_preserve_only_digest_evidence(self):
        from artifact_support import INSTALL, ROOT
        from c_route_models import ModelCases
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        targets = {item['target_id']: item for item in catalog['normative_targets']}
        cases = [
            ('KMIPKIT-ELEM-DATA-TYPE-BYTE-STRING', 'kmipkit_secret_bytes_into_ttlv_value', 6),
            ('KMIPKIT-ELEM-DATA-TYPE-TEXT-STRING', 'kmipkit_secret_text_into_ttlv_value', 6),
        ]
        models = ModelCases(INSTALL / 'lib/libkmipkit_ffi.so', catalog)
        for target_id, symbol, samples in cases:
            with self.subTest(symbol=symbol):
                result = models.execute(targets[target_id], '2.1', symbol)
                self.assertIsNotNone(result)
                self.assertEqual(result['sample_count'], samples)
                self.assertRegex(result['tree_sha256'], r'^[0-9a-f]{64}$')
                self.assertIn(symbol, result['invoked_symbols'])
                self.assertNotIn('normalized_tree', result)
                if 'text' in symbol:
                    self.assertEqual(result.get('negative_checks'), 1)
                else:
                    self.assertNotIn('negative_checks', result)

    def test_key_block_route_constructs_wrapping_information_and_key_value(self):
        from artifact_support import INSTALL, ROOT
        from c_route_models import ModelCases
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        target = next(item for item in catalog['normative_targets']
                      if item['target_id'] == 'KMIPKIT-ELEM-OBJECT-STRUCTURE-KEY-WRAPPING-DATA')
        result = ModelCases(INSTALL / 'lib/libkmipkit_ffi.so', catalog).execute(
            target, '2.1', 'kmipkit_key_block_from_fields')
        self.assertTrue(any(child[0] == 0x420046 for child in result['normalized_tree'][2]))
        self.assertIn('kmipkit_key_block_from_fields', result['invoked_symbols'])

    def test_key_block_route_roundtrips_cryptographic_algorithm_extension_boundaries(self):
        from artifact_support import INSTALL, ROOT
        from c_route_models import ModelCases
        catalog = json.loads((ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json').read_text())
        target = next(item for item in catalog['normative_targets']
                      if item['target_id'] == 'KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-EXTENSIONS-8XXXXXXX')
        result = ModelCases(INSTALL / 'lib/libkmipkit_ffi.so', catalog).execute(
            target, '2.1', 'kmipkit_key_block_from_fields')
        self.assertEqual(result['sample_count'], 2)
        self.assertEqual([sample[2][1][2] for sample in result['normalized_tree']],
                         ['80000000', '8fffffff'])


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

    def test_item_type_enum_route_uses_named_enumeration_constructor(self):
        executor = self.value_executor()
        result = executor.execute({'requires': {'enum_type': 'Item Type'}}, '2.1')
        self.assertEqual(result['known_values_checked'], 11)
        expected = sorted(int(target['wire_value'], 16) for target in executor.catalog['normative_targets']
                          if target['kind'] == 'enumeration_value'
                          and target['parent_element_ids'] == ['KMIPKIT-ELEM-ENUMERATION-ITEM-TYPE'])
        self.assertEqual(result['values_checked'], expected)
        self.assertIn('kmipkit_ttlv_value_enumeration', result['invoked_symbols'])


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

    def test_attribute_local_routes_cover_attribute_cases_without_protocol_scenarios(self):
        self.assertTrue(self.runner.should_execute_attribute_case(
            {'transport': None}, {'category': 'attributes', 'requires': {}}, {'kind': 'attribute'}))
        self.assertFalse(self.runner.should_execute_attribute_case(
            {'transport': None}, {'category': 'attributes', 'requires': {'protocol_scenario': 'lifecycle'}},
            {'kind': 'attribute'}))
        self.assertFalse(self.runner.should_execute_attribute_case(
            {'transport': 'raw_tls'}, {'category': 'attributes', 'requires': {}}, {'kind': 'attribute'}))
        self.assertFalse(self.runner.should_execute_attribute_case(
            {'transport': None}, {'category': 'object_lifecycle', 'requires': {}}, {'kind': 'attribute'}))

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
