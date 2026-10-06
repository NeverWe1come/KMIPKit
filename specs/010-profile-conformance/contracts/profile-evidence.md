# Contract: Profile Evidence and Claim Readiness

## Report contents

For each profile, report its role, applicability, selected-target state from `profile-targets.json`, exact source clauses, stable normative requirement IDs and keywords, dependency closure, related protocol elements, typed rule mappings and rule-test coverage, mandatory and optional official test IDs, fixture availability, implementation/verification assignments, accepted deviations, server prerequisites, evidence state, and blockers.

## Evidence rules

- Applicability, selected target, runtime validation selection, catalog `claim_state`, and published release claim are separate values. Baseline `claim_state` is `evidence_incomplete` while its required fixtures are unavailable; no public claim is emitted.
- Readiness reports expose target membership and runtime validation availability as separate booleans and one effective evidence/readiness lifecycle value; they do not create a duplicate readiness enum. The evaluator recomputes all required evidence before comparing it with catalog `claim_state`.
- Every applicable normative client requirement has an implementation and verification assignment before a claim is eligible.
- MUST/SHALL statements must pass; MUST NOT/SHALL NOT statements have negative tests; deviations from SHOULD/SHOULD NOT/RECOMMENDED require an accepted decision; in-scope MAY/OPTIONAL capabilities remain represented and usable.
- Every project-required mandatory test passes before evidence is complete. A missing fixture is unavailable evidence, never a pass.
- Catalog validation rejects `evidence_complete` when catalog-linked obligations, required tests, or required fixtures are missing or unavailable; the readiness report also checks current implementation and test results.
- A stale `evidence_complete` catalog value is downgraded to effective `evidence_incomplete` when current evidence has a blocker. If evidence passes but the catalog value has not been reviewed and advanced, the report retains the lower state and requires an explicit catalog update. Reports never silently advance catalog state.
- Test Cases and Usage Guide text do not create normative requirements unless an exact normative clause incorporates them.
- An unresolved discrepancy, unassigned obligation, missing dependency, missing mandatory fixture, failing test, or unknown condition keeps readiness incomplete.
- Readiness output never publishes a claim. Only a separately reviewed release process may make one after the evidence gate passes.

## Current Baseline state

The pinned inventory links 16 mandatory test cases to Baseline Client, and all 16 currently have unavailable fixture status. The readiness report must show this as a blocker and must not state or imply Baseline Client conformance.

## Stable output

The report is generated from reviewed, checked-in inputs and uses deterministic ordering and repository-relative references. It contains no host-specific paths, credentials, secret data, TLS private keys, or raw KMIP bodies.
