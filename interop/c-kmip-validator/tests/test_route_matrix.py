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


if __name__ == '__main__':
    unittest.main()
