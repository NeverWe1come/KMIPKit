"""Keep the Rust extension-registry module as a namespace-only exclusion."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

from tools.api_manifest.tests.t102_manifest_totals import (
    T102_APPROVED_EXCLUSIONS,
    T102_BLOCKED_CANDIDATES,
    T102_PUBLIC_DECLARATIONS,
)

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
PACKAGE = "rust-kmipkit-client"
NAMESPACE = "kmipkit_client::extension_registry"
OWNER_SPEC = "specs/043-public-api-bindings/spec.md"
EXPECTED_CHILD_EXPORTS = {
    "kmipkit_client::extension_registry::ClientConfiguration",
    "kmipkit_client::extension_registry::ClientExtensionRegistry",
    "kmipkit_client::extension_registry::ClientRequestMessageExtension",
    "kmipkit_client::extension_registry::ExtensionRecognition",
    "kmipkit_client::extension_registry::RegisteredExtensionValue",
    "kmipkit_client::extension_registry::ClientConfiguration::extension_registry",
    "kmipkit_client::extension_registry::ClientConfiguration::new",
    "kmipkit_client::extension_registry::client_extension_registry",
    "kmipkit_client::extension_registry::definition_at",
    "kmipkit_client::extension_registry::definition_count",
    "kmipkit_client::extension_registry::definition_for_identity",
    "kmipkit_client::extension_registry::inspect",
    "kmipkit_client::extension_registry::validate_extension_value",
    "kmipkit_client::extension_registry::client_request_message_extension",
    "kmipkit_client::extension_registry::generic_value",
    "kmipkit_client::extension_registry::is_recognized",
    "kmipkit_client::extension_registry::validated_value",
    "kmipkit_client::extension_registry::ExtensionRecognition::is_recognized",
    "kmipkit_client::extension_registry::ExtensionRecognition::validated_value",
    "kmipkit_client::extension_registry::limits",
}


class ClientExtensionRegistryNamespaceTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.declarations = {
            (row["packageId"], row["exportName"]): row
            for row in cls.manifest["publicDeclarations"]
        }
        cls.exclusions = {
            (row["packageId"], row["exportName"]): row
            for row in cls.manifest["approvedExclusions"]
        }

    def test_module_namespace_is_excluded_while_all_children_stay_inventoried(self) -> None:
        key = (PACKAGE, NAMESPACE)
        exclusion = self.exclusions[key]
        self.assertEqual("KMIPKIT-0043", exclusion["candidateSpecId"])
        self.assertEqual(OWNER_SPEC, exclusion["approvalSource"]["path"])
        self.assertEqual("Approved", exclusion["approvalSource"]["status"])
        self.assertIn("namespace container", exclusion["reason"])
        self.assertIn("KMIPKIT-0043-FR-020", exclusion["evidence"])
        self.assertIn("KMIPKIT-0043-FR-021", exclusion["evidence"])
        self.assertIn("specs/012-vendor-extension-registry/spec.md", exclusion["evidence"])
        self.assertNotIn(key, self.declarations)

        children = {
            export_name: row
            for (package_id, export_name), row in self.declarations.items()
            if package_id == PACKAGE and export_name.startswith(f"{NAMESPACE}::")
        }
        self.assertEqual(EXPECTED_CHILD_EXPORTS, set(children))
        for export_name, declaration in children.items():
            with self.subTest(export=export_name):
                self.assertEqual("vendor_extension_registry", declaration["capabilityId"])
                self.assertEqual("declared", declaration["declarationState"])
                self.assertNotIn("candidateSpecId", declaration)
                self.assertEqual(
                    {"rust", "c", "java", "python"},
                    set(declaration["languageMappings"]),
                )

        blocked = sum(
            row.get("candidateSpecId") == "KMIPKIT-0043" and row["declarationState"] == "blocked"
            for row in self.manifest["publicDeclarations"]
        )
        self.assertEqual(T102_BLOCKED_CANDIDATES, blocked)
        self.assertEqual(T102_PUBLIC_DECLARATIONS, len(self.manifest["publicDeclarations"]))
        self.assertEqual(T102_APPROVED_EXCLUSIONS, len(self.manifest["approvedExclusions"]))

        traceability = (ROOT / "specs/043-public-api-bindings/traceability.md").read_text(
            encoding="utf-8"
        )
        self.assertIn("T102 Rust extension-registry namespace container", traceability)
        for source in ("§8.3 Table 396", "§9.13 Table 418", "§7.13 Table 365"):
            with self.subTest(normative_source=source):
                self.assertIn(source, traceability)


if __name__ == "__main__":
    unittest.main()
