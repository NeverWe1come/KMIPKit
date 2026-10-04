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
| FR-009 | T005, T016, T017, T018, T019, T021, T025 |
| FR-010 | T008, T011, T019, T020 |
| FR-011 | T005, T020, T021 |
| FR-012 | T009, T011 |
| SC-001 | T010, T011 |
| SC-002 | T010, T011 |
| SC-003 | T013, T014, T015 |
| SC-004 | T013, T014 |
| SC-005 | T005, T016, T017, T018, T019, T021, T025 |
| SC-006 | T008, T009, T011 |
| SC-007 | T020, T021, T025 |
| SC-008 | T008, T009, T011 |

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
