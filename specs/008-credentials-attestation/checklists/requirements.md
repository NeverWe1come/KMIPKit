# Requirements Checklist: KMIPKIT-0008 Credentials and Attestation

**Purpose**: Reviewer-owned requirements-quality checklist for the credential and attestation specification.<br>
**Created**: 2026-10-05<br>
**Feature**: [spec.md](../spec.md)<br>
**Review Ownership**: Independent reviewer / human maintainer. All items remain unchecked until that review occurs.

## Normative coverage and applicability

- [ ] CHK001 Does every OASIS requirement cite the pinned Specification revision, exact section/table, and stable catalog requirement or element ID?
- [ ] CHK002 Does the spec identify §9.4's lowercase “must” against §1.2's uppercase RFC 2119 keywords, avoid client testing/enforcement of “all Credentials satisfied,” and track catalog-owner correction separately?
- [ ] CHK003 Does the spec represent every named Table 442 Credential Type and preserve unknown Credential Type and enum values?
- [ ] CHK004 Does the spec map each known Credential Value to the exact required/optional fields and encoding types in Tables 411–416?
- [ ] CHK005 Are all field-order and repeated-Credential rules explicit and testable without inventing rules for unknown extensions?
- [ ] CHK006 Does the spec distinguish base KMIP credential construction from profile-specific authentication applicability and claims assigned to 0010?

## Domain ambiguity and defaults

- [ ] CHK007 Does the spec represent/preserve every Table 412 field and gate only empty/minimum-field validation pending OD-002 review of the covered set?
- [ ] CHK008 Does the spec avoid claiming local/global uniqueness enforcement where client-local data cannot establish it?
- [ ] CHK009 Does the Hashed Password section require/preserve caller-owned hash bytes and Timestamp, expose effective SHA-256 when omitted, and defer monotonicity checks/tests pending OD-003 review?
- [ ] CHK010 Does the SHA-256 default preserve the optional wire-field absence while exposing an unambiguous effective value?
- [ ] CHK011 Does the Attestation Credential require Nonce and Attestation Type and at least one evidence field, while accurately describing whether both are allowed?
- [ ] CHK012 Does the Nonce contract identify the server as source and require exact ID/value preservation?
- [ ] CHK013 Is the Attestation Capable Indicator tied to a measurable public API capability and separated from evidence generation or server acceptance?
- [ ] CHK014 Is OTP's lowercase “may” wording correctly resolved as informative (`KMIPKIT-CLAUSE-SPEC-9.11-008`, no requirement ID), with no library-wide replay/single-use state or normative client enforcement, and kept distinct from KMIPKIT-0007 OD-006?

## Security, interfaces, and scope

- [ ] CHK015 Are every secret-bearing value and every diagnostic path covered by explicit redaction requirements?
- [ ] CHK016 Does the zeroization statement limit the guarantee to initialized KMIPKit-owned bytes and exclude spare capacity, pre-transfer reallocation remnants, borrowed/stack/register copies, and caller/dependency/runtime copies?
- [ ] CHK017 Does the spec resolve OD-004 by making KMIPKIT-0008 permanently in-memory, with any future send path requiring its own approved feature and candidate-callsite/owner-through-transport lifecycle evidence?
- [ ] CHK018 Does the spec gate only execution integration on an explicit 0007 handoff covering inherited defaults, request/batch replacement, omission, precedence, and one Request Header Authentication for the whole batch, while leaving standalone in-memory models independent of execution APIs?
- [ ] CHK019 Does the spec avoid adding a local hash provider, attestation generator/verifier, Nonce generator, server, retry, or persistence behavior?
- [ ] CHK020 Does the spec avoid claiming OASIS official test-vector evidence when the catalog links no requirement-specific fixture?
- [ ] CHK021 Are OD-001, OD-002, OD-003, and OD-005 tracked as open gates, OD-004 and OD-006 recorded as resolved dispositions, and no unbounded `NEEDS CLARIFICATION` placeholders present?
- [ ] CHK022 Are the success criteria objective, independently verifiable, and linked to requirements without claiming implementation is complete?
- [ ] CHK023 Are the affected 0005, 0006, 0007, and 0010 handoffs explicit and consistent with their feature scopes?
- [ ] CHK024 Does the document distinguish requirement-quality approval from implementation completion and leave status Draft until the human maintainer changes it?

## Notes

- This checklist evaluates requirement quality, not code completion.
- Do not mark any item complete in this draft. The independent reviewer owns the markers.
- Reviewer findings should cite CHK ID and exact spec lines, then update the spec and rerun Spec Kit analysis.
