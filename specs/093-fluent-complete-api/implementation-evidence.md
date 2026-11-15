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
