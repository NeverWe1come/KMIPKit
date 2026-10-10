# KMIPKIT-0020 Traceability: Query and Ping

This implementation follows the approved client scope in `spec.md`. References
below point to executable code and tests in this branch. Structural TTLV
fixtures are derived from the pinned OASIS KMIP v2.1 source; the cited upstream
Query/Ping XML fixtures are unavailable, so no official vector pass or server
conformance claim is made.

## Client normative requirements

| Stable requirement ID | Exact source | Implemented behavior | Implementation | Verification |
| --- | --- | --- | --- | --- |
| `KMIPKIT-REQ-SPEC-6.1.40-014` | OASIS KMIP Specification v2.1 §6.1.40, Table 282; catalog clause `KMIPKIT-CLAUSE-SPEC-6.1.40-014` | Query contains at least one Query Function. Empty input fails before any exchange with `NotSent` delivery evidence. | `crates/kmipkit-protocol/src/query.rs::QueryRequest::to_ttlv_payload`; `crates/kmipkit-client/src/execute.rs::Client::query` | `crates/kmipkit-protocol/tests/unit/query_operation_tests.rs::query_rejects_an_empty_function_list_before_it_can_be_encoded`; `crates/kmipkit-client/tests/unit/query_execution_tests.rs::empty_query_is_not_sent` |
| `KMIPKIT-REQ-SPEC-6.1.40-016` | OASIS KMIP Specification v2.1 §6.1.40, Table 282; catalog clause `KMIPKIT-CLAUSE-SPEC-6.1.40-016` | Repeated Query Function values retain caller order and raw values. | `crates/kmipkit-protocol/src/query.rs::QueryRequest`; `crates/kmipkit-client/src/execute.rs::Client::query` | `crates/kmipkit-protocol/tests/unit/query_operation_tests.rs::query_request_preserves_required_repeated_functions_and_object_group_order`; `crates/kmipkit-client/tests/unit/query_execution_tests.rs::query_sends_one_request_with_repeated_functions_and_ordered_object_groups` |

## Operation and request-structure assignments

| Catalog element | Source | Implementation | Verification |
| --- | --- | --- | --- |
| `KMIPKIT-ELEM-OP-C2S-PING` and operation value `0000003B` | §6.1.36, Tables 271–272; §11.36 | `crates/kmipkit-protocol/src/ping.rs`; `crates/kmipkit-client/src/execute.rs::Client::ping` | `ping_operation_tests::ping_request_has_an_empty_operation_payload`; `ping_execution_tests::ping_sends_one_empty_payload_and_returns_success`; `ping_execution_tests::ping_failure_preserves_transport_delivery_and_does_not_retry`; `ping_execution_tests::malformed_ping_response_is_rejected_after_one_exchange` |
| `KMIPKIT-ELEM-OP-C2S-QUERY` and operation value `00000018` | §6.1.40, Tables 281–284; §11.36 | `crates/kmipkit-protocol/src/query.rs`; `crates/kmipkit-client/src/execute.rs::Client::query` | `query_operation_tests::query_request_preserves_required_repeated_functions_and_object_group_order`; `query_execution_tests::query_sends_one_request_with_repeated_functions_and_ordered_object_groups`; `query_execution_tests::query_failure_preserves_kmip_result_and_transport_delivery_without_retry` |
| `KMIPKIT-ELEM-ENUMERATION-QUERY-FUNCTION`, its 14 assigned values, and extension range | §11.44, Table 476 | `crates/kmipkit-protocol/src/query.rs::QueryFunction` | `query_operation_tests::names_all_standard_query_function_values_and_preserves_future_values`; `query_operation_tests::query_request_preserves_required_repeated_functions_and_object_group_order` |
| `KMIPKIT-ELEM-OPERATION-STRUCTURE-7-23-OBJECT-GROUPS`, `KMIPKIT-ELEM-STRUCTURE-MEMBER-7-23-OBJECT-GROUP`, and `KMIPKIT-ELEM-ATTRIBUTE-OBJECT-GROUP` | §7.23, Table 375; §4.35, Tables 99–100 | `crates/kmipkit-protocol/src/query.rs::QueryRequest::with_object_groups` | `query_operation_tests::query_request_distinguishes_absent_and_present_empty_object_groups`; `query_operation_tests::query_request_preserves_required_repeated_functions_and_object_group_order`; `query_execution_tests::query_sends_one_request_with_repeated_functions_and_ordered_object_groups` |

The Object Group request structure is represented as an optional `Structure`
containing zero or more repeated Text String attributes. The separate
`KMIPKIT-ELEM-ENUMERATION-OBJECT-GROUP-MEMBER` (§11.33) is not part of this
Query request field. The catalog has no operation-specific asynchronous
assignment for Ping or Query; both inherit typed-batch behavior from KMIPKIT-0007
FR-008 and KMIP v2.1 §§9.1–9.2, §11.3, Tables 400–401 and 431–432. If the
effective indicator permits Pending, the client preserves the typed response
and exact correlation bytes. The operation conveniences use the default
synchronous indicator, and no automatic Poll, retry, or follow-up is issued.

## Query response members in Table 283

Each known member is represented by a typed `QueryResponseField` variant and
an accessor on `QueryResponse`. Complex structures and unknown top-level or
nested Items retain generic TTLV values. The test below constructs every
Table 283 member and checks field order, repetitions, typed accessors, and a
nested unknown Item.

| Table 283 member | Typed response access | Verification |
| --- | --- | --- |
| Operation | `QueryResponse::operations` | `query_operation_tests::query_response_exposes_all_table_283_members_and_unknown_nested_items` |
| Object Type | `QueryResponse::object_types` | same test |
| Vendor Identification | `QueryResponse::vendor_identification` | same test; `query_response_rejects_repeated_singleton_table_283_fields` |
| Server Information | `QueryResponse::server_information` | same test; `query_response_rejects_repetitions_of_each_singleton_table_283_member`; `query_server_information_rejects_non_structure_ttlv_values` |
| Application Namespace | `QueryResponse::application_namespaces` | same test |
| Extension Information | `QueryResponse::extension_information` | same test |
| Attestation Type | `QueryResponse::attestation_types` | same test |
| RNG Parameters | `QueryResponse::rng_parameters` | same test |
| Profile Information | `QueryResponse::profile_information` | same test |
| Validation Information | `QueryResponse::validation_information` | same test |
| Capability Information | `QueryResponse::capability_information` | same test |
| Client Registration Method | `QueryResponse::client_registration_methods` | same test |
| Defaults Information | `QueryResponse::defaults_information` | same test; `query_response_rejects_repetitions_of_each_singleton_table_283_member` |
| Protection Storage Masks | `QueryResponse::protection_storage_masks` | `query_operation_tests::query_response_accepts_empty_protection_storage_masks_list`; `query_response_rejects_missing_required_protection_storage_masks_in_structured_form`; `query_response_rejects_repetitions_of_each_singleton_table_283_member` |

`query_operation_tests::query_accepts_the_empty_response_payload_form_and_preserves_common_failure_results`
verifies the empty success form and common KMIP failure result. The empty
Protection Storage Masks list verifies the structured form. `KMIPKIT-DISC-047`
remains open: §6.1.40 prose describes the empty response payload when no values
are available, while Table 283 marks Protection Storage Masks required in a
structured response. The decoder accepts both without deciding server
conformance.

## OASIS test-case mapping and limitations

| Catalog test ID | Official label | Source | Relationship | Fixture status and claim |
| --- | --- | --- | --- | --- |
| `KMIPKIT-TEST-CN01-2-67` | `TC-PING-1-21` | Test Cases v2.1 §2.67 | Ping | Fixture unavailable; no official vector pass claim |
| `KMIPKIT-TEST-PROF-5-17-1` | `QS-M-1-12` | Profiles v2.1 §5.17.1 | Query | Fixture unavailable; mapping confidence weak and source/case label mismatch is recorded in the catalog |
| `KMIPKIT-TEST-PROF-5-3-3-1` | `MSGENC-HTTPS-M-1-21` | Profiles v2.1 §5.3.3.1 | Query, HTTPS profile | Fixture unavailable; HTTPS profile is not claimed by this TTLV operation specification |
| `KMIPKIT-TEST-PROF-5-4-4-1` | `MSGENC-XML-M-1-21` | Profiles v2.1 §5.4.4.1 | Query, XML profile | Fixture unavailable; XML is out of scope |
| `KMIPKIT-TEST-PROF-5-5-4-1` | `MSGENC-JSON-M-1-21` | Profiles v2.1 §5.5.4.1 | Query, JSON profile | Fixture unavailable; JSON is out of scope |

The catalog assigns the two applicable client Query requirements to KMIPKIT-0020
and records executable implementation and verification references. Server-only
response obligations remain server requirements. `KMIPKIT-DISC-002` is limited
to HTTPS profile evidence; `KMIPKIT-DISC-039` applies to Query Asynchronous
Requests (§6.1.41), not ordinary Query. Ping and Query async outcomes are
covered by `ping_pending_preserves_typed_response_and_correlation_without_follow_up`
and `query_pending_preserves_typed_response_and_correlation_without_follow_up`;
they exercise only the shared client batch contract. No profile support,
interoperability certification, or official OASIS test-suite pass is claimed.
