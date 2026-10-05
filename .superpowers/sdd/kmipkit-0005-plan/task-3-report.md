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

T003 Green passed independent review after the allocation-source follow-up below. The separate T004 refactor remains unstarted.

## Independent review follow-up — Red

Added `allocation_error_exposes_its_reservation_source` before changing the production error. The Red commit is `3ff1016`. The focused Red command was `cargo test -p kmipkit-client allocation_error_exposes_its_reservation_source`; it failed at the intended missing behavior because `EncodeError` did not implement `std::error::Error` (`E0277`, `EncodeError: StdError is not satisfied` at the `Error::source` assertion). No production implementation change is included in this Red step.

## Independent review follow-up — Green

The allocation error now retains `std::collections::TryReserveError` as its `Error::source()`. Its `Display` and `Debug` include only the safe error category, not the source text. The production reservation helper is exercised with impossible `usize::MAX` capacity, which returns capacity overflow without a large allocation. The focused source-preservation test passed.

Green verification passed: `cargo fmt --all --check`; `cargo clippy -p kmipkit-client --all-targets --all-features -- -D warnings`; `cargo test -p kmipkit-client` (32 unit tests, 6 integration tests, 0 doc tests); and `git diff --check`. The Green implementation commit is `b38640aa00007bb673c32173c17572721f2b877c`.

## Independent review disposition

The reviewer confirmed the allocation-source finding is addressed and reported no new issues. The reviewer verified that `Display` and `Debug` omit the retained source details and that the regression test forces capacity overflow before allocation. The reviewer did not run tests; the coordinator independently ran `cargo test -p kmipkit-client` after Green: 32 unit tests and 6 integration tests passed. T003 is complete. T004 remains unstarted.
