# Public API design

## Three supported levels

### High level

The 1.0 target includes idiomatic operation builders, but KMIPKIT-0007 does
not implement the general builder surface. The current typed client exposes
Discover Versions, Get Attributes, and Get Attribute List for batch execution,
along with separate Poll, Cancel, Process, and Query Asynchronous Requests
methods.
KMIPKIT-0013 adds synchronous production construction from a separate
immutable client extension configuration and validated transport
configuration. The typed client accepts neither raw request bytes nor an
arbitrary caller-implemented transport. This request set is not full KMIP 2.1
coverage. See the [client execution guide](../user-guide/en/client-execution.md)
for the current boundary and examples.


### Low-level transport

kmipkit-transport is a public reusable Rust crate under ADR-0003. ADR-0014
defines its documented bounded byte-exchange contract. It is not re-exported
by the kmipkit facade and cannot be injected into the typed client. Direct
callers supply and own request bytes; this API does not encode or validate
KMIP messages and does not provide the typed client's request-owner guarantee.
On success, `TransportResponse` lends bytes through a borrowed view, redacts
`Debug` output, and zeroizes initialized bytes in its current owned allocation
on drop. Partial and error cleanup limits, including allocation growth and
external copies, are defined by ADR-0014. KMIPKIT-0013 provides production
raw-TLS and HTTPS/HTTP 1.1 adapters; only the validated adapters are reachable
from the typed client. Direct users may call each adapter's bounded byte API.

### Typed protocol

KMIPKIT-0007 provides synchronous `Client::execute` over an ordered `ClientBatch`
with a closed typed request set. The currently implemented operation variants
are Discover Versions, Get Attributes, and Get Attribute List; the remaining
attribute operations are assigned to later KMIPKIT-0016 increments. The API
does not accept generic Item values, raw message bytes, or caller-implemented
conversions. Discover Versions is explicit and is never a hidden preflight.
Per-call `CodecLimits` bound request encoding and response decoding and supply
the transport response-byte cap.

KMIPKIT-0013 constructs the typed client from validated transport
configuration without arbitrary transport injection. The public low-level
`kmipkit-transport` exchange contract remains a separate direct caller-byte
API under ADR-0014. A typed result does not expose raw response bytes. See the
[client execution guide](../user-guide/en/client-execution.md) and
[ADR-0014](../adr/0014-public-transport-exchange-contract.md).

### Credential and Authentication values

KMIPKIT-0008 adds typed, raw-preserving Credential and Authentication values
to `kmipkit-protocol`. Authentication requires one or more Credentials in
caller-supplied order. Known Credential Values validate their KMIP 2.1 table
fields; unknown Credential Types, fields, and raw Enumeration values remain
available through the retained TTLV tree. Device values require at least one
of Device Serial Number, Network Identifier, Machine Identifier, or Media
Identifier. The caller remains responsible for supplying an identifier or
combination that is actually unique; KMIPKit neither chooses a comparison
scope nor checks actual uniqueness.

These values are in-memory protocol models. KMIPKIT-0008 does not add
Authentication or Credential data to the request payload or introduce a
secret-bearing request path. Every synchronous and asynchronous client
Request Header advertises `Attestation Capable Indicator = True` because the
public API can construct Attestation Credentials. That bit reports
construction capability only; it
does not claim to generate or verify evidence, submit a Credential, or predict
server acceptance. Callers cannot override it per request.

### Generic TTLV

Advanced users can build and inspect an in-memory ordered tree of KMIP items.
Each item has an allocation-checked Tag, including accepted extension tags,
and one of the eleven supported KMIP 2.1 typed Value variants; an unknown Item
Type code has no Value representation. The model checks that the Value
determines the Item Type. It also preserves child order, raw Enumeration
values, bitmask bits, and exact Big Integer Item Value octets. The tree has
KMIPKit's 64-level Structure limit.

The generic TTLV layer does not establish wire or protocol validity. It does
not store original framing, encoded lengths, or padding bytes, and does not
validate schema-specific field order, cardinality, required fields, or
operation semantics. The KMIP 2.1 message layer validates Request/Response
Message envelopes, common headers, batch items, result relationships, and
Message Extension shapes while retaining the source tree. It leaves
operation payload contents generic. The public `kmipkit-ttlv` codec surface
provides the bounded decoder and no byte-producing encoder. KMIPKIT-0007 owns
the private writer and sole production callsite within the typed
Client::execute path; callers cannot submit generic TTLV through that path.
See the [generic value-model specification](../../specs/004-generic-ttlv-model/spec.md)
and [client execution guide](../user-guide/en/client-execution.md).

### KMIP message model

`kmipkit-protocol` exposes owning `RequestMessage` and `ResponseMessage` values
created from a generic TTLV `Structure`. Parsing checks message and batch field
order, required fields, singleton and repeatable fields, Batch Count, request
item IDs, result-status constraints, and extension structure. Conversion back
to TTLV returns the original tree, including unknown values and source order.
Typed header, batch-item, and extension views lend nested values only for the
duration of an accessor callback. Raw Enumeration values and field presence
remain observable; absent Asynchronous Indicator, Batch Order Option, Batch
Error Continuation Option, and Attestation Capable Indicator expose their
specified effective defaults without materializing fields.

This model is for in-memory validation and inspection. It does not encode or
send a message, validate operation-specific payloads, apply client send policy,
or automatically Poll, Cancel, wait, or retry. See the
[English message-model guide](../user-guide/en/message-model.md) and
[Spanish message-model guide](../user-guide/es/modelo-mensaje.md) for the
application-facing behavior.

## Builder policy

Builders use ordinary strongly typed state and return detailed validation
errors. The project avoids typestate builders across the whole standard
because they would produce a large, difficult cross-language type surface.

Automatically supplied fields may include protocol version 2.1, batch count,
required batch identifiers, timestamp, and normative mechanical defaults.

Explicit application choices include algorithm, key length, cryptographic
usage, mode, padding, related parameters, export/protection policy, and
optional profile selection.

## Unknown values

- Generic Enumeration values preserve their raw 32-bit value, including
  unassigned values; bitmasks preserve unknown bits.
- Allocation-checked Tags retain their numeric value, including accepted
  extension tags.
- Generic nodes retain child order.
- Unknown Item Type codes are not represented by this model.
- Rust protocol enums remain non-exhaustive where the typed API retains raw
  values; Java and Python avoid closed enums where they would lose data.
- Value preservation does not imply preservation of the original wire bytes or
  padding; see [Generic TTLV](#generic-ttlv).

## Validation boundary

KMIPKit validates standard invariants before network I/O. It does not predict
server authorization, configured algorithms, policy, quotas, or product
capabilities. Those failures are returned from the KMIP response.

Profile validation is explicit. Query and Discover Versions are explicit
operations; they are never hidden prerequisites for ordinary calls.

## Batch

The high level batch builder supports heterogeneous operations and all KMIP
ordering and continuation options. KMIPKit preserves input order and unique
batch IDs and never splits, reorders, or retries a batch.

A valid batch response returns a `BatchResult` even when items fail. Each item
is completed, pending, or failed. Whole-message and transport errors fail the
call itself.

## Protocol asynchronous outcome

The 1.0 target includes explicit follow-up operations for KMIP asynchronous
results. KMIPKIT-0007 represents a Pending item as
`ClientBatchOutcome::Pending(PendingOutcome)`, preserving the opaque
Asynchronous Correlation Value behind an explicit borrowed accessor. The
Pending value retains the originating typed response; `response()` returns a
borrowed `ClientResponseView` with a shared operation-result accessor and
operation-specific accessors. The view also preserves Discover Versions'
supported-version accessor. Formatted output is redacted and KMIPKit-owned
correlation storage is zeroized when the Pending value is dropped. Poll,
Cancel, result processing, automatic waiting, and background execution are not
implemented by this foundation; see the
[client execution guide](../user-guide/en/client-execution.md).

## Secrets

Secret values use dedicated types that cannot be formatted or serialized.
Exposure requires an explicit operation and produces a controlled temporary
view or copy. Rust uses closure-scoped access, C an opaque secret handle, Java
an `AutoCloseable` secret, and Python a context-managed secret.

In `kmipkit-ttlv`, each `Value` privately owns a boxed representation and calls
the safe `Zeroize` API before releasing it. Payloads in nested Structures are
zeroized recursively. The dependency is locked to `zeroize` 1.9.0 with only
its `alloc` feature enabled; see the [dependency review](../../specs/004-generic-ttlv-model/dependency-review.md)
and the [pinned 1.9.0 source](https://docs.rs/crate/zeroize/1.9.0/source/src/lib.rs).

In that version, `Vec::zeroize` zeroizes the entire capacity of its current
backing allocation, including spare capacity, and then sets its length to
zero. `String::zeroize` delegates to its backing vector and has the same
current-allocation behavior. Neither can guarantee that copies left in an
earlier allocation by reallocation were cleared. The private outbound owner
used by KMIPKIT-0007 documents a narrower guarantee: its initialized encoded
byte range is zeroized after the synchronous exchange and before owner drop.
Although the pinned `Vec::zeroize` implementation also clears that owner's
current spare capacity, that extra behavior is not part of the documented
KMIPKit guarantee. This is not a guarantee that every process copy of a value
has been erased. Caller-side copies, buffers left by earlier reallocations,
copies deliberately made from borrowed views, temporary stack or register
copies, and copies retained by Java, Python, or another runtime are outside
this Rust model's guarantee.

The KMIPKIT-0007 execute path uses its private zeroizing request owner only for
a caller-requested typed operation. Initialized encoded bytes are zeroized
when that owner is dropped after exchange; broader copy and allocation limits
are defined by [ADR-0012](../adr/0012-caller-requested-wire-encoding-policy.md).
The separate low-level raw-byte exchange exception and its response-wrapper
limits are defined by [ADR-0014](../adr/0014-public-transport-exchange-contract.md). The typed client
always decodes the response wrapper and does not expose raw bodies.

Credential `SecretText` and `SecretBytes` take ownership of their supplied
String or byte-vector allocation without cloning it and zeroize the owned
value on drop. Consuming a wrapper into a TTLV value moves that allocation to
the TTLV owner. The guarantee applies to the owned current allocation before
its owner releases it. It cannot erase caller-created copies, prior
allocations left by earlier growth, copies made from callback-borrowed views,
temporary stack or register copies, or copies retained by a foreign runtime,
TLS implementation, operating system, or other dependency. The documented
guarantee covers initialized bytes in KMIPKit-owned current allocations; spare
or uninitialized capacity is covered only when initialized and cleanup is
verified. Logging, formatting, and validation diagnostics redact Credential
contents, including identifiers inside a Credential.

## Errors

The shared Rust result/error contract is exposed by the `kmipkit` facade:

- `ResultStatus` and `ResultReason` retain the raw 32-bit Enumeration value.
  `known_name()` resolves assigned values from the checked-in normative catalog;
  unknown values remain unchanged.
- `KmipOperationResult` contains status, optional reason, and optional
  `ResultMessage`. Construction requires a reason for `Operation Failed` and
  forbids one for `Success`. Pending, undone, and unknown statuses gain no
  additional reason rule in this feature.
- Applications can explicitly inspect Result Message text through
  `ResultMessage::as_str()` or `as_bytes()`. Its Debug output and the default
  Display/Debug output of `KmipOperationResult` redact the message.
- `ClientError` distinguishes validation, protocol, and transport failures
  from a complete server result. A server result retains the complete
  `KmipOperationResult` and has no local request-delivery state.
- Every local `ClientError` requires exactly one `RequestDeliveryState`;
  validation before transmission is `NotSent`. Constructors cannot omit this
  evidence.
- `RequestDeliveryState` reports `NotSent`, `PossiblySent`, or
  `ResponseStarted`. The first response byte advances the state to
  `ResponseStarted`; a zero-byte read does not. Delivery evidence alone does
  not imply that retrying is safe.
- Protocol, transport, and validation constructors consume and discard
  arbitrary source errors before retaining safe cause categories. Public error
  chains contain only those safe categories. This result/error feature adds no
  automatic serialization and no logger or logging call.

Later operation-specific errors may add configuration, codec, TLS, HTTP,
extension, unavailable-feature, and operation/batch context. They must preserve
the source-sanitization and redaction rules above when accepting untrusted
sources or server text.

Rust returns `Result`. Java exposes unchecked typed exceptions. Python exposes
an equivalent exception hierarchy. C returns stable codes and exposes detailed
thread-local context.

## Lifecycle and threading

- Clients are thread safe.
- One client serializes calls over one connection.
- `close` is idempotent and waits for active calls before releasing resources.
- Calls on a closed client return a stable error.
- Rust also releases through `Drop`; Java and Python finalizers are fallback
  mechanisms only.
