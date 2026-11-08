# Feature Specification: KMIPKit Proxy Google Cloud KMS and Secret Manager Bridge

## Document Metadata
- Specification ID: `KMIPKIT-0075`
- Title: KMIPKit Proxy Google Cloud KMS and Secret Manager Provider Bridge
- Status: Draft
- Author: AI Pair Programmer & Valdez
- References:
  - OASIS KMIP 2.0: Section 3, 4, 6
  - OASIS KMIP 2.1: Section 3, 4, 6
  - OASIS KMIP 3.0 (Draft): Section 3, 4, 6
  - ADR-0023: Stateless Zero-DB Cloud Proxy Architecture
  - KMIPKIT-0072: KMIPKit Cloud Proxy Foundation

## 1. Overview and Scope
This specification defines the Google Cloud KMS and Google Cloud Secret Manager backend provider connector for `kmipkit-proxy`. It implements the stateless, zero-database architecture defined in ADR-0023:
1. **Native Asymmetric Key Operations**: Backed by Google Cloud KMS `CryptoKeys` (purpose: `ASYMMETRIC_SIGN` and `ASYMMETRIC_DECRYPT`, supporting RSA-2048/3072/4096 and EC P-256/P-384).
2. **Auto-Envelope Symmetric Key Operations**: AES and symmetric key material is wrapped using AES-256-GCM Master KEK and stored in Google Cloud Secret Manager payloads.
3. **Stateless Cloud Tag-Stuffing**: KMIP object attributes (`State`, `CryptographicUsageMask`, `ActivationDate`, etc.) are serialized using `TagCodec` into GCP labels.
4. **Zero-DB Plaintext Key Protection**: Secret key material is held exclusively in `zeroize::Zeroizing<Vec<u8>>` and zeroized on drop.

## 2. Functional Requirements
- **KMIPKIT-0075-FR-001**: Implement `CloudKmsProvider` for Google Cloud in `GcpKmsProvider`.
- **KMIPKIT-0075-FR-002**: Support Google Cloud KMS `CryptoKey` creation, asymmetric decryption, asymmetric signing, disabling, and scheduling destruction.
- **KMIPKIT-0075-FR-003**: Support Google Cloud Secret Manager secret creation, versioning, accessing, label updates, and deletion for envelope symmetric keys.
- **KMIPKIT-0075-FR-004**: Implement label translation adhering to GCP label constraints (lowercase letters, numbers, underscores, hyphens, <=63 chars).
- **KMIPKIT-0075-FR-005**: Support custom `endpoint_url` for testing and local emulators.

## 3. Security Requirements
- **KMIPKIT-0075-SR-001**: Plaintext symmetric keys must never be logged or formatted; they must be zeroized using `Zeroizing<Vec<u8>>`.
- **KMIPKIT-0075-SR-002**: Transport security must enforce TLS 1.3 for KMIP endpoints.
