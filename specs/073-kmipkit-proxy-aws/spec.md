# KMIPKIT-0073: AWS KMS and Secrets Manager Provider Bridge

## 1. Context and Purpose
This specification defines the AWS Cloud Provider adapter for `kmipkit-proxy`. It implements the `CloudKmsProvider` trait against AWS KMS (native HSM/software keys) and AWS Secrets Manager (for exportable auto-enveloped symmetric keys).

## 2. Normative References
- OASIS KMIP 2.1 Specification, §3 (Cryptographic Objects), §4 (Attributes), §6 (Operations)
- OASIS KMIP 2.0 Specification
- OASIS KMIP 3.0 Draft Specification
- ADR-0023: Stateless Zero-DB Cloud Proxy Bridge
- KMIPKIT-0072: Cloud Proxy Foundation Specification

## 3. Requirements
- **KMIPKIT-0073-FR-001**: Implement `CloudKmsProvider` for `AwsKmsProvider` backed by `aws-sdk-kms` and `aws-sdk-secretsmanager`.
- **KMIPKIT-0073-FR-002**: Support Native KMS key operations (Create, Encrypt, Decrypt, Sign, Verify, GetPublicKey, ScheduleKeyDeletion).
- **KMIPKIT-0073-FR-003**: Support Auto-Enveloped exportable symmetric keys stored in AWS Secrets Manager using Base32Hex Cloud Tag-Stuffing for metadata.
- **KMIPKIT-0073-FR-004**: Provide configurable endpoint URL support for LocalStack and custom VPC endpoints.
- **KMIPKIT-0073-FR-005**: Provide end-to-end integration tests running against LocalStack KMS and Secrets Manager in `.github/workflows/proxy-aws.yml`.
