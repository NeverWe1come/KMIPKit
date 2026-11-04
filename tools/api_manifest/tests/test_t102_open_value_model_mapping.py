"""Map open KMIP values and OperationData to the generic four-language TTLV API."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

from tools.api_manifest.tests.t102_manifest_totals import T102_BLOCKED_CANDIDATES


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
CAPABILITY_ID = "generic_ttlv_model"
OWNER = "specs/004-generic-ttlv-model/spec.md"
EVIDENCE = "tools/api_manifest/tests/test_t102_open_value_model_mapping.py"

VALUE_TYPE = {
    "c": ("c-kmipkit", "kmipkit_ttlv_value_t"),
    "java": ("java-kmipkit", "org.kmipkit.ttlv.TtlvValue"),
    "python": ("python-kmipkit", "kmipkit.ttlv.Value"),
}
ENUMERATION = {
    "c": ("c-kmipkit", "kmipkit_ttlv_value_enumeration"),
    "java": ("java-kmipkit", "org.kmipkit.ttlv.TtlvValue.Enumeration"),
    "python": ("python-kmipkit", "kmipkit.ttlv.ttlv_value_enumeration"),
}
ENUMERATION_VIEW = {
    "c": ("c-kmipkit", "kmipkit_ttlv_value_view_enumeration"),
    "java": ("java-kmipkit", "org.kmipkit.ttlv.TtlvValueView.enumeration"),
    "python": ("python-kmipkit", "kmipkit.ttlv.ttlv_value_view_enumeration"),
}
BYTE_STRING = {
    "c": ("c-kmipkit", "kmipkit_ttlv_value_byte_string"),
    "java": ("java-kmipkit", "org.kmipkit.ttlv.TtlvValue.ByteString"),
    "python": ("python-kmipkit", "kmipkit.ttlv.ttlv_value_byte_string"),
}
INTEGER = {
    "c": ("c-kmipkit", "kmipkit_ttlv_value_integer"),
    "java": ("java-kmipkit", "org.kmipkit.ttlv.TtlvValue.Integer"),
    "python": ("python-kmipkit", "kmipkit.ttlv.ttlv_value_integer"),
}

OPEN_ENUMERATIONS = (
    (
        "rust-observed-dc0032050f0305079631",
        "rust-observed-dec84f7b6a69ff689d70",
        "rust-observed-6fc3fe5988e26eeb65b5",
        "CryptographicAlgorithm",
    ),
    (
        "rust-observed-a3ec2f8705724f6d8611",
        "rust-observed-9953f80901da4e75a82c",
        "rust-observed-5ccd360172fa4fd026a7",
        "DigitalSignatureAlgorithm",
    ),
    (
        "rust-observed-0ed998cd345aaf8d76e7",
        "rust-observed-41b8b82d29e8a60e71f1",
        "rust-observed-d54f99b7a6decc829c8c",
        "HashingAlgorithm",
    ),
    (
        "rust-observed-42106f2192556b6bc3a0",
        "rust-observed-273ada782f3046b109d9",
        "rust-observed-4341ca30af106f8c7c0c",
        "KeyCompressionType",
    ),
    (
        "rust-observed-00a9ce145d9c795d04b9",
        "rust-observed-4f6c801493055987f59c",
        "rust-observed-428078fd5721e475f493",
        "KeyFormatType",
    ),
    (
        "rust-observed-a502d9b4b8e3fddc9b77",
        "rust-observed-5db066e451c1609f6c05",
        "rust-observed-ceda5910bc2cfd294b0a",
        "KeyWrapType",
    ),
)

OPERATION_DATA = (
    ("rust-observed-eb292fa06230faab20f1", "kmipkit_protocol::OperationData", "type", VALUE_TYPE),
    ("rust-observed-688bd2525df810f7cd9e", "kmipkit_protocol::OperationData::ByteString", "variant", BYTE_STRING),
    ("rust-observed-84b43744a1e852d5f6e9", "kmipkit_protocol::OperationData::ByteString::0", "field", BYTE_STRING),
    ("rust-observed-f69d67f312bdc180f244", "kmipkit_protocol::OperationData::Enumeration", "variant", ENUMERATION),
    ("rust-observed-6a7f5a7d884b58e351fc", "kmipkit_protocol::OperationData::Enumeration::0", "field", ENUMERATION),
    ("rust-observed-401464420590f6dacfdb", "kmipkit_protocol::OperationData::Integer", "variant", INTEGER),
    ("rust-observed-a824df1a7c6587a7b97f", "kmipkit_protocol::OperationData::Integer::0", "field", INTEGER),
    ("rust-observed-e2a52bdd069792196013", "kmipkit_protocol::OperationData::into_ttlv_value", "associated-item", VALUE_TYPE),
)

GENERIC_TTLV_EVIDENCE = {
    "bindings/c/include/kmipkit.h",
    "bindings/java/src/main/java/org/kmipkit/ttlv/TtlvValue.java",
    "bindings/java/src/main/java/org/kmipkit/ttlv/TtlvValueView.java",
    "bindings/python/src/kmipkit/ttlv.py",
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/AdapterApiCoverageTest.java",
    "bindings/python/tests/test_async_operation_mapping.py",
}
CRYPTO_ENUM_EVIDENCE = GENERIC_TTLV_EVIDENCE | {
    "crates/kmipkit-protocol/tests/unit/cryptographic_operation_contract_tests.rs",
}
KEY_ENUM_EVIDENCE = GENERIC_TTLV_EVIDENCE | {
    "crates/kmipkit-protocol/tests/unit/get_tests.rs",
}
OPERATION_DATA_EVIDENCE = GENERIC_TTLV_EVIDENCE | {
    "bindings/java/src/test/java/org/kmipkit/CryptoOperationsJavaFacadeTest.java",
    "bindings/python/tests/test_encrypt_decrypt_operations.py",
    "crates/kmipkit-ffi/tests/support/cryptographic_operation_abi_tests.rs",
    "crates/kmipkit-protocol/tests/unit/crypto_robustness_property_tests.rs",
    "crates/kmipkit-protocol/tests/unit/operation_data_tests.rs",
}


def expected_declarations() -> list[tuple[str, str, str, dict[str, tuple[str, str]], set[str]]]:
    declarations = []
    for type_id, from_raw_id, raw_id, name in OPEN_ENUMERATIONS:
        rust_name = f"kmipkit_protocol::{name}"
        declarations.extend(
            (
                (type_id, rust_name, "type", VALUE_TYPE, CRYPTO_ENUM_EVIDENCE if name in {"CryptographicAlgorithm", "DigitalSignatureAlgorithm", "HashingAlgorithm"} else KEY_ENUM_EVIDENCE),
                (from_raw_id, f"{rust_name}::from_raw", "associated-item", ENUMERATION, CRYPTO_ENUM_EVIDENCE if name in {"CryptographicAlgorithm", "DigitalSignatureAlgorithm", "HashingAlgorithm"} else KEY_ENUM_EVIDENCE),
                (raw_id, f"{rust_name}::raw", "associated-item", ENUMERATION_VIEW, CRYPTO_ENUM_EVIDENCE if name in {"CryptographicAlgorithm", "DigitalSignatureAlgorithm", "HashingAlgorithm"} else KEY_ENUM_EVIDENCE),
            )
        )
    declarations.extend(
        (declaration_id, export_name, kind, mapping, OPERATION_DATA_EVIDENCE)
        for declaration_id, export_name, kind, mapping in OPERATION_DATA
    )
    return declarations


class T102OpenValueModelMappingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.declarations = {
            row["declarationId"]: row for row in cls.manifest["publicDeclarations"]
        }
        cls.capability = next(
            row
            for row in cls.manifest["capabilities"]
            if row["capabilityId"] == CAPABILITY_ID
        )

    def test_value_models_map_to_exact_generic_ttlv_representations(self) -> None:
        expected_requirements = self.capability["requirementIds"]
        for declaration_id, name, kind, foreign, evidence in expected_declarations():
            with self.subTest(declaration=name):
                declaration = self.declarations[declaration_id]
                mappings = {
                    "rust": {"packageId": "rust-kmipkit-protocol", "name": name},
                    **{
                        language: {"packageId": package, "name": export}
                        for language, (package, export) in foreign.items()
                    },
                }
                self.assertEqual(name, declaration["exportName"])
                self.assertEqual(kind, declaration["kind"])
                self.assertEqual(CAPABILITY_ID, declaration["capabilityId"])
                self.assertEqual(expected_requirements, declaration["requirementIds"])
                self.assertEqual("verified", declaration["declarationState"])
                self.assertEqual("four-language", declaration["parityScope"])
                self.assertEqual(OWNER, declaration["approvalSource"]["path"])
                self.assertEqual("Approved", declaration["approvalSource"]["status"])
                self.assertEqual(mappings, declaration["languageMappings"])
                self.assertTrue(evidence | {EVIDENCE} <= set(declaration["parityEvidenceIds"]))
                self.assertNotIn("candidateSpecId", declaration)
                self.assertNotIn("blockedReason", declaration)
                self.assertIn(name, self.capability["mappings"]["rust"])
                for language, (_, export) in foreign.items():
                    self.assertIn(export, self.capability["mappings"][language])

        blocked = sum(
            row.get("declarationState") == "blocked"
            and row.get("candidateSpecId") == "KMIPKIT-0043"
            for row in self.manifest["publicDeclarations"]
        )
        self.assertEqual(T102_BLOCKED_CANDIDATES, blocked)

    def test_normative_sources_and_scalar_constructors_cover_the_mapping(self) -> None:
        oasis = (ROOT / "specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html").read_bytes()
        for section in (
            "7.9 Data",
            "11.12 Cryptographic Algorithm",
            "11.16 Digital Signature Algorithm",
            "11.21 Hashing Algorithm Enumeration",
            "11.24 Key Compression Type Enumeration",
            "11.25 Key Format Type Enumeration",
            "11.29 Key Wrap Type Enumeration",
        ):
            self.assertIn(section.encode("ascii"), oasis)

        protocol = (ROOT / "crates/kmipkit-protocol/src/cryptographic_operation.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("An open KMIP Cryptographic Algorithm value from §11.12", protocol)
        self.assertIn("An open KMIP Digital Signature Algorithm value from §11.16", protocol)
        self.assertIn("An open KMIP Hashing Algorithm value from §11.21", protocol)
        self.assertIn(
            "cryptographic_enum_wrappers_retain_standard_extension_and_future_values",
            (ROOT / "crates/kmipkit-protocol/tests/unit/cryptographic_operation_contract_tests.rs").read_text(encoding="utf-8"),
        )
        get_tests = (ROOT / "crates/kmipkit-protocol/tests/unit/get_tests.rs").read_text(
            encoding="utf-8"
        )
        for test_name in (
            "request_preserves_the_optional_key_compression_type_enumeration",
            "request_preserves_the_optional_key_format_type_enumeration",
            "request_preserves_the_optional_key_wrap_type_enumeration",
        ):
            self.assertIn(test_name, get_tests)

        data_tests = (
            ROOT / "crates/kmipkit-protocol/tests/unit/operation_data_tests.rs"
        ).read_text(encoding="utf-8")
        for fragment in (
            "operation_data_preserves_byte_string_type_and_exact_value",
            "operation_data_preserves_enumeration_type_and_exact_unknown_value",
            "operation_data_preserves_integer_type_and_exact_signed_value",
            "byte_string_data_uses_secret_bytes_and_zeroizing_ttlv_ownership",
        ):
            self.assertIn(fragment, data_tests)

        c_header = (ROOT / "bindings/c/include/kmipkit.h").read_text(encoding="utf-8")
        self.assertIn("kmipkit_ttlv_value_enumeration(uint32_t value", c_header)
        self.assertIn("kmipkit_ttlv_value_view_enumeration", c_header)
        self.assertIn("kmipkit_ttlv_value_integer(int32_t value", c_header)
        self.assertIn("kmipkit_ttlv_value_byte_string", c_header)
        java_value = (ROOT / "bindings/java/src/main/java/org/kmipkit/ttlv/TtlvValue.java").read_text(encoding="utf-8")
        self.assertIn("TtlvValue Enumeration(long value)", java_value)
        self.assertIn("TtlvValue Integer(int value)", java_value)
        self.assertIn("TtlvValue ByteString(byte[] value", java_value)
        python_ttlv = (ROOT / "bindings/python/src/kmipkit/ttlv.py").read_text(encoding="utf-8")
        self.assertIn("def ttlv_value_enumeration(value: int)", python_ttlv)
        self.assertIn("def ttlv_value_view_enumeration(view", python_ttlv)
        self.assertIn("def ttlv_value_integer(value: int)", python_ttlv)
        self.assertIn("def ttlv_value_byte_string(value: bytes", python_ttlv)


if __name__ == "__main__":
    unittest.main()
