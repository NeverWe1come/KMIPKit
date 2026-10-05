# Dependency Policy Requirements Review

**Purpose**: Reviewer-owned checklist for evaluating the policy requirements.
**Feature**: [spec.md](../spec.md)

- [x] Do both root and fuzz graphs include all features, development licenses and duplicate checks, and supported target-specific dependencies?
- [x] Is the exact SPDX allowlist evidence-based and finite, with unknown or missing license data denied?
- [x] Do advisory defaults explicitly fail vulnerabilities, unsoundness, unmaintained crates, and yanked versions?
- [x] Are registry, Git, canonical path, duplicate (including dev-only), wildcard, and finite source-cited architecture-derived package bans explicit rather than dependent on tool defaults?
- [x] Are the ADR-0005 TLS-backend package bans non-waivable through the exception register?
- [x] Does every exception map to one exact finding, have mitigation and approval evidence, and expire within 90 days?
- [x] Does every pull request run the policy job regardless of changed paths, with `--locked`, successful online advisory refresh, and emitted RustSec commit SHA/time?
- [x] Does the scheduled job inspect the configured active release ref, and is its default-branch activation behavior documented?
- [x] Can contributors reproduce the same root and fuzz checks with the documented command?
- [x] Are CI permissions read-only, secret-free, and free of a mutable or unreviewed checker action?
- [x] Are the limitations of automated license metadata stated without implying legal completeness?
- [x] Are branch protection, release attestations, OASIS integrity, and language adapters clearly outside this feature?
- [x] Are the only manifest changes limited to FR-013's unpublished fuzz package license and matching local-path version metadata, with no resolved-version or lockfile change?

## Reviewer Findings

- **P1 — T017 verification remains open.** The task list still leaves T017 unchecked, and the implementation evidence says Linux/macOS and aggregate CI plus remaining repository gates are pending. Treat this review as code/policy review only; final readiness is not established.
- **P2 — CLOSED: FR-012 traceability.** The CSV and `EXPECTED_REQUIREMENT_TESTS` now require `CargoDenyDiagnosticTests.test_failure_report_retains_allowlisted_finding_fields_and_redacts_secrets`, which asserts package/version, rule, redacted source, advisory, and secret omission.
- **P2 — CLOSED: Missing/invalid license denial.** Dedicated cargo-deny fixtures now assert the exact `licenses` / `unlicensed` finding and package/version for both missing metadata and an invalid SPDX expression; the production config also explicitly denies private packages with missing license data.
- **P2 — CLOSED: FR-013/SC-008 lockfile and resolved-version coverage.** Both traceability rows now require `DependencyPolicyApiTests.test_policy_scans_preserve_both_lockfiles_and_resolved_package_versions`. It verifies cargo-deny 0.20.2, snapshots bytes and complete `name@version` sets from both lockfiles, and runs root/fuzz scans offline with `--locked` before comparing the snapshots.

## Re-review (2026-10-06)

The three P2 findings are closed. The focused verification command set `CARGO_DENY` to the installed cargo-deny 0.20.2 executable and ran the fixture suite, traceability suite, root/fuzz lockfile invariant, and FR-012 redaction test: 21 tests passed. The lockfile invariant exercised both root and fuzz scans in offline, locked mode and preserved both byte snapshots and all resolved `name@version` entries. The coordinating agent additionally recorded a passing end-to-end `scripts/Test-DependencyPolicy.ps1` run with both online RustSec refreshes and the newly wired offline invariant test.

T017 remains open: this review does not establish Linux/macOS or aggregate CI results, coverage and repository-wide gates, or final feature readiness.

## Initial Review Evidence (pre-fix)

Ran `scripts.tests.test_cargo_deny_fixtures` and `scripts.tests.test_requirement_traceability` with cargo-deny 0.20.2: 17 tests passed. Ran `scripts.tests.test_dependency_policy`: 48 tests passed, with the Windows directory-symlink case skipped because the host lacks the required privilege; the Linux/macOS CI matrix is expected to exercise it.
