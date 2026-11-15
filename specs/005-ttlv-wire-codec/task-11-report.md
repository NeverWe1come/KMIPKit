# T011 Refactor Report

## Scope and status

This report records the provisional T011 Refactor in commit
`1e904459b1addda85c7b702400fab58cf038dce8`. T011 remains unchecked pending
independent review. The T010 implementation and its commits are unchanged.

The refactor documents the decoder's established per-call accounting order and
the encoder's single preflight gate. No decoder validation order, error
category, or offset changed. No public API, `Client::execute`, production
writer callsite, permit, manifest, dependency, `approval-record.md`, or
`tasks.md` change was made. The writer still receives the same borrowed
`&CodecLimits` instance for each operation.

## Gap audit and coverage

Before editing, the existing tests already covered the principal boundaries:

- Decoder default limits: `codec_negative.rs` accepts an item at the 16 MiB
  cap and rejects one byte over, accepts depth 64 and rejects 65, and accepts
  100,000 Items and rejects 100,001. The T009/T010 private and public tests
  cover configured exact/one-over byte, depth, and count limits, getter values,
  U32 length parsing, checked arithmetic, and rejection before decoder-owned
  payload reservation/copy.
- Encoder planner: T002 tests cover default and configured byte, depth, count,
  and U32 exact/one-over boundaries with synthetic bounded plans. Depth 65 and
  100,001 Items are scalar plan values, not constructed trees. T010 also proves
  per-operation borrowed-instance identity.
- The gap was that the real `CodecLimits` adapter did not have a compact planner
  boundary test for all three dimensions, and the writer test observed payload
  copies but not the output reservation boundary.

The client tests now exercise exact/one-over default and configured byte,
Structure-depth, and Item-count boundaries through the real borrowed
`CodecLimits` adapter. The U32 ceiling remains a separate wire-fixed boundary:
`u32::MAX` succeeds and `u32::MAX + 1` fails without allocating either value.
The existing writer observer now records its actual output-reservation and
payload-copy boundaries under `cfg(test)`. A rejected preflight asserts zero
reservations and zero copies; two successful operations assert one reservation
per operation. The decoder's existing observer remains at the real
decoder-owned reservation/copy boundary and its rejection tests continue to
assert both counters remain zero.

The existing T002/T009/T010 tests were the behavioral baseline for this
Refactor; no new Red failure was claimed or manufactured. The added
`CodecLimits` planner boundary test passed against the T010 implementation
before documentation changes. During observer wiring, a focused build exposed
one test-only method visibility error (`E0624`); the observer method was made
visible to its parent module and the focused rejection test then passed. No
production behavior was changed to resolve it.

## Verification evidence

All final commands completed with exit code 0:

- `cargo fmt --all --check`
- `cargo clippy -p kmipkit-ttlv --all-targets --all-features -- -D warnings`
- `cargo clippy -p kmipkit-client --all-targets --all-features -- -D warnings`
- `cargo check -p kmipkit-ttlv --all-features`
- `cargo check -p kmipkit-client --all-features`
- `cargo test -p kmipkit-ttlv` — 100 passed, 0 failed, including unit,
  integration, compile-fail API, and doc tests.
- `cargo test -p kmipkit-client` — 42 passed, 0 failed, including unit and
  integration tests.
- `cargo doc -p kmipkit-ttlv --all-features --no-deps` — succeeded without
  warnings.
- `cargo doc -p kmipkit-client --all-features --no-deps` — succeeded without
  warnings.
- `git diff --check` — exit code 0; Git emitted only its CRLF-to-LF
  normalization notice for `src/codec/mod.rs`.

Focused checks also passed:

- `cargo test -p kmipkit-client codec_limits_adapter_enforces_default_and_configured_exact_boundaries`
- `cargo test -p kmipkit-client preflight_rejection_performs_zero_payload_copies`

The added reservation observer is compiled only under `cfg(test)`. The
Refactor adds no unsafe code, raw input retention, or production error
formatting of input bytes. Existing payload-redaction tests remain in the
passing crate suites.

## Review status

Pending independent review. T011 is not marked complete. No push or pull
request was created.
