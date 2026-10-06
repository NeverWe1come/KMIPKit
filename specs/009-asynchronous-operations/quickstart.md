# Review Quickstart: KMIP 2.1 Client Asynchronous Operations

This document is a behavior walkthrough for reviewers using the implemented Rust API. The client has no production constructor in this feature, so the example accepts an already available `Client` reference.

## Explicit pending follow-up

1. A caller submits a client-initiated operation with an Asynchronous Indicator that permits a Pending outcome.
2. The server may respond synchronously or return Pending with a server-generated Asynchronous Correlation Value.
3. The caller retains the pending outcome and decides whether to Poll, Cancel, Process, or stop. It reads the correlation value through the existing borrowed accessor and does not create an ordinary owned copy.
4. The caller selects `Client::execute_poll`, `Client::execute_cancel`, or `Client::execute_process`. Each call performs one bounded exchange. KMIPKit does not retry or issue another operation automatically.
5. If Poll reports Pending, the caller receives that outcome and explicitly decides whether to send another Poll. If Poll completes successfully, its original-operation payload remains generic until a typed model for that operation is available. If Poll reports terminal Failure, the caller receives its Result Reason and no payload.
6. If a transport error occurs, the returned delivery state indicates whether the request was not sent, possibly sent, or response reception had begun.

## Query outstanding requests

The caller may submit Query Asynchronous Requests with no filters or with correlation-value and operation filters. Until `KMIPKIT-DISC-039` is resolved, the response remains a generic TTLV value. It is not labeled as an authoritative typed interpretation of Table 286.

`Client::execute_query_async_requests` is the explicit one-exchange entry point. The `ClientOperationOutcome` exposes its payload only through a callback-scoped generic structure view. `Client::execute_process` accepts the caller-selected Asynchronous Indicator for the Process operation; a Pending result returns immediately for a later caller decision.

## Compilable Poll example

The same example is a doctest on `Client::execute_poll` and is compiled by the client crate's Rust documentation tests:

```rust
use kmipkit_client::{Client, ClientError};
use kmipkit_protocol::PollRequest;
use kmipkit_ttlv::codec::CodecLimits;

fn poll_once(
    client: &mut Client,
    correlation: &[u8],
) -> Result<bool, ClientError> {
    let outcome = client.execute_poll(
        PollRequest::new(correlation),
        &CodecLimits::defaults(),
    )?;
    if outcome.is_pending() {
        let _ = outcome.with_asynchronous_correlation_value(|bytes| bytes.len());
    }
    Ok(outcome.is_pending())
}
```

## Reviewer checks

- Compare source field order and required/optional markers with the pinned Tables 176, 177, 276, 278, 279, and 285.
- Verify that Process remains a distinct action and that Batch Order Option effects are described as server behavior.
- Verify that Cancel rejects Pending responses, even when the original request permitted asynchronous results.
- Verify that Pending values use the borrowed accessor without an ordinary unzeroized duplicate; Query filters and response bodies are not exposed in diagnostics. KMIPKit-owned copies are zeroized, while caller-owned Query input storage remains caller responsibility.
- Verify the source conflict and missing Process catalog requirement remain open and visible.
