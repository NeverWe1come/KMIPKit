# KMIPKIT-0074: Architecture and Implementation Plan

## Architectural Design
- `AzureKeyVaultProvider` connects to Azure Key Vault Keys and Secrets.
- For Native Key Vault keys, the KMIP `UniqueIdentifier` corresponds to the Key Vault Key Name or Key Identifier.
- For Exportable Auto-Envelope keys, the KMIP `UniqueIdentifier` corresponds to the Secret Name or Secret Identifier.
- All key material is handled exclusively in `Zeroizing<Vec<u8>>` and never logged or serialized into diagnostics.
- Metadata and attributes are stored directly in Azure Key Vault resource tags with Base32Hex encoding for non-alphanumeric tags.
