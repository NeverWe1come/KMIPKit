# KMIPKIT-0042 Traceability: Interop, Log, Validate

The tables link each source row, stable element, requirement, implementation location, and named executable test. Table fields are traced through catalog elements and approved FRs; do not invent client duties from server result tables.

| Stable catalog element / requirement | Exact source | Coverage | Implementation / executable verification |
|---|---|---|---|
| KMIPKIT-ELEM-OP-C2S-INTEROP; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-INTEROP-00000034 | §6.1.26 Tables 241–243; §11.36 | Interop payload and Operation value | interop.rs; protocol/client Interop suites |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-15-INTEROP-FUNCTION | §6.1.26 Table 241; §7.15 | Required open Enumeration; KMIPKIT-0042 FR-001/FR-008 | Required/type/unknown round trip; client serialization evidence only, no production-server availability claim |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-16-INTEROP-IDENTIFIER | §6.1.26 Table 241; §7.16 | Required Text String | Missing/type/round-trip tests |
| Table 242 empty success; Table 243 errors | §6.1.26 | Empty payload and server result | Empty success and common-result preservation |
| KMIPKIT-ELEM-OP-C2S-LOG; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-LOG-0000002C | §6.1.29 Tables 250–252; §11.36 | Log payload and Operation value | log.rs; protocol/client Log suites |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-19-LOG-MESSAGE | §6.1.29 Table 250; §7.19 | Required, redacted message | Required/type, secret-sentinel redaction |
| KMIPKIT-DISC-017 | §6.1.29.1 / Table 252 | Approved Query-caption correction for Log | Table 252 reason preservation; no Query mapping |
| KMIPKIT-ELEM-OP-C2S-VALIDATE; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-VALIDATE-00000017 | §6.1.57 Tables 340–342; §11.36 | Validate payload and Operation value | validate.rs; protocol/client Validate suites |
| KMIPKIT-ELEM-TAG-420013 | §6.1.57 Table 340 | Repeated Certificate material | Repeated field/type/order cases |
| KMIPKIT-ELEM-TAG-420094 | §6.1.57 Table 340 | Repeated Managed Certificate identifier | Repeated field/type/order cases |
| KMIPKIT-ELEM-TAG-42009A | §6.1.57 Table 340 | Optional Validity Date | Present/absent request serialization tests; no evaluation test |
| KMIPKIT-ELEM-TAG-42009B | §6.1.57 Table 341 | Required open Validity Indicator | Missing/type/unknown value tests |
| KMIPKIT-REQ-SPEC-6.1.57-001 | §6.1.57 opening paragraph plus paragraph 2/Table 340; SHALL | Each request has one chain with at least one total Certificate or Unique Identifier member | Mixed, Certificate-only and identifier-only lists pass; both lists empty or a date-only request fails; malformed Certificate wrapper is rejected by Table 2 checks |
| KMIPKIT-REQ-SPEC-6.1.57-002-001 | §6.1.57 second paragraph; MAY | Chain may use Certificate objects, identifiers, or both | Certificate-only, identifier-only, mixed input cases |
| KMIPKIT-REQ-SPEC-6.1.57-002-002 | §6.1.57 second paragraph; MAY | Caller may specify evaluation date | Supplied Validity Date round trip |
| KMIPKIT-REQ-SPEC-6.1.57-004 | §6.1.57 Table 340 / repeated field; MAY | Repeated Certificates | Multiple Certificate values preserved |
| KMIPKIT-REQ-SPEC-6.1.57-005 | §6.1.57 Table 340 / repeated field; MAY | Repeated Unique Identifiers | Multiple identifiers preserved |
| KMIPKIT-REQ-SPEC-6.1.57-006 | §6.1.57 Validity Date rule; SHALL, actor unspecified | The source passively states that current date/time is assumed when omitted | Client request serialization preserves omission; source actor and date evaluation are unverified |

The Interop production-availability sentence is server-only; no KMIPKit client test claims a server implementation. There are no direct catalog Test Cases for these operations. FR-001–FR-008, SC-001–SC-004, the five actor-attributed Validate client requirements plus 006 request-serialization evidence, approved DISC-017, source-derived vectors, redaction, and required repository gates must be evidenced before review.


## Source verification before implementation

Verified the assigned dedicated worktree and feature branch, merged
`origin/release/1.0.0` without conflicts, and ran
`python tools/normative_catalog/validate.py`: valid (4 sources, 1411 clauses,
4044 records). All required operation, structure, tag and enum records exist;
no additional wire record is needed. Pinned Specification sections 6.1.26,
6.1.29 and 6.1.57 supply the table rows above. Tag Table 487 assigns Interop
Function 420160, Interop Identifier 420161, Log Message 420141, Certificate
420013, Unique Identifier 420094, Validity Date 42009A and Validity Indicator
42009B. The open indicator type is reused with the Validate-specific tag.

Interop production availability remains a server duty. Approved DISC-017
assigns the miscaptioned error introduction to Log Table 252. Neither rule
creates a client policy check. Certificate and identifier lists compose one chain; there is no nested or
second-chain API. The six Validate IDs map
to the named requirement tests in `validate_operation_tests.rs` below.


## Executable field and error-table evidence

Paths below are repository-relative. All fixtures are project-derived from
pinned OASIS Specification v2.1; no direct official Test Case exists for these
operations in the catalog.

| Project row ID | Source row and stable element | Executable verification |
|---|---|---|
| KMIPKIT-028-T241-FUNCTION | Table 241 Interop Function; KMIPKIT-ELEM-OPERATION-STRUCTURE-7-15-INTEROP-FUNCTION; KMIPKIT-ELEM-TAG-420160 | interop_operation_tests::table_241_required_fields_preserve_order_and_unknown_function; interop_operation_tests::interop_function_standard_vendor_and_future_values_are_open |
| KMIPKIT-028-T241-IDENTIFIER | Table 241 Interop Identifier; KMIPKIT-ELEM-OPERATION-STRUCTURE-7-16-INTEROP-IDENTIFIER; KMIPKIT-ELEM-TAG-420161 | interop_operation_tests::table_241_required_fields_preserve_order_and_unknown_function |
| KMIPKIT-028-T242-EMPTY | Table 242 empty success; KMIPKIT-ELEM-OP-C2S-INTEROP | interop_operation_tests::table_242_success_requires_empty_payload; interop_execution_tests::interop_sends_required_fields_and_returns_empty_success |
| KMIPKIT-028-T243-ERROR | Table 243 common result; KMIPKIT-ELEM-OP-C2S-INTEROP | interop_operation_tests::table_243_failure_pending_and_unknown_results_are_preserved; interop_execution_tests::failure_reason_message_and_diagnostics_are_lossless_and_redacted |
| KMIPKIT-028-T250-MESSAGE | Table 250 Log Message; KMIPKIT-ELEM-OPERATION-STRUCTURE-7-19-LOG-MESSAGE; KMIPKIT-ELEM-TAG-420141 | log_operation_tests::table_250_required_message_is_preserved_and_redacted |
| KMIPKIT-028-T251-EMPTY | Table 251 empty success; KMIPKIT-ELEM-OP-C2S-LOG | log_operation_tests::table_251_success_requires_empty_payload; log_execution_tests::log_sends_required_fields_and_returns_empty_success |
| KMIPKIT-028-T252-ERROR | Table 252 per approved DISC-017; KMIPKIT-ELEM-OP-C2S-LOG | log_operation_tests::table_252_failure_pending_and_unknown_results_are_preserved; log_execution_tests::failure_reason_message_and_diagnostics_are_lossless_and_redacted |
| KMIPKIT-028-T340-CERTIFICATE | Table 340 Certificate; KMIPKIT-ELEM-TAG-420013 | validate_operation_tests::req_004_repeated_certificate_values_preserve_order; validate_operation_tests::table_2_certificate_structure_rejects_missing_wrong_and_duplicate_fields |
| KMIPKIT-028-T340-IDENTIFIER | Table 340 Unique Identifier; KMIPKIT-ELEM-TAG-420094 | validate_operation_tests::req_005_repeated_identifiers_preserve_order_and_open_values |
| KMIPKIT-028-T340-DATE | Table 340 Validity Date; KMIPKIT-ELEM-TAG-42009A | validate_operation_tests::req_002_002_optional_validity_date_preserves_exact_value; validate_operation_tests::req_006_omitted_date_remains_absent_in_request_serialization |
| KMIPKIT-028-T341-INDICATOR | Table 341 Validity Indicator; KMIPKIT-ELEM-TAG-42009B | validate_operation_tests::table_341_requires_one_open_indicator_and_retains_unknown_fields; validate_execution_tests::validate_sends_chain_and_date_and_returns_open_indicator |
| KMIPKIT-028-T342-ERROR | Table 342 common result; KMIPKIT-ELEM-OP-C2S-VALIDATE | validate_operation_tests::table_342_failure_and_pending_preserve_results_without_indicator; validate_execution_tests::failure_reason_message_and_diagnostics_are_lossless_and_redacted |

Protocol files: `crates/kmipkit-protocol/src/{interop,log,validate}.rs`;
protocol tests: `crates/kmipkit-protocol/tests/unit/{interop,log,validate}_operation_tests.rs`.
Client integration: `crates/kmipkit-client/src/execute.rs`; execution tests:
`crates/kmipkit-client/tests/unit/{interop,log,validate}_execution_tests.rs`.
Interop and Log constructors require the exact mandatory typed fields, so
omission and wrong request types are unrepresentable through those constructors.
Raw message parsing and resource limits remain the common message/TTLV contract.

## Validate client requirements and unspecified-actor evidence

| Requirement | Implementation | Executable verification |
|---|---|---|
| KMIPKIT-REQ-SPEC-6.1.57-001 | ValidateRequest one chain with a combined minimum of one member across two ordered lists; Table 2 structure checking | validate_operation_tests::req_001_rejects_empty_chain_even_when_validity_date_is_present; validate_operation_tests::req_001_models_one_chain_and_rejects_malformed_certificate_wrapper; profile_rejects_validate_request_without_a_chain_member; validate_without_chain_member_is_not_sent |
| KMIPKIT-REQ-SPEC-6.1.57-002-001 | ValidateRequest::with_certificate / with_unique_identifier | validate_operation_tests::req_002_001_certificate_identifier_and_mixed_inputs_are_available |
| KMIPKIT-REQ-SPEC-6.1.57-002-002 | ValidateRequest::with_validity_date | validate_operation_tests::req_002_002_optional_validity_date_preserves_exact_value |
| KMIPKIT-REQ-SPEC-6.1.57-004 | Ordered Certificate entries in one chain | validate_operation_tests::req_004_repeated_certificate_values_preserve_order |
| KMIPKIT-REQ-SPEC-6.1.57-005 | Ordered typed identifier entries in one chain | validate_operation_tests::req_005_repeated_identifiers_preserve_order_and_open_values |
| KMIPKIT-REQ-SPEC-6.1.57-006 | Source actor unspecified; client serialization preserves absence only | validate_operation_tests::req_006_omitted_date_remains_absent_in_request_serialization |

The catalog records all six IDs as `scoped_verified` for feature traceability:
five have `role=client` and track client obligations or capabilities; 006 has
`role=unspecified` and tracks request-serialization evidence only. Its
`client_1_0` scope denotes feature/API serialization applicability, not source
actor attribution. The passive SHALL's actor is unspecified, and date
evaluation is unverified. No test claims server conformance or current-date
evaluation.

## Common results, security and remaining gates

Each execution suite exercises one exchange, wrong-operation association,
malformed response, local NotSent failure, NotSent/PossiblySent/ResponseStarted
transport failures and Pending with exact correlation bytes. No retry or
implicit Poll is performed. Error Table reasons and unknown reason values
remain intact. `generic_unknown_response_data_remains_exact_and_redacted`
in each protocol suite checks retained generic fields and sentinel bytes.
`failure_reason_message_and_diagnostics_are_lossless_and_redacted` checks
explicit Result Message inspection and redacted Debug/Display. Log text uses
`SecretText`; Certificate structures use existing zeroizing TTLV ownership.
There is no logger in these operation modules. Callbacks can copy sensitive
contents outside KMIPKit ownership; Java/Python copies remain subject to their
runtime limits as documented in the security guidance.

FR-001/FR-002/FR-003/FR-005/FR-006/FR-008 and the Rust portion of SC-001/SC-002/SC-003 have Rust
source-derived evidence above. FR-007 is enforced by retaining the unavailable
official-fixture and server status. FR-004 and SC-001 remain open for the
KMIPKIT-0043 mapping and four-language parity. SC-004 remains open for final
repository gates and two-server integration. The independent two-server 1.0
release gate also remains open. No profile, certification or interoperability
claim is made. Full command evidence and TDD commit IDs are recorded in
`implementation-evidence.md` and `quickstart.md`.


### Request payload extensions

Each request offers `with_extension(Item)`. Unknown checked payload fields
retain their order and values, including repeated/vendor tags, through the
shared `crates/kmipkit-protocol/src/diagnostics.rs` helper. Table-defined fields
must use typed constructors/builders; attempts to replace them through an
extension fail before sending. Tests in each protocol suite:
`unknown_request_extensions_preserve_order_without_overriding_table_fields`
and `request_extension_depth_limit_is_checked_before_encoding`. Each client
suite verifies `unknown_request_extensions_are_sent_once_and_redacted` and
`unknown_result_status_is_preserved_with_generic_payload`. These references
close the unknown-request-data part of FR-002 and FR-003 without turning
server policy into a local requirement.

The req_001 test establishes the one-chain API and rejects a malformed
Certificate wrapper through Table 2 shape rules. It does not detect disjoint
chains inside opaque DER content; the caller supplies one chain and the server
controls chain construction/validation policy.


## Existing common Structure order: scoped evidence

KMIPKIT-REQ-SPEC-10.1.2-001 (section 10.1.2, unnumbered Type table Structure row) requires the order of
known fields from their respective descriptions. Table 340 therefore groups
Certificates, Unique Identifiers and optional Validity Date. Relative order
inside each repeated list is preserved; mixed builder-call order does not
change this wire layout. Section 2.1/Table 2 requires Certificate Type before
Certificate Value. Tests:
`validate_operation_tests::table_340_groups_certificates_before_identifiers_preserving_each_list_order`
and `validate_operation_tests::table_2_certificate_value_before_type_is_rejected_structurally`.
The global common-rule catalog record remains unassigned for all-structure
ownership; this feature supplies scoped implementation/evidence references
through its Validate operation record and this trace. No local DER/trust or
chain-connectivity policy is introduced. See Decision 6 in research.md.
