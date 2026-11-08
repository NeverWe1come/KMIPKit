# Specification: KMIPKit Proxy - Scaleway Key Manager Provider Bridge

## 1. Overview and Scope
This specification defines the Scaleway Key Manager and Secret Manager backend adapter for `kmipkit-proxy` (traceability ID: `KMIPKIT-0081`).
It maps OASIS KMIP 2.1 operations to Scaleway's native REST APIs (`key-manager/v1alpha1` and `secret-manager/v1beta1`) located in European Sovereign regions (`fr-par`, `nl-ams`, `pl-waw`).

## 2. Normative References
- OASIS KMIP 2.1 Specification Sections 4, 6.1 (Create, Get, Encrypt, Decrypt, Sign, Verify, Activate, Revoke, Destroy, Locate)
- Scaleway Key Manager API v1alpha1 Documentation
- Scaleway Secret Manager API v1beta1 Documentation

## 3. Requirements and Architecture
- **KMIPKIT-0081-FR-001**: Provider identification as `"scaleway"` with configuration for `region`, `project_id`, `endpoint_url`, `envelope_master_kek_id`, and `auth_token`.
- **KMIPKIT-0081-FR-002**: Direct REST HTTP transport via `reqwest` and `aws-lc-rs` TLS 1.3 without adding third-party Scaleway SDKs.
- **KMIPKIT-0081-FR-003**: Scaleway Key Manager native key lifecycle (`Create`, `Encrypt`, `Decrypt`, `GenerateDataKey`, `Enable`, `Disable`, `Delete`, `List`).
- **KMIPKIT-0081-FR-004**: Scaleway Secret Manager integration for envelope-wrapped symmetric keys and secrets (`CreateSecret`, `CreateSecretVersion`, `AccessSecretVersion`, `DeleteSecret`).
- **KMIPKIT-0081-FR-005**: Native array tag conversion between `CloudTagMap` and Scaleway's `tags: ["key:value"]` format.
- **KMIPKIT-0081-FR-006**: Stateless metadata tracking using native tags for custom KMIP attributes and activation state.
- **KMIPKIT-0081-FR-007**: Strict secret zeroization via `zeroize::Zeroizing<Vec<u8>>` and prevention of sensitive credentials in logs, errors, or URL formatting.
