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
