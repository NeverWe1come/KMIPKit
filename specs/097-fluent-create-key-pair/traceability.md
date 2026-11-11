# KMIPKIT-0097 Traceability

All Rust implementation paths below are in `crates/kmipkit-client/src/fluent.rs`;
the builder delegates execution to
`Client::execute_one_typed_request_with_async_indicator` in
`crates/kmipkit-client/src/execute.rs`. The existing payload and Table 191
validation remain in `crates/kmipkit-protocol/src/create_key_pair.rs`.
All named builder tests below are in
`crates/kmipkit-client/tests/unit/fluent_create_key_pair_tests.rs` and use the
controlled transport fixtures from
`crates/kmipkit-client/tests/unit/asynchronous_execution_test_support.rs`.

| Requirement | Source | Implementation and verification |
|---|---|---|
| KMIPKIT-0097-FR-001 | OASIS KMIP v2.1 §6.1.9 Tables 189–191; KMIPKIT-0093-FR-004 | `RequestBuilder::create_key_pair`, `CreateKeyPairBuilder`; `key_pair_builder_places_semantic_attributes_in_their_exact_key_groups`, `key_pair_batch_conversion_preserves_typed_request_and_ordered_attribute_groups` |
| KMIPKIT-0097-FR-002 | OASIS KMIP v2.1 §6.1.9 Table 191; KMIPKIT-0093-FR-007 | Explicit `algorithm`, `length`, private/public names and masks; `key_pair_builder_requires_distinct_explicit_algorithm_and_length_values`, `key_pair_builder_does_not_synthesize_key_usage_or_curve_attributes`, `key_pair_builder_preserves_exchange_options_and_explicit_prohibited_async_indicator` |
| KMIPKIT-0097-FR-003 | OASIS KMIP v2.1 §§4, 5.1–5.4; KMIPKIT-0093-FR-006; ADR-0012 | Typed attribute setters, `build_request`, payload-free `FluentBuildError`; `conflicting_table_191_key_algorithms_are_rejected_before_transport`, `key_pair_builder_missing_required_values_are_not_sent`, `key_pair_builder_invalid_async_indicator_is_not_sent`; generated-type checks in `crates/kmipkit-protocol/src/attribute.rs` |
| KMIPKIT-0097-FR-004 | OASIS KMIP v2.1 §§6.1.9, 8; KMIPKIT-0093-FR-005 | `build`, `build_request`, `to_batch_item`, `execute`; `key_pair_batch_conversion_preserves_typed_request_and_ordered_attribute_groups`, `key_pair_builder_executes_one_typed_exchange_without_inferring_response_attributes`, `key_pair_builder_preserves_failure_without_success_fields_or_retry`, `key_pair_builder_preserves_pending_and_exact_binary_correlation_without_polling`, `key_pair_builder_preserves_exchange_options_and_explicit_prohibited_async_indicator`, `key_pair_builder_transport_failure_preserves_delivery_state_without_retry` |
| KMIPKIT-0097-FR-005 | KMIPKIT-0093-FR-002, FR-011; repository API manifest contract | Rustdoc in `crates/kmipkit-client/src/fluent.rs`; bilingual marked examples in `docs/user-guide/en/client-execution.md` and `docs/user-guide/es/ejecucion-cliente.md`, compiled by `scripts/test_user_guide_examples.py`; exact root/module declarations in `specification/api/public-api.json`, checked by `tools/api_manifest/tests/test_0097_fluent_create_key_pair_manifest.py`, `tools/api_manifest/generate.py`, and `tools/api_manifest/audit_rust_exports.py` |
| KMIPKIT-0097-FR-006 | KMIPKIT-0093-FR-004; existing typed construction and validation boundary | `CreateKeyPairBuilder::validate` delegates to `build_request`; `key_pair_validation_is_local_non_consuming_and_matches_construction`, `key_pair_prebuilt_duplicates_in_each_group_are_rejected_before_transport`, `key_pair_prebuilt_conflicting_algorithms_use_the_existing_request_validator` |
| KMIPKIT-0097-FR-007 | OASIS KMIP v2.1 §6.1.9 Tables 189–191; KMIPKIT-0093-FR-004 | `common_attributes`, `private_key_attributes`, `public_key_attributes`; `key_pair_prebuilt_sets_preserve_group_internal_and_repeated_call_order`, `key_pair_prebuilt_duplicates_in_each_group_are_rejected_before_transport`, `key_pair_prebuilt_conflicting_algorithms_use_the_existing_request_validator` |

## Manifest and response scope

The manifest adds 46 exact Rust declarations: both public paths for the builder,
its 19 methods, the request-builder constructor, and the two new error variants.
All cite this owner's FRs and remain `blocked` with `four-language` parity scope
until equivalent C, Java, and Python mappings exist. Eighteen exact compiler
standard-trait paths are approved exclusions under the documented language-level
scope; no previous declaration changes. Existing response accessor declarations
remain unchanged. The typed success example reads both identifiers from the
server response; it does not infer returned attributes from request inputs.

`tools/api_manifest/generate.py --write` updates the checked-in generated parity
fixtures at `tests/fixtures/api/generated/public_api_parity.json` and
`bindings/java/src/test/java/org/kmipkit/generated/PublicApiParityFixtures.java`.
Blocked declarations do not generate ready foreign-language methods.
The compiler export audit is also recorded with its pre-existing unrelated
inventory gaps; it does not establish four-language exposure or conformance.
