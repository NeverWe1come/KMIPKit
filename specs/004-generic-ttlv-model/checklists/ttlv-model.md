# Protocol Model Checklist: KMIP Generic TTLV Value Model

**Purpose**: Review whether the value-model requirements are complete, clear, consistent, and ready for implementation.
**Created**: 2026-10-04
**Feature**: [spec.md](../spec.md)

**Note**: This custom checklist evaluates the quality of the requirements, not implementation completion.
**Review Ownership**: This is a reviewer-owned requirements-quality artifact. Mark an item `[x]` only after a reviewer confirms the criterion is satisfied.
**Marker Semantics**: `[x]` means reviewed and satisfactory as a requirement; it does not mean code or tests are complete.

## Scope and protocol coverage

- [ ] CHK001 Does the spec consistently distinguish this in-memory model from TTLV wire encoding/decoding, wire validity, protocol validity, and operation semantics?
- [ ] CHK002 Are all eleven KMIP 2.1 Item Types named and mapped to the exact value widths, signedness, or payload representation required by §10.1.2 and §11.23?
- [ ] CHK003 Does the tag-allocation requirement identify how exact individual entries, Reserved entries, residual ranges, and the §11.56 Extensions range are treated, while clearly labeling precedence as project policy?
- [ ] CHK004 Are requirements explicit that unknown Enumeration values, unknown extension tags, bit patterns, repeated tags, and Structure child order are preserved without implying schema validity?

## Security and observable behavior

- [ ] CHK005 Does the spec define exactly which payloads are secret-wrapped and how nested Structure payloads inherit the same ownership and zeroization behavior?
- [ ] CHK006 Does the closure-scoped exposure contract clearly separate borrowed-reference lifetime from caller-created copies, formatting, and disclosure?
- [ ] CHK007 Is the zeroization guarantee bounded to current KMIPKit-owned storage, including backing capacity where applicable, and does it avoid promising erasure of old allocations or external runtime copies while preventing Structure growth from relocating secret payloads?
- [ ] CHK008 Are the required redaction surfaces named precisely (`Debug`, optional `Display`, model errors, serializer traits, and implicit copy traits), with transport/client logging kept in its owning layer?
- [ ] CHK009 Does the spec prescribe a safe Drop-path probe without unsafe code or reads from freed memory?

## Validation and traceability

- [ ] CHK010 Does each normative requirement cite an exact pinned OASIS document clause and map to stable requirement identifiers?
- [ ] CHK011 Are acceptance outcomes measurable for value preservation, tag rejection, Structure order, diagnostic redaction, and zeroization boundaries?
- [ ] CHK012 Are prerequisites and exclusions explicit, especially ADR-0010 acceptance, KMIPKIT-0003 implementation merge, and the later codec feature?

## Notes

- Leave items unchecked until requirements review is performed.
- Checklist answers evaluate specification quality; implementation and test evidence belong in the PR and traceability inventory.
- The built-in `checklists/requirements.md` name is reserved for Spec Kit's specify/clarify workflow; this custom requirements-quality checklist is `ttlv-model.md`.
