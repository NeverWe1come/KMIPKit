"""Exact T102 mapping contract for synchronized runtime-version components."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = REPOSITORY_ROOT / "specification" / "api" / "public-api.json"
APPROVAL_SOURCE = {
    "kind": "feature-specification",
    "path": "specs/043-public-api-bindings/spec.md",
    "status": "Approved",
    "evidence": "**Status**: Approved for implementation",
}
COMPONENTS = ("major", "minor", "patch")
EXPECTED_COMPONENT_PATHS = {
    f"org.kmipkit.RuntimeMetadata.Version.{component}": component for component in COMPONENTS
} | {f"kmipkit.metadata.Version.{component}": component for component in COMPONENTS}
EXPECTED_JAVA_RECORD_EXCLUSIONS = {
    "org.kmipkit.RuntimeMetadata.Version.<init>",
    "org.kmipkit.RuntimeMetadata.Version.equals",
    "org.kmipkit.RuntimeMetadata.Version.hashCode",
    "org.kmipkit.RuntimeMetadata.Version.toString",
}


class T102RuntimeVersionComponentMappingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))

    def test_numeric_components_map_to_existing_runtime_metadata_capability(self) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("exportName") in EXPECTED_COMPONENT_PATHS
        }
        self.assertEqual(set(declarations), set(EXPECTED_COMPONENT_PATHS))

        expected_mappings: dict[str, dict[str, dict[str, str]]] = {}
        for export_name, component in EXPECTED_COMPONENT_PATHS.items():
            java_name = f"org.kmipkit.RuntimeMetadata.Version.{component}"
            python_name = f"kmipkit.metadata.Version.{component}"
            expected_mappings[export_name] = {
                "rust": {"packageId": "rust-kmipkit", "name": "kmipkit::RuntimeVersion::product_version"},
                "c": {"packageId": "c-kmipkit", "name": "kmipkit_version_get"},
                "java": {"packageId": "java-kmipkit", "name": java_name},
                "python": {"packageId": "python-kmipkit", "name": python_name},
            }

        for export_name, row in declarations.items():
            with self.subTest(export_name=export_name):
                component = EXPECTED_COMPONENT_PATHS[export_name]
                self.assertEqual(row["capabilityId"], "ffi_runtime_metadata")
                self.assertEqual(row["declarationState"], "declared")
                self.assertEqual(row["parityScope"], "four-language")
                self.assertNotIn("candidateSpecId", row)
                self.assertEqual(
                    set(row["requirementIds"]),
                    {"KMIPKIT-0043-FR-007", "KMIPKIT-0043-FR-010", "KMIPKIT-0043-FR-021"},
                )
                self.assertEqual(row["approvalSource"], APPROVAL_SOURCE)
                self.assertEqual(row["languageMappings"], expected_mappings[export_name])

        runtime_capability = next(
            row for row in self.manifest["capabilities"] if row.get("capabilityId") == "ffi_runtime_metadata"
        )
        for component in COMPONENTS:
            with self.subTest(component=component):
                self.assertIn(
                    f"org.kmipkit.RuntimeMetadata.Version.{component}",
                    runtime_capability["mappings"]["java"],
                )
                self.assertIn(
                    f"kmipkit.metadata.Version.{component}",
                    runtime_capability["mappings"]["python"],
                )
                self.assertIn(
                    "kmipkit::RuntimeVersion::product_version", runtime_capability["mappings"]["rust"]
                )
                self.assertIn("kmipkit_version_get", runtime_capability["mappings"]["c"])

    def test_java_record_local_value_methods_are_exact_exclusions(self) -> None:
        declarations = {
            row["exportName"]
            for row in self.manifest["publicDeclarations"]
            if row.get("exportName") in EXPECTED_JAVA_RECORD_EXCLUSIONS
        }
        exclusions = {
            row["exportName"]: row
            for row in self.manifest.get("approvedExclusions", [])
            if row.get("packageId") == "java-kmipkit"
            and row.get("exportName", "").startswith("org.kmipkit.RuntimeMetadata.Version.")
        }
        self.assertEqual(declarations, set())
        self.assertEqual(set(exclusions), EXPECTED_JAVA_RECORD_EXCLUSIONS)
        self.assertEqual(len(exclusions), len(EXPECTED_JAVA_RECORD_EXCLUSIONS))

        for export_name, row in exclusions.items():
            with self.subTest(export_name=export_name):
                self.assertEqual(row["approvalSource"], APPROVAL_SOURCE)
                self.assertIn("KMIPKIT-0043-FR-021", row["evidence"])
                self.assertIn("RuntimeMetadata.Version type", row["evidence"])
                self.assertIn("major/minor/patch components remain mapped", row["evidence"])
                if export_name.endswith(".<init>"):
                    self.assertIn("local runtime metadata snapshot", row["reason"])
                else:
                    self.assertIn("generated Java record", row["reason"])

        retained_paths = {
            "org.kmipkit.RuntimeMetadata.Version",
            "org.kmipkit.RuntimeMetadata.Version.abiVersion",
            "org.kmipkit.RuntimeMetadata.version",
            "kmipkit.metadata.Version",
            "kmipkit.metadata.Version.abi_version",
            "kmipkit.metadata.get_version",
        }
        retained = {
            row["exportName"]
            for row in self.manifest["publicDeclarations"]
            if row.get("exportName") in retained_paths and row.get("declarationState") == "declared"
        }
        self.assertEqual(retained, retained_paths)

    def test_existing_cross_language_checks_cover_the_component_values(self) -> None:
        checks = {
            "crates/kmipkit/tests/runtime_metadata.rs": (
                "public_version_matches_the_synchronized_package_version",
                'assert_eq!(kmipkit::VERSION, "1.0.0");',
            ),
            "crates/kmipkit-ffi/tests/support/metadata_tests.rs": (
                "version_query_matches_the_package_version_and_stable_abi",
                "package_version.split(['-', '+']).next()",
            ),
            "bindings/java/src/test/java/org/kmipkit/RuntimeMetadataTest.java": (
                "reportsSynchronizedProductAndAbiVersions",
                "new RuntimeMetadata.Version(1, 0, 0, 1)",
            ),
            "bindings/python/tests/test_metadata.py": (
                "test_version_reports_package_and_abi_components",
                "metadata.Version(1, 0, 0, 1)",
            ),
        }
        for relative_path, (test_name, value_assertion) in checks.items():
            with self.subTest(path=relative_path):
                source = (REPOSITORY_ROOT / relative_path).read_text(encoding="utf-8")
                self.assertIn(test_name, source)
                self.assertIn(value_assertion, source)


if __name__ == "__main__":
    unittest.main()
