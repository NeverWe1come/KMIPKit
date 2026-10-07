# Tasks: KMIP 2.1 Client Profile Conformance

**Input**: Design documents from `specs/010-profile-conformance/`

**Prerequisites**: `spec.md`, `plan.md`, `research.md`, `data-model.md`, and both contracts.

**Testing**: Strict Red, Green, Refactor TDD is required. Test tasks are explicit. Keep Red, Green, and Refactor evidence in separate signed commits; each task description names its intended file paths.

**Organization**: Tasks are grouped by foundation and user story. Work sequentially because the generated catalog records, Rust validator, and evidence report share one source of truth.

## Phase 1: Setup

- [ ] T001 Review every client profile and linked direct or inherited clause, requirement, dependency, test ID, and fixture status in `specification/catalog/kmip-2.1.json`; record the complete transitive normative-closure baseline and its source evidence in `specs/010-profile-conformance/research.md`, including the full client-applicable KMIP Specification closure inherited by Baseline Client.
- [ ] T002 Confirm the exact normative sections and requirement mappings for all 18 client profiles, including every `client_1_0`/`conditional` profile, against the pinned Specification and Profiles files; recursively include clauses normatively incorporated by reference and their applicable subclauses, distinguish informative references, and verify linked test IDs against the pinned Test Cases source. Explicitly verify Baseline Client's Profiles §6.1 item 1 -> the entire KMIP Specification v2.1 source (all client-applicable 1.0 requirements) and item 2 -> Profiles §3.1 (including applicable §§3.1.1 and 3.1.2 records); preserve server-only/out-of-scope dispositions and do not modify any OASIS source.
- [ ] T003 Add failing catalog and generator tests in `tools/normative_catalog/tests/test_validate.py` and `tools/normative_catalog/tests/test_generate_profiles.py` for all 18 client records, the 15 `client_1_0`/`conditional` profiles, exact source IDs, dependencies, and complete direct/inherited clause and requirement closure. Check every client's direct clauses, inclusion edges, full clause closure, and requirement IDs against an independently reviewed pinned-source oracle at `tools/normative_catalog/tests/fixtures/profile-closure-v2.1.json`; do not generate this oracle from the catalog. The Baseline oracle MUST cover every KMIP Specification v2.1 clause and every client-applicable `KMIPKIT-REQ-SPEC-*` requirement in the approved 1.0 scope through §6.1 item 1, plus Profile §3.1 inherited requirements; preserve server-only/out-of-scope dispositions and existing requirement ownership. Validate `normative_inclusions` with `inclusion_id`, `source_clause_id`, `included_source_id`, and nullable `included_section` (null means the whole source); test stable unique IDs, source/section resolution, duplicate/cycle rejection, whole-document and section closure, exact expansion through same-source subclauses, and exclusion of informative references. Include regression cases proving omission of any oracle-listed clause/requirement fails with stable IDs, explicitly covering `KMIPKIT-REQ-PROF-3.1.1-002`, `-004`, `-005`, `KMIPKIT-REQ-PROF-3.1.2-002`, and `-005`. Also test independent `profile-targets.json` selection, the 16 unavailable Baseline fixtures, and the invariant that generated prose metadata is never an executable rule mapping. Add negative target-manifest cases for input over 64 KiB, invalid UTF-8/JSON, duplicate JSON keys, missing/unknown fields, wrong JSON types, an empty profile list, empty/duplicate IDs, unresolved/out-of-scope IDs, a wrong release line, an empty/unresolved decision reference, and symlink/reparse-point paths; invalid and over-limit source catalogs; and adversarial Rust string values (quotes, backslashes, line breaks, tabs, control characters, and Unicode), with generated-source compilation and literal round-trip assertions.
- [ ] T004 Commit T003 as the signed Red commit and record the failing command and result in `specs/010-profile-conformance/implementation-evidence.md`.

## Phase 2: Foundational profile metadata

- [ ] T005 Extend `specification/catalog/kmip-2.1.json` and `specification/catalog/README.md` with the `normative_inclusions` schema, per-profile `direct_source_clause_ids`, and reviewed computed `source_clause_ids`/`requirement_ids` for all client profiles, matching the independent closure oracle; Baseline includes edges from `KMIPKIT-CLAUSE-PROF-6.1-001` to all of `KMIPKIT-SRC-spec` and from `KMIPKIT-CLAUSE-PROF-6.1-002` to Profiles §3.1. Then add `specification/catalog/profile-targets.json` selecting Baseline Client for release line 1.0. Implement the offline profile metadata generator in `tools/normative_catalog/generate_profiles.py` using the normative catalog for source facts and the target file for project target generation/reporting. Compute closure from direct source clauses, whole-source/section `normative_inclusions`, and same-source dot-delimited subclauses; require stored clause and requirement sets to match exactly, without parsing OASIS prose at runtime or treating informative references as normative. Load the catalog through repository safe I/O and `load_validated_catalog`; load the fixed target path through `safe_io.py` with a 64 KiB cap; strictly validate UTF-8, JSON without duplicate keys, the exact schema, unique IDs, catalog resolution, and approved client scope; reject symlink/reparse points before output. Do not use target membership for runtime selection.
- [ ] T006 Generate immutable Rust client-profile records in `crates/kmipkit-protocol/src/profiles_generated.rs`, preserving direct source-clause IDs, the computed direct/inherited clause and requirement closure, dependencies, transport/encoding conditions, test IDs, applicability, fixture status, and the separate selected-target flag.
- [ ] T007 Run generator tests and commit the minimal passing generator/output as the signed Green commit; record the exact command and result in `specs/010-profile-conformance/implementation-evidence.md`.
- [ ] T008 Refactor the generator and generated records for deterministic ordering, bounded input, a dedicated Rust string-literal encoder, and no duplicated source of truth; retain T003's adversarial compile/round-trip tests and record their passing result in the signed Refactor commit evidence at `specs/010-profile-conformance/implementation-evidence.md`.

## Phase 3: User Story 1 - Select and apply a client profile (Priority: P1)

- [ ] T009 Add failing profile-selection and dependency tests in `crates/kmipkit-protocol/tests/profile_validation.rs` for successful selection with complete typed mappings/tests, an in-scope profile not listed in the target manifest, unknown IDs, non-client roles, conditional profiles, dependency closure, cycles, conflicting constraints, and fail-closed rejection when any applicable requirement in the complete normative clause/subclause closure lacks a typed rule or passing executable test. Include Baseline regressions for oracle-listed KMIP Specification client requirements and each inherited §3.1.1/§3.1.2 requirement listed in T003.
- [ ] T010 Add failing client preflight tests in `crates/kmipkit-client/tests/unit/profile_preflight_tests.rs` proving an explicit message/transport violation or incomplete rule mapping causes zero fake-transport writes, while an extra base-protocol-valid operation remains allowed.
- [ ] T011 Commit T009-T010 as the signed Red commit and record exact failing commands/results in `specs/010-profile-conformance/implementation-evidence.md`.
- [ ] T012 Implement typed profile identities, requirement-keyed rule mappings, mapping/test completeness checks, validation results, and redacted profile errors in `crates/kmipkit-protocol/src/profile.rs` using `profiles_generated.rs`; never interpret free-form catalog descriptions as executable rules.
- [ ] T013 Integrate the per-exchange `Client::execute_with_profiles(request, selection)` API and explicit profile preflight into `crates/kmipkit-client/src/execute.rs`, and update the fake-client constructor in `crates/kmipkit-client/tests/unit/execute_test_support.rs`; keep the existing `execute` path unprofiled, do not add a production transport, constructor, or implicit client-wide selection state, and preserve request delivery-state semantics for response validation errors.
- [ ] T014 Run the focused Rust tests and commit the minimal passing implementation as the signed Green commit; record evidence in `specs/010-profile-conformance/implementation-evidence.md`.
- [ ] T015 Refactor profile condition evaluation and client preflight so selection order cannot resolve conflicts and explicit caller/cryptographic choices remain unchanged; commit as the signed Refactor commit and record evidence.

## Phase 4: User Story 2 - Inspect profile evidence and gaps (Priority: P1)

- [ ] T016 Add failing evidence-report tests in `tools/normative_catalog/tests/test_report.py` for complete direct/inherited clause and requirement closure, exact clause/requirement/test links, mandatory/optional status, dependency profiles, fixture availability, deterministic ordering, unresolved assignments, and redaction: feed secret-marker credential/private-key values and a raw KMIP-body marker through malformed evidence/diagnostic inputs and assert neither markers nor raw payload text appears in reports or errors.
- [ ] T017 Commit T016 as the signed Red commit and record the failing report command/result in `specs/010-profile-conformance/implementation-evidence.md`.
- [ ] T018 Extend `tools/normative_catalog/report.py` to render every catalogued client profile, its complete direct/inherited clause and requirement closure, independent target selection, applicability, requirement coverage, typed rule and rule-test coverage, official tests, fixtures, dependencies, deviations, catalog evidence blockers, and the separate unpublished release-claim state. Keep this committed report based only on stable checked-in inputs; current-run CI evaluation is implemented separately at T026.
- [ ] T019 Assign `KMIPKIT-0010` in `specification/catalog/kmip-2.1.json` to every profile-specific requirement and rule mapping owned by this feature after the complete closure is generated; keep base protocol/operation requirements assigned to their owning specifications and preserve every inherited requirement's exact profile, clause, condition, and source links.
- [ ] T020 Regenerate `specification/catalog/coverage-report.md` with the pinned repository tool; do not edit generated report lines manually.
- [ ] T021 Run focused report tests and commit the minimal passing implementation as the signed Green commit; record evidence in `specs/010-profile-conformance/implementation-evidence.md`.
- [ ] T022 Refactor report generation for deterministic output and clear separation of source facts, project evidence, readiness state, and public claim; commit as the signed Refactor commit and record evidence.

## Phase 5: User Story 3 - Determine claim eligibility without making a claim (Priority: P1)

- [ ] T023 Add failing catalog and readiness tests in `tools/normative_catalog/tests/test_validate.py` and `tools/normative_catalog/tests/test_report.py` for incomplete `normative_inclusions`, direct/inherited clause closure, missing normative links, unavailable required fixtures, missing typed rule mappings or rule tests, failed/missing mandatory tests, accepted SHOULD deviations, conditional profiles, dependency failures, stale/contradictory catalog `claim_state` reconciliation, and evidence-complete versus public-claim separation. Add manifest construction and invalid-context cases in `scripts/tests/test_ci_summary.py` for `pull_request`/`schedule`/other events, unprotected refs, wrong refs, repository/commit/workflow/run/attempt, each missing or failed required job ID including `coverage-gate`, duplicate verification IDs, malformed `needs` input, caller-supplied or stale results, missing/unreadable/malformed branch-policy API response, insufficient review/admin/force-push settings, absent or malformed `bypass_pull_request_allowances`, and each non-empty `users`, `teams`, or `apps` bypass list. Include a positive case proving a complete current policy with all three bypass lists present and empty is eligible. Verify run-specific data never changes committed deterministic generated output.
- [ ] T024 Add failing Rust readiness tests in `crates/kmipkit-protocol/tests/profile_evidence.rs` proving one missing/failing required record prevents `evidence_complete`, no readiness result publishes a claim, and Query/Discover Versions responses cannot infer or mutate profile status.
- [ ] T025 Commit T023-T024 as the signed Red commit and record exact failing commands/results in `specs/010-profile-conformance/implementation-evidence.md`.
- [ ] T026 Update `tools/normative_catalog/validate.py` to reject `evidence_complete` when the transitive normative clause/requirement closure, catalog-linked tests, or required fixtures are missing/unavailable; implement full readiness aggregation in `tools/normative_catalog/report.py` and profile eligibility in `crates/kmipkit-protocol/src/profile.rs`. Extend `.github/workflows/ci.yml` to run the required core, script-contract, inventory, coverage, and dependency-policy jobs for `push` to `release/1.0.0` as well as existing pull requests, retaining schedule-only jobs as ineligible; use the push event's `before` SHA for changed-code coverage boundaries. The final aggregate job MUST require the exact successful job IDs `core`, `script-contracts`, `normative-inventory`, `coverage`, `coverage-gate`, and `dependency-policy`; do not omit `coverage-gate`. Extend `scripts/ci_summary.py` to build the strict manifest from `CI_NEEDS_JSON` plus current GitHub context, including `event_name`, `ref`, `ref_protected`, exact job IDs/conclusions, and normalized branch-policy evidence. On each eligible push, mint a short-lived GitHub App installation token using a protected Actions secret; the app has only repository Administration:read permission and queries `GET /repos/{owner}/{repo}/branches/release%2F1.0.0/protection` in that run. Require a protected response, at least one approving review, stale-review dismissal, an explicitly present `bypass_pull_request_allowances` object with empty `users`, `teams`, and `apps` arrays, admin enforcement, and force-push/deletion disabled; an absent or malformed bypass policy, any bypass entry, missing secret/token, API error, missing/insufficient policy field, or malformed response fails closed. Keep the private key secret unavailable to PR/schedule jobs; never log token material. Accept evidence only when `event_name` is `push`, `ref` is exactly `refs/heads/release/1.0.0`, and `ref_protected` is true; reject `pull_request`, `schedule`, all other refs/events, and unprotected runs even if the aggregate job succeeds. Do not persist the manifest or embed run IDs or mutable CI results in committed generated artifacts. Recompute current implementation and test evidence before accepting catalog `evidence_complete`; downgrade stale/contradictory state to effective `evidence_incomplete`, and never silently advance a lower catalog state when evidence passes.
- [ ] T027 Update Baseline Client's `claim_state` in `specification/catalog/kmip-2.1.json` to `evidence_incomplete` while its 16 mandatory fixtures are unavailable; keep target membership in `specification/catalog/profile-targets.json`, keep the published claim absent, and show every fixture as a blocker.
- [ ] T028 Run focused readiness tests and commit the minimal passing implementation as the signed Green commit; record evidence in `specs/010-profile-conformance/implementation-evidence.md`.
- [ ] T029 Refactor state aggregation so `applicability`, target selection, runtime validation, evidence completion, and release publication remain independent; commit as the signed Refactor commit and record evidence.

## Phase 6: Polish, conformance, and review readiness

- [ ] T030 Add catalog traceability tests in `tools/normative_catalog/tests/test_feature_traceability.py` for complete profile clause/requirement closure, inherited profile requirement IDs, `KMIPKIT-0010` ownership, typed rule mappings, code references, and verification references.
- [ ] T031 Add regression cases in `crates/kmipkit-client/tests/unit/profile_preflight_tests.rs` for redaction, explicit defaults, no retries, no hidden Query/Discover Versions exchange, no inference of profile status from their explicit responses, and preserved delivery state.
- [ ] T032 Add English and Spanish profile-validation guidance in `docs/user-guide/en/profile-conformance.md` and `docs/user-guide/es/profile-conformance.md` without claiming support.
- [ ] T033 Update `docs/compliance/conformance.md` and `specification/catalog/README.md` with the explicit profile-readiness workflow, Baseline fixture blocker, distinction between catalog `claim_state` and public claims, and no-claim status.
- [ ] T034 Add an executable Rustdoc example to `crates/kmipkit-protocol/src/profile.rs` and run it through the existing documentation test command.
- [ ] T035 Regenerate `crates/kmipkit-protocol/src/profiles_generated.rs` and `specification/catalog/coverage-report.md`; verify the pinned generator produces no diff on a second run, preserves target/lifecycle separation, and never embeds CI run IDs, attempts, job conclusions, or temporary manifest data in committed output.
- [ ] T036 Run all available pinned client-profile test fixtures and map each result to the exact official case ID; retain unavailable fixture statuses and do not reconstruct missing cases.
- [ ] T037 Run `cargo fmt --all --check`, workspace Clippy with `-D warnings`, all relevant Rust tests, Python catalog tests, documentation tests, and the repository CI matrix; confirm the aggregate job's manifest/readiness output remains ephemeral and committed generated artifacts stay deterministic; record exact commands/results in the PR.
- [ ] T038 Measure changed-code and protocol/profile coverage against the approved thresholds; document any generated-code exclusion and its behavioral tests.
- [ ] T039 Run dependency/license, supply-chain, FFI-surface, and parser/catalog security checks affected by the feature; document redaction and security effects.
- [ ] T040 Complete implementation and verification assignments in `specification/catalog/kmip-2.1.json` for every selected-profile requirement in the full inherited clause/subclause closure so the generated `specification/catalog/coverage-report.md` links each to its exact OASIS section, explicit typed rule mapping, implementation/generated record, and executable verification evidence; leave missing fixtures explicitly blocked and regenerate rather than editing report output by hand.
- [ ] T041 Run independent specification QA against `spec.md`, `plan.md`, `tasks.md`, the pinned-source closure oracle, catalog assignments, tests, and generated reports; resolve findings before review.
- [ ] T042 Run independent security review of catalog safe-loading, bounded target-manifest parsing and target/runtime separation, Rust literal escaping, generated metadata, profile selection, error redaction, request preflight, CI evidence producer trust, and readiness-state reconciliation; resolve findings before review.
- [ ] T043 Verify live repository protection for `release/1.0.0` through the same read-only API contract used by readiness; record the currently observed policy state, and keep readiness blocked if administrator-owned controls or the protected Actions app secret are absent. Update the feature branch from the current `release/1.0.0`, resolve conflicts, rerun all required checks, and verify the branch is clean.
- [ ] T044 Open a draft PR to `release/1.0.0` describing changes, Red/Green/Refactor commits, verification evidence, risks, and the 16 missing mandatory fixture blockers; do not make a public conformance claim.

## Dependencies and gates

- Phase 2 depends on the reviewed catalog and the pinned offline generator.
- User Story 1 depends on generated profile metadata and the existing client fake-transport contract.
- User Story 2 depends on profile requirement ownership and report tooling.
- User Story 3 depends on the evidence records from User Stories 1 and 2.
- Missing Baseline mandatory fixtures are a release-claim blocker, not permission to reconstruct or fetch fixture content during builds.
- Human review and approval remain required before merge; this feature does not publish a release or profile claim.

## Traceability map

| Specification requirement | Primary tasks |
|---|---|
| FR-001, FR-002 | T001-T008, T016-T022, T030, T035, T040 |
| FR-003, FR-004 | T003-T008, T018-T030, T040 |
| FR-005, FR-006 | T009-T015, T031 |
| FR-007 | T009-T015, T023-T029 |
| FR-008, FR-009 | T003-T008, T016-T022, T023-T029, T036 |
| FR-010, FR-011 | T016-T030, T033, T040 |
| FR-012 | T016, T031, T039, T042 |
| FR-013 | T032-T034 and later API/bindings parity specifications |
| FR-014, SC-009 | T003, T009-T015, T018, T023-T031, T040-T042 |
| FR-015, FR-016, SC-011 | T003, T005-T008, T035, T042 |
| FR-017, SC-011 | T023-T029, T040, T042 |
| FR-018, SC-015 | T023, T026, T035, T037 |
| FR-004, SC-012 | T001-T003, T005-T006, T009, T016, T018-T020, T023, T026, T030, T040-T042 |
| SC-013 | T023-T029, T040, T042 |
| SC-014 | T009-T015, T031, T041-T042 |
| SC-010 | T037-T044 |
| SC-001 | T001-T008, T016-T022, T030, T035, T040 |
| SC-002 | T001-T003, T016-T022, T030, T036, T040 |
| SC-003 | T023-T030, T036, T040 |
| SC-004 | T023-T030, T036, T040 |
| SC-005 | T009-T015, T031 |
| SC-006 | T016-T022, T035, T037 |
| SC-007 | T003-T008, T016-T022, T036 |
| SC-008 | T009-T015, T024, T031 |

## Notes

- `[P]` is intentionally unused because these tasks share generated profile data, ownership assignments, and review gates.
- Each Red, Green, and Refactor task requires a distinct DCO-signed development commit with exact command evidence.
- Unavailable official fixtures remain blocked; the feature may report evidence state but cannot claim conformance.
