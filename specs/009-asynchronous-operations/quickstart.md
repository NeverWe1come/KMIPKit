# Review Quickstart: KMIP 2.1 Client Asynchronous Operations

This document is a behavior walkthrough for reviewers. Public function names are intentionally left to the approved implementation design; no unimplemented API is shown as executable code.

## Explicit pending follow-up

1. A caller submits a client-initiated operation with an Asynchronous Indicator that permits a Pending outcome.
2. The server may respond synchronously or return Pending with a server-generated Asynchronous Correlation Value.
3. The caller retains the pending outcome and decides whether to Poll, Cancel, Process, or stop.
4. A selected action performs one bounded exchange. KMIPKit does not retry or issue another operation automatically.
5. If Poll reports Pending, the caller receives that outcome and explicitly decides whether to send another Poll. If Poll completes, its original-operation payload remains generic until a typed model for that operation is available.
6. If a transport error occurs, the returned delivery state indicates whether the request was not sent, possibly sent, or response reception had begun.

## Query outstanding requests

The caller may submit Query Asynchronous Requests with no filters or with correlation-value and operation filters. Until `KMIPKIT-DISC-039` is resolved, the response remains a generic TTLV value. It is not labeled as an authoritative typed interpretation of Table 286.

## Reviewer checks

- Compare source field order and required/optional markers with the pinned Tables 176, 177, 276, 278, 279, and 285.
- Verify that Process remains a distinct action and that Batch Order Option effects are described as server behavior.
- Verify that Pending tokens and response bodies are not exposed in diagnostics.
- Verify the source conflict and missing Process catalog requirement remain open and visible.
