# KMIPKIT-0074: Azure Key Vault Provider Bridge

## 1. Context and Purpose
This specification defines the Azure Cloud Provider adapter for `kmipkit-proxy`. It implements the `CloudKmsProvider` trait against Azure Key Vault Keys (for native HSM/software RSA/EC keys) and Azure Key Vault Secrets (for exportable auto-enveloped symmetric keys).

## 2. Normative References
- OASIS KMIP 2.1 Specification, §3 (Cryptographic Objects), §4 (Attributes), §6 (Operations)
- OASIS KMIP 2.0 Specification
- OASIS KMIP 3.0 Draft Specification
- ADR-0023: Stateless Zero-DB Cloud Proxy Bridge
- KMIPKIT-0072: Cloud Proxy Foundation Specification

## 3. Requirements
- **KMIPKIT-0074-FR-001**: Implement `CloudKmsProvider` for `AzureKeyVaultProvider` backed by Azure Key Vault REST API / SDK.
- **KMIPKIT-0074-FR-002**: Support Native Key Vault key operations (Create, Encrypt, Decrypt, Sign, Verify, GetPublicKey, DeleteKey).
- **KMIPKIT-0074-FR-003**: Support Auto-Enveloped exportable symmetric keys stored in Azure Key Vault Secrets using Base32Hex Cloud Tag-Stuffing for metadata.
- **KMIPKIT-0074-FR-004**: Provide configurable vault URL and endpoint override support for Lowkey Vault / Azure Stack / custom endpoints.
- **KMIPKIT-0074-FR-005**: Provide end-to-end integration tests in `.github/workflows/proxy-azure.yml`.
