# Feature Specification: KMIPKit Proxy Oracle OCI Vault and Secrets Bridge

## Document Metadata
- Specification ID: `KMIPKIT-0076`
- Title: KMIPKit Proxy Oracle OCI Vault and Secrets Provider Bridge & Cloud Deployment
- Status: Draft
- Author: AI Pair Programmer & Valdez
- References:
  - OASIS KMIP 2.0: Section 3, 4, 6
  - OASIS KMIP 2.1: Section 3, 4, 6
  - OASIS KMIP 3.0 (Draft): Section 3, 4, 6
  - ADR-0023: Stateless Zero-DB Cloud Proxy Architecture
  - KMIPKIT-0072: KMIPKit Cloud Proxy Foundation

## 1. Overview and Scope
This specification defines the Oracle Cloud Infrastructure (OCI) Vault and Secrets backend provider connector and production deployment artifacts for `kmipkit-proxy`. It implements the stateless, zero-database architecture defined in ADR-0023:
1. **Native Asymmetric Key Operations**: Backed by OCI KMS / Vault Master Encryption Keys (algorithm `RSA` 2048/3072/4096 or `ECDSA` P-256/P-384, supporting asymmetric sign and decrypt).
2. **Auto-Envelope Symmetric Key Operations**: AES and symmetric key material is wrapped using AES-256-GCM Master KEK and stored in OCI Secrets service bundles.
3. **Stateless Cloud Tag-Stuffing**: KMIP object attributes are encoded into OCI `freeformTags`.
4. **Zero-DB Plaintext Key Protection**: Secret key material is held exclusively in `zeroize::Zeroizing<Vec<u8>>` and zeroized on drop.
5. **Production Deployment**: Containerization (`Dockerfile`) and Kubernetes Helm chart (`deploy/helm/kmipkit-proxy`).

## 2. Functional Requirements
- **KMIPKIT-0076-FR-001**: Implement `CloudKmsProvider` for Oracle OCI in `OciVaultProvider`.
- **KMIPKIT-0076-FR-002**: Support OCI KMS Key creation, asymmetric decryption, asymmetric signing, disabling, and scheduling deletion.
- **KMIPKIT-0076-FR-003**: Support OCI Secrets secret creation, versioning, accessing, freeform tag updates, and deletion for envelope symmetric keys.
- **KMIPKIT-0076-FR-004**: Implement tag translation adhering to OCI `freeformTags` constraints.
- **KMIPKIT-0076-FR-005**: Provide multi-stage `Dockerfile` and production Kubernetes Helm chart in `deploy/`.

## 3. Security Requirements
- **KMIPKIT-0076-SR-001**: Plaintext symmetric keys must never be logged or formatted; they must be zeroized using `Zeroizing<Vec<u8>>`.
- **KMIPKIT-0076-SR-002**: Transport security must enforce TLS 1.3 for KMIP endpoints.
