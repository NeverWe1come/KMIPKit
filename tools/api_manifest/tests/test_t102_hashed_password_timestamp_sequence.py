"""T102 contract for caller-owned Hashed Password timestamp sequencing."""

from __future__ import annotations

import csv
import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
CAPABILITY = "hashed_password_timestamp_sequence"
DECLARATIONS = {
    "rust-observed-25f837c102304b00719a": {
        "export": "kmipkit_protocol::HashedPasswordTimestampSequence",
        "kind": "type",
        "rust": "kmipkit_protocol::HashedPasswordTimestampSequence",
        "c": "kmipkit_hashed_password_timestamp_sequence_t",
        "java": "org.kmipkit.credentials.HashedPasswordTimestampSequence",
        "python": "kmipkit.credentials.HashedPasswordTimestampSequence",
    },
    "rust-observed-77ba52e1f640fdc9eb2d": {
        "export": "kmipkit_protocol::HashedPasswordTimestampSequence::new",
        "kind": "associated-item",
        "rust": "kmipkit_protocol::HashedPasswordTimestampSequence::new",
        "c": "kmipkit_hashed_password_timestamp_sequence_new",
        "java": "org.kmipkit.credentials.HashedPasswordTimestampSequence.create",
        "python": "kmipkit.credentials.HashedPasswordTimestampSequence.create",
    },
    "rust-observed-fa6b4407f05c32c6fe5d": {
        "export": "kmipkit_protocol::HashedPasswordTimestampSequence::record",
        "kind": "associated-item",
        "rust": "kmipkit_protocol::HashedPasswordTimestampSequence::record",
        "c": "kmipkit_hashed_password_timestamp_sequence_record",
        "java": "org.kmipkit.credentials.HashedPasswordTimestampSequence.record",
        "python": "kmipkit.credentials.HashedPasswordTimestampSequence.record",
    },
    "rust-observed-3ac6030b68ef122d9547": {
        "export": "kmipkit_protocol::HashedPasswordTimestampSequence::last_timestamp",
        "kind": "associated-item",
        "rust": "kmipkit_protocol::HashedPasswordTimestampSequence::last_timestamp",
        "c": "kmipkit_hashed_password_timestamp_sequence_last_timestamp",
        "java": "org.kmipkit.credentials.HashedPasswordTimestampSequence.lastTimestamp",
        "python": "kmipkit.credentials.HashedPasswordTimestampSequence.last_timestamp",
    },
    "rust-observed-a881aedc1066c475e335": {
        "export": "kmipkit_protocol::HashedPasswordTimestampSequenceError",
        "kind": "type",
        "rust": "kmipkit_protocol::HashedPasswordTimestampSequenceError",
        "c": "KMIPKIT_ERROR_INVALID_INPUT",
        "java": "org.kmipkit.InvalidInputException",
        "python": "kmipkit.errors.InvalidInputError",
    },
}
REQUIREMENTS = {
    "KMIPKIT-0008-FR-007",
    "KMIPKIT-REQ-SPEC-9.11-010-001",
    "KMIPKIT-0043-FR-004",
    "KMIPKIT-0043-FR-005",
    "KMIPKIT-0043-FR-010",
    "KMIPKIT-0043-FR-013",
    "KMIPKIT-0043-FR-017",
    "KMIPKIT-0043-FR-021",
    "KMIPKIT-0043-FR-022",
}


class T102HashedPasswordTimestampSequenceTests(unittest.TestCase):
    def test_exact_sequence_declarations_have_four_language_mappings(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        rows = {
            row["declarationId"]: row
            for row in manifest["publicDeclarations"]
            if row.get("declarationId") in DECLARATIONS
        }
        self.assertEqual(set(DECLARATIONS), set(rows))
        for declaration_id, expected in DECLARATIONS.items():
            with self.subTest(declaration=declaration_id):
                row = rows[declaration_id]
                self.assertEqual(expected["export"], row["exportName"])
                self.assertEqual(expected["kind"], row["kind"])
                self.assertEqual("declared", row["declarationState"])
                self.assertEqual(CAPABILITY, row["capabilityId"])
                self.assertEqual(
                    "specs/008-credentials-attestation/spec.md",
                    row["approvalSource"]["path"],
                )
                self.assertEqual("Approved", row["approvalSource"]["status"])
                self.assertEqual(REQUIREMENTS, set(row["requirementIds"]))
                self.assertEqual("four-language", row["parityScope"])
                self.assertEqual(
                    {"rust", "c", "java", "python"},
                    set(row["languageMappings"]),
                )
                for language, name in expected.items():
                    if language in {"export", "kind"}:
                        continue
                    self.assertEqual(name, row["languageMappings"][language]["name"])

    def test_sequence_capability_and_shared_fixture_are_traced(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        capabilities = {item["capabilityId"]: item for item in manifest["capabilities"]}
        self.assertIn(CAPABILITY, capabilities.keys())
        self.assertIn("KMIPKIT-0008-FR-007", capabilities[CAPABILITY]["requirementIds"])
        invalid_input = next(
            item for item in manifest["errorCategories"] if item["id"] == "invalid_input"
        )
        self.assertEqual("ready", invalid_input["mappingState"])

        fixture_path = ROOT / "tests" / "fixtures" / "api" / "shared-capability-parity.json"
        fixture = json.loads(fixture_path.read_text(encoding="utf-8"))
        cases = {case["id"]: case for case in fixture["cases"]}
        case = cases["credentials.hashed-password-timestamp-sequence.v1"]
        self.assertEqual(CAPABILITY, case["capability"])
        self.assertEqual(
            "OASIS KMIP v2.1 §9.11, Table 415; KMIPKIT-0008 OD-003",
            case["normativeSource"],
        )
        self.assertEqual("invalid_input", case["errorCategory"])
        self.assertEqual(0, case["expectedOutput"]["clockReads"])
        self.assertEqual(0, case["expectedOutput"]["hashCalculations"])
        self.assertEqual(0, case["expectedOutput"]["credentialTransmissions"])

    def test_owner_traceability_does_not_contradict_approved_monotonicity(self) -> None:
        traceability = ROOT / "specification" / "compliance" / "requirements" / "KMIPKIT-0008.csv"
        with traceability.open(encoding="utf-8", newline="") as stream:
            rows = list(csv.DictReader(stream))
        row = next(item for item in rows if item["requirement_id"] == "KMIPKIT-0008-FR-007")
        self.assertIn("strictly increasing", row["statement"])
        self.assertIn("KMIPKIT-REQ-SPEC-9.11-010-001", row["statement"])
        self.assertNotIn("do not calculate hashes or claim monotonicity behavior", row["statement"])

    def test_exact_sequence_trait_artifacts_are_excluded_by_the_credential_owner(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        report_path = (
            ROOT
            / "specs"
            / "043-public-api-bindings"
            / "reports"
            / "rust-export-report-2026-11-01.json"
        )
        report = json.loads(report_path.read_text(encoding="utf-8"))
        expected = {
            row["exportName"]
            for source in (report["declarations"], report["conditionalDeclarations"])
            for row in source
            if "HashedPasswordTimestampSequence" in row["exportName"]
            and " as " in row["exportName"]
        }
        self.assertEqual(36, len(expected))

        unresolved = {
            row["exportName"]
            for row in manifest["publicDeclarations"]
            if row.get("candidateSpecId") == "KMIPKIT-0043"
            and row["exportName"] in expected
        }
        self.assertEqual(set(), unresolved)

        exclusions = {
            row["exportName"]: row
            for row in manifest["approvedExclusions"]
            if row["exportName"] in expected
        }
        self.assertEqual(expected, set(exclusions))
        for export_name, exclusion in exclusions.items():
            with self.subTest(export=export_name):
                self.assertEqual("KMIPKIT-0008", exclusion["candidateSpecId"])
                self.assertIn("standard-trait implementation artifact", exclusion["reason"])
                self.assertEqual(
                    "specs/008-credentials-attestation/spec.md",
                    exclusion["approvalSource"]["path"],
                )
                self.assertEqual("Approved", exclusion["approvalSource"]["status"])
                self.assertIn(export_name, exclusion["evidence"])


if __name__ == "__main__":
    unittest.main()
