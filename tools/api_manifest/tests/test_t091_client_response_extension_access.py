"""T091 contract for lossless client-result Message Extension access."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = ROOT / "specification" / "api" / "public-api.json"

REQUIREMENTS = [
    "KMIPKIT-0006-FR-011",
    "KMIPKIT-0007-FR-010",
    "KMIPKIT-0012-FR-010",
    "KMIPKIT-0012-FR-011",
    "KMIPKIT-0043-FR-004",
    "KMIPKIT-0043-FR-005",
    "KMIPKIT-0043-FR-007",
    "KMIPKIT-0043-FR-010",
    "KMIPKIT-0043-FR-013",
    "KMIPKIT-0043-FR-021",
]
ASYNC_REQUIREMENTS = sorted(REQUIREMENTS + [
    "KMIPKIT-0009-FR-008",
    "KMIPKIT-0009-FR-010",
    "KMIPKIT-0043-FR-017",
])

APPROVAL_SOURCE = {
    "kind": "feature-specification",
    "path": "specs/007-client-execution/spec.md",
    "status": "Approved",
    "evidence": (
        "**Status**: Approved for implementation — delegated maintainer "
        "authorization, independent reviews, and merge of correction PR #39 are "
        "recorded in `approval-record.md`."
    ),
}

RUST_SURFACES = {
    "kmipkit_client::ClientBatchItemResponse::extensions": {
        "root": "kmipkit::ClientBatchItemResponse::extensions",
        "c": "kmipkit_client_batch_item_response_extension_count",
        "java": "org.kmipkit.extensions.ClientBatchItemResult.extensionCount",
        "python": "kmipkit.client.BatchItemResult.extension_count",
    },
    "kmipkit_client::ClientOperationOutcome::extensions": {
        "root": "kmipkit::ClientOperationOutcome::extensions",
        "c": "kmipkit_client_async_operation_outcome_extension_count",
        "java": "org.kmipkit.extensions.AsyncOperationOutcome.extensionCount",
        "python": "kmipkit.client.AsyncOperationOutcome.extension_count",
    },
    "kmipkit_client::ClientMessageExtension::with_ttlv": {
        "root": "kmipkit::ClientMessageExtension::with_ttlv",
        "c": "kmipkit_client_batch_item_response_extension_ttlv_at",
        "java": "org.kmipkit.extensions.ClientBatchItemResult.extensionTtlvAt",
        "python": "kmipkit.client.BatchItemResult.extension_ttlv_at",
    },
}


class T091ClientResponseExtensionAccessTests(unittest.TestCase):
    def test_response_extensions_map_to_bounded_four_language_ttlv_access(self) -> None:
        manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        declarations = {
            (row["packageId"], row["exportName"]): row
            for row in manifest["publicDeclarations"]
        }
        requirements = set(manifest["requirementIds"])
        self.assertTrue(set(REQUIREMENTS) <= requirements)

        for export_name, names in RUST_SURFACES.items():
            canonical = declarations[("rust-kmipkit-client", export_name)]
            root_export = names["root"]
            root = declarations[("rust-kmipkit", root_export)]
            for declaration in (canonical, root):
                with self.subTest(export=declaration["exportName"]):
                    self.assertEqual("declared", declaration["declarationState"])
                    self.assertNotIn("candidateSpecId", declaration)
                    self.assertEqual("client_execution", declaration["capabilityId"])
                    expected_requirements = (
                        ASYNC_REQUIREMENTS
                        if export_name == "kmipkit_client::ClientOperationOutcome::extensions"
                        else REQUIREMENTS
                    )
                    self.assertEqual(expected_requirements, declaration["requirementIds"])
                    self.assertEqual(APPROVAL_SOURCE, declaration["approvalSource"])
                    mappings = declaration["languageMappings"]
                    self.assertEqual(
                        {"packageId": declaration["packageId"], "name": declaration["exportName"]},
                        mappings["rust"],
                    )
                    for language in ("c", "java", "python"):
                        package_id = {
                            "c": "c-kmipkit",
                            "java": "java-kmipkit",
                            "python": "python-kmipkit",
                        }[language]
                        target_name = names[language]
                        self.assertEqual(
                            {"packageId": package_id, "name": target_name},
                            mappings[language],
                        )
                        target = declarations.get((package_id, target_name))
                        self.assertIsNotNone(target, f"Missing {language} API {target_name}")
                        self.assertEqual("declared", target["declarationState"])
                        self.assertEqual(expected_requirements, target["requirementIds"])
                        self.assertEqual("four-language", target["parityScope"])

    def test_async_extension_ttlv_access_maps_to_the_async_outcome_handle(self) -> None:
        manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        declarations = {
            (row["packageId"], row["exportName"]): row
            for row in manifest["publicDeclarations"]
        }
        expected = {
            "c": (
                "c-kmipkit",
                "kmipkit_client_async_operation_outcome_extension_count",
                "kmipkit_client_async_operation_outcome_extension_ttlv_at",
            ),
            "java": (
                "java-kmipkit",
                "org.kmipkit.extensions.AsyncOperationOutcome.extensionCount",
                "org.kmipkit.extensions.AsyncOperationOutcome.extensionTtlvAt",
            ),
            "python": (
                "python-kmipkit",
                "kmipkit.client.AsyncOperationOutcome.extension_count",
                "kmipkit.client.AsyncOperationOutcome.extension_ttlv_at",
            ),
        }
        for language, (package_id, count_name, ttlv_name) in expected.items():
            for export_name, target_name in (("count", count_name), ("ttlv_at", ttlv_name)):
                with self.subTest(package=package_id, accessor=export_name):
                    declaration = declarations.get((package_id, target_name))
                    self.assertIsNotNone(declaration, f"Missing async API {target_name}")
                    assert declaration is not None
                    self.assertEqual("declared", declaration["declarationState"])
                    self.assertEqual("client_execution", declaration["capabilityId"])
                    self.assertEqual(ASYNC_REQUIREMENTS, declaration["requirementIds"])
                    self.assertEqual("four-language", declaration["parityScope"])
                    self.assertEqual(
                        {
                            "rust": {
                                "packageId": "rust-kmipkit-client",
                                "name": "kmipkit_client::ClientOperationOutcome::extensions",
                            },
                            "c": {
                                "packageId": "c-kmipkit",
                                "name": expected["c"][1 if export_name == "count" else 2],
                            },
                            "java": {
                                "packageId": "java-kmipkit",
                                "name": expected["java"][1 if export_name == "count" else 2],
                            },
                            "python": {
                                "packageId": "python-kmipkit",
                                "name": expected["python"][1 if export_name == "count" else 2],
                            },
                        },
                        declaration["languageMappings"],
                    )

    def test_message_extension_type_maps_to_the_generic_ttlv_structure(self) -> None:
        manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        declarations = {
            (row["packageId"], row["exportName"]): row
            for row in manifest["publicDeclarations"]
        }
        canonical = declarations[("rust-kmipkit-client", "kmipkit_client::ClientMessageExtension")]
        root = declarations[("rust-kmipkit", "kmipkit::ClientMessageExtension")]
        for declaration in (canonical, root):
            with self.subTest(export=declaration["exportName"]):
                self.assertEqual("declared", declaration["declarationState"])
                self.assertNotIn("candidateSpecId", declaration)
                self.assertEqual("client_execution", declaration["capabilityId"])
                mappings = declaration["languageMappings"]
                self.assertEqual(
                    {"packageId": declaration["packageId"], "name": declaration["exportName"]},
                    mappings["rust"],
                )
                self.assertEqual(
                    {"packageId": "c-kmipkit", "name": "kmipkit_ttlv_structure_t"},
                    mappings["c"],
                )
                self.assertEqual(
                    {"packageId": "java-kmipkit", "name": "org.kmipkit.ttlv.TtlvStructure"},
                    mappings["java"],
                )
                self.assertEqual(
                    {"packageId": "python-kmipkit", "name": "kmipkit.ttlv.Structure"},
                    mappings["python"],
                )


if __name__ == "__main__":
    unittest.main()
