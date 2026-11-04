"""Keep opaque PKCS#11 request extensions mapped across all public languages.

KMIP Specification v2.1 §6.1.37 Table 273 lists the standard request fields
but does not define a generic extension field. These declarations represent
KMIPKIT-0040's caller-controlled opaque forwarding capability; they make no
claim that arbitrary extra Table 273 fields are standard or interoperable.
"""

from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
CAPABILITY = "KMIPKIT-ELEM-OP-C2S-PKCS-11"
OWNER = "specs/026-pkcs11/spec.md"
TARGETS = (
    (
        "rust-observed-ad5c5204337d486b06bb",
        "rust-kmipkit-client",
        "kmipkit_client::Pkcs11Request::with_extension",
        "kmipkit_client::Pkcs11Request::with_interface",
    ),
    (
        "rust-observed-358ceac77d04d65b6238",
        "rust-kmipkit-protocol",
        "kmipkit_protocol::Pkcs11Request::with_extension",
        "kmipkit_protocol::Pkcs11Request::with_interface",
    ),
)
OWNER_REQUIREMENTS = ("KMIPKIT-0040-FR-002", "KMIPKIT-0040-FR-004")
PARITY_EVIDENCE = {
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/extensions/Pkcs11MappingTest.java",
    "bindings/python/tests/test_pkcs11_operation.py",
    "crates/kmipkit-ffi/tests/support/pkcs11_abi_tests.rs",
}


class T102Pkcs11RequestExtensionMappingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.declarations = {
            row["declarationId"]: row for row in cls.manifest["publicDeclarations"]
        }
        cls.by_name = {
            (row["packageId"], row["exportName"]): row
            for row in cls.manifest["publicDeclarations"]
        }

    def test_opaque_extension_methods_map_to_the_approved_four_language_capability(self) -> None:
        native_function = next(
            row
            for row in self.manifest["functions"]
            if row["id"] == "client_batch_item_pkcs11_with_extensions_v1"
        )
        self.assertEqual(
            "kmipkit_client_batch_item_pkcs11_with_extensions_v1",
            native_function["c"]["symbol"],
        )
        operation = next(
            row
            for row in self.manifest["operations"]
            if row["operationId"] == CAPABILITY
        )
        c_declaration = next(
            row
            for row in self.manifest["publicDeclarations"]
            if row.get("exportName")
            == "kmipkit_client_batch_item_pkcs11_with_extensions_v1"
        )
        self.assertEqual("verified", c_declaration["declarationState"])
        self.assertEqual(CAPABILITY, c_declaration["capabilityId"])
        self.assertEqual(
            "kmipkit_client_batch_item_pkcs11_with_extensions_v1",
            c_declaration["languageMappings"]["c"]["name"],
        )
        self.assertIn(c_declaration["declarationId"], operation["declarations"]["c"])
        for declaration_id, package_id, export_name, sibling_name in TARGETS:
            with self.subTest(declaration=export_name):
                declaration = self.declarations[declaration_id]
                sibling = self.by_name[(package_id, sibling_name)]
                self.assertEqual(export_name, declaration["exportName"])
                self.assertEqual("verified", declaration["declarationState"])
                self.assertEqual(CAPABILITY, declaration["capabilityId"])
                self.assertEqual(OWNER, declaration["approvalSource"]["path"])
                self.assertEqual("Approved", declaration["approvalSource"]["status"])
                self.assertEqual(
                    list(OWNER_REQUIREMENTS)
                    + [
                        requirement_id
                        for requirement_id in sibling["requirementIds"]
                        if requirement_id.startswith("KMIPKIT-0043-")
                    ],
                    declaration["requirementIds"],
                )
                self.assertEqual("four-language", declaration["parityScope"])
                self.assertEqual(
                    sibling["languageMappings"]["java"],
                    declaration["languageMappings"]["java"],
                )
                self.assertEqual(
                    sibling["languageMappings"]["python"],
                    declaration["languageMappings"]["python"],
                )
                self.assertEqual(
                    "kmipkit_client_batch_item_pkcs11_with_extensions_v1",
                    declaration["languageMappings"]["c"]["name"],
                )
                self.assertTrue(
                    PARITY_EVIDENCE.issubset(set(declaration["parityEvidenceIds"]))
                )
                self.assertNotIn("candidateSpecId", declaration)
                self.assertNotIn("blockedReason", declaration)
                self.assertIn(declaration_id, operation["declarations"]["rust"])

    def test_java_raw_handle_bridge_is_excluded_behind_the_typed_facade(self) -> None:
        export_name = (
            "org.kmipkit.NativeExtensionRegistry.clientBatchItemPkcs11WithExtensions"
        )
        exclusion = next(
            row
            for row in self.manifest["approvedExclusions"]
            if row["packageId"] == "java-kmipkit" and row["exportName"] == export_name
        )
        self.assertEqual("specs/043-public-api-bindings/spec.md", exclusion["approvalSource"]["path"])
        self.assertEqual("Approved", exclusion["approvalSource"]["status"])
        self.assertIn("typed PKCS#11 facade", exclusion["reason"])
        self.assertIn("KMIPKIT-0040-FR-002/004", exclusion["evidence"])
        self.assertIn("KMIPKIT-0043-FR-008", exclusion["evidence"])
        self.assertIn("KMIPKIT-0043-FR-021", exclusion["evidence"])
        self.assertNotIn(
            ("java-kmipkit", export_name),
            {
                (row["packageId"], row["exportName"])
                for row in self.manifest["publicDeclarations"]
            },
        )


if __name__ == "__main__":
    unittest.main()
