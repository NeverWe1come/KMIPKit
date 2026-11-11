# KMIPKIT-0099 Traceability

All Rust builder code will be in `crates/kmipkit-client/src/fluent.rs`; typed
request and response semantics remain in
`crates/kmipkit-protocol/src/create_split_key.rs`. Unit tests use the existing
controlled exchange support in
`crates/kmipkit-client/tests/unit/asynchronous_execution_test_support.rs`.

| Requirement | Source | Implementation and verification |
|---|---|---|
| KMIPKIT-0099-FR-001 | OASIS KMIP v2.1 §6.1.10 Tables 193–195; KMIPKIT-0093-FR-004 | `RequestBuilder::create_split_key`, `CreateSplitKeyBuilder`; required-field and exact payload tests in `fluent_create_split_key_tests.rs` |
| KMIPKIT-0099-FR-002 | OASIS KMIP v2.1 §6.1.10 Tables 193–195; existing `CreateSplitKeyRequest` | Optional ID, Prime Field Size, Protection Storage Masks, empty Attributes, and attribute-order tests; KMIPKit's Prime Field Size policy remains covered by the existing typed validator |
| KMIPKIT-0099-FR-003 | OASIS KMIP v2.1 §§4–5; KMIPKIT-0093-FR-006 | `attribute`, `attribute_item`, `vendor_attribute`, `attribute_at`, and prebuilt `attributes`; typed type, vendor shape, unknown tag, position, and message-extension rejection tests |
| KMIPKIT-0099-FR-004 | KMIPKIT-0093-FR-004; KMIPKIT-0051-P005; existing typed request validation | `validate` delegates to `build_request`; non-consuming and no-exchange tests, Prime Field Size conditional rejection, Attribute cardinality rejection |
| KMIPKIT-0099-FR-005 | OASIS KMIP v2.1 §§6.1.10, 8–9; KMIPKIT-0093-FR-005, FR-008 | `build`, `build_request`, `to_batch_item`, `execute`; one-exchange result tests preserve Success/Failure/Pending, repeated identifiers, order, correlation, and delivery state |
| KMIPKIT-0099-FR-006 | ADR-0012, ADR-0022; KMIPKIT-0093-FR-007 | Existing `RequestOptions`, asynchronous indicator, and KMIP 2.1 typed route; no raw-send or version-selecting method in the builder |
| KMIPKIT-0099-FR-007 | KMIPKIT-0093-FR-002, FR-011; repository API manifest contract | Rustdoc, English/Spanish examples, generated manifest declarations, `tools/api_manifest` tests and exporter audit |

The fluent declarations in this Rust-only slice will be recorded with
cross-language parity pending. Generated manifest records are not evidence
that C, Java, or Python expose these methods.
