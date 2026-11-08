# Traceability: OASIS KMIP 2.0 Interoperability and Conformance (KMIPKIT-0070)

| Requirement | Description | Primary Verification Target |
|---|---|---|
| `REQ-070-001` | Normative Ingestion & Checksums | `specification/oasis/kmip-2.0/CHECKSUMS.sha256` |
| `REQ-070-002` | KMIP 2.0 Pre-Send Operation Gating | `crates/kmipkit-protocol/tests/version_gate_tests.rs` |
| `REQ-070-003` | KMIP 2.0 Attribute Mutation Policy | `crates/kmipkit-protocol/tests/version_gate_tests.rs` |
| `REQ-070-004` | Test Server 2.0 Mode | `crates/kmipkit-test-server/tests/server_version_negotiation_tests.rs` |
| `REQ-070-005` | Deterministic Client Conformance | `crates/kmipkit-client/tests/kmip20_client_tests.rs` |
| `REQ-070-006` | Commercial KMS Interoperability | `crates/kmipkit-client/tests/kmip20_interop_tests.rs` |
