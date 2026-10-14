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

The failure is the intended Red signal. Green evidence and final cross-platform/security review remain to be recorded by later implementation tasks; they are not claimed here.
