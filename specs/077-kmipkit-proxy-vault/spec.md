# Feature Specification: KMIPKit Proxy HashiCorp Vault and OpenBao Bridge

## Document Metadata
- Specification ID: `KMIPKIT-0077`
- Title: KMIPKit Proxy HashiCorp Vault and OpenBao Provider Bridge
- Status: Draft
- Author: AI Pair Programmer & Valdez
- References:
  - OASIS KMIP 2.0: Section 3, 4, 6
  - OASIS KMIP 2.1: Section 3, 4, 6
  - OASIS KMIP 3.0 (Draft): Section 3, 4, 6
  - ADR-0023: Stateless Zero-DB Cloud Proxy Architecture
  - KMIPKIT-0072: KMIPKit Cloud Proxy Foundation
  - HashiCorp Vault Transit Secrets Engine HTTP API
  - HashiCorp Vault KV Secrets Engine v2 HTTP API
  - OpenBao Transit and KV v2 API Specifications

## 1. Overview and Scope
This specification defines the HashiCorp Vault (Community / Open-Source, HCP) and OpenBao backend provider connector for `kmipkit-proxy`. It implements the stateless, zero-database architecture defined in ADR-0023:
1. **Transit Engine Key Lifecycle & Cryptography**: Backed by Vault / OpenBao `/v1/{transit_mount}/keys/*` (AES-128/256-GCM, RSA-2048/3072/4096, ECDSA P-256/P-384, Ed25519, HMAC).
2. **Dual-Path Key Retrieval for `Get`**:
   - **Direct Exportable Transit Keys**: Keys created with `"exportable": true` retrieved securely via `/v1/{transit_mount}/export/{type}/{id}` directly into zeroized memory buffers.
   - **AutoEnvelope Wrapping**: For non-exportable environments, data keys are wrapped with `/v1/{transit_mount}/datakey/wrapped/{kek}` and stored in KV v2, unwrapped via `/v1/{transit_mount}/decrypt/{kek}` on demand.
3. **Stateless Attribute Tag-Stuffing in KV v2 `custom_metadata`**: KMIP object attributes are stored in `/v1/{kv_mount}/metadata/{kv_prefix}{id}` within the `custom_metadata` JSON object.
4. **Authentication Methods**: Support direct Token (`X-Vault-Token`), AppRole (`role_id` and secret ID), and Kubernetes ServiceAccount JWT authentication.
5. **Zero-DB Plaintext Key Protection**: Secret key material is held exclusively in `zeroize::Zeroizing<Vec<u8>>` and zeroized on drop.

## 2. Functional Requirements
- **KMIPKIT-0077-FR-001**: Implement `CloudKmsProvider` for HashiCorp Vault and OpenBao in `VaultKmsProvider`.
- **KMIPKIT-0077-FR-002**: Support Vault Transit key creation (symmetric & asymmetric), encryption, decryption, signing, verification, HMAC, rotation (`ReKey`), and deletion.
- **KMIPKIT-0077-FR-003**: Support direct exportable key retrieval (`exportable: true`) and AutoEnvelope fallback via KV v2 data storage and transit unwrapping.
- **KMIPKIT-0077-FR-004**: Implement tag translation and persistence adhering to Vault KV v2 `custom_metadata` constraints.
- **KMIPKIT-0077-FR-005**: Support Token, AppRole, and Kubernetes auth methods with optional `X-Vault-Namespace`.

## 3. Security Requirements
- **KMIPKIT-0077-SR-001**: Plaintext symmetric and private keys must never be logged or formatted; they must be zeroized using `Zeroizing<Vec<u8>>`.
- **KMIPKIT-0077-SR-002**: Transport security must enforce TLS 1.3 for KMIP endpoints and HTTPS/TLS for Vault REST communications.
