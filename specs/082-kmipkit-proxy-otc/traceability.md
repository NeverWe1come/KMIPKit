# Traceability Matrix: KMIPKIT-0082 OTC Provider Bridge

| Requirement ID | Design Element | Verification Test |
|---|---|---|
| `KMIPKIT-0082-FR-001` | `src/config.rs: OtcConfig`, `src/providers/otc.rs` | `tests/otc_integration_tests.rs: test_otc_provider_creation` |
| `KMIPKIT-0082-FR-002` | `src/providers/otc.rs: OtcKmsProvider` | `tests/otc_integration_tests.rs: test_otc_key_lifecycle` |
| `KMIPKIT-0082-FR-003` | `src/providers/otc.rs: create_key, encrypt, decrypt` | `tests/otc_integration_tests.rs: test_otc_encrypt_decrypt` |
| `KMIPKIT-0082-FR-004` | `src/providers/otc.rs: create_secret, get_secret` | `tests/otc_integration_tests.rs: test_otc_secret_lifecycle` |
| `KMIPKIT-0082-FR-005` | `src/providers/otc.rs: parse_tags, format_tags` | `tests/otc_integration_tests.rs: test_otc_tags_roundtrip` |
| `KMIPKIT-0082-FR-006` | `src/providers/otc.rs: set_tag, get_meta` | `tests/otc_integration_tests.rs: test_otc_metadata_state` |
| `KMIPKIT-0082-FR-007` | `src/providers/otc.rs: Zeroizing` | `tests/otc_integration_tests.rs: test_otc_secret_zeroization` |
