# KMIPKIT-0100 Implementation Evidence

## Branch and dependency

- Branch: `feature/KMIPKIT-0100-fluent-register`.
- Base: PR #181 head `d1eaf235a6aa39fa5db4a53e0562f1bdddf5afe6` after rebase.
- Scope: Rust fluent Register builder only; C, Java, and Python parity is
  recorded as pending in the public API manifest.

## Red

- Commit `55fb03b3c` adds the approved feature specification, traceability,
  plan, and focused Register tests before production code.
- `cargo test -p kmipkit-client --all-features fluent_register_tests -- --nocapture`
  failed as expected because `RequestBuilder::register`, `RegisterBuilder`,
  and the two Register-specific build errors did not yet exist.

## Green

- Added the typed `RegisterBuilder` and `RequestBuilder::register()` with
  explicit typed and generic object inputs, attributes, optional masks and
  Opaque Object context, validation/build/batch conversion, and a single
  existing typed exchange.
- The builder delegates object, Table 293, version, and context validation to
  `RegisterRequest`; errors and debug representations do not include payloads.
- Focused Register tests: 14 passed, covering typed and unknown object inputs,
  attribute order and placement, optional fields, payload-free local errors,
  typed batch construction, Success/Failure/Pending outcomes, one exchange,
  correlation, and sentinel-byte redaction.
- `cargo test -p kmipkit-client --all-features`: 834 unit tests passed; the
  remaining integration, UI boundary, and documentation tests also passed.
- `cargo clippy -p kmipkit-client --all-targets --all-features -- -D warnings`:
  passed.
- The AST boundary guard initially rejected the newly authorized, explicit
  `Item` signatures. Its exact method-scoped exceptions are covered by a
  failing-then-passing signature test; the guard still rejects unbounded or
  mis-typed signatures. The full client suite passed after the guard update.

## Refactor and contract evidence

- `cargo fmt --all -- --check` and `git diff --check` passed.
- `python3 tools/api_manifest/generate.py --check` verified 12 generated API
  outputs. `test_0100_fluent_register_manifest.py` passed all 3 tests, including
  40 exact public declarations and 18 exact standard-trait exclusions.
- The bilingual user-guide examples compiled: 22 marked Rust examples from the
  English and Spanish lifecycle guides.
- `cargo llvm-cov -p kmipkit-client --all-features --json --summary-only` ran
  the full client suite and reported 95.66% line coverage for the client crate
  and 94.61% for `src/fluent.rs`.
- Rust export audit snapshot completed with no unsupported issues. All 40
  KMIPKIT-0100 manifest declarations matched exact compiler-observed paths or
  aliases. The repository-wide `--check` remains incomplete on this source
  snapshot (4,938 other paths missing from the manifest and one stale
  declaration); this feature does not reconcile unrelated API inventory.
- Example compilation emits two existing `dead_code` warnings for private
  helpers in `execute.rs`; the new builder examples themselves compile.

## Commit record

- Red: `55fb03b3c` (`test: specify fluent register builder`).
- Green: `3a8d32d7a` (`feat: add fluent register builder`).
- Refactor: `6e66f14b1` (`docs: record fluent register API evidence`); this
  follow-up records the rebased commit identities and dependency head.
