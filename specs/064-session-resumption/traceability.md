# Traceability: TLS 1.3 Session Resumption and HTTPS Keep-Alive Reconnect (KMIPKIT-0064)

| Requirement ID | Specification Clause | Source / Authority | Test Verification | Implementation Target |
|---|---|---|---|---|
| `REQ-064-001` | Raw TLS Session Resumption | RFC 8446 §2.2, ADR-0020 | `tests/raw_tls.rs::consecutive_raw_tls_exchanges_resume_tls13_session` | `crates/kmipkit-transport/src/raw_tls.rs` |
| `REQ-064-002` | 0-RTT Early Data Disabled | `AGENTS.md` §8, ADR-0020 | `tests/unit/tls_policy_tests.rs::client_config_disables_early_data` | `crates/kmipkit-transport/src/tls.rs` |
| `REQ-064-003` | Session Ticket Limits | ADR-0005, ADR-0020 | `tests/unit/tls_policy_tests.rs::tls13_resumption_uses_new_connections_and_is_bounded_to_sixteen_tickets` | `crates/kmipkit-transport/src/tls.rs` |
| `REQ-064-004` | HTTPS Transparent Reconnect on Idle Drop | RFC 9112 §9.3, ADR-0020 | `tests/https.rs::idle_closed_keepalive_connection_reconnects_transparently` | `crates/kmipkit-transport/src/https.rs` |
| `REQ-064-005` | Never Retry Once Dispatched | `AGENTS.md` §8 | `tests/https.rs::dispatched_request_never_retries_on_connection_failure` | `crates/kmipkit-transport/src/https.rs` |
