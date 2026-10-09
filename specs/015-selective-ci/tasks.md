# Tasks: Selective CI by Change Impact

**Input**: Approved `spec.md` and design artifacts in this directory.
**Execution**: One implementer; sequential Red, Green, Refactor commits with DCO sign-off. No parallel tasks.

## Requirement Traceability

| Requirements | Tasks |
|---|---|
| FR-001–FR-004, FR-009–FR-010; SC-004 | T004–T007, T012, T016 |
| FR-005; SC-001 | T003, T009, T016 |
| FR-006–FR-008; SC-002–SC-003 | T005–T007, T010–T013, T016 |
| FR-011; SC-006 | T006–T007, T011, T014, T016 |
| FR-012–FR-014; SC-005, SC-007 | T005, T008–T010, T012, T016 |
| FR-015 | T004, T012, T015 |
| FR-016 | T004–T008, T015 |
| FR-017 | T002, T014–T016 |
| SC-008 | T016 |

## Phase 1: Design and setup

- [ ] T001 Confirm current `release/1.0.0` base SHA, active job graph, coverage scopes, security invariants, and approved path classes; record any mapping uncertainty as full-CI.
- [ ] T002 Reconcile the conflicting per-PR execution, coverage, routing, and skip requirements in `specs/001-cross-platform-ci/spec.md`; retain schedule requirements and preserve all threshold numbers.
- [ ] T003 Identify executable documentation/traceability contracts that run without builds; add a dedicated lightweight validation command only if existing tests cannot satisfy FR-005.

## Phase 2: RED — expected routing and gate behavior

- [ ] T004 Add `scripts/tests/test_ci_impact.py` tests using temporary Git repositories for each reviewed path class, mixed paths, rename/copy sides, deletions, NUL paths with spaces/newlines, exact base/merge inputs, unknown paths, invalid SHAs, missing objects, malformed output, and full fallback.
- [ ] T005 Extend `scripts/tests/test_workflow.py` to require one always-triggered classifier job, split language jobs, three-platform checks for selected adapters, isolated Linux sanitizer/coverage producers, schedule independence, full fallback for CI/config paths, and no workflow-level path filters or new unpinned actions.
- [ ] T006 Extend `scripts/tests/test_coverage_gate.py` and `test_multilanguage_coverage.py` to validate each selected scope, preserve its current threshold, reject missing/malformed selected reports, and not require artifacts from unselected scopes; include Rust/FFI full-scope and partial-adapter fixtures.
- [ ] T007 Extend `scripts/tests/test_ci_summary.py` for valid/malformed/mismatched plans, every authorized skip, selected skip/failure/cancellation/missing results, all-component plans, and schedule results with no PR classifier.
- [x] T008 Run focused new tests and confirm they fail for missing behavior rather than test syntax/import errors. RED: `python -B -m unittest scripts.tests.test_ci_impact scripts.tests.test_ci_selective_workflow scripts.tests.test_ci_summary scripts.tests.test_multilanguage_coverage -v` ran 35 tests and exposed 68 assertion/subtest failures plus 1 error against the baseline (missing classifier, split workflow jobs, selected-scope coverage, and plan-aware Summary). Commit the tests separately with `git commit -s`.

## Phase 3: GREEN — classifier and selected validators

- [ ] T009 Implement the pure path mapping, exact NUL-delimited two-tree diff parser, plan schema, SHA validation, deterministic serialization, and safe full-CI fallback in `scripts/ci_impact.py`.
- [ ] T010 Split `language-bindings` into independent C, Java/JNI, and Python matrix jobs; split C and JNI sanitizer checks; preserve OS/toolchain/test commands and all pinned actions.
- [ ] T011 Split adapter coverage producers by Java, Python, and JNI; expose only selected artifacts and retain required full Rust/FFI producers for full and C/FFI modes.
- [ ] T012 Add impact-plan-driven job conditions and plan metadata to the existing workflow; keep the workflow trigger unchanged and keep schedule-only jobs independent.
- [ ] T013 Extend `scripts/coverage_gate.py` with validated explicit coverage scopes and changed-source filtering. Full mode must retain the existing complete artifact set and every current gate.
- [ ] T014 Make `scripts/ci_summary.py` validate the plan and actual `needs` job results; accept only authorized not-affected skips, describe reasons/results, and preserve schedule-specific summaries.
- [ ] T015 Reconcile spec 001 and update `docs/development/testing.md` and Spec Kit task/traceability artifacts with the final routing contract and threshold preservation.
- [ ] T016 Run focused RED tests to GREEN; then the full Python script suite, normative catalog checks/generators, PowerShell contracts, workflow contracts, Rust fmt/Clippy/test/doc, and available full coverage validation. Record output and commit the minimal working implementation with `git commit -s` separately.

## Phase 4: REFACTOR and review

- [ ] T017 Refactor classifier, coverage scope selection, and Summary policy for readability without changing approved output or routing; retain full regression coverage.
- [ ] T018 Re-run focused and full available verification, workflow/security review, coverage fail-closed checks, docs traceability, and `git diff --check`; record REFACTOR evidence and commit separately with `git commit -s`.
- [ ] T019 Rebase on the latest remote `release/1.0.0`, confirm it is the merge base, verify the CI configuration change selects full CI, push the feature branch, and prepare a draft PR with Red/Green/Refactor evidence and known limitations. Never merge.

## Required Scenario Matrix

| Scenario | Expected plan/result |
|---|---|
| Markdown-only docs | docs contracts + Summary; all build/test/coverage groups authorized skipped |
| Java-only | Java 3-OS tests + Java coverage/gate |
| Python-only | Python 3-OS tests + Python coverage/gate |
| C consumer-only | C 3-OS tests + FFI/C sanitizer and coverage gate |
| JNI-only | Java/JNI 3-OS tests + JNI sanitizer and coverage gate |
| Mixed component diff | Union of selected checks; shared/full trigger dominates |
| Rename/copy/delete | Classify all affected old/new paths, never drop old-path impact |
| Unknown/shared/workflow/tooling/config path | Full suite and every coverage scope |
| Classifier/diff/input failure | Full fallback if possible and Summary failure when the classifier failed |
| Selected producer missing report or job result | Coverage gate/Summary fails with scope/job name |
| Unexpected skip | Summary fails; plan-authorized skip only is accepted |
| Schedule | Existing scheduled jobs and informational branch behavior only |
