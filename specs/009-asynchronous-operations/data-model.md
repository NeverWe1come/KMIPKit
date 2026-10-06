
# Data Model: KMIP 2.1 Client Asynchronous Operations

All wire values use the existing generic TTLV model and source field order. Typed views validate known KMIP structure without discarding unknown enum values or extension subtrees. This document describes planned model boundaries; it does not authorize implementation before spec approval.

## Entities

### Asynchronous Correlation Value

- Opaque bytes returned by the server for an operation that is Pending.
- Distinct from Client Correlation Value and Unique Batch Item ID.
- No text decoding, normalization, token parsing, inferred token grammar, or application-level maximum is added. Existing `CodecLimits` still bound the enclosing message.
- Pending values are exposed only through KMIPKIT-0007's explicit borrowed accessor; KMIPKit-owned storage stays zeroizing and is never copied into ordinary unzeroized storage.
- All correlation values, including caller-supplied Query filters, are redacted in diagnostics. Caller-owned Query input storage remains the caller's responsibility; any KMIPKit-owned copy follows the accepted zeroization contract and is not ordinarily duplicated.

### Pending Operation

- The most recent Pending operation outcome, with exact correlation bytes and original request/batch association.
- Exposes those bytes only through KMIPKIT-0007's explicit borrowed accessor; callers cannot obtain an ordinary unzeroized owned duplicate through this model.
- Provides caller-visible data needed to choose a later Poll, Cancel, or Process action; does not send, loop, wait, schedule, retry, or cancel on drop.
- If a later Poll response remains Pending, the response's server-provided correlation value is retained for any next explicit call. Equality with the previous request token is not assumed unless the source explicitly requires it.

### Poll Request and Outcome

- Request: one required Asynchronous Correlation Value.
- Incomplete original operation: Result Status Pending, no original-operation response payload, and the Pending response correlation required by §8.6/Table 399.
- Completed original operation: expose the original operation's terminal status/reason and payload semantics. A successful operation returns its response payload, identical to the original operation's synchronous payload, represented as generic TTLV when its typed response model is not in scope. A terminal Failure exposes Result Reason and no response payload under the general Response Batch Item rules.
- Poll is not itself asynchronous, even when its synchronous response reports that the original operation remains Pending.

### Cancel Request and Outcome

- Request: one required Asynchronous Correlation Value.
- Successful response: the echoed value and required Cancellation Result Enumeration.
- Response mode: Cancel cannot itself be asynchronous. A Pending Cancel response is invalid even when the original request allowed Pending.
- Known Cancellation Result raw values 1 and 2 receive typed views for Canceled and Unable to Cancel; unknown values remain available unchanged.
- Response/error association uses the existing batch item rules.

### Process Request and Outcome

- Request: one required Asynchronous Correlation Value.
- Response payload: empty per §6.1.39/Table 279.
- Process result belongs to the Process request; it does not imply success of any later Poll or assert that other ordered batch items were unaffected.
- The general response model decides whether a Process response itself is Pending; no operation-specific synchronous-only constraint is invented.

### Query Asynchronous Requests

- Request filters: optional Asynchronous Correlation Values structure with zero or more values; optional Operations structure with zero or more operation values. Preserve order, repetition, and unknown values. Correlation filter values are redacted. Caller-owned input storage remains caller-owned; any KMIPKit-owned copy is zeroized and is not copied into ordinary unzeroized storage.
- Response: generic TTLV while Table 286's caption and placement remain disputed in `KMIPKIT-DISC-039`.
- The §7.2/Table 353 Asynchronous Request model is not exposed as a typed Query response until that conflict is reviewed.

## Relationships

```text
Original request ── Pending response ──> Pending Operation
                                         ├── one explicit Poll ──> Pending or original result
                                         ├── one explicit Cancel ──> Cancellation Result
                                         └── one explicit Process ──> Process result

Caller ── Query Asynchronous Requests(filters) ──> generic TTLV response
```

Each arrow that sends a request is a distinct call and one transport exchange. No arrow is automatic.

## Validation invariants

- Required correlation fields are present in Poll/Cancel/Process requests.
- Request and response item identifiers follow KMIPKIT-0007 association rules.
- Poll Pending permits the §6.1.38 no-payload exception while retaining the Pending correlation required by §8.6/Table 399.
- Poll completion is not decoded as an unrelated operation's type.
- Cancel echo must associate to its request; unknown cancellation enumeration values survive.
- Process success payload contains no fields; error and Pending cases follow general response semantics.
- Query filters and opaque response tree survive round trips; typed Table 286 semantics stay gated.
- All decode paths use configured `CodecLimits` before allocation.
