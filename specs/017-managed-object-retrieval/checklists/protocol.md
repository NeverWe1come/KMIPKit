# Protocol Requirements Checklist: KMIP 2.1 Get and Locate

**Purpose**: Review normative fidelity and requirement quality before approving the feature specification.<br>
**Created**: 2026-10-09<br>
**Feature**: [spec.md](../spec.md)<br>
**Review ownership**: Reviewer-owned. Leave every item unchecked until reviewed.

## Normative completeness

- [ ] CHK001 Does the feature identify all 15 catalog rows as 2 Get plus 13 Locate and avoid describing all 15 as proven client obligations? [Completeness, Spec §Normative scope]
- [ ] CHK002 Are request, success-response, and error payload rules linked to exact OASIS sections and tables? [Traceability, Spec §Normative scope]
- [ ] CHK003 Does the Get format text apply KMIPKIT-DEC-003 without claiming local PKCS#12 container validation or server output conformance? [Clarity, Spec §Inventory Dispositions]
- [ ] CHK004 Does the case table separate catalog IDs, official labels, profile cases, fixture availability, and mapping confidence? [Evidence, traceability.md]

## Actor and behavior boundaries

- [ ] CHK005 Are server-only ID Placeholder, group-selection, and Storage Status behavior explicitly distinguished from client forwarding and preservation? [Consistency, Spec FR-007 to FR-010]
- [ ] CHK006 Is Table 247 field order explicit and consistent in the spec, model, contract, vectors, and tasks? [Consistency, Spec US2 AC-1]
- [ ] CHK007 Do negative tests avoid attributing a server MUST NOT response obligation to client code? [Negative requirement, traceability.md]

## Security and acceptance quality

- [ ] CHK008 Are Any Object payloads opaque, redacted, and zeroized only while held in KMIPKit-owned storage? [Security, Spec FR-004]
- [ ] CHK009 Are malformed and over-limit response cases testable with sanitized diagnostics? [Coverage, Spec FR-012]
- [ ] CHK010 Are the two available PKCS#12 fixtures and five unavailable linked fixtures distinguished, with no official pass claim before full-case execution? [Measurability, Spec FR-013]
- [ ] CHK011 Does the feature avoid claiming profile support, cross-language parity, two-server interoperability, or certification? [Boundary, Spec Out of Scope]
