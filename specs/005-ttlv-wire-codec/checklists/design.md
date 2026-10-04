# Design Checklist: KMIP TTLV Wire Codec

**Purpose**: Review design decisions and implementation readiness before coding.
**Created**: 2026-10-04
**Feature**: [spec.md](../spec.md)
**Review Ownership**: Reviewer-owned; a checked item means its criterion has been independently verified, not that implementation is complete.

## Normative coverage

- [x] Pinned local KMIP v2.1 source is used; no upstream files are modified.
- [x] Tag, Type, Length, Value, and Padding clauses are referenced by exact section.
- [x] All eleven Item Types have a type-specific representation and vector expectation.
- [x] Length accounting distinguishes Structure, Big Integer, fixed 4-byte values, and strings/bytes.
- [x] Unknown Enumeration values, mask bits, and accepted extension Tags retain raw data.
- [ ] The normative inventory names typed protocol specification(s) that own schema-order enforcement; this codec only preserves supplied Structure order.

## Parser and security design

- [x] Input and declared lengths are checked before allocation based on them.
- [x] Default message/depth/element limits and counting semantics are explicit.
- [x] Errors contain no raw body or value payload.
- [x] Unsupported Type behavior, exact-one-item behavior, and trailing data are explicit.
- [x] Padding octet acceptance follows the source; canonical encoder fill is labeled as project policy.
- [ ] Proposed ADR-0011 has been reviewed/accepted, resolving `KMIPKIT-DISC-037`.
- [x] Depth configurability is bounded to 0–64, matching the 004 model hard cap and preserving a configurable decoder limit.

## Architecture and compatibility

- [x] Codec stays in `kmipkit-ttlv`; transport and operation/schema logic stay outside.
- [x] Public API is proposed as per-call immutable options with no global mutable state.
- [x] No new external dependency or unsafe code is required by the design.
- [x] Implementation branch is updated from the merged 004 release and the proposed contract is checked against its actual public APIs.

## Implementation evidence

- [ ] Separate Red, Green, and Refactor commits are planned for encoder, decoder, and limits.
- [ ] OASIS vectors, malformed input, boundaries, properties, and fuzz targets are planned.
- [ ] Catalog traceability, generated artifacts, docs, CI, and coverage gates are assigned.
- [ ] Independent QA and security reviews are complete before the implementation PR is ready.

## Notes

- The unchecked ADR acceptance, schema-order assignment, and implementation evidence items are gates. This design does not claim 100% traceability while the inventory assignment is open.
