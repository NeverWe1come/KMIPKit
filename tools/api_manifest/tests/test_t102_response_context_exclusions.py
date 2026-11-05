"""Pin exact exclusions for Rust-local cryptographic response framing helpers.

The represented operation result fields follow OASIS KMIP 2.1: §6.1.24
Tables 235–237, §6.1.32 Tables 259–261, §6.1.33 Tables 262–264,
§6.1.55 Tables 334–336, and §6.1.56 Tables 337–339.
"""

from __future__ import annotations

import json
import unittest
from pathlib import Path

from tools.api_manifest.tests.t102_manifest_totals import (
    T102_APPROVED_EXCLUSIONS,
    T102_BLOCKED_CANDIDATES,
    T102_PUBLIC_DECLARATIONS,
)


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
OWNER_SPEC = "specs/021-hash-mac-signature-operations/spec.md"
MAPPING_SPEC = "specs/043-public-api-bindings/spec.md"
EVIDENCE = "tools/api_manifest/tests/test_t102_response_context_exclusions.py"
CONTEXT_DECLARATIONS = {
    "rust-observed-105154cf933c88d0097a": "kmipkit_protocol::CryptographicOperationResponseContext",
    "rust-observed-d61e7db174296bffc941": "kmipkit_protocol::CryptographicOperationResponseContext::SinglePart",
    "rust-observed-c7f9909f0622669a2532": "kmipkit_protocol::CryptographicOperationResponseContext::MultipartNonFinal",
    "rust-observed-4733cafd49358dd11b06": "kmipkit_protocol::CryptographicOperationResponseContext::MultipartFinal",
    "rust-observed-cea44457a95c424fbf76": "kmipkit_protocol::VerificationResponseContext",
    "rust-observed-4978456989659fff3b83": "kmipkit_protocol::VerificationResponseContext::MultipartFinal",
    "rust-observed-523db7d4167b1d768b5b": "kmipkit_protocol::VerificationResponseContext::MultipartNonFinal",
    "rust-observed-fea788ac7ff4e43be1be": "kmipkit_protocol::VerificationResponseContext::SinglePart",
}


class T102ResponseContextExclusionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))

    def test_context_types_and_variants_have_exact_reviewed_exclusions(self) -> None:
        declarations = {
            row["declarationId"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("declarationId") in CONTEXT_DECLARATIONS
        }
        self.assertEqual({}, declarations)
        exclusions = {
            row["exportName"]: row
            for row in self.manifest["approvedExclusions"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("candidateSpecId") == "KMIPKIT-0043"
            and row.get("exportName") in CONTEXT_DECLARATIONS.values()
        }
        self.assertEqual(set(CONTEXT_DECLARATIONS.values()), set(exclusions))
        for declaration_id, export_name in CONTEXT_DECLARATIONS.items():
            with self.subTest(export_name=export_name):
                exclusion = exclusions[export_name]
                self.assertIn("response-framing context", exclusion["reason"])
                self.assertEqual(OWNER_SPEC, exclusion["approvalSource"]["path"])
                self.assertEqual("Approved", exclusion["approvalSource"]["status"])
                self.assertIn(declaration_id, exclusion["evidence"])
                self.assertIn(MAPPING_SPEC, exclusion["evidence"])
                self.assertIn("KMIPKIT-0021-FR-008", exclusion["evidence"])
                self.assertIn("KMIPKIT-0043-FR-021", exclusion["evidence"])
                self.assertIn(EVIDENCE, exclusion["evidence"])

    def test_exclusion_preserves_typed_operation_results_and_open_discussion(self) -> None:
        source = (ROOT / "specs/021-hash-mac-signature-operations/spec.md").read_text(
            encoding="utf-8"
        )
        self.assertIn("KMIPKIT-DISC-048", source)
        self.assertIn("Validity Indicator", source)
        operation_ids = {
            "KMIPKIT-ELEM-OP-C2S-HASH",
            "KMIPKIT-ELEM-OP-C2S-MAC",
            "KMIPKIT-ELEM-OP-C2S-MAC-VERIFY",
            "KMIPKIT-ELEM-OP-C2S-SIGN",
            "KMIPKIT-ELEM-OP-C2S-SIGNATURE-VERIFY",
        }
        operations = {
            row["operationId"]: row
            for row in self.manifest["operations"]
            if row.get("operationId") in operation_ids
        }
        self.assertEqual(operation_ids, set(operations))
        for operation in operations.values():
            self.assertEqual("declared", operation["apiState"])
            self.assertEqual({"rust", "c", "java", "python"}, set(operation["declarations"]))
            self.assertTrue(all(operation["declarations"][language] for language in ("rust", "c", "java", "python")))
        self.assertEqual(T102_PUBLIC_DECLARATIONS, len(self.manifest["publicDeclarations"]))
        self.assertEqual(T102_APPROVED_EXCLUSIONS, len(self.manifest["approvedExclusions"]))
        blocked = sum(
            row.get("declarationState") == "blocked"
            and row.get("candidateSpecId") == "KMIPKIT-0043"
            for row in self.manifest["publicDeclarations"]
        )
        self.assertEqual(T102_BLOCKED_CANDIDATES, blocked)


if __name__ == "__main__":
    unittest.main()
