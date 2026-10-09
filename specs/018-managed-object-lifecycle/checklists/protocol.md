# Protocol Requirements Checklist: KMIP 2.1 Managed-Object State Transitions

**Purpose**: Review completeness, clarity, consistency, and testability of the four operation requirements before implementation.
**Created**: 2026-10-09
**Feature**: [spec.md](../spec.md)

**Note**: This checklist is generated from KMIPKIT-0018's requirements and design artifacts.
**Review Ownership**: Reviewer-owned. Leave each item unchecked until a reviewer has assessed the written requirement; `[x]` does not mean implementation is complete.
**Depth**: Standard protocol and conformance review; audience is the independent specification/PR reviewer.

## Requirement Completeness

- [ ] CHK001 Does the scope identify all four in-scope client operations and exclude adjacent operations with distinct payloads? [Completeness, Spec §Normative scope; §Product boundaries]
- [ ] CHK002 Are the exact request, response, and error table ranges cited for Activate, Archive, Destroy, and Recover? [Traceability, Spec §Normative scope]
- [ ] CHK003 Are all applicable client requirement IDs present, and are Activate/Destroy server-only clauses kept out of client obligations? [Actor scope, Spec §Normative scope; §FR-003–FR-005]
- [ ] CHK004 Are optional request Unique Identifier omission and required successful-response identifier behavior specified for every operation? [Completeness, Spec §Normative scope; §FR-002]
- [ ] CHK005 Are Success, Failure, Pending, malformed success, and transport-delivery outcomes covered? [Scenario coverage, Spec §User Scenarios; §FR-006]

## Requirement Clarity

- [ ] CHK006 Is Archive clearly described as a client preference request without a claim that archival has completed? [Clarity, Spec §User Story 2; §FR-004]
- [ ] CHK007 Does the Recover requirement identify Poll and Get as separate caller-controlled actions rather than automatic follow-ups? [Clarity, Spec §User Story 3; §FR-005]
- [ ] CHK008 Is the condition for accepting Pending tied to the existing asynchronous request/response rules? [Clarity, Spec §FR-005–FR-007]
- [ ] CHK009 Are the server-authoritative lifecycle effects explicitly distinguished from client wire behavior? [Actor boundary, Spec §Product boundaries; §FR-003–FR-004]

## Requirement Consistency

- [ ] CHK010 Do operation-specific scenarios agree with the common one-exchange/no-retry contract? [Consistency, Spec §User Scenarios; §FR-007]
- [ ] CHK011 Does the spec consistently preserve unknown values through existing generic TTLV behavior without changing tag-allocation policy? [Consistency, Spec §Product boundaries; §FR-008]
- [ ] CHK012 Do the plan, public Rust contract, data model, and traceability matrix use the same names, operations, and response rules? [Consistency, plan.md; data-model.md; contracts/public-rust.md; traceability.md]

## Acceptance Criteria Quality

- [ ] CHK013 Can each success criterion be objectively judged from mapped operation vectors, fake-transport results, and traceability records? [Measurability, Spec §Success Criteria]
- [ ] CHK014 Does 100% traceability mean every applicable catalog requirement and operation-table obligation links to a specification, implementation, and executable verification? [Measurability, Spec §FR-009; §SC-002]
- [ ] CHK015 Are derived vectors distinguished from official OASIS Test Cases where the catalog has no official link? [Evidence quality, Spec §Normative scope; §SC-002; traceability.md]

## Edge Cases and Dependencies

- [ ] CHK016 Are omitted identifiers, valid batch ID Placeholder use, and response-correlation expectations bounded by shared KMIP rules? [Edge cases, Spec §Edge Cases; data-model.md]
- [ ] CHK017 Are missing, duplicate, wrong-typed, or malformed required response fields specified as protocol errors rather than typed success? [Exception coverage, Spec §Edge Cases; §FR-006]
- [ ] CHK018 Are dependencies on KMIPKIT-0006, KMIPKIT-0007, and KMIPKIT-0009 explicit without duplicating or changing their contracts? [Dependency, Spec §Assumptions; plan.md]
- [ ] CHK019 Are English and Spanish documentation criteria bounded to claims supported by the request and response tables? [Documentation, Spec §SC-004]

## Notes

- Static source and catalog references are listed in [research.md](../research.md) and [traceability.md](../traceability.md); independent reviewer completion is outstanding.
- The checklist evaluates requirements quality, not implementation completion or official KMIP certification.
