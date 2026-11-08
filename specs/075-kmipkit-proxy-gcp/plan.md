# Implementation Plan: KMIPKit Proxy Google Cloud KMS and Secret Manager Bridge

## Summary
Implements `GcpKmsProvider` within `proxy/kmipkit-proxy/src/providers/gcp.rs` connecting KMIP client requests with Google Cloud KMS and Google Cloud Secret Manager.

## Key Components
1. **`GcpKmsProvider`**: Implements `CloudKmsProvider` for Google Cloud KMS and Google Cloud Secret Manager.
2. **`GcpConfig`**: Configures project ID, location, key ring, endpoint URL, secret name prefix, and envelope master KEK.
3. **`azure_integration_tests.rs` / `gcp_integration_tests.rs`**: Complete test harness for mock and live GCP testing.
4. **CI Workflow**: `.github/workflows/proxy-gcp.yml` for isolated CI execution.
