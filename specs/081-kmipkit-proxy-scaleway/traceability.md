# Traceability Matrix: KMIPKIT-0081 Scaleway Provider Bridge

| Requirement ID | Design Element | Verification Test |
|---|---|---|
| `KMIPKIT-0081-FR-001` | `src/config.rs: ScalewayConfig`, `src/providers/scaleway.rs` | `tests/scaleway_integration_tests.rs: test_scaleway_provider_creation` |
| `KMIPKIT-0081-FR-002` | `src/providers/scaleway.rs: ScalewayKmsProvider` | `tests/scaleway_integration_tests.rs: test_scaleway_key_lifecycle` |
| `KMIPKIT-0081-FR-003` | `src/providers/scaleway.rs: create_key, encrypt, decrypt` | `tests/scaleway_integration_tests.rs: test_scaleway_encrypt_decrypt` |
| `KMIPKIT-0081-FR-004` | `src/providers/scaleway.rs: create_secret, get_secret` | `tests/scaleway_integration_tests.rs: test_scaleway_secret_lifecycle` |
| `KMIPKIT-0081-FR-005` | `src/providers/scaleway.rs: parse_tags, format_tags` | `tests/scaleway_integration_tests.rs: test_scaleway_tags_roundtrip` |
| `KMIPKIT-0081-FR-006` | `src/providers/scaleway.rs: set_tag, get_meta` | `tests/scaleway_integration_tests.rs: test_scaleway_metadata_state` |
| `KMIPKIT-0081-FR-007` | `src/providers/scaleway.rs: Zeroizing` | `tests/scaleway_integration_tests.rs: test_scaleway_secret_zeroization` |
