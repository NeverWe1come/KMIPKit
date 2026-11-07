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

The earlier statement that other platform results were outside this local
evidence describes the original Windows-only 048 snapshot. Later synchronized
Linux workspace, coverage, and adapter results that include this change are
recorded in `specs/053-api-parity-facade/implementation-evidence.md`.
Windows Java adapter coverage has also been collected. The macOS Rust platform
report, complete cross-platform aggregate, and hosted CI remain outstanding.
