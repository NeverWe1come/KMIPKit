"""Reconcile the KMIPKIT-0097 Rust slice with KMIPKIT-0093 parity ownership."""

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
METHODS = {
    "algorithm", "length", "private_usage_mask", "public_usage_mask",
    "private_name", "public_name", "common_attribute", "private_attribute",
    "public_attribute", "with_options", "with_asynchronous_indicator",
    "build", "build_request", "to_batch_item", "execute", "validate",
    "common_attributes", "private_key_attributes", "public_key_attributes",
}
DECLARED_PARITY_REQUIREMENTS = {
    "KMIPKIT-0093-FR-004",
    "KMIPKIT-0093-FR-005",
    "KMIPKIT-0093-FR-011",
}


class FluentCreateKeyPairManifestTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(
            (ROOT / "bindings/shared/public-api.json").read_text(encoding="utf-8")
        )

    def test_exact_public_paths_separate_0093_parity_from_remaining_0097_candidates(self) -> None:
        expected = {}
        for prefix in ("kmipkit_client::", "kmipkit_client::fluent::"):
            expected[f"{prefix}CreateKeyPairBuilder"] = "type"
            expected[f"{prefix}RequestBuilder<'a>::create_key_pair"] = "associated-item"
            methods = METHODS | {"public_attribute"}
            if prefix == "kmipkit_client::":
                methods |= {"detached"}
            for method in methods:
                expected[f"{prefix}CreateKeyPairBuilder<'a>::{method}"] = "associated-item"
            for variant in ("MissingCreateKeyPairAlgorithm", "MissingCreateKeyPairLength"):
                expected[f"{prefix}FluentBuildError::{variant}"] = "variant"

        rows = [
            row for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-client"
            and row.get("exportName") in expected
        ]
        self.assertEqual(expected, {row["exportName"]: row["kind"] for row in rows})
        self.assertEqual(len(expected), len(rows))

        blocked_paths = set()
        for prefix in ("kmipkit_client::", "kmipkit_client::fluent::"):
            blocked_paths.add(f"{prefix}CreateKeyPairBuilder<'a>::build_request")
            blocked_paths.add(f"{prefix}RequestBuilder<'a>::create_key_pair")
            for variant in ("MissingCreateKeyPairAlgorithm", "MissingCreateKeyPairLength"):
                blocked_paths.add(f"{prefix}FluentBuildError::{variant}")
        blocked_rows = [row for row in rows if row.get("candidateSpecId") == "KMIPKIT-0097"]
        self.assertEqual(blocked_paths, {row["exportName"] for row in blocked_rows})
        self.assertEqual(8, len(blocked_rows))
        for row in blocked_rows:
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

        parity_rows = [row for row in rows if row.get("candidateSpecId") is None]
        self.assertEqual(39, len(parity_rows))
        for row in parity_rows:
            with self.subTest(export=row["exportName"]):
                self.assertEqual("declared", row["declarationState"])
                self.assertEqual("fluent_create_key_pair_parity", row["capabilityId"])
                self.assertEqual("four-language", row["parityScope"])
                self.assertTrue(DECLARED_PARITY_REQUIREMENTS <= set(row["requirementIds"]))
                self.assertEqual({"rust", "c", "java", "python"}, set(row["languageMappings"]))

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

    def test_local_validation_and_prebuilt_sets_have_exact_requirement_owners(self) -> None:
        group_methods = {"common_attributes", "private_key_attributes", "public_key_attributes"}
        for row in self.manifest["publicDeclarations"]:
            if row.get("packageId") != "rust-kmipkit-client":
                continue
            name = row.get("exportName", "")
            if not name.endswith(tuple(f"::{method}" for method in group_methods | {"validate"})):
                continue
            if "CreateKeyPairBuilder<'a>" not in name:
                continue
            method = name.rsplit("::", 1)[-1]
            if method == "validate":
                self.assertIn("KMIPKIT-0093-FR-004", row["requirementIds"])
            elif method in group_methods:
                self.assertIn("KMIPKIT-0093-FR-005", row["requirementIds"])
