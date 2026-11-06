"""T096 contract for KMIPKIT-0014 server-generated creation declarations."""

from __future__ import annotations

import hashlib
import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
OWNER_SPEC = "KMIPKIT-0014"
OWNER_SPEC_PATH = "specs/014-server-generated-creation/spec.md"
BASE_CANDIDATE_COUNT = 316
BASE_CANDIDATE_SHA256 = "a829600a7a27ebb540f367952525d35ff992d3ff7a3d58713ea8de8ea9f4c32c"
BASE_TRAIT_COUNT = 226
BASE_TRAIT_SHA256 = "216648405667364abd3e6d7fcd7dc36f2e8ac833c0964870452dd2edbacc41c7"

SHARED_REQUIREMENTS = {
    "KMIPKIT-0043-FR-004",
    "KMIPKIT-0043-FR-005",
    "KMIPKIT-0043-FR-010",
    "KMIPKIT-0043-FR-017",
    "KMIPKIT-0043-FR-021",
    "KMIPKIT-0043-FR-022",
}
LANGUAGES = {"rust", "c", "java", "python"}

FUNCTIONAL_EXPORTS = {
    "kmipkit::Client::create",
    "kmipkit::Client::create_key_pair",
    "kmipkit::Client::create_split_key",
    "kmipkit::ClientRequest::Create::0",
    "kmipkit::ClientOperation::Create",
    "kmipkit::ClientRequest::Create",
    "kmipkit_client::Client::create",
    "kmipkit_client::Client::create_key_pair",
    "kmipkit_client::Client::create_split_key",
    "kmipkit_client::ClientResponseView::create",
    "kmipkit_client::ClientResponseView::create_key_pair",
    "kmipkit_client::ClientResponseView::create_split_key",
    "kmipkit_client::ClientRequest::Create::0",
    "kmipkit_protocol::CreateKeyPairRequest::has_common_attributes",
    "kmipkit_protocol::CreateKeyPairRequest::has_private_key_attributes",
    "kmipkit_protocol::CreateKeyPairRequest::has_public_key_attributes",
    "kmipkit_protocol::CreateKeyPairRequest::into_ttlv_payload",
    "kmipkit_protocol::CreateKeyPairRequest::new",
    "kmipkit_protocol::CreateKeyPairRequest::with_common_attributes",
    "kmipkit_protocol::CreateKeyPairRequest::with_common_protection_storage_masks",
    "kmipkit_protocol::CreateKeyPairRequest::with_private_key_attributes",
    "kmipkit_protocol::CreateKeyPairRequest::with_private_protection_storage_masks",
    "kmipkit_protocol::CreateKeyPairRequest::with_public_key_attributes",
    "kmipkit_protocol::CreateKeyPairRequest::with_public_protection_storage_masks",
    "kmipkit_protocol::CreateKeyPairResponse::result",
    "kmipkit_protocol::CreateKeyPairResponse::try_from_response_item",
    "kmipkit_protocol::CreateRequest::attributes",
    "kmipkit_protocol::CreateRequest::into_ttlv_payload",
    "kmipkit_protocol::CreateRequest::new",
    "kmipkit_protocol::CreateRequest::object_type",
    "kmipkit_protocol::CreateRequest::protection_storage_masks",
    "kmipkit_protocol::CreateRequest::with_protection_storage_masks",
    "kmipkit_protocol::CreateResponse::result",
    "kmipkit_protocol::CreateResponse::try_from_response_item",
    "kmipkit_protocol::CreateSplitKeyRequest::attributes",
    "kmipkit_protocol::CreateSplitKeyRequest::into_ttlv_payload",
    "kmipkit_protocol::CreateSplitKeyRequest::new",
    "kmipkit_protocol::CreateSplitKeyRequest::object_type",
    "kmipkit_protocol::CreateSplitKeyRequest::prime_field_size",
    "kmipkit_protocol::CreateSplitKeyRequest::protection_storage_masks",
    "kmipkit_protocol::CreateSplitKeyRequest::split_key_method",
    "kmipkit_protocol::CreateSplitKeyRequest::split_key_parts",
    "kmipkit_protocol::CreateSplitKeyRequest::split_key_threshold",
    "kmipkit_protocol::CreateSplitKeyRequest::unique_identifier",
    "kmipkit_protocol::CreateSplitKeyRequest::with_attributes",
    "kmipkit_protocol::CreateSplitKeyRequest::with_prime_field_size",
    "kmipkit_protocol::CreateSplitKeyRequest::with_protection_storage_masks",
    "kmipkit_protocol::CreateSplitKeyRequest::with_unique_identifier",
    "kmipkit_protocol::CreateSplitKeyResponse::result",
    "kmipkit_protocol::CreateSplitKeyResponse::try_from_response_item",
    "kmipkit_protocol::CreateSplitKeyResponse::unique_identifiers",
    "kmipkit_protocol::ObjectType::from_raw",
    "kmipkit_protocol::ObjectType::raw",
    "kmipkit_protocol::SplitKeyMethod::POLYNOMIAL_SHARING_GF_2_16",
    "kmipkit_protocol::SplitKeyMethod::POLYNOMIAL_SHARING_GF_2_8",
    "kmipkit_protocol::SplitKeyMethod::POLYNOMIAL_SHARING_PRIME_FIELD",
    "kmipkit_protocol::SplitKeyMethod::XOR",
    "kmipkit_protocol::SplitKeyMethod::from_raw",
    "kmipkit_protocol::SplitKeyMethod::raw",
    "kmipkit_protocol::CreateError::InvalidOperationResult::0",
    "kmipkit_protocol::CreateKeyPairError::InvalidOperationResult::0",
    "kmipkit_protocol::CreateSplitKeyError::InvalidOperationResult::0",
    "kmipkit_protocol::UniqueIdentifier::Enumeration::0",
    "kmipkit_protocol::UniqueIdentifier::Integer::0",
    "kmipkit_protocol::UniqueIdentifier::TextString::0",
    "kmipkit_protocol::CreateError",
    "kmipkit_protocol::CreateKeyPairError",
    "kmipkit_protocol::CreateKeyPairRequest",
    "kmipkit_protocol::CreateRequest",
    "kmipkit_protocol::CreateSplitKeyError",
    "kmipkit_protocol::CreateSplitKeyRequest",
    "kmipkit_protocol::CreateSplitKeyResponse",
    "kmipkit_protocol::ObjectType",
    "kmipkit_protocol::SplitKeyMethod",
    "kmipkit_protocol::CreateError::InvalidOperationResult",
    "kmipkit_protocol::CreateError::MalformedSuccessPayload",
    "kmipkit_protocol::CreateError::MissingResultStatus",
    "kmipkit_protocol::CreateError::MissingSuccessPayload",
    "kmipkit_protocol::CreateError::UnexpectedOperation",
    "kmipkit_protocol::CreateKeyPairError::InvalidOperationResult",
    "kmipkit_protocol::CreateKeyPairError::MalformedSuccessPayload",
    "kmipkit_protocol::CreateKeyPairError::MissingResultStatus",
    "kmipkit_protocol::CreateKeyPairError::MissingSuccessPayload",
    "kmipkit_protocol::CreateKeyPairError::UnexpectedOperation",
    "kmipkit_protocol::CreateSplitKeyError::InvalidOperationResult",
    "kmipkit_protocol::CreateSplitKeyError::MalformedSuccessPayload",
    "kmipkit_protocol::CreateSplitKeyError::MissingResultStatus",
    "kmipkit_protocol::CreateSplitKeyError::MissingSuccessPayload",
    "kmipkit_protocol::CreateSplitKeyError::PolynomialMethodRequiresPrimeFieldSize",
    "kmipkit_protocol::CreateSplitKeyError::UnexpectedOperation",
}

REQUEST_INSPECTION_EXPORTS = {
    "kmipkit_protocol::CreateRequest::attributes",
    "kmipkit_protocol::CreateRequest::into_ttlv_payload",
    "kmipkit_protocol::CreateRequest::object_type",
    "kmipkit_protocol::CreateRequest::protection_storage_masks",
    "kmipkit_protocol::CreateKeyPairRequest::has_common_attributes",
    "kmipkit_protocol::CreateKeyPairRequest::has_private_key_attributes",
    "kmipkit_protocol::CreateKeyPairRequest::has_public_key_attributes",
    "kmipkit_protocol::CreateKeyPairRequest::into_ttlv_payload",
    "kmipkit_protocol::CreateSplitKeyRequest::attributes",
    "kmipkit_protocol::CreateSplitKeyRequest::into_ttlv_payload",
    "kmipkit_protocol::CreateSplitKeyRequest::object_type",
    "kmipkit_protocol::CreateSplitKeyRequest::prime_field_size",
    "kmipkit_protocol::CreateSplitKeyRequest::protection_storage_masks",
    "kmipkit_protocol::CreateSplitKeyRequest::split_key_method",
    "kmipkit_protocol::CreateSplitKeyRequest::split_key_parts",
    "kmipkit_protocol::CreateSplitKeyRequest::split_key_threshold",
    "kmipkit_protocol::CreateSplitKeyRequest::unique_identifier",
}

RUST_ERROR_EXPORTS = {
    "kmipkit_protocol::CreateError",
    "kmipkit_protocol::CreateError::UnexpectedOperation",
    "kmipkit_protocol::CreateError::MissingResultStatus",
    "kmipkit_protocol::CreateError::InvalidOperationResult",
    "kmipkit_protocol::CreateError::InvalidOperationResult::0",
    "kmipkit_protocol::CreateError::MissingSuccessPayload",
    "kmipkit_protocol::CreateError::MalformedSuccessPayload",
    "kmipkit_protocol::CreateKeyPairError",
    "kmipkit_protocol::CreateKeyPairError::UnexpectedOperation",
    "kmipkit_protocol::CreateKeyPairError::MissingResultStatus",
    "kmipkit_protocol::CreateKeyPairError::InvalidOperationResult",
    "kmipkit_protocol::CreateKeyPairError::InvalidOperationResult::0",
    "kmipkit_protocol::CreateKeyPairError::MissingSuccessPayload",
    "kmipkit_protocol::CreateKeyPairError::MalformedSuccessPayload",
    "kmipkit_protocol::CreateSplitKeyError",
    "kmipkit_protocol::CreateSplitKeyError::UnexpectedOperation",
    "kmipkit_protocol::CreateSplitKeyError::MissingResultStatus",
    "kmipkit_protocol::CreateSplitKeyError::InvalidOperationResult",
    "kmipkit_protocol::CreateSplitKeyError::InvalidOperationResult::0",
    "kmipkit_protocol::CreateSplitKeyError::MissingSuccessPayload",
    "kmipkit_protocol::CreateSplitKeyError::MalformedSuccessPayload",
    "kmipkit_protocol::CreateSplitKeyError::PolynomialMethodRequiresPrimeFieldSize",
}

EXCLUDED_FUNCTIONAL_EXPORTS = REQUEST_INSPECTION_EXPORTS | RUST_ERROR_EXPORTS
MAPPED_FUNCTIONAL_EXPORTS = FUNCTIONAL_EXPORTS - EXCLUDED_FUNCTIONAL_EXPORTS

FACTORY_TARGETS = {
    "create": {
        "c": ("c-kmipkit", "kmipkit_client_batch_item_create"),
        "java": ("java-kmipkit", "org.kmipkit.extensions.ClientBatchItem.create"),
        "python": ("python-kmipkit", "kmipkit.extensions.client_batch_item_create"),
    },
    "create_key_pair": {
        "c": ("c-kmipkit", "kmipkit_client_batch_item_create_key_pair"),
        "java": (
            "java-kmipkit",
            "org.kmipkit.extensions.ClientBatchItem.createKeyPair",
        ),
        "python": (
            "python-kmipkit",
            "kmipkit.extensions.client_batch_item_create_key_pair",
        ),
    },
    "create_split_key": {
        "c": ("c-kmipkit", "kmipkit_client_batch_item_create_split_key"),
        "java": (
            "java-kmipkit",
            "org.kmipkit.extensions.ClientBatchItem.createSplitKey",
        ),
        "python": (
            "python-kmipkit",
            "kmipkit.extensions.client_batch_item_create_split_key",
        ),
    },
}
EXECUTE_TARGETS = {
    "c": ("c-kmipkit", "kmipkit_client_execute"),
    "java": ("java-kmipkit", "org.kmipkit.extensions.Client.execute"),
    "python": ("python-kmipkit", "kmipkit.client.Client.execute"),
}
RESPONSE_TARGETS = {
    "create": {
        "c": ("c-kmipkit", "kmipkit_client_batch_item_response_create"),
        "java": (
            "java-kmipkit",
            "org.kmipkit.extensions.ClientBatchItemResult.createResponse",
        ),
        "python": (
            "python-kmipkit",
            "kmipkit.client.BatchItemResult.create_object_type",
        ),
    },
    "create_key_pair": {
        "c": ("c-kmipkit", "kmipkit_client_batch_item_response_create_key_pair"),
        "java": (
            "java-kmipkit",
            "org.kmipkit.extensions.ClientBatchItemResult.createKeyPairResponse",
        ),
        "python": (
            "python-kmipkit",
            "kmipkit.client.BatchItemResult.private_key_unique_identifier",
        ),
    },
    "create_split_key": {
        "c": (
            "c-kmipkit",
            "kmipkit_client_batch_item_response_create_split_key_unique_identifier_at",
        ),
        "java": (
            "java-kmipkit",
            "org.kmipkit.extensions.ClientBatchItemResult.createSplitKeyUniqueIdentifiers",
        ),
        "python": (
            "python-kmipkit",
            "kmipkit.client.BatchItemResult.split_key_unique_identifiers",
        ),
    },
}
METADATA_TARGETS = {
    "c": ("c-kmipkit", "kmipkit_client_batch_item_response_metadata_v2"),
    "java": (
        "java-kmipkit",
        "org.kmipkit.extensions.ClientBatchItemResult.metadataV2",
    ),
    "python": ("python-kmipkit", "kmipkit.client.BatchItemResult.metadata_v2"),
}
OPERATION_TARGETS = {
    "c": ("c-kmipkit", "kmipkit_client_batch_item_metadata_v2_t.operation"),
    "java": (
        "java-kmipkit",
        "org.kmipkit.extensions.ClientBatchItemResult.operation",
    ),
    "python": ("python-kmipkit", "kmipkit.client.BatchItemResult.operation"),
}
UNIQUE_IDENTIFIER_TARGETS = {
    "c": ("c-kmipkit", "kmipkit_unique_identifier_v1_t"),
    "java": ("java-kmipkit", "org.kmipkit.extensions.UniqueIdentifier"),
    "python": ("python-kmipkit", "kmipkit.extensions.UniqueIdentifier"),
}


def _operation_kind(export_name: str) -> str:
    if "CreateSplitKey" in export_name or "create_split_key" in export_name or "SplitKeyMethod" in export_name:
        return "create_split_key"
    if "CreateKeyPair" in export_name or "create_key_pair" in export_name:
        return "create_key_pair"
    return "create"


def _expected_targets(export_name: str) -> dict[str, tuple[str, str]]:
    if "UniqueIdentifier::" in export_name:
        return UNIQUE_IDENTIFIER_TARGETS
    if "ClientOperation::Create" in export_name:
        return OPERATION_TARGETS
    if "Client::create" in export_name:
        return EXECUTE_TARGETS
    if "::result" in export_name:
        return METADATA_TARGETS
    operation = _operation_kind(export_name)
    if "Response" in export_name or "ClientResponseView::" in export_name:
        return RESPONSE_TARGETS[operation]
    return FACTORY_TARGETS[operation]


def _expected_owner_requirements(export_name: str) -> set[str]:
    if "UniqueIdentifier::" in export_name:
        ids = {"001", "002", "003", "005"}
    elif "ObjectType" in export_name:
        ids = {"002", "005", "006"}
    elif "SplitKeyMethod" in export_name:
        ids = {"005", "006", "015"}
    elif "ClientOperation::Create" in export_name:
        ids = {"001", "008"}
    elif "ClientRequest::Create" in export_name:
        ids = {"001", "002", "007"}
    elif "Client::create_split_key" in export_name:
        ids = {"001", "005", "006", "007", "012", "014", "015"}
    elif "Client::create_key_pair" in export_name:
        ids = {"001", "003", "004", "006", "007", "014"}
    elif "Client::create" in export_name:
        ids = {"001", "002", "006", "007", "014"}
    elif "CreateSplitKey" in export_name or "create_split_key" in export_name:
        if "Response" in export_name or "response" in export_name:
            ids = {"001", "005", "007", "012"}
        else:
            ids = {"001", "005", "006", "007", "012", "014", "015"}
    elif "CreateKeyPair" in export_name or "create_key_pair" in export_name:
        if "Response" in export_name or "response" in export_name:
            ids = {"001", "003", "004", "007", "013"}
        else:
            ids = {"001", "003", "004", "006", "014"}
    elif "Response" in export_name or "response" in export_name:
        ids = {"001", "002", "007"}
    else:
        ids = {"001", "002", "006", "014"}
    return {f"{OWNER_SPEC}-FR-{requirement}" for requirement in ids}


def _expected_exclusion_requirements(export_name: str) -> set[str]:
    if export_name in REQUEST_INSPECTION_EXPORTS:
        if "CreateSplitKeyRequest" in export_name:
            ids = {"005", "006", "012", "014", "015"}
        elif "CreateKeyPairRequest" in export_name:
            ids = {"003", "004", "014"}
        else:
            ids = {"002", "014"}
    elif "CreateSplitKeyError" in export_name:
        ids = {"005", "009", "012"}
        if "PolynomialMethodRequiresPrimeFieldSize" in export_name:
            ids.add("015")
    elif "CreateKeyPairError" in export_name:
        ids = {"003", "009", "013"}
    else:
        ids = {"002", "009"}
    return {f"{OWNER_SPEC}-FR-{requirement}" for requirement in ids}


def _identity_hash(rows: list[dict[str, object]]) -> str:
    identities = sorted(f"{row['packageId']}|{row['exportName']}" for row in rows)
    return hashlib.sha256("\n".join(identities).encode("utf-8")).hexdigest()


class T096CreationParityManifestTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))

    def test_base_candidate_inventory_is_pinned(self) -> None:
        mapped = [
            row
            for row in self.manifest["publicDeclarations"]
            if row.get("exportName") in MAPPED_FUNCTIONAL_EXPORTS
        ]
        excluded = [
            row
            for row in self.manifest["approvedExclusions"]
            if row["approvalSource"].get("path") == OWNER_SPEC_PATH
            and (row["exportName"] in EXCLUDED_FUNCTIONAL_EXPORTS or row["exportName"].startswith("<"))
        ]
        candidates = mapped + excluded
        self.assertEqual(BASE_CANDIDATE_COUNT, len(candidates))
        self.assertEqual(BASE_CANDIDATE_SHA256, _identity_hash(candidates))
        trait_rows = [row for row in candidates if row["exportName"].startswith("<")]
        self.assertEqual(BASE_TRAIT_COUNT, len(trait_rows))
        self.assertEqual(BASE_TRAIT_SHA256, _identity_hash(trait_rows))
        self.assertEqual(
            FUNCTIONAL_EXPORTS,
            {row["exportName"] for row in candidates if not row["exportName"].startswith("<")},
        )

    def test_creation_candidates_are_mapped_or_exactly_excluded(self) -> None:
        unresolved = [
            row
            for row in self.manifest["publicDeclarations"]
            if row.get("candidateSpecId") == OWNER_SPEC
        ]
        self.assertEqual(
            0,
            len(unresolved),
            "all accepted KMIPKIT-0014 declarations must be mapped or specifically excluded; "
            f"unresolved={[(row['packageId'], row['exportName']) for row in unresolved[:5]]}",
        )

        mapped = [
            row
            for row in self.manifest["publicDeclarations"]
            if row["exportName"] in MAPPED_FUNCTIONAL_EXPORTS
        ]
        self.assertEqual(
            MAPPED_FUNCTIONAL_EXPORTS,
            {row["exportName"] for row in mapped},
        )
        for row in mapped:
            with self.subTest(export=row["exportName"]):
                self.assertEqual("declared", row["declarationState"])
                self.assertEqual("four-language", row["parityScope"])
                self.assertEqual(
                    {
                        "kind": "feature-specification",
                        "path": OWNER_SPEC_PATH,
                        "status": "Accepted",
                        "evidence": "**Status**: Accepted for implementation.",
                    },
                    row["approvalSource"],
                )
                self.assertTrue(SHARED_REQUIREMENTS <= set(row["requirementIds"]))
                owner_requirements = {
                    requirement
                    for requirement in row["requirementIds"]
                    if requirement.startswith(f"{OWNER_SPEC}-FR-")
                }
                self.assertEqual(
                    _expected_owner_requirements(row["exportName"]),
                    owner_requirements,
                )
                self.assertEqual(LANGUAGES, set(row["languageMappings"]))
                for language, mapping in row["languageMappings"].items():
                    target = (mapping["packageId"], mapping["name"])
                    expected_target = (
                        (row["packageId"], row["exportName"])
                        if language == "rust"
                        else _expected_targets(row["exportName"])[language]
                    )
                    self.assertEqual(expected_target, target)
                    self.assertIn(
                        target,
                        {
                            (declaration["packageId"], declaration["exportName"])
                            for declaration in self.manifest["publicDeclarations"]
                        },
                    )

        functional_exclusions = [
            row
            for row in self.manifest["approvedExclusions"]
            if row["approvalSource"].get("path") == OWNER_SPEC_PATH
            and row["exportName"] in EXCLUDED_FUNCTIONAL_EXPORTS
        ]
        self.assertEqual(
            EXCLUDED_FUNCTIONAL_EXPORTS,
            {row["exportName"] for row in functional_exclusions},
        )
        for row in functional_exclusions:
            with self.subTest(exclusion=row["exportName"]):
                self.assertTrue(row["reason"].strip())
                self.assertTrue(row["evidence"].strip())
                self.assertIn("KMIPKIT-0043-FR-021", row["evidence"])
                self.assertEqual("Accepted", row["approvalSource"]["status"])
                for requirement in _expected_exclusion_requirements(row["exportName"]):
                    self.assertIn(requirement, row["evidence"])
                if row["exportName"] in REQUEST_INSPECTION_EXPORTS:
                    self.assertIn("request", row["reason"])
                elif "PolynomialMethodRequiresPrimeFieldSize" in row["exportName"]:
                    self.assertIn("invalid_input", row["reason"])
                else:
                    self.assertIn("parser diagnostic", row["reason"])

        trait_exclusions = [
            row
            for row in self.manifest["approvedExclusions"]
            if row["approvalSource"].get("path") == OWNER_SPEC_PATH
            and row["exportName"].startswith("<")
        ]
        self.assertEqual(BASE_TRAIT_COUNT, len(trait_exclusions))
        self.assertEqual(BASE_TRAIT_SHA256, _identity_hash(trait_exclusions))
        for row in trait_exclusions:
            with self.subTest(exclusion=row["exportName"]):
                if " as core::convert::From<" in row["exportName"]:
                    self.assertIn("From implementation", row["reason"])
                    self.assertIn("private `", row["reason"])
                    self.assertIn("internal Rust conversion", row["reason"])
                else:
                    self.assertIn("Rust trait implementation path", row["reason"])
                    self.assertIn("not a separate KMIPKit capability", row["reason"])
                    self.assertIn("independently mapped or exactly excluded", row["evidence"])
                self.assertIn("KMIPKIT-0043-FR-021", row["evidence"])
                self.assertIn("KMIPKIT-0014-FR-011", row["evidence"])
                self.assertEqual("Accepted", row["approvalSource"]["status"])

    def test_creation_capability_mapping_includes_new_ready_exports(self) -> None:
        capability = next(
            row
            for row in self.manifest["capabilities"]
            if row["capabilityId"] == "server_generated_creation"
        )
        declarations = [
            row
            for row in self.manifest["publicDeclarations"]
            if row["exportName"] in MAPPED_FUNCTIONAL_EXPORTS
            and row["capabilityId"] == "server_generated_creation"
        ]
        for language in LANGUAGES:
            expected = {
                row["languageMappings"][language]["name"]
                for row in declarations
            }
            with self.subTest(language=language):
                self.assertTrue(
                    expected <= set(capability["mappings"][language]),
                    f"missing {language} capability exports: "
                    f"{sorted(expected - set(capability['mappings'][language]))}",
                )


if __name__ == "__main__":
    unittest.main()
