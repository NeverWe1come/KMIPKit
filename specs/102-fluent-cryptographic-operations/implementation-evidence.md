# KMIPKIT-0102 Implementation Evidence

## Branch and dependency

- Branch: `feature/KMIPKIT-0102-fluent-cryptographic-operations`.
- Base: PR #184 at `3d9a53139` (unmerged; exact-head CI is running).
- Scope: Rust fluent builders for Encrypt, Decrypt, Sign, Signature Verify,
  MAC, MAC Verify, and Hash.

## Red

- The tests in
  `crates/kmipkit-client/tests/unit/fluent_cryptographic_operation_tests.rs`
  exercise all seven constructors, typed parameter input, expert ordered
  parameters, and exact typed batch variants.
- Command: `cargo test -p kmipkit-client --all-features --locked --lib
  fluent_cryptographic_operation_tests --no-run`.
- Expected Red result: compilation fails because `RequestBuilder` does not yet
  expose `encrypt`, `decrypt`, `sign`, `signature_verify`, `mac`, `mac_verify`,
  or `hash` constructors (8 missing-method errors across the test cases).

## Green

- Pending implementation.

## Refactor

- Pending implementation and public-source-boundary audit.

## Verification

- Pending focused and full validation, generated API reconciliation, coverage,
  and exact-head CI.
