# Protocol Review Checklist: KMIP 2.1 Client Asynchronous Operations

**Purpose**: Reviewer verification of operation semantics and source dispositions
**Created**: 2026-10-06
**Feature**: [spec.md](../spec.md)

**Review Ownership**: Human protocol reviewer. Keep all boxes unchecked until reviewed.

## Poll and correlation lifecycle

- [ ] CHK001 Poll request includes the required original asynchronous correlation value (§6.1.38, Table 276).
- [ ] CHK002 Poll is not treated as asynchronous, while an incomplete original operation can yield Pending with no operation payload (§6.1.38 and §8.6, Table 399).
- [ ] CHK003 Completed Poll exposes the original operation response payload, not a Poll result schema (§6.1.38).
- [ ] CHK004 `KMIPKIT-REQ-SPEC-9.1-001` and `KMIPKIT-REQ-SPEC-9.19-002` direct exact server-provided values to subsequent Poll/Cancel as applicable.

## Cancel and Process

- [ ] CHK005 Cancel request and response payloads match Tables 176 and 177; error cases match Table 178.
- [ ] CHK006 Cancellation Result known values and forward-compatible unknown values are preserved (§11.7).
- [ ] CHK007 Process request's required correlation and empty response match Tables 278 and 279.
- [ ] CHK008 Process's possible effects on other batch items are accurately conditioned on Batch Order Option (§6.1.39).
- [ ] CHK009 The source's lack of an explicit Process response async prohibition is not turned into an invented synchronous-only requirement.

## Query Asynchronous Requests

- [ ] CHK010 Request filter structures match Table 285 and §7.1/Table 352.
- [ ] CHK011 The exact Table 286 “PKCS#11 Response Payload” caption and competing interpretations are recorded in `KMIPKIT-DISC-039`.
- [ ] CHK012 Generic Query response access is explicit and no typed mapping or conformance claim is implied while the discrepancy remains open.

## Traceability and boundaries

- [ ] CHK013 Process Table 278 missing catalog requirement ID remains a visible catalog workflow item; generated JSON is not changed in this feature.
- [ ] CHK014 No official test case, profile support, certification, or server behavior is claimed without evidence.
- [ ] CHK015 Source paths under `specification/oasis/kmip-2.1/upstream/` remain immutable.
