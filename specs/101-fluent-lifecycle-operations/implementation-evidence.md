# KMIPKIT-0101 Implementation Evidence

## Branch and dependency

- Branch: `feature/KMIPKIT-0101-fluent-lifecycle-operations`.
- Base: PR #183 head `70d58564821009f91deef28bb13562b47ec5a9c9`, whose exact-head
  CI passed all 47 checks (2 conditional skips); PR #183 is ready for review.
- Scope: Rust fluent builders for Locate, Activate, Archive, Recover, Revoke,
  Re-key, Re-key Key Pair, and Re-certify.

## Red

- Commit `85661c3e7` (`test: specify fluent lifecycle builders`) adds focused
  typed request-variant tests for all eight operations.
- Before Green, `cargo test -p kmipkit-client --lib
  fluent_lifecycle_operation_tests --no-run` failed because `RequestBuilder`
  had no `activate`, `archive`, `recover`, `revoke`, `locate`, `rekey`,
  `rekey_key_pair`, or `recertify` methods.

## Green

- Commit `7f94a92e4` (`feat: add fluent lifecycle request builders`) adds the
  operation-specific builders using the existing request models, validators,
  typed batch variants, and one-exchange client route.
- Tests cover optional identifiers, semantic Revoke fields, explicit Locate
  filters, Re-key attribute groups, local rejection, protocol version checks,
  Success/Pending/Failure outcomes, correlation, one exchange, and redaction.

## Refactor

- Commit `46a5e04c1` (`refactor: make lifecycle builders explicit`) records the
  refactor. The client source-boundary audit rejected macro-generated public builder
  declarations as opaque. The refactor makes each public lifecycle builder
  and its methods explicit, and adds exact signature checks for the approved
  `AttributeSet` and `Structure` expert inputs.
- Manifest fixture setup now copies candidate feature specs and the enabled
  KMIP 3.0 draft catalog's pinned source, inventory, and checksum inputs.
- The creation parity regression accepts both `declared` and `verified`
  public-declaration states while still rejecting blocked records.

## Quality-gate repair

- SonarCloud reported 4.92% duplicated lines on new code (102 lines; limit
  3%) for head `5d9c223690a527ce9390debda5d3672f9e989d77`. The duplicated span
  was the common Re-key and Re-certify builder forwarding surface.
- The fix keeps both operation-specific public structs and their explicit
  signatures, while using their typed request setters directly in the Re-key
  wrapper. A generic public type alias was rejected by the client AST boundary
  and was discarded before commit.
- Commit `b4ba16415` adds regression coverage for both named builders and their
  concrete typed request outputs. The operation behavior is unchanged.

## Verification

- `cargo fmt --all -- --check`: passed.
- `cargo test -p kmipkit-client --all-features --locked`: 852 unit tests,
  compile-fail/UI checks, and 10 doctests passed.
- `cargo clippy -p kmipkit-client --all-targets --all-features -- -D warnings`:
  passed.
- `cargo llvm-cov --workspace --all-features`: passed, 90.29% line coverage,
  90.81% region coverage, and 92.05% function coverage workspace-wide.
- `cargo test -p kmipkit-client --all-features --locked --lib
  execute_boundary_tests::fluent_operation_ttlv_inputs_are_exact_and_method_scoped
  -- --exact`: passed.
- `python3 -B scripts/test_user_guide_examples.py`: 32 Rust examples compiled.
- Requirement traceability: 2 tests passed. Feature traceability: 31 tests
  passed.
- `python3 -B -m unittest tools.api_manifest.tests.test_manifest_v2`: 156 tests
  passed. The 28 reconciliation and shared-inventory unit tests also passed.
- `python3 -B tools/api_manifest/generate.py --check`: 12 outputs verified.
- Rust export audit: 29,862 observed paths, 9,145 exact matches, 20,717
  approved exclusions, and no missing, stale, kind-mismatched, or unresolved
  paths.
- Shared Rust inventory and binding audits passed. C, Java, and Python each
  matched all manifest declarations: 1,967, 2,265, and 2,445 exact matches.
- `python3 -B tools/api_manifest/audit_binding_exports.py --check`: C 1,967,
  Java 2,265, and Python 2,445 declarations matched; all three reports were
  complete with no issues.
- PR #183 exact-head CI passed; its branch remains unmerged. Draft PR #184
  remains `[WIP]` while its prior-head SonarCloud duplication failure is
  rechecked on the repaired head.

## Exact-head CI regression repair: Red

- Head `1885a509e8ce6f3f52c935d2b1dcb8bb647ef134`, CI run
  `43039339116`: the Script contracts job failed in
  `test_t102_slice_and_manifest_snapshots_remain_consistent`, with
  `15822 != 14208`. The approved KMIPKIT-0093 inventory reconciliation added
  1,614 declarations and 3,467 exclusions without updating the shared live
  manifest totals; historical Rust export report totals remain separate.
- Local reproduction:
  `python3 -B -m unittest tools.api_manifest.tests.test_t102_pkcs11_request_trait_exclusions.T102Pkcs11RequestTraitExclusionTests.test_t102_slice_and_manifest_snapshots_remain_consistent`
  failed with the same assertion before repair.
- The same run's Coverage gate rejected changed Rust coverage `162/329`,
  below the required 90%. Existing tests predominantly exercised local
  construction and Revoke execution, leaving other lifecycle exchanges and
  several setters unexecuted. This repair adds behavioral tests without
  changing coverage thresholds or excluding production lines.
