# Validation Quickstart: KMIPKIT-0021

Implementation of Hash, MAC, MAC Verify, Sign, and Signature Verify is present on `feature/KMIPKIT-0021-hash-mac-signature-implementation`. This guide records executed local evidence and the remaining review gates. It does not claim formal OASIS conformance, interoperability, or profile support.

## Executed Rust evidence

The Rust checks below were run in WSL Ubuntu 26.04 with Rust 1.94 during implementation. WSL is currently unavailable (`Wsl/Service/CreateInstance/E_FAIL`), so the Rust coverage run has not been refreshed after the latest test-only commit `0461a924`.

| Command or check | Observed result |
| --- | --- |
| `cargo fmt --all --check` | Passed. |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Passed after the T039 refactor. |
| `cargo test -p kmipkit-protocol cryptographic_operation_contract_tests` | 11/11 focused contract tests passed; formatting check passed in that run. |
| Full protocol test suite | 411 tests passed in the last coverage run before `0461a924`. |
| Full client test suite | 343 tests passed after convenience-dispatch coverage was added. |
| `cargo llvm-cov --no-clean --package kmipkit-protocol --all-features --summary-only` | Last measured protocol line coverage: 8,440/8,969 (94.10%), below the 95% gate. Changed/new executable production lines: 1,049/1,088 (96.42%), above the 95% gate. |
| `cargo llvm-cov --workspace --all-features --summary-only` | Historical workspace coverage: 91.02% in a run before the later coverage-test additions, above the 90% gate at that point. |

The protocol (94.10%) and changed/new production-line (96.42%) values are from the last available feature-branch coverage snapshot before test-only commit `0461a924`. The workspace 91.02% value is older and predates the later coverage-test additions. None of these metrics was refreshed after the latest test batch because WSL failed with EIO and then `Wsl/Service/CreateInstance/E_FAIL`; no coverage output was persisted in the worktree. T040 remains open until a fresh run verifies at least 95% protocol and changed/new production lines and the workspace threshold.

## Windows catalog and dependency-policy evidence

These checks were run sequentially on Windows to avoid concurrent safe-I/O directory locks:

```powershell
python tools/normative_catalog/validate.py
python tools/normative_catalog/report.py --check
python tools/normative_catalog/audit_sources.py --repo-root . --base-sha db51b7a1c93edbe81f89a9faf7902aa543caec3f --check
python tools/normative_catalog/check_immutable_sources.py --repo-root . --base-sha db51b7a1c93edbe81f89a9faf7902aa543caec3f
```

Observed results: catalog validation passed (`sources=4 clauses=1411 records=4029`); the generated coverage report was current; the source audit passed for 1,411 normative candidates; and the immutable-source check confirmed the pinned OASIS upstream tree matches the base commit. The deterministic TTLV-tag, result-value, attribute-type, attribute-policy, API-manifest, and extension-fixture generators also passed their `--check` commands with no drift.

The following Python policy and traceability tests passed:

```powershell
python -m unittest -v tools.normative_catalog.tests.test_feature_traceability tools.normative_catalog.tests.test_audit_sources tools.normative_catalog.tests.test_immutable_sources scripts.tests.test_requirement_traceability scripts.tests.test_dependency_policy scripts.tests.test_cargo_deny_fixtures
$env:CARGO_DENY = 'C:\Users\ramp1953\.cargo\bin\cargo-deny.exe'
python -m unittest -v scripts.tests.test_cargo_deny_fixtures
```

The first command ran 138 tests successfully with 24 skips: Windows symlink privilege limitations, the unset `CARGO_DENY` executable in that invocation, and a test whose contract requires the full policy runner to refresh the advisory database. The second command used cargo-deny 0.20.2 and passed all 18 offline negative-fixture tests. These Python tests do not replace the full dependency-policy run, parser/FFI checks, Linux/macOS CI, or the human security review; T041 and T043 remain open.

## Focused Rust commands

Run the operation model tests and fake-client checks with the pinned Rust toolchain:

```powershell
cargo test -p kmipkit-protocol hash_operation_tests
cargo test -p kmipkit-protocol mac_operation_tests
cargo test -p kmipkit-protocol mac_verify_operation_tests
cargo test -p kmipkit-protocol sign_operation_tests
cargo test -p kmipkit-protocol signature_verify_operation_tests
cargo test -p kmipkit-client hash_execution_tests
cargo test -p kmipkit-client mac_execution_tests
cargo test -p kmipkit-client mac_verify_execution_tests
cargo test -p kmipkit-client sign_execution_tests
cargo test -p kmipkit-client signature_verify_execution_tests
```

The tests exercise table fields, required and optional payload shapes, unknown-value round trips, result handling, redaction, and the single-exchange fake transport contract. No test runs a local cryptographic algorithm.

## Required gates before review

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo llvm-cov --workspace --all-features
python tools/normative_catalog/validate.py
python tools/normative_catalog/report.py --check
git diff --check
```

T040 is not complete while protocol coverage is below 95% or the fresh coverage report is unavailable. T041 is not complete without the full dependency/license/supply-chain runner and applicable parser/FFI security checks. T043 additionally requires independent sequential QA and security review and successful Linux, Windows, and macOS CI before a draft PR.

## Response discrepancy and official evidence

Focused MAC Verify and Signature Verify tests cover single-part, non-final multipart, and both tolerated final multipart Validity Indicator forms. `KMIPKIT-DISC-048` remains open; accepting both final forms is not a server-conformance claim.

The linked Hash, MAC, Sign, and Signature Verify OASIS fixtures are unavailable in the pinned repository, and MAC Verify has no source-linked official Test Case. Tests are source-derived from pinned normative tables. No official fixture pass or profile claim is made.
