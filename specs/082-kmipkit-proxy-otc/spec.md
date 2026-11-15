# Specification: KMIPKit Proxy - Open Telekom Cloud KMS Provider Bridge

## 1. Overview and Scope
This specification defines the Open Telekom Cloud (OTC) KMS / DEW (Data Encryption Workshop), TMS (Tag Management Service), and CSMS (Cloud Secret Management Service) backend adapter for `kmipkit-proxy` (traceability ID: `KMIPKIT-0082`).
It maps OASIS KMIP 2.1 operations to OTC's native REST APIs running in European Sovereign regions (`eu-de`, `eu-nl`).

## 2. Normative References
- OASIS KMIP 2.1 Specification Sections 4, 6.1 (Create, Get, Encrypt, Decrypt, Sign, Verify, Activate, Revoke, Destroy, Locate)
- Open Telekom Cloud Key Management Service (KMS) API v1.0 Documentation
- Open Telekom Cloud Cloud Secret Management Service (CSMS) API v1 Documentation
- Open Telekom Cloud Tag Management Service (TMS) API Documentation

## 3. Requirements and Architecture
- **KMIPKIT-0082-FR-001**: Provider identification as `"otc"` with configuration for `region`, `project_id`, `endpoint_url`, `envelope_master_kek_id`, `item_prefix`, and `auth_token`.
- **KMIPKIT-0082-FR-002**: Direct REST HTTP communication via `reqwest` and `aws-lc-rs` TLS 1.3 without external SDK dependencies.
- **KMIPKIT-0082-FR-003**: OTC KMS native key lifecycle (`create-key`, `describe-key`, `enable-key`, `disable-key`, `schedule-key-deletion`, `encrypt-data`, `decrypt-data`, `create-datakey`).
- **KMIPKIT-0082-FR-004**: OTC CSMS secret lifecycle for envelope-wrapped symmetric keys and secrets (`/v1/{project_id}/secrets` create, versioning, access, delete).
- **KMIPKIT-0082-FR-005**: OTC TMS tag management and filtering via `/v1.0/{project_id}/kms/{key_id}/tags` and `/resource_instances/action`.
- **KMIPKIT-0082-FR-006**: Stateless metadata tracking using native tags for custom KMIP attributes and activation state.
- **KMIPKIT-0082-FR-007**: Strict secret zeroization via `zeroize::Zeroizing<Vec<u8>>` and prevention of sensitive identifiers in URL interpolation (preventing CodeQL `rust/cleartext-transmission`).
