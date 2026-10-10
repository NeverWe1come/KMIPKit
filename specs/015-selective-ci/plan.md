# Implementation Plan: Selective CI by Change Impact

**Branch**: `feature/015-selective-ci` | **Date**: 2026-10-09 | **Spec**: [spec.md](spec.md)

## Summary

Add an always-triggered pull-request classifier that compares the exact PR base and merge trees, emits a deterministic impact plan, and controls job-level routing. Keep full Rust/shared CI as the conservative fallback. Split the current combined adapter test/coverage jobs so Java, Python, C, and JNI work can be selected independently. Make the coverage gate validate only selected scopes without relaxing any selected scope's existing threshold. Teach the final Summary to reject selected failures and unexpected skips while explaining authorized skips. Reconcile the superseded per-PR execution rules in specification 001.

## Technical Context

**Language/Version**: Python 3 standard library for classification, plan validation, and tests; GitHub Actions YAML; existing Rust 1.94/stable, Java 17, Python 3.12, C/C++, and PowerShell jobs.
**Primary Dependencies**: Git CLI with full PR base and merge objects; existing GitHub Actions pinned by full SHA; no new action or third-party runtime dependency.
**Storage**: A small JSON plan passed as a classifier-job output and retained as a workflow artifact only if the output-size/diagnostic contract requires it. No persistent data.
**Testing**: `unittest` tests with temporary Git repositories for exact-tree/path-status classification; existing workflow and Summary contracts; coverage gate fixture tests; YAML parsing/static workflow contracts; full existing script/catalog checks and Rust checks for full-CI selection.
**Target Platform**: GitHub-hosted Linux, Windows, and macOS PR jobs; current scheduled runner configuration remains unchanged.
**Project Type**: Rust workspace with multi-language bindings and repository-owned GitHub Actions automation.
**Performance Goals**: Documentation-only changes do not execute compiler, language build/test, sanitizer, fuzz, or coverage jobs. Isolated Java/Python/C/JNI changes omit unrelated components. Wall-clock savings are observational; job-count reduction is deterministic.
**Constraints**: No workflow-level path filters; no prior-SHA result reuse; exact two-tree diff; unknown/ambiguous inputs force full CI; selected coverage reports remain required; unchanged read-only permissions, fork safety, pinned actions, and schedules.

## Constitution Check

| Principle | Check |
|---|---|
| Approved scope and traceability | Pass. Approved spec 015 covers CI behavior only; no KMIP/OASIS normative clauses apply. Requirements are mapped to tests and workflow contracts in `tasks.md`. Existing spec 001 is reconciled in the same PR. |
| Test first | Pass. Classifier, coverage-scope, workflow-routing, and Summary regressions are committed RED before production changes; GREEN and Refactor are separate DCO-signed commits. |
| Security and fork handling | Pass. Continue using `pull_request`, read-only permissions, no secrets, no persisted checkout credentials, no new actions. Classifier treats repository path names as data and invokes Git with argument arrays (no shell interpolation). |
| Coverage policy | Pass. Existing thresholds and scope checks remain unchanged; only report requirements for plan-proven unselected scopes are omitted. Missing selected reports fail closed. |
| Human governance | Pass. Work remains in the feature worktree and draft PR. No release/master pushes, merge, protection, or secret changes. |

## Impact Classification Contract

The classifier consumes a validated 40- or 64-hex base SHA and merge SHA and runs `git diff --name-status -z --find-renames --find-copies <base> <merge>`. It parses NUL-delimited records, retains both old and new names for rename/copy statuses, classifies the previous name for deletion, rejects malformed/truncated records, and never reads a working-tree diff.

Path classes are explicit and conservative:

| Path class | Initial path scope | Selected behavior |
|---|---|---|
| Documentation | Root Markdown files; `docs/**/*.md`; `specs/**/*.md`; `changelog.d/**/*.md` | Documentation/traceability validators and Summary. A mixed diff adds the other class behavior. |
| Java | `bindings/java/src/**` and `bindings/java/examples/**`, excluding JNI-native paths | Java integration tests on Linux, Windows, and macOS; Java JaCoCo coverage. |
| Python | `bindings/python/src/**`, `bindings/python/tests/**`, and `bindings/python/examples/**` | Python build/tests/examples on Linux, Windows, and macOS; Python coverage. |
| C consumer | `bindings/c/include/**`, `bindings/c/tests/**`, and `bindings/c/examples/**` | C consumer checks on Linux, Windows, and macOS; Linux C-consumer FFI sanitizer and FFI/C coverage inputs. |
| JNI | `bindings/java/native/**` | Java/JNI integration tests on Linux, Windows, and macOS; Linux native sanitizer and JNI coverage. Java adapter coverage is also selected if Java sources are changed. |
| Full/shared | CI/workflow files, any scripts or tool configuration, any Cargo/Maven/Python/CMake manifest or dependency lock, Rust workspace/source/test files, generated-source generators or inputs, normative catalog/source/inventory, and any unrecognized path | Full existing PR validation and every coverage scope. |

Path classes are combined across all old/new diff names. Component source/test files are the only narrow implementation paths; component manifests/build configuration remain full-CI triggers. The feature PR changes the workflow and scripts, therefore it must classify as full CI.

## Job and Coverage Design

- Add `impact-plan` on pull requests only. Its output contains schema version, base/merge SHAs, classes, selected job IDs, selected coverage scopes, `full`, and stable human-readable reasons. A safe, narrowly bounded output is exposed through job outputs; if GitHub's output limit would be exceeded, publish a compact plan and artifact path. Invalid or absent classifier output causes downstream PR checks to select full CI and the Summary to fail for the classifier error.
- Preserve current `core`, script-contract, normative, fuzz, dependency-policy, and Rust coverage semantics for full selection. Route each based on `full` and its component map.
- Split `language-bindings` into `language-c`, `language-java`, and `language-python` jobs, each retaining the current supported OS matrix and only its relevant toolchains/steps. JNI native integration stays with Java's JNI bridge build/tests. Split the Linux sanitizer steps into separately routable C FFI and JNI sanitizer jobs.
- Split `adapter-coverage` into Linux-only Java, Python, and JNI producers. Keep the three-platform Rust `coverage` matrix for full/shared selection and for the relevant C/FFI scope. Upload unchanged named, scope-specific report artifacts.
- Extend `coverage_gate.py aggregate` with an explicit required-scope set. Full selection uses all current reports and all current Rust, workspace, crate, adapter, and changed-code thresholds. A Java/Python/JNI-only selection validates its adapter report and applicable 85% adapter and 95% changed-source gates without demanding Rust or unrelated adapter reports. C-only selects Rust/FFI reports and their currently applicable Rust/FFI gates. An empty coverage set skips the gate as not applicable. Any selected report missing, malformed, incomplete, or for the wrong sources fails.
- Make `run-summary` depend on every possible PR/schedule job and always run. It validates each actual job result against the plan's selected job set; only a skipped unselected job is accepted. A selected job that is skipped, missing, failed, or cancelled fails the Summary. Show classifier/full-fallback details and per-job skip reasons. On schedule, the classifier is not applicable and the current schedule policy remains authoritative.

## Project Structure

```text
.github/workflows/ci.yml
scripts/ci_impact.py
scripts/ci_summary.py
scripts/coverage_gate.py
scripts/tests/test_ci_impact.py
scripts/tests/test_ci_summary.py
scripts/tests/test_coverage_gate.py
scripts/tests/test_multilanguage_coverage.py
scripts/tests/test_workflow.py
docs/development/testing.md
specs/001-cross-platform-ci/spec.md
specs/015-selective-ci/{spec,plan,research,data-model,quickstart,tasks}.md
```

## Implementation Phases

1. Record the approved spec, finalized routing/design decisions, and FR/SC task map. Recheck GitHub base SHA and the workflow's current job graph.
2. RED: Add failing classifier tests for path classes, combinations, NUL parsing, renames/deletes/copies, invalid SHAs/diffs, and conservative full fallback; add failing coverage-scope, dynamic Summary, and workflow contracts. Run focused suites and record precise results.
3. GREEN: Implement the impact classifier, scope-aware coverage gate, component job split/routing, Summary validation, and specification-001 reconciliation. Keep each change minimal and prove focused suites pass.
4. REFACTOR: Simplify classification/summary/workflow contracts without changing behavior; retain all regressions and prove focused tests remain green.
5. Full verification: Run all Python script tests, normative catalog tests and generation checks, PowerShell contracts, Rust format/lint/test/doc and available coverage checks; validate workflow structure/action SHA pins, path mapping, docs, and `git diff --check`.
6. Rebase/update from `release/1.0.0`, inspect diff and job graph, push the feature branch, create a draft PR, and require the PR's CI to take the full-CI branch before asking for review. Do not merge.

## Risks and Mitigations

- **Missed path impact**: Unknown and shared paths choose full CI; classifier tests include unrecognized names, renames, deletes, malformed input, and missing objects.
- **False green from skipped jobs**: Summary derives the selected job set from validated plan data and compares it to `needs`; selected skips fail.
- **Coverage scope accidentally weakened**: Scope tests require exact source/report sets and preserve 85/90/95% thresholds; missing selected reports fail.
- **Oversized GitHub job outputs**: Keep the machine-readable plan compact; validate before exposing it; fail full/fail if serialization or output transport fails.
- **Manifest/build changes treated too narrowly**: Component manifests, lockfiles, tool configuration, build scripts, and all repository scripts map to full CI.
- **Scheduled regression**: Preserve schedule-only job conditions and add schedule contracts proving the PR classifier does not gate or alter scheduled checks.

## Complexity Tracking

No constitution violations. This change adds one first-party classifier and splits existing jobs; it adds no external workflow action, credential, permission, or application dependency.
