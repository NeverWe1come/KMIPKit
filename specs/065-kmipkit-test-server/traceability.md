# Traceability Matrix: KMIPKIT-0065

| Requirement | Tasks | Verification |
| :--- | :--- | :--- |
| `REQ-065-001` (Auto-PKI) | `TASK-065-002` | `pki::tests::test_auto_pki_generation_and_export` |
| `REQ-065-002` (mTLS & HTTPS) | `TASK-065-007` | `tests/client_integration.rs` |
| `REQ-065-003` (Multi-Version 1.0..2.1) | `TASK-065-004` | `v1_compat::tests::test_attribute_normalization` |
| `REQ-065-004` (JSON File Storage) | `TASK-065-003` | `store::tests::test_json_file_store_crud` |
| `REQ-065-005` (Realistic Crypto) | `TASK-065-006` | `ops::tests::test_crypto_operations` |
| `REQ-065-006` (Standalone CLI) | `TASK-065-008` | `tests/cli_tests.rs` |
