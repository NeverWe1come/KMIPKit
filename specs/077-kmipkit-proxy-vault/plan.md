# Implementation Plan: KMIPKit Proxy HashiCorp Vault and OpenBao Bridge

## Architecture Design
- Connector module: `proxy/kmipkit-proxy/src/providers/vault.rs`.
- Configuration: `VaultConfig` in `proxy/kmipkit-proxy/src/config.rs`.
- Target API: HashiCorp Vault / OpenBao `/v1/{transit_mount}/*` and `/v1/{kv_v2_mount}/*`.
- Memory Safety: `zeroize::Zeroizing<Vec<u8>>` for sensitive payload handling.
- Zero-SDK Bloat: Use existing `reqwest` HTTP client with `aws-lc-rs` and `serde_json`.

## Phased Execution
1. Red: Add failing unit and integration tests simulating Vault Transit & KV v2 endpoints.
2. Green: Implement `VaultKmsProvider` and `VaultConfig` satisfying all functional and security requirements.
3. Refactor: Code cleanup, documentation contracts, traceability, and CI integration.
