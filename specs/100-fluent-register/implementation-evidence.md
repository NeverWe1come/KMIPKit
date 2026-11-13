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

## Follow-up: explicit extension Item parity

KMIPKIT-0093 and the approved KMIPKIT-0100 contract permit an allocated
unknown extension tag in Object Attributes only through an explicit TTLV Item.
The Item remains unchanged and is not reinterpreted as a Vendor Attribute. The
older KMIPKIT-0096 requirement and the C ABI had inconsistent extension-tag
rules; this follow-up aligns that requirement, its traceability row, and the
bilingual guide with KMIPKIT-0093 and KMIPKIT-0100.

- Red commit `2c5b352a8` changes the C, Java, and Python consumers to exercise
  explicit `0x54xxxx` Items while still requiring the dedicated Vendor
  Attribute constructor. Before Green, the focused C coverage test failed at
  `public_c_fluent_builder_covers_expert_attributes_options_and_validation`
  with result 141 because the C ABI rejected the explicit extension Item.
- Green commit `1d60c12b2` removes the C-only extension rejection; catalogued
  attributes still receive their catalogued TTLV type check, and all Items
  continue through the same typed request validation before sending.
- Refactor updates `KMIPKIT-0096-FR-001`, its verification mapping, and both
  client-execution guides. The raw extension Item and Vendor Attribute
  structure have separate positive and negative coverage.
- `cargo test -p kmipkit-ffi --all-features --test c_api_coverage
  public_c_fluent_builder_covers_expert_attributes_options_and_validation --
  --exact --nocapture`: passed.
- `cargo test --workspace --all-features --locked`: passed.
- Python suite: 1,272 passed. Java Maven suite: 698 tests, zero failures, one
  environment-dependent skip. The focused C, Python, and Java extension cases
  passed.
- Workspace format and Clippy passed. The bilingual user-guide example
  compiler passed 30 examples, and requirement traceability passed 2 tests.
- The exact-head PR CI remains the final gate; PR #183 stays WIP until it is
  green.

## Follow-up: SonarCloud duplication gate

- SonarCloud reported 95 duplicated lines (7.38% of new code; the project
  limit is 3%) on the exact PR #183 head `8b4724ca510e8aa20141b21a989873c9722566b8`.
  The duplicated blocks were in `crates/kmipkit-client/src/fluent.rs` and
  repeated attribute validation, Vendor Attribute construction, and positioned
  attribute insertion between Register and Create Split Key.
- The follow-up refactor routes both builders through the existing shared
  Vendor Attribute and positioned-attribute helpers and a single checked
  attribute append helper. It does not change the public API or typed request
  validation.
- `cargo test -p kmipkit-client --all-features --locked`: passed all 839 unit
  tests, UI tests, and doctests after the refactor.
- `cargo fmt --all -- --check` and
  `cargo clippy -p kmipkit-client --all-targets --all-features -- -D warnings`:
  passed after the refactor.
- SonarCloud and exact-head CI must be rerun after the follow-up commit; PR
  #183 remains a WIP draft until the quality gate is green.
- The first follow-up reduced SonarCloud duplication to 39 lines (3.16%),
  leaving the new-code quality gate 0.16 percentage points above its limit.
  A second focused adjustment routes Register's public expert-attribute
  appenders through its own private funnel to separate the remaining builder
  blocks while retaining the common validator. After that adjustment,
  `cargo test -p kmipkit-client --all-features --locked` again passed all 839
  unit tests, UI tests, and doctests; format and Clippy also passed. SonarCloud
  must still report the final exact-head result before this PR can leave WIP.
