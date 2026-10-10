# Validation Quickstart: KMIPKIT-0021

Implementation of Hash, MAC, MAC Verify, Sign, and Signature Verify is present on `feature/KMIPKIT-0021-hash-mac-signature-implementation`. This guide records executed local evidence and the remaining review gates. It does not claim formal OASIS conformance, interoperability, or profile support.

## Executed Rust evidence

The Rust checks below were run in WSL Ubuntu 26.04 with Rust 1.94. Focused and full-workspace tests, formatting, and Clippy passed after the framing-context GREEN/REFACTOR and multipart fixture correction. The latest full instrumented test run had no test failures, but LLVM could not export a full-workspace coverage report because a generated `trybuild` scratch file was treated as an object; the coverage threshold gate remains open.

| Command or check | Observed result |
| --- | --- |
| `cargo fmt --all --check` | Passed. |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Passed after the T039 refactor. |
| `cargo test -p kmipkit-protocol --all-features` | Full protocol suite passed, including all corrected multipart response fixtures. |
| `cargo test -p kmipkit-client --all-features` | Full client suite passed, including Hash/MAC/Sign framing and reordered batch association. |
| `cargo test --workspace --all-features` | Full workspace tests and doctests passed; the C ABI consumer integration test passed. |
| `cargo llvm-cov --workspace --all-features --json` | Fresh instrumented workspace tests completed without test failures, but LLVM report generation failed because a generated `trybuild` `invoked.timestamp` file was treated as an object. No current full-workspace coverage report was produced; T040 remains open. |
| `pwsh -NoProfile -File .\scripts\tests\Test-Wsl.ps1` | All 8 PowerShell WSL runner contract tests passed. |
| `cargo test -p kmipkit-protocol cryptographic_operation_contract_tests` | 12/12 focused contract tests passed after the framing-context fix. |
| `cargo test -p kmipkit-client hash_mac_signature_execution_tests` | 12/12 client framing/dispatch tests passed after Refactor. |
| `cargo test -p kmipkit-client mac_verify_execution_tests`; `cargo test -p kmipkit-client signature_verify_execution_tests` | 4/4 tests passed in each suite after Refactor. |
| `cargo fmt --all -- --check`; `cargo clippy -p kmipkit-protocol -p kmipkit-client --all-targets --all-features -- -D warnings` | Both passed after Refactor. |
| Response output-cardinality RED, commit `a79d9a52` | `cargo test -p kmipkit-protocol single_part_hash_mac_and_sign_successes_require_their_output_fields` failed as expected because all three single-part parsers accepted a missing output field. `cargo test -p kmipkit-client hash_mac_signature_execution_tests` ran 11 tests: 8 passed and 3 failed as expected for missing single-part output, forbidden initial-multipart output, and reordered batch association. |
| Final multipart RED, commit `f4cbd048` | `cargo test -p kmipkit-client final_multipart_hash_mac_and_sign_responses_omit_output_data` failed as expected because Hash, MAC, and Sign accepted output data on final multipart responses. |
| Response output-cardinality GREEN, commit `cf899bbb` | `cargo test -p kmipkit-protocol cryptographic_operation_contract_tests` passed 12/12; `cargo test -p kmipkit-client hash_mac_signature_execution_tests` passed 12/12, covering successful single-part output presence, output absence across multipart parts, and out-of-order response association by batch item ID. |
| Verify regression suites after GREEN | Protocol `mac_verify_operation_tests` passed 10/10 and `signature_verify_operation_tests` passed 11/11; client `mac_verify_execution_tests` and `signature_verify_execution_tests` passed 4/4 each. The shared framing enum preserves the existing Verify behavior and DISC-048 handling. |
| Context extraction Refactor, commit `9ac2c0f2` | Client Hash/MAC/Sign tests passed 12/12, MAC Verify 4/4, and Signature Verify 4/4; the protocol/client Clippy and formatting checks above passed after the refactor. |
| Multipart protocol fixture context correction, test-only commit `99d50a81` | Static review found three protocol tests calling the response-only converter, whose documented convenience interpretation is `SinglePart`, even though each fixture represents a multipart response with no operation output. They now call the explicit converter with `MultipartNonFinal`. The existing fake-client test covers final multipart output cardinality, so no duplicate final case was added. The initial test rerun was blocked by `Wsl/Service/CreateInstance/E_FAIL`; the subsequent full protocol/client/workspace runs above passed with the corrected fixtures. |
| Earlier protocol/client test totals | 411 protocol and 343 client tests passed before the final fixture correction; superseded by the fresh full-suite results above. |
| Earlier protocol coverage snapshot | 8,440/8,969 (94.10%) and changed/new production 1,049/1,088 (96.42%); historical only and below the protocol threshold. |
| Earlier fresh coverage retry | An instrumented link failed because a cached `libtrybuild` archive was truncated at 16 MiB. A new target directory rebuilt dependencies and an earlier complete run passed; this historical run does not close the current coverage gate. |

The 98.27% TTLV, 95.45% protocol, 95.11% client, 92.16% workspace, and 97.41% changed-production measurements came from an earlier run and are retained as historical evidence. The latest full instrumented test run passed its tests but failed while exporting coverage because of a generated `trybuild` scratch file. T040 remains open until a valid full-workspace report verifies every local threshold; PR CI is expected to provide another reproducible report.

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

The first command ran 138 tests successfully with 24 skips: Windows symlink privilege limitations, the unset `CARGO_DENY` executable in that invocation, and a test whose contract requires the full policy runner to refresh the advisory database. The second command used cargo-deny 0.20.2 and passed all 18 offline negative-fixture tests. The redaction/CI diagnostic command passed all 36 tests. A later full Python script run passed 254 tests with 26 environment-dependent skips; the full dependency-policy runner below separately covered the cargo-deny fixtures and refreshed RustSec evidence.

The most recent dedicated feature-traceability attempt ran 15 tests successfully, skipped one, and errored once when writing a temporary fixture failed with `OSError: [Errno 28] No space left on device`. An earlier rerun after storage was restored completed with 17 tests and one Windows symlink-permission skip. The latest error is environmental and remains recorded as execution history. The redaction/CI diagnostic suite passed 36/36. After those earlier runs, `pwsh -NoProfile -File .\scripts\Test-DependencyPolicy.ps1` passed with pinned cargo-deny 0.20.2, verified both root and fuzz workspaces against RustSec commit `7eebec69c352c7191b1f13eb95dd510eeca5d1de` (2026-10-09), ran the policy fixtures, and confirmed both lockfiles remained unchanged. The general Python script suite passed 254 tests with 26 environment-dependent skips; the dependency-policy runner separately ran the cargo-deny fixtures with the pinned binary. The implementation diff changes no FFI or binding files and does not alter the pinned OASIS upstream tree.

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

T040 remains open because no current full-workspace coverage report is available. Independent security review found no branch-introduced finding, but T041 remains open because the formal security scan could not start after its selected working-tree snapshot became stale. Native sanitizer and fuzz checks, when run in the full workflow, and multi-platform PR CI remain under T043. T043 requires PR CI after a terminal-created draft PR.

## Response discrepancy and official evidence

Focused MAC Verify and Signature Verify tests cover single-part, non-final multipart, and both tolerated final multipart Validity Indicator forms. `KMIPKIT-DISC-048` remains open; accepting both final forms is not a server-conformance claim.

The linked Hash, MAC, Sign, and Signature Verify OASIS fixtures are unavailable in the pinned repository, and MAC Verify has no source-linked official Test Case. Tests are source-derived from pinned normative tables. No official fixture pass or profile claim is made.
