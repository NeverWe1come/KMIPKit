# KMIPKIT-0114 Traceability

| Requirement | Implementation | Verification | Normative / project source |
|---|---|---|---|
| KMIPKIT-0114-FR-001 | Ten C entry points in `crates/kmipkit-ffi/src/extension_registry/client_abi/fluent.rs`; declarations generated into `bindings/c/include/kmipkit.h` | `public_c_header_constructs_a_batch_and_reaches_the_encoding_override_entrypoint`; `public_c_fluent_client_executes_lifecycle_helpers_over_mutual_tls` | KMIP 2.1 §§6.1.1, 6.1.4, 6.1.28, 6.1.42, 6.1.44; KMIPKIT-0018 |
| KMIPKIT-0114-FR-002 | `required_lifecycle_identifier`, `copy_fluent_attribute_set`, and `revoke_request_by_id` | TLS fixture decodes the text identifier, a typed Object Type Locate filter, Revoke reason, optional message, and compromise date; C consumer rejects missing identifiers, mismatched handles, invalid flags, and an unsupported date before the first connection | KMIP 2.1 §§6.1.28, 6.1.44; KMIPKIT-0018 |
| KMIPKIT-0114-FR-003 | Existing `execute_simple_request` path and `borrow_fluent_request_options` | TLS fixture observes one exchange for each direct and timeout-options helper; all four timeout bounds are supplied on options calls | KMIPKIT-0093-FR-004, FR-007, FR-011 |
| KMIPKIT-0114-FR-004 | Existing FFI status, response-owner, redaction, and delivery-state helpers | Compiled C consumer checks status and null output for invalid inputs; TLS test reads each typed response before releasing its handle | KMIPKIT-0007-FR-005, FR-006; KMIPKIT-0093-FR-011 |
| KMIPKIT-0114-FR-005 | Compiled C consumer and controlled mutual-TLS fixture in `crates/kmipkit-ffi/tests/c_api_coverage.rs` | Compiled C consumer calls all ten symbols; fixture decodes and responds to all ten calls over mutual TLS; `python3 -B tools/api_manifest/generate.py --check` verifies generated declarations | KMIPKIT-0093-FR-013 |

