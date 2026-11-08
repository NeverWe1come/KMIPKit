# Implementation Plan: KMIPKIT-0079 IBM Cloud Key Protect Bridge

## Architecture
1. **Zero-SDK Direct HTTP**:
   - Reuses `reqwest` and `aws-lc-rs` with TLS 1.3.
   - Target endpoint: `https://{region}.kms.cloud.ibm.com`.
2. **Authentication**:
   - `IbmKmsProvider` exchanges API key against `https://iam.cloud.ibm.com/identity/token`.
   - In-memory token caching with thread-safe `tokio::sync::RwLock`.
3. **Key Lifecycle**:
   - Creates Standard Keys (`extractable: true`) for KMIP symmetric keys (AES-128/256) where client demands raw key bytes.
   - Creates Root Keys (`extractable: false`) when managed remotely.
   - Remote wrapping / unwrapping via Key Protect wrap actions.
   - Secure memory protection with `Zeroizing<Vec<u8>>`.
