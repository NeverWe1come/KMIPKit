# T007 Refactor report (provisional)

## Scope and status

This report records the T007 Refactor for KMIPKIT-0005. The implementation refactors only `crates/kmipkit-ttlv/src/codec/decoder.rs` and `crates/kmipkit-ttlv/src/codec/mod.rs`. No tests, protocol/client integration, configurable `CodecLimits`, `decode_with_limits`, or T008 roundtrip work were added. T007 remains unchecked in `tasks.md` pending independent review.

Refactor commit: `f576a82` (`refactor(ttlv): isolate decoder span checks`), with a DCO sign-off.

## Refactor details

`decode_item` remains the parsing coordinator. Header parsing and validation are grouped in `parse_item_header`; tag checks and Structure depth calculation are isolated in `checked_tag` and `structure_depth_for`. `item_span` calculates checked value and item end offsets through `checked_end_offset`. `validate_parent_boundary` applies parent Structure boundary, value truncation, and root padding extent checks in the same order and with the same error categories as before.

`DecodeError` and `DecodeErrorKind` now live in the codec facade module, separating the public error surface from decoder internals. Their visibility, variants, getters, formatting, and public import paths are unchanged. `decode_value`, default resource limits, and decoding semantics were not changed.

## Input and unsafe-code audit

`DecodeError` contains only the error kind and byte offset. Its Display implementation writes the static category message and offset; derived Debug formats only those fields. Neither formatting path receives or retains input bytes. The decoder borrows the input during parsing and does not store the raw input in decoder state or in the returned item. Validated payload values are copied into the returned typed model as required by the public API.

No unsafe code was added to either file. The crate-level `#![forbid(unsafe_code)]` remains in force. The existing public redaction test still passes. The only error kinds without deterministic direct test injection are `AllocationFailed` (requires allocator failure injection) and the defensive `ModelConstraint` mapping; the existing suite covers malformed wire, tag, limit, and Structure-boundary categories.

## Verification evidence

All commands below completed with exit code 0 after the refactor:

- `cargo fmt --all --check`
- `cargo clippy -p kmipkit-ttlv --all-targets --all-features -- -D warnings`
- `cargo check -p kmipkit-ttlv --all-features`
- `cargo test -p kmipkit-ttlv`
- `cargo doc -p kmipkit-ttlv --no-deps`
- `git diff --check`

The crate test run reported 82 passed, 0 failed, 0 ignored across 33 decoder unit tests, 1 public codec API test, 16 codec negative tests, 2 public API tests, 2 redaction tests, 8 tag allocation tests, 16 value model tests, 2 value zeroization tests, 1 zeroization test, and 1 doctest.

No behavior adjustment was needed. Independent T007 review is pending; this report does not close the task.
