"""Pin the Rust-only KMIPKIT-0097 surface without claiming binding parity."""

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
METHODS = {
    "algorithm", "length", "private_usage_mask", "public_usage_mask",
    "private_name", "public_name", "common_attribute", "private_attribute",
    "public_attribute", "with_options", "with_asynchronous_indicator",
    "build", "build_request", "to_batch_item", "execute",
}


class FluentCreateKeyPairManifestTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(
            (ROOT / "specification/api/public-api.json").read_text(encoding="utf-8")
        )

    def test_exact_public_paths_remain_owned_and_blocked_for_binding_parity(self) -> None:
        expected = {}
        for prefix in ("kmipkit_client::", "kmipkit_client::fluent::"):
            expected[f"{prefix}CreateKeyPairBuilder"] = "type"
            expected[f"{prefix}RequestBuilder<'a>::create_key_pair"] = "associated-item"
            for method in METHODS:
                expected[f"{prefix}CreateKeyPairBuilder<'a>::{method}"] = "associated-item"
            for variant in ("MissingCreateKeyPairAlgorithm", "MissingCreateKeyPairLength"):
                expected[f"{prefix}FluentBuildError::{variant}"] = "variant"
        rows = [row for row in self.manifest["publicDeclarations"]
                if row.get("candidateSpecId") == "KMIPKIT-0097"]
        self.assertEqual(expected, {row["exportName"]: row["kind"] for row in rows})
        self.assertEqual(len(rows), 38)
        for row in rows:
            with self.subTest(export=row["exportName"]):
                self.assertEqual(row["packageId"], "rust-kmipkit-client")
                self.assertEqual(row["declarationState"], "blocked")
                self.assertEqual(row["parityScope"], "four-language")
                self.assertEqual(row["capabilityId"], "fluent_dx_api")
                self.assertIn("KMIPKIT-0097-FR-005", row["requirementIds"])
                self.assertTrue(all(value.startswith("KMIPKIT-0097-FR-")
                                    for value in row["requirementIds"]))
                self.assertTrue(row["blockedReason"])
                self.assertNotIn("languageMappings", row)
                self.assertNotIn("parityEvidenceIds", row)

    def test_exclusions_only_cover_exact_standard_trait_paths(self) -> None:
        rows = [row for row in self.manifest["approvedExclusions"]
                if row.get("approvalSource", {}).get("path")
                == "specs/097-fluent-create-key-pair/spec.md"]
        self.assertEqual(len(rows), 18)
        for row in rows:
            with self.subTest(export=row["exportName"]):
                self.assertTrue(row["exportName"].startswith("<kmipkit_client::"))
                self.assertIn("CreateKeyPairBuilder<'a> as core::", row["exportName"])
                self.assertIn("KMIPKIT-0097-FR-005", row["evidence"])
                self.assertNotIn("*", row["exportName"])
