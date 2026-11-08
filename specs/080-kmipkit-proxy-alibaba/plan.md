# Implementation Plan: KMIPKIT-0080 Alibaba Cloud KMS Bridge

## Architecture
1. **Zero-SDK Direct HTTP**:
   - Reuses `reqwest` and `aws-lc-rs` with TLS 1.3.
   - Endpoint: `https://kms.{region_id}.aliyuncs.com`.
2. **Authentication**:
   - Computes HMAC-SHA256 signature using `aws-lc-rs::hmac`.
   - In-memory token support if Bearer token configured.
3. **Key Lifecycle**:
   - Creates native keys via `CreateKey` action.
   - Creates envelope-wrapped secret keys via `CreateSecret` action.
   - Remote cryptographic operations via `Encrypt` and `Decrypt`.
   - Destruction via `ScheduleKeyDeletion` or `DeleteSecret`.
