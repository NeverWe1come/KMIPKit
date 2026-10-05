# T010 Green Implementation Report

## Scope and status

This report records the provisional T010 Green implementation in commit
`302b8c60a75e22b9d590f1253c949d5369627f36`. It implements immutable public
`CodecLimits`, public per-call `decode_with_limits`, production decoder
preflight and counters, and adapts the private client writer to consume the
same borrowed limits instance. T010 remains unchecked pending independent
review; this report does not close the task.

No manifest or dependency changes were made. No production client callsite,
`Client::execute`, `OperationEncodingPermit`, T011 refactor, transport work, or
FR-013 behavior was added. Decoder errors remain payload-free. The decoder
does not retain input bytes. The client writer's limits adapter is private and
does not expose a public encoding API.

## Implementation and coverage

`CodecLimits` has private fields, no `Clone` or `Copy` implementation, default
constants of 16 MiB, Structure depth 64, and 100,000 Items, a validated
constructor, `defaults`, and read-only getters. Zero byte and Item limits and
depth zero are accepted; depth 65 is rejected. `decode` delegates to
`decode_with_limits` using defaults. The decoder checks total input size before
parsing, counts the root Item, checks nested Structure depth, validates checked
spans and boundaries, and uses fallible reservations before copying peer
payload bytes. Existing model constructors still use infallible allocations;
the public decoder docs record that OOM caveat.

T009's 11 private limit tests now exercise production helpers and APIs,
including exact/one-over byte, depth, and count cases, synthetic U32 and
arithmetic-overflow checks, and test-only observation that configured byte
limits and oversized declarations reject before payload reservation or copy.
The public integration suite in `tests/codec_limits_api.rs` verifies defaults,
configured getters, zero and low limits, depth constructor boundaries, and
exact/one-over decoding. It also proves raised byte limits above 16 MiB and
raised Item limits above 100,000 using bounded valid wire fixtures. Compile-fail
tests establish that `CodecLimits` is neither `Clone` nor `Copy`.

The private encoder adapter reads limits through getters and passes the exact
borrowed `&CodecLimits` through each operation without cloning or reconstruction.
The client test observes pointer identity for each writer preflight. T002's
synthetic byte/depth/count/U32 planning cases remain in place. No public client
configuration surface or production writer callsite was introduced.

## Verification evidence

All commands below completed with exit code 0:

- `cargo fmt --all --check`
- `cargo clippy -p kmipkit-ttlv --all-targets --all-features -- -D warnings`
- `cargo clippy -p kmipkit-client --all-targets --all-features -- -D warnings`
- `cargo check -p kmipkit-ttlv --all-features`
- `cargo check -p kmipkit-client --all-features`
- `cargo test -p kmipkit-ttlv` — 100 passed, 0 failed, including unit,
  integration, compile-fail API, and doc tests.
- `cargo test -p kmipkit-client` — 41 passed, 0 failed, including unit and
  integration tests.
- `cargo doc -p kmipkit-ttlv --all-features --no-deps` — succeeded without
  warnings.
- `cargo doc -p kmipkit-client --all-features --no-deps` — succeeded without
  warnings.
- `git diff --check` — exit code 0; Git emitted only its CRLF-to-LF
  normalization notice for `src/codec/mod.rs`.

The scoped production audit found no `unsafe`, panic-capable `unwrap()` or
`expect()`, `panic!`, `todo!`, or `unimplemented!` in the decoder, limits API,
or private writer production code. There are no production writer callsites.
`tasks.md` remains unchanged with T010 unchecked, and the manifests remain
unchanged.

## Review status

Pending independent review. No push or pull request was created.
