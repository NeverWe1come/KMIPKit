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

APPROVAL_SOURCE = {
    "kind": "feature-specification",
    "path": "specs/007-client-execution/spec.md",
    "status": "Approved",
    "evidence": (
        "**Status**: Approved for implementation — delegated maintainer "
        "authorization; see `approval-record.md`."
    ),
}

RUST_SURFACES = {
    "kmipkit_client::ClientBatchItemResponse::extensions": {
        "root": "kmipkit::ClientBatchItemResponse::extensions",
        "c": "kmipkit_client_batch_item_response_extension_count",
        "java": "org.kmipkit.extensions.ClientBatchItemResult.extensionCount",
        "python": "kmipkit.client.BatchItemResult.extension_count",
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
                    self.assertEqual(REQUIREMENTS, declaration["requirementIds"])
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
