# API Review Checklist: KMIP 2.1 Vendor Extension Registry

**Purpose**: Check the completeness and reviewability of the registry requirements before implementation.
**Created**: 2026-10-06
**Feature**: ../spec.md

**Note**: This reviewer-owned checklist evaluates the written requirements, not implementation completion.
**Review Ownership**: A reviewer may mark an item complete only after evaluating its requirements-quality criterion.
**Marker Semantics**: [x] means the requirement has been reviewed and judged clear and sufficient; it does not mean implementation is complete.

## Requirement completeness

- [ ] CHK001 Does the specification map every registry behavior to a stable requirement ID and exact OASIS clause where applicable? [Completeness, Spec §Scope]
- [ ] CHK002 Are the local extension identity fields distinguished from the standard KMIP Message Extension wire fields? [Clarity, Spec §Scope]
- [ ] CHK003 Are registration, construction, recognition, metadata, and language-parity capabilities each covered by functional requirements? [Completeness, Spec §Requirements]
- [ ] CHK004 Is the boundary between this registry slice and complete 1.0 API parity explicit? [Scope, Spec §Scope]

## Requirement clarity and consistency

- [ ] CHK005 Is an exact discriminator path, scalar type, and scalar value defined without allowing predicates or registration-order tie breaking? [Clarity, Spec §Clarification record]
- [ ] CHK006 Are zero-match, multiple-match, absent-path, repeated-path, and wrong-type outcomes specified consistently? [Consistency, Spec §User Story 2]
- [ ] CHK007 Is the data-only schema vocabulary precise enough to implement without scripts, callbacks, regexes, or arbitrary expressions? [Clarity, Spec §Clarification record]
- [ ] CHK008 Are ownership boundaries among protocol, client, FFI, and language facades consistent across the spec, plan, and contracts? [Consistency, Spec §Requirements]
- [ ] CHK009 Is the typed execution boundary closed against raw bodies, arbitrary generic Items, and caller conversion traits? [Clarity, Spec §Requirements]

## Acceptance criteria quality

- [ ] CHK010 Can every registration, repeated outbound extension, and inbound recognition/preservation fixture produce an objectively comparable outcome across Rust, C, Java, and Python? [Measurability, Spec §Success Criteria]
- [ ] CHK011 Are deterministic output, metadata order, and error-category equivalence measurable from shared fixtures? [Measurability, Spec §Success Criteria]
- [ ] CHK012 Does the feature define evidence needed to demonstrate zero payload disclosure in exercised diagnostics? [Measurability, Spec §Success Criteria]
- [ ] CHK013 Is requirement-to-source/spec/code/test traceability explicitly required at 100 percent? [Acceptance Criteria, Spec §Success Criteria]

## Scenario and edge-case coverage

- [ ] CHK014 Are valid, invalid, ambiguous, unknown-critical, and unknown-non-critical paths all addressed? [Coverage, Spec §User Stories]
- [ ] CHK015 Are unknown tags, enum values, bitmask bits, multiplicity, and order required to remain losslessly available? [Coverage, Spec §Requirements]
- [ ] CHK016 Are byte, depth, element, discriminator-value, constraint-member, payload-index, and lookup-comparison limits explicit, counted before allocation, and parity-tested? [Edge Cases, Spec §Edge Cases]
- [ ] CHK017 Are malformed compatibility ranges, conflicting definitions, and partial registry-construction failure addressed? [Edge Cases, Spec §User Story 1]
- [ ] CHK018 Are secret-bearing payloads and default error/debug/display redaction covered? [Security, Spec §Edge Cases]

## Non-functional and dependency requirements

- [ ] CHK019 Are zeroization guarantees distinguished from copies retained by C callers, Java, and Python runtimes? [Security, Spec §Requirements]
- [ ] CHK020 Is lookup/schema work bounded by one shared payload tag index, unique-child path resolution, per-step and aggregate comparison budgets, binary enum membership, and one-pass order-edge validation? [Performance, Spec §Requirements]
- [ ] CHK021 Are generated outputs, manifest ownership, deterministic regeneration, and no-hand-edit rules explicit? [Completeness, Spec §Requirements]
- [ ] CHK022 Are toolchain versions and required C, Java 17, Python 3.12, and Rust verification environments specified? [Dependencies, Plan §Technical Context]
- [ ] CHK023 Is the absence of server support, Query execution, dynamic executable plugins, and vendor-specific built-ins explicit? [Scope, Spec §Scope]
## Cross-artifact and ambiguity checks

- [ ] CHK024 Do tasks.md paths, test scenarios, language versions, coverage gates, and generated destinations agree with plan.md and the contracts? [Consistency, Tasks]
- [ ] CHK025 Does any remaining decision materially alter schema expressiveness, wire representation, ownership, security, or acceptance testing? [Ambiguities, Spec and Plan]
- [ ] CHK026 Are repeatable per-item attachment, caller order, and explicit Criticality Indicator requirements clear, including that KMIPKit applies no hidden default? [Completeness, Spec §FR-005; OASIS §8.3, Table 396]

## Notes

- Leave every item unchecked until an independent reviewer evaluates the requirement itself.
- Record findings inline and link any resulting correction or decision.
