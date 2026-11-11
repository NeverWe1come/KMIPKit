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

- Added the seven typed builders and `RequestBuilder` constructors. Builders
  preserve the existing KMIP request types, accept typed or expert ordered
  Cryptographic Parameters, validate before exchange, expose batch conversion,
  and execute one exchange with the existing success/failure/pending response
  model.
- `cargo test -p kmipkit-client --all-features --locked`: passed, 864 unit
  tests plus all package integration and documentation tests; three external
  Cosmian tests are intentionally ignored without a local KMS.
- `cargo clippy -p kmipkit-client --all-targets --all-features -- -D warnings`:
  passed.

## Refactor

- Extended the public-source boundary audit with exact method-scoped
  exceptions for secret byte setters and explicit expert Cryptographic
  Parameters structures. Negative checks reject other builders, method names,
  byte types, and generic signatures.
- Split operation-field preservation coverage by operation for focused test
  failures.
- `cargo test -p kmipkit-client --all-features --locked --lib
  execute_boundary_tests`: passed, 45 checks.
- `cargo fmt --all`: passed.
- Public Rust export audit completed with `matchesManifest: true` after adding
  the new declarations.

## Verification

- `python3 -m unittest tools.api_manifest.tests.test_0102_fluent_cryptographic_manifest`:
  passed, 2 tests.
- `python3 tools/api_manifest/generate.py --check`: passed; all 12 generated
  outputs are current.
- Full coverage and exact-head CI remain pending on the stacked draft PR.
