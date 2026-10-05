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

The existing test-only per-owner Drop observer remains in place. The test-only Drop implementation zeroizes the initialized `Vec` bytes, then passes only the live initialized slice to the observer before the owner's fields drop and the backing allocation is deallocated. It uses no unsafe code and performs no freed-memory read. Uninitialized spare capacity remains outside the guarantee.

## Verification

Baseline before editing: `cargo test -p kmipkit-client` passed with 32 unit tests, 6 integration tests, and 0 doc tests.

After the refactor:

- `cargo fmt --all --check` — passed.
- `cargo clippy -p kmipkit-client --all-targets --all-features -- -D warnings` — passed.
- `cargo test -p kmipkit-client` — passed: 32 unit tests, 6 integration tests, 0 doc tests.
- `cargo check -p kmipkit-client --all-features` — passed.
- `git diff --check` — passed.

The T002 Red and T003 Green behavior evidence remains in its existing commits; this T004 change is a separate Refactor phase. No T004 completion status is claimed before independent review.
