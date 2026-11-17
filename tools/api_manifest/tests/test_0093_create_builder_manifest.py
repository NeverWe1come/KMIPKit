"""Require the generic Create builder to have complete four-language parity."""

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
BUILDER_METHODS = {
    "attributes",
    "build",
    "execute",
    "object_type",
    "protection_storage_masks",
    "to_batch_item",
    "validate",
    "with_asynchronous_indicator",
    "with_options",
}
CREATE_FUNCTIONS = {
    "create_builder_create",
    "create_builder_set_object_type",
    "create_builder_set_attributes",
    "create_builder_set_protection_storage_masks",
    "create_builder_set_asynchronous_indicator",
    "create_builder_validate",
    "create_builder_build_batch_item",
    "create_builder_execute_v1",
    "create_builder_execute_with_options_v1",
    "create_builder_release",
}


class FluentCreateBuilderManifestTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(
            (ROOT / "specification/api/public-api.json").read_text(encoding="utf-8")
        )

    def test_rust_builder_exports_are_mapped_to_all_bindings(self) -> None:
        modules = ("kmipkit_client", "kmipkit_client::fluent")
        expected = {
            f"{module}::{builder}"
            for module in modules
            for builder in ("CreateBuilder",)
        }
        for module in modules:
            expected.add(f"{module}::RequestBuilder<'a>::create")
            expected.update(
                f"{module}::CreateBuilder<'a>::{method}"
                for method in BUILDER_METHODS
            )
        rows = [
            row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-client"
            and row["exportName"] in expected
        ]
        self.assertEqual(expected, {row["exportName"] for row in rows})
        for row in rows:
            with self.subTest(export=row["exportName"]):
                self.assertEqual("declared", row["declarationState"])
                self.assertEqual("four-language", row["parityScope"])
                self.assertEqual("KMIPKIT-0093", row["candidateSpecId"])
                self.assertEqual(["KMIPKIT-0093-FR-011"], row["requirementIds"])
                self.assertEqual(
                    {"rust", "c", "java", "python"},
                    set(row["languageMappings"]),
                )
                self.assertNotIn("blockedReason", row)

    def test_builder_c_abi_is_declared_in_the_canonical_manifest(self) -> None:
        actual = {
            row["id"]
            for row in self.manifest["functions"]
            if row["id"].startswith("create_builder_")
        }
        self.assertEqual(CREATE_FUNCTIONS, actual)

    def test_python_builder_lifecycle_is_mapped(self) -> None:
        expected = {
            "kmipkit.fluent.CreateBuilder.__init__",
            "kmipkit.fluent.CreateBuilder.__enter__",
            "kmipkit.fluent.CreateBuilder.__exit__",
        }
        rows = [
            row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "python-kmipkit"
            and row.get("exportName") in expected
        ]
        self.assertEqual(expected, {row["exportName"] for row in rows})
        for row in rows:
            with self.subTest(export=row["exportName"]):
                self.assertEqual("fluent_create_builder_parity", row["capabilityId"])
                self.assertEqual("declared", row["declarationState"])
                self.assertEqual(
                    {"rust", "c", "java", "python"},
                    set(row["languageMappings"]),
                )


if __name__ == "__main__":
    unittest.main()
