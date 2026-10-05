# Requirements Checklist: KMIPKIT-0008 Credentials and Attestation

**Purpose**: Reviewer-owned requirements-quality checklist for the credential and attestation specification.<br>
**Created**: 2026-10-05<br>
**Feature**: [spec.md](../spec.md)<br>
**Review Ownership**: Independent reviewer / human maintainer. All items remain unchecked until that review occurs.

## Normative coverage and applicability

- [ ] CHK001 Does every OASIS requirement cite the pinned Specification revision, exact section/table, and stable catalog requirement or element ID?
- [ ] CHK002 Does the spec separate client obligations from server authentication duties, especially §9.4's “all credentials must be satisfied” sentence?
- [ ] CHK003 Does the spec represent every named Table 442 Credential Type and preserve unknown Credential Type and enum values?
- [ ] CHK004 Does the spec map each known Credential Value to the exact required/optional fields and encoding types in Tables 411–416?
- [ ] CHK005 Are all field-order and repeated-Credential rules explicit and testable without inventing rules for unknown extensions?
- [ ] CHK006 Does the spec distinguish base KMIP credential construction from profile-specific authentication applicability and claims assigned to 0010?

## Domain ambiguity and defaults

- [ ] CHK007 Does the Device Credential requirement resolve which Table 412 fields satisfy “at least one field,” or clearly gate implementation pending OD-002?
- [ ] CHK008 Does the spec distinguish identifier uniqueness semantics from client-verifiable local validation?
- [ ] CHK009 Does the Hashed Password section define caller ownership for hash bytes and timestamp, or clearly gate monotonic timestamp enforcement pending OD-003?
- [ ] CHK010 Does the SHA-256 default preserve the optional wire-field absence while exposing an unambiguous effective value?
- [ ] CHK011 Does the Attestation Credential require Nonce and Attestation Type and at least one evidence field, while accurately describing whether both are allowed?
- [ ] CHK012 Does the Nonce contract identify the server as source and require exact ID/value preservation?
- [ ] CHK013 Is the Attestation Capable Indicator tied to a measurable public API capability and separated from evidence generation or server acceptance?
- [ ] CHK014 Is OTP's lowercase single-authentication wording distinguished from an uppercase normative client requirement unless the catalog resolves it?

## Security, interfaces, and scope

- [ ] CHK015 Are every secret-bearing value and every diagnostic path covered by explicit redaction requirements?
- [ ] CHK016 Does the zeroization statement limit the guarantee to initialized KMIPKit-owned bytes and exclude spare capacity, pre-transfer reallocation remnants, borrowed/stack/register copies, and caller/dependency/runtime copies?
- [ ] CHK017 Does the spec state that this feature adds no production credential writer/send path, and require any later path to have separate approval plus candidate-callsite/owner-through-transport lifecycle evidence?
- [ ] CHK018 Are inherited defaults, request/batch replacement, and explicit omission behavior at the 0007 boundary defined or explicitly gated pending OD-005?
- [ ] CHK019 Does the spec avoid adding a local hash provider, attestation generator/verifier, Nonce generator, server, retry, or persistence behavior?
- [ ] CHK020 Does the spec avoid claiming OASIS official test-vector evidence when the catalog links no requirement-specific fixture?
- [ ] CHK021 Are all OD items tied to a blocking task, and are there no unbounded `NEEDS CLARIFICATION` placeholders?
- [ ] CHK022 Are the success criteria objective, independently verifiable, and linked to requirements without claiming implementation is complete?
- [ ] CHK023 Are the affected 0005, 0006, 0007, and 0010 handoffs explicit and consistent with their feature scopes?
- [ ] CHK024 Does the document distinguish requirement-quality approval from implementation completion and leave status Draft until the human maintainer changes it?

## Notes

- This checklist evaluates requirement quality, not code completion.
- Do not mark any item complete in this draft. The independent reviewer owns the markers.
- Reviewer findings should cite CHK ID and exact spec lines, then update the spec and rerun Spec Kit analysis.
