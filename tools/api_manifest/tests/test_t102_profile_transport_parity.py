"""Require exact four-language parity for the KMIPKit profile transport model."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = ROOT / "specification" / "api" / "public-api.json"
REQUIREMENTS = [
    "KMIPKIT-0010-FR-005",
    "KMIPKIT-0010-FR-013",
    "KMIPKIT-0043-FR-004",
    "KMIPKIT-0043-FR-005",
    "KMIPKIT-0043-FR-010",
    "KMIPKIT-0043-FR-019",
    "KMIPKIT-0043-FR-021",
    "KMIPKIT-0043-FR-022",
]
RUST_TYPES = {
    "rust-kmipkit": "kmipkit::ProfileTransport",
    "rust-kmipkit-client": "kmipkit_client::ProfileTransport",
    "rust-kmipkit-protocol": "kmipkit_protocol::ProfileTransport",
}
FOREIGN_TYPES = {
    "c-kmipkit": "kmipkit_profile_transport_t",
    "java-kmipkit": "org.kmipkit.extensions.ProfileTransport",
    "python-kmipkit": "kmipkit.client.ProfileTransport",
}
VALUES = {
    "Https": (
        2,
        "KMIPKIT_PROFILE_TRANSPORT_HTTPS",
        "org.kmipkit.extensions.ProfileTransport.HTTPS",
        "kmipkit.client.ProfileTransport.HTTPS",
    ),
    "RawTtlvTls": (
        1,
        "KMIPKIT_PROFILE_TRANSPORT_RAW_TTLV_TLS",
        "org.kmipkit.extensions.ProfileTransport.RAW_TTLV_TLS",
        "kmipkit.client.ProfileTransport.RAW_TTLV_TLS",
    ),
}


class T102ProfileTransportParityTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        cls.rows = {
            (row["packageId"], row["exportName"]): row
            for row in cls.manifest["publicDeclarations"]
        }

    def test_type_and_variants_have_one_closed_four_language_representation(self) -> None:
        type_record = next(
            (record for record in self.manifest["types"] if record["id"] == "ProfileTransport"),
            None,
        )
        self.assertIsNotNone(type_record, "ProfileTransport type metadata is missing")
        self.assertEqual("closed-numeric", type_record["enumRepresentation"]["shape"])
        self.assertEqual("reject-unsupported", type_record["enumRepresentation"]["unknownValuePolicy"])
        self.assertEqual(
            {name: value[0] for name, value in VALUES.items()},
            {value["name"]: value["value"] for value in type_record["enumRepresentation"]["values"]},
        )
        self.assertEqual(FOREIGN_TYPES["c-kmipkit"], type_record["c"]["name"])
        self.assertEqual(FOREIGN_TYPES["java-kmipkit"], f"{type_record['java']['package']}.{type_record['java']['name']}")
        self.assertEqual(FOREIGN_TYPES["python-kmipkit"], f"{type_record['python']['module']}.{type_record['python']['name']}")

        expected_types = {
            (package, name) for package, name in RUST_TYPES.items()
        } | {(package, name) for package, name in FOREIGN_TYPES.items()}
        for package, export_name in expected_types:
            row = self.rows[(package, export_name)]
            with self.subTest(package=package, export_name=export_name):
                self.assertEqual("declared", row["declarationState"])
                self.assertEqual("client_profile_validation", row["capabilityId"])
                self.assertEqual(REQUIREMENTS, row["requirementIds"])
                self.assertNotIn("candidateSpecId", row)
                self.assertNotIn("blockedReason", row)
                self.assertEqual(
                    {"rust", "c", "java", "python"},
                    set(row["languageMappings"]),
                )

    def test_each_transport_variant_maps_to_c_java_and_python_values(self) -> None:
        for variant, (_, c_name, java_name, python_name) in VALUES.items():
            for package, rust_type in RUST_TYPES.items():
                export_name = f"{rust_type}::{variant}"
                row = self.rows[(package, export_name)]
                with self.subTest(package=package, variant=variant):
                    self.assertEqual("declared", row["declarationState"])
                    self.assertEqual(
                        {
                            "rust": {"packageId": package, "name": export_name},
                            "c": {"packageId": "c-kmipkit", "name": c_name},
                            "java": {"packageId": "java-kmipkit", "name": java_name},
                            "python": {"packageId": "python-kmipkit", "name": python_name},
                        },
                        row["languageMappings"],
                    )

            for package, export_name, language, name in (
                ("c-kmipkit", c_name, "c", c_name),
                ("java-kmipkit", java_name, "java", java_name),
                ("python-kmipkit", python_name, "python", python_name),
            ):
                row = self.rows[(package, export_name)]
                with self.subTest(package=package, variant=variant):
                    self.assertEqual("declared", row["declarationState"])
                    self.assertEqual(name, row["languageMappings"][language]["name"])


if __name__ == "__main__":
    unittest.main()
