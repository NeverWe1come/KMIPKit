# KMIPKIT-0019 Traceability

## Implementation preflight evidence

Recorded 2026-10-10 before implementation. The implementation branch `feature/KMIPKIT-0019-encrypt-decrypt-implementation` is based on `origin/release/1.0.0` at `b901fea43e55ab178414616d0e55c2a2f862f047`; this exact release commit is an ancestor of the branch. The required merged dependencies are ancestors of that release commit: KMIPKIT-0005 `548dda0e0e15abda0023264abf85f51179127987`, KMIPKIT-0006 `35a445f500d0ac0b55fe39cd95bf25984ea65216`, KMIPKIT-0007 `4126d62f16927753a6ae6fc3f5e628d29245683f`, KMIPKIT-0009 `ae87b89d43957e4fc028e785dc181e69b0165dac`, KMIPKIT-0013 `ba6421152f739b21204b0792f9e7fc1e12234c29`, and KMIPKIT-0018 Recover `5779451d86335f1f59ee4f6b23cd9d0c21cb32b3`.

The immutable normative source `specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html` has SHA-256 `8bf9d914c097e98a6509aa1ffcbf03406f738066e940597aee93d0a5e07addcf`, matching `specification/catalog/kmip-2.1.json`; no upstream source change is part of this work. The relevant clauses and tables were checked in that pinned file: §6.1 multipart rules and correlation handling; §6.1.11 Tables 196–198; §6.1.17 Tables 214–216; §4.16; and §§7.3, 7.4, 7.8, 7.9, 7.14, and 7.17. Data request values preserve the three §7.9 encodings (Byte String, Enumeration, and Integer); response Data is a Byte String. Cryptographic Parameters are optional at the operation level, while variable-IV modes require IV Length and GCM requires Tag Length as applicable. Table 59's “REQUIRED” extraction was confirmed to be its column heading rather than the Cryptographic Parameters row's requiredness; `KMIPKIT-CLAUSE-SPEC-4.16-004` therefore remains excluded with the existing rationale and no requirement link.

The catalog assigns the required Encrypt/Decrypt tags, and `python tools/normative_catalog/generate_ttlv_tags.py --check` passes for `crates/kmipkit-ttlv/src/generated/tag_allocations.rs`. Representative operation and shared-field allocations include Data `0x4200C2`, Cryptographic Parameters `0x42002B`, IV/Counter/Nonce `0x42003D`, Correlation Value `0x4200D6`, Init Indicator `0x4200D7`, Final Indicator `0x4200D8`, Authenticated Encryption Additional Data `0x4200FE`, and Authenticated Encryption Tag `0x4200FF`. Rust uses the generic `RawTag`/`Tag` API; this does not generate or require named constants. DISC-045 remains open: accept Init=true and Final=true when Data is present, and reject only that shape with omitted Data locally before transmission while the discrepancy has no authoritative disposition. DISC-046 remains open as recorded above.

## Requirement-to-implementation traceability

Feature: KMIPKIT-0019, Encrypt and Decrypt. Normative source is the pinned KMIP v2.1 Specification. The rows below identify implemented shared behavior and its focused tests where available; paths and tests labeled later remain assigned to the Encrypt/Decrypt request-model tasks. The shared Cryptographic Parameters validator checks required-member presence and preserves the supplied ordered Structure; known-member TTLV type and cardinality validation remains assigned to those later request conversions. Other planned paths become verified implementation/test references only after their implementation and checks. The feature owns 21 applicable client requirement IDs. Four related server-only ID Placeholder requirements remain in the normative catalog but are unassigned to KMIPKIT-0019; the client permission to omit Unique Identifier is represented by KMIPKIT-REQ-SPEC-6.1-003-002.

| Stable catalog requirement | Source | Feature requirement | Planned implementation | Planned verification |
| --- | --- | --- | --- | --- |
| KMIPKIT-REQ-SPEC-6.1-001-001 | §6.1 | FR-001 | crates/kmipkit-client/src/execute.rs | encrypt_and_decrypt_dispatch_once |
| KMIPKIT-REQ-SPEC-6.1-001-002 | §6.1 | FR-001, FR-005, FR-007 | crates/kmipkit-client/src/execute.rs | success_failure_and_pending_response_shapes |
| KMIPKIT-REQ-SPEC-6.1-003-002 | §6.1 | FR-002 | crates/kmipkit-client/src/execute.rs | eligible_placeholder_item_preserves_server_failure_after_prior_item_failure |
| KMIPKIT-REQ-SPEC-6.1-005 | §6.1 | FR-010 | KMIPKIT-0018 Recover model plus caller sequence through crates/kmipkit-client/src/execute.rs | caller_can_recover_archived_object_before_encrypt |
| KMIPKIT-REQ-SPEC-4.16-001-001 | §4.16 | FR-003 | `crates/kmipkit-protocol/src/cryptographic_parameters.rs` (shared validator); `crates/kmipkit-protocol/src/encrypt.rs` and `decrypt.rs` (later typed request conversion) | `omitted_cryptographic_parameters_remain_absent_without_validation`; later `encrypt_optional_parameters_roundtrip` and `decrypt_optional_parameters_roundtrip` |
| KMIPKIT-REQ-SPEC-4.16-001-002 | §4.16 | FR-003 | `crates/kmipkit-protocol/src/cryptographic_parameters.rs` (shared ordered preservation); `crates/kmipkit-protocol/src/encrypt.rs` and `decrypt.rs` (later typed request conversion) | `unknown_parameter_members_survive_in_original_order_and_encoding`; later `encrypt_optional_parameters_roundtrip` and `decrypt_optional_parameters_roundtrip` |
| KMIPKIT-REQ-SPEC-4.16-002 | §4.16 | FR-003 | `crates/kmipkit-protocol/src/cryptographic_parameters.rs` (shared presence validator); `crates/kmipkit-protocol/src/encrypt.rs` and `decrypt.rs` (later typed request conversion) | `variable_iv_mode_requires_iv_length_and_preserves_supplied_parameters`; `ccm_requires_iv_length_without_requiring_gcm_tag_length`; later operation-model request tests |
| KMIPKIT-REQ-SPEC-4.16-003 | §4.16 | FR-003 | `crates/kmipkit-protocol/src/cryptographic_parameters.rs` (shared presence validator); `crates/kmipkit-protocol/src/encrypt.rs` and `decrypt.rs` (later typed request conversion) | `gcm_requires_tag_length_and_preserves_supplied_parameters`; later operation-model request tests |
| KMIPKIT-REQ-SPEC-6.1.11-001-001 | §6.1.11 Table 196 | FR-002, FR-003 | crates/kmipkit-protocol/src/decrypt.rs | decrypt_optional_parameters_roundtrip |
| KMIPKIT-REQ-SPEC-6.1.11-001-002 | §6.1.11 Table 196 | FR-002, FR-003 | crates/kmipkit-protocol/src/decrypt.rs | decrypt_iv_omission_is_preserved |
| KMIPKIT-REQ-SPEC-6.1.11-004-001 | §6.1.11 | FR-003 | crates/kmipkit-protocol/src/decrypt.rs | decrypt_supplied_parameters_match_requested_method |
| KMIPKIT-REQ-SPEC-6.1.11-004-002 | §6.1.11 | FR-003 | crates/kmipkit-client/src/execute.rs | decrypt_missing_remote_parameters_preserves_server_result |
| KMIPKIT-REQ-SPEC-6.1.11-005 | §6.1.11 | FR-006 | crates/kmipkit-protocol/src/decrypt.rs | decrypt_aad_is_present_on_initial_part |
| KMIPKIT-REQ-SPEC-6.1.11-006 | §6.1.11 | FR-006 | crates/kmipkit-protocol/src/decrypt.rs | decrypt_tag_is_present_on_initial_part |
| KMIPKIT-REQ-SPEC-6.1.17-001-001 | §6.1.17 Table 214 | FR-002, FR-003 | crates/kmipkit-protocol/src/encrypt.rs | encrypt_optional_parameters_roundtrip |
| KMIPKIT-REQ-SPEC-6.1.17-001-002 | §6.1.17 Table 214 | FR-002, FR-003 | crates/kmipkit-protocol/src/encrypt.rs | encrypt_iv_omission_is_preserved |
| KMIPKIT-REQ-SPEC-6.1.17-005-001 | §6.1.17 | FR-003 | crates/kmipkit-protocol/src/encrypt.rs | encrypt_supplied_parameters_match_requested_method |
| KMIPKIT-REQ-SPEC-6.1.17-005-002 | §6.1.17 | FR-003 | crates/kmipkit-client/src/execute.rs | encrypt_missing_remote_parameters_preserves_server_result |
| KMIPKIT-REQ-SPEC-6.1.17-006 | §6.1.17 | FR-006 | crates/kmipkit-protocol/src/encrypt.rs | encrypt_aad_is_present_on_initial_part |
| KMIPKIT-REQ-SPEC-7.8-001 | §7.8 | FR-004 | crates/kmipkit-protocol/src/encrypt.rs; crates/kmipkit-protocol/src/decrypt.rs | multipart_follow_up_uses_first_response_correlation |
| KMIPKIT-REQ-SPEC-6.1-001-003 | §6.1; §§8.1–8.6 | FR-001; shared batch mechanics from KMIPKIT-0006 | crates/kmipkit-client/src/execute.rs | encrypt_decrypt_can_be_batched_with_other_operations |

KMIPKIT-REQ-SPEC-6.1-001-003 is assigned to KMIPKIT-0019 because the feature exposes Encrypt/Decrypt within multi-operation messages. Generic batching, Batch Order Option, and ID Placeholder substitution still reuse KMIPKIT-0006's message/batch model; this feature does not redefine those shared mechanics. Other common batch, result, transport, and async requirements are inherited from KMIPKIT-0006/0007/0009 and named in the feature specification.

## Operation and shared-field elements

| Catalog element | OASIS source | Feature use | Verification target |
| --- | --- | --- | --- |
| KMIPKIT-ELEM-OP-C2S-ENCRYPT | §6.1.17 | Encrypt request/result | encrypt_table_payload_roundtrip |
| KMIPKIT-ELEM-ATTRIBUTE-CRYPTOGRAPHIC-PARAMETERS | §4.16 | Optional managed-object attribute value and recognized IV Length/Tag Length conditions | `crates/kmipkit-protocol/src/cryptographic_parameters.rs`; `variable_iv_mode_requires_iv_length_and_preserves_supplied_parameters`, `ccm_requires_iv_length_without_requiring_gcm_tag_length`, `gcm_requires_tag_length_and_preserves_supplied_parameters`, `unknown_parameter_members_survive_in_original_order_and_encoding`, `omitted_cryptographic_parameters_remain_absent_without_validation` |
| KMIPKIT-ELEM-OP-C2S-DECRYPT | §6.1.11 | Decrypt request/result | decrypt_table_payload_roundtrip |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-3-AUTHENTICATED-ENCRYPTION-ADDITIONAL-DATA | §7.3 | Initial multipart input | encrypt_aad_is_present_on_initial_part; decrypt_aad_is_present_on_initial_part |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-4-AUTHENTICATED-ENCRYPTION-TAG | §7.4 | Decrypt input, Encrypt output | decrypt_tag_is_present_on_initial_part; encrypt_response_preserves_tag |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-8-CORRELATION-VALUE | §7.8 | Multipart follow-up | multipart_follow_up_uses_first_response_correlation |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-9-DATA | §7.9 | Shared request Data union; Encrypt/Decrypt request models and Byte String response Data follow in later tasks | `crates/kmipkit-protocol/src/operation_data.rs`; `operation_data_preserves_byte_string_type_and_exact_value`, `operation_data_preserves_enumeration_type_and_exact_unknown_value`, `operation_data_preserves_integer_type_and_exact_signed_value`, `debug_and_display_redact_every_operation_data_variant`, `byte_string_data_uses_secret_bytes_and_zeroizing_ttlv_ownership` |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-14-FINAL-INDICATOR | §7.14 | Final part control | multipart_final_indicator_is_caller_controlled |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-17-INIT-INDICATOR | §7.17 | Initial part control | multipart_initial_indicator_is_caller_controlled |

## OASIS Test Case records

The three fixtures contain 13, 13, and 11 request-response exchanges. The adapter selects 10 Encrypt pairs from §2.99, 10 Encrypt pairs from §2.100, and 4 Encrypt plus 4 Decrypt pairs from §2.101: 28 in-scope pairs total. Catalog operation-to-case associations follow the HTML descriptions: §2.100 names both Encrypt and Decrypt, while §2.101 names Encrypt only. The linked §2.100 XML has no Decrypt item, and the §2.101 XML does contain four Decrypt pairs. KMIPKIT-DISC-046 records this mismatch between case descriptions and executable fixture items. Tests use only messages actually present in each XML file. Filter to Encrypt/Decrypt items before validating substitutions: those selected messages use $NOW, $UNIQUE_IDENTIFIER_0, and $CORRELATION_VALUE; other setup/cleanup operations include symbols such as $NOW-3600 and are outside this adapter's validation scope. The selected-item adapter fails on unrecognized symbols or skipped in-scope pairs. Extracted item evidence is not a complete official-case pass.

| Catalog test ID | Official case | Mapping | Evidence status |
| --- | --- | --- | --- |
| KMIPKIT-TEST-CN01-2-99 | TC-STREAM-ENC-1-21, §2.99 | 10 Encrypt pairs; workflow also contains Create, Revoke, and Destroy | Pinned at `specification/oasis/kmip-2.1/fixtures/TC-STREAM-ENC-1-21.xml`; test the in-scope item as fixture-derived evidence, not a complete case pass |
| KMIPKIT-TEST-CN01-2-100 | TC-STREAM-ENC-2-21, §2.100 | HTML describes Encrypt and Decrypt; linked XML contains 10 Encrypt pairs only; workflow also contains Register, Revoke, and Destroy | Pinned at `specification/oasis/kmip-2.1/fixtures/TC-STREAM-ENC-2-21.xml`; test only the Encrypt items present as fixture-derived evidence, not a complete case pass |
| KMIPKIT-TEST-CN01-2-101 | TC-STREAM-ENCDEC-1-21, §2.101 | HTML describes Encrypt; linked XML contains 4 Encrypt and 4 Decrypt pairs, plus Register, Revoke, and Destroy | Pinned at `specification/oasis/kmip-2.1/fixtures/TC-STREAM-ENCDEC-1-21.xml`; test the in-scope items present as fixture-derived evidence, not a complete case pass |

## Feature requirement and success-criteria coverage

| Feature requirement / criterion | Planned tasks |
| --- | --- |
| FR-001 Typed Encrypt/Decrypt models and Data encodings | T007–T025, T034–T040 |
| FR-002 Request fields and eligible ID Placeholder use | T015–T023, T028, T031 |
| FR-003 Cryptographic Parameters preservation and recognized conditions | T009, T012, T015–T024 |
| FR-004 Multipart indicators, Data matrix, and DISC-045 local gate | T026–T033 |
| FR-005 Success payload shapes and optional response Data | T017, T019–T025, T035, T039 |
| FR-006 Initial-part AAD and Decrypt Tag placement | T016, T026–T032 |
| FR-007 Completed failures, unknown Result Reasons, and Pending | T018, T022, T035, T039 |
| FR-008 Single-exchange behavior, delivery state, and no retry/poll | T034–T041 |
| FR-009 Redaction and zeroization | T008, T011, T036, T042 |
| FR-010 No local cryptography/Usage Limits allocation; explicit Recover sequence | T005, T038–T042, T044 |
| FR-011 Complete normative traceability | T001–T004, T043 |
| FR-012 Positive/negative tests and fixture-derived OASIS operation items | T015–T024, T034–T042, T045 |
| SC-001 Typed fields and exact round trips | T015–T025, T026–T033 |
| SC-002 Malformed/missing fields and safe diagnostics | T019, T026, T028, T035–T036 |
| SC-003 One exchange and outcome preservation | T034–T040 |
| SC-004 Diagnostic redaction and owned-byte zeroization | T008, T036, T042 |
| SC-005 Catalog and implementation traceability | T001–T004, T043 |
| SC-006 All 28 fixture-derived operation items; no complete-case claim | T015–T024, T045 |
| SC-007 Bilingual executable examples | T044 |

## Open source issues

- KMIPKIT-DISC-045 records the Data-omission conflict for a one-request Init=true/Final=true form. The typed client accepts the form when Data is present and gates only Data omission with a local validation error before transmission while the discrepancy remains open; no Data requiredness interpretation is selected.
- KMIPKIT-CLAUSE-SPEC-4.16-004 is excluded with rationale and no linked requirement because “REQUIRED” is the Table 59 column heading, not the Cryptographic Parameters row value; this is a catalog audit disposition, not an open OASIS discrepancy.
- KMIPKIT-DISC-036 remains open for the global fixture gap. The three fixtures assigned here are present locally; implementation tests cover only their in-scope Encrypt/Decrypt items. Register, Revoke, and Destroy items remain owned by their operation specifications; a complete scenario pass belongs to 1.0 interoperability evidence after those operations are supported.
