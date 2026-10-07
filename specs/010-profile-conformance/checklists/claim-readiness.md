# Profile Claim Readiness Requirements Quality Checklist

**Purpose**: Reviewer checklist for the completeness and clarity of profile evidence and release-claim requirements
**Created**: 2026-10-07
**Feature**: [spec.md](../spec.md)

**Ownership**: Reviewer-owned. `[x]` means the reviewer accepts the requirements quality; agents do not mark these items complete during generation or implementation.

## Evidence Completeness

- [ ] CHK001 Are clauses, subclauses, requirements, dependencies, elements, and required tests all required before evidence can be complete? [Completeness, Spec §FR-004, §FR-008]
- [ ] CHK002 Are implementation and verification links required for every applicable normative requirement? [Traceability, Spec §FR-010]
- [ ] CHK003 Are MUST/SHALL, prohibitions, SHOULD deviations, and MAY/OPTIONAL capabilities handled explicitly? [Coverage, Spec §FR-004, §FR-008]
- [ ] CHK004 Are mandatory and optional official test cases distinguished without treating tests as independent normative sources? [Clarity, Spec §FR-008]
- [ ] CHK005 Is unavailable fixture status a blocker when a test is required by profile or project policy? [Edge Case, Spec §FR-009]

## Claim Boundaries

- [ ] CHK006 Is profile readiness kept separate from release publication and formal certification? [Scope, Spec §FR-011]
- [ ] CHK007 Does the readiness report avoid emitting a claim even when its evidence state is complete? [Clarity, Spec §FR-011]
- [ ] CHK008 Is Baseline Client the initial target while its claim remains ineligible with the current fixture evidence? [Acceptance Criteria, Spec §FR-003, §SC-004]
- [ ] CHK009 Are server prerequisites, deviations, source discrepancies, and missing evidence visible as blockers or qualified notes? [Completeness, Spec §FR-010]
- [ ] CHK010 Are server profiles, JSON/XML profiles, and unsupported conditional profiles excluded from 1.0 claims? [Scope, Spec §FR-007]

## Reviewability and Safety

- [ ] CHK011 Does the report preserve exact source and requirement references for each blocker? [Traceability, Spec §FR-001, §FR-010]
- [ ] CHK012 Are claims prevented when any applicable normative clause is unresolved or failing? [Acceptance Criteria, Spec §FR-004, §FR-011]
- [ ] CHK013 Is deterministic output required for identical evidence inputs? [Measurability, Spec §FR-010]
- [ ] CHK014 Are secrets, TLS private keys, and raw KMIP bodies excluded from errors and reports? [Security, Spec §FR-012]
- [ ] CHK015 Does profile readiness remain incomplete when any applicable requirement lacks an explicit typed rule mapping or executable test? [Fail Closed, Spec §FR-014]
- [ ] CHK016 Are branch freshness, required checks, independent reviews, and draft-only PR handling required before review? [Review Gate, Spec §SC-010]
- [ ] CHK017 Is readiness recomputed from current evidence, with stale `evidence_complete` downgraded and a lower catalog state never silently advanced? [Consistency, Spec §FR-017]
- [ ] CHK018 Do report tests prove that secret-bearing evidence/diagnostic inputs and raw KMIP-body markers never appear in reports or errors? [Security, Spec §FR-012]
- [ ] CHK019 Is current verification evidence bound to the exact repository, commit, workflow, run, and attempt, with missing, failed, duplicate, stale, and caller-asserted checks blocking readiness? [Evidence, Spec §FR-017]
- [ ] CHK020 Are run-specific manifests and readiness results excluded from committed generated reports and catalog lifecycle state? [Determinism, Spec §FR-018]

## Notes

- This checklist assesses written claim requirements; it is not evidence that any profile currently conforms.
