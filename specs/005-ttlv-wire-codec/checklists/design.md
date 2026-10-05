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
- [ ] Every applicable client 1.0 Structure has approved typed-spec ownership plus implementation and executable order-verification references; this codec only preserves supplied Structure order.

## Parser and security design

- [x] Input and declared lengths are checked before allocation based on them.
- [x] Default message/depth/element limits and counting semantics are explicit.
- [x] Errors contain no raw body or value payload.
- [x] Proposed private encoder output uses an internal borrow-only owner that zeroizes the initialized encoded byte range before deallocation/owner drop; spare or otherwise uninitialized `Vec` capacity is outside the guarantee unless it is explicitly initialized and its cleanup is verified; no encoded owner is public.
- [x] The draft identifies the secret-bearing protocol-wire encoding as a conditional proposal and preserves the current prohibition until approval.
- [x] No public encoder or encoded-byte owner is proposed; private encoded output cannot be cloned, formatted, generally serialized, mutated, or extracted as an ordinary byte vector.
- [x] Unsupported Type behavior, exact-one-item behavior, and trailing data are explicit.
- [x] Padding octet acceptance follows the source; canonical encoder fill is labeled as project policy.
- [ ] Proposed ADR-0011 has been reviewed/accepted, resolving `KMIPKIT-DISC-037`.
- [ ] The three FR-013 human approvals are distinct: ADR-0012 acceptance, approval of this feature specification, and approval of the enforceable boundary design. None is marked satisfied by this checklist.
- [ ] Human review approves the proposed future client boundary: the first client feature/spec defines a private child writer requiring an `OperationEncodingPermit` whose type/private constructor belong to the module containing `Client::execute`; execute is its sole mint site and sole production writer callsite, with parent-restricted writer visibility. Execute accepts only a closed set of concrete typed KMIP requests, not `Item`, raw body bytes, or a caller-implementable conversion trait. KMIPKIT-0005 does not implement this API, permit, or callsite. The first client feature/spec owns the exact public request/limit API, exact-one callsite/mint audit, and typed-input/permit/owner-through-transport test. If review rejects this boundary or it cannot be enforced, no secret-bearing request path is approved or implemented.
- [x] Depth configurability is bounded to 0–64, matching the 004 model hard cap and preserving a configurable decoder limit.

## Architecture and compatibility

- [x] Public generic TTLV values and bounded decoding stay in `kmipkit-ttlv`; KMIPKIT-0005's byte-producing writer is private to `kmipkit-client` and has no production callsite. The future client feature adds the execute-owned permit and closed typed-request path.
- [x] Public `CodecLimits` is proposed as immutable per-call options with read-only value getters, no setters/builders, and no global mutable state; the private client writer receives the same borrowed instance per operation.
- [x] No new external dependency or unsafe code is required by the design.
- [x] Implementation branch is updated from the merged 004 release and the proposed contract is checked against its actual public APIs.

## Implementation evidence

- [ ] Separate Red, Green, and Refactor commits are planned for encoder, decoder, and limits.
- [ ] T005/T009 Red tests use private `#[cfg(test)]` candidate seams/fixtures, not production decoder or `CodecLimits` symbols that do not yet exist; they compile and fail behavior assertions. T006/T010 promote the implementations and include external-crate tests of the public APIs.
- [ ] KMIPKIT-0005 adds no `Client::execute`, permit type/constructor, or production writer callsite; T015 verifies their absence. The first client feature/spec owns the exact-one production callsite/mint audit. Its candidate PR must contain both the sole production callsite and its owner-through-transport integration test; CI must run and pass that test against the candidate callsite before merge, enablement, or release. Until then, the release branch must contain no production callsite or secret-bearing send.
- [ ] OASIS vectors, malformed input, decoder/private-encoder default and configured byte/depth/count/U32 boundaries, properties, and fuzz targets are planned.
- [ ] Catalog traceability, generated artifacts, docs, CI, and coverage gates are assigned.
- [ ] Independent QA and security reviews are complete before the implementation PR is ready.

## Notes

- The unchecked ADR acceptance and implementation evidence items are implementation gates. The schema-order assignment and its implementation/verification evidence remain a separate 1.0 traceability gate; this design does not claim 100% traceability while that coverage is open.
