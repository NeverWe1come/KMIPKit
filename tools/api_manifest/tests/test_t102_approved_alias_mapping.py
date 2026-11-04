"""Exact T102 mappings for Rust aliases of already-owned four-language APIs."""
from __future__ import annotations

import copy
import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
EXPECTED_ALIAS_PAIRS = (
    ('rust-kmipkit', 'kmipkit::DelegatedLoginRequest', 'rust-kmipkit-protocol', 'kmipkit_protocol::DelegatedLoginRequest'),
    ('rust-kmipkit', 'kmipkit::DelegatedLoginRequest::new', 'rust-kmipkit-protocol', 'kmipkit_protocol::DelegatedLoginRequest::new'),
    ('rust-kmipkit', 'kmipkit::DelegatedLoginRequest::with_lease_time_date_time', 'rust-kmipkit-protocol', 'kmipkit_protocol::DelegatedLoginRequest::with_lease_time_date_time'),
    ('rust-kmipkit', 'kmipkit::DelegatedLoginRequest::with_lease_time_interval', 'rust-kmipkit-protocol', 'kmipkit_protocol::DelegatedLoginRequest::with_lease_time_interval'),
    ('rust-kmipkit', 'kmipkit::DelegatedLoginRequest::with_request_count', 'rust-kmipkit-protocol', 'kmipkit_protocol::DelegatedLoginRequest::with_request_count'),
    ('rust-kmipkit', 'kmipkit::DelegatedLoginRequest::with_usage_limits', 'rust-kmipkit-protocol', 'kmipkit_protocol::DelegatedLoginRequest::with_usage_limits'),
    ('rust-kmipkit', 'kmipkit::DelegatedLoginResponse', 'rust-kmipkit-protocol', 'kmipkit_protocol::DelegatedLoginResponse'),
    ('rust-kmipkit', 'kmipkit::DelegatedLoginResponse::ticket', 'rust-kmipkit-protocol', 'kmipkit_protocol::DelegatedLoginResponse::ticket'),
    ('rust-kmipkit', 'kmipkit::GetConstraintsRequest', 'rust-kmipkit-protocol', 'kmipkit_protocol::GetConstraintsRequest'),
    ('rust-kmipkit', 'kmipkit::GetConstraintsRequest::new', 'rust-kmipkit-protocol', 'kmipkit_protocol::GetConstraintsRequest::new'),
    ('rust-kmipkit', 'kmipkit::GetConstraintsResponse', 'rust-kmipkit-protocol', 'kmipkit_protocol::GetConstraintsResponse'),
    ('rust-kmipkit', 'kmipkit::GetConstraintsResponse::constraints', 'rust-kmipkit-protocol', 'kmipkit_protocol::GetConstraintsResponse::constraints'),
    ('rust-kmipkit', 'kmipkit::GetConstraintsResponse::result', 'rust-kmipkit-protocol', 'kmipkit_protocol::GetConstraintsResponse::result'),
    ('rust-kmipkit', 'kmipkit::GetUsageAllocationRequest', 'rust-kmipkit-protocol', 'kmipkit_protocol::GetUsageAllocationRequest'),
    ('rust-kmipkit', 'kmipkit::GetUsageAllocationRequest::new', 'rust-kmipkit-protocol', 'kmipkit_protocol::GetUsageAllocationRequest::new'),
    ('rust-kmipkit', 'kmipkit::GetUsageAllocationRequest::with_unique_identifier', 'rust-kmipkit-protocol', 'kmipkit_protocol::GetUsageAllocationRequest::with_unique_identifier'),
    ('rust-kmipkit', 'kmipkit::GetUsageAllocationRequest::usage_limits_count', 'rust-kmipkit-protocol', 'kmipkit_protocol::GetUsageAllocationRequest::usage_limits_count'),
    ('rust-kmipkit', 'kmipkit::GetUsageAllocationResponse', 'rust-kmipkit-protocol', 'kmipkit_protocol::GetUsageAllocationResponse'),
    ('rust-kmipkit', 'kmipkit::GetUsageAllocationResponse::result', 'rust-kmipkit-protocol', 'kmipkit_protocol::GetUsageAllocationResponse::result'),
    ('rust-kmipkit', 'kmipkit::GetUsageAllocationResponse::unique_identifier', 'rust-kmipkit-protocol', 'kmipkit_protocol::GetUsageAllocationResponse::unique_identifier'),
    ('rust-kmipkit', 'kmipkit::LoginRequest', 'rust-kmipkit-protocol', 'kmipkit_protocol::LoginRequest'),
    ('rust-kmipkit', 'kmipkit::LoginRequest::new', 'rust-kmipkit-protocol', 'kmipkit_protocol::LoginRequest::new'),
    ('rust-kmipkit', 'kmipkit::LoginRequest::with_lease_time_date_time', 'rust-kmipkit-protocol', 'kmipkit_protocol::LoginRequest::with_lease_time_date_time'),
    ('rust-kmipkit', 'kmipkit::LoginRequest::with_lease_time_interval', 'rust-kmipkit-protocol', 'kmipkit_protocol::LoginRequest::with_lease_time_interval'),
    ('rust-kmipkit', 'kmipkit::LoginRequest::with_request_count', 'rust-kmipkit-protocol', 'kmipkit_protocol::LoginRequest::with_request_count'),
    ('rust-kmipkit', 'kmipkit::LoginRequest::with_usage_limits', 'rust-kmipkit-protocol', 'kmipkit_protocol::LoginRequest::with_usage_limits'),
    ('rust-kmipkit', 'kmipkit::LoginResponse', 'rust-kmipkit-protocol', 'kmipkit_protocol::LoginResponse'),
    ('rust-kmipkit', 'kmipkit::LoginResponse::ticket', 'rust-kmipkit-protocol', 'kmipkit_protocol::LoginResponse::ticket'),
    ('rust-kmipkit', 'kmipkit::LogoutRequest', 'rust-kmipkit-protocol', 'kmipkit_protocol::LogoutRequest'),
    ('rust-kmipkit', 'kmipkit::LogoutRequest::new', 'rust-kmipkit-protocol', 'kmipkit_protocol::LogoutRequest::new'),
    ('rust-kmipkit', 'kmipkit::LogoutResponse', 'rust-kmipkit-protocol', 'kmipkit_protocol::LogoutResponse'),
    ('rust-kmipkit', 'kmipkit::LogoutResponse::result', 'rust-kmipkit-protocol', 'kmipkit_protocol::LogoutResponse::result'),
    ('rust-kmipkit', 'kmipkit::ProfileValidationSelection::validate', 'rust-kmipkit-client', 'kmipkit_client::ProfileValidationSelection::validate'),
    ('rust-kmipkit', 'kmipkit::RngRetrieveRequest', 'rust-kmipkit-protocol', 'kmipkit_protocol::RngRetrieveRequest'),
    ('rust-kmipkit', 'kmipkit::RngRetrieveRequest::new', 'rust-kmipkit-protocol', 'kmipkit_protocol::RngRetrieveRequest::new'),
    ('rust-kmipkit', 'kmipkit::RngRetrieveResponse', 'rust-kmipkit-protocol', 'kmipkit_protocol::RngRetrieveResponse'),
    ('rust-kmipkit', 'kmipkit::RngRetrieveResponse::data', 'rust-kmipkit-protocol', 'kmipkit_protocol::RngRetrieveResponse::data'),
    ('rust-kmipkit', 'kmipkit::RngRetrieveResponse::with_ttlv', 'rust-kmipkit-protocol', 'kmipkit_protocol::RngRetrieveResponse::with_ttlv'),
    ('rust-kmipkit', 'kmipkit::RngSeedRequest', 'rust-kmipkit-protocol', 'kmipkit_protocol::RngSeedRequest'),
    ('rust-kmipkit', 'kmipkit::RngSeedRequest::new', 'rust-kmipkit-protocol', 'kmipkit_protocol::RngSeedRequest::new'),
    ('rust-kmipkit', 'kmipkit::RngSeedResponse', 'rust-kmipkit-protocol', 'kmipkit_protocol::RngSeedResponse'),
    ('rust-kmipkit', 'kmipkit::RngSeedResponse::data_length', 'rust-kmipkit-protocol', 'kmipkit_protocol::RngSeedResponse::data_length'),
    ('rust-kmipkit', 'kmipkit::RngSeedResponse::with_ttlv', 'rust-kmipkit-protocol', 'kmipkit_protocol::RngSeedResponse::with_ttlv'),
    ('rust-kmipkit', 'kmipkit::SetConstraintsRequest', 'rust-kmipkit-protocol', 'kmipkit_protocol::SetConstraintsRequest'),
    ('rust-kmipkit', 'kmipkit::SetConstraintsRequest::constraints', 'rust-kmipkit-protocol', 'kmipkit_protocol::SetConstraintsRequest::constraints'),
    ('rust-kmipkit', 'kmipkit::SetConstraintsRequest::new', 'rust-kmipkit-protocol', 'kmipkit_protocol::SetConstraintsRequest::new'),
    ('rust-kmipkit', 'kmipkit::SetConstraintsResponse', 'rust-kmipkit-protocol', 'kmipkit_protocol::SetConstraintsResponse'),
    ('rust-kmipkit', 'kmipkit::SetConstraintsResponse::result', 'rust-kmipkit-protocol', 'kmipkit_protocol::SetConstraintsResponse::result'),
    ('rust-kmipkit', 'kmipkit::SetDefaultsRequest', 'rust-kmipkit-protocol', 'kmipkit_protocol::SetDefaultsRequest'),
    ('rust-kmipkit', 'kmipkit::SetDefaultsRequest::defaults_information', 'rust-kmipkit-protocol', 'kmipkit_protocol::SetDefaultsRequest::defaults_information'),
    ('rust-kmipkit', 'kmipkit::SetDefaultsRequest::new', 'rust-kmipkit-protocol', 'kmipkit_protocol::SetDefaultsRequest::new'),
    ('rust-kmipkit', 'kmipkit::SetDefaultsRequest::with_defaults_information', 'rust-kmipkit-protocol', 'kmipkit_protocol::SetDefaultsRequest::with_defaults_information'),
    ('rust-kmipkit', 'kmipkit::SetDefaultsResponse', 'rust-kmipkit-protocol', 'kmipkit_protocol::SetDefaultsResponse'),
    ('rust-kmipkit', 'kmipkit::SetDefaultsResponse::result', 'rust-kmipkit-protocol', 'kmipkit_protocol::SetDefaultsResponse::result'),
    ('rust-kmipkit', 'kmipkit::SetDefaultsResponse::with_ttlv', 'rust-kmipkit-protocol', 'kmipkit_protocol::SetDefaultsResponse::with_ttlv'),
    ('rust-kmipkit', 'kmipkit::Ticket', 'rust-kmipkit-protocol', 'kmipkit_protocol::Ticket'),
    ('rust-kmipkit', 'kmipkit::Ticket::ticket_type', 'rust-kmipkit-protocol', 'kmipkit_protocol::Ticket::ticket_type'),
    ('rust-kmipkit', 'kmipkit::Ticket::with_ticket_value', 'rust-kmipkit-protocol', 'kmipkit_protocol::Ticket::with_ticket_value'),
    ('rust-kmipkit', 'kmipkit::Ticket::with_ttlv', 'rust-kmipkit-protocol', 'kmipkit_protocol::Ticket::with_ttlv'),
    ('rust-kmipkit', 'kmipkit::TicketType', 'rust-kmipkit-protocol', 'kmipkit_protocol::TicketType'),
    ('rust-kmipkit', 'kmipkit::TicketType::raw', 'rust-kmipkit-protocol', 'kmipkit_protocol::TicketType::raw'),
    ('rust-kmipkit', 'kmipkit::UsageAllocationContext', 'rust-kmipkit-client', 'kmipkit_client::UsageAllocationContext'),
    ('rust-kmipkit', 'kmipkit::UsageAllocationContext::allocated_units', 'rust-kmipkit-client', 'kmipkit_client::UsageAllocationContext::allocated_units'),
    ('rust-kmipkit', 'kmipkit::UsageAllocationContext::available_units', 'rust-kmipkit-client', 'kmipkit_client::UsageAllocationContext::available_units'),
    ('rust-kmipkit', 'kmipkit::UsageAllocationContext::from_successful_allocation', 'rust-kmipkit-client', 'kmipkit_client::UsageAllocationContext::from_successful_allocation'),
    ('rust-kmipkit', 'kmipkit::UsageAllocationContext::reserved_units', 'rust-kmipkit-client', 'kmipkit_client::UsageAllocationContext::reserved_units'),
    ('rust-kmipkit', 'kmipkit::UsageAllocationContext::unique_identifier', 'rust-kmipkit-client', 'kmipkit_client::UsageAllocationContext::unique_identifier'),
    ('rust-kmipkit', 'kmipkit::UsageAllocationContext::usage_limits_unit', 'rust-kmipkit-client', 'kmipkit_client::UsageAllocationContext::usage_limits_unit'),
    ('rust-kmipkit-client', 'kmipkit_client::Pkcs11Response::with_correlation_value', 'rust-kmipkit-protocol', 'kmipkit_protocol::Pkcs11Response::with_correlation_value'),
    ('rust-kmipkit-client', 'kmipkit_client::Pkcs11Response::with_function', 'rust-kmipkit-protocol', 'kmipkit_protocol::Pkcs11Response::with_function'),
    ('rust-kmipkit-client', 'kmipkit_client::Pkcs11Response::with_interface', 'rust-kmipkit-protocol', 'kmipkit_protocol::Pkcs11Response::with_interface'),
    ('rust-kmipkit-client', 'kmipkit_client::Pkcs11Response::with_output_parameters', 'rust-kmipkit-protocol', 'kmipkit_protocol::Pkcs11Response::with_output_parameters'),
    ('rust-kmipkit-client', 'kmipkit_client::Pkcs11Response::with_return_code', 'rust-kmipkit-protocol', 'kmipkit_protocol::Pkcs11Response::with_return_code'),
    ('rust-kmipkit-protocol', 'kmipkit_protocol::ProfileValidationSelection::validate', 'rust-kmipkit-client', 'kmipkit_client::ProfileValidationSelection::validate'),
)
EXPECTED_BLOCKED_CANDIDATES_AFTER_BATCH = 665


class T102ApprovedAliasMappingTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        self.declarations = {
            (row["packageId"], row["exportName"]): row
            for row in self.manifest["publicDeclarations"]
        }

    def test_exact_aliases_reuse_only_their_approved_four_language_owner_mapping(self) -> None:
        self.assertEqual(74, len(EXPECTED_ALIAS_PAIRS))
        for target_package, target_name, source_package, source_name in EXPECTED_ALIAS_PAIRS:
            with self.subTest(target=target_name):
                target = self.declarations[(target_package, target_name)]
                source = self.declarations[(source_package, source_name)]
                self.assertIn(target_package, {"rust-kmipkit", "rust-kmipkit-client", "rust-kmipkit-protocol"})
                self.assertEqual(source["kind"], target["kind"])
                self.assertIn(source["declarationState"], {"verified", "declared"})
                self.assertEqual(source["capabilityId"], target["capabilityId"])
                self.assertEqual(source["requirementIds"], target["requirementIds"])
                self.assertEqual(source["approvalSource"], target["approvalSource"])
                if "parityEvidenceIds" in source:
                    self.assertEqual(source["parityEvidenceIds"], target["parityEvidenceIds"])
                else:
                    self.assertNotIn("parityEvidenceIds", target)
                self.assertEqual(source["declarationState"], target["declarationState"])
                self.assertEqual("four-language", target["parityScope"])
                expected_mappings = copy.deepcopy(source["languageMappings"])
                expected_mappings["rust"] = {"packageId": target_package, "name": target_name}
                self.assertEqual(expected_mappings, target["languageMappings"])
                self.assertNotIn("candidateSpecId", target)
                self.assertNotIn("blockedReason", target)
                capability_id = target["capabilityId"]
                operation = next((item for item in self.manifest["operations"] if item["operationId"] == capability_id), None)
                if operation is not None:
                    self.assertIn(target["declarationId"], operation["declarations"]["rust"])
                else:
                    capability = next(item for item in self.manifest["capabilities"] if item["capabilityId"] == capability_id)
                    self.assertIn(target_name, capability["mappings"]["rust"])

    def test_t102_candidate_total_excludes_only_this_exact_batch(self) -> None:
        remaining = sum(
            row.get("declarationState") == "blocked" and row.get("candidateSpecId") == "KMIPKIT-0043"
            for row in self.manifest["publicDeclarations"]
        )
        self.assertEqual(EXPECTED_BLOCKED_CANDIDATES_AFTER_BATCH, remaining)


if __name__ == "__main__":
    unittest.main()
