"""T101 contract for exact KMIPKIT-0021 declaration reconciliation."""

from __future__ import annotations

import hashlib
import json
import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
OWNER_SPEC = "specs/021-hash-mac-signature-operations/spec.md"
T101_TEST = "tools/api_manifest/tests/test_t101_hash_mac_signature_declaration_mapping.py"
BASE_CANDIDATE_COUNT = 377
BASE_CANDIDATE_SHA256 = "f612e24ee5e7caebe571b44e4764f6b57a6a0de5bd17e9aab57c9b64b725c282"
BASE_PACKAGE_COUNTS = {"rust-kmipkit-protocol": 330, "rust-kmipkit-client": 24, "rust-kmipkit": 23}
BASE_FAMILY_COUNTS = {"Hash": 71, "Mac": 75, "MacVerify": 67, "Sign": 76, "SignatureVerify": 68}
BASE_SHARED_FACADE_COUNT = 20
T101_MAPPING_COUNT = 83
T101_EXCLUSION_COUNT = 294
SUCCESS_TTLV_OPERATIONS = ("Hash", "Mac", "MacVerify", "Sign", "SignatureVerify")
SHARED_REQUIREMENTS = {
    "KMIPKIT-0043-FR-004", "KMIPKIT-0043-FR-005", "KMIPKIT-0043-FR-010",
    "KMIPKIT-0043-FR-017", "KMIPKIT-0043-FR-021", "KMIPKIT-0043-FR-022",
}
SUCCESS_TTLV_REQUIREMENTS = SHARED_REQUIREMENTS | {"KMIPKIT-0021-FR-009", "KMIPKIT-0043-FR-011"}
LANGUAGES = {"rust", "c", "java", "python"}
APPROVAL_SOURCE = {"kind": "feature-specification", "path": OWNER_SPEC, "status": "Approved"}
OPS = {
    "Hash": ("hash", "KMIPKIT-ELEM-OP-C2S-HASH", "kmipkit_client_batch_item_hash", "org.kmipkit.extensions.ClientBatchItem.hash", "kmipkit.extensions.client_batch_item_hash", ("with_data", "with_correlation_value", "with_init_indicator", "with_final_indicator")),
    "Mac": ("mac", "KMIPKIT-ELEM-OP-C2S-MAC", "kmipkit_client_batch_item_mac", "org.kmipkit.extensions.ClientBatchItem.mac", "kmipkit.extensions.client_batch_item_mac", ("with_unique_identifier", "with_cryptographic_parameters", "with_data", "with_correlation_value", "with_init_indicator", "with_final_indicator")),
    "MacVerify": ("mac_verify", "KMIPKIT-ELEM-OP-C2S-MAC-VERIFY", "kmipkit_client_batch_item_mac_verify", "org.kmipkit.extensions.ClientBatchItem.macVerify", "kmipkit.extensions.client_batch_item_mac_verify", ("with_unique_identifier", "with_cryptographic_parameters", "with_data", "with_mac_data", "with_correlation_value", "with_init_indicator", "with_final_indicator")),
    "Sign": ("sign", "KMIPKIT-ELEM-OP-C2S-SIGN", "kmipkit_client_batch_item_sign", "org.kmipkit.extensions.ClientBatchItem.sign", "kmipkit.extensions.client_batch_item_sign", ("with_unique_identifier", "with_cryptographic_parameters", "with_data", "with_digested_data", "with_correlation_value", "with_init_indicator", "with_final_indicator")),
    "SignatureVerify": ("signature_verify", "KMIPKIT-ELEM-OP-C2S-SIGNATURE-VERIFY", "kmipkit_client_batch_item_signature_verify", "org.kmipkit.extensions.ClientBatchItem.signatureVerify", "kmipkit.extensions.client_batch_item_signature_verify", ("with_unique_identifier", "with_cryptographic_parameters", "with_data", "with_digested_data", "with_signature_data", "with_correlation_value", "with_init_indicator", "with_final_indicator")),
}
REQUEST_SURFACE_REQUIREMENTS = {
    "Hash": {"KMIPKIT-0021-FR-002", "KMIPKIT-0021-FR-009", "KMIPKIT-0021-FR-011", "KMIPKIT-0021-FR-013"},
    "Mac": {"KMIPKIT-0021-FR-004", "KMIPKIT-0021-FR-005", "KMIPKIT-0021-FR-006", "KMIPKIT-0021-FR-009", "KMIPKIT-0021-FR-011", "KMIPKIT-0021-FR-013"},
    "MacVerify": {"KMIPKIT-0021-FR-005", "KMIPKIT-0021-FR-006", "KMIPKIT-0021-FR-007", "KMIPKIT-0021-FR-009", "KMIPKIT-0021-FR-011", "KMIPKIT-0021-FR-013"},
    "Sign": {"KMIPKIT-0021-FR-004", "KMIPKIT-0021-FR-005", "KMIPKIT-0021-FR-006", "KMIPKIT-0021-FR-009", "KMIPKIT-0021-FR-011", "KMIPKIT-0021-FR-013"},
    "SignatureVerify": {"KMIPKIT-0021-FR-005", "KMIPKIT-0021-FR-006", "KMIPKIT-0021-FR-007", "KMIPKIT-0021-FR-009", "KMIPKIT-0021-FR-011", "KMIPKIT-0021-FR-013"},
}
RESPONSE_SURFACE_REQUIREMENTS = {
    "Hash": {"KMIPKIT-0021-FR-003", "KMIPKIT-0021-FR-009", "KMIPKIT-0021-FR-010"},
    "Mac": {"KMIPKIT-0021-FR-001", "KMIPKIT-0021-FR-009", "KMIPKIT-0021-FR-010", "KMIPKIT-0021-FR-015"},
    "MacVerify": {"KMIPKIT-0021-FR-008", "KMIPKIT-0021-FR-009", "KMIPKIT-0021-FR-010", "KMIPKIT-0021-FR-015"},
    "Sign": {"KMIPKIT-0021-FR-001", "KMIPKIT-0021-FR-009", "KMIPKIT-0021-FR-010", "KMIPKIT-0021-FR-015"},
    "SignatureVerify": {"KMIPKIT-0021-FR-008", "KMIPKIT-0021-FR-009", "KMIPKIT-0021-FR-010", "KMIPKIT-0021-FR-015"},
}
EVIDENCE = {
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/CryptoOperationsJavaFacadeTest.java",
    "bindings/python/tests/test_hash_mac_signature_operations.py",
    "crates/kmipkit-client/tests/unit/hash_mac_signature_execution_tests.rs",
    "crates/kmipkit-ffi/tests/support/client_abi_tests.rs",
    "crates/kmipkit-protocol/tests/unit/hash_operation_tests.rs",
    "crates/kmipkit-protocol/tests/unit/mac_operation_tests.rs",
    "crates/kmipkit-protocol/tests/unit/mac_verify_operation_tests.rs",
    "crates/kmipkit-protocol/tests/unit/sign_operation_tests.rs",
    "crates/kmipkit-protocol/tests/unit/signature_verify_operation_tests.rs",
    "tests/fixtures/api/generated/public_api_parity.json",
    "tools/api_manifest/tests/test_hash_mac_signature_parity_manifest.py",
    T101_TEST,
}
SUCCESS_TTLV_DECLARATIONS = {
    (op, lang, name)
    for op, details in OPS.items()
    for lang, name in (
        ("c", f"kmipkit_client_batch_item_response_{details[0]}_ttlv"),
        ("java", f"org.kmipkit.extensions.ClientBatchItemResult.{op}Response.responseTtlv"),
        ("python", f"kmipkit.client.{op}Response.response_ttlv"),
    )
}
RESULT_TARGETS = {"c": "kmipkit_client_batch_item_response_t", "java": "org.kmipkit.extensions.ClientBatchItemResult", "python": "kmipkit.client.BatchItemResult"}
METADATA_TARGETS = {"c": "kmipkit_client_batch_item_response_metadata_v2", "java": "org.kmipkit.extensions.ClientBatchItemResult.metadataV2", "python": "kmipkit.client.BatchItemResult.metadata_v2"}


def _owner_frs(*numbers: int) -> set[str]:
    return {f"KMIPKIT-0021-FR-{number:03}" for number in numbers}


def _operation(export_name: str) -> str:
    if export_name.startswith("<"):
        export_name = export_name[1:].split(" as ", maxsplit=1)[0]
    for name in sorted(OPS, key=len, reverse=True):
        if f"::{name}" in export_name or f"::{OPS[name][0]}" in export_name:
            return name
    raise AssertionError(f"T101 identity has no operation classification: {export_name}")


def _request_member_frs(operation: str, member: str) -> set[str]:
    base = {
        "Hash": _owner_frs(2), "Mac": _owner_frs(4), "MacVerify": _owner_frs(7),
        "Sign": _owner_frs(4), "SignatureVerify": _owner_frs(7),
    }[operation]
    if member in {"type", "new", "ClientRequest", "ClientOperation"}:
        return set(REQUEST_SURFACE_REQUIREMENTS[operation])
    if member in {"response_context", "verification_response_context"}:
        requirements = _owner_frs(9, 10, 11)
        if operation in {"MacVerify", "SignatureVerify"}:
            requirements |= _owner_frs(8)
        return requirements
    if member in {"unique_identifier", "with_unique_identifier"}:
        return base | _owner_frs(5)
    if member == "with_cryptographic_parameters":
        return base | _owner_frs(6, 13)
    if member == "with_data":
        extra = _owner_frs(9, 13)
        if operation in {"Hash", "Mac", "Sign"}:
            extra |= _owner_frs(11)
        if operation in {"MacVerify", "SignatureVerify"}:
            extra |= _owner_frs(6)
        return base | extra
    if member in {"with_mac_data", "with_signature_data", "with_digested_data"}:
        return base | _owner_frs(9, 13)
    if member in {"with_correlation_value", "with_init_indicator", "with_final_indicator", "validate_multipart_shape"}:
        return base | _owner_frs(11)
    if member == "to_ttlv_payload":
        return base | _owner_frs(9)
    raise AssertionError(f"T101 request member has no exact FR set: {operation}::{member}")


def _response_member_frs(operation: str, member: str) -> set[str]:
    if member == "result":
        return _owner_frs(10)
    if member in {"response_context", "verification_response_context"}:
        return set(RESPONSE_SURFACE_REQUIREMENTS[operation]) | _owner_frs(11)
    return set(RESPONSE_SURFACE_REQUIREMENTS[operation])


def _is_shared_facade(export_name: str) -> bool:
    if "::Client::" in export_name or "::ClientResponseView::" in export_name:
        return True
    return "::ClientRequest::" in export_name and export_name.split("::")[-1] in {
        "hash", "mac", "mac_verify", "sign", "signature_verify",
    }


def _expected_owner_frs(export_name: str) -> set[str]:
    if export_name.startswith("<"):
        model = export_name[1:].split(" as ", maxsplit=1)[0].split("::")[-1]
        op = _operation(export_name)
        if model.endswith("Request"):
            return set(REQUEST_SURFACE_REQUIREMENTS[op])
        if model.endswith("Response"):
            return set(RESPONSE_SURFACE_REQUIREMENTS[op])
        return _owner_frs(12) | set(RESPONSE_SURFACE_REQUIREMENTS[op])
    op = _operation(export_name)
    if "kmipkit_protocol::" in export_name:
        if "Request" in export_name:
            suffix = export_name.split("Request", maxsplit=1)[1].removeprefix("::")
            member = suffix.split("::")[-1] if suffix else "type"
            return _request_member_frs(op, member)
        if "Response::" in export_name:
            member = export_name.split("Response::", maxsplit=1)[1]
            if member == "with_ttlv":
                return _owner_frs(9)
            return _response_member_frs(op, member)
        if "Error" in export_name:
            return _owner_frs(12) | set(RESPONSE_SURFACE_REQUIREMENTS[op])
    if "::Client::" in export_name:
        return _owner_frs(1, 10, 11, 12, 13)
    if "::ClientRequest::" in export_name or "::ClientOperation::" in export_name:
        return _request_member_frs(op, "ClientRequest")
    if "::ClientBatchOutcome::" in export_name or "::ClientResponseView::" in export_name:
        return set(RESPONSE_SURFACE_REQUIREMENTS[op])
    raise AssertionError(f"T101 identity has no exact owner requirement set: {export_name}")


def _expected_mapping_identities() -> set[tuple[str, str]]:
    expected: set[tuple[str, str]] = set()
    for op, details in OPS.items():
        request = f"kmipkit_protocol::{op}Request"
        package = "rust-kmipkit-protocol"
        expected.update({(package, request), (package, f"{request}::new"), (package, f"kmipkit_protocol::{op}Response::result"), ("rust-kmipkit-client", f"kmipkit_client::ClientResponseView::{details[0]}")})
        expected.update((package, f"{request}::{method}") for method in details[5])
    for op in ("Hash", "Mac", "Sign"):
        lower = OPS[op][0]
        for package, prefix in (("rust-kmipkit-client", "kmipkit_client"), ("rust-kmipkit", "kmipkit")):
            expected.add((package, f"{prefix}::ClientOperation::{op}"))
            expected.add((package, f"{prefix}::ClientBatchOutcome::{op}"))
        expected.update({
            ("rust-kmipkit", f"kmipkit::ClientBatchOutcome::{op}::0"),
            ("rust-kmipkit", f"kmipkit::ClientRequest::{op}"),
            ("rust-kmipkit", f"kmipkit::ClientRequest::{op}::0"),
            ("rust-kmipkit", f"kmipkit::ClientRequest::{lower}"),
        })
        if op in {"Mac", "Sign"}:
            expected.add(("rust-kmipkit-client", f"kmipkit_client::ClientRequest::{lower}"))
    expected.update(
        ("rust-kmipkit-protocol", f"kmipkit_protocol::{op}Response::with_ttlv")
        for op in SUCCESS_TTLV_OPERATIONS
    )
    return expected


def _expected_targets(export_name: str) -> tuple[str, dict[str, str]]:
    op = _operation(export_name)
    details = OPS[op]
    if export_name.endswith("Response::with_ttlv"):
        return details[1], {
            "rust": export_name,
            "c": f"kmipkit_client_batch_item_response_{details[0]}_ttlv",
            "java": f"org.kmipkit.extensions.ClientBatchItemResult.{op}Response.responseTtlv",
            "python": f"kmipkit.client.{op}Response.response_ttlv",
        }
    if "Response::result" in export_name:
        return details[1], {"rust": export_name, **METADATA_TARGETS}
    if "ClientBatchOutcome::" in export_name or "ClientResponseView::" in export_name:
        return details[1], {"rust": export_name, **RESULT_TARGETS}
    return details[1], {"rust": export_name, "c": details[2], "java": details[3], "python": details[4]}


class T101HashMacSignatureDeclarationMappingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))

    def _mapped(self) -> list[dict[str, object]]:
        expected = _expected_mapping_identities()
        return [
            r for r in self.manifest["publicDeclarations"]
            if r.get("approvalSource", {}).get("path") == OWNER_SPEC
            and (r["packageId"], r["exportName"]) in expected
        ]

    def _excluded(self) -> list[dict[str, object]]:
        return [r for r in self.manifest["approvedExclusions"] if r.get("approvalSource", {}).get("path") == OWNER_SPEC and "T101" in r.get("evidence", "")]

    def test_baseline_identity_fingerprint_is_pinned(self) -> None:
        candidates = [r for r in self.manifest["publicDeclarations"] if r.get("candidateSpecId") == "KMIPKIT-0021"]
        rows = candidates or (self._mapped() + self._excluded())
        identities = sorted(f"{r['packageId']}|{r['exportName']}" for r in rows)
        self.assertEqual(BASE_CANDIDATE_COUNT, len(identities))
        self.assertEqual(BASE_CANDIDATE_SHA256, hashlib.sha256("\n".join(identities).encode()).hexdigest())
        self.assertEqual(BASE_PACKAGE_COUNTS, {
            package: sum(row["packageId"] == package for row in rows)
            for package in BASE_PACKAGE_COUNTS
        })
        self.assertEqual(BASE_SHARED_FACADE_COUNT, sum(_is_shared_facade(row["exportName"]) for row in rows))
        operation_rows = [row for row in rows if not _is_shared_facade(row["exportName"])]
        self.assertEqual(BASE_CANDIDATE_COUNT - BASE_SHARED_FACADE_COUNT, len(operation_rows))
        self.assertEqual(BASE_FAMILY_COUNTS, {
            op: sum(_operation(row["exportName"]) == op for row in operation_rows)
            for op in OPS
        })

    def test_exact_83_294_partition_preserves_all_baseline_identities(self) -> None:
        mapped, excluded = self._mapped(), self._excluded()
        unresolved = [r for r in self.manifest["publicDeclarations"] if r.get("candidateSpecId") == "KMIPKIT-0021"]
        self.assertEqual(T101_MAPPING_COUNT, len(mapped))
        self.assertEqual(T101_EXCLUSION_COUNT, len(excluded))
        self.assertEqual(_expected_mapping_identities(), {(r["packageId"], r["exportName"]) for r in mapped})
        self.assertEqual([], unresolved)
        all_rows = mapped + excluded
        identities = sorted(f"{r['packageId']}|{r['exportName']}" for r in all_rows)
        self.assertEqual(BASE_CANDIDATE_COUNT, len(identities))
        self.assertEqual(BASE_CANDIDATE_SHA256, hashlib.sha256("\n".join(identities).encode()).hexdigest())

    def test_mappings_reuse_exact_four_language_facades_and_exact_owner_requirements(self) -> None:
        self.assertEqual(T101_MAPPING_COUNT, len(self._mapped()))
        for row in self._mapped():
            with self.subTest(export=row["exportName"]):
                capability, targets = _expected_targets(row["exportName"])
                self.assertEqual(capability, row["capabilityId"])
                self.assertEqual("verified", row["declarationState"])
                self.assertEqual("four-language", row["parityScope"])
                self.assertEqual(APPROVAL_SOURCE, {k: row["approvalSource"][k] for k in APPROVAL_SOURCE})
                self.assertEqual(LANGUAGES, set(row["languageMappings"]))
                self.assertEqual(targets, {lang: row["languageMappings"][lang]["name"] for lang in LANGUAGES})
                self.assertEqual({"rust": row["packageId"], "c": "c-kmipkit", "java": "java-kmipkit", "python": "python-kmipkit"}, {lang: row["languageMappings"][lang]["packageId"] for lang in LANGUAGES})
                expected_requirements = _expected_owner_frs(row["exportName"]) | SHARED_REQUIREMENTS
                if row["exportName"].endswith("Response::with_ttlv"):
                    expected_requirements |= {"KMIPKIT-0043-FR-011"}
                self.assertEqual(expected_requirements, set(row["requirementIds"]))
                self.assertTrue(EVIDENCE <= set(row["parityEvidenceIds"]))
                for path in EVIDENCE:
                    self.assertTrue((ROOT / path).is_file(), path)

    def test_exclusions_have_exact_owner_and_shared_requirement_sets(self) -> None:
        excluded = self._excluded()
        self.assertEqual(T101_EXCLUSION_COUNT, len(excluded))
        identities = [(r["packageId"], r["exportName"]) for r in excluded]
        self.assertEqual(len(identities), len(set(identities)))
        for row in excluded:
            with self.subTest(export=row["exportName"]):
                owner_frs = _expected_owner_frs(row["exportName"])
                self.assertEqual("KMIPKIT-0021", row["candidateSpecId"])
                self.assertEqual(APPROVAL_SOURCE, {k: row["approvalSource"][k] for k in APPROVAL_SOURCE})
                self.assertEqual(owner_frs, set(re.findall(r"KMIPKIT-0021-FR-\d{3}", row["reason"])))
                self.assertEqual(owner_frs, set(re.findall(r"KMIPKIT-0021-FR-\d{3}", row["evidence"])))
                self.assertEqual(SHARED_REQUIREMENTS, set(re.findall(r"KMIPKIT-0043-FR-\d{3}", row["evidence"])))
                self.assertIn("T101", row["evidence"])

    def test_error_response_ttlv_remains_limited_to_the_approved_fr015_accessors(self) -> None:
        rows = [r for r in self.manifest["publicDeclarations"] if r["capabilityId"] == "crypto_response_ttlv"]
        expected_rust = {
            ("rust-kmipkit-client", "kmipkit_client::ClientError::response_ttlv"),
            ("rust-kmipkit-client", "kmipkit_client::ClientError::ProtocolResponse::response_ttlv"),
            ("rust-kmipkit", "kmipkit::ClientError::response_ttlv"),
            ("rust-kmipkit", "kmipkit::ClientError::ProtocolResponse::response_ttlv"),
        }
        self.assertEqual(expected_rust, {(r["packageId"], r["exportName"]) for r in rows if r["packageId"].startswith("rust-")})
        expected_frs = {"KMIPKIT-0021-FR-015", "KMIPKIT-0043-FR-005", "KMIPKIT-0043-FR-010", "KMIPKIT-0043-FR-021"}
        for row in rows:
            self.assertEqual(expected_frs, set(row["requirementIds"]))
            self.assertEqual("kmipkit_client_execute_with_error_ttlv_v1", row["languageMappings"]["c"]["name"])
            self.assertEqual("org.kmipkit.ProtocolException.responseTtlv", row["languageMappings"]["java"]["name"])
            self.assertEqual("kmipkit.errors.ClientExecutionError.response_ttlv", row["languageMappings"]["python"]["name"])

    def test_success_response_ttlv_has_five_narrow_four_language_declarations(self) -> None:
        expected_rows = set(SUCCESS_TTLV_DECLARATIONS)
        rows = [
            row for row in self.manifest["publicDeclarations"]
            if row.get("declarationId", "").startswith("T101:")
            and "response-ttlv" in row.get("declarationId", "")
        ]
        observed = set()
        for row in rows:
            declaration_id = row["declarationId"]
            language = row["packageId"].removesuffix("-kmipkit")
            identity = next(
                item for item in expected_rows
                if item[1] == language and item[2] == row["exportName"]
            )
            operation = identity[0]
            details = OPS[operation]
            observed.add(identity)
            expected_names = {
                "rust": f"kmipkit_protocol::{operation}Response::with_ttlv",
                "c": f"kmipkit_client_batch_item_response_{details[0]}_ttlv",
                "java": f"org.kmipkit.extensions.ClientBatchItemResult.{operation}Response.responseTtlv",
                "python": f"kmipkit.client.{operation}Response.response_ttlv",
            }
            expected_packages = {
                "rust": "rust-kmipkit-protocol", "c": "c-kmipkit",
                "java": "java-kmipkit", "python": "python-kmipkit",
            }
            expected_kinds = {"c": "c-symbol", "java": "java-member", "python": "field"}
            with self.subTest(declaration=declaration_id):
                self.assertEqual(f"T101:{details[0]}:response-ttlv:{language}", declaration_id)
                self.assertEqual(expected_packages[language], row["packageId"])
                self.assertEqual(expected_kinds[language], row["kind"])
                self.assertEqual(details[1], row["capabilityId"])
                self.assertEqual("verified", row["declarationState"])
                self.assertEqual("four-language", row["parityScope"])
                self.assertEqual(APPROVAL_SOURCE, {k: row["approvalSource"][k] for k in APPROVAL_SOURCE})
                self.assertEqual(SUCCESS_TTLV_REQUIREMENTS, set(row["requirementIds"]))
                self.assertEqual(LANGUAGES, set(row["languageMappings"]))
                self.assertEqual(expected_names, {key: value["name"] for key, value in row["languageMappings"].items()})
                self.assertEqual(expected_packages, {key: value["packageId"] for key, value in row["languageMappings"].items()})
                self.assertTrue(EVIDENCE <= set(row["parityEvidenceIds"]))
        self.assertEqual(expected_rows, observed)

    def test_success_response_ttlv_functions_generate_owned_c_and_cffi_accessors(self) -> None:
        functions = {row["id"]: row for row in self.manifest["functions"]}
        header = (ROOT / "bindings" / "c" / "include" / "kmipkit.h").read_text(encoding="utf-8")
        cffi = (ROOT / "bindings" / "python" / "src" / "kmipkit" / "_client_ffi.py").read_text(encoding="utf-8")
        expected = {
            "Hash": ("hash", "HashResponse"),
            "Mac": ("mac", "MacResponse"),
            "MacVerify": ("mac_verify", "MacVerifyResponse"),
            "Sign": ("sign", "SignResponse"),
            "SignatureVerify": ("signature_verify", "SignatureVerifyResponse"),
        }
        for operation, (snake_name, response_type) in expected.items():
            function_id = f"client_batch_item_response_{snake_name}_ttlv"
            symbol = f"kmipkit_{function_id}"
            with self.subTest(operation=operation):
                if function_id not in functions:
                    self.fail(f"missing T101 function record: {function_id}")
                function = functions[function_id]
                self.assertEqual(SUCCESS_TTLV_REQUIREMENTS, set(function["requirementIds"]))
                self.assertEqual(f"kmipkit_protocol::{response_type}", function["rust"]["module"])
                self.assertEqual("with_ttlv", function["rust"]["name"])
                self.assertEqual("protocol", function["rust"]["owner"])
                self.assertEqual(
                    [
                        {"name": "item", "kind": "handle", "type": "kmipkit_client_batch_item_response_t *", "ownership": "borrowed"},
                        {"name": "out_present", "kind": "output-scalar-pointer", "type": "uint8_t *", "valueType": "uint8_t", "ownership": "none"},
                        {"name": "out_response_ttlv", "kind": "output", "type": "kmipkit_ttlv_structure_t **", "ownership": "library-owned", "nullable": True},
                    ],
                    function["c"]["parameters"],
                )
                self.assertEqual("org.kmipkit.extensions", function["java"]["package"])
                self.assertEqual(
                    f"ClientBatchItemResult.{response_type}", function["java"]["class"]
                )
                self.assertEqual("responseTtlv", function["java"]["method"])
                self.assertEqual(
                    f"kmipkit.client.{response_type}.response_ttlv",
                    function["python"]["function"],
                )
                self.assertEqual(symbol, function["python"]["cffiSymbol"])
                self.assertIn(symbol, header)
                self.assertIn("kmipkit_ttlv_structure_t ** out_response_ttlv", header)
                self.assertIn(symbol, cffi)
                docs = " ".join(function["c"]["documentation"])
                self.assertIn("successful", docs)
                self.assertIn("ordered", docs)
                self.assertIn("Unknown fields", docs)
                self.assertIn("caller-owned", docs)
                self.assertIn("non-success", docs)

    def test_validity_indicator_mappings_keep_raw_unknown_values(self) -> None:
        rows = {r["exportName"]: r for r in self.manifest["publicDeclarations"]}
        expected = {
            "kmipkit_protocol::MacVerifyResponse::validity_indicator": ("kmipkit_client_batch_item_response_mac_verify", "MacVerifyResponse"),
            "kmipkit_protocol::SignatureVerifyResponse::validity_indicator": ("kmipkit_client_batch_item_response_signature_verify", "SignatureVerifyResponse"),
        }
        for export, (c_target, response_type) in expected.items():
            row = rows[export]
            with self.subTest(export=export):
                self.assertEqual("verified", row["declarationState"])
                self.assertEqual(c_target, row["languageMappings"]["c"]["name"])
                self.assertEqual(f"org.kmipkit.extensions.ClientBatchItemResult.{response_type}.validityIndicatorRaw", row["languageMappings"]["java"]["name"])
                self.assertEqual(f"kmipkit.client.{response_type}.validity_indicator_raw", row["languageMappings"]["python"]["name"])
                self.assertIn("KMIPKIT-0021-FR-008", row["requirementIds"])
                self.assertTrue(EVIDENCE <= set(row["parityEvidenceIds"]))


if __name__ == "__main__":
    unittest.main()
