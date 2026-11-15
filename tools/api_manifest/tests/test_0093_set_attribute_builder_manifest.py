"""Pin the Rust Set Attribute builder paths until binding parity is complete."""

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
BUILDERS = {
    "SetAttributeBuilder": {
        "attribute",
        "attribute_item",
        "build",
        "build_request",
        "execute",
        "to_batch_item",
        "unique_identifier",
        "validate",
        "vendor_attribute",
        "with_asynchronous_indicator",
        "with_options",
    }
}


class FluentSetAttributeBuilderManifestTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(
            (ROOT / "specification/api/public-api.json").read_text(encoding="utf-8")
        )

    def test_every_public_rust_path_is_traced_and_binding_gap_is_explicit(self) -> None:
        expected: dict[str, str] = {}
        for prefix in ("kmipkit_client::", "kmipkit_client::fluent::"):
            for builder, methods in BUILDERS.items():
                expected[f"{prefix}{builder}"] = "type"
                for method in methods:
                    expected[f"{prefix}{builder}<'a>::{method}"] = "associated-item"
            expected[f"{prefix}RequestBuilder<'a>::set_attribute"] = "associated-item"

        rows = [
            row
            for row in self.manifest["publicDeclarations"]
            if row.get("candidateSpecId") == "KMIPKIT-0093"
            and "SetAttributeBuilder" in row["exportName"]
            or row.get("candidateSpecId") == "KMIPKIT-0093"
            and "RequestBuilder<'a>::set_attribute" in row["exportName"]
        ]
        self.assertEqual(expected, {row["exportName"]: row["kind"] for row in rows})
        for row in rows:
            with self.subTest(export=row["exportName"]):
                self.assertEqual("rust-kmipkit-client", row["packageId"])
                self.assertEqual("fluent_dx_api", row["capabilityId"])
                self.assertEqual("four-language", row["parityScope"])
                self.assertEqual(["KMIPKIT-0093-FR-004"], row["requirementIds"])
                self.assertEqual("blocked", row["declarationState"])
                self.assertIn("binding parity", row["blockedReason"])
                self.assertNotIn("languageMappings", row)
                self.assertNotIn("approvalSource", row)


if __name__ == "__main__":
    unittest.main()
