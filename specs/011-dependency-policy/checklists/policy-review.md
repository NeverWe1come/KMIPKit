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

- **P1 — CLOSED: T017 current-head CI and coverage evidence.** PR #37 run [37387765790](https://github.com/NeverWe1come/KMIPKit/actions/runs/37387765790) completed on head `dff30f84547260f98c69f69d86ae3398baf95d4e` with 15 successful checks and two expected skips (informational nightly branch coverage and active-release schedule). The six Core jobs across Linux, Windows, and macOS on stable and Rust 1.94 passed; script contracts, the dependency-policy PR check, normative inventory, all three coverage jobs, and the three-platform coverage gate also passed.
- **P2 — CLOSED: FR-012 traceability.** The CSV and `EXPECTED_REQUIREMENT_TESTS` now require `CargoDenyDiagnosticTests.test_failure_report_retains_allowlisted_finding_fields_and_redacts_secrets`, which asserts package/version, rule, redacted source, advisory, and secret omission.
- **P2 — CLOSED: Missing/invalid license denial.** Dedicated cargo-deny fixtures now assert the exact `licenses` / `unlicensed` finding and package/version for both missing metadata and an invalid SPDX expression; the production config also explicitly denies private packages with missing license data.
- **P2 — CLOSED: FR-013/SC-008 lockfile and resolved-version coverage.** Both traceability rows now require `DependencyPolicyApiTests.test_policy_scans_preserve_both_lockfiles_and_resolved_package_versions`. It verifies cargo-deny 0.20.2, snapshots bytes and complete `name@version` sets from both lockfiles, and runs root/fuzz scans offline with `--locked` before comparing the snapshots.

## Re-review (2026-10-06)

The three P2 findings are closed. The focused verification command set `CARGO_DENY` to the installed cargo-deny 0.20.2 executable and ran the fixture suite, traceability suite, root/fuzz lockfile invariant, and FR-012 redaction test: 21 tests passed. The lockfile invariant exercised both root and fuzz scans in offline, locked mode and preserved both byte snapshots and all resolved `name@version` entries. The coordinating agent additionally recorded a passing end-to-end `scripts/Test-DependencyPolicy.ps1` run with both online RustSec refreshes and the newly wired offline invariant test.

The current-head CI portion of T017 is complete: run 37387765790 passed the Linux, Windows, and macOS stable/Rust 1.94 Core matrix, script-contract jobs, dependency-policy PR check, normative-inventory check, and three-platform coverage gate.

## Initial Review Evidence (pre-fix)

Ran `scripts.tests.test_cargo_deny_fixtures` and `scripts.tests.test_requirement_traceability` with cargo-deny 0.20.2: 17 tests passed. Ran `scripts.tests.test_dependency_policy`: 48 tests passed, with the Windows directory-symlink case skipped because the host lacks the required privilege; the Linux/macOS CI matrix is expected to exercise it.

## Independent QA review (2026-10-06)

Reviewed the current dff30f8 head against the approved specification, T028-T032, the 21 FR/SC traceability rows, the policy parser/configuration, contributor docs, and implementation evidence. No additional blocking or high-severity implementation issue was found.

T028-T032 evidence is consistent with the implementation: waiver-free scans are parsed as complete structured output and matched against exact exceptions; yanked exceptions are package/version-specific; cargo-deny local exception files are rejected; summary helps counts are validated without requiring diagnostic counterparts; and cargo-deny's 0.20.2 check exit status is validated as an exact section bitset (advisories/bans/licenses/sources = 1/2/4/8). The pinned upstream [0.20.2 exit-bit implementation](https://raw.githubusercontent.com/EmbarkStudios/cargo-deny/0.20.2/src/cargo-deny/stats.rs#L69-L81) confirms the mapping. Regression coverage includes exit 4 for license-only findings, exit 6 for bans plus licenses, the sources bit, mismatched masks, and out-of-range masks.

Fresh local verification with cargo-deny 0.20.2:

- python -X utf8 -m unittest scripts.tests.test_dependency_policy: 63 passed, 4 skipped.
- python -X utf8 -m unittest scripts.tests.test_cargo_deny_fixtures: 18 passed.
- python -X utf8 -m unittest scripts.tests.test_requirement_traceability: 2 passed; all FR-001 through FR-013 and SC-001 through SC-008 rows are covered.
- python -X utf8 -m unittest scripts.tests.test_workflow: 16 passed.
- python -X utf8 -m unittest discover -s scripts/tests: 142 passed, 5 skipped.
- DependencyPolicyApiTests.test_policy_scans_preserve_both_lockfiles_and_resolved_package_versions: passed with CARGO_DENY set to the installed 0.20.2 executable.
- git diff --check: passed.

The independent QA review found no new code finding. With run 37387765790 complete and the current-head PR-policy criterion satisfied, the QA condition for T018 is clear to close. The final run result is 15 successful checks and two expected skips.
