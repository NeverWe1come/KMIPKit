# KMIPKIT-0079: KMIPKit Proxy IBM Cloud Key Protect Provider Bridge

## 1. Overview and Scope

This specification defines the normative requirements and design for the IBM Cloud Key Protect and Hyper Protect Crypto Services provider bridge for `kmipkit-proxy`.
The bridge enables transparent bridging of OASIS KMIP 2.1 protocol operations to IBM Cloud Key Protect instances using direct REST HTTP communication.

### Normative References
- OASIS KMIP 2.1 Standard (§4.x Cryptographic Operations, §5.x Attributes).
- IBM Cloud Key Protect API v2 specification (`/api/v2/keys`).
- IBM Cloud IAM Identity Services API (`/identity/token`).
- IBM Cloud Global Tagging v3 API (`/v3/tags/attach`).

## 2. Functional Requirements

- **KMIPKIT-0079-FR-001**: Provider identification must be `"ibm"`.
- **KMIPKIT-0079-FR-002**: Direct REST HTTP communication must be used via `reqwest`, without external IBM SDK dependencies.
- **KMIPKIT-0079-FR-003**: The provider must authenticate via IBM Cloud IAM (`https://iam.cloud.ibm.com/identity/token`) using an API key (`grant_type=urn:ibm:params:oauth:grant-type:apikey&apikey={api_key}`) and cache the resulting Bearer token until expiry.
- **KMIPKIT-0079-FR-004**: All requests to Key Protect must include `Authorization: Bearer <token>` and `Bluemix-Instance: <instance_id>`.
- **KMIPKIT-0079-FR-005**: Support creation of Standard Keys (`extractable: true`) for direct export of symmetric and asymmetric key material, and Root Keys (`extractable: false`) for envelope encryption.
- **KMIPKIT-0079-FR-006**: Support direct key retrieval via `GET /api/v2/keys/{id}` returning payload in `Zeroizing<Vec<u8>>`.
- **KMIPKIT-0079-FR-007**: Support envelope key wrapping and unwrapping via `/api/v2/keys/{id}/actions/wrap` and `/api/v2/keys/{id}/actions/unwrap`.
- **KMIPKIT-0079-FR-008**: Support metadata persistence and custom KMIP attribute tag stuffing in key `description` and optional IBM Global Tagging v3.
- **KMIPKIT-0079-FR-009**: Support key deletion via `DELETE /api/v2/keys/{id}` and key listing via `GET /api/v2/keys`.
