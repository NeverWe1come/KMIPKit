# KMIPKIT-0005 T003 Green report

## Scope

Promoted the reviewed `kmipkit-ttlv` and `zeroize` client dependencies to normal dependencies. Kept `serde` and `static_assertions` test-only. Replaced the test stub with a private outbound encoder and zeroizing owner. No public encoder or owner API, production writer callsite, `Client::execute`, or permit was added; T004 was not started.

## Dependency gate

Before the manifest edit, `cargo metadata --locked --offline` and `cargo tree --locked --offline -p kmipkit-client --edges normal,dev` matched `specs/005-ttlv-wire-codec/dependency-review.md`. The reviewed selections were `kmipkit-ttlv` 0.1.0 with no declared features; `zeroize` 1.9.0 with defaults disabled and only `alloc`; `serde` 1.0.228 with its `std` default for tests; and `static_assertions` 1.1.0 with defaults disabled. The graph was rechecked after the move and retains the same package versions and feature selections.

## TDD evidence

At base commit `97fac0bbfad574e839f9f5b8208a5274ab84b46a`, `cargo test --locked --offline -p kmipkit-client` compiled and ran the T002 Red suite: 31 unit tests, 3 passed and 28 failed on the expected missing behavior. T002 assertions were retained and adapted to exercise the production private encoder; the no-copy observer is attached to the shared production payload-copy boundary.

## Implementation

The encoder validates and measures the tree before output allocation, checks arithmetic and every U32 Item Length, enforces per-call byte/depth/count limits with a private borrowed view and a hard depth cap of 64, and makes one complete fallible reservation. The bounded writer encodes all eleven Item Types, retains Structure order and repeated tags, preserves raw enum/bitmask values, applies canonical zero padding, and sign-extends Big Integers. The private output owner wraps its vector in `Zeroizing`; test-only observers verify zero payload copies on preflight rejection and clearing before deallocation. Errors carry no payload data.

## Verification

- `cargo fmt --all --check` passed.
- `cargo clippy -p kmipkit-client --all-targets --all-features -- -D warnings` passed.
- `cargo test -p kmipkit-client` passed: 31 unit tests, 6 integration tests, 0 doc tests.
- `cargo check -p kmipkit-client --all-features` passed.
- `git diff --check` passed.

T003 Green is ready for independent review. The separate T004 refactor remains unstarted.
