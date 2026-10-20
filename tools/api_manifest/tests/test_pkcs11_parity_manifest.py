"""Manifest and binding regressions for the T057 KMIP 2.1 PKCS#11 mapping."""

from __future__ import annotations

import importlib.util
import contextlib
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
OWNER_SPEC = ROOT / "specs" / "026-pkcs11" / "spec.md"
OWNER_REQUIREMENTS = {
    f"KMIPKIT-0040-FR-{number:03d}" for number in range(1, 9)
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
    "crates/kmipkit-ffi/tests/c_api_coverage.rs",
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/extensions/Pkcs11MappingTest.java",
    "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java",
    "bindings/python/tests/test_pkcs11_operation.py",
}


def _public_jni_audit_module():
    audit_path = ROOT / "tools" / "api_manifest" / "audit_jni_public_exports.py"
    with patch.object(sys, "path", [str(audit_path.parent), *sys.path]):
        specification = importlib.util.spec_from_file_location(
            "audit_jni_public_exports", audit_path
        )
        if specification is None or specification.loader is None:
            raise AssertionError(f"could not load {audit_path}")
        module = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(module)
    return module


class Pkcs11ParityManifestTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        self.fixture = json.loads(PARITY_FIXTURE.read_text(encoding="utf-8"))
        self.operations = [
            row
            for row in self.manifest["operations"]
            if row["operationId"] == OPERATION_ID
        ]

    def test_pkcs11_mapping_is_verified_without_falsifying_spec_acceptance(self) -> None:
        self.assertEqual(1, len(self.operations))
        operation = self.operations[0]
        self.assertEqual("PKCS#11", operation["name"])
        self.assertEqual("6.1.37", operation["section"])
        self.assertEqual("client_1_0", operation["scope"])
        self.assertEqual("verified", operation["apiState"])
        self.assertEqual("KMIPKIT-0040", operation["featureSpecId"])
        self.assertEqual(
            {"state": "approved", "specId": "KMIPKIT-0040"},
            operation["ownerDependency"],
        )
        self.assertEqual("Approved", operation["approvalEvidence"]["status"])
        self.assertIn(
            "Approved for implementation",
            operation["approvalEvidence"]["evidence"],
        )
        self.assertEqual(
            {"rust", "c", "java", "python"}, set(operation["declarations"])
        )
        self.assertTrue(operation["parityEvidenceIds"])
        declarations = {
            row["declarationId"]: row for row in self.manifest["publicDeclarations"]
        }
        for language, declaration_ids in operation["declarations"].items():
            with self.subTest(language=language):
                self.assertTrue(declaration_ids)
                for declaration_id in declaration_ids:
                    declaration = declarations[declaration_id]
                    self.assertIn(language, declaration["languageMappings"])
                    self.assertEqual("verified", declaration["declarationState"])

        authorization = self.fixture["authorization"]
        self.assertEqual(
            "Approved for implementation — 2026-10-11", authorization["ownerStatus"]
        )
        self.assertEqual(
            {
                "kind": "direct-human-instruction",
                "date": "2026-10-20",
                "scope": (
                    "Implementation of the approved-for-implementation KMIPKIT-0040 "
                    "contract; this authorization is not formal specification acceptance."
                ),
            },
            authorization["implementationAuthorization"],
        )
        self.assertFalse(authorization["formalAcceptance"])
        self.assertEqual("verified", authorization["manifestApiState"])

        owner_spec = OWNER_SPEC.read_text(encoding="utf-8")
        self.assertIn("**Status**: Approved for implementation — 2026-10-11", owner_spec)
        self.assertIn("This approval does not start implementation", owner_spec)

    def test_public_mappings_match_the_shared_rust_c_java_python_fixture(self) -> None:
        mappings = self.fixture["implementationMappings"]
        self.assertEqual({"rust", "c", "java", "python"}, set(mappings))
        mapped_names = {name for names in mappings.values() for name in names}
        public_declarations = {
            row["exportName"]
            for row in self.manifest["publicDeclarations"]
            if row["capabilityId"] == OPERATION_ID
        }
        self.assertTrue(mapped_names <= public_declarations)
        self.assertTrue(
            {
                "kmipkit_client_batch_item_pkcs11",
                "kmipkit_client_batch_item_response_pkcs11_ttlv",
                "kmipkit_client_batch_item_response_pkcs11_interface_ttlv",
                "kmipkit_client_batch_item_response_pkcs11_function_ttlv",
                "kmipkit_client_batch_item_response_pkcs11_correlation_value",
                "kmipkit_client_batch_item_response_pkcs11_output_parameters_ttlv",
                "kmipkit_client_batch_item_response_pkcs11_return_code_ttlv",
            }
            <= public_declarations
        )
        self.assertTrue(self.fixture["publicNativeExports"]["javaJni"])

        expected_sources = {
            "rust": ("pkcs11.rs", "pub struct Pkcs11Request"),
            "c": ("client_abi_consumer.c", "kmipkit_client_batch_item_pkcs11"),
            "java": ("ClientBatchItem.java", "NativeExtensionRegistry"),
            "python": ("extensions.py", "client_batch_item_pkcs11"),
        }
        for language, (source_name, token) in expected_sources.items():
            with self.subTest(language=language):
                sources = "\n".join(
                    (ROOT / path).read_text(encoding="utf-8")
                    for path in self.fixture["mappingSourceFiles"][language]
                )
                self.assertIn(source_name, "\n".join(self.fixture["mappingSourceFiles"][language]))
                self.assertIn(token, sources)
                for mapped_name in mappings[language]:
                    self.assertTrue(mapped_name.strip())

    def test_shared_fixture_records_authorization_and_complete_parity_evidence(self) -> None:
        self.assertTrue(PARITY_FIXTURE.is_file())
        self.assertEqual(
            "OASIS KMIP Specification v2.1 §6.1.37, Tables 273–275",
            self.fixture["normativeSource"],
        )
        self.assertEqual(
            {"rust", "c", "java", "python"}, set(self.fixture["consumerTests"])
        )
        self.assertTrue(OWNER_REQUIREMENTS <= set(self.fixture["requirementIds"]))
        self.assertTrue(API_REQUIREMENTS <= set(self.fixture["requirementIds"]))
        self.assertTrue(PARITY_EVIDENCE <= set(self.fixture["parityEvidence"]))
        for evidence in PARITY_EVIDENCE:
            self.assertTrue((ROOT / evidence).is_file(), evidence)
        self.assertNotIn("profileSupport", self.fixture["expectedOutput"])
        self.assertEqual(
            [
                "0x0042015A",
                "0x004200D6",
                "0x0042015C",
                "0x0042015D",
                "0x00541234",
            ],
            self.fixture["expectedOutput"]["responseTags"],
        )
        self.assertFalse(self.fixture["expectedOutput"]["automaticRetry"])
        self.assertFalse(self.fixture["expectedOutput"]["automaticFollowUp"])

    def test_default_c_abi_exports_the_public_pkcs11_symbols(self) -> None:
        native_exports = set(audit_binding_exports.build_c_binary_exports(ROOT))
        self.assertTrue(
            {
                "kmipkit_client_batch_item_pkcs11",
                "kmipkit_client_batch_item_response_pkcs11_ttlv",
            }
            <= native_exports
        )

    def test_default_jni_library_must_export_all_public_pkcs11_symbols(self) -> None:
        audit_module = _public_jni_audit_module()
        expected = set(self.fixture["publicNativeExports"]["javaJni"])
        with tempfile.TemporaryDirectory() as directory:
            binary = Path(directory) / "libkmipkit_jni.so"
            binary.touch()
            with patch.object(audit_module, "read_native_exports", return_value=expected):
                self.assertEqual(0, audit_module.audit(binary))
            unlisted_pkcs11_export = (
                "Java_org_kmipkit_NativeExtensionRegistry_"
                "clientBatchItemResponsePkcs11Candidate"
            )
            with (
                patch.object(
                    audit_module,
                    "read_native_exports",
                    return_value=expected | {unlisted_pkcs11_export},
                ),
                contextlib.redirect_stderr(io.StringIO()),
            ):
                self.assertEqual(1, audit_module.audit(binary))
            unlisted_pkcs11_export = (
                "Java_org_kmipkit_NativeExtensionRegistry_"
                "clientBatchItemResponsePkcs11Uninventoried"
            )
            with (
                patch.object(
                    audit_module,
                    "read_native_exports",
                    return_value=expected | {unlisted_pkcs11_export},
                ),
                contextlib.redirect_stderr(io.StringIO()),
            ):
                self.assertEqual(1, audit_module.audit(binary))
            approved_internal_export = (
                "Java_org_kmipkit_NativeExtensionRegistry_releaseOwnedHandle"
            )
            with patch.object(
                audit_module,
                "read_native_exports",
                return_value=expected | {approved_internal_export},
            ):
                self.assertEqual(0, audit_module.audit(binary))
            with (
                patch.object(audit_module, "read_native_exports", return_value=[]),
                contextlib.redirect_stderr(io.StringIO()),
            ):
                self.assertEqual(1, audit_module.audit(binary))

    def test_default_jni_audit_ignores_dependency_candidate_symbols(self) -> None:
        audit_module = _public_jni_audit_module()
        expected = set(self.fixture["publicNativeExports"]["javaJni"])
        dependency_candidates = {
            "_ZN13openssl_probe19candidate_cert_dirs17he65a59d48d868d1eE",
            "_ZN17kmipkit_transport8resolver8Resolver23lookup_candidates_until"
            "17h51e3a68a679a9549E",
        }
        with tempfile.TemporaryDirectory() as directory:
            binary = Path(directory) / "libkmipkit_jni.so"
            binary.touch()
            with patch.object(
                audit_module,
                "read_native_exports",
                return_value=expected | dependency_candidates,
            ):
                self.assertEqual(0, audit_module.audit(binary))

    def test_default_jni_audit_rejects_unlisted_jni_candidate_symbols(self) -> None:
        audit_module = _public_jni_audit_module()
        expected = set(self.fixture["publicNativeExports"]["javaJni"])
        jni_candidate = (
            "Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemCandidate"
        )
        with tempfile.TemporaryDirectory() as directory:
            binary = Path(directory) / "libkmipkit_jni.so"
            binary.touch()
            with (
                patch.object(
                    audit_module,
                    "read_native_exports",
                    return_value=expected | {jni_candidate},
                ),
                contextlib.redirect_stderr(io.StringIO()),
            ):
                self.assertEqual(1, audit_module.audit(binary))

    def test_rust_build_has_no_candidate_only_pkcs11_gate(self) -> None:
        metadata = subprocess.run(
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
        ffi_package = next(
            package
            for package in json.loads(metadata.stdout)["packages"]
            if package["name"] == "kmipkit-ffi"
        )
        self.assertNotIn("test-pkcs11-candidate", ffi_package["features"])
        for crate in ("kmipkit-protocol", "kmipkit-client", "kmipkit-ffi"):
            with self.subTest(crate=crate):
                build_script = ROOT / "crates" / crate / "build.rs"
                if build_script.exists():
                    text = build_script.read_text(encoding="utf-8")
                    self.assertNotIn("PKCS11_CANDIDATE", text)
                    self.assertNotIn("pkcs11-candidate", text)


if __name__ == "__main__":
    unittest.main()
