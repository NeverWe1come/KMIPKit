# Data Model: KMIP 2.1 Typed Client Execution

**Status**: Draft proposal; exact public names await KMIPKIT-0005/0006 acceptance.

## Typed Request and Response

### DiscoverVersionsRequest

- Operation: Discover Versions (`KMIPKIT-ELEM-OP-C2S-DISCOVER-VERSIONS`).
- Protocol version list: KMIPKit 1.0 sends exactly `(2, 1)` in decreasing preference order.
- Request payload: typed protocol-version values; no generic TTLV or caller-supplied bytes.
- Shared batch metadata: unique item ID when the batch has more than one item; IDs are pairwise distinct per the KMIPKIT-0006 project invariant.

### DiscoverVersionsResponse

- Operation: Discover Versions (`KMIPKIT-ELEM-OP-S2C-DISCOVER-VERSIONS`).
- Result: the existing KMIPKIT-0003 typed result contract.
- Payload: ordered list of server-supported versions; an empty list remains a valid Discover Versions result.
- Correlation: response item Operation and ID are checked against the originating request. A supplied request ID must be echoed.

## ClientBatch

- One or more closed `ClientRequest` enum values, each paired with an optional Unique Batch Item ID where KMIPKIT-0006 permits it.
- Common options: Batch Order Option, Batch Error Continuation Option, Asynchronous Indicator, and caller correlation value only where supported by the approved 0006 model. The 0007 Discover Versions API does not expose peer-visible Maximum Response Size because this operation is not classified as likely-large.
- Request Time Stamp: optional caller-supplied Date-Time, preserved exactly when present and omitted when absent. The supplied value is not included in diagnostics or errors and is never logged. This feature has no generated time-source interface; countdown-derived outgoing values are outside its scope under OD-004.
- Server Correlation Value: never present in client-initiated request types. Client Correlation Value remains distinct from every item ID and is not used for matching.
- Duplicate request IDs are rejected before the permit is minted.
- The peer-visible Maximum Response Size field is absent from Discover Versions requests; local response caps and `CodecLimits` always apply.
- A single-item batch rejects Batch Error Continuation Option.
- `Continue` and `Undo` are represented as caller-selected values; no server execution/rollback semantics are attached while `KMIPKIT-DISC-001` is unresolved.
- The result collection preserves item identity and exposes completed and Pending outcomes. It does not poll or wait automatically.

## RequestDeliveryState

Monotonic states reused from `kmipkit-transport`:

| State | Meaning |
|---|---|
| `NotSent` | No request bytes were written or may have reached the peer. |
| `PossiblySent` | At least one request byte may have reached the peer, but no response byte has begun. |
| `ResponseStarted` | At least one response byte has begun arriving. |

An error preserves the greatest observed state. A successful exchange returns its typed result and does not trigger a retry.

## Private Encoded Request Owner

- Owns the encoded request buffer only for the synchronous execute exchange.
- Exists only after typed validation and permit creation.
- Is borrowed by the transport for the whole exchange; it cannot be freed/reused while the transport is using it.
- Zeroizes initialized bytes when dropped.
- Is never exposed through public diagnostics, general-purpose encoding, persistence, or caller-selected raw-body APIs.

## Extension Handling

- Generic message extension fields are preserved by the accepted message model.
- This slice has no registered critical extension handlers.
- Any unrecognized critical extension rejects the message as required by OASIS §9.13; an unknown non-critical extension is retained as opaque data under KMIPKit project policy (OASIS permits processing it as absent), subject to decoder limits.
- The concrete registry and outbound extension-building surface are not resolved here; `KMIPKIT-0007-OD-002` blocks approval until an owner and acceptance criteria are selected.
