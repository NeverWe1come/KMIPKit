# Tasks: Dependency Policy Gates

**Input**: [spec.md](spec.md), [plan.md](plan.md), [data-model.md](data-model.md), and [contracts/dependency-policy.md](contracts/dependency-policy.md)

**Status**: Approved implementation tasks. T002 is evidenced on the active release branch.

## Phase 0: Baseline and implementation gate

- [x] T001 [US1] Record the exact active-release and PR-base SHAs; inventory the root/fuzz manifests and lockfiles; record core CI runner triples with direct versus inferred evidence clearly distinguished; and complete the independent cargo-deny 0.20.2 tool review in [dependency-review.md](dependency-review.md). Run the candidate check on both unfiltered target graphs in report-only mode and record every license expression, advisory, source, ban, wildcard, and duplicate finding without accepting any automatically.
- [x] T002 [US1] **IMPLEMENTATION GATE**: verify the approved/merged KMIPKIT-0011 specification revision is present on the current `release/1.0.0` base; record the exact approved revision and any required ADR disposition in `approval-record.md`; review the exact third-party license-ID inventory from T001; confirm no unaddressed ADR-0005 hard-ban finding exists; map every other baseline finding to an explicit in-scope remediation or a reviewed exception only if waiver-eligible; and confirm both workspaces and every core CI target are covered. The current fuzz wildcard and missing first-party license field are mapped to FR-013 and T006, so they MUST be covered by Red tests before the Green metadata changes and policy implementation. Any hard-ban or out-of-scope non-waivable finding keeps this gate blocked. Until T002 passes, do not change the workflow, deny configuration, manifests, lockfiles, or policy scripts.

## Phase 1: Red commits

- [x] T003 [P] [US1] Add failing Python `unittest` fixtures in `scripts/tests/test_dependency_policy.py` for root/fuzz manifest selection; missing and stale lockfiles; locked operation; full-feature metadata with no platform filter; an unfiltered cargo-deny graph that includes the lockfile's UEFI-only `r-efi` edge; runtime `rustc -vV` host-triple capture and rejection when the observed host is absent from the reviewed CI runner set; optional-feature-only and target-specific external path dependencies; canonical workspace-member paths; symlink escapes; online refresh for each workspace and its emitted RustSec commit SHA/timestamp; mismatch or unreadable database evidence; unchanged lockfiles; and FR-013's explicit `Apache-2.0` fuzz-package license and `kmipkit-ttlv` version requirement matching the local crate. Assert the candidate scan no longer reports those two fuzz metadata findings after Green. The test suite must fail behaviorally before the runner exists.
- [x] T004 [P] [US1] Add failing static workflow-contract tests in `scripts/tests/test_workflow.py` requiring the PR policy check for all `master` and `release/**` pull requests regardless of changed paths, read-only permissions, no secrets or privileged trigger, exact tool pin, both workspace invocations, and safe fork-runner routing.
- [x] T010 [US2] Add table-driven exception tests in `scripts/tests/test_dependency_policy.py` for exact `advisory`, `license`, `source`, and `duplicate` findings; prove a `ban` exception cannot waive ADR-0005's three architecture bans or a wildcard requirement; test missing rationale/mitigation/owner/reviewer/approval reference, missing human-reviewed evidence for a license clarification, future review date, expired date, expiry over 90 days, duplicate IDs, wildcard scope, orphaned entries, credentials in source URLs and secret-safe diagnostics, and unrelated packages. Explicitly prove that an exception for one version of a crate does not accept a different version of that same crate.
- [x] T013 [US3] Add failing workflow-contract tests in `scripts/tests/test_workflow.py` requiring the daily schedule to check out the configured active release ref and report the scanned commit plus each workspace's RustSec database commit SHA/timestamp. Assert the docs explain that scheduled workflows execute from the default branch and become active there only after integration.
- [x] T005 [US1] Run the focused policy and workflow tests after T003, T004, T010, and T013. Record exact behavioral RED failures (not harness or syntax failures) in `implementation-evidence.md`, then commit the test-only changes with DCO sign-off.

## Phase 2: Green commits

- [x] T006 [US1] In the Green commit, update only the existing unpublished fuzz manifest metadata required by FR-013: declare `license = "Apache-2.0"` and add a `version` requirement to its existing `kmipkit-ttlv` path dependency that exactly matches `crates/kmipkit-ttlv/Cargo.toml`; verify both lockfiles and resolved package versions remain unchanged. Add `.cargo/deny.toml` for all-feature/target graphs and development licenses/duplicates; explicitly set advisory `unsound = "all"`, `unmaintained = "all"`, `yanked = "deny"`, source `unknown-registry = "deny"`, `unknown-git = "deny"`, `required-git-spec = "rev"`, bans `multiple-versions = "deny"`, `multiple-versions-include-dev = true`, `wildcards = "deny"`, and `allow-wildcard-paths = false`; explicitly allow the reviewed crates.io registry and ban exactly `native-tls`, `openssl`, and `openssl-sys` under ADR-0005, with a comment citing the ADR. Do not add pattern-based bans or exception entries for these packages. Include only baseline-reviewed exact exceptions from T001; keep the initial exception register empty if none are needed.
- [x] T007 [US1] Add `specification/compliance/dependency-policy-exceptions.json` with a versioned schema and no default waivers, plus `scripts/dependency_policy.py` to validate exact package/rule matching, field completeness, human-reviewed license evidence, owner/reviewer separation, approval evidence, dates, 90-day maximum expiry, bidirectional exception correspondence, rejection of `ban` exceptions for ADR-0005 packages, and every source-less Cargo metadata package against the canonical union of root/fuzz workspace member manifests beneath the canonical checkout root. Reject out-of-repository paths and symlink escapes unconditionally.
- [x] T008 [US1] Add `scripts/Test-DependencyPolicy.ps1` to install/use exact reviewed cargo-deny 0.20.2 with `cargo install --locked --version 0.20.2 cargo-deny`, verify the installed version, capture `rustc -vV`, and fail if the executing runner's host triple is absent from the reviewed target set. Run `cargo metadata --locked --format-version 1 --all-features` for both manifests without `--filter-platform` before path/exception validation. Run cargo-deny online against root and fuzz separately with `--workspace --all-features --locked`; do not pass `--offline` or `--frozen`, modify lockfiles, auto-fix, or suppress failures. Give the job an isolated temporary `CARGO_HOME`; after each successful workspace check verify the RustSec remote and emit that invocation's database commit SHA and ISO timestamp.
- [x] T009 [US1] Add a read-only policy job to `.github/workflows/ci.yml` for every pull request regardless of changed paths; use hosted Linux for fork PRs and the established least-privilege Linux ARM64 runner for same-repository PRs; leave cargo-deny's target graph unfiltered so it checks every target-specific edge (including all supported CI triples), validate the executing host triple, and include dev-only duplicates; refresh advisory data online; and report each workspace separately without secrets or write permissions.
- [x] T011 [US2] Ensure `.cargo/deny.toml` exception syntax and the JSON exception register are synchronized and checked in both directions; produce rule-specific diagnostics naming the exception ID without exposing secret values.
- [x] T012 [US2] Add `docs/security/dependency-policy.md` explaining exception review, evidence, mitigations, expiry, renewal, removal, the finite package-ban rationale, and cargo-deny's license metadata limits; link to the exception register and independent tool review.
- [x] T014 [US3] Update `docs/development/testing.md`, `docs/development/rust-workspace.md`, and `docs/security/dependency-policy.md` with the one local command, exact tool pin, root/fuzz coverage, online advisory behavior, per-workspace RustSec evidence, exception process, failure semantics, and automated-license-evidence limitation.
- [x] T015 [US3] Add `specification/compliance/requirements/KMIPKIT-0011.csv` mapping every FR/SC to configuration, script/workflow location, and executable test; wire the policy job into the existing daily schedule against the configured active release ref without write permission.

## Phase 3: Refactor, convergence, and final evidence

- [x] T016 [US1] Make one separate Refactor commit after Green: reduce duplication in validator/runner logic without changing behavior; rerun focused tests and retain distinct Red, Green, and Refactor commit evidence.
- [ ] T019 [US3] Run `/speckit-converge` after implementation and Refactor. Add any remaining buildable gaps to this task list, implement each with Red/Green/Refactor commits, and repeat convergence until no buildable gap remains.
- [ ] T017 [US1] After convergence is clear, run the full dependency policy for root/fuzz and all supported targets; verify FR-013 produces no wildcard or missing-license finding and no lockfile or resolved-version change; run Python/PowerShell checks, formatting, Clippy, workspace tests/docs, generators, coverage, and Linux/Windows/macOS CI. Record commands/results, lockfile immutability, and environment limitations.
- [ ] T018 [US2] Obtain independent QA and security reviews of the final spec and implementation, including exception validation, policy configuration, tool review, workflow permissions/fork routing, and diagnostic output. Resolve all blocking/high findings. If a review or convergence adds work, repeat TDD, convergence, full affected verification, QA, and security review before proceeding. Reviewer-owned checklists must be completed by the reviewers.
- [ ] T020 [US3] Update from the active release branch without force-pushing a shared branch, rerun affected checks, prepare a detailed terminal-created draft PR with exact Red/Green/Refactor commits, traceability, tool/dependency review, security effects, actual CI evidence, risks, and limitations, and leave it unapproved/unmerged.

## Dependencies and execution order

```text
T001 -> T002 -> (T003-T004, T010, T013 Red) -> T005
     -> (T006-T009, T011-T012, T014-T015 Green)
     -> T016 Refactor -> T019 converge loop -> T017 verification
     -> T018 independent QA/security loop -> T020 draft PR
```

The root and fuzz graphs are separate policy inputs. Metadata validation covers all features and target platforms. Each cargo-deny invocation refreshes advisory data online and records its own RustSec revision. The scheduled run follows the checked-in active-release reference and can execute only after the workflow reaches the default branch. GitHub branch protection, release artifacts, SBOM, provenance, and signatures remain outside this feature.

## Requirement-to-task coverage

| Requirement | Tasks |
|---|---|
| FR-001 | T001, T003, T006, T008, T009, T017 |
| FR-002 | T003, T008, T009, T017 |
| FR-003 | T001, T003, T006, T007, T010, T017 |
| FR-004 | T001, T003, T008, T009, T013, T015, T017 |
| FR-005 | T003, T007, T008, T017 |
| FR-006 | T002, T003, T006, T010, T017 |
| FR-007 | T010, T011, T012, T018 |
| FR-008 | T001, T006, T008, T014, T017 |
| FR-009 | T004, T009, T017 |
| FR-010 | T013, T015, T017 |
| FR-011 | T012, T014 |
| FR-012 | T003, T007, T011, T017, T018 |
| FR-013 | T001, T002, T003, T005, T006, T017 |
| SC-001 | T004, T009, T017 |
| SC-002 | T003, T005, T010, T017 |
| SC-003 | T005, T006, T017 |
| SC-004 | T013, T015, T017 |
| SC-005 | T010, T011, T018 |
| SC-006 | T012, T014, T017 |
| SC-007 | T004, T009, T017 |
| SC-008 | T003, T005, T006, T017 |

T002 is the common start gate; T016 applies a behavior-preserving refactor to
implemented requirements; T019 converges any remaining FR/SC gap; and T020
packages the complete evidence for review. T015 generates the final
machine-readable compliance traceability artifact from this planning map.
