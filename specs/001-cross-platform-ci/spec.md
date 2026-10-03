# Feature Specification: Cross-Platform CI and Local Tests

**Feature Branch**: `feature/KMIPKIT-0001-cross-platform-ci`

**Created**: 2026-10-04

**Status**: Authorized for autonomous execution by direct user instruction (2026-10-04)

**Input**: User description: Establish the infrastructure foundation before protocol work: GitHub Actions must run checks on Linux, Windows, and macOS, and developers using PowerShell on Windows must be able to run the Rust test suite through the already installed WSL2 Ubuntu environment. Preserve Rust 1.94 as the MSRV and keep the workflow safe for contributions from forks.

## Context and References

This feature changes contributor and pull-request verification only; it introduces no KMIP protocol behavior, so no OASIS clauses apply. The constraints and existing check commands come from `AGENTS.md`, `.specify/memory/constitution.md`, `docs/development/testing.md`, `docs/development/rust-workspace.md`, and `docs/development/git-and-releases.md` (Package integrity). `docs/security/threat-model.md` identifies compromised build actions as a high supply-chain risk.

This feature includes coverage collection and enforcement of the thresholds already documented in `docs/development/testing.md`. When no production Rust source is eligible for instrumentation, coverage is reported as unavailable and no threshold is claimed. When eligible source exists but coverage data cannot be collected, the check fails. The offline OASIS source-integrity and dependency/advisory/license policy checks are separate specifications in the infrastructure phase. Deterministic-generation checks depend on the later approved catalog and generator. Normative traceability becomes applicable with the catalog and protocol code; ABI and adapter checks become applicable when those surfaces exist. Administrator-only branch-protection settings remain outside repository code changes.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Pull request platform checks (Priority: P1)

As a contributor, I need every pull request to receive the same core Rust checks on Linux, Windows, and macOS so that platform-specific regressions are visible before review.

**Why this priority**: Cross-platform validation is a prerequisite for accepting implementation work and is explicitly required by the project testing strategy.

**Independent Test**: Open a pull request with a deliberately failing test or lint condition and verify the relevant check fails; restore it and verify the required platform checks pass.

**Acceptance Scenarios**:

1. **Given** a pull request targeting a supported integration branch, **When** the workflow runs, **Then** it reports test results for Linux, Windows, and macOS.
2. **Given** the workspace declares Rust 1.94 as its MSRV, **When** supported Rust checks run, **Then** tests are exercised on the MSRV and the current stable toolchain.
3. **Given** a formatting, lint, test, or documentation check fails, **When** the workflow completes, **Then** the pull request receives a failing result identifying the check.
4. **Given** a pull request originates from a fork, **When** checks run, **Then** they need no repository secrets, do not run fork-controlled code in a privileged base-context workflow, and do not expose an unnecessary repository token to fork-controlled steps; any token used has no write-capable permission in any scope.
5. **Given** a CI run completes, **When** its results are reported, **Then** it has not published a package or release and has not changed repository settings or branch protections.
6. **Given** the workflow uses externally sourced actions, **When** its action references are reviewed, **Then** every reference points to a full commit SHA and identifies the upstream version in a comment.

---

### User Story 2 - Run tests from Windows through WSL (Priority: P1)

As a Windows contributor, I need one documented PowerShell command to run the workspace tests in the installed WSL2 Ubuntu environment so that local operating-system execution restrictions do not prevent TDD.

**Why this priority**: The current Windows host blocks a test executable, while the installed WSL distribution can run the Rust suite.

**Independent Test**: From the repository root in PowerShell, run the documented command and verify it executes the workspace test suite in WSL and returns the test process exit status.

**Acceptance Scenarios**:

1. **Given** WSL2, an Ubuntu distribution, and the required Rust toolchain are available, **When** the contributor runs the documented command, **Then** all workspace tests with all features execute in the current checkout.
2. **Given** a test fails in WSL, **When** the command returns, **Then** PowerShell exits with a non-zero status and preserves useful test output.
3. **Given** WSL, Ubuntu, or the required toolchain is unavailable, **When** the command is run, **Then** it fails promptly with an actionable message and does not install software or change machine configuration automatically.
4. **Given** the repository path contains spaces or non-ASCII characters, **When** the command is run, **Then** WSL resolves the same checkout and runs the tests there.
5. **Given** more than one Ubuntu distribution is installed, **When** no distribution is selected, **Then** the command reports the ambiguity; **When** a distribution is selected explicitly, **Then** the tests run in that distribution.

---

### User Story 3 - Coverage readiness (Priority: P2)

As a maintainer, I need the validation workflow to make coverage collection and its status visible so that the documented coverage gates can be enforced as testable code is added.

**Why this priority**: The project defines coverage thresholds, but the current workspace has no production Rust logic that can be measured.

**Independent Test**: Run the workflow with eligible production source and coverage-producing tests, then verify that it exposes a report and distinguishes a failed or unavailable result from a passing threshold.

**Acceptance Scenarios**:

1. **Given** production Rust source is eligible for instrumentation and coverage data is collected, **When** the coverage check runs, **Then** it reports coverage against the applicable thresholds documented in `docs/development/testing.md`.
2. **Given** the workspace has no production source eligible for instrumentation, **When** coverage collection runs, **Then** the result is explicitly marked as unavailable and is not represented as passing a coverage threshold.
3. **Given** eligible production source exists but coverage data is missing or incomplete, **When** the coverage check runs, **Then** the check fails.
4. **Given** eligible production source exists, **When** the separate nightly branch-coverage job runs, **Then** it reports branch coverage as informational and cannot fail the required pull-request checks based on its percentage or tool availability.
5. **Given** eligible production source exists and any applicable documented line-coverage threshold is not met, **When** the coverage check runs, **Then** the check fails.
6. **Given** coverage reports exist for Linux, Windows, and macOS, **When** changed-line coverage is calculated, **Then** the checker uses LLVM JSON function code-region records to count changed executable Rust lines across the three reports and fails closed on a missing or malformed report.
7. **Given** the pull request changes no executable Rust lines, **When** the changed-code threshold is evaluated, **Then** it reports `not applicable` and does not claim a passing percentage; applicable workspace and crate thresholds still run.
8. **Given** the repository has no executable production Rust function bodies, **When** per-platform coverage collection runs, **Then** each platform emits an explicit `unavailable` status artifact and aggregation reports unavailable; if executable production code exists, an absent LLVM report fails.

---

### Edge Cases

- WSL has no Ubuntu distribution, or more than one Ubuntu distribution is installed and no selection is supplied.
- The checkout is on a Windows path containing spaces or non-ASCII characters.
- A CI job is cancelled, times out, or cannot download a toolchain.
- A workflow is triggered by an untrusted fork pull request.
- A coverage command finds no instrumented tests or partial coverage data.
- A pull request changes no executable Rust lines, or the initial workspace contains no executable Rust function bodies.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Every pull request targeting `master` or a `release/*` branch MUST run the core workspace checks on Linux, Windows, and macOS.
- **FR-002**: The test matrix MUST exercise both the declared Rust 1.94 MSRV and the current stable Rust toolchain.
- **FR-003**: The core checks MUST include formatting, Clippy with warnings denied, workspace tests with all features, and documentation with warnings denied.
- **FR-004**: A documented PowerShell entry point MUST run `cargo test --workspace --all-features` in the current checkout through WSL2 Ubuntu, explicitly setting the WSL working directory to the translated current checkout path.
- **FR-005**: The PowerShell entry point MUST return a non-zero exit code when prerequisites or tests fail, preserve test output, and provide an actionable error for missing prerequisites.
- **FR-006**: The PowerShell entry point MUST NOT install toolchains, mutate WSL distributions, or alter host configuration automatically.
- **FR-007**: The PowerShell entry point MUST handle paths containing spaces and non-ASCII characters and MUST support explicit Ubuntu distribution selection when automatic selection is ambiguous.
- **FR-008**: Fork pull-request checks MUST require no secrets, MUST NOT run fork-controlled code in a privileged base-context workflow, and MUST NOT expose an unnecessary repository token to fork-controlled steps. Any token used MUST have no write-capable permission in any scope.
- **FR-009**: CI MUST include all non-test, non-generated Rust source covered by the project coverage policy; any generated-source exclusion MUST have a documented reason. Coverage MUST be collected on Linux, Windows, and macOS for the exact GitHub pull-request merge commit checked out by the workflow. The checker MUST compare that tree to the exact pull-request base commit using a two-tree diff. The coverage gate MUST parse LLVM JSON function code-region records and compute changed-code coverage from executable Rust lines changed between those trees, combining execution counts across the three platform reports. Non-executable source lines MUST NOT enter the denominator. If executable Rust lines changed, the changed-code threshold MUST be evaluated; if none changed, the changed-code result MUST be reported as `not applicable`, never as a passing percentage. A missing or malformed platform report MUST fail whenever executable production source exists. The `unavailable` exception is permitted only after a complete conservative scan of all non-test, non-generated Rust files under `crates/*/src/` finds no executable function body. The scanner MUST ignore comments and string/character literals, distinguish body-bearing functions from declarations ending in `;`, and treat unreadable files or syntax it cannot classify as eligible source so coverage is required. Inline `#[cfg(test)]` modules in production source MUST fail preflight with guidance to move tests to excluded test paths. Each platform MUST emit an explicit `unavailable` status artifact only after this preflight succeeds with no eligible body; aggregation MUST report unavailable and MUST NOT represent it as passing any threshold. When eligible executable source exists, CI MUST fail if collection is incomplete or if any applicable line threshold in `docs/development/testing.md` is missed, including the 95 percent changed-code threshold.
- **FR-010**: CI MUST NOT publish packages or releases, change branch protections, or require credentials unavailable to fork pull requests.
- **FR-011**: When eligible production source exists, CI MUST attempt branch coverage in a separate informational nightly-toolchain job and MUST NOT gate pull-request success on its percentage or on that job's tool availability. Branch coverage MUST NOT become a required gate until its reliability is proven and a separately reviewed change promotes it.
- **FR-012**: Every externally sourced GitHub Actions `uses` reference in the workflow MUST be pinned to a full commit SHA; a comment MUST identify the corresponding upstream release or version.

### Key Entities *(include if feature involves data)*

- **CI run**: A verification run associated with a commit or pull request, including platform, toolchain, check result, and diagnostic output.
- **Local test invocation**: A PowerShell request to test the current checkout in a selected WSL Ubuntu distribution, including its exit status.
- **Coverage result**: Per-platform coverage report/status and aggregate threshold status: measured, failed, unavailable when no production executable function bodies exist, or changed-code `not applicable` when the PR changes no executable Rust lines.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Every pull request targeting `master` or a `release/*` branch reports core check outcomes for all three required operating systems.
- **SC-002**: The core test suite is exercised under both Rust 1.94 and stable on every required operating system.
- **SC-003**: A Windows contributor can start the full workspace test suite with one documented PowerShell command and receive the actual test result as the command exit status.
- **SC-004**: Missing WSL prerequisites produce a non-zero result with a corrective message and no automatic host changes.
- **SC-005**: All eligible Rust source is included in coverage unless a generated-source exclusion has a documented reason. Coverage status artifacts exist for Linux, Windows, and macOS and describe the checked-out PR merge tree. The changed-code gate reports executable changed lines and fails if required line data is missing/malformed or coverage is below 95 percent; when no executable Rust lines changed, it reports `not applicable`, not a passing percentage. When eligible source exists, missing/incomplete data or failure to meet any applicable documented line-coverage threshold fails CI. With no executable production Rust function bodies, every platform emits `unavailable`, never a passing threshold.
- **SC-006**: Fork pull requests can complete checks without secrets or write-capable repository permissions.
- **SC-007**: When eligible production source exists, a separate informational job attempts a branch-coverage report; neither branch percentage nor nightly tool availability can fail a pull request before a separately reviewed gate change.
- **SC-008**: All externally sourced workflow actions are pinned to full commit SHAs, and fork-controlled steps run without secrets, write permissions, or unnecessary repository tokens.

## Assumptions

- The repository already has GitHub Actions enabled and hosted runners are available for the required operating systems.
- Rust 1.94 is the minimum supported version; stable means the current stable release at workflow execution time.
- The contributor has WSL2 and at least one Ubuntu distribution installed. When multiple Ubuntu distributions are present, the command accepts an explicit distribution selection.
- The WSL runner uses the current repository checkout through its Windows-mounted path, sets that translated path as WSL's working directory using `wsl.exe --cd`, and does not create a second checkout.
- Coverage thresholds remain governed by `docs/development/testing.md`; this feature must not lower or silently waive them.
- The initial workspace has no production Rust logic to measure. Coverage is unavailable only while no production Rust source is eligible for instrumentation; once eligible source exists, absent or incomplete data fails the check and the documented thresholds apply.
- Rust coverage exclusions are limited to generated source with a documented reason; production code cannot be classified as unavailable to avoid a threshold.
- Changed executable lines are derived from LLVM JSON function code-region records, not text-line heuristics or LCOV records that may omit lines. The checker maps each region's file ID through its function filename table and combines per-line execution counts from Linux, Windows, and macOS reports; a changed line enters the denominator if LLVM identifies it as executable in at least one required-platform report.
- GitHub's default pull-request checkout is the merge commit. `fetch-depth: 0` makes the event base commit available; coverage uses the checked-out `github.sha` tree and a two-tree diff from `github.event.pull_request.base.sha`, so the diff and reports describe the same tree.
- The no-code exception is determined before coverage collection by a complete scan of all non-test, non-generated `*.rs` files under `crates/*/src/`. The scanner ignores comments and string/character literals and recognizes function bodies, not declarations ending in `;`. If it cannot read a file or confidently classify syntax, it treats the source as eligible so coverage is required. Inline `#[cfg(test)]` modules in production source fail preflight and must be moved to cargo-llvm-cov-excluded test paths. Only a successful scan with no executable function bodies produces one explicit unavailable status artifact per platform; a coverage tool failure after executable source is detected is a failed result.
- When a PR contains no changed executable lines, only the changed-code metric is `not applicable`; all package/workspace coverage gates remain applicable if executable source exists.
- To keep test code out of production line-coverage totals, Rust tests must live in `tests/`, `tests.rs`, `*_tests.rs`, or `*-tests.rs` paths that cargo-llvm-cov excludes by default; no inline `#[cfg(test)]` modules are added to production source files.
- Externally sourced GitHub Actions are pinned by full commit SHA as required by the package-integrity guidance.
- OASIS source hash checking and dependency/advisory/license policy are separate specifications in the infrastructure phase. Deterministic generated-file checking follows the approved catalog and generator. Normative traceability, ABI, and adapter checks are added before those code surfaces become applicable. Administrator-only branch protection configuration is not performed by repository code changes.
