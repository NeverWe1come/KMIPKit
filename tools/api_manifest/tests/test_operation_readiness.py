"""Readiness contracts for operations, capabilities, and public declarations."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

from jsonschema import Draft202012Validator

from tools.api_manifest.tests.test_manifest_v2 import _valid_v2_manifest


ROOT = Path(__file__).resolve().parents[3]
SCHEMA_PATH = ROOT / "specification" / "api" / "public-api.schema.json"
LANGUAGES = ("rust", "c", "java", "python")


def _contract_manifest() -> dict[str, object]:
    return _valid_v2_manifest()


def _discover_versions_operation(manifest: dict[str, object]) -> dict[str, object]:
    return next(
        operation
        for operation in manifest["operations"]
        if operation["name"] == "Discover Versions"
    )


def _approved_feature_source() -> dict[str, str]:
    return {
        "kind": "feature-specification",
        "path": "specs/0007-client-execution/approval-record.md",
        "status": "Approved",
        "evidence": "Active release-line approval record",
    }


def _four_language_mappings() -> dict[str, list[str]]:
    return {
        "rust": ["kmipkit_client::Client::execute"],
        "c": ["kmipkit_client_execute"],
        "java": ["org.kmipkit.Client.execute"],
        "python": ["kmipkit.Client.execute"],
    }


class OperationReadinessSchemaTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.schema = json.loads(SCHEMA_PATH.read_text(encoding="utf-8"))
        cls.validator = Draft202012Validator(cls.schema)

    def errors(self, manifest: dict[str, object]) -> list[object]:
        return list(self.validator.iter_errors(manifest))

    def test_blocked_operation_records_only_candidate_and_reason(self) -> None:
        manifest = _contract_manifest()
        operation = manifest["operations"][0]
        operation["candidateSpecId"] = "KMIPKIT-0009"
        self.assertEqual(self.errors(manifest), [])

        operation["featureSpecId"] = "KMIPKIT-0009"
        operation["approvalEvidence"] = _approved_feature_source()
        operation["declarations"] = _four_language_mappings()
        self.assertTrue(self.errors(manifest))

    def test_declared_operation_requires_approved_owner_and_all_language_mappings(self) -> None:
        manifest = _contract_manifest()
        operation = _discover_versions_operation(manifest)
        operation.pop("blockedReason")
        operation["apiState"] = "declared"
        operation["featureSpecId"] = "KMIPKIT-0007"
        operation["approvalEvidence"] = _approved_feature_source()
        operation["declarations"] = _four_language_mappings()
        self.assertEqual(self.errors(manifest), [])

        del operation["declarations"]["java"]
        self.assertTrue(self.errors(manifest))

    def test_verified_operation_requires_resolvable_parity_evidence(self) -> None:
        manifest = _contract_manifest()
        operation = _discover_versions_operation(manifest)
        operation.pop("blockedReason")
        operation["apiState"] = "verified"
        operation["featureSpecId"] = "KMIPKIT-0007"
        operation["approvalEvidence"] = _approved_feature_source()
        operation["declarations"] = _four_language_mappings()
        self.assertTrue(self.errors(manifest))

        operation["parityEvidenceIds"] = ["tests/fixtures/api/client-parity.json"]
        self.assertEqual(self.errors(manifest), [])

    def test_blocked_observed_export_requires_exact_candidate_and_reason_without_mappings(self) -> None:
        manifest = _contract_manifest()
        manifest["publicDeclarations"] = [
            {
                "declarationId": "observed_draft_export",
                "packageId": "rust-kmipkit-client",
                "kind": "function",
                "exportName": "kmipkit_client::candidate",
                "capabilityId": "candidate_operation",
                "requirementIds": ["KMIPKIT-0043-FR-001"],
                "declarationState": "blocked",
                "parityScope": "four-language",
                "candidateSpecId": "KMIPKIT-0017",
                "blockedReason": "The owning operation specification is not approved.",
            }
        ]
        self.assertEqual(self.errors(manifest), [])

        del manifest["publicDeclarations"][0]["candidateSpecId"]
        self.assertTrue(self.errors(manifest))

    def test_capability_state_uses_required_parity_scope_mappings(self) -> None:
        manifest = _contract_manifest()
        manifest["capabilities"] = [
            {
                "capabilityId": "shared_client",
                "requirementIds": ["KMIPKIT-0043-FR-001"],
                "capabilityState": "declared",
                "parityScope": "four-language",
                "ownerSpecId": "KMIPKIT-0007",
                "approvalEvidence": _approved_feature_source(),
                "mappings": _four_language_mappings(),
            }
        ]
        self.assertEqual(self.errors(manifest), [])

        del manifest["capabilities"][0]["mappings"]["python"]
        self.assertTrue(self.errors(manifest))

if __name__ == "__main__":
    unittest.main()
