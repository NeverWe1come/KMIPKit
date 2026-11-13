# KMIPKIT-0114 Implementation Evidence

## Development commits

- Red: `eada7ab9 test(0114): add red C lifecycle facade consumer`. The compiled
  C consumer called all ten planned symbols before they existed. The focused
  command failed at C compilation with implicit-declaration errors for the
  new functions, as expected.
- Green: `d6baab53 feat(0114): add C lifecycle facade helpers`. Added typed C
  adapters, the feature-owned API capability and declarations, and generated
  public API outputs.
- Refactor: `aa250ef4 refactor(0114): consolidate lifecycle facade validation`. The
  Request Options pointer contract is shared, and the mTLS fixture, traceability,
  and evidence now cover all five operations.
- Follow-up test expansion: the controlled C consumer and mTLS peer now invoke
  all ten entry points with valid inputs. The fixture exercises a nonempty
  Locate filter, optional Revoke message/date, and bounded exchange timeouts.

## Focused verification

| Command | Result |
|---|---|
| `cargo test -p kmipkit-ffi --features coverage-c-consumer --test c_api_coverage public_c_header_constructs_a_batch_and_reaches_the_encoding_override_entrypoint -- --exact` | Passed; compiled C consumer called all ten symbols with invalid inputs and received the expected status without output handles. |
| `cargo test -p kmipkit-ffi --features coverage-c-consumer --test c_api_coverage public_c_fluent_client_executes_lifecycle_helpers_over_mutual_tls -- --exact` | Passed; controlled mTLS fixture decoded and answered all ten helper calls, including a typed Locate filter, optional Revoke message/date, bounded timeouts, and typed response reads. |
| `cargo test -p kmipkit-ffi --all-features` | Passed: 480 unit tests, 56 compiled C ABI tests, and the operation-parity test. |
| `cargo test --workspace --all-features` | Passed for the complete Rust workspace and doctests. |
| `cargo fmt --all --check` | Passed. |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Passed. |
| `cargo llvm-cov --workspace --all-features` | Passed: 91.01% workspace line coverage. `kmipkit-ffi` is 93.15%, and the touched `client_abi/fluent.rs` file is 93.41%. |
| `python3 -B tools/api_manifest/generate.py --check` | Passed; all 12 generated public API outputs are current. |
| `python3 -B tools/api_manifest/audit_binding_exports.py --repo-root . --c-only --check` | Passed; 2,022 exact C export matches and no missing symbols or mismatches. |
| `python3 -B -m unittest scripts.tests.test_requirement_traceability -v` | Passed: 2 tests. |
| `python3 -B -m unittest discover -s tools/normative_catalog/tests -p 'test_feature_traceability.py' -v` | Passed: 31 tests. |
| CI-equivalent nightly AddressSanitizer build and C harness (`ffi-sanitizer-c`) | Passed; instrumented `kmipkit-ffi` built and the compiled C consumer completed with leak detection and halt-on-error enabled. |

## Remaining required checks

The draft PR must remain marked `[WIP]` until all required hosted checks pass on
its exact head SHA. Resolve any red checks before requesting review.

## Hosted CI repair

The first hosted run failed `Verify fluent operation inventory` because the
generated inventory did not include the new compiled C consumer and ABI test
references. Regenerated `specification/catalog/fluent-operation-inventory.json`
with `tools/normative_catalog/generate_fluent_operation_inventory.py`, then
verified the generated output with `--check` and ran
`python3 -B -m unittest tools.normative_catalog.tests.test_fluent_operation_inventory`
(16 tests passed). The full hosted CI rerun is required on the updated head.

The following hosted run exposed a stale T102 manifest-size snapshot:
`test_t102_slice_and_manifest_snapshots_remain_consistent` expected 16,423
declarations while the approved 0114 manifest contains 16,433. The ten added
rows are exactly the new C lifecycle helpers with and without exchange options.
The Red was observed in GitHub Actions and reproduced with
`python3.12 -B -m unittest tools.api_manifest.tests.test_t102_pkcs11_request_trait_exclusions.T102Pkcs11RequestTraitExclusionTests.test_t102_slice_and_manifest_snapshots_remain_consistent -v`.
Green commit `286e97a7` updates the shared T102 snapshot and its rationale.
Verification: all six tests in
`python3.12 -B -m unittest tools.api_manifest.tests.test_t102_pkcs11_request_trait_exclusions -v`
pass. Exact-head CI must pass on the new branch head before this PR is ready.

The follow-up full T102 run also exposed stale non-target declaration and
operation-row fingerprints in
`test_non_target_declarations_and_operation_rows_match_snapshot`: adding the
ten C lifecycle declarations changes those snapshots even though the test's
focused target is the unrelated Certify request builder. The Red was reproduced
locally. Green commit `9d467eb4` refreshes both fingerprints and documents the
five affected operation rows. The focused test
`python3.12 -B -m unittest tools.api_manifest.tests.test_t102_certify_request_builder_mapping.T102CertifyRequestBuilderMappingTests.test_non_target_declarations_and_operation_rows_match_snapshot -v`
passes. The local all-T102 discovery run was interrupted before its summary, so
it is not counted as passing; the hosted checks must validate the complete
suite at the exact PR head.

SonarCloud on head `2130bc05` found 9.3% duplicated new lines in the C fluent
adapter (quality gate: at most 3%). The duplicate blocks were shared option
and identifier validation for five lifecycle requests and repeated symmetric
key builder request extraction. The implementation now centralizes those
paths in `execute_lifecycle_request_with_options` and
`build_symmetric_key_client_request`, preserving each operation's typed
request. Red is the hosted SonarCloud quality-gate failure; Green is the
behavior-preserving helper extraction. Local verification on the refactor:

| Command | Result |
|---|---|
| `cargo test -p kmipkit-ffi --features coverage-c-consumer --test c_api_coverage public_c_fluent_client_executes_lifecycle_helpers_over_mutual_tls -- --exact` | Passed; all ten lifecycle helper variants execute against the controlled mTLS peer. |
| `cargo test -p kmipkit-ffi --features coverage-c-consumer --test c_api_coverage public_c_header_constructs_a_batch_and_reaches_the_encoding_override_entrypoint -- --exact` | Passed; the compiled public C consumer validates the builder execution entry points. |
| `cargo clippy -p kmipkit-ffi --all-targets --all-features -- -D warnings` | Passed. |
| `cargo fmt --all --check` | Passed. |
| `git diff --check` | Passed. |

The same hosted CI run's Ubuntu Python job timed out while fetching the pinned
`uv` version manifest from `raw.githubusercontent.com`, before Python tests
started. This is an infrastructure failure; the next exact-head CI run must
complete the Python setup and tests.

## Exact-head hosted CI correction (2026-11-13)

The hosted run on `91cc9022` failed the workspace test and coverage jobs on
Linux and Windows. The failing invariant was
`every_exported_c_function_starts_inside_the_shared_panic_boundary`; the five
new `*_with_options_v1` lifecycle entry points delegated to the shared helper
without first entering `ffi_entrypoint`. Coverage collection also failed when
the same workspace test failed. The failure was reproduced locally with:

```text
cargo test -p kmipkit-ffi --all-features extension_registry::metadata_tests::every_exported_c_function_starts_inside_the_shared_panic_boundary -- --exact
```

Green wraps Get, Destroy, Activate, Archive, and Recover option-aware exports
in the existing `ffi_entrypoint` panic boundary before they call the shared
lifecycle helper. The focused test now passes (1 passed), and
`cargo fmt --all --check` plus `git diff --check` pass. A full
`cargo test --workspace --all-features --locked` attempt did not yield a
terminal result in this session, so it is not counted as passing. The complete
`kmipkit-ffi` suite did pass locally: 480 unit tests, 56 C ABI integration
tests, and 1 operation-parity integration test. The strict FFI Clippy command
also passed. A fresh hosted run on the corrected head is still required,
including the Linux and Windows workspace and coverage jobs.

## T102 manifest audit follow-up (2026-11-13)

A full local audit of the 108 `test_t102*.py` modules found stale scope
snapshots and incomplete response traceability on the cumulative 0114 branch.
Locate now has 76 declarations because KMIPKIT-0114 adds direct and
exchange-options C helpers. The approved KMIPKIT-0093-FR-008 response contract
was also missing from 25 operation-specific result declarations; the two
`kmipkit::protocol::LocateResponse` aliases now match their canonical protocol
metadata. Updated the non-target declaration and operation fingerprints and
regenerated the public API parity fixtures from `specification/api/public-api.json`.

| Verification | Result |
|---|---|
| Isolated execution of all 108 `test_t102*.py` modules (`python3.12 -B -m unittest`, groups of four modules per process) | Passed: 405 tests, zero failed groups. |
| `python3.12 -B tools/api_manifest/generate.py --check` | Passed; 12 generated outputs verified. |
| `python3.12 -B tools/api_manifest/audit_binding_exports.py --repo-root . --c-only --check` | Passed; 2,022 exact C export matches and no missing symbols or mismatches. |
| Focused Locate, operation-result, optional-structure, response-view, Certify, and KMIPKIT-0093 typed-response tests | Passed: 29 tests. |
| `git diff --check` | Passed. |

The updated branch head still requires a fresh hosted CI run. PR #210 remains
WIP until all required checks pass on that exact head SHA.
