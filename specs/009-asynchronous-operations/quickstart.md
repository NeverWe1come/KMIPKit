# Review Quickstart: KMIP 2.1 Client Asynchronous Operations

This document is a behavior walkthrough for reviewers. Public function names are intentionally left to the approved implementation design; no unimplemented API is shown as executable code.

## Explicit pending follow-up

1. A caller submits a client-initiated operation with an Asynchronous Indicator that permits a Pending outcome.
2. The server may respond synchronously or return Pending with a server-generated Asynchronous Correlation Value.
3. The caller retains the pending outcome and decides whether to Poll, Cancel, Process, or stop. It reads the correlation value through the existing borrowed accessor and does not create an ordinary owned copy.
4. A selected action performs one bounded exchange. KMIPKit does not retry or issue another operation automatically.
5. If Poll reports Pending, the caller receives that outcome and explicitly decides whether to send another Poll. If Poll completes successfully, its original-operation payload remains generic until a typed model for that operation is available. If Poll reports terminal Failure, the caller receives its Result Reason and no payload.
6. If a transport error occurs, the returned delivery state indicates whether the request was not sent, possibly sent, or response reception had begun.

## Query outstanding requests

The caller may submit Query Asynchronous Requests with no filters or with correlation-value and operation filters. Until `KMIPKIT-DISC-039` is resolved, the response remains a generic TTLV value. It is not labeled as an authoritative typed interpretation of Table 286.

## Reviewer checks

- Compare source field order and required/optional markers with the pinned Tables 176, 177, 276, 278, 279, and 285.
- Verify that Process remains a distinct action and that Batch Order Option effects are described as server behavior.
- Verify that Cancel rejects Pending responses, even when the original request permitted asynchronous results.
- Verify that Pending values use the borrowed accessor without an ordinary unzeroized duplicate; Query filters and response bodies are not exposed in diagnostics. KMIPKit-owned copies are zeroized, while caller-owned Query input storage remains caller responsibility.
- Verify the source conflict and missing Process catalog requirement remain open and visible.
