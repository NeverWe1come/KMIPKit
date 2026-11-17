"""Keep the Rust Usage Allocation builder blocked until binding parity lands."""

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
METHODS = {
    "build",
    "execute",
    "to_batch_item",
    "unique_identifier",
    "usage_limits_count",
    "validate",
    "with_asynchronous_indicator",
    "with_options",
}


class FluentGetUsageAllocationBuilderManifestTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(
            (ROOT / "specification/api/public-api.json").read_text(encoding="utf-8")
        )
        cls.declarations = {
            row["exportName"]: row
            for row in cls.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-client"
        }

    def test_builder_exports_are_traced_and_blocked_for_four_language_parity(self) -> None:
        expected = {
            "kmipkit_client::FluentBuildError::MissingUsageLimitsCount",
            "kmipkit_client::fluent::FluentBuildError::MissingUsageLimitsCount",
            "kmipkit_client::GetUsageAllocationBuilder",
            "kmipkit_client::fluent::GetUsageAllocationBuilder",
            "kmipkit_client::RequestBuilder<'a>::get_usage_allocation",
            "kmipkit_client::fluent::RequestBuilder<'a>::get_usage_allocation",
        }
        for module in ("kmipkit_client", "kmipkit_client::fluent"):
            expected.update(
                f"{module}::GetUsageAllocationBuilder<'a>::{method}"
                for method in METHODS
            )

        self.assertTrue(expected <= self.declarations.keys())
        for export_name in expected:
            with self.subTest(export=export_name):
                declaration = self.declarations[export_name]
                self.assertEqual("blocked", declaration["declarationState"])
                self.assertEqual("four-language", declaration["parityScope"])
                self.assertEqual("KMIPKIT-0093", declaration["candidateSpecId"])
                self.assertEqual(["KMIPKIT-0093-FR-011"], declaration["requirementIds"])
                self.assertEqual("public_binding_surface", declaration["capabilityId"])
                self.assertNotIn("languageMappings", declaration)
                self.assertNotIn("approvalSource", declaration)


if __name__ == "__main__":
    unittest.main()
