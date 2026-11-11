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
- Focused Register tests: 19 passed, covering typed and unknown object inputs,
  attribute order and placement, optional fields, payload-free local errors,
  typed batch construction, Success/Failure/Pending outcomes, one exchange,
  correlation, and sentinel-byte redaction. Additional tests cover all four
  KMIP 3.0 credential object types, one explicit KMIP 3.0 Register exchange,
  rejection of the assigned non-attribute Operation tag before transport, and
  rejection of unassigned tags by the TTLV allocation check. Explicit Items in
  the allocated `0x54xxxx` extension range remain the same raw tag and TTLV
  type through Create, Create Key Pair, Create Split Key, and Register; they
  are never converted into Vendor Attributes.
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
  40 exact public declarations and 18 exact standard-trait exclusions. The
  targeted downstream T102 mapping/snapshot suites passed 20 tests.
- `python3 tools/normative_catalog/generate_attribute_types.py --check` verified
  seven generated outputs, including the new private fluent attribute-tag
  catalog. Its generator suite passed all 11 tests.
- The bilingual user-guide examples compiled: 22 marked Rust examples from the
  English and Spanish lifecycle guides.
- `cargo test -p kmipkit-client --all-features --quiet` passed all 839 unit
  tests; the remaining integration, UI boundary, and documentation tests also
  passed. `cargo llvm-cov -p kmipkit-client --all-features --json --summary-only`
  reported 95.72% line coverage for the client crate and 95.15% for
  `src/fluent.rs`.
- Rust export audit snapshot completed with no unsupported issues. All 40
  KMIPKIT-0100 manifest declarations matched exact compiler-observed paths or
  aliases. The repository-wide `--check` remains incomplete on this source
  snapshot (4,938 other paths missing from the manifest and one stale
  declaration); this feature does not reconcile unrelated API inventory.
- Example compilation emits two existing `dead_code` warnings for private
  helpers in `execute.rs`; the new builder examples themselves compile.

## Shared Manifest Snapshot Update

The Register API intentionally adds 40 public declarations and 18 exact
standard-trait exclusions. Updated the historical T102 full-manifest totals
from the #181 base (13,980 declarations / 19,000 exclusions) to 14,020 / 19,018,
and refreshed the three downstream fingerprint suites that include all
non-target declarations. The focused T102 tests pass with those new entries
present; no operation or protocol mapping snapshot changed. Register also
validates expert attributes against the generated source-backed attribute tag
catalog, so a known non-attribute tag cannot pass into `AttributeSet` just
because its TTLV type is valid. The same guard now covers Create, Create Key
Pair, Create Split Key, and Register fluent builders. The attribute-type
generator produces and checks the private client-side catalog output.

The KMIP 3.0 credential model already represented Object Types 13–16, but the
shared object-tag matcher omitted their tags. Added the four mappings so the
Register validator accepts Password, Device, One Time Password, and Hashed
Password Credential objects when KMIP 3.0 is explicitly selected. The spec now
cites §6.1.50 Tables 400–402: Register accepts Any Object (Section 2), which
includes these system credentials.

## Commit record

- Red: `55fb03b3c` (`test: specify fluent register builder`).
- Green: `3a8d32d7a` (`feat: add fluent register builder`).
- Refactor: `6e66f14b1` (`docs: record fluent register API evidence`); this
  follow-up records the rebased commit identities and dependency head.
