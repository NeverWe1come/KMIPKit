# Traceability: KMIP 2.1 Managed-Object State Transitions

This final matrix records the applicable client requirements, their pinned KMIP
2.1 sources, implementation, and executable verification. Source-derived tests
are project conformance evidence; they are not official OASIS Test Cases unless
an exact catalog Test Cases identifier is cited.

| Catalog item | Normative source | Specification requirement | Implementation | Verification |
| --- | --- | --- | --- | --- |
| `KMIPKIT-ELEM-OP-C2S-ACTIVATE` | KMIP 2.1 §6.1.1 Tables 164–166 (request/response payloads: Tables 164–165); §4.58 Tables 145–146 define permitted Unique Identifier encodings; §11.56 Table 487 assigns tag 0x420094 to Unique Identifier | FR-001, FR-002, FR-003 | `crates/kmipkit-protocol/src/activate.rs`; `crates/kmipkit-protocol/src/lib.rs`; `crates/kmipkit-client/src/execute.rs`; `crates/kmipkit-client/src/lib.rs` | `crates/kmipkit-protocol/tests/unit/activate_operation_tests.rs`; `crates/kmipkit-client/tests/unit/activate_execution_tests.rs` |
| `KMIPKIT-CLAUSE-SPEC-6.1.1-001` (server-only) | KMIP 2.1 §6.1.1 | FR-003 | No client-side state effect; `crates/kmipkit-client/src/execute.rs` | `crates/kmipkit-client/tests/unit/activate_execution_tests.rs` verifies no local state simulation |
| `KMIPKIT-ELEM-OP-C2S-ARCHIVE` | KMIP 2.1 §6.1.4 Tables 173–175 (request/response payloads: Tables 173–174); §4.58 Tables 145–146 define permitted Unique Identifier encodings; §11.56 Table 487 assigns tag 0x420094 to Unique Identifier | FR-001, FR-002, FR-004 | `crates/kmipkit-protocol/src/archive.rs`; `crates/kmipkit-protocol/src/lib.rs`; `crates/kmipkit-client/src/execute.rs`; `crates/kmipkit-client/src/lib.rs` | `crates/kmipkit-protocol/tests/unit/archive_operation_tests.rs`; `crates/kmipkit-client/tests/unit/archive_execution_tests.rs` |
| `KMIPKIT-REQ-SPEC-6.1.4-001` | KMIP 2.1 §6.1.4 | FR-004 | `crates/kmipkit-protocol/src/archive.rs`; `crates/kmipkit-protocol/src/lib.rs`; `crates/kmipkit-client/src/execute.rs`; `crates/kmipkit-client/src/lib.rs` | `crates/kmipkit-protocol/tests/unit/archive_operation_tests.rs`; `crates/kmipkit-client/tests/unit/archive_execution_tests.rs` for optional Unique Identifier, omission, success, failure, and Pending |
| `KMIPKIT-ELEM-OP-C2S-DESTROY` | KMIP 2.1 §6.1.15 Tables 208–210 (request/response payloads: Tables 208–209); §4.58 Tables 145–146 define permitted Unique Identifier encodings; §11.56 Table 487 assigns tag 0x420094 to Unique Identifier | FR-001, FR-002, FR-003 | `crates/kmipkit-protocol/src/destroy.rs`; `crates/kmipkit-protocol/src/lib.rs`; `crates/kmipkit-client/src/execute.rs`; `crates/kmipkit-client/src/lib.rs` | `crates/kmipkit-protocol/tests/unit/destroy_operation_tests.rs`; `crates/kmipkit-client/tests/unit/destroy_execution_tests.rs` |
| `KMIPKIT-CLAUSE-SPEC-6.1.15-001` (server-only) | KMIP 2.1 §6.1.15 | FR-003 | No client-side state effect; `crates/kmipkit-client/src/execute.rs` | `crates/kmipkit-client/tests/unit/destroy_execution_tests.rs` verifies no local state simulation |
| `KMIPKIT-ELEM-OP-C2S-RECOVER` | KMIP 2.1 §6.1.42 Tables 288–290 (request/response payloads: Tables 288–289); §4.58 Tables 145–146 define permitted Unique Identifier encodings; §11.56 Table 487 assigns tag 0x420094 to Unique Identifier | FR-001, FR-002, FR-005, FR-006 | `crates/kmipkit-protocol/src/recover.rs`; `crates/kmipkit-protocol/src/lib.rs`; `crates/kmipkit-client/src/execute.rs`; `crates/kmipkit-client/src/lib.rs` | `crates/kmipkit-protocol/tests/unit/recover_operation_tests.rs`; `crates/kmipkit-client/tests/unit/recover_execution_tests.rs` |
| `KMIPKIT-REQ-SPEC-6.1.42-001-001` | KMIP 2.1 §6.1.42 | FR-005 | `crates/kmipkit-protocol/src/recover.rs`; `crates/kmipkit-protocol/src/lib.rs`; `crates/kmipkit-client/src/execute.rs`; existing KMIPKIT-0009 Pending/Poll contract | `crates/kmipkit-protocol/tests/unit/recover_operation_tests.rs`; `crates/kmipkit-client/tests/unit/recover_execution_tests.rs` for permitted Pending, exact correlation bytes, caller-controlled Poll, and no retry |
| `KMIPKIT-REQ-SPEC-6.1.42-001-002` | KMIP 2.1 §6.1.42 | FR-005 | `crates/kmipkit-protocol/src/lib.rs`; `crates/kmipkit-client/src/execute.rs`; caller-controlled Get remains a separate invocation | `crates/kmipkit-client/tests/unit/recover_execution_tests.rs` verifies Recover completion does not issue automatic Get |
| Common message, batch, and result model | KMIP 2.1 §§8.1–8.6, 9.1–9.2, 9.5–9.6, 9.12, 9.16, 9.19–9.21; KMIPKIT-0006/0007/0009 | FR-002, FR-005–FR-008 | `crates/kmipkit-client/src/execute.rs`; existing `ClientBatch`, `ClientBatchOutcome`, `Client::execute`, and transport error contracts | `crates/kmipkit-client/tests/unit/activate_execution_tests.rs`; `crates/kmipkit-client/tests/unit/destroy_execution_tests.rs`; `crates/kmipkit-client/tests/unit/archive_execution_tests.rs`; `crates/kmipkit-client/tests/unit/recover_execution_tests.rs` cover correlation, delivery state, malformed success, one exchange, and no automatic follow-up |
| Generic TTLV and secret handling | KMIP 2.1 §5.1; §11.56 Table 487 assigns tag 0x420094 to Unique Identifier (Tag Enumeration); KMIPKIT-0004/0005 and accepted security policy | FR-008, FR-010 | `crates/kmipkit-protocol/src/create.rs` (`UniqueIdentifier` Debug redaction); `crates/kmipkit-client/src/execute.rs`; bounded generic response and error handling | `crates/kmipkit-client/tests/unit/lifecycle_execution_tests.rs::lifecycle_results_preserve_unknown_status_reason_and_accepted_extension_values`; `crates/kmipkit-client/tests/unit/lifecycle_redaction_tests.rs::lifecycle_request_and_response_debug_redact_unique_identifiers`; `crates/kmipkit-client/tests/unit/lifecycle_redaction_tests.rs::lifecycle_errors_and_debug_never_include_result_text_or_raw_response_body` |
| Requirement traceability | KMIPKIT-0018-FR-009; repository normative-catalog rules | Every applicable client obligation; server-only clauses explicitly excluded from client behavior | This matrix; `specs/018-managed-object-lifecycle/spec.md`; `specs/018-managed-object-lifecycle/implementation-evidence.md` | `python -B -m unittest discover -s tools/normative_catalog/tests -p 'test_feature_traceability.py' -v`; source rows link operation-specific protocol and fake-transport tests above |

## Lifecycle support test modules

- `crates/kmipkit-client/tests/unit/lifecycle_execution_tests.rs`
- `crates/kmipkit-client/tests/unit/lifecycle_redaction_tests.rs`

## Official test evidence

The catalog does not link a requirement-specific official Test Cases ID to
`KMIPKIT-REQ-SPEC-6.1.4-001` or either Recover requirement. The source-derived
protocol and fake-transport tests cited above do not become official evidence.
The absence of official evidence remains explicit; this feature does not claim
formal OASIS certification or official test-case passes.

## Verification records

Red, Green, and Refactor commands and results are recorded in
[`implementation-evidence.md`](implementation-evidence.md). This matrix links
the lifecycle requirements to the actual implementation and focused tests; the
feature-wide and workspace checks are recorded there after T035 completes.
