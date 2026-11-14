# KMIPKIT-0114 Implementation Plan

## Design

Add the lifecycle entry points to the existing C fluent adapter module. Keep
pointer validation, handle ownership, response construction, error recording,
and transport dispatch in the FFI crate. Parse C values into the existing
KMIP protocol request models, then call the corresponding typed `Client`
method with the supplied codec limits and optional exchange-local options.

Use the existing `kmipkit_unique_identifier_v1_t`, attribute-set handle,
revocation-reason handle, request-options record, and batch-response handle.
Do not add an operation-code selector or a raw-message execution path. Update
the public API manifest and regenerate outputs with the repository generator.

## Test strategy

1. Add a compiled C consumer that calls each helper and options variant.
2. Add fake/controlled-transport tests that decode requests and verify the
   operation, identifier, filter attributes, revoke reason, and optional date.
3. Exercise malformed or mismatched handles and prove the request was not
   sent; assert only the established `int32_t` status and delivery state.
4. Run the C ABI, lifecycle, generated-manifest, export-audit, coverage,
   workspace format, Clippy, and relevant Rust/C test gates.

