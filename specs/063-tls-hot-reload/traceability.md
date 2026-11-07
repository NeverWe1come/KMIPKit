# Traceability: File-backed TLS Hot-Reload (KMIPKIT-0063)

| Requirement | Source / Policy | Implementation Target | Verification Test |
|---|---|---|---|
| `KMIPKIT-063-REQ-001` (Retain & Redact File Descriptors) | ADR-0019; AGENTS.md §8 | `crates/kmipkit-transport/src/config.rs` | `tests/tls_config.rs` |
| `KMIPKIT-063-REQ-002` (On-demand File Refresh) | ADR-0019 | `crates/kmipkit-transport/src/tls.rs` | `tests/raw_tls.rs`, `tests/https.rs` |
| `KMIPKIT-063-REQ-003` (Atomic Validation & CertifiedKey check) | ADR-0005, ADR-0019 | `crates/kmipkit-transport/src/tls.rs` | `tests/raw_tls.rs` |
| `KMIPKIT-063-REQ-004` (Resumption & Keep-Alive Invalidation) | ADR-0005, ADR-0015, ADR-0019 | `crates/kmipkit-transport/src/raw_tls.rs`, `src/https.rs` | `tests/unit/tls_session_store_tests.rs`, `tests/https.rs` |
| `KMIPKIT-063-REQ-005` (Fail-Closed on Desync / Partial Write) | ADR-0014, ADR-0019 | `crates/kmipkit-transport/src/tls.rs` | `tests/raw_tls.rs` |
| `KMIPKIT-063-REQ-006` (Last-Known-Good for Deleted Temporaries) | ADR-0019 | `crates/kmipkit-transport/src/tls.rs` | `tests/support/secret_redaction/file_sources.rs` |
| `KMIPKIT-063-REQ-007` (Zeroize Replaced Private Keys & Redact Errors) | AGENTS.md §8, ADR-0019 | `crates/kmipkit-transport/src/secret.rs`, `src/config.rs` | `tests/support/secret_redaction/file_sources.rs` |
| `KMIPKIT-063-REQ-008` (Zero Public API Changes) | ADR-0006, AGENTS.md §9 | `specification/api/public-api.json` | `tools/api_manifest/generate.py --check` |
