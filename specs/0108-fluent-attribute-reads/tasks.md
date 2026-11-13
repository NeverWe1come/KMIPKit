# KMIPKIT-0108 Delivery Tasks

- [x] T001 Add failing controlled tests for both builders, ordered references,
  omitted identifiers, duplicate-reference validation, and one-exchange
  execution. Red: commit `aa3402158`; `cargo test -p kmipkit-client
  fluent_attribute_read_tests --lib` failed because the two `RequestBuilder`
  constructors did not exist.
- [x] T002 Implement typed request, batch conversion, options, asynchronous
  indicator, and execution for Get Attributes and Get Attribute List. Green:
  all five focused tests pass through the existing typed exchange path.
- [x] T003 Document both APIs in English and Spanish; update traceability and
  generated API manifest/parity artifacts with the canonical generator.
- [ ] T004 Run local gates, commit Red/Green/Refactor separately, open a `[WIP]`
  PR, and resolve every red CI check before marking ready.

## Local Verification

- `cargo fmt --all --check` passed.
- `cargo test -p kmipkit-client --all-features` passed: 885 unit tests,
  integration tests, UI tests, and doctests; three tests remain ignored by the
  existing suite.
- `cargo clippy -p kmipkit-client --all-targets --all-features -- -D warnings`
  passed.
- `cargo llvm-cov -p kmipkit-client --lib --all-features --text` passed; the
  client `fluent.rs` file reports 97.23 percent line coverage.
- The focused API manifest tests passed, and `tools/api_manifest/generate.py
  --check` verified all 12 generated outputs.
- `audit_rust_exports.py` matched all 9,415 observed declarations; it reported
  zero missing declarations, issues, or kind mismatches.
