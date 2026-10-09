# Protocol Requirements Checklist: Server-Generated Object Creation

**Purpose**: Review completeness, clarity, and normative correctness of the operation requirements.

**Created**: 2026-10-07
**Feature**: spec.md

**Review ownership**: This is a reviewer-owned requirements-quality checklist. Mark [x] only after independent review. The marker does not mean implementation is complete.

## Completeness

- [ ] CHK001 Are all Table 186-187 fields represented with exact type, requiredness, and cardinality, including the required outer Attributes structure that may be empty under §5.1? [Spec §§5.1 and 6.1.8]
- [ ] CHK002 Are all Table 189-190 request/response fields and every Table 192 Result Reason covered? [Spec section 6.1.9]
- [ ] CHK003 Are the four Table 191 attributes named, and are Common fallback and key-specific overrides resolved using §6.1.9 precedence before comparing the effective Private/Public values? [Spec section 6.1.9]
- [ ] CHK004 Are all Table 193 fields covered, including its required Attributes structure that may be empty under §5.1, along with repeated Table 194 identifiers and Table 195 errors? [Spec §§5.1 and 6.1.10]
- [ ] CHK013 Does the review distinguish the KMIPKit caller-input restriction for Polynomial Sharing Prime Field from normative evidence: §2.8/Table 9 describes the Split Key object, while Table 193 marks request Prime Field Size optional? [Spec §§2.8 and 6.1.10]
- [ ] CHK014 Do attribute groups preserve ordered direct §4 Object Attribute TTLV items and their tags/typed values under §§5.1–5.4, Tables 157–160, while Table 150 is applied only to the §4.60 Vendor Attribute with required Vendor Identification? [Spec §§4.60 and 5.1–5.4]
- [ ] CHK005 Are applicable shared message, batch, result, asynchronous, and delivery-state clauses identified? [Spec sections 7-9]

## Clarity and consistency

- [ ] CHK006 Does the spec distinguish client preservation of Common/Private/Public groups from the server's union behavior? [Spec FR-003]
- [ ] CHK007 Does it distinguish requested split-key attributes from effective attributes of an input key? [Spec FR-005]
- [ ] CHK008 Are cryptographic parameters explicit only when the cited operation tables require them? [Spec FR-006]
- [ ] CHK009 Are success, Failure, Pending, malformed payloads, and missing required fields unambiguous? [Spec FR-002, FR-005, FR-008]

## Acceptance evidence

- [ ] CHK010 Does the review distinguish the locally available full `TC-CREATE-SD-1-21` XML fixture from the Create-only derived test and avoid claiming the complete two-operation official case passes? [Spec FR-010, SC-001 through SC-006]
- [ ] CHK011 Does the contract use the finalized client API and single execute writer without a second path? [Plan Constitution Check]
- [ ] CHK012 Are redaction, zeroization, one-exchange, and no-retry claims measurable through named tests? [Spec FR-007 through FR-009]

## Notes

Leave every checkbox unchanged until an independent reviewer evaluates the requirements.
