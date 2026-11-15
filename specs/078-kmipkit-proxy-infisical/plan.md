# Implementation Plan: KMIPKit Proxy Infisical KMS Bridge

## Architecture Design
- Connector module: `proxy/kmipkit-proxy/src/providers/infisical.rs`.
- Configuration: `InfisicalConfig` in `proxy/kmipkit-proxy/src/config.rs`.
- Target API: Infisical KMS `/api/v1/kms/keys/*` and Universal Auth `/api/v1/auth/universal-auth/login`.
- Memory Safety: `zeroize::Zeroizing<Vec<u8>>` for sensitive payload handling.
- Zero-SDK Bloat: Use existing `reqwest` HTTP client with `aws-lc-rs` and `serde_json`.

## Phased Execution
1. Red: Add failing unit and integration tests simulating Infisical KMS endpoints.
2. Green: Implement `InfisicalKmsProvider` satisfying all functional and security requirements.
3. Refactor: Code cleanup, documentation contracts, traceability, and CI integration.
