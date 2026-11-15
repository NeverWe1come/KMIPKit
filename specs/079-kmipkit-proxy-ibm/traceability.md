# Traceability Matrix: KMIPKIT-0079 IBM Cloud Key Protect Bridge

| Requirement ID | OASIS Clause / Source | Component / Method | Test Reference |
|---|---|---|---|
| KMIPKIT-0079-FR-001 | KMIP 2.1 §4.1 | `IbmKmsProvider::provider_name` | `tests::test_ibm_provider_direct_operations` |
| KMIPKIT-0079-FR-002 | Architecture ADR-0021 | Zero-SDK HTTP client | `tests::test_ibm_proxy_server_end_to_end` |
| KMIPKIT-0079-FR-003 | IBM IAM Spec | `IbmKmsProvider::get_token` | `tests::test_ibm_provider_direct_operations` |
| KMIPKIT-0079-FR-004 | IBM Key Protect API | `IbmKmsProvider::client_headers` | `tests::test_ibm_proxy_server_end_to_end` |
| KMIPKIT-0079-FR-005 | KMIP 2.1 §4.1 Create | `IbmKmsProvider::create_key` | `tests::test_ibm_provider_direct_operations` |
| KMIPKIT-0079-FR-006 | KMIP 2.1 §4.2 Get | `IbmKmsProvider::get_key` | `tests::test_ibm_proxy_server_end_to_end` |
| KMIPKIT-0079-FR-007 | KMIP 2.1 §4.16 Encrypt/Decrypt | `IbmKmsProvider::crypto_op` | `tests::test_ibm_provider_direct_operations` |
| KMIPKIT-0079-FR-008 | KMIP 2.1 §5.1 Attributes | `IbmKmsProvider::encode_tags` | `tests::test_ibm_provider_direct_operations` |
| KMIPKIT-0079-FR-009 | KMIP 2.1 §4.7 Destroy | `IbmKmsProvider::destroy_key` | `tests::test_ibm_provider_direct_operations` |
