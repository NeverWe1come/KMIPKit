# Traceability Matrix: KMIPKIT-0080 Alibaba Cloud KMS Bridge

| Requirement ID | OASIS Clause / Source | Component / Method | Test Reference |
|---|---|---|---|
| KMIPKIT-0080-FR-001 | KMIP 2.1 §4.1 | `AlibabaKmsProvider::provider_name` | `tests::test_alibaba_provider_direct_operations` |
| KMIPKIT-0080-FR-002 | Architecture ADR-0021 | Zero-SDK HTTP client | `tests::test_alibaba_proxy_server_end_to_end` |
| KMIPKIT-0080-FR-003 | Alibaba OpenAPI Spec | `AlibabaKmsProvider::apply_auth` | `tests::test_alibaba_provider_direct_operations` |
| KMIPKIT-0080-FR-004 | KMIP 2.1 §4.1 Create | `AlibabaKmsProvider::create_native_key` | `tests::test_alibaba_provider_direct_operations` |
| KMIPKIT-0080-FR-005 | KMIP 2.1 §4.2 Get | `AlibabaKmsProvider::get_key_material` | `tests::test_alibaba_provider_direct_operations` |
| KMIPKIT-0080-FR-006 | KMIP 2.1 §4.16 Encrypt/Decrypt | `AlibabaKmsProvider::encrypt`/`decrypt` | `tests::test_alibaba_provider_direct_operations` |
| KMIPKIT-0080-FR-007 | KMIP 2.1 §5.1 Attributes | `AlibabaKmsProvider::get_tags_and_metadata` | `tests::test_alibaba_provider_direct_operations` |
| KMIPKIT-0080-FR-008 | KMIP 2.1 §4.7 Destroy | `AlibabaKmsProvider::schedule_destruction` | `tests::test_alibaba_provider_direct_operations` |
