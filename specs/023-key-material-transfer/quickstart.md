# Validation Quickstart: KMIPKIT-0037

This is the implementation validation guide for Derive Key, Export, Import, and Join Split Key. It is not a formal OASIS conformance, profile, certification, or interoperability claim.

## Prerequisites

- Start from the active `release/1.0.0` line in the dedicated implementation worktree.
- Confirm shared TTLV, object/attribute, operation-data, Cryptographic Parameters, batch/result, client, transport, and secret-byte contracts are available.
- Verify pinned OASIS source hashes and do not alter upstream files.
- Rust protocol/client operation work can proceed without KMIPKIT-0043. Complete C/Java/Python mapping only after the canonical v2 manifest and operation tasks are available.
- Join Split Key callers must provide one known threshold context for every identifier occurrence; no implicit object lookup is allowed.

## Focused validation

Run each operation's protocol unit tests and fake-transport client tests. Cover tables in [spec.md](spec.md), operation IDs, one exchange, common result and delivery state, negative/malformed input, unknown-value retention, and secret redaction. Exercise the Derive Key §7.12/Table 364 and Join Split Key threshold rules directly.

The pinned inventory links Derive Key to KMIPKIT-TEST-CN01-2-14 through -19; Export and Import to -43 through -47; Join Split Key has no direct catalog case ID. Use only local fixtures if present, and label table-derived vectors separately.

## Repository gates

From the repository root, run focused cargo targets and then:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo llvm-cov --workspace --all-features
python tools/normative_catalog/validate.py
python tools/normative_catalog/report.py --check
```

Run pinned generator drift, security/dependency, and supported-platform CI checks per repository automation. Do not report success without output from the command or CI run.

## Release evidence

Record separate Red, Green, and Refactor commit IDs per story. The implementation PR includes focused/full check commands and outputs, coverage/traceability/catalog evidence, parity mapping state, fixture limitations, and risks. Update this file with actual evidence only during implementation.

## Implementation setup evidence

- Managed worktree: `C:\Users\ramp1953\.codex\worktrees\kmipkit-0037-implementation\KMIPKit`.
- Branch: `feature/KMIPKIT-0037-key-material-transfer-implementation` (the prescribed `feature/KMIPKIT-0037-key-material-transfer` name was already checked out by another task; that checkout was not modified).
- Base: active `origin/release/1.0.0` at `af172974cd5c6f56f7f938574e1546d4e295a9ef`; worktree was clean at branch creation.
- Rebased on `origin/release/1.0.0` at `a99e419fdb7d61ed52bb4637796e5e18bbc3009c` on 2026-10-11; rebase completed cleanly and retained all 23 feature commits.
- `Get-FileHash -Algorithm SHA256` comparison against `specification/oasis/kmip-2.1/CHECKSUMS.sha256`: all pinned source checksums verified; upstream tree clean.
- Baseline `cargo test --workspace --all-features`: passed on the unchanged release base (exit 0).
- Shared Rust API review confirmed `AttributeSet`, `ObjectType`, `UniqueIdentifier`, `OperationData`, `Cryptographic Parameters`, `KmipOperationResult`, generic ordered TTLV, typed batch/result/delivery, fake transport tests, and secret byte/item ownership are available. Cross-language bindings remain gated on KMIPKIT-0043's canonical v2 manifest and mappings.

## Derive Key cycle evidence

- Red commits: protocol `3ca0d0b2`; fake-client `6d583aa0`.
- Green commits: protocol model `7616b79d`; client dispatch `bcb0ad9f`.
- Refactor commit: `3ede3903`.
- Passed `cargo test -p kmipkit-protocol --lib derive_key_operation_tests` (6 tests, including the Secret Data total-length and AES-CBC parameter-preservation assertions) and `cargo test -p kmipkit-client --lib derive_key_execution_tests` (1 test).
- Passed `cargo fmt --all --check`, `cargo clippy -p kmipkit-protocol -p kmipkit-client --all-targets --all-features -- -D warnings`, and `git diff --check`.
- Fixture vectors derive from KMIP 2.1 tables and do not establish official conformance or independent implementation interoperability.

## Export cycle evidence

- Red commit: `74c1ed86`.
- Green commit: `84bdd676`.
- Refactor commit: `3c0c5c07`.
- Passed `cargo test -p kmipkit-protocol --lib export_operation_tests` (3 tests) and `cargo test -p kmipkit-client --lib export_execution_tests` (1 test).
- Passed `cargo fmt --all --check`, `cargo clippy -p kmipkit-protocol -p kmipkit-client --all-targets --all-features -- -D warnings`, and `git diff --check`.
- Table-derived vectors exercise caller-selected options, full successful response retention/redaction, unknown result values, single dispatch, and server-error passthrough; they do not establish official conformance or independent implementation interoperability.

## Import cycle evidence

- Red commits: `3c001747` (Import request/response/client contracts), `64b5ec50` (Table 470 Import/Recover operation-code regression), and `337fb396` (Key Wrap Type conditionality regression).
- Green commit: `c824bfb9`.
- Refactor commit: `ab79ca50`.
- Passed `cargo test -p kmipkit-protocol --lib import_operation_tests` (5 tests), `cargo test -p kmipkit-client --lib import_execution_tests` (1 test), the Recover wire-code test (1 test), and `cargo test -p kmipkit-client --lib recover_execution_tests` (9 tests).
- Passed the complete protocol library suite (554 tests) and client library suite (424 tests), `cargo fmt --all --check`, `cargo clippy -p kmipkit-protocol -p kmipkit-client --all-targets --all-features -- -D warnings`, and `git diff --check`.
- Corrected the pre-existing Recover operation code collision from `0x2A` to Table 470's `0x16`; Import uses `0x2A`. Updated existing Recover fixtures and traceability. Import enforces Table 238's Key Wrap Type iff condition against Key Wrapping Data in key objects without unwrapping or interpreting key material.
- These local vectors do not establish official conformance or independent implementation interoperability.


## Join Split Key cycle evidence

- Red commit: `9148bbd7`.
- Green commit: `2ff342bb`.
- Refactor commit: `6b0ad9a3`.
- Passed `cargo test -p kmipkit-protocol --lib join_split_key_operation_tests` (6 tests) and `cargo test -p kmipkit-client --lib join_split_key_execution_tests` (3 tests). The protocol suite asserts caller threshold context, maximum threshold, repeated ID ordering, no wire threshold, error passthrough, and redaction; the client suite asserts one dispatch and NotSent on missing/insufficient context.
- Package-level Clippy and `cargo fmt --all --check` passed after the operation refactor.

## Documentation, traceability, and catalog evidence

- T019: `python scripts/test_user_guide_examples.py --guide docs/user-guide/en/key-derivation-and-transfer.md --guide docs/user-guide/es/derivacion-y-transferencia-de-claves.md` compiled 2 examples; default `python scripts/test_user_guide_examples.py` compiled all 16 marked examples. Both exited 0 on 2026-10-11.
- T020: DCO Red commit `1ec2ec63` added the catalog/traceability regression test; it failed on the missing feature mapping as intended. The catalog and traceability Green edits now pass `python -m unittest tools.normative_catalog.tests.test_key_derivation_transfer_traceability` (3 tests) and `python tools/normative_catalog/validate.py` (`sources=4 clauses=1411 records=4044`). Every stable ledger ID now includes exact implementation and executable test references; mandatory and conditional payload rows link to code, test, and catalog element where one exists.
- T021: `python tools/normative_catalog/report.py --write` regenerated `specification/catalog/coverage-report.md`; the subsequent `--check` passed. API binding artifacts remain gated by T018; no generated binding output was edited by hand.
- T004/T005 final harness evidence: the shared protocol and fake-transport test modules are now registered from their crate test harnesses. `cargo test -p kmipkit-protocol --lib key_transfer_operation_contract_tests` passed 4 tests; `cargo test -p kmipkit-client --lib key_transfer_operations_execution_tests` passed 4 tests. Registration exposed stale synthetic fixture construction (reserved extension tags, old request builders, incomplete PBKDF2 inputs, and a Pending response without its required Response Payload); fixtures were corrected without production behavior changes. Added successful typed response and accessor assertions exercise both completed and Pending outcomes. The separate operation-specific Red commits remain the Red evidence for their respective implementations.
- Updated focused protocol suites on 2026-10-11 passed: Derive Key 9, Export 6, Import 9, and Join Split Key 9. `cargo fmt --all --check` passed.
- Clean workspace coverage on the rebased head: `cargo llvm-cov clean --workspace` followed by `cargo llvm-cov --workspace --all-features --locked` exited 0 on 2026-10-11. Coverage is 23,283/26,101 lines = 89.20%, below the 90% repository gate. Feature modules are Derive Key 95.24%, Export 96.76%, Import 95.26%, and Join Split Key 95.26%. In `crates/kmipkit-client/src/execute.rs`, 233 of 234 executable lines added by this branch are covered (99.57%); the single uncovered defensive `UnsupportedValue` fallback cannot be reached through its closed Derive Key/Export dispatcher. The full file is 96.71%. The largest workspace gap is the unchanged C FFI extension registry at 22.7%; `c_api_coverage` ran 0 tests with `cfg(coverage)`. The workspace gate remains open pending an in-scope path to cover the C interface.
- Rebased workspace quality checks on 2026-10-11: `cargo fmt --all --check`, `git diff --check`, and `cargo clippy --workspace --all-targets --all-features -- -D warnings` passed. An earlier `cargo test --workspace --all-features` attempt exited 1 after three `kmipkit-test-support` loopback fixtures received WinSock 10013 (`PermissionDenied`); the bounded retry `cargo test -p kmipkit-test-support --test transport_fixtures` passed all 7 tests. After rebasing, the exact full command `cargo test --workspace --all-features --locked` passed (exit 0), including all workspace and doctest targets. Thus the earlier socket restriction was transient and did not reproduce in the rebase run.
- Normative checks on 2026-10-11: `python tools/normative_catalog/validate.py` passed (`sources=4 clauses=1411 records=4044`); `python tools/normative_catalog/report.py --check` passed; and `python -m unittest tools.normative_catalog.tests.test_key_derivation_transfer_traceability` passed 3 tests. The default `python scripts/test_user_guide_examples.py` compiled all 16 marked Rust examples.
- Pinned generated-output checks on 2026-10-11 passed: `python tools/normative_catalog/generate_ttlv_tags.py --repo-root . --check`, `generate_result_values.py --repo-root . --check`, `generate_attribute_types.py --repo-root . --check`, and `generate_attribute_policy.py --repo-root . --check`. Each generator verified its existing committed output; no generated file was edited by hand.
- The sequential Codex Security diff review completed on 2026-10-11 with 0 reportable findings after reviewing all 10 changed source-like files. Its report is `C:\Users\ramp1953\.codex\state\plugins\codex-security\scans\KMIPKit\f29a5501925fa721326ce68008c45a832e4c37ca_20261011T155819Z_ihs3czou\report.md`; security scan token usage was unavailable from the workbench. Dependency, parser, and FFI checks and supported-platform CI remain pending. Independent QA is in progress; do not report T022/T023 complete until remaining checks and review outcomes are recorded.
- The new Derive Key positive test confirms caller-supplied AES-CBC Block Cipher Mode is preserved. KMIPKit does not infer remote source-key properties. The catalog and guides record this scope and caller responsibility for a source object's Derive Key usage bit.
- Import uses operation value `0x2A` and Recover uses `0x16` per pinned OASIS §11.36/Table 470. The Import/Recover wire-code regression is linked in the catalog and the Recover client test.

## Cross-language dependency gate

- A read-only check of the KMIPKIT-0043 worktree on 2026-10-11 found the v2 manifest includes all four operation IDs. Its `tasks.md` operation mapping tasks T046, T047, T050, and T052 remain unchecked, and the operation-dependencies contract still gates binding declarations on canonical Rust mappings and four-language parity evidence. No files on that task's branch were modified. T018 remains open until those owner mappings are available and can be reconciled through the approved generator.
- Official case IDs in the normative inventory have not been executed by this feature; source-derived tests do not claim official fixture, profile, certification, or interoperability passes.
