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
- [x] The draft limits secret-bearing wire encoding to a conditional proposal and states that the current policy remains in force until all three human approvals and the first-client integration gate are satisfied
- [ ] The three approvals for FR-013 are explicit and distinct: human acceptance of ADR-0012, approval of this feature specification, and approval of the enforceable boundary design. These do not implement or authorize a production request path.

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
- The draft proposes only temporary outbound TTLV for an explicitly caller-requested typed operation, held in a private zeroizing owner through the transport write, with initialized encoded bytes zeroized before owner deallocation/drop; spare or otherwise uninitialized `Vec` capacity is outside the guarantee unless explicitly initialized and cleanup is verified. Public generic values and bounded decoding stay in `kmipkit-ttlv`. KMIPKIT-0005 may implement/test the private writer but adds no permit, `Client::execute`, or production writer callsite. The first client feature/spec defines the caller-facing request/limit API, execute-owned permit, only production callsite/mint site, exact-one audit, and integration test. `AGENTS.md` §8 remains in force unless all three approvals are obtained and the first client integration test passes; the candidate first-client feature PR must contain both the sole production callsite and its owner-through-transport integration test. CI must run and pass that test against the candidate callsite before merge, enablement, or release; until then, the release branch must contain no production callsite or secret-bearing send.
- FR-013 is a KMIPKit policy requirement, not an OASIS clause. Its policy/owner verification belongs in the task traceability map. No public `encode(&Item)` API or general-purpose encoder is proposed. Private codec tests cover default/configured encoder bytes/depth/count and U32 preflight. If review rejects the permit/closed-request boundary or it cannot be enforced, do not approve or implement a secret-bearing request path.
## T001 reconciliation note — 2026-10-05

T001 records ADR-0011 as accepted catalog decision KMIPKIT-DEC-001, closes KMIPKIT-DISC-037 against OASIS KMIP Specification v2.1 §11.56, and validates/regenerates the catalog report before T006. The delegated FR-013 scope records the three distinct design decisions (ADR-0012 acceptance, specification approval, and enforceable-boundary approval) in approval-record.md. This does not pass the first-client integration gate or authorize a production callsite. Dependency dispositions, the OOM limitation, and the schema-order follow-on are also recorded there and in the feature artifacts. This note does not change reviewer-owned checklist status; the reviewer must reconcile the still-open schema-order coverage and implementation evidence.
