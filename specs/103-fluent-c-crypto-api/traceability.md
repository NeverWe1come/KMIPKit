# KMIPKIT-0103 Traceability

| Requirement | Source | Implementation | Verification |
|---|---|---|---|
| KMIPKIT-0103-FR-001 | KMIPKIT-0093-FR-004–011; KMIPKIT-0102-FR-001; OASIS KMIP v2.1 §§6.1.11, 6.1.17, 6.1.24, 6.1.32, 6.1.33, 6.1.55, 6.1.56 | `crates/kmipkit-ffi/src/extension_registry/client_abi/fluent_crypto.rs`; `bindings/c/include/kmipkit.h` generated declarations | Compiled C consumer constructs each operation and checks the existing operation enumeration |
| KMIPKIT-0103-FR-002 | OASIS KMIP v2.1 §4.16, Table 59; §§7.8, 7.9, 7.14, 7.17, 7.20, 7.38, 7.39; KMIPKIT-0102-FR-002 | Typed C builders map semantic fields to existing protocol request models | Request-shape assertions for fields, omissions, and multipart values |
| KMIPKIT-0103-FR-003 | KMIPKIT-0102-FR-004; OASIS KMIP v2.1 §§7.8, 7.9, 7.14, 7.17, 7.20, 7.38, 7.39; AGENTS.md Security invariants | `SecretBytes` copy helpers and redacted typed requests | Caller-buffer independence, exact payload fixture checks, and diagnostic redaction tests |
| KMIPKIT-0103-FR-004 | KMIPKIT-0093-FR-004–011; KMIPKIT-0102-FR-005; ADR-0012 and ADR-0018 | Existing `ClientRequest`, batch item, and `Client::execute_with_options` route | TLS fixture verifies one exchange, version rejection before send, and typed Success/Pending/Failure |
| KMIPKIT-0103-FR-005 | KMIPKIT-0091-FR-015–016; KMIPKIT-0102-FR-006; AGENTS.md §9 | Opaque handle validation, ABI versioned option records, and FFI status mapping | Null/wrong-kind/invalid-record tests and separate options/header tests |
| KMIPKIT-0103-FR-006 | AGENTS.md §§3, 9–11; repository API manifest contract | `specification/api/public-api.json`, generated outputs, C guide | Generator `--check`, C export audit, traceability contract, documentation CI |
