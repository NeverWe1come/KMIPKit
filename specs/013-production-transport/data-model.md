# Data Model: Production TLS and HTTPS Transports

This feature adds configuration and lifecycle models around the existing
`Transport`, `TransportResponse`, and typed client request/response models.
Secret-bearing fields use owned secret wrappers and have redacted formatting.

## Entities

### `TransportConfig`

Validated, immutable input used to construct one production client.

| Field | Type / rule |
|---|---|
| `endpoint` | One absolute raw-TLS socket endpoint or HTTPS URI. Reject unsupported schemes, user information, fragments, invalid ports, and malformed targets. |
| `transport_kind` | `RawTtlvTls` or `HttpsTtlv`; determines allowed endpoint form. |
| `tls_identity` | Required client certificate chain and private key, supplied in-memory or by explicitly selected PEM/DER files. |
| `trust_source` | Explicit caller CA set or explicitly selected platform roots loaded with `rustls-native-certs`; no implicit source. |
| `crls` | Optional caller-supplied CRLs; no online retrieval. |
| `tls_server_name` | Optional verification name for an IP endpoint; affects certificate-name verification only. |
| `target_uri` | HTTPS origin-form request target (absolute path with optional query), default `/kmip`; cannot set authority or TLS name. Absent for raw TLS. |
| `timeouts` | Immutable client default `TimeoutPolicy`, with per-request overrides. |
| `codec_limits` | Existing typed-client limits; passed unchanged to exchange and used before encode/decode allocations. |
| `max_request_bytes` | Positive direct-transport request limit; defaults to 16 MiB. The typed client also enforces its effective `CodecLimits::max_message_bytes()` before calling the adapter. |

Construction parses all certificates, keys, trust roots, and CRLs, validates
the endpoint and configuration, creates the worker, and opens no socket. Key
files are read once; a constructed configuration stores parsed material, not
the source path.

`TransportConfig` contains no client extension registry or registry-provenance
state. It configures transport behavior only.

### Production typed `Client`

The production typed client retains two distinct immutable configuration
owners: the KMIPKIT-0012 `ClientConfiguration`, which owns that client's
extension registry snapshot, and the validated transport configuration used to
construct its production adapter and worker. Constructing a production
transport does not replace, copy into, or mutate the `ClientConfiguration`.

Before constructing an outgoing KMIP `RequestMessage`, encoding it, or calling
the adapter, `Client::execute` checks the private registry provenance of every
attached `ClientMessageExtension` against the registry owned by the retained
`ClientConfiguration`. A mismatch returns sanitized `InvalidInput` with
`DeliveryState::NotSent`; no outgoing KMIP request is constructed or encoded,
and the adapter is not invoked. A same-client extension passes this check and
uses the existing typed request, encoding, and exchange path without changing
its wire representation.

### `SecretInput`

Owned PEM or DER bytes or parsed key material used during configuration.
Initialized KMIPKit-owned bytes are zeroized on drop. PEM/DER is explicit;
unencrypted PKCS#8, PKCS#1, and SEC1 are accepted only when supported by the
selected provider. Encrypted keys and passphrase callbacks are rejected.
Caller copies and TLS/library/OS copies are outside KMIPKit's zeroization
guarantee.

### `TlsSessionStore`

Private rustls `ClientSessionStore` owned by one immutable client
configuration. It stores no more than 16 TLS 1.3 tickets, each with a local
monotonic insertion time, and returns a ticket for resumption only within one
hour and while rustls considers the server-provided ticket lifetime valid.
Ticket values are treated as highly sensitive and are never persisted or
shared. Resumption inherits the full handshake's identity and peer-validation
decisions; rebuilding the client creates an empty store.

### `TimeoutPolicy`

Client defaults and per-request overrides for `connect`, `write`, `read`, and
`total`. Each member is `Bounded(Duration)` or `Unbounded`, so zero and
unbounded cannot collapse into the same representation. Zero is an immediate
deadline. Client defaults are 10s/30s/30s/60s respectively.

`total` becomes an absolute monotonic `Instant` at public exchange entry and
travels with the worker command. `connect` covers name resolution, TCP, and TLS
setup. `write` and `read` limit one blocked application I/O operation at a
time; Hyper sender-readiness waits are also bounded by the write deadline.
Observed progress restarts that direction's inactivity clock. Each I/O wait
uses the earlier of its phase deadline and remaining total deadline.

### `RequestOptions`

Optional per-exchange timeout overrides shared by the typed client and direct
production-adapter byte exchanges. The typed client exposes an options-bearing
variant for every public operation method. Each concrete production adapter
exposes `exchange_with_options` alongside the existing `Transport::exchange`
default-policy path; omitted values inherit the client's `TimeoutPolicy`.
The absolute total deadline is computed before sending a command to the
worker, so bounded queue wait counts against the total budget. The worker
checks the deadline before connection and waits for Hyper sender readiness
before atomically committing dispatch and handing the request to the HTTP/TLS
writer; a readiness timeout remains `NotSent`. An expired queued exchange
returns `NotSent` and can never be dispatched later.

### `ExchangeControl`

Shared cancellation, dispatch, response-observation, and terminal-result state
for one caller exchange and its worker command. The exchange transitions from
`Queued` to `Preparing` to `WriterReady` to `DispatchCommitted`, then to
`Finished`; cancellation or timeout may finalize only through the same
linearization gate. That gate contains both delivery evidence and a
`Finalized` bit, implemented by a packed compare/exchange state or a lock.
Caller cancellation may win only before dispatch commits. The worker waits for
Hyper `SendRequest::ready()` before committing and immediately before handing
a request to the HTTP/TLS writer. If cancellation wins, no request write may
occur. If dispatch wins, the outcome is conservatively `PossiblySent` even if
the writer later reports zero accepted bytes. This avoids inferring HTTP body
progress from serialized bytes below Hyper, where headers and body may share
one I/O write. The TLS I/O wrapper records a positive decrypted read into the
same gate before returning those bytes to Hyper. If response observation
wins before terminal finalization, the state is `ResponseStarted`; if
finalization wins first, it cancels the request/connection driver and
invalidates the connection before capturing the returned state. No later I/O
may revise a finalized result. This linearization guarantees that an
exchange reported `NotSent` cannot be sent later.

### `ClientWorker`

Private per-client state running on one owned OS thread. It owns the current-
thread Tokio runtime, a Hickory resolver initialized from system configuration,
TLS config, one endpoint connection, and the bounded command/result channels.
It processes no more than one exchange for a client at a time and admits at
most one waiting exchange. The worker connects lazily, reuses a healthy HTTPS
connection, closes raw-TLS connections after one response frame, invalidates
connections after timeout or protocol/network error, and never resends an
exchange. It accepts shutdown, cancels pending async I/O, closes the socket,
releases secret configuration, and exits without waiting on a blocking OS
resolver call.

### `ConnectionState`

```text
Unconnected -> Connecting -> Ready -> InFlight -> Ready (HTTPS healthy)
                  |           |         |
                  +-----------+---------+----> Invalidated -> Connecting
                                      InFlight -> Closed -> Unconnected (raw TLS)

Any state -> Closing -> Closed
```

`InFlight` represents exactly one accepted exchange. A successful HTTPS
exchange returns to `Ready` only if Hyper reports a reusable connection; a
successful raw-TLS exchange closes the connection after one response frame
and returns to `Unconnected`. If an error occurs after dispatch commit, the
state becomes `Invalidated`; a later distinct exchange may establish a new
connection. No state transition replays the failed request.

### `DeadlineIo`

Private asynchronous I/O wrapper around the established TLS stream. It
enforces independent read/write inactivity deadlines plus the request's
absolute deadline and records request dispatch and first decrypted response
bytes through the shared `ExchangeControl` linearization gate. For HTTPS, it
waits for Hyper `SendRequest::ready()` before dispatch commit. The commit
occurs before the request is handed to Hyper or the raw TLS writer; response
observation advances delivery state before returning positive `poll_read`
bytes to Hyper and takes precedence over timeout if observed before terminal
finalization. The wrapper applies write deadlines to `poll_write` and
`poll_flush`; if it implements vectored writes, they receive the same deadline,
otherwise it reports vectored writes unsupported. Its state is scoped to one
exchange and is reset only when another exchange can safely use the HTTPS
connection. The HTTP parser input buffer is limited to 64 KiB, separately from
the body cap.

### `TransportResponse`

Existing public zeroizing owner returned to direct low-level callers. The
production adapters accumulate a bounded response into one KMIPKit-owned
allocation, avoid growth after response bytes are initialized, and wrap the
completed bytes. Partial response bytes are zeroized before errors return.
The typed client borrows the bytes for decode and drops the wrapper before
returning its typed result.

### `DeliveryState`

Existing enum with three observable states:

| State | Transition evidence |
|---|---|
| `NotSent` | Local validation, resolution, TCP connect, TLS handshake, queue, or other failure before request-dispatch commit. |
| `PossiblySent` | Request-dispatch commit has occurred and no response application byte has been observed; for HTTPS this includes HTTP headers sent without a TTLV body. |
| `ResponseStarted` | At least one response application byte is observed, even if subsequent parsing, framing, or decode fails. |

A zero-byte read does not enter `ResponseStarted`. Delivery-state updates are
monotonic: the first decrypted response byte advances the state to
`ResponseStarted`; if that observation wins before timeout-result finalization,
the returned error must carry `ResponseStarted`. Error formatting contains
only safe error categories, delivery state, phase, and transport kind. It
omits endpoint host/path/query, credential paths, and dependency error text.
