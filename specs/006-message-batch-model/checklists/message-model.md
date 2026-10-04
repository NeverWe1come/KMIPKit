# Message Model Requirements Checklist

**Purpose**: Review the completeness and clarity of the message-model requirements.
**Created**: 2026-10-04
**Feature**: [spec.md](../spec.md)

**Review ownership**: Reviewer-owned. `[x]` means the requirement wording was reviewed and found adequate; it does not indicate implementation completion.

## Requirement Completeness

- [ ] CHK001 [Spec §FR-001, FR-002] Are all fields in Tables 395–399 represented or assigned to a named follow-on specification?
- [ ] CHK002 [Spec §FR-003] Are message cardinality and Batch Count relationships defined for requests and responses?
- [ ] CHK003 [Spec §FR-004] Does the specification state the Table 396 multi-item ID condition and the §9.21 response echo condition without moving request-response matching into `kmipkit-protocol`?
- [ ] CHK004 [Spec §FR-006, FR-007] Are option presence, effective defaults, and the `KMIPKIT-DISC-001` limit specified?
- [ ] CHK005 [Spec §FR-009, FR-010] Are Pending, asynchronous correlation, and deferred client actions explicitly covered, including Result Message prohibition for Success/Pending?

## Clarity and Consistency

- [ ] CHK006 [Spec §FR-001, FR-013] Is the relationship between canonical known-field order and exact preservation of unknown source fields unambiguous?
- [ ] CHK007 [Spec §FR-008] Are Text String message correlation values distinguished from Byte String batch and async IDs?
- [ ] CHK008 [Spec §FR-011] Does the spec separate Message Extension structural validation from client-registry recognition and criticality enforcement?
- [ ] CHK009 [Spec §FR-012] Does the spec preserve raw version data while deferring the `KMIPKIT-DISC-022` compatibility policy?
- [ ] CHK010 [Spec §FR-017, plan.md] Is the scoped read-only TTLV view sufficient for structural validation without payload copies?

## Acceptance and Edge Coverage

- [ ] CHK011 [Spec §SC-001–SC-007] Can each success criterion be verified by an implementation test or coverage report?
- [ ] CHK012 [Spec §Edge Cases] Are absent versus explicit defaults, unknown values, malformed counts, reordered responses, and redacted diagnostics covered?
- [ ] CHK013 [Spec §Scope and normative sources] Are official OASIS fixtures clearly distinguished from derived table-based tests?
- [ ] CHK014 [Spec §Exclusions] Are encoding, transport, operation payload schemas, and server-initiated behavior explicitly excluded?
- [ ] CHK015 [Spec §FR-004, catalog §9.8] Is request-ID distinctness labeled as a KMIPKit project invariant, and is the Batch Order execution duty classified as server-only?
- [ ] CHK016 [Spec §FR-004] Is response Operation's conditional requiredness preserved and assigned to paired client validation?
- [ ] CHK017 [Spec §FR-011] Are repeated request Message Extensions distinguished from the optional singleton response field, with a duplicate-response negative test?
- [ ] CHK018 [Spec §Follow-on ownership] Do each deferred client behavior and its verification have a specific downstream feature owner and test?
- [ ] CHK019 [Spec §FR-008, FR-022] Does Server Correlation Value remain response metadata for client-to-server 1.0 traffic, with 1.1 request use deferred?
- [ ] CHK020 [Spec §FR-019] Do message parsing/conversion preserve raw option Enumeration values, while `KMIPKIT-0007-client-execution` accepts assigned and OASIS extension values and rejects unassigned outbound values outside Tables 432/435?
- [ ] CHK021 [Spec §FR-011] Does the specification attribute registry behavior to the extension architecture document and client-execution feature rather than to ADR-0007?
