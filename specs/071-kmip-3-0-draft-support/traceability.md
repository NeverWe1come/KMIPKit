# Traceability: OASIS KMIP 3.0 Working Draft Implementation (KMIPKIT-0071)

| Requirement | Description | Primary Verification Target |
|---|---|---|
| `REQ-071-001` | Normative Ingestion & Checksums | `specification/oasis/kmip-3.0/CHECKSUMS.sha256` |
| `REQ-071-002` | Link Normalizer | `crates/kmipkit-protocol/tests/unit/link_normalizer_tests.rs` |
| `REQ-071-003` | Exact KMIP 3.0 Operation Enumeration assignments (§11.37, Table 583); unassigned codes receive no inferred span | `crates/kmipkit-protocol/tests/unit/kmip30_operations_tests.rs` |
| `REQ-071-004` | KMIP 3.0 Object Class and Object Type assignments (§§11.34–11.35, Tables 581–582) | `crates/kmipkit-protocol/tests/unit/kmip30_operations_tests.rs` |
| `REQ-071-005` | PQC & KEM Models | `crates/kmipkit-protocol/tests/unit/kem_tests.rs` |
| `REQ-071-006` | Test Server KMIP 3.0 Mode | `crates/kmipkit-test-server/tests/server_version_negotiation_tests.rs` |
