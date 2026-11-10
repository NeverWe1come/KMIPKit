# Requirement Traceability

Implementation paths and verification entries below identify the foundation
delivery. Later Fluent DX operation families remain outside this specification.

| Requirement | Normative / project source | Implemented surface | Verification |
|---|---|---|---|
| KMIPKIT-0091-FR-001 | KMIP 2.1 §§6.1.13, 6.1.15, 6.1.19; KMIPKit-0007, -0052 | `kmipkit_client::Client::request`, C direct helpers, `KmipClient.request`, `Client.request` | `fluent_client_completes_create_get_destroy_over_mutual_tls`; `public_c_fluent_client_completes_create_get_destroy_over_mutual_tls`; `test_fluent_create_get_destroy_uses_server_created_identifier`; Java 0053 consumer lifecycle |
| KMIPKIT-0091-FR-002 | KMIP 2.1 §6.1.13; KMIP 2.1 §11.56; KMIPKit-0047 | `CreateSymmetricKeyBuilder` and per-language adapters | `symmetric_key_builder_requires_algorithm_length_and_usage_mask`; `aes_256_builder_emits_typed_attributes_and_interpreted_text_name`; Java `createBuilderUsesRustValidationAndProducesTypedCreateItem`; Python `test_fluent_symmetric_builder_requires_explicit_key_properties` |
| KMIPKIT-0091-FR-003 | KMIP 2.1 §7.34; KMIPKit-0048 | Rust `name`, C/Java/Python builder name setters | Four Create→Get→Destroy TLS journeys assert/reuse the server identifier; Rust `aes_256_builder_emits_typed_attributes_and_interpreted_text_name` |
| KMIPKIT-0091-FR-004 | KMIP 2.1 §§4, 7.1; KMIPKit-0048, -0051 | Rust attribute accumulator; C ABI and Java/Python adapters | `catalogued_attribute_item_rejects_a_wrong_ttlv_type`; public builder validation tests |
| KMIPKIT-0091-FR-005 | KMIP 2.1 §§4, 7.1, 4.60, 11.56; KMIPKit-0048 | Rust explicit Item and Vendor Attribute construction, exposed through C/Java/Python | `explicit_unknown_attribute_is_preserved_but_extension_tags_are_rejected`; `explicit_generic_attribute_and_vendor_attribute_are_retained` |
| KMIPKIT-0091-FR-006 | KMIP 2.1 §§6.1.15, 6.1.19; KMIPKit-0049 | Rust `GetBuilder`/`DestroyBuilder`; direct and fluent C/Java/Python methods | `get_and_destroy_builders_require_a_typed_unique_identifier`; four Create→Get→Destroy TLS journeys |
| KMIPKIT-0091-FR-007 | KMIP 2.1 §6.1; KMIPKit-0007, -0052 | Existing typed response/outcome path | `execute_exposes_pending_correlation_without_polling`; Java 0053 consumer verifies Success, Pending correlation, and Failure |
| KMIPKIT-0091-FR-008 | KMIPKit-0007 and -0013 | `RequestOptions` and explicit Asynchronous Indicator batch header | Java `buildersExposeBuildBatchAndExecutionMethods`; Python `test_fluent_async_indicator_is_encoded_in_request_header`; Java mTLS consumer exercises RequestOptions and indicator |
| KMIPKIT-0091-FR-009 | Project constitution III; KMIPKit-0053 | `crates/kmipkit-ffi/src/extension_registry/client_abi/`, `bindings/c/include/kmipkit.h` | `public_c_fluent_builder_constructs_a_typed_create_item_and_checks_versioned_input`; `public_c_fluent_client_completes_create_get_destroy_over_mutual_tls` |
| KMIPKIT-0091-FR-010 | KMIPKit-0053 | `org.kmipkit.client.KmipClient`, `org.kmipkit.fluent` | Java `FluentFacadeShapeTest`; Java 0053 consumer TLS lifecycle |
| KMIPKIT-0091-FR-011 | KMIPKit-0053 | `bindings/python/src/kmipkit/fluent.py` and Client facade | Python `test_fluent_create_get_destroy_uses_server_created_identifier`; `test_builder_context_manager_releases_native_handle`; `test_legacy_extensions_remain_available` |
| KMIPKIT-0091-FR-012 | Project constitution III; ADR-0014 | Rust client core, C ABI, Java JNI, Python CFFI | `python3 tools/api_manifest/generate.py --check`; Java JNI and C ABI export audits |
| KMIPKIT-0091-FR-013 | ADR-0014; KMIPKit-0007, -0051 | Existing typed request and batch execution boundaries | `generic_item_and_raw_body_inputs_are_rejected`; fluent methods return existing typed request or batch-item types |
| KMIPKIT-0091-FR-014 | KMIP 2.1 §11.56; ADR-0022; KMIPKit-0069 | Versioned typed routes and client transport gate | `public_c_fluent_builder_constructs_a_typed_create_item_and_checks_versioned_input`; `invalid_fluent_create_execution_keeps_not_sent_delivery_state` |
| KMIPKIT-0091-FR-015 | Project constitution IV; KMIPKit-0043, -0049 | Existing secret owners, redaction, native-handle lifetimes | C lifecycle release paths; Java NativeHandle close test; Python `test_builder_context_manager_releases_native_handle` |
| KMIPKIT-0091-FR-016 | Project constitution I, III; KMIPKit-0053 | `specification/api/public-api.json`, generated exports, English/Spanish guide | Generator `--check`, export audits, four-language lifecycle tests, and this traceability table |

References to project requirement IDs are resolved through the approved
specifications listed in [spec.md](spec.md). CI remains the final gate for the
combined branch, including the complete parity runner and repository checks.
