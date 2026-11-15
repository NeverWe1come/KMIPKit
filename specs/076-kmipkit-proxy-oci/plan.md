# Implementation Plan: KMIPKit Proxy Oracle OCI Vault and Secrets Bridge

## Summary
Implements `OciVaultProvider` within `proxy/kmipkit-proxy/src/providers/oci.rs` connecting KMIP client requests with Oracle Cloud Infrastructure (OCI) KMS Vault and OCI Secrets, plus container and Helm deployment artifacts.

## Key Components
1. **`OciVaultProvider`**: Implements `CloudKmsProvider` for OCI KMS / Vault and OCI Secrets.
2. **`OciConfig`**: Configures compartment ID, vault ID, endpoints, and envelope master KEK.
3. **`oci_integration_tests.rs`**: Integration test harness for mock and live OCI testing.
4. **CI Workflow**: `.github/workflows/proxy-oci.yml` for isolated CI execution.
5. **Deployment**: `deploy/Dockerfile` and `deploy/helm/kmipkit-proxy/` chart.
