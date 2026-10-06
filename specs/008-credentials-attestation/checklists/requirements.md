# Requirements Checklist: KMIPKIT-0008 Credentials and Attestation

**Purpose**: Reviewer-owned requirements-quality checklist for the credential and attestation specification.<br>
**Created**: 2026-10-05<br>
**Feature**: [spec.md](../spec.md)<br>
**Review Ownership**: Independent reviewer / human maintainer. The independent reviewer owns the markers; items remain unchecked until the exact-revision review occurs.

## Normative coverage and applicability

- [ ] CHK001 Does every OASIS requirement cite the pinned Specification revision, exact section/table, and stable catalog requirement or element ID?
- [ ] CHK002 Does the spec identify §9.4's lowercase “must” against §1.2's uppercase RFC 2119 keywords, avoid client testing/enforcement of “all Credentials satisfied,” and accurately record the accepted catalog correction, open `KMIPKIT-DISC-041`, and lack of explicit catalog-owner sign-off?
- [ ] CHK003 Does the spec represent every named Table 442 Credential Type and preserve unknown Credential Type and enum values?
- [ ] CHK004 Does the spec map each known Credential Value to the exact required/optional fields and encoding types in Tables 411–416?
- [ ] CHK005 Are all field-order and repeated-Credential rules explicit and testable without inventing rules for unknown extensions?
- [ ] CHK006 Does the spec distinguish base KMIP credential construction from profile-specific authentication applicability and claims assigned to 0010?

## Domain ambiguity and defaults

- [ ] CHK007 Does the spec preserve all six Table 412 fields, reject a Device with none present, avoid imposing non-empty text, and keep uniqueness separate?
- [ ] CHK008 Does the spec retain the caller's OASIS uniqueness obligation for the four named identifiers, leave the source's comparison scope unspecified, and avoid claiming that KMIPKit verifies or enforces uniqueness from client-local data?
- [ ] CHK009 Does the Hashed Password section require/preserve caller-owned hash bytes and Timestamp, expose effective SHA-256 when omitted, and defer monotonicity checks/tests pending OD-003 review?
- [ ] CHK010 Does the SHA-256 default preserve the optional wire-field absence while exposing an unambiguous effective value?
- [ ] CHK011 Does the Attestation Credential require Nonce and Attestation Type and at least one evidence field, while accurately describing whether both are allowed?
- [ ] CHK012 Does the Nonce contract identify the server as source and require exact ID/value preservation?
- [ ] CHK013 Is the Attestation Capable Indicator emitted by the existing execute header builder from the public API capability, with no new writer, Authentication selection, Credential payload, evidence-generation, or server-acceptance claim?
- [ ] CHK014 Is OTP's lowercase “may” wording correctly resolved as informative (`KMIPKIT-CLAUSE-SPEC-9.11-008`, no requirement ID), with no library-wide replay/single-use state or normative client enforcement, and kept distinct from KMIPKIT-0007 OD-006?

## Security, interfaces, and scope

- [ ] CHK015 Are every secret-bearing value and every diagnostic path covered by explicit redaction requirements?
- [ ] CHK016 Does the zeroization statement limit the guarantee to initialized KMIPKit-owned bytes and exclude spare capacity, pre-transfer reallocation remnants, borrowed/stack/register copies, and caller/dependency/runtime copies?
- [ ] CHK017 Does the spec keep Credential/Authentication values in-memory, limit execute integration to the non-secret indicator on the existing writer, and require a separate approved feature plus candidate-callsite/owner-through-transport evidence for any future secret-bearing send path?
- [ ] CHK018 Does the spec gate only future execution integration on an explicit 0007 handoff covering inherited defaults, request/batch replacement, omission, precedence, and one Request Header Authentication for the whole batch, while leaving standalone in-memory models independent of execution APIs?
- [ ] CHK019 Does the spec avoid adding a local hash provider, attestation generator/verifier, Nonce generator, server, retry, or persistence behavior?
- [ ] CHK020 Does the spec avoid claiming OASIS official test-vector evidence when the catalog links no requirement-specific fixture?
- [ ] CHK021 Are OD-001, OD-003, and OD-005 tracked as open decisions with bounded effects; OD-002, OD-004, and OD-006 recorded as resolved dispositions; and no unbounded `NEEDS CLARIFICATION` placeholders present?
- [ ] CHK022 Are the success criteria objective, independently verifiable, and linked to requirements without claiming implementation is complete?
- [ ] CHK023 Are the affected 0005, 0006, 0007, and 0010 handoffs explicit and consistent with their feature scopes?
- [ ] CHK024 Does the document distinguish requirement-quality review from implementation completion and state feature status and delegated authorization unambiguously?

## Notes

- This checklist evaluates requirement quality, not code completion.
- Mark an item complete only when the independent reviewer confirms the requirement-quality criterion against the exact specification revision; this does not mean implementation is complete.
- Reviewer findings should cite CHK ID and exact spec lines, then update the spec and rerun Spec Kit analysis.
