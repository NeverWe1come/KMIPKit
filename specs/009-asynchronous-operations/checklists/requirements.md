# Specification Quality Checklist: KMIP 2.1 Client Asynchronous Operations

**Purpose**: Validate specification completeness and readiness for planning/implementation
**Created**: 2026-10-06
**Feature**: [spec.md](../spec.md)

**Review Ownership**: Reviewer-owned checklist. All items remain unchecked until reviewed against the exact specification revision. Author self-review is documented separately in the PR.

## Content Quality

- [ ] CHK001 The user stories describe caller-visible outcomes and priorities.
- [ ] CHK002 Mandatory sections are complete and contain no template placeholders.
- [ ] CHK003 Scope distinguishes client actions from server-only behavior.
- [ ] CHK004 Product boundaries (KMIP 2.1, TTLV, client initiated, no bindings/transport expansion) are preserved.

## Normative Accuracy and Traceability

- [ ] CHK005 Verify every OASIS citation names the pinned document, exact section, and table where applicable.
- [ ] CHK006 Verify Poll Pending semantics reconcile §6.1.38/Table 276 with §8.6/Table 399 without treating Poll as recursively asynchronous.
- [ ] CHK007 Verify Cancel request/response semantics, Pending-response rejection, and known/unknown Cancellation Result handling against Tables 176–178, §6.1.5, and §11.7.
- [ ] CHK008 Verify Process's required request field and empty response payload against Tables 278–280; confirm the missing catalog requirement ID stays visible.
- [ ] CHK009 Verify `KMIPKIT-DISC-039` exact caption conflict, generic response disposition, and blocked typed mapping.
- [ ] CHK010 Verify `KMIPKIT-REQ-SPEC-9.1-001`, `KMIPKIT-REQ-SPEC-9.19-002`, and `KMIPKIT-CLAUSE-SPEC-8.6-003` are assigned to the correct client/server duties.
- [ ] CHK011 Verify every feature FR/SC has a source or project-policy basis and executable verification plan.

## Security and Delivery Semantics

- [ ] CHK012 Verify Pending and Query-filter correlation bytes cannot appear in Debug, Display, errors, logs, or raw-body diagnostics.
- [ ] CHK013 Verify one-shot follow-up preserves `NotSent`, `PossiblySent`, and `ResponseStarted` states and does not retry.
- [ ] CHK014 Verify Poll Pending returns control to the caller and no wait, backoff, recursive Poll, or background work is described.
- [ ] CHK015 Verify Pending values use only the explicit borrowed accessor, no ordinary unzeroized duplicate exists, KMIPKit-owned Pending and Query-filter copies are zeroized within accepted ownership limits, and caller-owned Query input storage remains caller responsibility.
- [ ] CHK016 Verify Batch Order Option effects are described as possible server behavior, not a client guarantee.

## Readiness and Exclusions

- [ ] CHK017 Verify KMIPKIT-0006/0007/ADR-0014 dependencies match the active release branch.
- [ ] CHK018 Verify original-operation Poll completion remains generic unless its typed model is available.
- [ ] CHK019 Verify no transport constructor, server operation, binding, JSON/XML, profile, or cryptographic provider scope was added.
- [ ] CHK020 Verify the implementation gate requires approval of this exact spec and reviewer checklist.

## Notes

- The author pass found no unresolved template placeholders or `[NEEDS CLARIFICATION]` markers. OD-001 and OD-002 are explicit tracked review gates, not hidden assumptions.
- This checklist is intentionally unchecked for reviewer evaluation.
