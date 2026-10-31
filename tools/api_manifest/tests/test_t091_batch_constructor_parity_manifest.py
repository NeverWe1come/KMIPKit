"""T091 contract for ClientBatch constructor parity mappings."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = ROOT / "specification" / "api" / "public-api.json"

REQUIREMENTS = [
    "KMIPKIT-0006-FR-005",
    "KMIPKIT-0043-FR-004",
    "KMIPKIT-0043-FR-005",
    "KMIPKIT-0043-FR-021",
]
APPROVAL_SOURCE = {
    "kind": "feature-specification",
    "path": "specs/006-message-batch-model/spec.md",
    "status": "Approved",
    "evidence": (
        "**Status**: Approved for implementation under delegated maintainer "
        "authorization; see [approval-record.md](approval-record.md)."
    ),
}

DECLARATIONS = {
    ("rust-kmipkit-client", "kmipkit_client::ClientBatch::new"): {
        "rust": "kmipkit_client::ClientBatch::new",
        "root": "kmipkit::ClientBatch::new",
    },
    ("rust-kmipkit-client", "kmipkit_client::ClientBatch::from_items"): {
        "rust": "kmipkit_client::ClientBatch::from_items",
        "root": "kmipkit::ClientBatch::from_items",
    },
}

LANGUAGE_OPERATIONS = {
    "c": (
        "c-kmipkit",
        "kmipkit_client_batch_create + kmipkit_client_batch_add_item",
        ["kmipkit_client_batch_create", "kmipkit_client_batch_add_item"],
    ),
    "java": (
        "java-kmipkit",
        "org.kmipkit.extensions.ClientBatch.create + "
        "org.kmipkit.extensions.ClientBatch.add",
        [
            "org.kmipkit.extensions.ClientBatch.create",
            "org.kmipkit.extensions.ClientBatch.add",
        ],
    ),
    "python": (
        "python-kmipkit",
        "kmipkit.client.Batch.create + kmipkit.client.Batch.add_item",
        ["kmipkit.client.Batch.create", "kmipkit.client.Batch.add_item"],
    ),
}


class T091BatchConstructorParityManifestTests(unittest.TestCase):
    def test_all_four_batch_constructor_declarations_map_to_existing_builders(self) -> None:
        manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        declarations = {
            (row["packageId"], row["exportName"]): row
            for row in manifest["publicDeclarations"]
        }
        requirements = set(manifest["requirementIds"])
        self.assertTrue(set(REQUIREMENTS) <= requirements)

        expected_pairs = {
            pair
            for (package_id, canonical_export), names in DECLARATIONS.items()
            for pair in (
                (package_id, canonical_export),
                ("rust-kmipkit", names["root"]),
            )
        }
        candidate_pairs = {
            (row["packageId"], row["exportName"])
            for row in manifest["publicDeclarations"]
            if row.get("candidateSpecId") == "KMIPKIT-0007"
            and (row["packageId"], row["exportName"]) in expected_pairs
        }
        self.assertEqual(expected_pairs, candidate_pairs)

        for (package_id, canonical_export), names in DECLARATIONS.items():
            canonical = declarations[(package_id, canonical_export)]
            root_export = names["root"]
            root = declarations[("rust-kmipkit", root_export)]
            for declaration in (canonical, root):
                with self.subTest(export=declaration["exportName"]):
                    self.assertEqual("declared", declaration["declarationState"])
                    self.assertNotIn("candidateSpecId", declaration)
                    self.assertEqual("client_batch_model", declaration["capabilityId"])
                    self.assertEqual(REQUIREMENTS, declaration["requirementIds"])
                    self.assertEqual(APPROVAL_SOURCE, declaration["approvalSource"])

                    mappings = declaration["languageMappings"]
                    rust_package = declaration["packageId"]
                    self.assertEqual(
                        {"packageId": rust_package, "name": declaration["exportName"]},
                        mappings["rust"],
                    )
                    for language, (foreign_package, composite_name, exports) in (
                        LANGUAGE_OPERATIONS.items()
                    ):
                        with self.subTest(export=declaration["exportName"], language=language):
                            self.assertEqual(
                                {"packageId": foreign_package, "name": composite_name},
                                mappings[language],
                            )
                            self.assertEqual(
                                exports,
                                [name.strip() for name in composite_name.split("+")],
                            )
                            for export in exports:
                                target = declarations.get((foreign_package, export))
                                self.assertIsNotNone(
                                    target,
                                    f"{language} mapping target {export} is not inventoried",
                                )
                                self.assertEqual("declared", target["declarationState"])


if __name__ == "__main__":
    unittest.main()
