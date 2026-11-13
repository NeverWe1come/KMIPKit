# KMIPKIT-0114 Traceability

| Requirement | Implementation | Verification | Normative / project source |
|---|---|---|---|
| KMIPKIT-0114-FR-001 | C direct lifecycle helpers in `client_abi/fluent.rs` | Compiled C consumer and TLS fixture request decoding | KMIP 2.1 §§6.1.1, 6.1.4, 6.1.28, 6.1.42, 6.1.44; KMIPKIT-0018 |
| KMIPKIT-0114-FR-002 | Typed identifier, attribute-set, and revocation-reason conversion | Valid, invalid, and mismatched-handle tests | KMIP 2.1 §§6.1.28, 6.1.44; KMIPKIT-0018 |
| KMIPKIT-0114-FR-003 | Existing one-exchange `Client` methods and separate timeout variants | Fixture observes exactly one request; request-option separation test | KMIPKIT-0093-FR-004, FR-007, FR-011 |
| KMIPKIT-0114-FR-004 | Existing FFI status, response-owner, redaction, and delivery-state helpers | C ABI ownership and error-state tests | KMIPKIT-0007-FR-005, FR-006; KMIPKIT-0093-FR-011 |
| KMIPKIT-0114-FR-005 | Compiled C consumer and controlled TLS fixture | FFI integration suite and generated export audit | KMIPKIT-0093-FR-013 |

