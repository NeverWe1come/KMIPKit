# KMIPKIT-0073: Architecture and Implementation Plan

## Architectural Design
- `AwsKmsProvider` encapsulates both `aws_sdk_kms::Client` and `aws_sdk_secretsmanager::Client`.
- For Native KMS keys, the KMIP `UniqueIdentifier` corresponds to the KMS KeyId or Key ARN.
- For Exportable Auto-Envelope keys, the KMIP `UniqueIdentifier` corresponds to the Secrets Manager Secret Name or ARN.
- All key material is handled exclusively in `Zeroizing<Vec<u8>>` and never formatted or logged.
