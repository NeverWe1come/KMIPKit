# KMIPKIT-0101 Implementation Evidence

## Branch and dependency

- Branch: `feature/KMIPKIT-0101-fluent-lifecycle-operations`.
- Planned base: PR #183 head `8b4724ca5` (the branch is being rebased onto this
  exact head before it is pushed).
- Scope: Rust fluent builders for Locate, Activate, Archive, Recover, Revoke,
  Re-key, Re-key Key Pair, and Re-certify.

## Red

- Commit `5305541f1` (`test: specify fluent lifecycle builders`) adds focused
  typed request-variant tests for all eight operations.
- Before Green, `cargo test -p kmipkit-client --lib
  fluent_lifecycle_operation_tests --no-run` failed because `RequestBuilder`
  had no `activate`, `archive`, `recover`, `revoke`, `locate`, `rekey`,
  `rekey_key_pair`, or `recertify` methods.

## Green

- Commit `9d738f820` (`feat: add fluent lifecycle request builders`) adds the
  operation-specific builders using the existing request models, validators,
  typed batch variants, and one-exchange client route.
- Tests cover optional identifiers, semantic Revoke fields, explicit Locate
  filters, Re-key attribute groups, local rejection, protocol version checks,
  Success/Pending/Failure outcomes, correlation, one exchange, and redaction.

## Refactor

- The client source-boundary audit rejected macro-generated public builder
  declarations as opaque. The refactor makes each public lifecycle builder
  and its methods explicit, and adds exact signature checks for the approved
  `AttributeSet` and `Structure` expert inputs.
- Manifest fixture setup now copies candidate feature specs and the enabled
  KMIP 3.0 draft catalog's pinned source, inventory, and checksum inputs.
- The creation parity regression accepts both `declared` and `verified`
  public-declaration states while still rejecting blocked records.
- Refactor commit and final validation results will be recorded before push.

## Verification

- `cargo test -p kmipkit-client --all-features --locked`: passed.
- `cargo clippy -p kmipkit-client --all-targets --all-features -- -D warnings`:
  passed.
- `cargo test -p kmipkit-client --all-features --locked --lib
  execute_boundary_tests::fluent_operation_ttlv_inputs_are_exact_and_method_scoped
  -- --exact`: passed.
- `python3 -B scripts/test_user_guide_examples.py`: 32 Rust examples compiled.
- Requirement traceability: 2 tests passed. Feature traceability: 31 tests
  passed.
- `python3 -B tools/api_manifest/generate.py --check`: 12 outputs verified.
- `python3 -B tools/api_manifest/audit_binding_exports.py --check`: C 1,967,
  Java 2,265, and Python 2,445 declarations matched; all three reports were
  complete with no issues.
- Full API manifest suite and workspace LLVM coverage are still running.
- PR #183 exact-head CI is running on `8b4724ca510e8aa20141b21a989873c9722566b8`;
  no failed jobs have been reported so far.
