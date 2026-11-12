"""Pin the journey-supported key-pair subset and remaining parity gaps."""

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
JOURNEY_METHODS = {
    "algorithm", "length", "private_usage_mask", "public_usage_mask",
    "with_options", "with_asynchronous_indicator", "build", "to_batch_item",
    "execute", "validate",
}


class FluentCreateKeyPairManifestTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(
            (ROOT / "specification/api/public-api.json").read_text(encoding="utf-8")
        )

    def test_exact_public_paths_preserve_supported_and_blocked_subsets(self) -> None:
        expected = {}
        for prefix in ("kmipkit_client::", "kmipkit_client::fluent::"):
            expected[f"{prefix}CreateKeyPairBuilder"] = "type"
            expected[f"{prefix}RequestBuilder<'a>::create_key_pair"] = "associated-item"
            for method in METHODS:
                expected[f"{prefix}CreateKeyPairBuilder<'a>::{method}"] = "associated-item"
            for variant in ("MissingCreateKeyPairAlgorithm", "MissingCreateKeyPairLength"):
                expected[f"{prefix}FluentBuildError::{variant}"] = "variant"
        rows = [row for row in self.manifest["publicDeclarations"]
                if row.get("exportName") in expected]
        self.assertEqual(expected, {row["exportName"]: row["kind"] for row in rows})
        self.assertEqual(len(rows), 46)
        for row in rows:
            with self.subTest(export=row["exportName"]):
                self.assertEqual(row["packageId"], "rust-kmipkit-client")
                self.assertEqual(row["parityScope"], "four-language")
                self.assertIn("KMIPKIT-0097-FR-005", row["requirementIds"])
                export = row["exportName"]
                supported = export.startswith("kmipkit_client::fluent::") and (
                    export.endswith(("::CreateKeyPairBuilder", "::create_key_pair"))
                    or export.rsplit("::", 1)[-1] in JOURNEY_METHODS
                )
                if supported:
                    self.assertEqual(row["declarationState"], "declared")
                    self.assertEqual(row["capabilityId"], "fluent_dx_journeys")
                    self.assertEqual(set(row["languageMappings"]), {"rust", "c", "java", "python"})
                    self.assertIn("KMIPKIT-0093-FR-011", row["requirementIds"])
                    self.assertEqual(row["approvalSource"]["path"], "specs/093-fluent-complete-api/spec.md")
                    self.assertNotIn("blockedReason", row)
                    continue
                self.assertEqual(row["candidateSpecId"], "KMIPKIT-0097")
                self.assertEqual(row["declarationState"], "blocked")
                self.assertEqual(row["capabilityId"], "fluent_dx_api")
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

    def test_local_key_pair_build_maps_to_a_local_c_constructor(self) -> None:
        for row in self.manifest["publicDeclarations"]:
            export = row["exportName"]
            if row.get("capabilityId") != "fluent_dx_journeys":
                continue
            if ("CreateKeyPairBuilder" not in export
                    or export.rsplit("::", 1)[-1].rsplit(".", 1)[-1]
                    not in {"build", "to_batch_item", "toBatchItem", "validate"}):
                continue
            with self.subTest(export=export):
                self.assertEqual(
                    row["languageMappings"]["c"]["name"],
                    "kmipkit_client_batch_item_create_key_pair",
                )

    def test_key_pair_execution_maps_to_existing_builder_signatures(self) -> None:
        functions = {row["id"]: row for row in self.manifest["functions"]}
        for function_id in ("client_create_key_pair_v1", "client_create_key_pair_with_options_v1"):
            for language in ("java", "python"):
                with self.subTest(function=function_id, language=language):
                    self.assertEqual(
                        functions[function_id][language]["parameters"],
                        [{"name": "limits", "type": "CodecLimits"}],
                    )

    def test_local_validation_and_prebuilt_sets_have_exact_requirement_owners(self) -> None:
        for row in self.manifest["publicDeclarations"]:
            if "KMIPKIT-0097-FR-005" not in row.get("requirementIds", []):
                continue
            method = row["exportName"].rsplit("::", 1)[-1]
            if method == "validate":
                self.assertIn("KMIPKIT-0097-FR-006", row["requirementIds"])
            elif method in {"common_attributes", "private_key_attributes", "public_key_attributes"}:
                self.assertIn("KMIPKIT-0097-FR-007", row["requirementIds"])
