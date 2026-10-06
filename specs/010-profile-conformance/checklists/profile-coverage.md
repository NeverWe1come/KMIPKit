# Profile Coverage Requirements Quality Checklist

**Purpose**: Reviewer checklist for completeness and clarity of client profile requirements
**Created**: 2026-10-07
**Feature**: [spec.md](../spec.md)

**Ownership**: Reviewer-owned. `[x]` means the reviewer accepts the requirements quality; agents do not mark these items complete during generation or implementation.

## Requirement Completeness

- [ ] CHK001 Are all 18 catalogued client profiles included, with the 15 currently applicable or conditional client profiles accounted for? [Completeness, Spec §FR-002]
- [ ] CHK002 Are profile clauses, normative requirements, dependencies, elements, transport/encoding conditions, and official test IDs all part of the profile record? [Completeness, Spec §FR-002]
- [ ] CHK003 Are server-only and out-of-scope client profiles visibly excluded from 1.0 claims? [Coverage, Spec §FR-007]
- [ ] CHK004 Are exact OASIS source sections and stable catalog IDs required for every profile obligation? [Traceability, Spec §FR-001, §FR-002]

## Requirement Clarity and Consistency

- [ ] CHK005 Does the specification distinguish required minimum capabilities from prohibited operations? [Clarity, Spec §FR-005]
- [ ] CHK006 Are applicability, target selection for generation/reporting, explicit runtime validation selection, evidence state, and public claim separate, with no target-manifest allowlist? [Consistency, Spec §FR-003, §FR-015]
- [ ] CHK007 Are conditional profiles prevented from becoming eligible until their exact catalogued conditions are satisfied? [Clarity, Spec §FR-007]
- [ ] CHK008 Are normative defaults limited to their exact conditions and kept separate from explicit cryptographic choices? [Security, Spec §FR-006]

## Acceptance Criteria and Edge Cases

- [ ] CHK009 Are missing, failing, conflicting, or unassigned profile obligations defined as blockers? [Acceptance Criteria, Spec §FR-004, §FR-010]
- [ ] CHK010 Are missing official fixtures clearly defined as unavailable evidence rather than a pass? [Edge Case, Spec §FR-008, §FR-009]
- [ ] CHK011 Is the behavior for extra base-protocol-conformant operations defined without treating a profile list as an allowlist? [Edge Case, Spec §FR-005]
- [ ] CHK012 Are hidden Query/Discover calls and retries explicitly excluded from profile validation? [Scope, Spec §FR-005]
- [ ] CHK013 Are profile dependency cycles, unknown IDs, and conflicting constraints covered? [Edge Case, Spec §FR-007]

## Non-Functional Requirements and Assumptions

- [ ] CHK014 Are deterministic report ordering and the ban on environment-specific output stated? [Measurability, Spec §FR-010]
- [ ] CHK015 Are redaction and secret-handling constraints stated for profile errors and reports? [Security, Spec §FR-012]
- [ ] CHK016 Are later Rust/C/Java/Python API parity requirements separated from this core semantic feature? [Dependency, Spec §FR-013]
- [ ] CHK017 Is Baseline Client identified as a selected target without implying it is mandated specifically by §14.1? [Clarity, Spec §FR-003]
- [ ] CHK018 Is the missing-fixture source acquisition prerequisite kept distinct from any implementation claim? [Dependency, Spec §FR-009]
- [ ] CHK019 Is every runtime-selectable profile backed by explicit typed mappings and executable tests for all applicable requirements, with no interpretation of catalog prose? [Fail Closed, Spec §FR-014]
- [ ] CHK020 Is target-manifest input fixed-path, bounded, strict, duplicate-safe, semantically valid, and confined to in-scope client profiles? [Security, Spec §FR-015]
- [ ] CHK021 Are catalog-derived Rust strings encoded by a tested Rust-literal encoder that cannot inject syntax? [Security, Spec §FR-016]

## Notes

- `$speckit-implement` does not modify reviewer-owned checkbox markers.
