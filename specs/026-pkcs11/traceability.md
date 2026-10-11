# KMIPKIT-0040 Traceability: PKCS#11

Before review, link each table field and assigned catalog element to implementation and executable verification. Do not invent client normative clauses from structural table headings.

| Stable catalog element | Exact source | Contract | Implementation / executable verification |
|---|---|---|---|
| KMIPKIT-ELEM-OP-C2S-PKCS-11 | Spec v2.1 §6.1.37 | Operation request/response | pkcs11.rs; execute.rs / pkcs11_operation_tests; pkcs11_execution_tests |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-PKCS-11-00000033 | §11.36 | Operation enum value | Catalog-backed registration / fake-transport operation assertion |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-26-PKCS-11-FUNCTION | §6.1.37 Tables 273–274; §7.26 | Required function Structure; Enumeration or Integer | Required, alternate-type, unknown-value, round-trip tests |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-28-PKCS-11-INTERFACE | Tables 273–274; §7.28 | Optional Interface | Absent/present tests |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-8-CORRELATION-VALUE | Tables 273–274; §7.8 | Optional exact bytes | Round trip, Pending passthrough, redaction |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-27-PKCS-11-INPUT-PARAMETERS | Table 273; §7.27 | Optional opaque input | Nested-value preservation and redaction |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-29-PKCS-11-OUTPUT-PARAMETERS | Table 274; §7.29 | Optional opaque output | Response preservation and redaction |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-30-PKCS-11-RETURN-CODE | Table 274; §7.30 | Required return-code structure | Missing/malformed and unknown-member tests |
| Table 275 result alternatives | §6.1.37 Table 275 | Server-reported Failure reasons | Shared KmipOperationResult preservation; never synthesize |

The approved spec has no separate §6.1.37 client-clause ID and no direct catalog Test Case. FR-001–FR-008 and SC-001–SC-004 are covered by protocol, fake-transport, redaction, catalog, and final-gate tests. Do not claim Profile support, a fixture pass, certification, or interoperability.

## Implementation targets confirmed before coding

The protocol tests will be named `table_273_request_preserves_order_and_opaque_members`,
`table_273_function_preserves_enumeration_or_integer`,
`table_274_success_retains_required_optional_and_unknown_fields`,
`table_274_rejects_missing_duplicate_wrong_type_and_out_of_order_fields`,
`table_275_preserves_failure_reasons`, and `diagnostics_redact_all_pkcs11_sentinels`.
They target `crates/kmipkit-protocol/src/pkcs11.rs` and source-derived fixtures
only. Nested Function and Return Code are checked against §§7.26 and 7.30,
Tables 378 and 382, with the approved Table 273/274 Integer alternative for
vendor functions. Interface and parameters stay opaque generic Structures.
Client tests target `execute.rs`, the `0x33` operation, one exchange,
Pending/correlation, malformed response delivery, and no retry.

The existing catalog contains all six operation-structure records, their tag
records, and the operation/operation-enumeration records. No missing source
record or generated runtime constant is needed. Baseline catalog validation
and the feature branch/release merge are recorded in implementation evidence.

## Executed Rust verification

All fixtures below are source-derived, not official OASIS test vectors.
The protocol suite has 12 tests and the client suite has 7 tests.

| Contract / catalog element | Implementation | Executable tests |
|---|---|---|
| Table 273 Interface / tag `KMIPKIT-ELEM-TAG-420159` | `Pkcs11Request::with_interface` | `table_273_request_preserves_order_and_opaque_members` |
| Table 273/274 Function / tag `KMIPKIT-ELEM-TAG-42015A` | `Pkcs11Request::new`; `validate_payload`; `with_function` | `table_273_function_preserves_enumeration_or_integer`; `table_273_rejects_missing_duplicate_or_wrong_function`; `table_274_rejects_missing_duplicate_wrong_type_and_out_of_order_fields` |
| Table 273/274 Correlation / tag `KMIPKIT-ELEM-TAG-4200D6` | `with_correlation_value`; `SecretBytes`; retained zeroizing TTLV | `table_273_request_preserves_order_and_opaque_members`; `table_274_success_retains_required_optional_and_unknown_fields` |
| Table 273 Input / tag `KMIPKIT-ELEM-TAG-42015B` | `with_input_parameters` | `table_273_request_preserves_order_and_opaque_members`; `request_preserves_empty_input_and_rejects_known_extension_duplicates_and_depth` |
| Table 274 Interface / tag `KMIPKIT-ELEM-TAG-420159` | `Pkcs11Response::with_interface` | `table_274_absence_remains_absent`; `table_274_success_retains_required_optional_and_unknown_fields` |
| Table 274 Output / tag `KMIPKIT-ELEM-TAG-42015C` | `with_output_parameters` | `table_274_absence_remains_absent`; `table_274_success_retains_required_optional_and_unknown_fields`; `table_274_rejects_malformed_optional_fields_and_return_value` |
| Table 274 Return Code / tag `KMIPKIT-ELEM-TAG-42015D` | `validate_payload`; `with_return_code` | `table_274_success_retains_required_optional_and_unknown_fields`; `table_274_rejects_missing_duplicate_wrong_type_and_out_of_order_fields`; `table_274_rejects_malformed_optional_fields_and_return_value` |
| Table 275 all 12 named reason alternatives | common `KmipOperationResult` | `table_275_preserves_failure_reasons`; `pkcs11_failure_preserves_all_table_275_reasons_without_retry` |
| Operation `0x33` | `ClientRequest::Pkcs11`; `ClientOperation::Pkcs11`; `pkcs11_response_outcome` | `pkcs11_sends_table_273_once_and_exposes_table_274_losslessly` |
| Unknown nested/outer values and generic access | zeroizing retained `OwnedResponseTtlv`; `with_ttlv`; `with_ttlv_payload` | `table_274_success_retains_required_optional_and_unknown_fields`; `pkcs11_sends_table_273_once_and_exposes_table_274_losslessly` |
| Pending and exact batch correlation, without Poll | shared `read_operation_outcome` / `PendingResponse::Pkcs11` | `pending_retains_result_and_generic_payload_without_fabricated_fields`; `pkcs11_pending_preserves_exact_batch_correlation_without_poll` |
| Validation / delivery / no retry | common one-exchange executor | `pkcs11_invalid_function_is_not_sent_and_diagnostics_are_redacted`; `pkcs11_transport_failure_preserves_delivery_and_never_retries`; `pkcs11_malformed_response_retains_response_started_delivery` |
| Formatting and sanitized error sources | redacted TTLV/SecretBytes; `Pkcs11Error` | `diagnostics_redact_all_pkcs11_sentinels`; `public_conversion_errors_have_only_sanitized_categories_and_sources`; `pkcs11_options_method_and_request_enum_preserve_redaction` |

The seven operation/field structure rows in the original table have direct
source-to-code-to-test links. There is no separate applicable §6.1.37 client
normative-clause ID; no synthetic clause is introduced. Table 273/274 mandatory
Function/Return Code requirements and optional rows are each represented above.

| Requirement | Evidence / remaining gate |
|---|---|
| FR-001 | Typed request/response, closed operation associations, explicit `Client::pkcs11` and options method; protocol/client tests |
| FR-002 | Required fields, Enum/Integer, malformed known outer/nested types, duplicates/order, depth, unknown preservation tests |
| FR-003 | Table 275 reasons, open reason values, retained complete generic response, Pending correlation, delivery and no-retry tests; shared Result Message redaction contract |
| FR-004 | Rust typed and generic TTLV complete; C/Java/Python gated by T019 / KMIPKIT-0043 |
| FR-005 | TTLV owned payloads and `SecretBytes` zeroize using existing storage; diagnostic sentinel tests; adapters/runtime-copy documentation remains gated with T019 |
| FR-006 | No independent catalog client clauses; structural source rows and corresponding relevant negatives verified |
| FR-007 | Official PKCS#11 fixture unavailable; no profile, official vector, interoperability, certification or FIPS claim |
| FR-008 | Table rows, operation enum, tag and nested structure-member catalog records link to implementation/tests; generated report validated |
| SC-001 | Rust operation implemented; four-language gate remains T019 |
| SC-002 | All Table 273/274 fields and Table 275 alternatives mapped and locally exercised; no separate client clauses |
| SC-003 | Fake transport confirms one exchange, typed Success/Failure/Pending, unknowns, malformed response and delivery state |
| SC-004 | Local and release quality gates recorded in `implementation-evidence.md`; multi-platform coverage and two independent servers remain required |

## Explicit outstanding gates

T019 remains blocked: active release commit
`a0846037eb9930b24d9b9e3d8c63b2e66b438d1a` has no integrated approved
KMIPKIT-0043 capability mapping. Existing API-manifest outputs are checked
without adding invented binding mappings. T022 remains open until its full
coverage/platform/security and two-independent-server conditions are met.
No live PKCS#11 interoperability exchange was performed in this increment.
