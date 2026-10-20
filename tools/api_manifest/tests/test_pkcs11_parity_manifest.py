"""Manifest regressions for the T057 KMIP 2.1 PKCS#11 mapping."""

from __future__ import annotations

import contextlib
import importlib.util
import io
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from tools.api_manifest import audit_binding_exports

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
PARITY_FIXTURE = ROOT / "tests" / "fixtures" / "api" / "pkcs11-parity.json"
OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-PKCS-11"
OWNER_SPEC = "specs/026-pkcs11/spec.md"
IMPLEMENTATION_LANGUAGES = {"rust", "c"}
TEST_HARNESS_LANGUAGES = {"java", "python"}
OWNER_REQUIREMENTS = {
    f"KMIPKIT-0040-FR-{number:03d}" for number in range(1, 9)
}
OASIS_ELEMENTS = {
    "KMIPKIT-ELEM-OP-C2S-PKCS-11",
    "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-26-PKCS-11-FUNCTION",
    "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-28-PKCS-11-INTERFACE",
    "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-8-CORRELATION-VALUE",
    "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-27-PKCS-11-INPUT-PARAMETERS",
    "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-29-PKCS-11-OUTPUT-PARAMETERS",
    "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-30-PKCS-11-RETURN-CODE",
}
EXPECTED_MAPPINGS = {
    "rust": {
        "kmipkit_client::Client::pkcs11",
        "kmipkit_client::ClientRequest::Pkcs11",
        "kmipkit_client::ClientBatchOutcome::Pkcs11",
        "kmipkit_protocol::Pkcs11Request",
        "kmipkit_protocol::Pkcs11Request::new",
        "kmipkit_protocol::Pkcs11Response",
        "kmipkit_protocol::Pkcs11Response::with_ttlv",
        "kmipkit_protocol::Pkcs11Response::with_ttlv_payload",
    },
    "c": {
        "kmipkit_client_batch_item_pkcs11",
        "kmipkit_client_batch_item_response_pkcs11_ttlv",
    },
}
EXPECTED_TEST_HARNESSES = {
    "java": {
        "org.kmipkit.extensions.Pkcs11NativeBridge.create",
        "org.kmipkit.extensions.Pkcs11NativeBridge.responseTtlv",
    },
    "python": {
        "test_pkcs11_operation.test_pkcs11_cffi_candidate_constructor_accepts_optional_structures_and_bytes",
        "test_pkcs11_operation.test_pkcs11_cffi_response_accessor_clears_outputs_for_invalid_handle",
    },
}
PKCS11_CANDIDATE_EXPORTS = {
    ("c-kmipkit", "c-symbol", "kmipkit_client_batch_item_pkcs11"),
    ("c-kmipkit", "c-symbol", "kmipkit_client_batch_item_response_pkcs11_ttlv"),
    ("java-kmipkit", "java-member", "org.kmipkit.extensions.ClientBatchItem.pkcs11"),
    ("java-kmipkit", "type", "org.kmipkit.extensions.ClientBatchItemResult.Pkcs11Response"),
    ("java-kmipkit", "java-member", "org.kmipkit.extensions.ClientBatchItemResult.Pkcs11Response.close"),
    ("java-kmipkit", "java-member", "org.kmipkit.extensions.ClientBatchItemResult.Pkcs11Response.responseTtlv"),
    ("java-kmipkit", "java-member", "org.kmipkit.extensions.ClientBatchItemResult.Pkcs11Response.toString"),
    ("java-kmipkit", "java-member", "org.kmipkit.extensions.ClientBatchItemResult.pkcs11Response"),
    ("python-kmipkit", "field", "kmipkit.client.BatchItemResult.pkcs11_response"),
    ("python-kmipkit", "type", "kmipkit.client.Pkcs11Response"),
    ("python-kmipkit", "field", "kmipkit.client.Pkcs11Response.__dataclass_fields__"),
    ("python-kmipkit", "field", "kmipkit.client.Pkcs11Response.__dataclass_params__"),
    ("python-kmipkit", "python-export", "kmipkit.client.Pkcs11Response.__delattr__"),
    ("python-kmipkit", "python-export", "kmipkit.client.Pkcs11Response.__eq__"),
    ("python-kmipkit", "python-export", "kmipkit.client.Pkcs11Response.__hash__"),
    ("python-kmipkit", "python-export", "kmipkit.client.Pkcs11Response.__init__"),
    ("python-kmipkit", "field", "kmipkit.client.Pkcs11Response.__match_args__"),
    ("python-kmipkit", "python-export", "kmipkit.client.Pkcs11Response.__repr__"),
    ("python-kmipkit", "python-export", "kmipkit.client.Pkcs11Response.__setattr__"),
    ("python-kmipkit", "field", "kmipkit.client.Pkcs11Response.__slots__"),
    ("python-kmipkit", "python-export", "kmipkit.client.Pkcs11Response.close"),
    ("python-kmipkit", "field", "kmipkit.client.Pkcs11Response.response_ttlv"),
    ("python-kmipkit", "python-export", "kmipkit.extensions.client_batch_item_pkcs11"),
}
API_REQUIREMENTS = {
    "KMIPKIT-0043-FR-004",
    "KMIPKIT-0043-FR-005",
    "KMIPKIT-0043-FR-007",
    "KMIPKIT-0043-FR-008",
    "KMIPKIT-0043-FR-009",
    "KMIPKIT-0043-FR-010",
    "KMIPKIT-0043-FR-011",
    "KMIPKIT-0043-FR-012",
    "KMIPKIT-0043-FR-013",
    "KMIPKIT-0043-FR-017",
    "KMIPKIT-0043-FR-018",
    "KMIPKIT-0043-FR-021",
    "KMIPKIT-0043-FR-022",
}
PARITY_EVIDENCE = {
    "tests/fixtures/api/pkcs11-parity.json",
    "crates/kmipkit-client/tests/unit/pkcs11_execution_tests.rs",
    "crates/kmipkit-ffi/tests/support/pkcs11_abi_tests.rs",
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/extensions/Pkcs11MappingTest.java",
    "bindings/python/tests/test_pkcs11_operation.py",
}


class Pkcs11ParityManifestTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        self.operations = [
            row
            for row in self.manifest["operations"]
            if row["operationId"] == OPERATION_ID
        ]
    def test_pkcs11_remains_blocked_without_formal_owner_acceptance(self) -> None:
        self.assertEqual(1, len(self.operations))
        operation = self.operations[0]
        self.assertEqual("PKCS#11", operation["name"])
        self.assertEqual("6.1.37", operation["section"])
        self.assertEqual("client_1_0", operation["scope"])
        self.assertEqual("blocked", operation["apiState"])
        self.assertEqual(
            "The KMIPKIT-0040 owner authorized implementation but has not formally accepted the specification; the operation remains blocked.",
            operation["blockedReason"],
        )
        self.assertEqual("approved", operation["ownerDependency"]["state"])
        self.assertEqual("KMIPKIT-0040", operation["ownerDependency"]["specId"])
        self.assertNotIn("featureSpecId", operation)
        self.assertNotIn("approvalEvidence", operation)
        self.assertNotIn("declarations", operation)
        self.assertNotIn("parityEvidenceIds", operation)
        self.assertIn("blockedReason", operation)
        owner_spec = (ROOT / OWNER_SPEC).read_text(encoding="utf-8")
        self.assertIn(
            "**Status**: Approved for implementation — 2026-10-11", owner_spec
        )
        self.assertIn("This approval does not start implementation", owner_spec)

        self.assertEqual(
            "**Status**: Approved for implementation — 2026-10-11",
            next(line for line in owner_spec.splitlines() if "**Status**:" in line),
        )
        self.assertIn("This approval does not start implementation", owner_spec)

    def test_candidate_bindings_are_not_public_before_formal_owner_acceptance(self) -> None:
        inventory = {
            (row["packageId"], row["kind"], row["exportName"])
            for row in self.manifest["publicDeclarations"]
            if (row["packageId"], row["kind"], row["exportName"])
            in PKCS11_CANDIDATE_EXPORTS
        }
        self.assertFalse(inventory)

    def test_candidate_native_symbols_are_in_private_harness_inventory_only(self) -> None:
        fixture = json.loads(PARITY_FIXTURE.read_text(encoding="utf-8"))
        c_symbols = {
            "kmipkit_client_batch_item_pkcs11",
            "kmipkit_client_batch_item_response_pkcs11_ttlv",
        }
        jni_symbols = {
            "Java_org_kmipkit_extensions_Pkcs11NativeBridge_createNative",
            "Java_org_kmipkit_extensions_Pkcs11NativeBridge_responseTtlvNative",
        }

        self.assertEqual(c_symbols, set(fixture["candidateImplementationMappings"]["c"]))
        self.assertEqual(jni_symbols, set(fixture["candidateOnlyNativeExports"]["javaJni"]))
        public_names = {row["exportName"] for row in self.manifest["publicDeclarations"]}
        self.assertFalse((c_symbols | jni_symbols) & public_names)

        c_consumer = (ROOT / fixture["consumerTests"]["c"]).read_text(encoding="utf-8")
        jni_bridge = (ROOT / "bindings/java/native/kmipkit_jni.cpp").read_text(
            encoding="utf-8"
        )
        for symbol in c_symbols:
            with self.subTest(symbol=symbol):
                self.assertIn(symbol, c_consumer)
        for symbol in jni_symbols:
            with self.subTest(symbol=symbol):
                self.assertIn(symbol, jni_bridge)

    def test_candidate_build_gate_is_not_a_cargo_feature(self) -> None:
        completed = subprocess.run(
            [
                "cargo",
                "metadata",
                "--no-deps",
                "--format-version",
                "1",
                "--manifest-path",
                str(ROOT / "crates" / "kmipkit-ffi" / "Cargo.toml"),
            ],
            cwd=ROOT,
            check=True,
            capture_output=True,
            text=True,
            timeout=60,
        )
        metadata = json.loads(completed.stdout)
        ffi_package = next(
            package for package in metadata["packages"] if package["name"] == "kmipkit-ffi"
        )

        self.assertNotIn("test-pkcs11-candidate", ffi_package["features"])
        for crate in ("kmipkit-protocol", "kmipkit-client", "kmipkit-ffi"):
            with self.subTest(crate=crate):
                build_script = (ROOT / "crates" / crate / "build.rs").read_text(
                    encoding="utf-8"
                )
                self.assertIn("CARGO_TARGET_DIR", build_script)
                self.assertIn("pkcs11-candidate", build_script)

    def test_default_c_abi_does_not_export_owner_gated_pkcs11_symbols(self) -> None:
        native_exports = set(audit_binding_exports.build_c_binary_exports(ROOT))
        self.assertFalse(
            {
                "kmipkit_client_batch_item_pkcs11",
                "kmipkit_client_batch_item_response_pkcs11_ttlv",
            }
            & native_exports
        )

    def test_default_jni_audit_rejects_candidate_c_abi_symbols(self) -> None:
        candidate_c_symbol = "kmipkit_client_batch_item_pkcs11"
        audit_path = Path(__file__).resolve().parents[1] / "audit_jni_candidate_exports.py"
        with patch.object(sys, "path", [str(audit_path.parent), *sys.path]):
            specification = importlib.util.spec_from_file_location(
                "audit_jni_candidate_exports", audit_path
            )
            self.assertIsNotNone(specification)
            self.assertIsNotNone(specification.loader)
            audit_module = importlib.util.module_from_spec(specification)
            specification.loader.exec_module(audit_module)
        with tempfile.TemporaryDirectory() as temporary_directory:
            binary = Path(temporary_directory) / "libkmipkit_jni.so"
            binary.touch()
            stderr = io.StringIO()
            with (
                patch.object(
                    sys,
                    "argv",
                    [
                        "audit_jni_candidate_exports.py",
                        "--mode",
                        "default",
                        "--binary",
                        str(binary),
                    ],
                ),
                patch.object(
                    audit_module,
                    "read_native_exports",
                    return_value=[candidate_c_symbol],
                ),
                contextlib.redirect_stderr(stderr),
            ):
                exit_code = audit_module.main()

        self.assertEqual(1, exit_code)
        self.assertIn(candidate_c_symbol, stderr.getvalue())

    def test_pkcs11_fixture_separates_implementations_from_test_harnesses(self) -> None:
        self.assertTrue(PARITY_FIXTURE.is_file(), "shared PKCS#11 parity fixture is missing")
        fixture = json.loads(PARITY_FIXTURE.read_text(encoding="utf-8"))
        mappings = fixture["candidateImplementationMappings"]
        harnesses = fixture["candidateTestHarnessMappings"]
        self.assertEqual(IMPLEMENTATION_LANGUAGES, set(mappings))
        self.assertEqual(TEST_HARNESS_LANGUAGES, set(harnesses))
        self.assertTrue(PARITY_EVIDENCE <= set(fixture["parityEvidence"]))
        self.assertTrue(OWNER_REQUIREMENTS <= set(fixture["requirementIds"]))
        self.assertTrue(API_REQUIREMENTS <= set(fixture["requirementIds"]))
        for evidence in PARITY_EVIDENCE:
            self.assertTrue((ROOT / evidence).is_file(), evidence)
        expected_by_language = EXPECTED_MAPPINGS | EXPECTED_TEST_HARNESSES
        actual_by_language = mappings | harnesses
        for language, expected in expected_by_language.items():
            self.assertTrue(expected <= set(actual_by_language[language]), language)
            source_files = fixture["mappingSourceFiles"][language]
            source = "\n".join(
                (ROOT / path).read_text(encoding="utf-8") for path in source_files
            )
            for token in fixture["mappingSourceTokens"][language]:
                self.assertIn(token, source, f"missing {language} mapping token: {token}")

    def test_pkcs11_shared_fixture_links_normative_source_and_all_consumers(self) -> None:
        self.assertTrue(PARITY_FIXTURE.is_file(), "shared PKCS#11 parity fixture is missing")
        fixture = json.loads(PARITY_FIXTURE.read_text(encoding="utf-8"))
        self.assertEqual("KMIPKIT-ELEM-OP-C2S-PKCS-11", fixture["capability"])
        self.assertEqual("OASIS KMIP Specification v2.1 §6.1.37, Tables 273–275", fixture["normativeSource"])
        self.assertEqual({"rust", "c", "java", "python"}, set(fixture["consumerTests"]))
        self.assertTrue(set(fixture["requirementIds"]) >= OWNER_REQUIREMENTS)
        self.assertTrue(set(fixture["oasisElementIds"]) >= OASIS_ELEMENTS)
        self.assertNotIn("profileSupport", fixture["expectedOutput"])
        self.assertFalse(fixture["expectedOutput"]["automaticRetry"])
        self.assertFalse(fixture["expectedOutput"]["automaticFollowUp"])


if __name__ == "__main__":
    unittest.main()
