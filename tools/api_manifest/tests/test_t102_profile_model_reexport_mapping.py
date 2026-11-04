"""Pin the exact T102 Rust profile-model reexports and their parity mappings.

KMIP Specification v2.1 §14.1 and KMIP Profiles v2.1 §6.1 are profile
context only. These are KMIPKit API reexports of the client model, not new
OASIS protocol capabilities.
"""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = ROOT / "specification" / "api" / "public-api.json"
REEXPORT_PACKAGES = {
    "rust-kmipkit": "kmipkit",
    "rust-kmipkit-protocol": "kmipkit_protocol",
}
CLIENT_CRATE = "kmipkit_client"
MODEL_SUFFIXES = {
    "ProfileTransport": ("", "::Https", "::RawTtlvTls"),
    "ProfileObjectState": (
        "",
        "::new",
        "::with_get_usage_allocation_changed_count",
        "::with_get_usage_allocation_performed",
        "::with_original_creation_date_present",
        "::with_protection_use_started",
        "::with_usage_limits_count",
        "::with_usage_limits_present",
        "::with_usage_limits_total",
    ),
    "ProfileValidationContext": ("", "::new", "::with_object_state"),
    "ProfileValidationSelection": ("", "::new", "::profile_ids", "::validate"),
}
RESULT_SUFFIXES = (
    "",
    "::profile_ids",
    "::requirement_ids",
    "::validate_request",
    "::validate_request_with_context",
    "::validate_response",
    "::validate_response_with_context",
    "::validate_transport",
)


class T102ProfileModelReexportMappingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))

    def test_exact_root_and_protocol_profile_model_reexports_map_to_client_surfaces(self) -> None:
        expected = {
            (package, f"{crate}::{type_name}{suffix}")
            for package, crate in REEXPORT_PACKAGES.items()
            for type_name, suffixes in MODEL_SUFFIXES.items()
            for suffix in suffixes
        }
        self.assertEqual(38, len(expected))
        self.assertEqual(
            2,
            sum(name.endswith("::ProfileValidationSelection::validate") for _, name in expected),
        )
        self.assertFalse(any("ProfileValidationResult" in name for _, name in expected))

        rows = {
            (row["packageId"], row["exportName"]): row
            for row in self.manifest["publicDeclarations"]
            if (row.get("packageId"), row.get("exportName")) in expected
        }
        self.assertEqual(expected, set(rows))
        declarations = {
            (row["packageId"], row["exportName"]): row
            for row in self.manifest["publicDeclarations"]
        }

        for (package, export_name), row in rows.items():
            crate = REEXPORT_PACKAGES[package]
            suffix = export_name.removeprefix(f"{crate}::")
            client_export = f"{CLIENT_CRATE}::{suffix}"
            client_row = declarations[("rust-kmipkit-client", client_export)]
            expected_mappings = dict(client_row["languageMappings"])
            expected_mappings["rust"] = {"packageId": package, "name": export_name}
            with self.subTest(export_name=export_name):
                self.assertEqual("client_profile_validation", row["capabilityId"])
                self.assertEqual("declared", row["declarationState"])
                self.assertEqual("four-language", row["parityScope"])
                self.assertEqual(row["requirementIds"], client_row["requirementIds"])
                self.assertEqual(row["approvalSource"], client_row["approvalSource"])
                self.assertEqual(row["languageMappings"], expected_mappings)
                self.assertNotIn("candidateSpecId", row)
                self.assertNotIn("blockedReason", row)

        capability = next(
            row
            for row in self.manifest["capabilities"]
            if row["capabilityId"] == "client_profile_validation"
        )
        self.assertEqual("declared", capability["capabilityState"])
        for package, crate in REEXPORT_PACKAGES.items():
            for name in (f"{crate}::ProfileValidationResult",):
                declaration = declarations[(package, name)]
                self.assertEqual("blocked", declaration["declarationState"])
                self.assertNotIn(name, capability["mappings"]["rust"])
        self.assertEqual(
            {name for _, name in expected},
            {
                name
                for name in capability["mappings"]["rust"]
                if any(f"{crate}::" in name for crate in REEXPORT_PACKAGES.values())
                and any(f"::{model}" in name for model in MODEL_SUFFIXES)
            },
        )

    def test_profile_validation_result_has_exact_four_language_parity(self) -> None:
        expected = {
            (package, f"{crate}::ProfileValidationResult{suffix}")
            for package, crate in REEXPORT_PACKAGES.items()
            for suffix in RESULT_SUFFIXES
        }
        self.assertEqual(16, len(expected))
        declarations = {
            (row["packageId"], row["exportName"]): row
            for row in self.manifest["publicDeclarations"]
        }
        self.assertTrue(expected <= declarations.keys())

        for package, export_name in sorted(expected):
            row = declarations[(package, export_name)]
            with self.subTest(package=package, export_name=export_name):
                self.assertEqual("client_profile_validation", row["capabilityId"])
                self.assertEqual("declared", row["declarationState"])
                self.assertEqual("four-language", row["parityScope"])
                self.assertTrue(
                    {
                        "KMIPKIT-0010-FR-005",
                        "KMIPKIT-0010-FR-012",
                        "KMIPKIT-0010-FR-013",
                        "KMIPKIT-0010-FR-014",
                        "KMIPKIT-0010-FR-019",
                    } <= set(row["requirementIds"])
                )
                self.assertEqual(
                    {"rust", "c", "java", "python"},
                    set(row["languageMappings"]),
                )
                self.assertEqual(
                    {
                        "kind": "feature-specification",
                        "path": "specs/010-profile-conformance/spec.md",
                        "status": "Accepted",
                    },
                    {key: row["approvalSource"].get(key)
                     for key in ("kind", "path", "status")},
                )
                self.assertTrue(row["approvalSource"].get("evidence"))
                self.assertNotIn("candidateSpecId", row)
                self.assertNotIn("blockedReason", row)


if __name__ == "__main__":
    unittest.main()
