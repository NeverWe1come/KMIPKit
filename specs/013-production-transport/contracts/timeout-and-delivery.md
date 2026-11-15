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

The per-client worker resolves with `ToSocketAddrs` inside Tokio's blocking
pool. It acquires a permit from the shared 32-permit governor for the loaded
KMIPKit library instance before submitting the job and retains the permit in
the closure until the native call exits. If capacity is unavailable, the
attempt fails immediately as `NotSent`. It makes at most one lookup call for
the configured host and port per connection-establishment attempt and retains
at most the first 16 returned addresses in operating-system order and tries
them sequentially. The OS
owns address-family behavior, DNS packet retries, upstream concurrency,
caching and invalidation, hosts/search rules, and split-DNS/VPN routing;
KMIPKit does not maintain a DNS cache or claim numeric bounds for those OS
behaviors. Returned addresses are alternatives for only the configured
endpoint and may be tried only before dispatch and within connect and total
deadlines. Server-supplied alternate endpoints are never used.

If a caller times out or cancels while native resolution is running, KMIPKit
returns `NotSent` without waiting for that call; the call and OS DNS traffic
may continue and retain a governor permit until the call exits. An unstarted
blocking job is aborted when possible. Any late result is discarded and
cannot start a TCP/TLS candidate connection or dispatch KMIP data. Client
shutdown must not wait indefinitely for an already-started native resolver
call. See [ADR-0016](../../../docs/adr/0016-native-system-name-resolution.md).

The total deadline is created before queue submission. The queue holds at
most one pending exchange. The worker checks the deadline and cancellation
token before resolver admission, after a resolver result, before every
candidate TCP/TLS connection, and immediately before request dispatch. The
resolver uses `ToSocketAddrs` inside Tokio `spawn_blocking` after acquiring a
shared permit for the loaded library instance. A fail-fast admission failure
is `NotSent`. The permit remains held until the native call exits. A caller
timeout or cancellation returns without waiting for a started native call;
its late address list is discarded and cannot open a candidate connection or
dispatch KMIP data. OS DNS traffic already in progress may continue. Worker
shutdown does not wait indefinitely for a started native lookup. See
[ADR-0016](../../../docs/adr/0016-native-system-name-resolution.md) for
platform-owned retries, caching, routing, and the 16-address limit.

Exchange control atomically commits immediately before handing the request to
the HTTP/TLS writer. If cancellation wins before that commit, the caller
receives `NotSent` and the worker cannot write that request. If dispatch wins,
later failures are conservatively `PossiblySent` even if the writer accepts
no request bytes. Hyper may serialize headers and body together, so the
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
initialized buffers, releases TLS/key state, and exits without waiting
indefinitely for an already-started native resolver call. Such a call may
continue in the blocking pool, retaining its permit, and its result is
discarded.
