# Implementation Evidence: Rustls Advisory Remediation

**Feature**: KMIPKIT-0045
**Branch**: `feature/KMIPKIT-0045-rustls-advisory-remediation`
**Date**: 2026-10-14

## Advisory baseline

The active `release/1.0.0` workspace pins `rustls = "=0.23.45"` with `default-features = false` and the `aws_lc_rs`/`std` features. The transport crate inherits this workspace dependency. The fuzz workspace has no Rustls dependency.

The RustSec advisory records published on 2026-10-14 list `>=0.23.46` as patched for RUSTSEC-2026-0346, RUSTSEC-2026-0347, RUSTSEC-2026-0348, and RUSTSEC-2026-0349. Stable Rustls 0.23.46 is available and declares an MSRV below KMIPKit's 1.94.

CI run [38596298478](https://github.com/NeverWe1come/KMIPKit/actions/runs/38596298478), on the PR #90 dependency state before this remediation, reports the `Dependency policy / pull request` check as failed. Other checks in that run are still in progress at the time of this record. The current workflow log is not yet available because the run has not completed.

The local online baseline command `pwsh -NoLogo -NoProfile -File .\scripts\Test-DependencyPolicy.ps1` also exited with code 1 after validating the reviewed Windows Rust host and cargo-deny 0.20.2. Its redacted diagnostic reported Rustls 0.23.45 findings for all four advisory IDs and no exact registered exception. The policy correctly failed closed; no waiver was added.

## Red evidence

Command:

```text
python -m unittest scripts.tests.test_rustls_advisory_pin -v
```

Result before changing the manifest or lockfile: **failed as expected**. Four tests ran; the exact-version/feature and traceability contracts passed, and `test_workspace_pin_is_exact_stable_patched_release` failed because `(0, 23, 45)` is below `(0, 23, 46)`.

The failure is the intended Red signal. Green results are recorded below; final cross-platform CI and independent review remain open.

## Green evidence

After changing only the exact workspace pin to `=0.23.46` and updating `Cargo.lock` with Cargo:

- `python -m unittest scripts.tests.test_rustls_advisory_pin -v`: **4 tests passed**.
- `cargo test -p kmipkit-transport --all-features`: **exit code 0** on Windows. The AWS-LC build emitted non-fatal environment warnings because NASM was unavailable, but the test command completed successfully.
- `pwsh -NoLogo -NoProfile -File .\scripts\Test-DependencyPolicy.ps1`: **passed** for root and fuzz workspaces with successful online RustSec refresh. Both report database commit `caee11f41fab07d676c0a4655a2a5dd1f94da97c`, timestamp `2026-10-14T15:17:45+02:00`. Existing dependency-policy exception IDs `KMIPKIT-0011-EX-003` and `KMIPKIT-0011-EX-004` remain reported; no Rustls advisory exception was added.
- Cargo reported only `rustls 0.23.45 -> 0.23.46` in `Cargo.lock`; no other package resolution changed. The manifest's `aws_lc_rs` and `std` feature list and disabled defaults are unchanged.

Full workspace verification, coverage, cross-platform CI, and independent QA/security review remain open; no final readiness claim is made yet.
