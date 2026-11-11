# KMIPKIT-0102 Implementation Evidence

## Branch and dependency

- Branch: `feature/KMIPKIT-0102-fluent-cryptographic-operations`.
- Base: PR #184 at `1885a509e` (unmerged; exact-head CI is running).
- Draft PR: #185, based on #184; title retains `[WIP]`.
- Red/Green/Refactor commits: `c077230d2`, `a8fc68cf3`, and `8e536b121`.
- Commit `2c495141d` merges the latest #184 branch head into this already-pushed
  branch without rewriting the published feature commits.
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
- SonarCloud reported 26.8% duplicated lines on new code (552 lines; limit
  3%) at head `fd5b0bdaa201cd3b2cf64d1c39ed6d6976cbe17c`. The repeated spans
  were fluent setters shared across operation builders. The refactor routes
  byte and typed-parameter inputs through each builder's existing semantic
  setters while preserving the seven explicit public builder types and all
  signatures.
- Commit `5dab7e23b` refreshes the historical non-target manifest count and
  fingerprint to account for KMIPKIT-0102's generated Rust API declarations.

## Verification

- `python3 -m unittest tools.api_manifest.tests.test_0102_fluent_cryptographic_manifest`:
  passed, 2 tests.
- `python3 tools/api_manifest/generate.py --check`: passed; all 12 generated
  outputs are current.
- Full-client verification with the updated #184 head:
  `cargo test -p kmipkit-client --all-features --locked --quiet` — passed
  (865 unit tests, package integration tests, and doctests; 3 external KMS
  tests ignored because no KMS is configured).
- `cargo clippy -p kmipkit-client --all-targets --all-features -- -D
  warnings` — passed.
- `python3 -m unittest tools.api_manifest.tests.test_0102_fluent_cryptographic_manifest tools.api_manifest.tests.test_t102_certify_request_builder_mapping`:
  passed, 10 tests.
- `python3 -m unittest tools.normative_catalog.tests.test_feature_traceability`:
  passed, 31 tests.
- `python3 -B tools/api_manifest/generate.py --check`: passed; 12 outputs
  verified.
- Full coverage and exact-head CI remain pending on the stacked draft PR.
