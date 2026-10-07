# Contract: Profile Evidence and Claim Readiness

## Report contents

For each profile, report its role, applicability, selected-target state from `profile-targets.json`, exact source clauses, stable normative requirement IDs and keywords, dependency closure, related protocol elements, typed rule mappings and rule-test coverage, mandatory and optional official test IDs, fixture availability, implementation/verification assignments, accepted deviations, server prerequisites, evidence state, and blockers.

The source-clause and requirement lists MUST include the complete normative closure: every directly referenced clause, every clause or source document normatively incorporated by reference, all applicable subclauses, and each applicable normative statement linked to those clauses. Baseline Client's Profiles §6.1 item 1 includes every KMIP Specification v2.1 clause and its client-applicable requirements within the approved 1.0 scope; item 2 includes Basic Authentication Suite §3.1 and its applicable subclauses. The report MUST identify a missing closure member by exact clause or requirement ID.

## Evidence rules

- Applicability, selected target, runtime validation selection, catalog `claim_state`, and published release claim are separate values. Baseline `claim_state` is `evidence_incomplete` while its required fixtures are unavailable; no public claim is emitted.
- Readiness reports expose target membership and runtime validation availability as separate booleans and one effective evidence/readiness lifecycle value; they do not create a duplicate readiness enum. The evaluator recomputes all required evidence before comparing it with catalog `claim_state`.
- Every applicable normative client requirement has an implementation and verification assignment before a claim is eligible.
- A verification assignment uses stable test IDs and current CI check IDs. An arbitrary result artifact path or a manually asserted `verified` value is not evidence.
- MUST/SHALL statements must pass; MUST NOT/SHALL NOT statements have negative tests; deviations from SHOULD/SHOULD NOT/RECOMMENDED require an accepted decision; in-scope MAY/OPTIONAL capabilities remain represented and usable.
- Every project-required mandatory test passes before evidence is complete. A missing fixture is unavailable evidence, never a pass.
- Catalog validation rejects `evidence_complete` when catalog-linked obligations, required tests, or required fixtures are missing or unavailable; the readiness report also checks current implementation and test results.
- Current test results are accepted only from a strict manifest created by the final aggregate job in `.github/workflows/ci.yml` during the same `push` run on the protected `release/1.0.0` ref. It is generated from the current GitHub Actions `needs` context and records event name, ref, protection status, canonical repository, checked-out commit SHA, workflow reference, run ID/attempt, current branch-protection API result, and unique verification/job conclusions. The evaluator requires `event_name: push`, `ref: refs/heads/release/1.0.0`, `ref_protected: true`, matches the manifest to the current checkout and workflow context, and requires success for exact job IDs `core`, `script-contracts`, `normative-inventory`, `coverage`, `coverage-gate`, and `dependency-policy`. It does not download or accept user-supplied result artifacts.
- Every eligible run queries the branch-protection API with a short-lived GitHub App installation token scoped to read-only Administration permission. The live response must show at least one required approving review, stale-review dismissal, an explicitly present `bypass_pull_request_allowances` object with empty `users`, `teams`, and `apps` arrays, admin enforcement, force pushes disabled, and deletions disabled. A missing object or array, malformed value, any bypass entry, API error, absent/expired token, other missing field, or insufficient policy is a blocker; the token is unavailable to PR/schedule jobs and never logged.
- A local report without the current same-run protected-release-push CI manifest may show catalog facts and blockers, but MUST leave test-result evidence unverified and MUST NOT produce `evidence_complete`. A `pull_request`, `schedule`, or other event, unprotected/wrong ref, previous run, another commit/repository/workflow, failed or duplicate check, missing metadata, or mismatched run attempt is a blocker.
- The CI manifest and current-run readiness result are ephemeral and MUST NOT be embedded in or used to modify committed generated reports, catalog records, or release claims. The committed `coverage-report.md` is generated from stable checked-in inputs and remains byte-identical across runs; CI may expose its run-scoped evaluation separately.
- A stale `evidence_complete` catalog value is downgraded to effective `evidence_incomplete` when current evidence has a blocker. If evidence passes but the catalog value has not been reviewed and advanced, the report retains the lower state and requires an explicit catalog update. Reports never silently advance catalog state.
- Test Cases and Usage Guide text do not create normative requirements unless an exact normative clause incorporates them.
- An unresolved discrepancy, unassigned obligation, missing dependency, missing mandatory fixture, failing test, or unknown condition keeps readiness incomplete.
- Readiness output never publishes a claim. Only a separately reviewed release process may make one after the evidence gate passes.

## Current Baseline state

The pinned inventory links 16 mandatory test cases to Baseline Client, and all 16 currently have unavailable fixture status. The readiness report must show this as a blocker and must not state or imply Baseline Client conformance.

## Stable output

The committed coverage report is generated from reviewed, checked-in inputs and uses deterministic ordering and repository-relative references. It contains no host-specific paths, run-specific CI identifiers, credentials, secret data, TLS private keys, or raw KMIP bodies. Any ephemeral CI readiness output contains only stable check IDs, conclusions, and current run metadata, never raw logs or command output.

Implementation references are canonical repository-relative paths validated beneath the checkout root. Verification references use stable test/check IDs. CI output includes only result metadata, not logs or raw command output, so run freshness can be shown without leaking secret-bearing diagnostics or making reports depend on host-specific artifact paths.
