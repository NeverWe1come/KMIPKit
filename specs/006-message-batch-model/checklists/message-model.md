# Message Model Requirements Checklist

**Purpose**: Review the completeness and clarity of the message-model requirements.
**Created**: 2026-10-04
**Feature**: [spec.md](../spec.md)
**Reviewed spec blob**: `85a63666337fac0883918acce711b6be1c1e4835`

**Revision note**: CHK020 wording is clarified in this PR. Its existing checked status and reviewed blob refer to the prior wording only. KMIPKIT-0006 T001 remains open and requires an independent reviewer to evaluate the checklist against the exact updated specification revision before implementation.

**Review ownership**: Reviewer-owned. `[x]` means the requirement wording was reviewed and found adequate; it does not indicate implementation completion.

## Requirement Completeness

- [x] CHK001 [Spec §FR-001, FR-002] Are all fields in Tables 394–399 represented or assigned to a named follow-on specification?
- [x] CHK002 [Spec §FR-003] Are message cardinality and Batch Count relationships defined for requests and responses?
- [x] CHK003 [Spec §FR-004] Does the specification state the Table 396 multi-item ID condition and the §9.21 response echo condition without moving request-response matching into `kmipkit-protocol`?
- [x] CHK004 [Spec §FR-006, FR-007] Are option presence, effective defaults, and the `KMIPKIT-DISC-001` limit specified?
- [x] CHK005 [Spec §FR-009, FR-010] Are Pending, asynchronous correlation, and deferred client actions explicitly covered, including Result Message prohibition for Success/Pending?

## Clarity and Consistency

- [x] CHK006 [Spec §FR-001, FR-013] Is the relationship between canonical known-field order and exact preservation of unknown source fields unambiguous?
- [x] CHK007 [Spec §FR-008] Are Text String message correlation values distinguished from Byte String batch and async IDs?
- [x] CHK008 [Spec §FR-011] Does the spec separate Message Extension structural validation from client-registry recognition and criticality enforcement?
- [x] CHK009 [Spec §FR-012] Does the model preserve raw version data while assigning exact-2.1 send/response acceptance and non-2.1 rejection tests to KMIPKIT-0007 under ADR-0002, with the §9.16 scope exception recorded as `KMIPKIT-DISC-022`?
- [x] CHK010 [Spec §FR-017, plan.md] Is the scoped read-only TTLV view sufficient for structural validation without payload copies?

## Acceptance and Edge Coverage

- [x] CHK011 [Spec §SC-001–SC-007] Can each success criterion be verified by an implementation test or coverage report?
- [x] CHK012 [Spec §Edge Cases] Are absent versus explicit defaults, unknown values, malformed counts, reordered responses, and redacted diagnostics covered?
- [x] CHK013 [Spec §Scope and normative sources] Are official OASIS fixtures clearly distinguished from derived table-based tests?
- [x] CHK014 [Spec §Exclusions] Are encoding, transport, operation payload schemas, and server-initiated behavior explicitly excluded?
- [x] CHK015 [Spec §FR-004, catalog §9.8] Is request-ID distinctness labeled as a KMIPKit project invariant, and is the Batch Order execution duty classified as server-only?
- [x] CHK016 [Spec §FR-004] Is response Operation's conditional requiredness preserved and assigned to paired client validation?
- [x] CHK017 [Spec §FR-011] Are repeated request Message Extensions distinguished from the optional singleton response field, with a duplicate-response negative test?
- [x] CHK018 [Spec §Follow-on ownership] Do each deferred client behavior and its verification have a specific downstream feature owner and test?
- [x] CHK019 [Spec §FR-008, FR-022] Does Server Correlation Value remain response metadata for client-to-server 1.0 traffic, with 1.1 request use deferred?
- [x] CHK020 [Spec §FR-019] Do message parsing/conversion preserve raw option Enumeration values, while `KMIPKIT-0007-client-execution` validates Asynchronous Indicator values against its assigned and extension allocations, accepts assigned Batch Error Continuation values, rejects values outside its assigned and extension allocations, and defers extension-range Batch Error Continuation policy until `KMIPKIT-0007-OD-001` receives source/catalog-owner disposition?
- [x] CHK021 [Spec §FR-011] Does the specification attribute registry behavior to the extension architecture document and client-execution feature rather than to ADR-0007?

- [x] CHK022 [Spec SC-002–SC-004; plan Property-test contract] Are generated test bounds, fixed seed, case count, raw-value domains, and byte-string samples stated as test-only constraints rather than protocol acceptance limits?
