# KMIPKIT-0042 Traceability: Interop, Log, Validate

The tables link each source row, stable element, requirement, implementation location, and named executable test. Table fields are traced through catalog elements and approved FRs; do not invent client duties from server result tables.

| Stable catalog element / requirement | Exact source | Coverage | Implementation / executable verification |
|---|---|---|---|
| KMIPKIT-ELEM-OP-C2S-INTEROP; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-INTEROP-00000034 | §6.1.26 Tables 241–243; §11.36 | Interop payload and Operation value | interop.rs; protocol/client Interop suites |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-15-INTEROP-FUNCTION | §6.1.26 Table 241; §7.15 | Required open Enumeration | Required/type/unknown round trip |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-16-INTEROP-IDENTIFIER | §6.1.26 Table 241; §7.16 | Required Text String | Missing/type/round-trip tests |
| Table 242 empty success; Table 243 errors | §6.1.26 | Empty payload and server result | Empty success and common-result preservation |
| KMIPKIT-ELEM-OP-C2S-LOG; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-LOG-0000002C | §6.1.29 Tables 250–252; §11.36 | Log payload and Operation value | log.rs; protocol/client Log suites |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-19-LOG-MESSAGE | §6.1.29 Table 250; §7.19 | Required, redacted message | Required/type, secret-sentinel redaction |
| KMIPKIT-DISC-017 | §6.1.29.1 / Table 252 | Approved Query-caption correction for Log | Table 252 reason preservation; no Query mapping |
| KMIPKIT-ELEM-OP-C2S-VALIDATE; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-VALIDATE-00000017 | §6.1.57 Tables 340–342; §11.36 | Validate payload and Operation value | validate.rs; protocol/client Validate suites |
| KMIPKIT-ELEM-TAG-420013 | §6.1.57 Table 340 | Repeated Certificate material | Repeated field/type/order cases |
| KMIPKIT-ELEM-TAG-420094 | §6.1.57 Table 340 | Repeated Managed Certificate identifier | Repeated field/type/order cases |
| KMIPKIT-ELEM-TAG-42009A | §6.1.57 Table 340 | Optional Validity Date | Present/absent and server-default interpretation tests |
| KMIPKIT-ELEM-TAG-42009B | §6.1.57 Table 341 | Required open Validity Indicator | Missing/type/unknown value tests |
| KMIPKIT-REQ-SPEC-6.1.57-001 | §6.1.57 opening paragraph; SHALL | A request contains only one certificate chain | Two ordered lists compose one chain; malformed Certificate wrapper rejected by Table 2 checks |
| KMIPKIT-REQ-SPEC-6.1.57-002-001 | §6.1.57 second paragraph; MAY | Chain may use Certificate objects, identifiers, or both | Certificate-only, identifier-only, mixed input cases |
| KMIPKIT-REQ-SPEC-6.1.57-002-002 | §6.1.57 second paragraph; MAY | Caller may specify evaluation date | Supplied Validity Date round trip |
| KMIPKIT-REQ-SPEC-6.1.57-004 | §6.1.57 Table 340 / repeated field; MAY | Repeated Certificates | Multiple Certificate values preserved |
| KMIPKIT-REQ-SPEC-6.1.57-005 | §6.1.57 Table 340 / repeated field; MAY | Repeated Unique Identifiers | Multiple identifiers preserved |
| KMIPKIT-REQ-SPEC-6.1.57-006 | §6.1.57 Validity Date rule; SHALL | Omitted date means server assumes current date/time | Omission remains absent; request test does not synthesize local time |

The Interop production-availability sentence is server-only; no KMIPKit client test claims a server implementation. There are no direct catalog Test Cases for these operations. FR-001–FR-008, SC-001–SC-004, all six Validate requirements, approved DISC-017, source-derived vectors, redaction, and required repository gates must be evidenced before review.


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
| KMIPKIT-028-T241-FUNCTION | Table 241 Interop Function; KMIPKIT-ELEM-OPERATION-STRUCTURE-7-15-INTEROP-FUNCTION; KMIPKIT-ELEM-TAG-420160; KMIPKIT-0042-FR-001/003/005 | `interop_operation_tests::table_241_required_fields_preserve_order_and_unknown_function`; `interop_operation_tests::interop_function_standard_vendor_and_future_values_are_open`; Rust/C/Java/Python constructor tests in `interop_abi_tests.rs`, `InteropJavaFacadeTest`, `test_interop_operation.py`, and compiled C consumer `interop_constructor_is_callable_from_c` |
| KMIPKIT-028-T241-IDENTIFIER | Table 241 Interop Identifier; KMIPKIT-ELEM-OPERATION-STRUCTURE-7-16-INTEROP-IDENTIFIER; KMIPKIT-ELEM-TAG-420161; KMIPKIT-0042-FR-002/005/006 | `interop_operation_tests::table_241_required_fields_preserve_order_and_unknown_function`; UTF-8 rejection and redaction in Rust/C `interop_abi_tests.rs`, Java `InteropJavaFacadeTest`, and Python `test_interop_operation.py` |
| KMIPKIT-028-T242-EMPTY | Table 242 empty success; KMIPKIT-ELEM-OP-C2S-INTEROP | interop_operation_tests::table_242_success_requires_empty_payload; interop_execution_tests::interop_sends_required_fields_and_returns_empty_success |
| KMIPKIT-028-T243-ERROR | Table 243 common result; KMIPKIT-ELEM-OP-C2S-INTEROP; KMIPKIT-0042-FR-003/004/007/008 | `interop_operation_tests::table_243_failure_pending_and_unknown_results_are_preserved`; `interop_execution_tests::failure_reason_message_and_diagnostics_are_lossless_and_redacted`; C ABI `ffi_interop_ttlv_getter_copies_success_failure_and_pending_responses` exercises the native FFI getter for Success/Failure/Pending; Java `InteropResponseMappingTest` and Python `test_interop_response_owns_complete_ttlv_and_redacts_repr` verify owned-TTLV conversion and redaction with a native TTLV structure while stubbing the response-accessor tuple; compiled C consumer `interop_response_accessor_clears_invalid_outputs_from_c` verifies the native C symbol clears invalid outputs. The language wrapper tests do not independently execute all three native response statuses. |
| KMIPKIT-028-T250-MESSAGE | Table 250 Log Message; KMIPKIT-ELEM-OPERATION-STRUCTURE-7-19-LOG-MESSAGE; KMIPKIT-ELEM-TAG-420141 | log_operation_tests::table_250_required_message_is_preserved_and_redacted |
| KMIPKIT-028-T251-EMPTY | Table 251 empty success; KMIPKIT-ELEM-OP-C2S-LOG | log_operation_tests::table_251_success_requires_empty_payload; log_execution_tests::log_sends_required_fields_and_returns_empty_success |
| KMIPKIT-028-T252-ERROR | Table 252 per approved DISC-017; KMIPKIT-ELEM-OP-C2S-LOG | log_operation_tests::table_252_failure_pending_and_unknown_results_are_preserved; log_execution_tests::failure_reason_message_and_diagnostics_are_lossless_and_redacted |
| KMIPKIT-028-T340-CERTIFICATE | Table 340 Certificate; KMIPKIT-ELEM-TAG-420013 | validate_operation_tests::req_004_repeated_certificate_values_preserve_order; validate_operation_tests::table_2_certificate_structure_rejects_missing_wrong_and_duplicate_fields |
| KMIPKIT-028-T340-IDENTIFIER | Table 340 Unique Identifier; KMIPKIT-ELEM-TAG-420094 | validate_operation_tests::req_005_repeated_identifiers_preserve_order_and_open_values |
| KMIPKIT-028-T340-DATE | Table 340 Validity Date; KMIPKIT-ELEM-TAG-42009A | validate_operation_tests::req_002_002_optional_validity_date_preserves_exact_value; validate_operation_tests::req_006_omitted_date_remains_absent_for_server_default |
| KMIPKIT-028-T341-INDICATOR | Table 341 Validity Indicator; KMIPKIT-ELEM-TAG-42009B | validate_operation_tests::table_341_requires_one_open_indicator_and_retains_unknown_fields; validate_execution_tests::validate_sends_chain_and_date_and_returns_open_indicator |
| KMIPKIT-028-T342-ERROR | Table 342 common result; KMIPKIT-ELEM-OP-C2S-VALIDATE | validate_operation_tests::table_342_failure_and_pending_preserve_results_without_indicator; validate_execution_tests::failure_reason_message_and_diagnostics_are_lossless_and_redacted |

Protocol files: `crates/kmipkit-protocol/src/{interop,log,validate}.rs`;
protocol tests: `crates/kmipkit-protocol/tests/unit/{interop,log,validate}_operation_tests.rs`.
Client integration: `crates/kmipkit-client/src/execute.rs`; execution tests:
`crates/kmipkit-client/tests/unit/{interop,log,validate}_execution_tests.rs`.
Interop and Log constructors require the exact mandatory typed fields, so
omission and wrong request types are unrepresentable through those constructors.
Raw message parsing and resource limits remain the common message/TTLV contract.

## Six normative Validate client requirements

| Requirement | Implementation | Executable verification |
|---|---|---|
| KMIPKIT-REQ-SPEC-6.1.57-001 | ValidateRequest one chain with two ordered lists; Table 2 structure checking | validate_operation_tests::req_001_models_one_chain_and_rejects_malformed_certificate_wrapper |
| KMIPKIT-REQ-SPEC-6.1.57-002-001 | ValidateRequest::with_certificate / with_unique_identifier | validate_operation_tests::req_002_001_certificate_identifier_and_mixed_inputs_are_available |
| KMIPKIT-REQ-SPEC-6.1.57-002-002 | ValidateRequest::with_validity_date | validate_operation_tests::req_002_002_optional_validity_date_preserves_exact_value |
| KMIPKIT-REQ-SPEC-6.1.57-004 | Ordered Certificate entries in one chain | validate_operation_tests::req_004_repeated_certificate_values_preserve_order |
| KMIPKIT-REQ-SPEC-6.1.57-005 | Ordered typed identifier entries in one chain | validate_operation_tests::req_005_repeated_identifiers_preserve_order_and_open_values |
| KMIPKIT-REQ-SPEC-6.1.57-006 | Optional date retained absent; no local timestamp | validate_operation_tests::req_006_omitted_date_remains_absent_for_server_default |

All six are cataloged as `scoped_verified`: Rust client structure and
forwarding are verified. No test claims that a server applied its own policy,
evaluated a certificate, or used its current time.

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


## Validate four-language parity (T069)

The six request traces above are implemented by the approved Rust request model and mapped through the common C ABI to Java/JNI and Python/CFFI. They remain client input capabilities: these tests do not claim that a remote server accepts a chain or applies any certificate policy. The exact OASIS source remains immutable.

| Surface | Evidence |
|---|---|
| Rust and C ABI | `validate_abi_tests::validate_constructor_preserves_identifier_and_optional_date`; `validate_abi_tests::validate_constructor_borrows_and_copies_payload_extensions`; `validate_abi_tests::validate_constructor_rejects_noncanonical_absent_date_and_clears_output` |
| Compiled C consumer | `validate_constructor_is_callable_from_c`; `validate_constructor_copies_payload_extensions` in `bindings/c/tests/client_abi_consumer.c` |
| Java and JNI | `ValidateJavaFacadeTest` covers identifier-only, omitted-date, input-list validation, and borrowed TTLV extension ownership; `ValidateResponseMappingTest` covers typed response mapping |
| Python and CFFI | `test_validate_builder_accepts_identifier_only_chain_and_optional_date`; `test_validate_builder_rejects_missing_lists_and_out_of_range_date`; `test_validate_builder_copies_unknown_payload_extensions_and_preserves_input`; response mapping and cleanup tests in `test_validate_operation.py` |
| Manifest and generated API | `test_validate_parity_manifest`; pinned generator `--check`; generated C/JNI/Python API declarations and parity fixtures |

Unknown checked payload extensions preserve their caller order and are copied before the builder returns. C/JNI/Python apply the shared 100,000-input-item ceiling; the Rust FFI checks the aggregate encoded input budget and default item limits before cloning the supplied structures/items. The caller retains ownership of each input. Validity Date omission remains absent for server-side current-time evaluation. Success-only Validity Indicator and response TTLV accessors do not expose operation success values for Failure or Pending.

The T069 input-limit follow-up also rejects overlap between any C input-array span and `out_item` before either region is written, and enforces one aggregate TTLV element ceiling across the Validate payload before cloning. Regression coverage: `validate_constructor_rejects_output_overlapping_payload_extension_array`, `validate_constructor_enforces_one_aggregate_ttlv_element_limit`, `builderRejectsOversizedExtensionListBeforeCopyingIt`, and `test_validate_builder_preflights_aggregate_identifier_bytes_before_copying`. JNI snapshots each text identifier once while enforcing the cumulative byte ceiling, so later copies cannot observe a replaced `byte[][]` entry.
