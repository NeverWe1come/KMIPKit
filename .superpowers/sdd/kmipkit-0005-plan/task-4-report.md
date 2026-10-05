# KMIPKIT-0005 T004 Refactor report

## Scope and status

Refactored only the private encoder's internal measurement and writing structure in `crates/kmipkit-client/src/wire_encoder.rs`. Existing T002/T003 behavior tests remain unchanged. This report records the Refactor phase; T004 remains unchecked in `specs/005-ttlv-wire-codec/tasks.md` pending independent review.

No production writer callsite, public encoder/owner API, `Client::execute`, permit, generated artifact, catalog file, or OASIS source was added or changed. No new OASIS-derived regression case was added.

## Refactor

Measurement now returns a `ValueLengths` pair that separates the TTLV Item Length from the actual bytes emitted after the header. `measure_value` dispatches to documented helpers for Structure and each of the ten scalar Item Types. String and byte helpers share the minimal-padding calculation; Big Integer measurement counts sign-extension padding in the Item Length. Structure planning continues to measure each child in insertion order and uses checked arithmetic and U32 Item Length validation.

Writing now dispatches to documented per-type value helpers. Shared helpers write four-byte values with four following padding bytes, eight-byte values, minimally padded strings/bytes, Big Integer sign extension, and bounded padding from fixed stack arrays. Structure writing iterates the model's children in their existing order. Existing vectors continue to verify all eleven Item Types, nested encoding, repeated Tags, canonical padding, and exact bytes.

The test-only payload-copy observer is held by `Writer` under `cfg(test)` and remains at the shared payload-copy boundary. The existing `static_assertions::assert_not_impl_any!` checks still reject `Clone`, `Copy`, `Debug`, `Display`, `serde::Serialize`, `AsMut<[u8]>`, and `Into<Vec<u8>>` for `EncodedOwner`. Its byte field remains private, and its only owner byte accessor is private `as_bytes(&self) -> &[u8]`; there is no public mutation or extraction accessor. Golden-vector copying remains confined to a test helper.

## Error and zeroization invariants

Tree measurement, checked size/limit validation, conversion to the allocation size, and the single complete `try_reserve_exact` occur before `Writer` is constructed. `Writer` and all per-type write helpers return no `Result`, so they have no fallible encode-error exit after a payload copy starts. The output is placed in `EncodedOwner` backed by `Zeroizing<Vec<u8>>` before writing begins. Payload-free `EncodeError` formatting and the retained `TryReserveError` source are unchanged.

The original observer claim above was incorrect. In the test-only `Drop`, `self.bytes.zeroize()` resolves to `Vec<u8>::zeroize`, which wipes the elements and clears the vector length before the observer runs. The observer therefore received an empty slice, and `all(byte == 0)` succeeded vacuously. Production zeroization through `Zeroizing<Vec<u8>>` remained active; the test did not establish that initialized bytes were cleared before deallocation.

## Independent review follow-up — Red

Changed the test-only observer to record the initialized slice length as well as whether every observed byte is zero. The owner Drop test now requires a nonzero observed length and all-zero bytes. No production code change is part of this Red phase.

- `cargo fmt --all` — passed.
- `cargo test -p kmipkit-client owner_drop_zeroizes_initialized_bytes_before_backing_allocation_deallocation` — failed as expected at `observer must see initialized bytes`; 0 passed, 1 failed, 31 filtered out. This confirms the observer sees no initialized bytes after the current `Vec::zeroize` call.

## Verification

Baseline before editing: `cargo test -p kmipkit-client` passed with 32 unit tests, 6 integration tests, and 0 doc tests.

After the refactor:

- `cargo fmt --all --check` — passed.
- `cargo clippy -p kmipkit-client --all-targets --all-features -- -D warnings` — passed.
- `cargo test -p kmipkit-client` — passed: 32 unit tests, 6 integration tests, 0 doc tests.
- `cargo check -p kmipkit-client --all-features` — passed.
- `git diff --check` — passed.

The T002 Red and T003 Green behavior evidence remains in its existing commits; this T004 change is a separate Refactor phase. No T004 completion status is claimed before independent review.
