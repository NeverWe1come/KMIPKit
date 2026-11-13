"""Pin the Rust-only KMIPKIT-0101 lifecycle builder declarations."""

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
COMMON = {
    "validate", "build_request", "to_batch_item", "execute",
    "with_options", "with_asynchronous_indicator",
}
BUILDERS = {
    "ActivateBuilder": COMMON | {"unique_identifier"},
    "ArchiveBuilder": COMMON | {"unique_identifier"},
    "RecoverBuilder": COMMON | {"unique_identifier"},
    "RevokeBuilder": COMMON | {
        "unique_identifier", "revocation_message", "compromise_occurrence_date",
    },
    "LocateBuilder": COMMON | {
        "attributes", "maximum_items", "offset_items", "storage_status_mask",
        "object_group_member",
    },
    "RekeyBuilder": COMMON | {
        "unique_identifier", "offset", "attributes", "protection_storage_masks",
    },
    "RekeyKeyPairBuilder": COMMON | {
        "private_key_unique_identifier", "offset", "common_attributes",
        "private_key_attributes", "public_key_attributes",
        "common_protection_storage_masks", "private_protection_storage_masks",
        "public_protection_storage_masks",
    },
    "RecertifyBuilder": COMMON | {
        "unique_identifier", "certificate_request_unique_identifier",
        "certificate_request_type", "certificate_request_value", "offset", "attributes",
        "protection_storage_masks",
    },
}
CONSTRUCTORS = {
    "activate", "archive", "recover", "revoke", "locate", "rekey",
    "rekey_key_pair", "recertify",
}


class FluentLifecycleManifestTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(
            (ROOT / "specification/api/public-api.json").read_text(encoding="utf-8")
        )

    def test_exact_root_and_module_public_paths_are_blocked_for_binding_parity(self) -> None:
        expected = {}
        for prefix in ("kmipkit_client::", "kmipkit_client::fluent::"):
            for builder, methods in BUILDERS.items():
                expected[f"{prefix}{builder}"] = "type"
                for method in methods:
                    expected[f"{prefix}{builder}<'a>::{method}"] = "associated-item"
            for constructor in CONSTRUCTORS:
                expected[f"{prefix}RequestBuilder<'a>::{constructor}"] = "associated-item"

        rows = [
            row for row in self.manifest["publicDeclarations"]
            if row.get("candidateSpecId") == "KMIPKIT-0101"
        ]
        self.assertEqual(expected, {row["exportName"]: row["kind"] for row in rows})
        self.assertEqual(188, len(rows))
        for row in rows:
            with self.subTest(export=row["exportName"]):
                self.assertEqual("rust-kmipkit-client", row["packageId"])
                self.assertEqual("blocked", row["declarationState"])
                self.assertEqual("four-language", row["parityScope"])
                self.assertEqual("fluent_dx_api", row["capabilityId"])
                self.assertTrue(row["requirementIds"])
                self.assertTrue(all(
                    value.startswith("KMIPKIT-0101-FR-")
                    for value in row["requirementIds"]
                ))
                self.assertTrue(row["blockedReason"])
                self.assertNotIn("languageMappings", row)
                self.assertNotIn("parityEvidenceIds", row)

    def test_each_operation_constructor_is_mapped_to_its_builder_requirement(self) -> None:
        rows = {
            row["exportName"].rsplit("::", 1)[-1]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("candidateSpecId") == "KMIPKIT-0101"
            and "RequestBuilder<'a>::" in row["exportName"]
        }
        for constructor in CONSTRUCTORS:
            with self.subTest(constructor=constructor):
                self.assertIn("KMIPKIT-0101-FR-001", rows[constructor]["requirementIds"])
                self.assertIn("KMIPKIT-0101-FR-005", rows[constructor]["requirementIds"])

    def test_locate_and_rekey_fields_are_traceable_to_attribute_requirement(self) -> None:
        rows = {
            row["exportName"].rsplit("::", 1)[-1]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("candidateSpecId") == "KMIPKIT-0101"
        }
        for method in (
            "attributes", "storage_status_mask", "object_group_member",
            "common_attributes", "private_key_attributes", "public_key_attributes",
        ):
            with self.subTest(method=method):
                matches = [row for name, row in rows.items() if name == method]
                self.assertTrue(matches)
                self.assertTrue(all(
                    "KMIPKIT-0101-FR-003" in row["requirementIds"] for row in matches
                ))


if __name__ == "__main__":
    unittest.main()
