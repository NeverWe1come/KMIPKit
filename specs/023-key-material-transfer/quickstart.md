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
- Corrected the pre-existing Recover operation code collision from `0x2A` to Table 470's `0x16`; Import uses `0x2A`. Updated existing Recover fixtures and traceability. Import enforces Table 238's Key Wrap Type iff condition against Key Wrapping Data in key objects, including PGP Key, without unwrapping or interpreting key material.
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
- T021 final verification: `python -B tools/normative_catalog/report.py --write` regenerated `specification/catalog/coverage-report.md`; the subsequent `--check` passed, and `git diff -- specification/catalog/coverage-report.md` was empty. `python -B tools/normative_catalog/validate.py --repo-root .` passed (`sources=4 clauses=1411 records=4044`). The pinned checks `python -B tools/normative_catalog/generate_ttlv_tags.py --repo-root . --check`, `python -B tools/normative_catalog/generate_result_values.py --repo-root . --check`, `python -B tools/normative_catalog/generate_attribute_types.py --repo-root . --check`, and `python -B tools/normative_catalog/generate_attribute_policy.py --repo-root . --check` all passed when invoked sequentially; no generated output was hand-edited. Concurrent validator processes on this Windows host can contend for the repository directory handles and return a sharing violation, so the final checks were run serially. Four-language API declaration generation remains gated by T018 and was not changed.
- T004/T005 final harness evidence: the shared protocol and fake-transport test modules are now registered from their crate test harnesses. `cargo test -p kmipkit-protocol --lib key_transfer_operation_contract_tests` passed 4 tests; `cargo test -p kmipkit-client --lib key_transfer_operations_execution_tests` passed 4 tests. Registration exposed stale synthetic fixture construction (reserved extension tags, old request builders, incomplete PBKDF2 inputs, and a Pending response without its required Response Payload); fixtures were corrected without production behavior changes. Added successful typed response and accessor assertions exercise both completed and Pending outcomes. The separate operation-specific Red commits remain the Red evidence for their respective implementations.
- Updated focused protocol suites on 2026-10-11 passed: Derive Key 9, Export 6, Import 9, and Join Split Key 9. `cargo fmt --all --check` passed.
- Independent QA follow-up corrections (2026-10-12): Red `a4e184a6` → Green `7d7cbf8b` → Refactor `7688c89a` corrected Import `Replace Existing` from `0x420072` (Q String) to `0x420124` (§11.56/Table 487). Red `c6fb272d` → Green `ef48cc5f` corrected the Import `Encryption Key Information` fixture tag to `0x420036` (§3.3/Table 15; `0x42003D` is IV/Counter/Nonce) and the Derive Key Secret Data Object Type to 7 (§11.34/Table 468). Red `3bbbb5fb` → Green `651d92bb` → Refactor `370f3527` corrected the wrapped-key fixture to use a Key Value Byte String, as required by §3.1/Table 12, with synthetic bytes and no local unwrap. Red `6f62f4ed` → Green `df06142f` → Refactor `2e4b6870` added PGP Key (Object Type 9) to the wrapped-key condition from §2.4/Table 5 and §6.1.25/Table 238. The focused Import and Derive Key protocol suites each pass 10 tests; protocol Clippy, formatting, and `git diff --check` pass. Independent QA and exact-range security evidence are recorded above. PR #99 final CI evidence is recorded below.
- Final independent QA correction (2026-10-12): Red `0ea4a72a` added an assertion that a wrapped PGP Key's Key Block contains Key Format Type (`0x420042`) as an Enumeration; it failed because the fixture omitted the required field. Green `b2d6c8bd` adds Raw (`1`) to the fixture, and Refactor `cb5ab73d` cites OASIS KMIP 2.1 §3.1/Table 12 at the assertion. The pinned source marks Key Format Type REQUIRED. Independent QA reviewed the final five-file diff and found no blocker; Import protocol (10), Derive Key protocol (10), Import client (1), formatting, and diff checks passed. The completed exact-range security scan found zero reportable candidates; report ID `6bc360b3-9696-4f16-990d-95a6a1d19aa3`.
- Clean workspace coverage on the rebased development head: `cargo llvm-cov clean --workspace` followed by `cargo llvm-cov --workspace --all-features --locked` exited 0 on 2026-10-11. That intermediate tree measured 23,283/26,101 lines = 89.20%, below the 90% gate. The later release-line aggregate on commit `7a430d5f`, checked by PR #98 coverage run [38211113445](https://github.com/NeverWe1come/KMIPKit/actions/runs/38211113445/job/114687344804), measured 25,811/26,709 lines = 96.64%, satisfying the workspace gate. The same aggregate reports TTLV 99.19%, protocol 96.97%, and FFI 95.23%. Feature modules on the development head were Derive Key 95.24%, Export 96.76%, Import 95.26%, and Join Split Key 95.26%. In `crates/kmipkit-client/src/execute.rs`, 233 of 234 executable lines added by this branch were covered (99.57%); the single uncovered defensive `UnsupportedValue` fallback cannot be reached through its closed Derive Key/Export dispatcher. The earlier 22.7% unchanged C FFI extension-registry figure and zero `c_api_coverage` tests therefore describe only that intermediate local run, not the release-line aggregate.
- Rebased workspace quality checks on 2026-10-11: `cargo fmt --all --check`, `git diff --check`, and `cargo clippy --workspace --all-targets --all-features -- -D warnings` passed. An earlier `cargo test --workspace --all-features` attempt exited 1 after three `kmipkit-test-support` loopback fixtures received WinSock 10013 (`PermissionDenied`); the bounded retry `cargo test -p kmipkit-test-support --test transport_fixtures` passed all 7 tests. After rebasing, the exact full command `cargo test --workspace --all-features --locked` passed (exit 0), including all workspace and doctest targets. Thus the earlier socket restriction was transient and did not reproduce in the rebase run.
- Final local verification on 2026-10-12: `cargo test --workspace --all-features --locked` and `cargo clippy --workspace --all-targets --all-features -- -D warnings` passed on Windows; `cargo fmt --all --check`, `git diff --check`, `python tools/normative_catalog/validate.py` (`sources=4 clauses=1411 records=4044`), and `python tools/normative_catalog/report.py --check` passed. The fresh local LLVM run completed; its normalized Windows-only production-source projection was 24,255/26,709 lines (90.81%), with TTLV 99.19%, protocol 96.96%, transport 95.16%, and FFI 22.88%. That local projection lacks the Linux-only C consumer report and the other platform artifacts, so it is not the CI aggregate gate; the final cross-platform/FFI coverage result is recorded below.
- Normative checks on 2026-10-11: `python tools/normative_catalog/validate.py` passed (`sources=4 clauses=1411 records=4044`); `python tools/normative_catalog/report.py --check` passed; and `python -m unittest tools.normative_catalog.tests.test_key_derivation_transfer_traceability` passed 3 tests. The default `python scripts/test_user_guide_examples.py` compiled all 16 marked Rust examples.
- Pinned generated-output checks on 2026-10-11 passed: `python tools/normative_catalog/generate_ttlv_tags.py --repo-root . --check`, `generate_result_values.py --repo-root . --check`, `generate_attribute_types.py --repo-root . --check`, and `generate_attribute_policy.py --repo-root . --check`. Each generator verified its existing committed output; no generated file was edited by hand.
- Security/dependency/parser script checks on 2026-10-11: the sequential Codex Security diff review completed with 0 reportable findings after reviewing all 10 changed source-like files. Its report is `C:\Users\ramp1953\.codex\state\plugins\codex-security\scans\KMIPKit\f29a5501925fa721326ce68008c45a832e4c37ca_20261011T155819Z_ihs3czou\report.md`; workbench token usage was unavailable. `pwsh -File .\scripts\Test-DependencyPolicy.ps1` passed with pinned cargo-deny 0.20.2; fresh RustSec root/fuzz database evidence used commit `7eebec69c352c7191b1f13eb95dd510eeca5d1de` at `2026-10-09T10:12:02+02:00`, approved exception IDs validated, and both lockfile hashes remained unchanged. `pwsh -File .\scripts\tests\Test-Wsl.ps1` passed all 8 cases. With `CARGO_DENY=C:\Users\ramp1953\.cargo\bin\cargo-deny.exe`, `python -m unittest discover -s scripts/tests -p 'test_*.py' -v` ran 255 tests successfully; 6 platform-dependent symlink cases were skipped because Windows denied symlink creation. Supported-platform CI and Linux FFI sanitizer/C consumer/fuzz-smoke checks passed on PR #93 run [38156503260](https://github.com/NeverWe1come/KMIPKit/actions/runs/38156503260); only scheduled release/nightly jobs were skipped. The later release-line aggregate [38211113445](https://github.com/NeverWe1come/KMIPKit/actions/runs/38211113445/job/114687344804) measured 25,811/26,709 = 96.64%. This evidence predates the follow-up corrections and is supplemental; the final PR #99 checks are recorded below.
- The new Derive Key positive test confirms caller-supplied AES-CBC Block Cipher Mode is preserved. KMIPKit does not infer remote source-key properties. The catalog and guides record this scope and caller responsibility for a source object's Derive Key usage bit.
- Import uses operation value `0x2A` and Recover uses `0x16` per pinned OASIS §11.36/Table 470. The Import/Recover wire-code regression is linked in the catalog and the Recover client test.

## Cross-language dependency gate

- A read-only check of the KMIPKIT-0043 worktree on 2026-10-11 found the v2 manifest includes all four operation IDs. Its `tasks.md` operation mapping tasks T046, T047, T050, and T052 remain unchecked, and the operation-dependencies contract still gates binding declarations on canonical Rust mappings and four-language parity evidence. No files on that task's branch were modified. T018 remains open until those owner mappings are available and can be reconciled through the approved generator.
- Official case IDs in the normative inventory have not been executed by this feature; source-derived tests do not claim official fixture, profile, certification, or interoperability passes.


## Final PR #99 verification (2026-10-12)

- PR: [#99](https://github.com/NeverWe1come/KMIPKit/pull/99), branch `feature/KMIPKIT-0037-ci-evidence`, final verified head `4f7c13ae4061cde96078d916d5830bad9fc990f3`; base `release/1.0.0` at `7a430d5f382c2726e2167bc9d703f8957a8b7acc`.
- GitHub Actions run [38247165872](https://github.com/NeverWe1come/KMIPKit/actions/runs/38247165872): `gh pr checks 99 --json name,state` reported **33 SUCCESS, 2 SKIPPED, 0 failed, 0 pending**. The skipped jobs were scheduled release dependency policy and informational nightly coverage.
- Core checks passed on Linux, macOS, and Windows for Rust 1.94/stable; C consumers passed on Linux/macOS/Windows; Java/JNI integration passed on Linux/macOS/Windows; Python bindings passed on Linux/macOS/Windows; C/JNI sanitizers, dependency policy, documentation/traceability, normative inventory, fuzz smoke, and script contracts passed. Coverage passed on Linux/macOS/Windows and the selected-scope aggregate gate passed.
- Independent QA reviewed the implementation diff at `33bbaad7`, then reviewed the current-head verification update at `4f7c13ae`; no blocker remained. Focused rechecks passed: Import protocol 10/10, Derive Key protocol 10/10, Import client 1/1, `cargo fmt --all --check`, and `git diff --check`.
- The exact-range Codex Security review of the implementation diff completed with zero reportable candidates (`6bc360b3-9696-4f16-990d-95a6a1d19aa3`). The later changes on this branch are documentation-only.
- These checks close T022 and T023. T018's four-language API mappings and the generated API artifacts in T021 remain gated on the approved KMIPKIT-0043 mapping; this PR does not claim those API tasks complete.
