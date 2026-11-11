# KMIPKIT-0095 binding implementation evidence

## Red

- `cargo test -p kmipkit-ffi client_response_extension_abi_returns_ordered_generic_ttlv_copy --lib`: exit 101, E0425 for the three missing C response renderers.
- `mvn -q -f bindings/java/pom.xml test-compile`: exit 1, missing `ClientBatchItemResult.ttlvHex/json/xml(CodecLimits)`.
- Python contracts request the same three explicit methods and verify native ownership transfer with no transport available in the fake bridge.

The approved scope is T009 only. Exact remote CI remains T008 and is owned by the parent task.
