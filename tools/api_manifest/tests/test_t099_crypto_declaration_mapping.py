"""T099 contract for KMIPKIT-0019 Encrypt and Decrypt declarations."""

from __future__ import annotations

import hashlib
import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
OWNER_SPEC = "specs/019-cryptographic-operations/spec.md"
BASE_CANDIDATE_COUNT = 165
BASE_CANDIDATE_SHA256 = "8622dae80e3772bab0af270c14d70c6c4ef658f73a73f059c0d4a7eb9d2ba470"
OWNER_IDENTITY_COUNT = 229
OWNER_IDENTITY_SHA256 = "9ea6a5e0e3ce1c9c37859a2d9232289fbf5c02c1398b5f8d6b6b99438a3535b3"
OWNER_MAPPING_COUNT = 95
OWNER_EXCLUSION_COUNT = 134
T099_MAPPING_COUNT = 43
T099_EXCLUSION_COUNT = 122
SHARED_REQUIREMENTS = {
    "KMIPKIT-0043-FR-004",
    "KMIPKIT-0043-FR-005",
    "KMIPKIT-0043-FR-010",
    "KMIPKIT-0043-FR-017",
    "KMIPKIT-0043-FR-021",
    "KMIPKIT-0043-FR-022",
}
OWNER_REQUIREMENTS = {
    f"KMIPKIT-0019-FR-{number:03}"
    for number in (1, 2, 3, 4, 5, 6, 7, 9, 11, 12)
}
LANGUAGES = {"rust", "c", "java", "python"}
APPROVAL_SOURCE = {
    "kind": "feature-specification",
    "path": OWNER_SPEC,
    "status": "Approved",
}
PARITY_EVIDENCE = {
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/CryptoOperationsJavaFacadeTest.java",
    "bindings/python/tests/test_encrypt_decrypt_operations.py",
    "crates/kmipkit-ffi/tests/support/client_abi_tests.rs",
}
OPERATION_TARGETS = {
    "Encrypt": {
        "id": "KMIPKIT-ELEM-OP-C2S-ENCRYPT",
        "request": {
            "rust": "kmipkit_client::ClientRequest::Encrypt",
            "c": "kmipkit_client_batch_item_encrypt",
            "java": "org.kmipkit.extensions.ClientBatchItem.encrypt",
            "python": "kmipkit.extensions.client_batch_item_encrypt",
        },
        "response": {
            "rust": "kmipkit_client::ClientBatchOutcome::encrypt_response",
            "c": "kmipkit_client_batch_item_response_encrypt",
            "java": "org.kmipkit.extensions.ClientBatchItemResult.encryptResponse",
            "python": "kmipkit.client.BatchItemResult.encrypt_response",
        },
        "result": {
            "rust": "kmipkit_client::ClientBatchItemResponse::outcome",
            "c": "kmipkit_client_batch_item_response_metadata_v2",
            "java": "org.kmipkit.extensions.ClientBatchItemResult.metadataV2",
            "python": "kmipkit.client.BatchItemResult.metadata_v2",
        },
    },
    "Decrypt": {
        "id": "KMIPKIT-ELEM-OP-C2S-DECRYPT",
        "request": {
            "rust": "kmipkit_client::ClientRequest::Decrypt",
            "c": "kmipkit_client_batch_item_decrypt",
            "java": "org.kmipkit.extensions.ClientBatchItem.decrypt",
            "python": "kmipkit.extensions.client_batch_item_decrypt",
        },
        "response": {
            "rust": "kmipkit_client::ClientBatchOutcome::decrypt_response",
            "c": "kmipkit_client_batch_item_response_decrypt",
            "java": "org.kmipkit.extensions.ClientBatchItemResult.decryptResponse",
            "python": "kmipkit.client.BatchItemResult.decrypt_response",
        },
        "result": {
            "rust": "kmipkit_client::ClientBatchItemResponse::outcome",
            "c": "kmipkit_client_batch_item_response_metadata_v2",
            "java": "org.kmipkit.extensions.ClientBatchItemResult.metadataV2",
            "python": "kmipkit.client.BatchItemResult.metadata_v2",
        },
    },
}
OPERATION_NORMATIVE_REQUIREMENTS = {
    "Encrypt": {
        "KMIPKIT-REQ-SPEC-4.16-001-001",
        "KMIPKIT-REQ-SPEC-4.16-001-002",
        "KMIPKIT-REQ-SPEC-4.16-002",
        "KMIPKIT-REQ-SPEC-4.16-003",
        "KMIPKIT-REQ-SPEC-6.1.17-001-001",
        "KMIPKIT-REQ-SPEC-6.1.17-001-002",
        "KMIPKIT-REQ-SPEC-6.1.17-005-001",
        "KMIPKIT-REQ-SPEC-6.1.17-005-002",
        "KMIPKIT-REQ-SPEC-6.1.17-006",
    },
    "Decrypt": {
        "KMIPKIT-REQ-SPEC-4.16-001-001",
        "KMIPKIT-REQ-SPEC-4.16-001-002",
        "KMIPKIT-REQ-SPEC-4.16-002",
        "KMIPKIT-REQ-SPEC-4.16-003",
        "KMIPKIT-REQ-SPEC-6.1.11-001-001",
        "KMIPKIT-REQ-SPEC-6.1.11-001-002",
        "KMIPKIT-REQ-SPEC-6.1.11-004-001",
        "KMIPKIT-REQ-SPEC-6.1.11-004-002",
        "KMIPKIT-REQ-SPEC-6.1.11-005",
        "KMIPKIT-REQ-SPEC-6.1.11-006",
    },
}
RESULT_CONTAINER_TARGETS = {
    "c": "kmipkit_client_batch_item_response_t",
    "java": "org.kmipkit.extensions.ClientBatchItemResult",
    "python": "kmipkit.client.BatchItemResult",
}
REQUEST_METHODS = {
    "Encrypt": (
        "with_authenticated_encryption_additional_data",
        "with_correlation_value",
        "with_cryptographic_parameters",
        "with_final_indicator",
        "with_init_indicator",
        "with_iv_counter_nonce",
    ),
    "Decrypt": (
        "with_authenticated_encryption_additional_data",
        "with_authenticated_encryption_tag",
        "with_correlation_value",
        "with_cryptographic_parameters",
        "with_final_indicator",
        "with_init_indicator",
        "with_iv_counter_nonce",
    ),
}


def _identity_hash(rows: list[dict[str, object]]) -> str:
    identities = sorted(f"{row['packageId']}|{row['exportName']}" for row in rows)
    return hashlib.sha256("\n".join(identities).encode("utf-8")).hexdigest()


def _expected_mapped_exports() -> set[str]:
    mapped: set[str] = set()
    for operation, details in OPERATION_TARGETS.items():
        prefix = f"kmipkit_protocol::{operation}"
        mapped.update(
            {
                f"{prefix}Request",
                f"{prefix}Request::new",
                *(f"{prefix}Request::{method}" for method in REQUEST_METHODS[operation]),
                f"{prefix}Response::result",
            }
        )
        client_prefix = "kmipkit_client"
        root_prefix = "kmipkit"
        for package_prefix in (client_prefix, root_prefix):
            mapped.update(
                {
                    f"{package_prefix}::ClientBatchOutcome::{operation}",
                    f"{package_prefix}::ClientBatchOutcome::{operation}::0",
                    f"{package_prefix}::ClientOperation::{operation}",
                    f"{package_prefix}::ClientRequest::{operation}::0",
                    f"{package_prefix}::ClientRequest::{operation.lower()}",
                }
            )
        mapped.add(f"{root_prefix}::ClientRequest::{operation}")
        if operation == "Encrypt":
            mapped.add("kmipkit_client::ClientResponseView::encrypt")
        else:
            mapped.add("kmipkit_client::ClientResponseView::decrypt")
    return mapped


def _expected_targets(export_name: str) -> tuple[str, dict[str, str], str]:
    operation = "Encrypt" if "Encrypt" in export_name or "encrypt" in export_name else "Decrypt"
    details = OPERATION_TARGETS[operation]
    if export_name.startswith("kmipkit_protocol::"):
        if export_name.endswith("Response::result"):
            targets = details["result"].copy()
            targets["rust"] = export_name
            return details["id"], targets, "operation-result-metadata-v2"
        targets = details["request"].copy()
        targets["rust"] = export_name
        return details["id"], targets, "operation-request-facade"
    if "ClientBatchOutcome" in export_name or "ClientResponseView" in export_name:
        targets = {**RESULT_CONTAINER_TARGETS}
        targets["rust"] = export_name
        return details["id"], targets, "operation-outcome-facade"
    targets = details["request"].copy()
    targets["rust"] = export_name
    return details["id"], targets, "operation-request-facade"


class T099CryptoDeclarationMappingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))

    def test_base_candidate_and_full_owner_identities_are_pinned(self) -> None:
        candidates = [
            row
            for row in self.manifest["publicDeclarations"]
            if row.get("candidateSpecId") == "KMIPKIT-0019"
        ]
        mapped = [
            row
            for row in self.manifest["publicDeclarations"]
            if row.get("approvalSource", {}).get("path") == OWNER_SPEC
        ]
        excluded = [
            row
            for row in self.manifest["approvedExclusions"]
            if row.get("approvalSource", {}).get("path") == OWNER_SPEC
        ]
        identities = mapped + excluded + candidates
        self.assertEqual(OWNER_IDENTITY_COUNT, len(identities))
        self.assertEqual(OWNER_IDENTITY_SHA256, _identity_hash(identities))
        if candidates:
            self.assertEqual(BASE_CANDIDATE_COUNT, len(candidates))
            self.assertEqual(BASE_CANDIDATE_SHA256, _identity_hash(candidates))
        else:
            self.assertEqual([], candidates)

    def test_no_kmipkit_0019_candidates_remain_blocked(self) -> None:
        unresolved = [
            row
            for row in self.manifest["publicDeclarations"]
            if row.get("candidateSpecId") == "KMIPKIT-0019"
        ]
        self.assertEqual([], unresolved)

    def test_owner_declarations_have_the_exact_mapping_and_exclusion_counts(self) -> None:
        mapped = [
            row
            for row in self.manifest["publicDeclarations"]
            if row.get("approvalSource", {}).get("path") == OWNER_SPEC
        ]
        excluded = [
            row
            for row in self.manifest["approvedExclusions"]
            if row.get("approvalSource", {}).get("path") == OWNER_SPEC
        ]
        t099_excluded = [row for row in excluded if "T099" in row.get("evidence", "")]
        self.assertEqual(OWNER_MAPPING_COUNT, len(mapped))
        self.assertEqual(OWNER_EXCLUSION_COUNT, len(excluded))
        t099_mapped = [row for row in mapped if row["exportName"] in _expected_mapped_exports()]
        self.assertEqual(T099_MAPPING_COUNT, len(t099_mapped))
        self.assertEqual(T099_EXCLUSION_COUNT, len(t099_excluded))
        self.assertEqual(_expected_mapped_exports(), {row["exportName"] for row in t099_mapped})

    def test_t099_mappings_reuse_the_exact_four_language_operation_facades(self) -> None:
        mapped = [
            row
            for row in self.manifest["publicDeclarations"]
            if row.get("approvalSource", {}).get("path") == OWNER_SPEC
            and row["exportName"] in _expected_mapped_exports()
        ]
        self.assertEqual(T099_MAPPING_COUNT, len(mapped))
        for row in mapped:
            with self.subTest(export=row["exportName"]):
                capability_id, expected_targets, _ = _expected_targets(row["exportName"])
                self.assertEqual(capability_id, row["capabilityId"])
                self.assertEqual("verified", row["declarationState"])
                self.assertEqual("four-language", row["parityScope"])
                self.assertEqual(APPROVAL_SOURCE, {key: row["approvalSource"][key] for key in APPROVAL_SOURCE})
                self.assertEqual(LANGUAGES, set(row["languageMappings"]))
                self.assertEqual(row["packageId"], row["languageMappings"]["rust"]["packageId"])
                self.assertEqual(
                    expected_targets,
                    {language: row["languageMappings"][language]["name"] for language in LANGUAGES},
                )
                expected_packages = {
                    "rust": row["packageId"],
                    "c": "c-kmipkit",
                    "java": "java-kmipkit",
                    "python": "python-kmipkit",
                }
                self.assertEqual(
                    expected_packages,
                    {
                        language: row["languageMappings"][language]["packageId"]
                        for language in LANGUAGES
                    },
                )
                requirements = set(row["requirementIds"])
                self.assertTrue(SHARED_REQUIREMENTS <= requirements)
                self.assertTrue(OWNER_REQUIREMENTS <= requirements)
                operation = "Encrypt" if "Encrypt" in row["exportName"] or "encrypt" in row["exportName"] else "Decrypt"
                self.assertTrue(OPERATION_NORMATIVE_REQUIREMENTS[operation] <= requirements)
                self.assertTrue(PARITY_EVIDENCE <= set(row["parityEvidenceIds"]))
                for evidence in PARITY_EVIDENCE:
                    self.assertTrue((ROOT / evidence).is_file(), evidence)

    def test_outcome_declarations_map_to_complete_result_containers(self) -> None:
        expected_exports = set()
        for operation in OPERATION_TARGETS:
            lower = operation.lower()
            for package_prefix in ("kmipkit", "kmipkit_client"):
                expected_exports.update(
                    {
                        f"{package_prefix}::ClientBatchOutcome::{operation}",
                        f"{package_prefix}::ClientBatchOutcome::{operation}::0",
                        f"{package_prefix}::ClientBatchOutcome::{lower}_response",
                    }
                )
            expected_exports.add(f"kmipkit_client::ClientResponseView::{lower}")

        declarations = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
        }
        self.assertTrue(expected_exports <= declarations.keys())
        for export_name in expected_exports:
            with self.subTest(export=export_name):
                row = declarations[export_name]
                expected_targets = {
                    "rust": export_name,
                    **RESULT_CONTAINER_TARGETS,
                }
                self.assertEqual(
                    expected_targets,
                    {
                        language: row["languageMappings"][language]["name"]
                        for language in LANGUAGES
                    },
                )

    def test_each_t099_exclusion_is_exact_and_owner_backed(self) -> None:
        excluded = [
            row
            for row in self.manifest["approvedExclusions"]
            if row.get("approvalSource", {}).get("path") == OWNER_SPEC
            and "T099" in row.get("evidence", "")
        ]
        self.assertEqual(T099_EXCLUSION_COUNT, len(excluded))
        for row in excluded:
            with self.subTest(export=row["exportName"]):
                self.assertEqual(APPROVAL_SOURCE, {key: row["approvalSource"][key] for key in APPROVAL_SOURCE})
                self.assertTrue(row["reason"].strip())
                self.assertTrue(row["evidence"].strip())
                self.assertTrue(any(requirement in row["reason"] for requirement in OWNER_REQUIREMENTS))
                self.assertTrue(any(requirement in row["evidence"] for requirement in OWNER_REQUIREMENTS))
                self.assertTrue(
                    all(requirement in row["evidence"] for requirement in SHARED_REQUIREMENTS)
                )

        traits = [row for row in excluded if row["exportName"].startswith("<")]
        errors = [
            row
            for row in excluded
            if not row["exportName"].startswith("<")
            and ("EncryptError" in row["exportName"] or "DecryptError" in row["exportName"])
        ]
        parsers = [row for row in excluded if "::try_from_" in row["exportName"]]
        request_helpers = [
            row
            for row in excluded
            if "Request::" in row["exportName"]
            and row not in parsers
            and not any(error in row["exportName"] for error in ("EncryptError", "DecryptError"))
            and not row["exportName"].startswith("<")
        ]
        self.assertEqual(92, len(traits))
        self.assertEqual(18, len(errors))
        self.assertEqual(4, len(parsers))
        self.assertEqual(8, len(request_helpers))


if __name__ == "__main__":
    unittest.main()
