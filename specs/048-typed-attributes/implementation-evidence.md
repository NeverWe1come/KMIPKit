# KMIPKIT-0048 Implementation Evidence

## Red

Red contracts are recorded in separate commits:

- `3763a38a8` adds the typed-attribute generator and protocol API contracts.
- `329170f0f` adds manifest-reconciliation tests.
- `0be449ba0` specifies corrections for inherited KMIPKIT-0047 mask routes.
- `bd892edfb` demonstrates that reconciliation must preserve reviewed manifest
  order. The original test failed while the reconciler sorted existing rows.

The initial focused Rust and generator tests failed because the typed API,
registry collector, and renderer did not exist. Manifest tests also failed on
missing reconciliation, incorrect named-mask exports, and reordering existing
manifest rows.

## Green

`f6aa811db` implements typed access for 62 direct catalogued attribute values.
`8f305cdf5` adds the manifest reconciliation tool and generated binding
inventories. The reconciler adds exact route declarations or
exact trait-projection exclusions, preserves existing manifest ordering, and
fails closed on unknown Rust exports. The three invalid inherited mask-name
routes are removed from Rust/Java/Python mappings; C's single-bit name helpers
remain explicitly C-only. Java `knownBits` and `unknownBits` routes use their
published camelCase names.

- Rust export audit: **6,265 / 6,265 matched**, zero missing, zero unobserved,
  zero kind mismatches, and zero audit issues.
- C/Java/Python export audit: all three scanners complete; every manifest
  declaration matches exactly and there are zero missing, stale, kind-mismatch,
  or audit issues. The checked-in audit snapshot records C 1,701, Java 1,874,
  and Python 1,990 exact declarations.
- API manifest generator `--check`: all 12 generated outputs verified.
- Reconciliation, bounded-manifest, and binding-documentation tests: 15 tests
  passed.
- `cargo test --offline -p kmipkit-protocol --all-features`: 1,273 unit tests,
  all integration tests, and 3 doctests passed.
- `cargo clippy --offline -p kmipkit-protocol --all-targets --all-features -- -D warnings`
  passed.
- `cargo fmt --all --check` passed.
- `cargo clippy --offline -p kmipkit-protocol --all-targets --all-features -- -D warnings`
  passed after the final coverage-focused tests.
- `cargo test --offline -p kmipkit-protocol --all-features` passed after the
  final test additions: 1,273 unit tests, all integration tests, and 3 doctests.
- Full Windows workspace coverage completed with
  `cargo llvm-cov --workspace --all-features --no-clean --json`. After applying
  the repository's exact generated-source exclusions, the measured line
  coverage was TTLV 99.19%, protocol 96.19%, transport 95.16%, FFI 87.71%,
  workspace 93.80%, and changed production lines 100% (78/78). All 124 eligible
  Rust production sources were present in the report. The catalog-generated
  typed marker implementation is excluded by its exact path because its 62
  tag constants are generated from the reviewed catalog, checked against all
  62 runtime marker tags, and its generated output is verified by the pinned
  generator; handwritten typed API code remains in coverage.
- The typed-attribute catalog generator's focused suite passed 14 tests and
  measured 98% line coverage with the pinned coverage tool.
- `python -B tools/normative_catalog/generate_attribute_types.py --check`
  passed for both generated Rust outputs.

## Security review and publication

- Codex Security completed a scoped diff review of 12 changed security
  surfaces with zero reportable findings. The independent architecture
  cross-check verified the Rust attribute-to-client path, manifest readiness
  semantics, secret handling, and transport boundaries. This automated review
  does not replace the independent human security audit required before 1.0.
- PR #115 was opened in draft against `feature/KMIPKIT-0047-named-values`.
  Its local Windows evidence is recorded above; hosted CI and other platforms
  remain pending while the GitHub billing issue is unresolved.

The manifest input is 43,045,067 bytes under the explicit 44 MiB limit. No
raw rustdoc audit report is checked in; the exact report is reproducible with
`tools/api_manifest/audit_rust_exports.py --check`.

## Refactor

The Refactor commit extracts ordered mapping filtering into `_without_names`,
so removing exact unsupported routes cannot reorder the reviewed inventory.
The behavior is covered by `test_preserves_reviewed_manifest_order_for_existing_rows`
and the existing named-mask reconciliation tests. This commit also records the
Red/Green/Refactor and verification evidence in this file.

## Verification

- `cargo fmt --all --check` passed.
- `cargo clippy --offline -p kmipkit-protocol --all-targets --all-features -- -D warnings`
  passed.
- `cargo test --offline -p kmipkit-protocol --all-features` passed: 1,269 unit
  tests, all integration tests, and 3 doctests.
- `python -B -m unittest tools.api_manifest.tests.test_reconcile_0048_typed_attributes tools.api_manifest.tests.test_t102_manifest_size_limit tools.api_manifest.tests.test_c_api_docs -v`
  passed (15 tests).
- `python -B tools/normative_catalog/generate_attribute_types.py --check`
  passed for both generated Rust outputs.
- `python -B tools/api_manifest/generate.py --check` verified all 12 generated
  outputs.
- After merging the current 0047 head (`a2dbcccac`) into this stack at
  `43b6fef5f`, `python -B tools/api_manifest/generate.py --check --repo-root .`
  again verified all 12 outputs, including the regenerated Java public parity
  fixture. `python -B -m unittest tools.api_manifest.tests.test_parity -v`
  passed all 24 parity tests.
- `python -B tools/api_manifest/audit_rust_exports.py --check` passed with
  6,265 exact matches, zero missing routes, zero unobserved declarations, zero
  kind mismatches, and zero audit issues.
- `python -B tools/api_manifest/audit_binding_exports.py --report
  specs/043-public-api-bindings/reports/binding-export-report-2026-11-05.json
  --check` passed for C, Java, and Python.
- The 15 focused reconciliation, size-limit, and binding-documentation tests
  passed again after the Refactor change.
- The final focused run across reconciliation, manifest bounds, binding docs,
  coverage-gate rules, and typed-attribute generation passed 92 tests (3
  Windows symlink tests skipped because the current account lacks the required
  symlink privilege).

## Remaining controls and limitations

Local verification for this specification was performed on Windows. The
broader `test_encrypt_decrypt_parity_manifest` failure described in the
original review was resolved downstream in KMIPKIT-0053: the test now selects
canonical declarations by ID, and the four DecryptRequest helper routes are
attributed to Decrypt with §6.1.11 requirements. The updated five-test module
passes. Cross-platform and remote-CI results remain outside this local
evidence; hosted CI is currently ignored under the maintainer's billing
instruction. This feature remains stacked and is not merged into
`release/1.0.0`.

The correction's separate TDD commits are Red `d3c5cfa79`, Green `7cafcaa02`,
and Refactor `5d942198c`. The Green commit changes the reviewed manifest input
and regenerates parity outputs; the Refactor extracts declaration selection
by public route name. KMIPKIT-0053 inherits the same test, manifest mapping,
and generated fixtures through the approved PR stack.

Remote CI remains unavailable under the maintainer's billing-related
instruction, and full platform/adapter coverage remains outside this local
evidence. This feature is stacked on `feature/KMIPKIT-0047-named-values`; it is
not merged into `release/1.0.0`.

## Release-stack synchronization (2026-11-07)

Merged the current 0047 release-stack head
`3b0e3143978792efc5889841af0a4b4ded879469` into this feature branch. The
resolution keeps the 2026-11-07 binding audit report and documentation from
0047, combines both features' generated-source coverage exclusions, and
preserves the four Decrypt capability and normative-requirement corrections.
No declaration or exclusion export key from either parent is lost.

The final manifest can be checked for idempotence using its tracked post-sync
audit with
`python -B tools/api_manifest/reconcile_0048_typed_attributes.py --report specs/048-typed-attributes/reports/rust-export-report-post-sync-2026-11-07.json --write`.
That final audit has no missing routes; it is not a reconstruction input.
The tracked pre-reconciliation report and its reproducible derivation and
isolated restoration command are documented in [the audit evidence guide](reports/README.md).
All 12 outputs were regenerated with
`python -B tools/api_manifest/generate.py --write`. The resulting inventory
has 11,874 public declarations and 16,818 approved exclusions. The T102
declaration and other-manifest fingerprints now pin this combined inventory;
the operation fingerprint and normalization logic are unchanged.

Fresh local verification:

- `cargo fmt --all --check` passed.
- `cargo clippy --offline --locked -p kmipkit-protocol --all-targets --all-features -- -D warnings` passed.
- `cargo test --offline --locked -p kmipkit-protocol --all-features` passed:
  1,277 unit tests, all integration tests, and 3 doctests.
- `python -B -m unittest tools.api_manifest.tests.test_reconcile_0048_typed_attributes tools.api_manifest.tests.test_t102_manifest_size_limit tools.api_manifest.tests.test_c_api_docs tools.api_manifest.tests.test_t102_response_view_mappings tools.api_manifest.tests.test_encrypt_decrypt_parity_manifest tools.api_manifest.tests.test_047_binding_audit_reconciliation tools.api_manifest.tests.test_t102_remaining_response_view_mappings -q`
  passed all 32 tests after refreshing the combined-inventory snapshots.
- `python -B -m unittest tools.normative_catalog.tests.test_generate_attribute_types tools.normative_catalog.tests.test_generate_named_values scripts.tests.test_coverage_gate -q`
  passed 92 tests with 3 Windows symlink-privilege skips.
- `python -B tools/normative_catalog/generate_attribute_types.py --check`
  verified both outputs, and
  `python -B tools/normative_catalog/generate_named_values.py --check`
  verified all four language outputs.
- `python -B tools/api_manifest/generate.py --check` verified all 12 outputs.
- Repeating the 0048 reconciliation without `--write` reported zero added
  declarations and zero added exclusions.
- `python -B tools/api_manifest/audit_rust_exports.py --report specs/048-typed-attributes/reports/rust-export-report-post-sync-2026-11-07.json --check`
  passed with 21,507 observed routes, 6,285 exact matches, 15,222 exclusions,
  and zero missing, unobserved, kind, conditional, or unsupported-scope issues.
- `python -B tools/api_manifest/audit_shared_rust_inventory.py` generated the
  paired 0048 inventory and its `--check` passed. The mapped shared inventory
  remains 3,697 declarations; typed attribute parity stays blocked for 0053.
- The expanded parity/shared-inventory run initially passed 43 of 52 tests
  and raised nine errors because it combined the historical 0047 Rust audit
  with the current 0048 manifest. The official auditors now use the two new
  `post-sync` snapshots under this feature's reports directory. Current-source
  tests pin their exact counts and paths; the 0047 Rust, shared-inventory, and
  binding snapshots remain byte-identical to the incoming parent.
- `python -B -m unittest tools.api_manifest.tests.test_parity tools.api_manifest.tests.test_catalog_parity tools.api_manifest.tests.test_t109_binding_export_audit_reconciliation tools.api_manifest.tests.test_shared_rust_inventory -q`
  then passed all 52 tests in 116.861 seconds, without failures or errors.
- `python -B -m unittest tools.api_manifest.tests.test_manifest_v2.PublicApiManifestV2SchemaTests.test_checked_in_manifest_inventories_all_rust_public_access_paths -q`
  passed against the new Rust snapshot, including all 33 conditional Any
  lifetime routes and exact exclusion reconciliation.
- The whitespace check of all conflict-resolution files passed. The full
  merge diff retains two pre-existing whitespace diagnostics from the incoming
  release stack: a final blank line in
  `interop/c-kmip-validator/src/c_validator.c` and trailing whitespace in
  `interop/shared/vectors/generate_vectors.py`. Neither file was edited by
  this resolution.

The local untracked Rust audit report and all target directories were
preserved. No hosted CI status was queried, and this synchronization does not
add fresh coverage or cross-platform evidence.

## Reconciler review follow-up (2026-11-07)

The review identified that the reconciler accepted every missing `<...>`
associated-item path as a 0048 trait exclusion before checking its owner.
The correction validates the exact package namespace and visible attribute
owner before producing either an exclusion or a declaration. A typed value
appearing only in a trait argument does not establish ownership.

- RED `34d14bfc6`: the minimal transport
  `<kmipkit_transport::TlsConfig as core::fmt::Debug>::fmt` regression failed
  with `ReconciliationError not raised`.
- GREEN `957eaf5f8`: all 12 reconciler tests passed, including foreign owners
  in accepted packages, package/crate mismatches, root-name collisions, typed
  trait arguments, and valid typed projections in both published crates.
- REFACTOR: the package namespaces and exact root-type names are centralized
  in constants; ownership and reconciliation behavior remain unchanged.

Fresh verification after the refactor:

- The focused reconciliation, manifest size, C documentation, T102 response
  views, Encrypt/Decrypt, and 0047 binding-reconciliation suites passed all
  35 tests.
- Reconstruction of the 286 typed declarations and 2,072 0048 trait exclusions
  is verified using the tracked pre-reconciliation report and official
  comparison/reconciliation procedure in [the audit evidence guide](reports/README.md).
- `cargo fmt --all --check` passed.
- `python -B tools/api_manifest/generate.py --check` verified all 12 outputs.
- The exact Rust audit ran with the pinned offline compiler and a temporary
  report destination. It passed with 21,507 observed routes, 6,285 exact
  matches, 15,222 exclusions, and zero missing, unobserved, kind, conditional,
  or inventory issues.
- The documented idempotence command above was run in a temporary export
  made with `git checkout-index --all`. That export contained only tracked
  files and lacked the pre-existing untracked report. The command exited 0,
  reported zero added declarations/exclusions, and preserved the complete
  manifest content.
- Review confirmed that only the reconciler, its tests, and this evidence
  document changed; `git diff --check` passed. The existing untracked report,
  target directories, and all generated artifacts were preserved. No hosted
  CI status was queried.

## Scoped ownership and reconstruction follow-up (2026-11-07)

The broader `attribute::` namespace also contains earlier generic attribute
APIs. Those exports are not owned by 0048 merely because they share a module.

- RED `70d917aa8`: the two regressions for the `ClientAttributeMutation`
  Debug projection and `client_vendor_attribute_mutation_is_prohibited`
  failed with `ReconciliationError not raised`.
- GREEN `d5c070936`: the filter now accepts the `attribute::typed` marker
  namespace and exact permitted root-type names/members, including the
  `attribute::` aliases of those types. All 14 reconciler tests pass; the
  unrelated projection and function are rejected.
- REFACTOR: the marker namespace is centralized as a named scope constant.
  The existing 286 typed declarations and 2,072 exact projections remain
  unchanged.

The final post-sync report has no missing routes and proves final audit
consistency and idempotence only. The tracked pre-reconciliation report is
instead derived with the official auditor's `compare` implementation, using
the same compiler observations against a manifest with exactly the 0048
declarations/exclusions removed. Its deterministic input and complete
restoration command are recorded in [the audit evidence guide](reports/README.md).

The guide's Python block was executed verbatim in an export containing only
tracked files. It regenerated a byte-identical pre-reconciliation report and
invoked the official reconciler against a temporary baseline, restoring
`+286 declarations, +2072 exclusions` and the complete original manifest.
The historical untracked audit and final post-sync reports were untouched.

Fresh verification also passed all 14 focused reconciler tests,
`cargo fmt --all --check`, the 12-output API generator `--check`, and the
official exact Rust audit. That final audit still reports 21,507 observed
routes, 6,285 exact matches, 15,222 exclusions, and zero discrepancies.
`git diff --check` passed. No hosted CI was queried and no other feature
worktree, PR metadata, or branch alias was changed.

## SonarCloud quality gate remediation for PR #140 (2026-11-08)

The current KMIPKIT-0048 head failed SonarCloud on one unreliable determinism
assertion and duplicated lines in generated typed-attribute implementations,
the `CurrentAttribute`/`NewAttribute` wrappers, and the 0048 manifest CLI.

- RED `4fbbec344`: the generator test failed because the rendered source did
  not define the shared attribute macro, and the new shared-CLI contract tests
  failed because the runner module did not yet exist.
- GREEN `ddda3419d`: the generator now emits each catalogued marker through a
  shared codec implementation and regenerates the Rust output. The 0046 and
  0048 reconcilers now use a common CLI runner while preserving their output
  messages and write behavior.
- REFACTOR `410b48b18`: the two public attribute wrappers share their methods
  and redacted Debug implementation; the determinism test compares results
  from independent input copies; the superseded full-implementation renderer
  was removed.

Verification after the refactor:

- `cargo test -p kmipkit-protocol` passed all 1,327 unit tests and all
  integration, guide-example, and doctests.
- `cargo test -p kmipkit-protocol --lib typed_attribute_tests` passed all 15
  typed-attribute tests.
- The focused generator, shared CLI, 0048 reconciler, and 0046 reconciler
  Python suites passed all 29 tests.
- `python -B tools/normative_catalog/generate_attribute_types.py --check`
  verified both generated Rust files.
- `cargo fmt --all --check`,
  `cargo clippy -p kmipkit-protocol --all-targets --all-features -- -D warnings`,
  and `git diff --check` passed.
- The workspace-wide Clippy attempt could not complete on this Windows host:
  `aws-lc-sys` reported that NASM was unavailable. The affected protocol crate's
  all-target, all-feature Clippy check passed.

## Linux literal-generator coverage remediation (2026-11-08)

The stacked hosted run found that
`tools/normative_catalog/generate_attribute_types.py` had 262 statements and
28 uncovered statements (89%), below the unchanged 90% normative-generator
gate.

- RED `7c96c4a61`: the focused Rust-string test covered only quotes and
  backslashes. Its Linux coverage result remained below the 90% gate.
- GREEN `338cd7c86`: the same test now exercises newline, carriage return,
  tab, control byte, DEL, and Unicode handling. On the current 0048 head, the
  Linux coverage report is 262 statements, 27 missed, 90%; the eight tests
  other than the mount-sensitive direct-CLI test pass under WSL.
- REFACTOR `51734bea1`: the test name now describes its complete escape
  coverage. The complete nine-test module passes on Windows.

The WSL checkout is mounted under `/mnt/c`; in that environment,
`test_direct_cli_check_exercises_script_import_path` returns exit code 2. This
mount-specific result does not reproduce on Windows, where all nine tests pass,
or in the earlier native hosted run, whose only reported failure was the
89-percent coverage threshold. The current 0048 HEAD still needs a fresh hosted
CI result; prior queued runs were for superseded SHAs and were cancelled.

## Stacked-base refresh and T102 snapshot reconciliation (2026-11-08)

PR #140 became unmergeable after its 0047 base advanced. The 0048 worktree now
merges the current 0047 head with a regular merge commit; no force push was
used. That merge exposed two stale non-target declaration fingerprints in the
Certify and optional-request T102 tests.

- RED: after merge commit `965223f5f`, the six affected T102 modules ran 28
  tests and failed only the two stale snapshot assertions. The observed
  fingerprints were `6c73940a...` for Certify and `b77c675a...` for optional
  request structures; the previous values represented the pre-0047 stack.
- GREEN `1e310bd24`: the two snapshots now pin the merged 0047+0048 manifest.
  All 28 tests across the six affected modules pass.
- Verification: `tools/api_manifest/generate.py --check` verified all 12
  generated outputs, and `git diff --check` passed. The manifest outputs did
  not need regeneration.

The updated 0048 head must receive hosted CI after this merge-conflict repair;
the previously queued workflows covered earlier SHAs.

## KMIPKIT-0048 link normalization correction (2026-11-08)

This correction follows pinned OASIS KMIP 2.1 §4.31 Table 90 and §11.28
Table 465, plus KMIP 3.0 §§4.35.1–4.35.20 and §11.58. In particular, KMIP 3.0
link attribute encodings list Reference, Name Reference, Unique Identifier
Enumeration, or Integer; they do not list Text String. The converter therefore
preserves Enumeration and Integer values and rejects Text String and all
unrepresentable identifier forms before emitting a result.

- **Red:** `a20fe669a`, `8ce6503b9`, `2a8dc8b7b`, and `587d9086a`. The final
  focused Red test failed because the implementation accepted a Text String
  identifier for a KMIP 3.0 Certificate Link; the pinned §4.35 encoding does
  not permit it.
- **Green:** `3296bc6bd` adds the checksum-pinned 20-tag KMIP 3.0 catalog,
  merges exact assignments into the supported-version allocation union,
  replaces range-derived LinkType mappings with the 14 explicit semantics,
  and rejects lossy identifier conversions and unknown children.
- **Refactor:** `9c7dc5a15` records the exact source-derived conversion
  boundary in the specifications, traceability, migration note, and evidence;
  code and generated output are unchanged.

Local verification on the Green code:

- `cargo test -p kmipkit-protocol --lib link_normalizer_tests -- --nocapture`:
  6 passed.
- `cargo test -p kmipkit-ttlv --test tag_allocation`: 8 passed.
- `cargo test -p kmipkit-protocol --all-features`: 1,331 unit tests, all
  integration tests, and 3 doctests passed.
- `cargo clippy -p kmipkit-protocol --all-targets --all-features -- -D
  warnings` and `cargo fmt --all --check` passed.
- `python -B -m unittest tools.normative_catalog.tests.test_generate_ttlv_tags
  -q`: 17 passed; `python -B tools/normative_catalog/generate_ttlv_tags.py
  --check` passed.
- `python -B tools/api_manifest/generate.py --check --repo-root .` verified all
  12 generated API outputs; named-value manifest reconciliation and
  `git diff --check` passed.
- The Rust export auditor ran with zero incomplete-analysis issues and all
  6,353 existing manifest declarations matched. It also reported 4,201 public
  Rust declarations outside the manifest; this broader inventory remains in
  KMIPKIT-0053's parity/reconciliation scope and is not represented here as
  resolved.

The full Windows command `cargo llvm-cov --workspace --all-features --locked
--json --output-path target/kmipkit-workspace-coverage-raw.json` passed. Its
report was normalized using `scripts/coverage_gate.py normalize`, then parsed
with the gate's own `parse_llvm_export` and exact documented-source exclusion
logic. KMIP protocol coverage was 21,027/21,833 (96.31%), and the changed
`link_normalizer.rs` executable lines were 145/147 (98.64%). This meets the
95-percent protocol and changed-code thresholds. The separate package-only
summary was not used because its partial export lacks workspace dependency
segments required by the coverage parser.

Hosted PR checks are still pending on the updated head; T019 and TASK-071-008
remain open until those controls finish.
