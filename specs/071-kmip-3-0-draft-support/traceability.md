# Traceability: OASIS KMIP 3.0 Working Draft Implementation (KMIPKIT-0071)

| Requirement | Description | Primary Verification Target |
|---|---|---|
| `REQ-071-001` | Normative Ingestion & Checksums | `specification/oasis/kmip-3.0/CHECKSUMS.sha256` |
| `REQ-071-002` | Link Normalizer | `crates/kmipkit-protocol/tests/unit/link_normalizer_tests.rs` |
| `REQ-071-003` | KMIP 3.0 Operations in VersionGate | `crates/kmipkit-protocol/tests/unit/kmip30_operations_tests.rs` |
| `REQ-071-004` | System Objects and Object Class | `crates/kmipkit-protocol/tests/unit/kmip30_system_objects_tests.rs` |
| `REQ-071-005` | PQC & KEM Models | `crates/kmipkit-protocol/tests/unit/kem_tests.rs` |
| `REQ-071-006` | Test Server KMIP 3.0 Mode | `crates/kmipkit-test-server/tests/server_version_negotiation_tests.rs` |
