# Contract: Timeout and Delivery

## Deadline policy

| Deadline | Default | Starts | Reset behavior |
|---|---:|---|---|
| Connect | 10 s | Before hostname resolution for the exchange | One deadline covers resolution, TCP candidate attempts, and TLS handshake. |
| Write | 30 s | When Hyper sender readiness or an application write/flush is first pending | Restarts only after positive write/flush progress. |
| Read | 30 s | When an application read is first pending | Restarts only after positive read progress. |
| Total | 60 s | At public exchange entry, before queue submission | Absolute monotonic deadline; never reset. |

Each timeout may be overridden per client and per exchange through the typed
client's `execute_with_options` family and the direct production adapter's
options-bearing byte-exchange method. Explicit unbounded is a distinct value
from a zero-duration immediate deadline. For every wait, use the earlier of
the active phase deadline and total deadline. Connect, write, read, and total
timeouts invalidate the active connection. `SendRequest::ready()` MUST occur
before dispatch commit and MUST be bounded by the active write and total
deadlines; expiry there is `NotSent`. The dispatch commit MUST occur only
after readiness succeeds.

The per-client worker uses Hickory's asynchronous resolver configured from
the platform's system DNS settings, with at most one retry after the initial
query (two attempts total), at most two concurrent nameserver requests per
query, at most 32 active DNS requests on each multiplexed upstream connection
(not an aggregate per-client limit), and a per-client response cache of at
most 128 entries. A lookup returns at most 16 address candidates across A/AAAA
results, preserving resolver order. Its effective timeout is capped by the
connect deadline. Returned A/AAAA addresses are
connect candidates for the one configured hostname; another candidate may be
tried only before request-dispatch commit and within the
connect deadline. Server-supplied alternate endpoints are never used. If the
deadline expires during resolution, KMIPKit returns within the exchange
deadline, reports `NotSent`, and sends no KMIP request. Canceling an exchange
cancels its caller-visible lookup future. Client shutdown drops owned async
lookup tasks and must not wait on a blocking OS resolver call.

The total deadline is created before queue submission. The queue holds at
most one pending exchange. The worker checks the deadline and cancellation
token immediately before every new connection. Exchange control then
atomically commits immediately before handing the request to the HTTP/TLS
writer. If cancellation wins before that commit, the caller receives
`NotSent` and the worker cannot write that request. If dispatch wins, later
failures are conservatively `PossiblySent` even if the writer accepts no
request bytes. Hyper may serialize headers and body together, so the
implementation does not infer a body boundary from bytes observed below
Hyper.

## Delivery-state mapping

| Event | Returned state |
|---|---|
| Local validation (including extension-registry provenance mismatch), DNS, TCP, TLS, queue, or other failure before request-dispatch commit | `NotSent` |
| Request-dispatch commit occurred; no response byte read, including HTTPS headers sent without a TTLV body | `PossiblySent` (conservative; commit does not prove any request byte reached TLS or the peer) |
| At least one decrypted HTTP response or raw TTLV response byte read, followed by any failure | `ResponseStarted` |

The typed client retains the immutable KMIPKIT-0012 `ClientConfiguration`
separately from transport configuration. Before constructing the outgoing
KMIP `RequestMessage`, encoding, or adapter invocation, it compares the
private registry provenance of every attached `ClientRequestMessageExtension` with
the registry owned by that retained configuration. A mismatch is sanitized as
`InvalidInput` with `NotSent`; it produces no outgoing request message, no
encoded request, and no adapter call. An extension validated by the retained
configuration continues through the existing typed request and encoding path.

For HTTPS, the dispatch commit occurs after `SendRequest::ready()` succeeds
and immediately before the request is handed to Hyper; it does not require
identifying a serialized header/body boundary. For raw
TLS, commit occurs immediately before the caller-supplied frame is handed to
the TLS writer. The delivery state and terminal finalization share one
linearization gate containing the delivery state and a `Finalized` bit (a
packed compare/exchange value or a lock). The TLS wrapper commits the first
positive decrypted read to that gate before returning bytes to Hyper. If a
decrypted response byte is observed before timeout finalization,
`ResponseStarted` takes precedence over the timeout's earlier snapshot; if
timeout finalization wins first, it cancels the request future/connection
driver and invalidates the connection before capturing the returned state.
No later I/O may revise a finalized result. A zero-byte read does not mean a
response started. Errors preserve only safe
phase and cause categories; they omit dependency error text, payloads,
endpoint host/path/query, and credential paths. The client never retries a
failed exchange; only a later distinct request may reconnect.

## Worker lifecycle

One private worker owns the Tokio runtime, resolver, and at most one
connection for one client. HTTPS may reuse a healthy connection; raw TLS
closes its connection after one frame. The public API remains synchronous.
The bounded command/result channel admits one active exchange and at most one
waiting exchange. Client shutdown signals the worker; the worker cancels
async I/O, invalidates/closes the connection, zeroizes KMIPKit-owned
initialized buffers, releases TLS/key state, and exits. Shutdown does not
wait on a blocking OS DNS resolver call.
