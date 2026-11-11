"""Pin the Rust-only KMIPKIT-0099 API without claiming binding parity."""

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
METHODS = {
    "object_type", "method", "parts", "threshold", "unique_identifier",
    "prime_field_size", "protection_storage_masks", "attribute",
    "attribute_item", "attributes", "vendor_attribute", "attribute_at",
    "with_options", "with_asynchronous_indicator", "validate", "build",
    "build_request", "to_batch_item", "execute",
}
ERRORS = {
    "MissingCreateSplitKeyObjectType", "MissingCreateSplitKeyMethod",
    "MissingCreateSplitKeyParts", "MissingCreateSplitKeyThreshold",
}


class FluentCreateSplitKeyManifestTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(
            (ROOT / "specification/api/public-api.json").read_text(encoding="utf-8")
        )

    def test_exact_public_paths_are_owned_and_blocked_for_binding_parity(self) -> None:
        expected = {}
        for prefix in ("kmipkit_client::", "kmipkit_client::fluent::"):
            expected[f"{prefix}CreateSplitKeyBuilder"] = "type"
            expected[f"{prefix}RequestBuilder<'a>::create_split_key"] = "associated-item"
            for method in METHODS:
                expected[f"{prefix}CreateSplitKeyBuilder<'a>::{method}"] = "associated-item"
            for error in ERRORS:
                expected[f"{prefix}FluentBuildError::{error}"] = "variant"

        rows = [row for row in self.manifest["publicDeclarations"]
                if row.get("candidateSpecId") == "KMIPKIT-0099"]
        self.assertEqual(expected, {row["exportName"]: row["kind"] for row in rows})
        self.assertEqual(50, len(rows))
        for row in rows:
            with self.subTest(export=row["exportName"]):
                self.assertEqual(row["packageId"], "rust-kmipkit-client")
                self.assertEqual(row["declarationState"], "blocked")
                self.assertEqual(row["parityScope"], "four-language")
                self.assertEqual(row["capabilityId"], "fluent_dx_api")
                self.assertTrue(row["requirementIds"])
                self.assertTrue(all(value.startswith("KMIPKIT-0099-FR-")
                                    for value in row["requirementIds"]))
                self.assertTrue(row["blockedReason"])
                self.assertNotIn("languageMappings", row)
                self.assertNotIn("parityEvidenceIds", row)

    def test_methods_are_tied_to_the_behavior_they_expose(self) -> None:
        rows = {row["exportName"].rsplit("::", 1)[-1]: row
                for row in self.manifest["publicDeclarations"]
                if row.get("candidateSpecId") == "KMIPKIT-0099"
                and "CreateSplitKeyBuilder<'a>::" in row["exportName"]}
        for method in ("object_type", "method", "parts", "threshold"):
            self.assertIn("KMIPKIT-0099-FR-001", rows[method]["requirementIds"])
        for method in ("unique_identifier", "prime_field_size", "protection_storage_masks"):
            self.assertIn("KMIPKIT-0099-FR-002", rows[method]["requirementIds"])
        for method in ("attribute", "attribute_item", "attributes", "vendor_attribute", "attribute_at"):
            self.assertIn("KMIPKIT-0099-FR-003", rows[method]["requirementIds"])
        self.assertIn("KMIPKIT-0099-FR-004", rows["validate"]["requirementIds"])
        self.assertIn("KMIPKIT-0099-FR-006", rows["with_options"]["requirementIds"])
        self.assertIn("KMIPKIT-0099-FR-006", rows["with_asynchronous_indicator"]["requirementIds"])

    def test_exclusions_only_cover_exact_standard_trait_paths(self) -> None:
        rows = [row for row in self.manifest["approvedExclusions"]
                if row.get("approvalSource", {}).get("path")
                == "specs/099-fluent-create-split-key/spec.md"]
        self.assertEqual(len(rows), 18)
        for row in rows:
            with self.subTest(export=row["exportName"]):
                self.assertTrue(row["exportName"].startswith("<kmipkit_client::"))
                self.assertIn("CreateSplitKeyBuilder<'a> as core::", row["exportName"])
                self.assertIn("KMIPKIT-0099-FR-001", row["evidence"])
                self.assertNotIn("*", row["exportName"])


if __name__ == "__main__":
    unittest.main()
