# Validation Quickstart: KMIPKIT-0021

Implementation of Hash, MAC, MAC Verify, Sign, and Signature Verify is present on `feature/KMIPKIT-0021-hash-mac-signature-implementation`. This guide records executed local evidence and the remaining review gates. It does not claim formal OASIS conformance, interoperability, or profile support.

## Executed Rust evidence

The Rust checks below were run in WSL Ubuntu 26.04 with Rust 1.94. Focused tests, formatting, and Clippy ran after the framing-context GREEN/REFACTOR. The latest fresh workspace coverage attempt stopped during instrumented compilation because WSL's ext4 root filesystem remounted read-only; the coverage gates therefore remain unverified after these changes.

| Command or check | Observed result |
| --- | --- |
| `cargo fmt --all --check` | Passed. |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Passed after the T039 refactor. |
| `cargo test -p kmipkit-protocol cryptographic_operation_contract_tests` | 12/12 focused contract tests passed after the framing-context fix. |
| `cargo test -p kmipkit-client hash_mac_signature_execution_tests` | 12/12 client framing/dispatch tests passed after Refactor. |
| `cargo test -p kmipkit-client mac_verify_execution_tests`; `cargo test -p kmipkit-client signature_verify_execution_tests` | 4/4 tests passed in each suite after Refactor. |
| `cargo fmt --all -- --check`; `cargo clippy -p kmipkit-protocol -p kmipkit-client --all-targets --all-features -- -D warnings` | Both passed after Refactor. |
| Response output-cardinality RED, commit `a79d9a52` | `cargo test -p kmipkit-protocol single_part_hash_mac_and_sign_successes_require_their_output_fields` failed as expected because all three single-part parsers accepted a missing output field. `cargo test -p kmipkit-client hash_mac_signature_execution_tests` ran 11 tests: 8 passed and 3 failed as expected for missing single-part output, forbidden initial-multipart output, and reordered batch association. |
| Final multipart RED, commit `f4cbd048` | `cargo test -p kmipkit-client final_multipart_hash_mac_and_sign_responses_omit_output_data` failed as expected because Hash, MAC, and Sign accepted output data on final multipart responses. |
| Response output-cardinality GREEN, commit `cf899bbb` | `cargo test -p kmipkit-protocol cryptographic_operation_contract_tests` passed 12/12; `cargo test -p kmipkit-client hash_mac_signature_execution_tests` passed 12/12, covering successful single-part output presence, output absence across multipart parts, and out-of-order response association by batch item ID. |
| Verify regression suites after GREEN | Protocol `mac_verify_operation_tests` passed 10/10 and `signature_verify_operation_tests` passed 11/11; client `mac_verify_execution_tests` and `signature_verify_execution_tests` passed 4/4 each. The shared framing enum preserves the existing Verify behavior and DISC-048 handling. |
| Context extraction Refactor, commit `9ac2c0f2` | Client Hash/MAC/Sign tests passed 12/12, MAC Verify 4/4, and Signature Verify 4/4; the protocol/client Clippy and formatting checks above passed after the refactor. |
| Full protocol test suite | 411 tests passed in the last coverage run before `0461a924`. |
| Full client test suite | 343 tests passed after convenience-dispatch coverage was added. |
| `cargo llvm-cov --no-clean --package kmipkit-protocol --all-features --summary-only` | Last measured protocol line coverage: 8,440/8,969 (94.10%), below the 95% gate. Changed/new executable production lines: 1,049/1,088 (96.42%), above the 95% gate. |
| `cargo llvm-cov --no-clean --workspace --all-features --json --output-path /home/ramp1953/kmipkit-0021-coverage-target/kmipkit-0021-workspace.json` | Latest fresh attempt failed during instrumented compilation with `Read-only file system (os error 30)` writing under the persistent target; WSL `/` is ext4 mounted read-only. No report was produced. |

The protocol (94.10%) and changed/new production-line (96.42%) values are from the last available feature-branch coverage snapshot before the latest framing tests. The workspace 91.02% value is older and predates later coverage-test additions. A fresh post-change attempt was made, but WSL's `/` filesystem remounted read-only during compilation and no report was produced. T040 remains open until a fresh run verifies at least 95% protocol and changed/new production lines and the workspace threshold.

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
python -m unittest -v scripts.tests.test_diagnostic_redaction scripts.tests.test_ci_diagnostics scripts.tests.test_ci_impact scripts.tests.test_ci_summary
```

The first command ran 138 tests successfully with 24 skips: Windows symlink privilege limitations, the unset `CARGO_DENY` executable in that invocation, and a test whose contract requires the full policy runner to refresh the advisory database. The second command used cargo-deny 0.20.2 and passed all 18 offline negative-fixture tests. The redaction/CI diagnostic command passed all 36 tests. These Python tests do not replace the full dependency-policy run, parser/FFI checks, Linux/macOS CI, or the human security review; T041 and T043 remain open.

The most recent dedicated feature-traceability attempt ran 15 tests successfully, skipped one, and errored once when writing a temporary fixture failed with `OSError: [Errno 28] No space left on device`. An earlier rerun after storage was restored completed with 17 tests and one Windows symlink-permission skip. The latest error is environmental and remains recorded as execution history. The redaction/CI diagnostic suite passed 36/36.

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
