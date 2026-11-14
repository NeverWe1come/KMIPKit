"""Pin KMIPKIT-0108 read builders and their current cross-language status."""

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
PENDING_BINDING_EXPORTS = {
    "kmipkit_client::GetAttributeListBuilder<'a>::build_request",
    "kmipkit_client::GetAttributesBuilder<'a>::build_request",
    "kmipkit_client::fluent::GetAttributeListBuilder",
    "kmipkit_client::fluent::GetAttributeListBuilder<'a>::build_request",
    "kmipkit_client::fluent::GetAttributesBuilder",
    "kmipkit_client::fluent::GetAttributesBuilder<'a>::build_request",
}


class FluentAttributeReadManifestTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(
            (ROOT / "specification/api/public-api.json").read_text(encoding="utf-8")
        )

    def test_exact_builder_paths_are_traced_with_current_binding_status(self) -> None:
        expected: dict[str, tuple[str, str]] = {}
        for prefix in ("kmipkit_client::", "kmipkit_client::fluent::"):
            if prefix.endswith("fluent::"):
                expected[f"{prefix}GetAttributesBuilder"] = (
                    "type", "KMIPKIT-0108-FR-001"
                )
                expected[f"{prefix}GetAttributeListBuilder"] = (
                    "type", "KMIPKIT-0108-FR-002"
                )
            expected[f"{prefix}RequestBuilder<'a>::get_attributes"] = (
                "associated-item", "KMIPKIT-0108-FR-001"
            )
            expected[f"{prefix}RequestBuilder<'a>::get_attribute_list"] = (
                "associated-item", "KMIPKIT-0108-FR-002"
            )
            for builder, methods, factory_requirement in (
                (
                    "GetAttributesBuilder",
                    {
                        "unique_identifier": "KMIPKIT-0108-FR-001",
                        "attribute_reference": "KMIPKIT-0108-FR-001",
                        "execute": "KMIPKIT-0108-FR-004",
                    },
                    "KMIPKIT-0108-FR-001",
                ),
                (
                    "GetAttributeListBuilder",
                    {
                        "unique_identifier": "KMIPKIT-0108-FR-002",
                        "execute": "KMIPKIT-0108-FR-004",
                    },
                    "KMIPKIT-0108-FR-002",
                ),
            ):
                for method in (
                    "with_options",
                    "with_asynchronous_indicator",
                    "validate",
                    "build_request",
                    "build",
                    "to_batch_item",
                    *methods,
                ):
                    requirement = methods.get(method, "KMIPKIT-0108-FR-003")
                    expected[f"{prefix}{builder}<'a>::{method}"] = (
                        "associated-item", requirement
                    )

        rows = [
            row
            for row in self.manifest["publicDeclarations"]
            if row.get("candidateSpecId") == "KMIPKIT-0108"
        ]
        self.assertEqual(expected, {
            row["exportName"]: (row["kind"], row["requirementIds"][0]) for row in rows
        })
        self.assertEqual(
            PENDING_BINDING_EXPORTS,
            {
                row["exportName"]
                for row in rows
                if row["declarationState"] == "blocked"
            },
        )
        for row in rows:
            with self.subTest(export=row["exportName"]):
                self.assertEqual("rust-kmipkit-client", row["packageId"])
                self.assertEqual("fluent_dx_api", row["capabilityId"])
                self.assertEqual("four-language", row["parityScope"])
                self.assertEqual("KMIPKIT-0108", row["candidateSpecId"])
                if row["declarationState"] == "blocked":
                    self.assertIn("binding mappings remain pending", row["blockedReason"])
                    self.assertNotIn("approvalSource", row)
                    self.assertNotIn("languageMappings", row)
                else:
                    self.assertEqual("declared", row["declarationState"])
                    self.assertEqual(
                        {"c", "java", "python", "rust"},
                        set(row["languageMappings"]),
                    )
                    self.assertEqual("Approved", row["approvalSource"]["status"])

    def test_root_builder_aliases_are_declared_by_the_binding_delivery(self) -> None:
        expected = {
            "kmipkit_client::GetAttributesBuilder",
            "kmipkit_client::GetAttributeListBuilder",
        }
        rows = [
            row
            for row in self.manifest["publicDeclarations"]
            if row.get("candidateSpecId") == "KMIPKIT-0109"
            and row.get("exportName") in expected
        ]
        self.assertEqual(expected, {row["exportName"] for row in rows})
        for row in rows:
            with self.subTest(export=row["exportName"]):
                self.assertEqual("declared", row["declarationState"])
                self.assertEqual(
                    {"c", "java", "python", "rust"},
                    set(row["languageMappings"]),
                )

    def test_exclusions_cover_only_the_exact_standard_trait_paths(self) -> None:
        rows = [
            row
            for row in self.manifest["approvedExclusions"]
            if row.get("approvalSource", {}).get("path")
            == "specs/093-fluent-complete-api/spec.md"
            and (
                "GetAttributesBuilder" in row["exportName"]
                or "GetAttributeListBuilder" in row["exportName"]
            )
        ]
        self.assertEqual(36, len(rows))
        for row in rows:
            self.assertIn("KMIPKIT-0093-FR-011", row["evidence"])
            self.assertTrue(row["exportName"].startswith("<kmipkit_client::"))


if __name__ == "__main__":
    unittest.main()
