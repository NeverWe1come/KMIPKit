# Implementation Plan: Rustls Advisory Remediation

**Branch**: `feature/KMIPKIT-0045-rustls-advisory-remediation` | **Date**: 2026-10-14 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/045-rustls-advisory-remediation/spec.md`

## Summary

Replace the affected exact Rustls workspace pin with the earliest stable release explicitly patched for the four current RustSec advisories. Add a failing dependency-pin contract before changing the manifest, update only the root Rustls resolution, preserve TLS feature selection, and record fresh dependency-policy evidence for both workspaces.

## Technical Context

**Language/Version**: Rust Edition 2024, MSRV 1.94; Python 3.12 for repository script-contract tests
**Primary Dependencies**: Rustls 0.23.45 currently pinned; cargo-deny 0.20.2 for the approved dependency policy
**Storage**: Cargo manifests and committed lockfiles
**Testing**: Python unittest contract, Cargo transport/workspace tests, online `scripts/Test-DependencyPolicy.ps1`, CI matrix
**Target Platform**: Linux, Windows, and macOS CI; Rustls remains a shared Rust dependency
**Project Type**: Rust library workspace with repository validation scripts
**Performance Goals**: No runtime performance change is intended
**Constraints**: Keep an exact stable 0.23.x pin at or above 0.23.46; preserve the existing AWS-LC feature set and all non-Rustls package versions; do not modify policy exceptions or checks
**Scale/Scope**: One direct Rustls version update, one lockfile resolution, regression/traceability/evidence updates

## Constitution Check

- Specification and stable FR traceability are provided by this feature and `specification/compliance/requirements/KMIPKIT-0045.csv`; OASIS references are not applicable to a dependency-only maintenance change.
- Red/Green/Refactor evidence is required and kept in separate DCO commits.
- No KMIP protocol, public API, ABI, architecture, or TLS policy boundary changes.
- Existing Rustls/aws-lc-rs backend and TLS security defaults remain intact.
- The branch is based on `release/1.0.0`; final integration remains subject to human review and merge.

## Design Decisions

1. Update the exact workspace pin from `=0.23.45` to `=0.23.46`. This is the lowest compatible stable release that covers the four advisory patched ranges verified on 2026-10-14.
2. Use Cargo to refresh the committed root lockfile and verify that the only resolved package version change is Rustls. Do not edit lockfiles by hand.
3. Add a script-contract test that fails while the pin is below the advisory floor and passes after the update. The online dependency-policy workflow remains the authoritative dynamic advisory check.
4. Keep `.cargo/deny.toml`, its baseline, the exception register, policy scripts, and CI policy unchanged. An available patched release does not warrant an exception.
5. Preserve the `aws_lc_rs` and `std` feature set and keep optional features disabled.

## Project Structure

```text
specs/045-rustls-advisory-remediation/
├── spec.md
├── plan.md
├── research.md
├── clarifications.md
├── quickstart.md
├── tasks.md
├── implementation-evidence.md
└── checklists/
    ├── requirements.md
    ├── implementation.md
    └── security-review.md

Cargo.toml
Cargo.lock
scripts/tests/test_rustls_advisory_pin.py
specification/compliance/requirements/KMIPKIT-0045.csv
```

**Structure Decision**: Use the existing workspace manifest and lockfile, the established Python script-contract test suite, and the normative maintenance traceability directory. No production source module or generated API file changes are needed.

## Verification Strategy

1. Run the new focused contract before the manifest change and preserve its failing result in the Red commit.
2. Update the exact pin and Cargo lockfile; run the focused contract, transport tests, and the root/fuzz dependency-policy runner in online mode.
3. Verify Cargo feature metadata and compare the complete lockfile change to confirm no unrelated package version moved.
4. Run formatting, Clippy, workspace tests, the relevant script-contract suite, coverage, generators, and CI-required checks available for this feature.
5. Record RustSec database evidence from both workspace scans, the advisory IDs, the test commands/results, and platform limitations in `implementation-evidence.md` and the draft PR.

## Complexity Tracking

No constitution violations or architectural exceptions are introduced.
