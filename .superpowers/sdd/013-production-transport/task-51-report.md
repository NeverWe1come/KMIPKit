# T051 Refactor evidence

## Refactor

- Refactor commit: `16e4499` (`refactor(client): isolate typed response ownership`).
- `decode_transport_response` now owns the `TransportResponse` through bounded decoding, computes
  delivery evidence once, drops the zeroizing response wrapper before converting decoder failures
  to public client errors, and returns only a typed `ResponseMessage` with its delivery state.
- Oversize and malformed decode failures preserve their existing sanitized protocol categories and
  delivery-state mapping.

## Verification

- T049 tests remain green: `cargo test -p kmipkit-client --all-targets --all-features --offline --quiet`
  passed (203 library tests and all integration targets, including 14 production-client tests and
  both compile-fail contracts).
- `cargo clippy -p kmipkit-client -p kmipkit --all-targets --all-features --offline -- -D warnings`:
  passed.
- `cargo fmt --all --check` and `git diff --check`: passed.
