# KMIPKIT-0093 Implementation Evidence

## KMIPKIT-0122 binding audit report regeneration

### Red

- On PR #221 head `3595d0c8cf416b9bb9b024cac9f86fe5c938de87`, ran:
  `python3 -B -m unittest tools.api_manifest.tests.test_c_api_docs -v`
- The four tests failed because the checked-in binding export report predates
  the versioned catalog additions. Its per-package manifest counts were short
  by 62 declarations: C `2,002` versus `2,064`, Java `2,406` versus `2,468`,
  and Python `2,617` versus `2,679`. The dedicated C report check failed on
  the same stale count.
- The finding is a stale generated report and documentation snapshot; the
  manifest and binding declarations are already present on this branch.

### Green

- Regenerated the current C, Java, and Python export audit with:
  `python3 -B tools/api_manifest/audit_binding_exports.py --repo-root . --report specs/043-public-api-bindings/reports/binding-export-report-2026-11-15-versioned-catalogs.json`
- The generated report is complete for every package, with zero missing,
  stale, kind-mismatched, or otherwise unclassified exports. Its manifest
  counts are C `2,064`, Java `2,468`, and Python `2,679`.
- Updated the current-report references and documentation snapshot to the
  generated inventory. The previous dated report remains unchanged as history.
- `python3 -B -m unittest tools.api_manifest.tests.test_c_api_docs -v` passes
  all 4 tests. Regenerating the inventory with the documented `--check`
  command also passes without changing the checked-in report.

### Refactor

- Extracted the shared manifest-to-package declaration grouping used by both
  report assertions, keeping the blocked C declaration rule in one place.
- Re-ran `python3 -B -m unittest tools.api_manifest.tests.test_c_api_docs -v`;
  all 4 tests pass.

## KMIPKIT-0124 manifest snapshot repair

### Red

- On PR #223 head `f57660fa60be9d9c86161204333169e24d602136`, CI run
  `43700578979` failed `Script contracts / windows-latest` in
  `test_t102_slice_and_manifest_snapshots_remain_consistent`.
- Reproduced with:
  `python3 -B -m unittest tools.api_manifest.tests.test_t102_pkcs11_request_trait_exclusions.T102Pkcs11RequestTraitExclusionTests.test_t102_slice_and_manifest_snapshots_remain_consistent`
- The test reported `AssertionError: 16816 != 16772`. Comparing the public API
  manifest against the PR base identified 44 added declarations: 28 KMIP 3.0
  fluent operations and 16 asynchronous batch bindings. The inherited T102
  public-declaration total had not been refreshed for those approved additions.

## KMIPKIT-0124 binding audit report regeneration

### Red

- After rebasing PR #223 on updated PR #221 at local head
  `adf5eaecc3a3c4554a8906d340fc353726fe7f1f`, ran:
  `python3 -B -m unittest tools.api_manifest.tests.test_c_api_docs -v`
- The four tests failed because the inherited report contains the KMIPKIT-0122
  base counts but omits 11 KMIPKIT-0124 declarations in each binding package:
  C `2,075` versus `2,064`, Java `2,479` versus `2,468`, and Python `2,690`
  versus `2,679`.
- The generated audit then identified 11 unclassified public JNI bridge methods
  and four Java facade mappings that name `createGroupFromAttributes`,
  `createUserFromAttributes`, `deactivateFromFields`, and
  `obliterateFromIdentifier`, while the source exposes those typed operations
  as overloaded draft method names. The exact bridge symbols need approved
  internal exclusions, and the semantic facade names need to match the
  approved manifest mappings.
- Added manifest/documentation assertions and changed the Java consumer test to
  call the manifested method names. Red evidence:
  `python3 -B -m unittest tools.api_manifest.tests.test_c_api_docs -v` fails
  with the stale Java audit and the new method/exclusion assertions;
  `bash ./gradlew test --tests org.kmipkit.extensions.Kmip30SemanticBatchItemTest`
  fails compilation because those four manifested semantic methods are absent.
- The expanded T102 run then exposed a traceability mismatch on the public
  `kmipkit::protocol::LocateResponse::try_from_response_item` re-export:
  its exact `rust-kmipkit` alias omits KMIPKIT-0093-FR-005 and FR-008 present
  on the canonical `rust-kmipkit-protocol` declaration. Reproduction:
  `python3 -B -m unittest discover --failfast -s tools/api_manifest/tests -p 'test_t102*.py'`
  fails `test_locate_client_declarations_exclude_server_only_size_requirement`.
- The same alias test subsequently exposed KMIPKIT-0093-FR-008 missing from
  the public `kmipkit::protocol::LocateResponse::result` alias while present
  on its canonical protocol declaration.
- Continuing the same T102 suite exposed a second response traceability gap:
  `kmipkit::DelegatedLoginResponse::result` omits KMIPKIT-0093-FR-008, while
  its shared `ClientResponseView` carries it. The suite fails
  `test_exact_operation_result_declarations_map_to_shared_metadata` for
  `KMIPKIT-ELEM-OP-C2S-DELEGATED-LOGIN`.
- The focused response-mapping test confirms the same missing FR-008 on 23
  additional operation-result declarations; each paired response view has
  that requirement and no other requirement-set difference. Reproduction:
  `python3 -B -m unittest tools.api_manifest.tests.test_t102_operation_result_mappings -v`.
- The full T102 run also found one historical non-target manifest fingerprint
  that must include KMIPKIT-0124's 44 approved declarations and the response
  traceability corrections. It fails
  `test_all_non_target_declarations_and_operations_match_the_pinned_snapshot`
  in `test_t102_optional_request_structure_mapping.py`.

### Green

- Updated the shared T102 declaration snapshot to 16,816 and recorded the
  KMIPKIT-0124 additions that account for the increase.
- The focused reproduction command above passes (`Ran 1 test`, `OK`).

### Refactor

- Split the current public-manifest count assertions from the historical Rust
  audit totals so future approved public API additions do not obscure the
  unchanged Rust audit snapshot.
- `python3 -B -m unittest tools.api_manifest.tests.test_t102_pkcs11_request_trait_exclusions`
  passes all 7 tests.

### Green follow-up after the response and export corrections

- Regenerated the 12 public API manifest outputs with
  `python3 -B tools/api_manifest/generate.py --write`; the subsequent
  `python3 -B tools/api_manifest/generate.py --check` verified all 12 outputs.
- Reconciled `LocateResponse::try_from_response_item` and
  `LocateResponse::result` alias requirements with their canonical protocol
  declarations. Added KMIPKIT-0093-FR-008 to the 25 operation-result
  declarations whose paired response views already expose it.
- Updated the non-target T102 fingerprints and exclusion count for the
  KMIPKIT-0124 additions. The exact focused regression run
  `python3 -B -m unittest tools.api_manifest.tests.test_t102_locate_request_builder_mapping tools.api_manifest.tests.test_t102_certify_request_builder_mapping tools.api_manifest.tests.test_t102_optional_request_structure_mapping.T102OptionalRequestStructureMappingTests.test_all_non_target_declarations_and_operations_match_the_pinned_snapshot tools.api_manifest.tests.test_t102_operation_result_mappings`
  passes all 17 tests.
- Renamed the Java semantic factories to the manifest names while retaining
  the separate TTLV overloads. The focused Java test
  `bash bindings/java/gradlew --project-dir bindings/java --no-daemon --console=plain --quiet test --tests org.kmipkit.extensions.Kmip30SemanticBatchItemTest`
  passes.
- The updated binding report records complete scans with no issues or missing,
  stale, or kind-mismatched exports: C 2,135 observed / 2,075 matched, Java
  3,190 / 2,479, and Python 3,699 / 2,690. The documented C and binding audit
  consistency tests pass (`python3 -B -m unittest
  tools.api_manifest.tests.test_c_api_docs -v`, 6 tests).
- `git diff --check` passes. The full T102 discovery run was interrupted before
  it returned a result; it is not counted as passing evidence.

### Refactor

- Reviewed the Java semantic factories and manifest renderer after the green
  run. The factories already share `systemObjectFromAttributes`, and the
  renderer remains the single source for generated outputs, so no additional
  production refactor was warranted.
- Re-ran the 17 focused T102/manifest regression tests, the six binding report
  tests, and the focused Java semantic batch-item test after that review; all
  pass.

## KMIPKIT-0124 follow-up after exact-head CI run 43721677012

### Red

- The exact PR #223 head `f882f8d73d89ac81eeacddd115606e10552de0e5` failed
  `Script contracts / ubuntu-latest` in
  `test_internal_bridge_and_dataclass_artifacts_use_exact_exclusions` because
  the expected JNI bridge inventory omitted 11 KMIPKIT-0124 bridge symbols.
  Reproduction before the fix:
  `python3 -m unittest tools.api_manifest.tests.test_binding_exports.BindingExportAuditTests.test_internal_bridge_and_dataclass_artifacts_use_exact_exclusions`.
- Extending the expected inventory then exposed that the audit accepted the
  older bridge requirements but not the approved KMIPKIT-0093-FR-011 ownership
  evidence. The test failed at its exact evidence check until that requirement
  was included.
- The same run failed the changed-production coverage gate at 502/578 lines
  (86.85%). Its macOS and Windows script-contract jobs were cancelled after the
  Ubuntu matrix failure; they are not independent product failures.

### Green

- The exact bridge inventory now includes the 11 KMIPKIT-0124 JNI dispatch
  methods, and the binding audit accepts their KMIPKIT-0093-FR-011 evidence.
  The focused binding-export audit passes.
- Async response tests now exercise typed response views, result access, and
  Debug/Display rendering for Cancel, Poll, Process, and Query Async Requests;
  pending async responses also exercise their typed borrowed views.
  `cargo test -p kmipkit-client --lib --all-features async_batch_item_` passes
  all 5 matching tests.
- The Java semantic batch-item suite now checks rejection and messages for
  missing inputs before JNI loading. The focused Gradle test
  `bash bindings/java/gradlew --project-dir bindings/java --no-daemon --console=plain --quiet test --tests org.kmipkit.extensions.Kmip30SemanticBatchItemTest`
  passes.
- The exact-head CI coverage gate has not yet run on these updates; its result
  remains pending and will be recorded before the PR leaves WIP.
