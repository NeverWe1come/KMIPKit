# Specification Quality Checklist: KMIP 2.1 Managed-Object State Transitions

**Purpose**: Validate that this draft is bounded, testable, and tied to the approved KMIPKit roadmap and OASIS KMIP 2.1 sources.
**Created**: 2026-10-09
**Feature**: [spec.md](../spec.md)

**Review Ownership**: This is a reviewer-owned requirements-quality artifact. Items remain unchecked until independent review confirms them. Author self-checks and generated tests do not count as approval.

## Content Quality

- [ ] Scope names exactly four client-to-server operations and cites each request, response, and error table.
- [ ] Scenarios explain the caller value and keep server object-state behavior server-authoritative.
- [ ] The specification separates OASIS normative requirements from product/API behavior and assumptions.
- [ ] All mandatory sections are complete and no template placeholders remain.

## Requirement Completeness

- [ ] Every functional requirement is testable and uses a stable KMIPKIT-0018 identifier.
- [ ] Every applicable catalog client requirement is named; server-only clauses are not recast as client obligations.
- [ ] Success criteria are measurable and map to acceptance scenarios.
- [ ] Recover Pending, explicit Poll/Get, request omission, batch ID Placeholder, malformed success, failure, and delivery-state cases are defined.
- [ ] Scope, dependencies, official-test evidence limits, and exclusions are explicit.

## Feature Readiness

- [ ] All four operation families have independently testable acceptance scenarios.
- [ ] The criteria forbid automatic retry, polling, follow-up Get, local state mutation, and unsupported server guarantees.
- [ ] The scope fits KMIPKit 1.0 boundaries and does not add a new architecture decision.
- [ ] Normative references and catalog actor classifications agree with the pinned local sources.

## Notes

- Static source evidence is recorded in the draft; independent checklist approval remains outstanding.
- Activate and Destroy have client request/response tables but their catalog prose clauses are classified server-only.
- The catalog has no requirement-specific official test-case link for Archive or Recover; derived tests cannot claim official-case passes.
