"""T091 parity contract for Rust's guarded-protection convenience methods."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = ROOT / "specification" / "api" / "public-api.json"

REQUIREMENTS = {
    "KMIPKIT-0039-FR-002",
    "KMIPKIT-0039-FR-004",
    "KMIPKIT-0039-FR-009",
    "KMIPKIT-0043-FR-004",
    "KMIPKIT-0043-FR-005",
    "KMIPKIT-0043-FR-010",
    "KMIPKIT-0043-FR-017",
    "KMIPKIT-0043-FR-021",
    "KMIPKIT-REQ-SPEC-6.1.23-001-001",
    "KMIPKIT-REQ-SPEC-6.1.23-001-002",
    "KMIPKIT-REQ-SPEC-6.1.23-002-001",
    "KMIPKIT-REQ-SPEC-6.1.23-002-002",
}

APPROVAL_SOURCE = {
    "kind": "feature-specification",
    "path": "specs/043-public-api-bindings/spec.md",
    "status": "Approved",
    "evidence": "**Status**: Approved for implementation",
}

SURFACES = {
    "execute_guarded_protection": {
        "c": "kmipkit_client_execute_guarded_protection_item",
        "java": "org.kmipkit.extensions.Client.executeGuardedProtectionItem",
        "python": "kmipkit.client.Client.execute_guarded_protection_item",
    },
    "execute_guarded_protection_with_asynchronous_indicator": {
        "c": "kmipkit_client_execute_guarded_protection_item_with_asynchronous_indicator",
        "java": "org.kmipkit.extensions.Client.executeGuardedProtectionItemWithAsynchronousIndicator",
        "python": "kmipkit.client.Client.execute_guarded_protection_item_with_asynchronous_indicator",
    },
}


class T091GuardedProtectionMappingTests(unittest.TestCase):
    def test_guarded_protection_methods_map_to_the_existing_item_facades(self) -> None:
        manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        declarations = {
            (row["packageId"], row["exportName"]): row
            for row in manifest["publicDeclarations"]
        }

        for suffix, targets in SURFACES.items():
            rust_names = (
                ("rust-kmipkit-client", f"kmipkit_client::Client::{suffix}"),
                ("rust-kmipkit", f"kmipkit::Client::{suffix}"),
            )
            for package_id, rust_name in rust_names:
                declaration = declarations[(package_id, rust_name)]
                with self.subTest(export=rust_name):
                    self.assertEqual("declared", declaration["declarationState"])
                    self.assertNotIn("candidateSpecId", declaration)
                    self.assertEqual("usage_allocation_guard", declaration["capabilityId"])
                    self.assertEqual(REQUIREMENTS, set(declaration["requirementIds"]))
                    self.assertEqual(APPROVAL_SOURCE, declaration["approvalSource"])
                    mappings = declaration["languageMappings"]
                    self.assertEqual(
                        {"packageId": package_id, "name": rust_name}, mappings["rust"]
                    )
                    for language, package in (
                        ("c", "c-kmipkit"),
                        ("java", "java-kmipkit"),
                        ("python", "python-kmipkit"),
                    ):
                        target_name = targets[language]
                        self.assertEqual(
                            {"packageId": package, "name": target_name},
                            mappings[language],
                        )
                        target = declarations.get((package, target_name))
                        self.assertIsNotNone(target, f"Missing {language} API {target_name}")
                        self.assertEqual("declared", target["declarationState"])


if __name__ == "__main__":
    unittest.main()
