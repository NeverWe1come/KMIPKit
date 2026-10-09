# Traceability: KMIP 2.1 Managed-Object State Transitions

This matrix is a planning map. Implementation and verification references are added with their changes; draft assignment does not claim that code or tests already exist.

| Catalog item | Normative source | Specification requirement | Planned implementation | Planned verification |
| --- | --- | --- | --- | --- |
| `KMIPKIT-ELEM-OP-C2S-ACTIVATE` | KMIP 2.1 §6.1.1 Tables 164–166 | FR-001, FR-002, FR-003 | `crates/kmipkit-protocol/src/activate.rs`; client dispatch in `crates/kmipkit-client/src/execute.rs` | `activate_operation_tests.rs`; `activate_execution_tests.rs` |
| `KMIPKIT-CLAUSE-SPEC-6.1.1-001` (server-only) | KMIP 2.1 §6.1.1 | FR-003 | No client-side state effect | Verify it stays out of client requirement assignments; tests assert no local state simulation |
| `KMIPKIT-ELEM-OP-C2S-ARCHIVE` | KMIP 2.1 §6.1.4 Tables 173–175 | FR-001, FR-002, FR-004 | `crates/kmipkit-protocol/src/archive.rs`; client dispatch | `archive_operation_tests.rs`; `archive_execution_tests.rs` |
| `KMIPKIT-REQ-SPEC-6.1.4-001` | KMIP 2.1 §6.1.4 | FR-004 | Archive request model and client convenience | Test optional Unique Identifier, omission, success, failure, and Pending behavior |
| `KMIPKIT-ELEM-OP-C2S-DESTROY` | KMIP 2.1 §6.1.15 Tables 208–210 | FR-001, FR-002, FR-003 | `crates/kmipkit-protocol/src/destroy.rs`; client dispatch | `destroy_operation_tests.rs`; `destroy_execution_tests.rs` |
| `KMIPKIT-CLAUSE-SPEC-6.1.15-001` (server-only) | KMIP 2.1 §6.1.15 | FR-003 | No client-side state effect | Verify it stays out of client requirement assignments; tests assert no local state simulation |
| `KMIPKIT-ELEM-OP-C2S-RECOVER` | KMIP 2.1 §6.1.42 Tables 288–290 | FR-001, FR-002, FR-005, FR-006 | `crates/kmipkit-protocol/src/recover.rs`; client dispatch | `recover_operation_tests.rs`; `recover_execution_tests.rs` |
| `KMIPKIT-REQ-SPEC-6.1.42-001-001` | KMIP 2.1 §6.1.42 | FR-005 | Reuse KMIPKIT-0009 Pending/Poll contract | Pending only when permitted; exact correlation bytes; caller-controlled Poll; no retry |
| `KMIPKIT-REQ-SPEC-6.1.42-001-002` | KMIP 2.1 §6.1.42 | FR-005 | Keep Get as a separate caller invocation | Recover completion does not issue automatic Get |
| Common message, batch, and result model | KMIP 2.1 §§8.1–8.6, 9.1–9.2, 9.5–9.6, 9.12, 9.16, 9.19–9.21; KMIPKIT-0006/0007/0009 | FR-002, FR-005–FR-008 | Existing `ClientBatch`, `ClientBatchOutcome`, `Client::execute`, and transport error contracts | Fake-transport tests for correlation, delivery state, malformed success, no retry, and redaction |
| Generic TTLV and secret handling | KMIP 2.1 §§5.1, 11.56; KMIPKIT-0004/0005 and accepted security policy | FR-008, FR-010 | Existing generic `Item`, bounded codec, redacted errors | Extension/unknown preservation, configured limits, and redaction tests |

## Official test evidence

The catalog does not link a requirement-specific official Test Cases ID to `KMIPKIT-REQ-SPEC-6.1.4-001` or either Recover requirement. Add source-derived and negative tests, preserve the absence of official evidence in the generated report, and do not describe derived vectors as official OASIS test-case passes.
