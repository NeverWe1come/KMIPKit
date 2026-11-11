"""Pin the Python fluent cryptographic operation exports."""

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
COMMON = {
    "__init__",
    "__enter__",
    "__exit__",
    "build",
    "close",
    "correlation_value",
    "cryptographic_parameters",
    "cryptographic_parameters_structure",
    "data",
    "data_value",
    "execute",
    "final_indicator",
    "init_indicator",
    "to_batch_item",
    "validate",
    "with_options",
    "asynchronous_indicator",
}
BUILDER_FIELDS = {
    "EncryptBuilder": COMMON
    | {
        "unique_identifier",
        "iv_counter_nonce",
        "authenticated_encryption_additional_data",
    },
    "DecryptBuilder": COMMON
    | {
        "unique_identifier",
        "iv_counter_nonce",
        "authenticated_encryption_additional_data",
        "authenticated_encryption_tag",
    },
    "SignBuilder": COMMON | {"unique_identifier", "digested_data"},
    "SignatureVerifyBuilder": COMMON
    | {"unique_identifier", "digested_data", "signature_data"},
    "MacBuilder": COMMON | {"unique_identifier"},
    "MacVerifyBuilder": COMMON | {"unique_identifier", "mac_data"},
    "HashBuilder": COMMON,
}
FACTORIES = {
    "encrypt",
    "decrypt",
    "sign",
    "signature_verify",
    "mac",
    "mac_verify",
    "hash",
}
LANGUAGE_IDS = {"rust", "c", "java", "python"}


class FluentPythonCryptoManifestTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(
            (ROOT / "specification/api/public-api.json").read_text(encoding="utf-8")
        )

    def test_all_python_factories_and_builder_methods_are_declared(self) -> None:
        expected = {
            f"kmipkit.fluent.Request.{factory}": "python-export"
            for factory in FACTORIES
        }
        for builder, methods in BUILDER_FIELDS.items():
            expected[f"kmipkit.fluent.{builder}"] = "type"
            for method in methods:
                expected[f"kmipkit.fluent.{builder}.{method}"] = "python-export"

        rows = [
            row
            for row in self.manifest["publicDeclarations"]
            if row.get("candidateSpecId") == "KMIPKIT-0105"
        ]
        self.assertEqual(expected, {row["exportName"]: row["kind"] for row in rows})
        for row in rows:
            with self.subTest(export=row["exportName"]):
                self.assertEqual("python-kmipkit", row["packageId"])
                self.assertEqual("fluent_dx_api", row["capabilityId"])
                self.assertEqual("four-language", row["parityScope"])
                self.assertEqual("declared", row["declarationState"])
                self.assertEqual(LANGUAGE_IDS, set(row["languageMappings"]))
                self.assertIn("KMIPKIT-0105-FR-007", row["requirementIds"])
                self.assertEqual("Approved", row["approvalSource"]["status"])


if __name__ == "__main__":
    unittest.main()
