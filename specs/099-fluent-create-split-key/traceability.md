# KMIPKIT-0099 Traceability

All Rust builder code will be in `crates/kmipkit-client/src/fluent.rs`; typed
request and response semantics remain in
`crates/kmipkit-protocol/src/create_split_key.rs`. Unit tests use the existing
controlled exchange support in
`crates/kmipkit-client/tests/unit/asynchronous_execution_test_support.rs`.

| Requirement | Source | Implementation and verification |
|---|---|---|
| KMIPKIT-0099-FR-001 | OASIS KMIP v2.1 §6.1.10 Tables 193–195; KMIPKIT-0093-FR-004 | `RequestBuilder::create_split_key`, `CreateSplitKeyBuilder`; `split_key_builder_requires_all_four_explicit_semantic_fields`, `split_key_builder_preserves_explicit_and_optional_fields_and_empty_attributes` |
| KMIPKIT-0099-FR-002 | OASIS KMIP v2.1 §6.1.10 Tables 193–195; `CreateSplitKeyRequest` in `crates/kmipkit-protocol/src/create_split_key.rs` | `split_key_builder_preserves_explicit_and_optional_fields_and_empty_attributes`; `prime_field_method_requires_explicit_size_before_transport`; the existing typed validator remains authoritative for Prime Field Size policy |
| KMIPKIT-0099-FR-003 | OASIS KMIP v2.1 §§4–5; KMIPKIT-0093-FR-006 | `attribute`, `attribute_item`, `attributes`, `vendor_attribute`, and `attribute_at`; `split_key_builder_preserves_typed_prebuilt_vendor_and_positioned_attributes`, `split_key_builder_rejects_malformed_attributes_duplicate_positions_and_extensions`, and `fluent_split_key_signature_exceptions_are_exact_and_method_scoped` |
| KMIPKIT-0099-FR-004 | KMIPKIT-0093-FR-004; existing `CreateSplitKeyRequest` typed validation | `validate` delegates to `build_request`; `split_key_builder_validate_is_non_consuming_and_batch_conversion_is_typed`, `prime_field_method_requires_explicit_size_before_transport`, `split_key_builder_rejects_duplicate_singleton_attributes_before_transport`, `split_key_builder_rejects_malformed_attributes_duplicate_positions_and_extensions`, and the complete production AST boundary inventory |
| KMIPKIT-0099-FR-005 | OASIS KMIP v2.1 §§6.1.10, 8–9; KMIPKIT-0093-FR-005, FR-008 | `build`, `build_request`, `to_batch_item`, `execute`; `split_key_builder_executes_one_typed_exchange_and_preserves_all_response_ids`, `split_key_builder_preserves_failure_and_pending_correlation_without_follow_up`, `split_key_builder_preserves_options_and_transport_delivery_state` |
| KMIPKIT-0099-FR-006 | ADR-0012, ADR-0022; KMIPKIT-0093-FR-007 | `split_key_builder_preserves_options_and_transport_delivery_state`, `split_key_builder_preserves_failure_and_pending_correlation_without_follow_up`; existing `RequestOptions`, asynchronous indicator, and KMIP 2.1 typed route |
| KMIPKIT-0099-FR-007 | KMIPKIT-0093-FR-002, FR-011; repository API manifest contract | Rustdoc, bilingual examples compiled by `scripts/test_user_guide_examples.py`, exact manifest contract test `tools/api_manifest/tests/test_0099_fluent_create_split_key_manifest.py`, generated API manifest, and Rust export audit |

The 50 fluent declarations in this Rust-only slice are recorded with
cross-language parity pending. Generated manifest records are not evidence
that C, Java, or Python expose these methods. The operation tests use the
controlled `ExchangeScript` transport; the TLS user journeys remain part of
the later end-to-end delivery and are not claimed by this slice.
