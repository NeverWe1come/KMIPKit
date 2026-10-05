# T015 Execution Report: Codec Owner and Policy Verification

## Scope

This task verifies KMIPKit-owned security and API boundaries established by
KMIPKIT-0005. It changes no production code and does not claim a request-path,
transport-lifecycle, or client execution integration test; those gates remain
assigned to the first client feature/spec.

## Verification evidence

- `cargo test -p kmipkit-client --all-features --locked wire_encoder::tests`
  — 37 private wire-encoder tests passed, including exact OASIS vectors,
  deterministic round trips, error redaction, zero payload-copy preflight,
  immutable borrowed-limit identity, and owner zeroization before backing
  allocation deallocation.
- `cargo test -p kmipkit-ttlv --all-features --locked --test codec_negative
  --test codec_fixtures --test public_api --test redaction` — 21 tests passed.
  This covers malformed inputs, attribution-backed fixtures, public API
  behavior, and payload redaction.
- `cargo clippy -p kmipkit-client --all-targets --all-features --locked --
  -D warnings` — passed.
- `cargo fmt --all --check` and `git diff --check` — passed.

Static source/API audit:

- `EncodedOwner` is private and owns `Zeroizing<Vec<u8>>`; its only byte
  accessor is private `as_bytes(&self) -> &[u8]`. Compile-time
  `assert_not_impl_any!` checks reject `Clone`, `Copy`, `Debug`, `Display`,
  `serde::Serialize`, `AsMut<[u8]>`, and `Into<Vec<u8>>`.
- The writer module and `encode_item` function are private. The only
  production `encode_item` occurrence is its uncalled definition; all
  invocations are inside the `#[cfg(test)]` module. Search found no
  `OperationEncodingPermit`, `Client::execute`, public writer export, or
  production callsite/mint in the client/TTLV source trees.
- `writer_uses_the_same_borrowed_codec_limits_instance_for_each_operation`
  checks pointer identity with `std::ptr::eq` across two calls; it also checks
  the payload-copy and output-reservation observers.
- `owner_drop_zeroizes_initialized_bytes_before_backing_allocation_deallocation`
  observes the initialized range in the owner drop path before Vec
  deallocation and confirms every byte is zero.
- Encoder preflight tests observe zero output reservation and zero payload
  copies on rejection. Error formatting tests confirm payload bytes do not
  appear in diagnostics.
- Decoder review confirms it parses an input slice into typed model values and
  does not store or expose the complete original wire body, padding, or a
  re-emission API. Typed byte-string and Big Integer values remain represented
  as their modeled payloads; this is not arbitrary raw-message retention.
  Decoder errors are tested against a payload sentinel.

No permit, execute API, production writer callsite, automatic retry, or
transport integration is introduced or claimed here. T015 is complete; T016
full automation and coverage follows.
