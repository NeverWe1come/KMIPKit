# KMIPKIT-0095 Traceability

| Requirement | Implementation | Evidence |
|---|---|---|
| KMIPKIT-0095-FR-001 | `ClientBatchItemResponse::{ttlv_hex,json,xml}` | `crates/kmipkit-client/tests/unit/response_boundary_tests.rs::response_item_renders_the_retained_full_message_in_all_three_encodings` |
| KMIPKIT-0095-FR-002 | Full retained `ResponseMessage` tree is cloned and wrapped as a `ResponseMessage` root before encoding | Same test compares ordered TTLV bytes and verifies header, batch, and vendor extension in JSON/XML |
| KMIPKIT-0095-FR-003 | Zeroizing owned output and payload-free `ResponseEncodingError`; existing `Debug` remains payload-free | Focused limit test checks that error text contains no vendor payload; the current Rust inventory maps or excludes every response-view declaration |
| KMIPKIT-0095-FR-004 | `private_wire_writer::render_for_inspection` has no request execution permit and is called only from response rendering | `execute_boundary_tests::response_inspection_codec_dispatch_is_limited_to_its_private_renderer` and `production_source_inventory_is_complete_and_execute_owns_the_only_writer_permit_pair` |
| KMIPKIT-0095-FR-005 | Existing TTLV/JSON/XML codec limits passed into encoder | Focused test asserts a bounded XML render fails as `LimitExceeded` |

## TDD evidence

The first Red test referenced the three missing public rendering methods and
failed to compile with E0599. The implementation then failed compilation once
because the retained callback accepts an owned view; the closure was corrected
to call `try_clone()` on that view. The final focused and 783-test client suite,
Clippy, structural catalog validation, feature traceability tests, API generation
check, and formatting all pass locally. PR CI is still required for the exact
remote head.

## T009 binding traceability

| Requirement | C / Java / Python implementation | Binding evidence |
|---|---|---|
| KMIPKIT-0095-FR-001 | `kmipkit_client_batch_item_response_{ttlv_hex,json,xml}`, `ClientBatchItemResult.{ttlvHex,json,xml}`, `BatchItemResult.{ttlv_hex,json,xml}` call the Rust methods | Compiled `client_abi_consumer.c`, Java `ClientFacadeTlsIntegrationTest`, Python `test_response_raw_navigation.py` |
| KMIPKIT-0095-FR-002 | The binding passes the retained response item directly to the Rust full-message renderer | Native Java/Python fixtures compare exact canonical TTLV hex and ordered multi-item header/batch views; FFI vendor-extension fixture |
| KMIPKIT-0095-FR-003 | Existing native `SecretBytes` owner; Java AutoCloseable and Python context-manager ownership, scoped and cleared callback copies | Independent-owner FFI test; Java/Python native tests exercise source closure, callback erasure, and redacted representations |
| KMIPKIT-0095-FR-004 | Bindings accept only a response handle and codec limits; helper calls only response rendering and owned-handle construction | Python fake bridge exposes no transport functions; native render tests after client closure; existing AST request-boundary inventory |
| KMIPKIT-0095-FR-005 | C borrows `CodecLimits`; JNI creates equivalent native limits; Python borrows native limits | Java/Python native tests reject size, depth and element limits; compiled C rejects size limits and null handles/outputs |

The C ABI maps limit/allocation failures to the existing resource-limit code;
invalid retained trees use invalid-input; codec encoding failures use the
existing payload-free internal code.
This deliberately preserves the existing fixed-width ABI and language error
families. It does not claim independent external profile certification.

### Exact binding error translation

| Rust response encoding category | C status | Java | Python |
|---|---|---|---|
| `InvalidResponseTree` | `KMIPKIT_ERROR_INVALID_INPUT` (4) | `InvalidInputException` | `InvalidInputError` |
| `LimitExceeded` | `KMIPKIT_ERROR_RESOURCE_LIMIT` (6) | `ResourceLimitException` | `ResourceLimitError` |
| `AllocationFailed` | `KMIPKIT_ERROR_RESOURCE_LIMIT` (6) | `ResourceLimitException` | `ResourceLimitError` |
| `EncodingFailed` | `KMIPKIT_ERROR_INTERNAL` (7) | `KmipException` | `KmipKitError` |

The existing binding status model combines allocation and configured resource
limits. It distinguishes invalid retained trees, output/resource limits, and
codec representation failures. No category includes a response or error source.
The manifest maps the Rust error type and EncodingFailed variant to the exact
renderer failure surfaces. Generic internal exceptions remain excluded as
standalone capabilities under KMIPKIT-0043; this delivery does not change that
panic-fallback contract. `response_view_error_categories_are_stable_and_payload_free`
checks every Rust variant against this translation.
