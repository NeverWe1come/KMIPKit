"""Pin the Rust-only KMIPKIT-0100 API without claiming binding parity."""

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
METHODS = {
    "object", "object_item", "attribute", "attribute_item", "attributes",
    "attribute_at", "vendor_attribute", "protection_storage_masks",
    "opaque_object_context", "with_options", "with_asynchronous_indicator",
    "validate", "build", "build_request", "to_batch_item", "execute",
}
ERRORS = {"MissingRegisterObject", "InvalidRegisterObject"}


class FluentRegisterManifestTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(
            (ROOT / "specification/api/public-api.json").read_text(encoding="utf-8")
        )

    def test_exact_public_paths_are_owned_and_blocked_for_binding_parity(self) -> None:
        expected = {}
        for prefix in ("kmipkit_client::", "kmipkit_client::fluent::"):
            expected[f"{prefix}RegisterBuilder"] = "type"
            expected[f"{prefix}RequestBuilder<'a>::register"] = "associated-item"
            for method in METHODS:
                expected[f"{prefix}RegisterBuilder<'a>::{method}"] = "associated-item"
            for error in ERRORS:
                expected[f"{prefix}FluentBuildError::{error}"] = "variant"

        rows = [row for row in self.manifest["publicDeclarations"]
                if row.get("candidateSpecId") == "KMIPKIT-0100"]
        self.assertEqual(expected, {row["exportName"]: row["kind"] for row in rows})
        self.assertEqual(40, len(rows))
        for row in rows:
            with self.subTest(export=row["exportName"]):
                self.assertEqual(row["packageId"], "rust-kmipkit-client")
                self.assertEqual(row["declarationState"], "blocked")
                self.assertEqual(row["parityScope"], "four-language")
                self.assertEqual(row["capabilityId"], "fluent_dx_api")
                self.assertTrue(row["requirementIds"])
                self.assertTrue(all(value.startswith("KMIPKIT-0100-FR-")
                                    for value in row["requirementIds"]))
                self.assertTrue(row["blockedReason"])
                self.assertNotIn("languageMappings", row)
                self.assertNotIn("parityEvidenceIds", row)

    def test_methods_map_to_their_register_behavior(self) -> None:
        rows = {row["exportName"].rsplit("::", 1)[-1]: row
                for row in self.manifest["publicDeclarations"]
                if row.get("candidateSpecId") == "KMIPKIT-0100"
                and "RegisterBuilder<'a>::" in row["exportName"]}
        for method in ("object", "object_item", "attributes", "attribute_item"):
            self.assertIn("KMIPKIT-0100-FR-001", rows[method]["requirementIds"])
            self.assertIn("KMIPKIT-0100-FR-002", rows[method]["requirementIds"])
        for method in ("validate", "build", "build_request", "to_batch_item"):
            self.assertIn("KMIPKIT-0100-FR-003", rows[method]["requirementIds"])
        self.assertIn("KMIPKIT-0100-FR-004", rows["execute"]["requirementIds"])
        self.assertIn("KMIPKIT-0100-FR-005", rows["execute"]["requirementIds"])
        self.assertIn("KMIPKIT-0100-FR-004", rows["with_options"]["requirementIds"])

    def test_exclusions_only_cover_exact_standard_trait_paths(self) -> None:
        rows = [row for row in self.manifest["approvedExclusions"]
                if row.get("approvalSource", {}).get("path")
                == "specs/100-fluent-register/spec.md"]
        self.assertEqual(len(rows), 18)
        for row in rows:
            with self.subTest(export=row["exportName"]):
                self.assertTrue(row["exportName"].startswith("<kmipkit_client::"))
                self.assertIn("RegisterBuilder<'a> as core::", row["exportName"])
                self.assertIn("KMIPKIT-0100-FR-001", row["evidence"])
                self.assertNotIn("*", row["exportName"])


if __name__ == "__main__":
    unittest.main()
