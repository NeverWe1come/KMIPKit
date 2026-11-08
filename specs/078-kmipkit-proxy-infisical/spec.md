# Feature Specification: KMIPKit Proxy Infisical KMS Bridge

## Document Metadata
- Specification ID: `KMIPKIT-0078`
- Title: KMIPKit Proxy Infisical KMS Provider Bridge
- Status: Draft
- Author: AI Pair Programmer & Valdez
- References:
  - OASIS KMIP 2.0: Section 3, 4, 6
  - OASIS KMIP 2.1: Section 3, 4, 6
  - OASIS KMIP 3.0 (Draft): Section 3, 4, 6
  - ADR-0023: Stateless Zero-DB Cloud Proxy Architecture
  - KMIPKIT-0072: KMIPKit Cloud Proxy Foundation
  - Infisical KMS Platform Concepts & Security Architecture
  - Infisical REST API: `/api/v1/kms/keys/*` and Universal Auth

## 1. Overview and Scope
This specification defines the Infisical KMS (Cloud and Self-Hosted) backend provider connector for `kmipkit-proxy`. It implements the stateless, zero-database architecture defined in ADR-0023:
1. **Infisical KMS Lifecycle & Cryptography**: Backed by `/api/v1/kms/keys/*` (AES-128-GCM, AES-256-GCM, RSA-4096, ECC NIST P-256).
2. **Direct Key Material Export via `/private-key` & `/public-key`**:
   - Secret key material retrieved directly from `GET /api/v1/kms/keys/{keyId}/private-key` into `zeroize::Zeroizing<Vec<u8>>`.
   - Public keys retrieved via `GET /api/v1/kms/keys/{keyId}/public-key`.
3. **Stateless Attribute Tag-Stuffing in Key Description**: KMIP object attributes are encoded into compact semicolon-delimited key-value tags (`k=v;...`) stored in the key's `description` field (or project secret metadata).
4. **Universal Auth and Machine Identity**: Support Machine Identity Universal Auth (`/api/v1/auth/universal-auth/login` with `clientId` and `clientSecret`) and direct Bearer token.
5. **Zero-DB Plaintext Key Protection**: Secret key material is held exclusively in `zeroize::Zeroizing<Vec<u8>>` and zeroized on drop.

## 2. Functional Requirements
- **KMIPKIT-0078-FR-001**: Implement `CloudKmsProvider` for Infisical in `InfisicalKmsProvider`.
- **KMIPKIT-0078-FR-002**: Support Infisical KMS key creation (symmetric & asymmetric), encryption, decryption, signing, verification, rotation (`ReKey`), and deletion.
- **KMIPKIT-0078-FR-003**: Support direct key extraction via `/private-key` and `/public-key` into zeroized memory buffers.
- **KMIPKIT-0078-FR-004**: Implement tag translation and persistence adhering to Infisical key metadata constraints.
- **KMIPKIT-0078-FR-005**: Support Machine Identity Universal Auth and Bearer token authentication.

## 3. Security Requirements
- **KMIPKIT-0078-SR-001**: Plaintext symmetric and private keys must never be logged or formatted; they must be zeroized using `Zeroizing<Vec<u8>>`.
- **KMIPKIT-0078-SR-002**: Transport security must enforce TLS 1.3 for KMIP endpoints and HTTPS/TLS for Infisical REST communications.
