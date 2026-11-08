# Traceability Matrix: Multi-Version Modern KMIP Foundation (KMIPKIT-0069)

| Requirement ID | Description | Source Reference | Design Component | Verification |
|---|---|---|---|---|
| `REQ-069-001` | Zero-Breakage Default (KMIP 2.1) | OASIS KMIP 2.1 §8, ADR-0022 | `ClientConfiguration::default` | Unit test: `test_zero_breakage_default_version` |
| `REQ-069-002` | Client Configuration Version Selection | OASIS KMIP 2.1 §6.1.16, ADR-0022 | `ClientConfiguration::with_protocol_version` | Unit test: `test_client_configuration_version` |
| `REQ-069-003` | Per-Request / Per-Batch Version Override | OASIS KMIP 2.1 §8, ADR-0022 | `RequestOptions`, `ClientBatch` | Unit test: `test_request_options_version_override` |
| `REQ-069-004` | Strict Pre-Send VersionGate Enforcement | OASIS KMIP 2.0/2.1/3.0, AGENTS.md §8 | `VersionGate` | Unit test: `test_version_gate_rejection` |
| `REQ-069-005` | Non-Catalogued Version Rejection | ADR-0022, KMIPKit Constitution | `ProtocolVersion::is_catalogued` | Unit test: `test_non_catalogued_version_rejection` |
| `REQ-069-006` | Multi-Version Discover Versions | OASIS KMIP 2.1 §6.1.16 | `DiscoverVersionsRequest::with_versions` | Unit test: `test_multi_version_discover_versions` |
| `REQ-069-007` | Four-Language Parity | Constitution Principle III | FFI, Java, Python | Cross-language parity tests |
