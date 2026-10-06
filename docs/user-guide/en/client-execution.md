# Typed client execution

This guide describes the current KMIPKIT-0007 execution foundation. It is not
a live-server setup guide: this feature provides no production `Client`
constructor and no TCP/TLS or HTTPS backend. The only transport
implementation is a deterministic test fake.

## Typed request boundary

The synchronous `kmipkit_client::Client::execute` API accepts a
`ClientBatch` containing only the closed `ClientRequest` variants. This first
slice has one operation: an explicitly requested client-to-server Discover
Versions request. It advertises exactly the KMIP 2.1 version pair (2, 1),
as specified by OASIS KMIP Specification v2.1 §6.1.16, Tables 211–213.
Passing generic TTLV `Item` or
`Structure` values, encoded message bytes, or caller-defined conversion hooks
to this API is not supported.

Discover Versions is an ordinary operation that the caller must request. The
client does not run it as a hidden preflight or use it to negotiate another
operation. The response reports versions only; it does not establish support
for other operations. An empty version list or a server's `Operation Not
Supported` result is returned as an ordinary typed outcome.

KMIPKit 1.0 is scoped to KMIP 2.1. This slice emits and accepts Protocol
Version 2.1 only under the product decision recorded for
[`KMIPKIT-DISC-022`](../../../specs/007-client-execution/spec.md); this is not a
claim that the implementation provides KMIP §9.16 same-major backward
compatibility. See [ADR-0002](../../adr/0002-kmip-21-release-scope.md) for the release
scope.

Although `Client::execute` and the request types are documented Rust APIs,
applications cannot construct a production `Client` in this feature. The
internal fake exists for deterministic execution tests only. No TLS/HTTPS
adapter or live-server readiness is provided here.

## Credentials and attestation capability

KMIPKIT-0008 provides typed, raw-preserving Credential and Authentication
values in `kmipkit-protocol`. Authentication values contain one or more
Credentials in caller-supplied order. These are in-memory models in this
increment; the request payload does not include Authentication or Credential
data.

A typed Hashed Password Credential requires the caller's Timestamp and hashed
bytes. Hashing Algorithm is optional. If omitted, the model reports effective
SHA-256 (raw Enumeration value `6`) while leaving the field absent in the
retained TTLV tree. KMIPKit does not calculate the hash.

A typed Device Credential must include at least one of Device Serial Number,
Network Identifier, Machine Identifier, or Media Identifier. The caller is
responsible for choosing an identifier or combination that is actually
unique. KMIPKit does not define the comparison scope or check actual
uniqueness. Password and Device Identifier fields do not
replace this requirement; their presence does not imply non-empty text.

The existing Request Header builder emits `Attestation Capable Indicator =
True` because the Rust API can construct an Attestation Credential. This
advertises construction capability only: it does not generate or verify
attestation evidence, submit a Credential, or predict server acceptance. There
is no per-request override. The value appears in the captured fake-transport
exchange; this feature still has no production client constructor or live
server transport.

KMIPKit does not log Credential contents; Debug, Display, and validation
diagnostics redact them. `SecretText` and `SecretBytes` zeroize initialized
bytes in the KMIPKit-owned current allocation when its owner is dropped;
conversion into TTLV moves ownership without cloning. Spare or uninitialized
capacity is covered only when initialized and cleanup is verified. This does
not erase caller-made copies, older allocations left by buffer growth, copies
made from callback-borrowed views, temporary stack/register copies, or copies
retained by foreign runtimes or dependencies.

## Request metadata and batch results

The optional Request Header Time Stamp is omitted by default. If supplied by
the caller, its KMIP Date-Time value is sent exactly as supplied. The client
does not generate a timestamp from a clock or countdown timer. Client-initiated
requests omit Server Correlation Value. Client Correlation Value, if present,
is independent metadata and never identifies a batch item.

For a multi-item batch, every request item has a Unique Batch Item ID. The
client associates each response with its request by that ID, even when the
server returns response items in a different order. Results are returned in
request order. A supplied ID must be echoed; a single-item response may omit
the ID when the request did. Duplicate, missing, unexpected, or mismatched
identities and mismatched operations fail as redacted protocol errors. Batch
Order Option controls server execution order; it does not select the response
matching rule.

Each valid item result is represented as completed or Pending. Pending is
accepted only when the request's effective Asynchronous Indicator permits it
and the response contains its required Asynchronous Correlation Value. The
correlation value is opaque: KMIPKit preserves its bytes and exposes them only
through an explicit borrowed accessor. It is redacted from formatted output,
errors, and logs, and KMIPKit-owned storage is zeroized when the Pending value
is dropped.

## Explicit asynchronous operations

`Client::execute_poll`, `Client::execute_cancel`,
`Client::execute_process`, and `Client::execute_query_async_requests` each
perform one explicit synchronous exchange. They never poll again, retry, wait,
or schedule background work. A transport error retains the same delivery-state
meaning described below; that state does not establish whether a retry is safe.

The caller constructs a `PollRequest`, `CancelRequest`, or `ProcessRequest`
with the exact bytes obtained from a pending outcome's borrowed correlation
accessor. `execute_poll` returns Pending without issuing another request. Its
successful terminal payload stays generic TTLV until the original operation's
typed response model is available; terminal Failure exposes its result and
reason without a payload. `execute_cancel` verifies that a successful response
echoes the exact requested correlation bytes and exposes the assigned or
unknown Cancellation Result. A Pending Cancel response is rejected under
OASIS KMIP v2.1 §6.1.5, Tables 176–178, and §11.7, Tables 437–438.

`execute_process` is a separate server operation. The caller selects whether
the Process request permits an asynchronous result. Every non-Failure Process
response, including Pending, carries the empty Response Payload required by
OASIS KMIP v2.1 §8.6, Table 399, and defined by §6.1.39, Table 279. Failure
has no payload under §8.6. A Pending outcome is returned to the caller;
KMIPKit does not assert that a later Poll will complete. The §6.1.39 prose
notes that Process may affect other batch items when Batch Order Option is
true, its default. KMIPKit does not claim to control those server-side effects.

`execute_query_async_requests` supports the optional correlation-value and
operation filters from OASIS KMIP v2.1 §6.1.41, Table 285. Its response payload
is exposed as generic TTLV while `KMIPKIT-DISC-039` remains open. KMIPKit does
not present a typed interpretation of the disputed Table 286 caption. The
catalog gap for the required Process request field in Table 278 also remains
open under `OD-002`; no generated catalog output or upstream OASIS source was
changed.

## Resource limits

`CodecLimits` applies to each request execution. Its defaults are:

- Complete message size: 16 MiB.
- Nested Structure depth: 64.
- TTLV Item count, including the root Item: 100,000.

Callers can configure the message-size and Item-count limits, including zero;
Structure depth can be lowered but cannot exceed the model maximum of 64. The
same borrowed `CodecLimits` value is used for request encoding and response
decoding. Its `max_message_bytes()` is also the exact response-byte cap passed
to the transport, and the client checks the returned length before decoding.
There is no separate per-value byte limit; the complete-message limit bounds
any one value. These are local resource limits, not KMIP header fields.

Discover Versions is not classified as a likely-large response, so this API
does not include the peer-visible Maximum Response Size field. That field is
distinct from the local byte cap. Future operation specifications must assess
their response sizes separately. This feature also exposes no connect, read,
write, or total network deadlines because no production I/O backend is
available.

## Errors, delivery state, and redaction

Local `ClientError` values distinguish validation, protocol, and transport
failures and retain only safe cause categories plus the strongest available
`RequestDeliveryState`:

- `NotSent`: no request byte was sent.
- `PossiblySent`: transmission began, but no response byte arrived.
- `ResponseStarted`: at least one response byte arrived, but no complete
  result is available.

A complete KMIP server result is returned as a server result, without local
delivery-state metadata. Delivery state does not establish that a retry is
safe. The client performs one exchange per call and never retries, fails over,
polls, or waits automatically.

Request and response bodies, credentials, secret key material, and opaque
asynchronous correlation values are not logged, formatted into client errors,
or retained in their exposed error-source chains. The private encoded request
owner remains alive through the synchronous exchange and zeroizes its
initialized bytes after it is dropped. This guarantee does not cover spare or
uninitialized capacity, copies made by the caller, earlier allocations not
cleared before reallocation, or copies owned by TLS, the operating system, or
third-party libraries. The approved policy and its precise limits are in
[ADR-0012](../../adr/0012-caller-requested-wire-encoding-policy.md).

## Message Extensions

KMIP §9.13 requires rejection of an unrecognized critical Message
Extension. An unknown
non-critical response extension is preserved as opaque TTLV for explicit
inspection; preservation does not interpret or validate vendor semantics.
This slice does not register or send vendor extensions. KMIPKIT-0012 owns the
immutable client extension registry and validated typed vendor-extension
adapters before the 1.0 API is frozen, as recorded in
[ADR-0013](../../adr/0013-client-extension-registry-ownership.md).

## Low-level transport contract

`kmipkit-transport::Transport::exchange` is a documented public API for direct
Rust users, but the top-level `kmipkit` facade does not re-export it and
`kmipkit-client` does not accept caller-implemented transports. The contract
exchanges caller-supplied request bytes synchronously with a caller-supplied
response-byte cap. It does not encode or validate KMIP messages, and it does
not provide the typed client's request-owner guarantee. The caller owns its
request buffer and is responsible for its validity and lifecycle.

On success, direct transport callers construct a `TransportResponse` with the public
`TransportResponse::new(Vec<u8>)` constructor; its
`as_bytes()` method lends a borrowed view, its `Debug` output is redacted, and
dropping it zeroizes the initialized bytes in its current owned allocation. The typed `Client::execute` path decodes successful wrappers and never returns raw response bodies.
Transport implementations must enforce the response cap while reading and stop
before growing response storage beyond it. They must not log or retain request
bytes after the exchange, must not retry, and must zeroize KMIPKit-owned
temporary request and partial response bytes before releasing them. A concrete
adapter must prevent response-buffer reallocation after storing bytes or
zeroize earlier allocations before release on success and error paths. These
guarantees do not cover
spare capacity, prior allocations unless cleared before reallocation, caller
copies, or external TLS/operating-system/library copies. See
[ADR-0014](../../adr/0014-public-transport-exchange-contract.md) for the exact
contract.

A separately approved TLS/HTTPS feature will own construction from validated
transport configuration and must not expose arbitrary transport injection.
The [transport architecture](../../architecture/transport-security.md) records
the intended TLS and HTTPS profile; it is not evidence that an adapter is
available today.

## Related guides and decisions

- [Inspecting KMIP messages](message-model.md)
- [Public API architecture](../../architecture/public-api.md)
- [KMIP 2.1 asynchronous operations review quickstart](../../../specs/009-asynchronous-operations/quickstart.md)
- [ADR-0002: KMIP 2.1 release scope](../../adr/0002-kmip-21-release-scope.md)
- [ADR-0012: caller-requested wire encoding](../../adr/0012-caller-requested-wire-encoding-policy.md)
- [ADR-0013: client extension registry ownership](../../adr/0013-client-extension-registry-ownership.md)
- [ADR-0014: public low-level transport contract](../../adr/0014-public-transport-exchange-contract.md)
