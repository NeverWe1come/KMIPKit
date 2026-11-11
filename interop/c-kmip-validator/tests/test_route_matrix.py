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


if __name__ == '__main__':
    unittest.main()
