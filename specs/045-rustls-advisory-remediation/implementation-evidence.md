# Implementation Evidence: Rustls Advisory Remediation

**Feature**: KMIPKIT-0045
**Branch**: `feature/KMIPKIT-0045-rustls-advisory-remediation`
**Date**: 2026-10-14

## Advisory baseline

The active `release/1.0.0` workspace pins `rustls = "=0.23.45"` with `default-features = false` and the `aws_lc_rs`/`std` features. The transport crate inherits this workspace dependency. The fuzz workspace has no Rustls dependency.

The RustSec advisory records published on 2026-10-14 list `>=0.23.46` as patched for RUSTSEC-2026-0346, RUSTSEC-2026-0347, RUSTSEC-2026-0348, and RUSTSEC-2026-0349. Stable Rustls 0.23.46 is available and declares an MSRV below KMIPKit's 1.94.

CI run [38596298478](https://github.com/NeverWe1come/KMIPKit/actions/runs/38596298478), on the PR #90 dependency state before this remediation, completed with the `Dependency policy / pull request` check failed. All other completed checks passed; the profile evidence aggregate was skipped, and the CI summary failed because of the dependency-policy result.

The local online baseline command `pwsh -NoLogo -NoProfile -File .\scripts\Test-DependencyPolicy.ps1` also exited with code 1 after validating the reviewed Windows Rust host and cargo-deny 0.20.2. Its redacted diagnostic reported Rustls 0.23.45 findings for all four advisory IDs and no exact registered exception. The policy correctly failed closed; no waiver was added.

## Red evidence

Command:

```text
python -m unittest scripts.tests.test_rustls_advisory_pin -v
```

Result before changing the manifest or lockfile: **failed as expected**. Four tests ran; the exact-version/feature and traceability contracts passed, and `test_workspace_pin_is_exact_stable_patched_release` failed because `(0, 23, 45)` is below `(0, 23, 46)`.

The failure is the intended Red signal. Green results are recorded below. Independent QA and security reviews completed against the implementation through `4eef85dd`; required CI for the draft PR remains pending.

### QA traceability correction (2026-10-14)

- Red: `python -m unittest scripts.tests.test_rustls_advisory_pin.RustlsAdvisoryPinTests.test_fr004_traceability_references_tls_policy_tests -v` failed because FR-004 did not link the existing TLS 1.3/mTLS and peer-validation tests.
- Green: FR-004 was narrowed to the preserved Rustls backend/features and TLS 1.3 mutual-TLS policy; the traceability row now links `caller_ca_and_matching_mtls_identity_build_a_tls13_only_config` and `untrusted_chain_expired_leaf_and_hostname_mismatch_fail_closed`.
- After the correction, `python -m unittest scripts.tests.test_rustls_advisory_pin -v` passed all five tests.

## Green evidence

After changing only the exact workspace pin to `=0.23.46` and updating `Cargo.lock` with Cargo:

- `python -m unittest scripts.tests.test_rustls_advisory_pin -v`: **4 tests passed**.
- `cargo test -p kmipkit-transport --all-features`: **exit code 0** on Windows. The AWS-LC build emitted non-fatal environment warnings because NASM was unavailable, but the test command completed successfully.
- `pwsh -NoLogo -NoProfile -File .\scripts\Test-DependencyPolicy.ps1`: **passed** for root and fuzz workspaces with successful online RustSec refresh. Both report database commit `caee11f41fab07d676c0a4655a2a5dd1f94da97c`, timestamp `2026-10-14T15:17:45+02:00`. Existing dependency-policy exception IDs `KMIPKIT-0011-EX-003` and `KMIPKIT-0011-EX-004` remain reported; no Rustls advisory exception was added.
- Cargo reported only `rustls 0.23.45 -> 0.23.46` in `Cargo.lock`; no other package resolution changed. The manifest's `aws_lc_rs` and `std` feature list and disabled defaults are unchanged.

The workspace checks and coverage completed below, and independent QA/security review passed for the implementation. The final full script-contract suite and fresh online dependency-policy check are recorded below. Cross-platform PR CI remains pending, so this branch is not yet ready to leave draft.

## Local verification after Refactor

| Check | Result |
|---|---|
| `python -m unittest scripts.tests.test_rustls_advisory_pin -v` | 4 passed |
| `python -B -m unittest discover -s scripts/tests -p 'test_*.py' -q` | 272 passed, 26 skipped |
| API manifest test suite in an isolated Python 3.12.14 venv with `tools/api_manifest/requirements-test.txt` | 222 passed, 7 skipped |
| `tools/api_manifest/generate.py --check` | 12 generated outputs verified |
| CI feature-traceability suite `test_feature_traceability.py` | 15 passed, 1 skipped because Windows denied temporary symlink creation |
| `tools/normative_catalog/validate.py --repo-root .` | Valid: 4 sources, 1,411 clauses, 4,044 records |
| `tools/normative_catalog/report.py --check --repo-root .` | Generated coverage report verified |
| `cargo fmt --all --check` | Passed |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | Passed |
| `cargo test --workspace --all-features --locked --quiet` | Exit code 0; all reported tests and doctests passed |
| `cargo llvm-cov --workspace --all-features --locked` | Completed; workspace line coverage 92.92%, above the 90% overall threshold |
| `cargo doc --workspace --all-features --no-deps --locked` | Passed |
| `git diff --check` | Passed |
| `pwsh -NoLogo -NoProfile -File .\scripts\Test-DependencyPolicy.ps1` | Passed for both workspaces with fresh RustSec evidence recorded above |

The full `tools/normative_catalog` test discovery was started but its final result was not retained by the local command session. It is not counted as passed. The CI-specific feature-traceability suite, normative catalog validation, and generated report check above did complete successfully. Local Rust builds emitted non-fatal AWS-LC/NASM environment warnings; the affected test, lint, coverage, and documentation commands still returned success.

## Final verification and review snapshot

The following checks ran at branch head `4e8d926a` on 2026-10-14. Subsequent evidence, task-status, and reviewer-owned checklist changes are documentation-only; the dependency and runtime code are unchanged.

| Check | Result |
|---|---|
| `python -B -m unittest discover -s scripts/tests -p 'test_*.py' -q` | 273 passed, 26 skipped; exit code 0. The suite intentionally prints a synthetic failed-CI summary fixture. |
| `pwsh -NoLogo -NoProfile -File .\scripts\Test-DependencyPolicy.ps1` | Passed for root and fuzz workspaces after online RustSec refresh; database commit `caee11f41fab07d676c0a4655a2a5dd1f94da97c`, timestamp `2026-10-14T15:17:45+02:00`; lockfile hashes unchanged. |
| Independent QA review | Passed with no remaining findings on implementation through `4eef85dd`; reviewer checklist recorded in `4e8d926a` and its accurate-limitations confirmation in `6e840e8a`. |
| Independent security review | Passed with no actionable findings on implementation through `4eef85dd`; no TLS policy, feature, secret handling, or dependency exception changes. Checklist recorded in `e537cbe1`; the PR-evidence item remains open until the draft PR exists. |
| Required GitHub PR CI | Pending on draft [PR #105](https://github.com/NeverWe1come/KMIPKit/pull/105); keep the PR in draft until the platform matrix passes. |

The implementation and security reviews examined the same code and dependency diff. Commit `4e8d926a` only records QA checklist results. The draft PR must remain open until required platform CI completes successfully.
