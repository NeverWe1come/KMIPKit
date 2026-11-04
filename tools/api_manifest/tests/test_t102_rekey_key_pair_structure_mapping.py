"""Pin the exact six Re-key Key Pair Table 310 Structure setters for T102."""

from __future__ import annotations

import hashlib
import json
import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
OASIS_HTML = ROOT / "specification" / "oasis" / "kmip-2.1" / "upstream" / "kmip-spec-v2.1-os.html"
OWNER = "specs/022-object-lifecycle/spec.md"
OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-RE-KEY-KEY-PAIR"
OPERATION_NAME = "kmipkit_protocol::RekeyKeyPairRequest::"
OWNER_FR = {
    "KMIPKIT-0036-FR-001",
    "KMIPKIT-0036-FR-002",
    "KMIPKIT-0036-FR-004",
    "KMIPKIT-0036-FR-008",
}
FORBIDDEN_REQUEST_REQUIREMENTS = {
    "KMIPKIT-REQ-SPEC-6.1.47-001",
    "KMIPKIT-REQ-SPEC-6.1.47-004-001",
    "KMIPKIT-REQ-SPEC-6.1.47-004-002",
}
T102_BLOCKED_CANDIDATES_AFTER_BATCH = 663
EVIDENCE = {
    "crates/kmipkit-protocol/tests/unit/rekey_key_pair_operation_tests.rs",
    "crates/kmipkit-ffi/tests/c_api_coverage.rs",
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java",
    "bindings/python/tests/test_rekey_key_pair_native_integration.py",
    "tools/api_manifest/tests/test_t102_rekey_key_pair_structure_mapping.py",
}
TARGETS = (
    ("rust-observed-9deea2dde23e9a92294d", "with_common_attributes", 0x420126),
    ("rust-observed-f67674816d29b1f2879f", "with_private_key_attributes", 0x420127),
    ("rust-observed-c084e31187a809b1d6bb", "with_public_key_attributes", 0x420128),
    ("rust-observed-f428c5bc2011a1634a10", "with_common_protection_storage_masks", 0x420163),
    ("rust-observed-d51793dbfa80b42a8e94", "with_private_protection_storage_masks", 0x420164),
    ("rust-observed-1bdba1f1266aff84ed52", "with_public_protection_storage_masks", 0x420165),
)
EXPECTED_FOREIGN_MAPPINGS = {
    "c": {"packageId": "c-kmipkit", "name": "kmipkit_client_batch_item_rekey_key_pair"},
    "java": {
        "packageId": "java-kmipkit",
        "name": "org.kmipkit.extensions.ClientBatchItem.rekeyKeyPair",
    },
    "python": {
        "packageId": "python-kmipkit",
        "name": "kmipkit.extensions.client_batch_item_rekey_key_pair",
    },
}


class T102RekeyKeyPairStructureMappingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.declarations = {
            row["declarationId"]: row for row in cls.manifest["publicDeclarations"]
        }

    def test_exact_six_table_310_setters_map_to_existing_four_language_builder(self) -> None:
        self.assertEqual(6, len(TARGETS))
        operation = next(
            row for row in self.manifest["operations"]
            if row["operationId"] == OPERATION_ID
        )
        rust_declarations = set(operation["declarations"]["rust"])
        for declaration_id, method, tag in TARGETS:
            with self.subTest(method=method, tag=f"{tag:06x}"):
                declaration = self.declarations[declaration_id]
                expected_name = OPERATION_NAME + method
                self.assertEqual(expected_name, declaration["exportName"])
                self.assertEqual("verified", declaration["declarationState"])
                self.assertNotIn("candidateSpecId", declaration)
                self.assertNotIn("blockedReason", declaration)
                self.assertEqual("associated-item", declaration["kind"])
                self.assertEqual(OPERATION_ID, declaration["capabilityId"])
                self.assertEqual("four-language", declaration["parityScope"])
                self.assertEqual(OWNER_FR, {
                    requirement for requirement in declaration["requirementIds"]
                    if requirement.startswith("KMIPKIT-0036-")
                })
                self.assertFalse(
                    FORBIDDEN_REQUEST_REQUIREMENTS.intersection(declaration["requirementIds"])
                )
                self.assertFalse(
                    any(value.startswith("KMIPKIT-REQ-SPEC-6.1.47-")
                        for value in declaration["requirementIds"])
                )
                expected_mappings = {
                    "rust": {"packageId": "rust-kmipkit-protocol", "name": expected_name},
                    **EXPECTED_FOREIGN_MAPPINGS,
                }
                self.assertEqual(expected_mappings, declaration["languageMappings"])
                self.assertEqual(
                    {"kind": "feature-specification", "path": OWNER, "status": "Approved",
                     "evidence": "**Status**: Approved for implementation — 2026-10-11"},
                    declaration["approvalSource"],
                )
                self.assertTrue(EVIDENCE <= set(declaration["parityEvidenceIds"]))
                self.assertIn(declaration_id, rust_declarations)

    def test_oasis_table_numbering_and_four_language_wire_evidence_are_pinned(self) -> None:
        raw_html = OASIS_HTML.read_bytes()
        self.assertEqual(
            "8BF9D914C097E98A6509AA1FFCBF03406F738066E940597AEE93D0A5E07ADDCF",
            hashlib.sha256(raw_html).hexdigest().upper(),
        )
        html = raw_html.decode("cp1252")
        text = " ".join(re.sub(r"<[^>]+>", " ", html).split())
        table_titles = (
            "Table 309: Re-key Key Pair Attribute Requirements",
            "Table 310: Re-key Key Pair Request Payload",
            "Table 311: Re-key Key Pair Response Payload",
            "Table 312: Re-key Key Pair Errors",
        )
        positions = [text.index(title) for title in table_titles]
        self.assertEqual(sorted(positions), positions)
        toc_positions = [text.rindex(title) for title in table_titles]
        self.assertEqual(sorted(toc_positions), toc_positions)
        self.assertIn("Table 308: Computing New Dates from Offset during Re-key Key Pair", text)

        # The pinned source body header is displaced from its ToC/caption title.
        table_310_body = text.index(table_titles[1])
        self.assertIn(
            "Table 310: Re-key Key Pair Request Payload &nbsp; Response Payload",
            text[table_310_body:table_310_body + 120],
        )
        for label in (
            "Common Attributes", "Private Key Attributes", "Public Key Attributes",
            "Common Protection Storage Masks", "Private Protection Storage Masks",
            "Public Protection Storage Masks",
        ):
            self.assertIn(label, text)
        tag_catalog = json.loads(
            (ROOT / "specification" / "catalog" / "kmip-2.1.json").read_text(encoding="utf-8")
        )
        catalog_ids = {row.get("element_id") for row in tag_catalog.get("elements", [])}
        self.assertTrue({
            "KMIPKIT-ELEM-TAG-420126", "KMIPKIT-ELEM-TAG-420127",
            "KMIPKIT-ELEM-TAG-420128", "KMIPKIT-ELEM-TAG-420163",
            "KMIPKIT-ELEM-TAG-420164", "KMIPKIT-ELEM-TAG-420165",
        } <= catalog_ids)

        source_expectations = {
            "crates/kmipkit-protocol/tests/unit/rekey_key_pair_operation_tests.rs": (
                "request_emits_table_310_fields_once_in_order_and_preserves_groups",
            ),
            "crates/kmipkit-ffi/tests/c_api_coverage.rs": (
                "C_REKEY_FIELD_TAGS", "C_REKEY_MARKERS", "optional fields are not in KMIP Table 310 order",
            ),
            "bindings/c/tests/client_abi_consumer.c": (
                "0x541101", "0x541102", "0x541103", "0x541104", "0x541105", "0x541106",
            ),
            "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java": (
                "expectedTags = {0x420058, 0x420126, 0x420127, 0x420128, 0x420163, 0x420164, 0x420165}",
                "assertRekeyKeyPairRequest(request)",
            ),
            "bindings/python/tests/test_rekey_key_pair_native_integration.py": (
                "expected_tags = (", "0x420126", "0x420127", "0x420128", "0x420163", "0x420164", "0x420165",
                "common-attributes", "private-masks", "public-masks",
            ),
        }
        for relative_path, required_fragments in source_expectations.items():
            with self.subTest(source=relative_path):
                source = (ROOT / relative_path).read_text(encoding="utf-8")
                for fragment in required_fragments:
                    self.assertIn(fragment, source)

    def test_only_the_exact_six_candidates_leave_the_blocked_inventory(self) -> None:
        blocked = sum(
            row.get("candidateSpecId") == "KMIPKIT-0043"
            and row["declarationState"] == "blocked"
            for row in self.manifest["publicDeclarations"]
        )
        self.assertEqual(T102_BLOCKED_CANDIDATES_AFTER_BATCH, blocked)
        self.assertEqual(6_470, len(self.manifest["publicDeclarations"]))
        self.assertEqual(8_554, len(self.manifest["approvedExclusions"]))


if __name__ == "__main__":
    unittest.main()
