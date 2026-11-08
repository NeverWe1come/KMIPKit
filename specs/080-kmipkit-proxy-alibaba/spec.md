# KMIPKIT-0080: KMIPKit Proxy Alibaba Cloud KMS & Secrets Manager Provider Bridge

## 1. Overview and Scope

This specification defines the normative requirements and design for the Alibaba Cloud Key Management Service (KMS) and Secrets Manager provider bridge for `kmipkit-proxy`.
The bridge enables transparent bridging of OASIS KMIP 2.1 protocol operations to Alibaba Cloud KMS endpoints using direct REST HTTP communication.

### Normative References
- OASIS KMIP 2.1 Standard (§4.x Cryptographic Operations, §5.x Attributes).
- Alibaba Cloud KMS OpenAPI / ROA Specification (`kms.{region_id}.aliyuncs.com`).
- Alibaba Cloud Signature Protocol (RPC and ROA ACS3-HMAC-SHA256).

## 2. Functional Requirements

- **KMIPKIT-0080-FR-001**: Provider identification must be `"alibaba"`.
- **KMIPKIT-0080-FR-002**: Direct REST HTTP communication must be used via `reqwest`, without external Alibaba Cloud SDK dependencies.
- **KMIPKIT-0080-FR-003**: The provider must authenticate via Alibaba Cloud Signature Protocol (`x-acs-signature-method: HMAC-SHA256` or `Authorization: ACS3-HMAC-SHA256`) using AccessKey ID and Secret, or Bearer token header.
- **KMIPKIT-0080-FR-004**: Support key creation via `CreateKey` (AES-128/256, RSA, ECC) and secrets creation via `CreateSecret` with `AutoEnvelope`.
- **KMIPKIT-0080-FR-005**: Support key material extraction via `GetSecretValue` or direct export into `Zeroizing<Vec<u8>>`.
- **KMIPKIT-0080-FR-006**: Support remote cryptographic operations: `Encrypt`, `Decrypt`, `AsymmetricSign`, `AsymmetricVerify`.
- **KMIPKIT-0080-FR-007**: Support metadata persistence and custom KMIP attribute tag stuffing in `Description` or `TagResources`.
- **KMIPKIT-0080-FR-008**: Support key lifecycle management: `ScheduleKeyDeletion`, `EnableKey`, `DisableKey`, and `ListKeys`.
