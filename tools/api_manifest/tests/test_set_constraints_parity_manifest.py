"""Require an approved four-language Set Constraints mapping before release readiness."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

from tools.api_manifest.tests.parity_support import assert_declaration_mappings

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
GENERATED_PARITY = ROOT / "tests" / "fixtures" / "api" / "generated" / "public_api_parity.json"
OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-SET-CONSTRAINTS"
OWNER_SPEC = "specs/025-policy-controls/spec.md"
LANGUAGES = {"rust", "c", "java", "python"}
EXPECTED_MAPPINGS = {
    "rust": {
        "kmipkit_client::Client::set_constraints",
        "kmipkit_client::ClientRequest::SetConstraints",
        "kmipkit_client::ClientBatchOutcome::SetConstraints",
        "kmipkit_protocol::SetConstraintsRequest",
        "kmipkit_protocol::SetConstraintsRequest::constraints",
        "kmipkit_protocol::SetConstraintsResponse",
        "kmipkit_protocol::SetConstraintsResponse::with_ttlv",
    },
    "c": {
        "kmipkit_client_batch_item_set_constraints",
        "kmipkit_client_batch_item_response_set_constraints_ttlv",
    },
    "java": {
        "org.kmipkit.extensions.ClientBatchItem.setConstraints",
        "org.kmipkit.extensions.ClientBatchItemResult.setConstraintsResponse",
        "org.kmipkit.extensions.ClientBatchItemResult.SetConstraintsResponse",
        "org.kmipkit.extensions.ClientBatchItemResult.SetConstraintsResponse.responseTtlv",
        "org.kmipkit.extensions.ClientBatchItemResult.SetConstraintsResponse.close",
    },
    "python": {
        "kmipkit.extensions.client_batch_item_set_constraints",
        "kmipkit.client.BatchItemResult.set_constraints_response",
        "kmipkit.client.SetConstraintsResponse",
        "kmipkit.client.SetConstraintsResponse.response_ttlv",
        "kmipkit.client.SetConstraintsResponse.close",
    },
}
PARITY_EVIDENCE = {
    "crates/kmipkit-client/tests/unit/set_constraints_execution_tests.rs",
    "crates/kmipkit-ffi/tests/support/set_constraints_abi_tests.rs",
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/SetConstraintsJavaFacadeTest.java",
    "bindings/java/src/test/java/org/kmipkit/extensions/SetConstraintsResponseMappingTest.java",
    "bindings/python/tests/test_set_constraints_operation.py",
}
REQUIREMENTS = {
    "KMIPKIT-0039-FR-001",
    "KMIPKIT-0039-FR-002",
    "KMIPKIT-0039-FR-004",
    "KMIPKIT-0039-FR-005",
    "KMIPKIT-0039-FR-006",
    "KMIPKIT-0039-FR-008",
    "KMIPKIT-0043-FR-004",
    "KMIPKIT-0043-FR-005",
    "KMIPKIT-0043-FR-011",
    "KMIPKIT-0043-FR-021",
}


class SetConstraintsParityManifestTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        self.operations = [
            row for row in self.manifest["operations"] if row["operationId"] == OPERATION_ID
        ]
        self.declarations = {
            row["declarationId"]: row for row in self.manifest["publicDeclarations"]
        }

    def test_manifest_kinds_match_java_types_and_python_fields(self) -> None:
        expected_kinds = {
            "T066:result:type:java": "type",
            "T066:result:type:python": "type",
            "T066:result:responseTtlv:java": "java-member",
            "T066:result:responseTtlv:python": "field",
            "T066:response:python": "field",
        }
        for declaration_id, kind in expected_kinds.items():
            with self.subTest(declaration=declaration_id):
                self.assertEqual(kind, self.declarations[declaration_id]["kind"])

    def test_set_constraints_has_approved_four_language_mapping_and_evidence(self) -> None:
        self.assertEqual(1, len(self.operations))
        operation = self.operations[0]
        self.assertEqual("Set Constraints", operation["name"])
        self.assertEqual("6.1.52", operation["section"])
        self.assertEqual("client_1_0", operation["scope"])
        self.assertEqual("verified", operation["apiState"])
        self.assertEqual("KMIPKIT-0039", operation["featureSpecId"])
        self.assertEqual("approved", operation["ownerDependency"]["state"])
        self.assertEqual(OWNER_SPEC, operation["approvalEvidence"]["path"])
        self.assertEqual(LANGUAGES, set(operation["declarations"]))
        self.assertTrue(PARITY_EVIDENCE <= set(operation["parityEvidenceIds"]))
        for evidence in PARITY_EVIDENCE:
            self.assertTrue((ROOT / evidence).is_file(), evidence)

        assert_declaration_mappings(
            self,
            operation,
            self.declarations,
            operation_id=OPERATION_ID,
            languages=LANGUAGES,
            expected_mappings=EXPECTED_MAPPINGS,
            required_requirements=REQUIREMENTS,
            expected_approval_source=OWNER_SPEC,
        )

    def test_generated_parity_inventory_contains_verified_mapping(self) -> None:
        generated = json.loads(GENERATED_PARITY.read_text(encoding="utf-8"))
        operations = [
            row for row in generated["operations"] if row["operationId"] == OPERATION_ID
        ]
        self.assertEqual(1, len(operations))
        self.assertEqual("verified", operations[0]["apiState"])
        self.assertEqual(LANGUAGES, set(operations[0]["declarations"]))

    def test_set_constraints_python_dataclass_artifacts_are_exactly_excluded(self) -> None:
        generated_members = (
            "__dataclass_fields__",
            "__dataclass_params__",
            "__delattr__",
            "__eq__",
            "__hash__",
            "__init__",
            "__match_args__",
            "__repr__",
            "__setattr__",
            "__slots__",
        )
        expected = {
            f"kmipkit.client.SetConstraintsResponse.{member}"
            for member in generated_members
        }
        exclusions = {
            row["exportName"]: row
            for row in self.manifest["approvedExclusions"]
            if row["packageId"] == "python-kmipkit"
            and row["exportName"].startswith(
                "kmipkit.client.SetConstraintsResponse."
            )
        }
        self.assertEqual(expected, set(exclusions))


if __name__ == "__main__":
    unittest.main()
