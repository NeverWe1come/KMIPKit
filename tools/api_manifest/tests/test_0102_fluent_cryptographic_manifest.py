"""Pin the Rust-first KMIPKIT-0102 cryptographic builder surface."""

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
COMMON = {"build", "to_batch_item", "execute", "with_options", "with_asynchronous_indicator"}
BUILDERS = {
    "EncryptBuilder": COMMON | {
        "unique_identifier", "cryptographic_parameters", "cryptographic_parameters_structure",
        "data", "data_value", "iv_counter_nonce", "correlation_value", "init_indicator",
        "final_indicator", "authenticated_encryption_additional_data",
    },
    "DecryptBuilder": COMMON | {
        "unique_identifier", "cryptographic_parameters", "cryptographic_parameters_structure",
        "data", "data_value", "iv_counter_nonce", "correlation_value", "init_indicator",
        "final_indicator", "authenticated_encryption_additional_data", "authenticated_encryption_tag",
    },
    "SignBuilder": COMMON | {
        "unique_identifier", "cryptographic_parameters", "cryptographic_parameters_structure",
        "data", "data_value", "digested_data", "correlation_value", "init_indicator", "final_indicator",
    },
    "SignatureVerifyBuilder": COMMON | {
        "unique_identifier", "cryptographic_parameters", "cryptographic_parameters_structure",
        "data", "data_value", "digested_data", "signature_data", "correlation_value",
        "init_indicator", "final_indicator",
    },
    "MacBuilder": COMMON | {
        "unique_identifier", "cryptographic_parameters", "cryptographic_parameters_structure",
        "data", "data_value", "correlation_value", "init_indicator", "final_indicator",
    },
    "MacVerifyBuilder": COMMON | {
        "unique_identifier", "cryptographic_parameters", "cryptographic_parameters_structure",
        "data", "data_value", "mac_data", "correlation_value", "init_indicator", "final_indicator",
    },
    "HashBuilder": COMMON | {
        "cryptographic_parameters", "cryptographic_parameters_structure", "data", "data_value",
        "correlation_value", "init_indicator", "final_indicator",
    },
}
CONSTRUCTORS = {"encrypt", "decrypt", "sign", "signature_verify", "mac", "mac_verify", "hash"}


class FluentCryptographicManifestTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(
            (ROOT / "specification/api/public-api.json").read_text(encoding="utf-8")
        )

    def test_exact_root_and_module_paths_are_blocked_for_binding_parity(self) -> None:
        expected = {}
        for prefix in ("kmipkit_client::", "kmipkit_client::fluent::"):
            for builder, methods in BUILDERS.items():
                expected[f"{prefix}{builder}"] = "type"
                for method in methods:
                    expected[f"{prefix}{builder}<'a>::{method}"] = "associated-item"
            for constructor in CONSTRUCTORS:
                expected[f"{prefix}RequestBuilder<'a>::{constructor}"] = "associated-item"
            expected[f"{prefix}FluentBuildError::MissingHashCryptographicParameters"] = "variant"

        rows = [
            row for row in self.manifest["publicDeclarations"]
            if row.get("candidateSpecId") == "KMIPKIT-0102"
        ]
        self.assertEqual(expected, {row["exportName"]: row["kind"] for row in rows})
        self.assertEqual(228, len(rows))
        for row in rows:
            with self.subTest(export=row["exportName"]):
                self.assertEqual("rust-kmipkit-client", row["packageId"])
                self.assertEqual("fluent_dx_api", row["capabilityId"])
                self.assertEqual("blocked", row["declarationState"])
                self.assertEqual("four-language", row["parityScope"])
                self.assertTrue(row["blockedReason"])
                self.assertTrue(row["requirementIds"])
                self.assertTrue(all(value.startswith("KMIPKIT-0102-FR-") for value in row["requirementIds"]))
                self.assertNotIn("languageMappings", row)
                self.assertNotIn("parityEvidenceIds", row)

    def test_operation_constructors_and_execution_methods_have_traceable_owners(self) -> None:
        rows = {
            row["exportName"].rsplit("::", 1)[-1]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("candidateSpecId") == "KMIPKIT-0102"
        }
        constructor_rows = [
            row for row in self.manifest["publicDeclarations"]
            if row.get("candidateSpecId") == "KMIPKIT-0102"
            and "RequestBuilder<'a>::" in row["exportName"]
        ]
        self.assertTrue(all("KMIPKIT-0102-FR-001" in row["requirementIds"] for row in constructor_rows))
        execute_rows = [
            row for row in self.manifest["publicDeclarations"]
            if row.get("candidateSpecId") == "KMIPKIT-0102"
            and row["exportName"].endswith("Builder<'a>::execute")
        ]
        self.assertEqual(14, len(execute_rows))
        self.assertTrue(all("KMIPKIT-0102-FR-007" in row["requirementIds"] for row in execute_rows))
        self.assertIn("KMIPKIT-0102-FR-003", rows["MissingHashCryptographicParameters"]["requirementIds"])
        self.assertIn("KMIPKIT-0102-FR-004", rows["data"]["requirementIds"])


if __name__ == "__main__":
    unittest.main()
