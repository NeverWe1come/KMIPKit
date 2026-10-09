# Feature Specification: Selective CI by Change Impact

**Feature Branch**: `feature/015-selective-ci`

**Created**: 2026-10-08

**Status**: Approved for implementation — 2026-10-09

**Input**: User-approved plan to retain a CI run for each pull-request update while running only the checks affected by the changed files, reporting proven skips in the GitHub run Summary, and falling back to full CI whenever impact is uncertain.

## Context and References

This is a change to contributor and pull-request verification only; no KMIP protocol behavior changes and no OASIS clauses apply. The current contract in `specs/001-cross-platform-ci/spec.md` requires core and coverage jobs across Linux, Windows, and macOS on every pull request (FR-001 and FR-009), prohibits changes to job routing (FR-014), and treats every skipped required job as a failure (FR-016). This specification proposes a selective-execution policy and, once approved, those conflicting requirements and their success criteria MUST be reconciled in the same change.

The implementation must preserve `AGENTS.md`, `.specify/memory/constitution.md`, `docs/development/testing.md`, `docs/development/rust-workspace.md`, `docs/development/git-and-releases.md`, and the existing CI security requirements. External GitHub Actions remain pinned to full commit SHAs. Pull-request workflows remain unprivileged, need no secrets, and do not gain write permissions. The scheduled workflow retains its current behavior.

GitHub job checks are tied to the commit under test; results from an earlier SHA are not reused as checks for a new SHA. The workflow itself therefore continues to run on every pull-request update. Job-level skips are acceptable only when the current run's impact plan proves that the job is not applicable. Relevant GitHub documentation: [required status checks](https://docs.github.com/en/pull-requests/how-tos/merge-and-close-pull-requests/troubleshooting-required-status-checks) and [job conditions](https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/control-jobs-with-conditions).

The current workflow groups Java, Python, and the C consumer in one three-OS job, and aggregates Rust and adapter coverage together. The implementation must separate those jobs and coverage scopes enough to route components independently; it must not weaken thresholds or treat absent required coverage as success.

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Get fast, trustworthy feedback on documentation changes (Priority: P1)

As a contributor changing only documentation, I want a CI run that checks documentation contracts and traceability without rebuilding and retesting every language and platform.

**Why this priority**: Documentation-only changes currently wait for the full matrix even though they cannot affect compiled components.

**Independent Test**: Classify a documentation-only two-tree diff, run its selected validators, and verify that only documented, genuinely unaffected jobs are skipped in the run Summary.

**Acceptance Scenarios**:

1. **Given** a pull request changes only recognized documentation paths, **When** CI runs, **Then** it executes documentation, contract, and traceability validation and the final Summary reports their outcomes.
2. **Given** documentation-only impact is proven, **When** unrelated component jobs are skipped, **Then** the Summary labels each as “not affected” with the classifier's reason and the overall result can pass only if every selected required job succeeds.
3. **Given** a documentation change also modifies code, **When** CI classifies the combined diff, **Then** it includes the documentation checks and every check selected for the code change.

---

### User Story 2 — Run checks for only the affected implementation component (Priority: P1)

As a maintainer, I want isolated Java, Python, C, or JNI changes to run their relevant tests across supported operating systems and their relevant coverage and safety checks, while leaving unrelated component jobs out of that run.

**Why this priority**: Language-specific edits should not wait for independent components, but must retain the platform and coverage guarantees for the changed component.

**Independent Test**: Exercise each component path class independently and verify its selected jobs, platforms, sanitizer scope, coverage scope, and explicit skipped-job reasons.

**Acceptance Scenarios**:

1. **Given** only Java implementation or Java test paths change, **When** CI runs, **Then** Java tests run on Linux, Windows, and macOS and Java coverage is collected and gated; unrelated Python, C-consumer, Rust-core, and Rust-coverage jobs are not run.
2. **Given** only Python binding or Python test paths change, **When** CI runs, **Then** Python tests run on Linux, Windows, and macOS and Python coverage is collected and gated; unrelated language and Rust-core jobs are not run.
3. **Given** only C consumer/interface test paths change, **When** CI runs, **Then** C consumer checks run on Linux, Windows, and macOS, applicable FFI sanitizer checks run, and C/FFI coverage is required.
4. **Given** JNI native or JNI test paths change, **When** CI runs, **Then** Java/JNI tests run on Linux, Windows, and macOS, applicable native sanitizer checks and JNI coverage run, and unrelated adapter suites are omitted.
5. **Given** more than one component is affected, **When** CI runs, **Then** it runs the union of all affected component checks and coverage gates.

---

### User Story 3 — Fall back safely when a change may affect shared behavior (Priority: P1)

As a maintainer, I want shared, high-risk, unknown, or unclassifiable changes to run full CI so that path-based optimization cannot hide a regression.

**Why this priority**: A false negative in impact classification is more harmful than spending extra CI time.

**Independent Test**: Feed shared, workflow, configuration, manifest, generator, normative-source, unknown-path, and classifier-failure cases into the classifier and verify that all pull-request checks are selected.

**Acceptance Scenarios**:

1. **Given** changes touch CI workflow/configuration, shared Rust or FFI implementation, workspace/package manifests or lockfiles, generators, normative catalog/source inputs, or other declared shared infrastructure, **When** CI classifies them, **Then** it selects full CI.
2. **Given** a rename or deletion crosses path categories, **When** CI classifies the diff, **Then** it includes both the old and new path impacts; a deletion uses its former path's impact.
3. **Given** a path does not match a reviewed category, base/merge data is unavailable, or classification fails, **When** CI runs, **Then** it selects full CI and explains the fallback in the Summary.
4. **Given** a mixed diff includes documentation and a full-CI trigger, **When** CI runs, **Then** full CI runs and the Summary names the triggering path class.

---

### User Story 4 — Understand which checks ran for the current commit (Priority: P1)

As a maintainer, I want the run Summary to distinguish passed checks from justified omissions and unexpected failures so that a green PR represents complete validation for its classified impact.

**Why this priority**: Selective execution is useful only if skipped work is auditable and cannot turn a missing or failed required result green.

**Independent Test**: Exercise valid and malformed impact plans alongside success, failure, cancellation, missing job results, and unexpected skips; verify the Summary text and exit status.

**Acceptance Scenarios**:

1. **Given** the impact classifier emits a valid plan, **When** the run Summary is rendered, **Then** it shows the plan, affected components, reason, each required job's result, and the jobs omitted as not affected.
2. **Given** a job selected by the plan fails, is cancelled, or has no result, **When** the Summary is written, **Then** the overall run fails and names the affected check and its diagnostic location.
3. **Given** a job is skipped although the plan selected it, or a plan is malformed/inconsistent, **When** the Summary is rendered, **Then** the run fails as an unexpected omission or invalid plan.
4. **Given** the current SHA has a green result for a job on an earlier run, **When** that job is selected for this SHA, **Then** CI executes it in the current run rather than reusing the earlier result.
5. **Given** the workflow runs on schedule, **When** it completes, **Then** existing scheduled jobs, conditions, and informational semantics remain unchanged.

## Edge Cases

- A rename has old and new paths in different impact classes, or a file is deleted.
- The pull-request base SHA, merge SHA, diff, or one side of the diff cannot be read.
- A changed file is in a new directory or has an extension not covered by the reviewed mapping.
- A PR changes documentation and one or more language components.
- A path changes CI logic, a reusable script, a tool version, a manifest, or a generator input that has broader effects than its directory suggests.
- A selected coverage producer succeeds but omits its report, emits an invalid report, or reports only a subset of the required scope.
- An unaffected job is skipped as planned; a selected job is skipped unexpectedly; a selected job is cancelled or returns no result.
- A required check fails before the Summary job can check out the repository or load the classifier output.
- A workflow edit changes the classifier or routing rules themselves.
- A scheduled run is unchanged by pull-request impact classification.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Every supported pull-request update MUST start the CI workflow for the current pull-request merge SHA. Workflow-level path filters MUST NOT be used to omit the run, and results from previous SHAs MUST NOT satisfy jobs in the current run.
- **FR-002**: Before component jobs are routed, a classifier MUST compare the exact pull-request base tree and merge tree for the current run. It MUST process additions, modifications, deletions, renames, and copies without losing either side's path; if exact diff information is unavailable or malformed, it MUST select full CI.
- **FR-003**: The classifier MUST output a machine-readable impact plan and a human-readable reason that identify the changed path classes, selected check groups, required coverage scopes, and whether full CI was selected. The classifier MUST be deterministic for the same two trees and mapping version.
- **FR-004**: The reviewed path map MUST distinguish at least documentation/contracts/traceability, Java, Python, C consumer/FFI, JNI, Rust/shared implementation, and CI/tooling/manifests/generator/normative inputs. Changes to multiple classes MUST combine their impact; comments in source files MUST be treated as changes to that source component.
- **FR-005**: Documentation-only changes MUST run lightweight documentation validation, relevant contract checks, normative requirement traceability checks, and the final Summary. The precise validators MUST be named in the approved implementation plan; they MUST validate the documentation and traceability inputs changed by the PR.
- **FR-006**: Isolated Java or Python changes MUST run that component's tests on Linux, Windows, and macOS and collect and gate its corresponding adapter coverage. Java and Python checks MUST be separable so an isolated change does not run the unrelated adapter suite.
- **FR-007**: Isolated C consumer/interface changes MUST run the C consumer checks on Linux, Windows, and macOS, applicable FFI sanitizers, and the corresponding FFI/C coverage scope. Isolated JNI changes MUST run JNI-relevant tests on Linux, Windows, and macOS, applicable native sanitizers, and the JNI coverage scope. These scopes MUST be independently selectable where their implementation is independent.
- **FR-008**: Any Rust core/protocol/model change or change to a shared implementation on which multiple components depend MUST run the current full Rust/platform validation and every dependent adapter check and coverage scope. Mixed changes MUST run the union of selected jobs; any full-CI trigger MUST dominate narrower selections.
- **FR-009**: Changes to `.github/workflows/**`, CI classifiers/routing/summary/coverage tooling, repository-wide scripts or tool configuration, workspace and package manifests/lockfiles, Rust toolchain configuration, generated-code generators or inputs, normative catalog/source/inventory, or an unrecognized path MUST select full CI. The approved path map MAY identify additional full-CI triggers where impact is shared or uncertain.
- **FR-010**: If classification fails, the plan is missing/invalid, or an input path cannot be classified, CI MUST fail closed to full CI when execution can continue. The final Summary MUST clearly state why full CI was selected or why a classifier error occurred.
- **FR-011**: Coverage production and gating MUST be independently scoped. Every selected coverage scope MUST produce the report/status required by its current threshold policy, and the gate MUST fail on a missing, malformed, or incomplete report. Reports for deliberately unselected scopes MUST NOT be required. No coverage threshold, source eligibility rule, or documented exclusion may be weakened.
- **FR-012**: Each component job MUST run only when selected by the current valid impact plan, except for jobs explicitly required for all PRs and the final Summary. An unselected job MUST have a deterministic “not affected” reason recorded in run-summary input. Job-level conditions MUST preserve a current-SHA check result without filtering out the entire workflow.
- **FR-013**: The final run Summary MUST validate actual job conclusions against the impact plan: all selected required jobs must succeed; failed, cancelled, missing, or unexpectedly skipped selected jobs must fail the Summary; only plan-authorized skips may be reported as not affected. It MUST show affected components, routing rationale, selected results, skipped reasons, and the existing run/commit context and failure diagnostics.
- **FR-014**: A CI configuration/classifier/routing change MUST select full CI, including the CI change's own pull request. Scheduled workflow behavior MUST remain unchanged and MUST NOT depend on a pull-request impact plan.
- **FR-015**: The change MUST preserve existing fork safety, read-only permissions, pinned external actions, self-hosted/hosted runner security routing, test commands, platform requirements for selected components, and all existing coverage thresholds. It MUST add no secrets, write permissions, or third-party workflow actions.
- **FR-016**: The approved implementation MUST include automated tests for documentation-only, each isolated component, mixed changes, rename across classes, deletion, unknown path, shared/full trigger, unavailable or malformed diff, classifier failure, selected-job failure/cancellation/missing result, unexpected skip, missing selected coverage, and successful plan-authorized skips.
- **FR-017**: Existing requirements in `specs/001-cross-platform-ci/spec.md` that require every component on every PR, prohibit routing changes, or reject every skipped job MUST be reconciled in the same change so the authoritative specifications do not conflict. Scheduled requirements and full-platform guarantees for components that do run MUST remain represented.

### Key Entities

- **Impact plan**: A deterministic classification of the current merge commit relative to its exact PR base, containing affected path classes, selected job groups, required coverage scopes, full-CI decision, reason, and classifier/schema version.
- **Check group result**: A current-run job conclusion associated with one selected group or a plan-authorized not-affected skip.
- **Coverage scope**: A component-specific coverage report and unchanged applicable threshold, such as Rust workspace, Java, Python, FFI/C, or JNI.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A documentation-only pull request still receives a current-SHA CI run and Summary, executes documentation/contract/traceability validation, and does not execute Rust build/test/coverage, adapter build/test/coverage, sanitizer, or fuzz work.
- **SC-002**: A Java-only or Python-only pull request runs the affected language tests on all three supported operating systems and its own coverage gate while unrelated component tests are omitted with a reason.
- **SC-003**: C-only and JNI-only changes run their required three-platform checks plus their own applicable sanitizer and coverage scopes; mixed changes run the union of all affected scopes.
- **SC-004**: Every unknown path, classifier error, invalid/missing diff input, CI configuration change, and shared/high-risk input selects full CI or yields a failing result that clearly explains the fallback condition; none can silently result in a narrow green plan.
- **SC-005**: The run Summary reports the impact decision and the actual result of every selected job; a failed, cancelled, missing, or unexpectedly skipped selected job cannot produce an overall PASS, and every omitted job has a plan-backed reason.
- **SC-006**: All existing coverage thresholds and applicable source/platform requirements continue to pass for selected scopes, while omitted scopes do not require artifacts that were not produced.
- **SC-007**: Schedule-triggered checks retain their current selected jobs and behavior, regardless of pull-request path routing.
- **SC-008**: For representative documentation-only and isolated-component PRs, the count of executed jobs is lower than the current full PR job set; after deployment, measured job counts and wall-clock duration are compared with available full-CI baseline runs, without treating shared-runner queue time as a deterministic guarantee.

## Assumptions

- The active integration branch is `release/1.0.0`; implementation work starts from its current remote tip on a dedicated `feature/015-selective-ci` branch.
- The workflow remains triggered for every supported pull-request update; no previous commit's green check is reused for a new SHA.
- The initial path map is conservative. An unreviewed or ambiguous path maps to full CI until a reviewed mapping is added.
- Rust implementation and shared build inputs receive full validation. Narrow Rust-core test selection is out of scope for this version.
- Java and Python tests remain required on Linux, Windows, and macOS when their respective component changes. C/JNI platform guarantees and the exact independently separable sanitizer/coverage jobs are confirmed during the implementation plan after reviewing existing build coupling.
- Existing coverage thresholds and unavailable-data semantics remain governed by `docs/development/testing.md` and current coverage contracts.
- The scheduled dependency-policy and informational branch-coverage workflow remains as currently specified.
- CI duration depends on GitHub scheduling, queue time, cache state, and workload; the deterministic optimization measure is fewer executed jobs for documentation-only and isolated-component changes, with elapsed-time comparison reported as observational evidence.
- Administrator-only required-check settings remain outside repository code changes.
