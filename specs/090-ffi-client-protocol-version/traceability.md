# Requirement Traceability

| Requirement | Normative reference | Implementation | Verification |
|---|---|---|---|
| FR-090-001 | OASIS KMIP 2.0 §9.16, Table 383; KMIP 2.1 §9.16, Table 421 | `crates/kmipkit-ffi/src/extension_registry/client_abi.rs`: client configuration reconstruction in `kmipkit_client_create` | `ffi_client_preserves_configured_kmip_20_protocol_version_on_the_wire` in `crates/kmipkit-ffi/tests/support/client_abi_tests.rs`; controlled mTLS peer checks the emitted header |
| FR-090-002 | OASIS KMIP 2.0 §9.16, Table 383; KMIP 2.1 §9.16, Table 421 | Preserve protocol version together with the existing registry and encoding | Regression test plus existing FFI client configuration tests |
