# KMIPKIT-0096 Delivery Tasks

- [x] T001 Add the failing insertion-path and position validation tests.
- [x] T002 Commit Red evidence separately from production code; the focused
  command failed at compile time because the planned public type and method
  are not implemented yet.
- [x] T003 Implement the typed Object Attributes path, ordered insertion, and
  payload-free position errors.
- [x] T004 Document public Rust APIs and update generated API manifest outputs.
- [ ] T005 Pass focused tests, crate tests, format, Clippy, traceability,
  manifest checks, and PR CI; mark ready only after green CI.
- [ ] T006 Carry equivalent insertion semantics into dependent C, Java, and
  Python binding deliveries.

## Local Verification Evidence

- `cargo test -p kmipkit-client --all-targets --locked --quiet`: 787 library
  tests passed; all invoked integration and UI tests passed (three tests were
  intentionally ignored).
- `cargo clippy -p kmipkit-client --all-targets --all-features --locked --
  -D warnings`: passed.
- `cargo fmt --all -- --check`, `git diff --check`, and
  `python3 -B tools/api_manifest/generate.py --check`: passed.
- `python3 -B tools/normative_catalog/validate.py --structural-only` and the
  feature traceability suite: passed (31 tests).
- Rust export report maps all four new declarations with zero audit issues.
  The repository-wide audit still reports 4,938 unrelated existing declarations
  missing from its manifest; these are not introduced by this delivery.
- PR CI remains required before T005 can be completed and this PR marked ready.

## Dependent Binding Delivery Evidence

- C exposes `kmipkit_symmetric_key_builder_insert_attribute_item`; the compiled
  C consumer builds a valid positioned request and rejects an out-of-range
  position. `cargo test -p kmipkit-ffi --all-features --locked` passed 471
  unit tests, 56 C API tests, and the 56-operation parity test on the C branch.
  Its exact-head hosted CI is still pending.
- Java exposes `SymmetricKeyBuilder.attributeItemAt(int, TtlvItem)` and the
  focused `FluentFacadeShapeTest` passes, including negative and out-of-range
  position rejection. Red, Green, and Refactor evidence are commits
  `6f58e371e`, `14b338716`, and `ffc996f18` in the dependent Java delivery.
- Python binding parity and cross-language response views remain outstanding;
  T006 stays open until those dependent deliveries pass exact-head CI.
