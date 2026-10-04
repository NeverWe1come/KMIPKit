# Specification Quality Checklist: KMIP TTLV Wire Codec

**Purpose**: Check the completeness and clarity of the feature specification before planning.
**Created**: 2026-10-04
**Feature**: [spec.md](../spec.md)

## Content Quality

- [x] No implementation detail beyond constraints necessary to define this protocol feature
- [x] Focused on caller and client value
- [x] All mandatory sections are complete

## Requirement Completeness

- [x] Requirements are testable and unambiguous within the stated scope
- [x] Success criteria are measurable
- [x] All primary acceptance scenarios are defined
- [x] Malformed-input and resource edge cases are identified
- [x] Scope boundaries and dependencies are explicit
- [x] Normative references identify exact OASIS document clauses
- [ ] Proposed ADR-0011 has an approved disposition for reserved-tag receipt under KMIPKIT-DISC-037
- [x] Configurable depth semantics are bounded to 0–64, consistent with the 004 model contract
- [x] The draft limits secret-bearing wire encoding to a conditional proposal and states that the current policy remains in force until both approval gates are met
- [ ] ADR-0012 is human-accepted, this feature specification is approved, and the request-only API boundary is enforceable before implementation relies on FR-013

## Feature Readiness

- [x] Each implementable functional requirement has acceptance criteria
- [x] Encoder, decoder, and resource-limits journeys are independently testable
- [x] Implementation and transport exclusions are stated
- [ ] All normative coverage can be implemented without relying on unresolved policy

## Notes

- The unchecked ADR item is an implementation gate, not an implied OASIS interpretation. The KMIPKIT-0004 model implementation is already merged into the release base; implementation still requires specification approval and the ADR disposition.
- OASIS §10.1.2 does not explicitly define empty Big Integer behavior; this draft records a project validity rule rejecting it. Schema field-order requirement `KMIPKIT-REQ-SPEC-10.1.2-001` remains unassigned pending typed-spec ownership for every applicable client 1.0 Structure.
- Full roadmap traceability remains gated until those Structures have approved typed-spec ownership plus implementation and executable order-verification references; this gap does not block implementation of the generic codec, and this PR does not claim the assignment is complete.
- The normative traceability table separates OASIS requirements from KMIPKit API and security policy.
- The draft proposes only temporary outbound TTLV for an explicitly caller-requested KMIP operation, held in a zeroizing, borrow-only owner through the transport write. `AGENTS.md` §8 remains in force until ADR-0012 is human-accepted and this feature specification is approved.
- FR-013 is a KMIPKit policy requirement, not an OASIS clause. Its policy/owner verification belongs in the task traceability map, and the public `encode(&Item)` proposal must be resolved so it cannot expose general-purpose serialization.
