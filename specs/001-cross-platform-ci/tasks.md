# Tasks: Cross-Platform CI and Local Tests

**Input**: Design artifacts under `specs/001-cross-platform-ci/`
**Prerequisites**: `spec.md`, `plan.md`, `research.md`, `data-model.md`, `contracts/powershell-cli.md`, `quickstart.md`

**Execution note**: One implementer is active during the foundation phase. Tasks are intentionally sequential; there are no `[P]` tasks. Keep Red, Green, and Refactor evidence in separate signed commits.

## Requirement Traceability

| Requirement | Implementing tasks |
|---|---|
| FR-001 | T008, T010, T011 |
| FR-002 | T010, T011 |
| FR-003 | T010, T011, T025 |
| FR-004 | T004, T013, T015 |
| FR-005 | T004, T013, T014 |
| FR-006 | T004, T013, T015 |
| FR-007 | T004, T012, T013, T014 |
| FR-008 | T008, T009, T011 |
| FR-009 | T005, T016, T017, T018, T019, T021, T025, T034, T035, T036, T037, T038, T039 |
| FR-010 | T008, T011, T019, T020 |
| FR-011 | T005, T020, T021 |
| FR-012 | T009, T011 |
| FR-013 | T040, T041 |
| FR-014 | T043, T044, T045, T046, T047 |
| FR-015 | T049, T050, T051, T052 |
| FR-016 | T049, T050, T051, T052 |
| FR-017 | T049, T050, T051, T052 |
| SC-001 | T010, T011 |
| SC-002 | T010, T011 |
| SC-003 | T013, T014, T015 |
| SC-004 | T013, T014 |
| SC-005 | T005, T016, T017, T018, T019, T021, T025, T034, T035, T036, T037, T038, T039 |
| SC-006 | T008, T009, T011 |
| SC-007 | T020, T021, T025 |
| SC-008 | T008, T009, T011 |
| SC-009 | T040, T041, T042 |
| SC-010 | T043, T044, T045, T046, T047 |
| SC-011 | T049, T050, T051, T052 |

## Phase 1: Setup

**Purpose**: Establish traceable implementation and review evidence.

- [x] T001 Verify the FR/SC task mapping above and confirm no external hooks are configured in `.specify/extensions.yml`.
- [x] T002 Validate all paths and decisions across `specs/001-cross-platform-ci/spec.md`, `plan.md`, `research.md`, `data-model.md`, `contracts/powershell-cli.md`, and `quickstart.md`.
- [x] T003 Resolve critical/high QA and security findings in the feature artifacts before implementation.

## Phase 2: Foundational test harness

**Purpose**: Define expected WSL and coverage behavior before production scripts exist.

- [x] T004 Add self-contained PowerShell tests for WSL listing parsing, Ubuntu filtering, explicit/ambiguous selection, WSL2 validation, translated `--cd` working-directory arguments, safe native argv, and exit-code preservation in `scripts/tests/Test-Wsl.ps1`.
- [x] T005 Add Python `unittest` cases for LLVM JSON function-region validation, conservative Rust executable-function preflight (body vs semicolon declaration, nested comments, raw strings containing `fn`, excluded test paths, unreadable/invalid input and ambiguous syntax treated as eligible, and inline `#[cfg(test)]` rejection), coverage paths, changed Rust line parsing, executable region handling, multi-platform merge, threshold boundaries, unavailable source, and fail-closed report errors in `scripts/tests/test_coverage_gate.py`; include static workflow policy cases in `scripts/tests/test_workflow.py`.
- [x] T006 Run the PowerShell and Python test suites before implementation; confirm their failures identify the missing modules/functions rather than syntax or harness errors.
- [x] T007 Commit the failing tests and record the exact RED commands/results with `git commit -s`.

## Phase 3: User Story 1 - Pull request platform checks (Priority: P1)

**Goal**: Every pull request to `master` or `release/*` receives the Rust check matrix on Linux, Windows, and macOS.

**Independent Test**: Inspect a workflow run for all six OS/toolchain combinations and confirm a deliberately failing core check fails its job.

- [x] T008 [US1] Add `.github/workflows/ci.yml` with `pull_request` targets `master` and `release/**`, explicit `contents: read`, no secrets, no privileged triggers, and checkout with credentials not persisted.
- [x] T009 [US1] Pin checkout v7.0.1, upload-artifact v7.0.1, and download-artifact v8.0.1 in `.github/workflows/ci.yml` to the verified full commit SHAs and annotate each release.
- [x] T010 [US1] Add the Ubuntu/Windows/macOS × Rust 1.94/stable matrix to `.github/workflows/ci.yml`; run `cargo fmt --all --check`, workspace Clippy with `-D warnings`, workspace tests with all features, and rustdoc with warnings denied.
- [x] T011 [US1] Run the static workflow contract tests in `scripts/tests/test_workflow.py`; confirm target branches, read-only permissions, safe triggers, and full SHA pins match the approved spec.

## Phase 4: User Story 2 - Run tests from Windows through WSL (Priority: P1)

**Goal**: A single documented PowerShell command runs tests in an existing WSL2 Ubuntu distribution and returns the Cargo status.

**Independent Test**: Run the command in the current checkout; exercise default and explicit distro selection; confirm missing prerequisites and test failures return non-zero.

- [x] T012 [US2] Implement pure parsing, Ubuntu selection, WSL2 checks, and native argument builder functions in `scripts/KmipKit.WslTest.psm1` to satisfy the RED tests.
- [x] T013 [US2] Implement `scripts/Test-Wsl.ps1` to preflight PowerShell 7.3+, repository root, WSL2 Ubuntu, existing Cargo 1.94.0, and `wslpath`; invoke Cargo via `wsl.exe --exec` without shell interpolation.
- [x] T014 [US2] Run `scripts/tests/Test-Wsl.ps1` and the documented workspace command on WSL Ubuntu; verify output and native non-zero status propagation.
- [x] T015 [US2] Extend `docs/development/rust-workspace.md` and `docs/development/testing.md` with the exact PowerShell command, prerequisites, multi-Ubuntu selection behavior, no-provisioning guarantee, and troubleshooting.

## Phase 5: User Story 3 - Coverage readiness (Priority: P2)

**Goal**: Report coverage as unavailable only with no eligible executable production source; otherwise enforce all documented line thresholds from strict three-platform reports.

**Independent Test**: Run the fixture suite for valid, below-threshold, missing, malformed, path-escaping, platform-specific, and no-code cases; verify exit status and diagnostics.

- [x] T016 [US3] Implement strict LLVM JSON parsing; derive per-line counts from file segments, cross-check file line summaries, validate function-region schema and source coordinates, and reject unsupported schema, invalid counts, and paths outside the workspace.
- [x] T017 [US3] Implement exact base-commit-to-checked-merge-commit Rust diff extraction, Git-quoted UTF-8 path decoding, and changed executable line aggregation across Linux, Windows, and macOS reports in `scripts/coverage_gate.py`.
- [x] T018 [US3] Enforce 95% changed-code when changed executable lines exist, 95% TTLV/protocol, 85% transport/FFI/bindings, and 90% workspace line coverage in `scripts/coverage_gate.py`; return `not applicable` for zero changed executable lines and `unavailable` only when source preflight finds no production function bodies.
- [x] T019 [US3] Add stable three-OS coverage jobs and uniquely named JSON report or no-code sentinel artifacts, plus a dependent Linux aggregation job, to `.github/workflows/ci.yml`; use the exact checked merge SHA and base SHA.
- [x] T020 [US3] Add a separate nightly-only Linux branch-coverage job with `continue-on-error: true` to `.github/workflows/ci.yml`; publish branch data as informational and ensure job/tool failures cannot gate pull-request success.
- [x] T021 [US3] Update `docs/development/testing.md` with report-segment semantics, per-file summary validation, thresholds, unavailable/failure distinctions, source-tree inclusion, and reproduction commands.

## Phase 6: Green and Refactor evidence

**Purpose**: Establish passing behavior and improve maintainability without changing scope.

- [x] T022 Run all script tests after the minimal implementation and record GREEN commands/results.
- [x] T023 Commit the minimal passing implementation with `git commit -s`, separate from RED and Refactor commits (`9bd52db`).
- [x] T024 Refactor symlink-scan diagnostics into a shared helper while retaining tests and error semantics.
- [x] T025 Run format, Clippy, Rust tests, Rust docs, PowerShell tests, Python tests, workflow syntax/action-pin checks, and `git diff --check`; record REFACTOR results.
- [x] T026 Commit refactoring and CI/docs integration with `git commit -s`; confirm distinct RED, GREEN, and Refactor commits remain visible.

## Phase 7: Polish and cross-cutting review

- [x] T027 Run Spec Kit convergence against the final diff; no remaining gaps or convergence tasks were found.
- [x] T028 Run QA review against FRs, SCs, contracts, documentation, and test evidence; resolve findings. Final review found no gaps.
- [x] T029 Run independent security review of fork permissions, action pins, WSL argument flow, and coverage path/schema handling; resolve findings. Final review found no blocking issues.
- [x] T030 Verify the final branch is based on `release/1.0.0`, run all locally available checks, confirm no ignored generated/OASIS source was changed, and prepare draft PR #3 with scope, rationale, Red/Green/Refactor evidence, verification, risks, and limitations.
- [x] T031 Add a workflow regression test requiring `branch-coverage` to run only for `schedule`, fix the missing job-level condition, and refactor shared job extraction. Evidence: RED `2104698`; GREEN `e2965b3`; REFACTOR `c8769d4`; `python -m unittest discover -s scripts/tests -p 'test_*.py' -v` passed 35 tests with 3 Windows symlink-permission skips.
- [x] T032 Align the testing guide with the nightly-only branch-coverage trigger and add a regression contract for the documented schedule-only behavior. Evidence: RED `19e57b7`; GREEN `ae64bba`; REFACTOR `df71ff4`; `python -m unittest discover -s scripts/tests -p 'test_*.py' -v` passed 36 tests with 3 Windows symlink-permission skips.
- [x] T033 Re-run final verification in WSL Ubuntu 26.04 at `b1a4dac`: Python script suite passed 36 tests with no skips; `cargo fmt --all --check`, workspace Clippy, workspace tests, and rustdoc passed. Rust crates currently contain zero unit tests; coverage preflight reports unavailable because no production function bodies exist.
- [x] T034 [US3] Add regressions for LLVM export schema 3.1, per-function line summaries, function region start-line mapping, summary-only uncovered residuals, platform residual aggregation, and shared physical lines. RED evidence: commit `62752f5`; `python -m unittest discover -s scripts/tests -p 'test_coverage_gate.py' -v` ran 38 tests and exposed 9 failures against the prior parser (8 errors, 1 assertion failure; 3 Windows symlink skips).
- [x] T035 [US3] Reconcile LLVM function summaries and physical file segments, validate region start-line mappings, and conservatively count unexplained uncovered residuals in package, workspace, and changed-code coverage. GREEN evidence: commit `b914cb4`; `python -m unittest discover -s scripts/tests -p 'test_coverage_gate.py'` passed 38 tests with 3 Windows symlink skips.
- [x] T036 [US3] Update the approved coverage specification, data model, research record, and testing guide to state reviewed schema support and conservative summary reconciliation; verify the documentation and full script suite. Evidence: `python -m unittest discover -s scripts/tests -p 'test_*.py'` passed 45 tests with 3 Windows symlink skips; normative catalog tests passed 130 tests with 6 platform skips; catalog validation/report checks and `git diff --check` passed.
- [x] T037 [US3] Add a multi-export regression where one `CoverageMapping` reports an uncovered shared line and another reports it covered; require the parser to preserve or reject the repeated workspace source path rather than reconciling across mappings. RED evidence: commit `863bab2`; the focused test failed because the parser returned without raising.
- [x] T038 [US3] Reject repeated workspace source paths across distinct LLVM export mappings and duplicate file records within one mapping. GREEN evidence: commit `24a3567`; the focused regression passed and the coverage-gate suite passed 39 tests with 3 Windows symlink skips.
- [x] T039 [US3] Document the repeated-path fail-closed rule and the LLVM export-object boundary in the specification, data model, research record, and testing guide; rerun all script/catalog checks and obtain a scoped review. Evidence: full script suite passed 47 tests with 3 Windows symlink skips; normative catalog suite passed 130 tests with 6 platform skips; catalog validation, report check, source audit, and `git diff --check` passed; scoped review found no remaining findings.

## Phase 8: Self-hosted ARM64 Linux runner routing

- [x] T040 Add a workflow contract requiring same-repository Linux PR jobs and scheduled branch coverage to use the self-hosted ARM64 runner, while fork PRs and Windows/macOS jobs remain hosted. RED evidence: commit `c820eaa`; `python -m unittest scripts.tests.test_workflow -v` failed the new routing contract for all five pull-request jobs and the scheduled job.
- [x] T041 [US1] Route trusted Linux workflow jobs to the self-hosted ARM64 runner, preserve GitHub-hosted fallback for fork PRs and Windows/macOS, and update the approved requirements and testing guide. GREEN evidence: focused workflow contracts passed 8 tests; full Python script suite passed 48 tests with 3 Windows symlink-permission skips; `pwsh -File scripts/tests/Test-Wsl.ps1` passed; `git diff --check` passed.
- [x] T042 Run workflow contract and repository validation checks, record GREEN/REFACTOR results, update task evidence, and prepare the draft PR against `release/1.0.0`. REFACTOR contract tests passed (8); full Python suite passed (48, 3 Windows symlink-permission skips); WSL contracts passed (8); `git diff --check` passed. Draft PR: #17.

## Phase 9: At-a-glance GitHub run summaries

**Purpose**: Make the outcome of existing CI checks easy to understand in the GitHub run Summary without changing what executes.

- [x] T043 [US4] Add failing contracts for pull-request and schedule summaries, event-inapplicable rows, required failure propagation, informational branch coverage, coverage metrics/diagnostics, and workflow wiring. RED evidence: commit `65fff76`; the new summary contracts failed because the renderer and final job did not exist.
- [x] T044 [US4] Implement the event-aware run summary renderer and append measured, unavailable, or failed coverage details to the coverage gate job Summary. GREEN evidence: script suite passed 156 tests (25 environment/tool skips); summary and coverage gate regressions passed.
- [x] T045 [US4] Update the approved feature specification, plan, data model, research, quickstart, and testing guide with FR-014/SC-010 and the presentation-only behavior.
- [x] T046 [US4] Run the complete Python, normative catalog, PowerShell, Rust, workflow, and diff checks; review summary correctness and evidence before requesting review. Evidence: Python script suite 156 passed (25 skipped because the pinned cargo-deny binary or Windows symlink privileges were unavailable); normative catalog suite 169 passed (7 Windows symlink skips); catalog validate/report/generated checks passed; PowerShell WSL suite passed 8 contracts; `pwsh -File scripts/Test-Wsl.ps1` passed; Rust 1.94.0 fmt, Clippy, and rustdoc passed; isolated-target `cargo llvm-cov --workspace --all-features --locked --summary-only` passed with 95.69% line coverage; `git diff --check` passed. Workflow syntax was reviewed through workflow contract tests; standalone actionlint/YAML parser was not installed.
- [x] T047 [US4] Refactor the summary implementation without changing its output contract; rerun relevant checks and record separate Refactor evidence. Refactor evidence: `python -m unittest discover -s scripts/tests -p 'test_*.py'` passed 156 tests (25 environment/tool skips); `git diff --check` passed.
- [x] T048 [US4] Push the feature branch and prepare the draft PR against `release/1.0.0` with rationale, verification, risks, and Red/Green/Refactor evidence. Draft PR: [#42](https://github.com/NeverWe1come/KMIPKit/pull/42).

## Phase 10: Per-job CI failure diagnostics

- [x] T049 [US5] Extend summary and workflow contracts to require actionable final diagnostics for every job and complete required-job accounting in the run Summary. RED evidence: commit `a43c51a`; running the focused contracts from that commit archive produced the expected 31 failures and 2 errors across 51 tests.
- [x] T050 [US5] Add a tested diagnostic renderer for failed steps, wire an always-running final diagnostic step into each job, and include all event-required jobs and available diagnostics in the run Summary. GREEN evidence: commits `e7152ac`, `aeffe68`, and `91e68f8`; the focused diagnostic/summary/workflow suite passed 54 tests and the full script suite passed 216 tests (26 skipped). Follow-up RED commits `b770e6a` and `b7c00df` captured GitHub's null matrix context and specific normative/coverage failure guidance.
- [x] T051 [US5] Update the testing guide and approved CI specification/task traceability; add a fallback Summary if run-summary checkout/rendering fails; verify Markdown escaping, exit-status preservation, event-specific skips, matrix failures, and scheduled informational behavior. The approved specification and traceability were included with RED; the testing guide documents the job summaries, step logs, matrix behavior, and fallback.
- [x] T052 Run the complete Python script contracts, normative catalog suite and generated checks, PowerShell contracts, relevant Rust checks, and `git diff --check`; record Refactor evidence and prepare a draft PR from `feature/001-ci-failure-diagnostics` to `release/1.0.0`. Verification: `python -B -m unittest discover -s scripts/tests -p 'test_*.py'` passed 216 tests (26 skipped); normative catalog tests passed 170 (7 skipped); catalog validation, generated report/tag/result checks, and immutable OASIS source check passed; `pwsh -File scripts/tests/Test-Wsl.ps1` passed 8 contracts; `cargo fmt --all --check`, workspace Clippy, tests, rustdoc, and `cargo llvm-cov --workspace --all-features --locked --summary-only` passed (77.84% aggregate; this command does not enforce the multi-platform CI gate); workflow contracts passed; `git diff --check` passed. No standalone YAML parser or actionlint is installed in this environment. Draft PR: [#54](https://github.com/NeverWe1come/KMIPKit/pull/54).
- [x] T053 Fix run-summary fallback ordering so it runs only after the normal renderer had a chance to set `summary_written`; regression RED: commit `5f32a44` failed because fallback preceded the renderer; GREEN: commit `afea0a3` moved fallback after rendering; REFACTOR evidence commit updates this record. The focused workflow contract passed; `python -m unittest discover -s scripts/tests -v` passed 216 tests (26 skipped); `git diff --check` passed. GitHub verification is pending the next PR run.

## Dependencies

```text
Setup (T001–T003)
  └── Foundational RED tests (T004–T007)
       ├── US1 CI matrix (T008–T011)
       ├── US2 WSL runner (T012–T015)
       └── US3 coverage (T016–T021)
            └── Green commit and Refactor (T022–T026)
                 └── Converge, QA, security, and draft PR (T027–T030)
```

The user stories are independently verifiable after the shared test harness, but their implementation is sequential during foundation stabilization. There are no parallel execution assignments for this feature.

## Implementation Strategy

Deliver all three user stories in this infrastructure PR. Tests lead each script behavior change. The Green commit contains the minimal working scripts, workflow, and docs; the Refactor commit contains cleanup that preserves behavior. The result prepares the repository for protocol implementation; it does not implement KMIP protocol behavior.
