# Public API design

## Three supported levels

### High level

Every client initiated operation in the 1.0 scope receives an idiomatic
builder. Builders hide message structure and fill mechanical fields while
requiring security-sensitive choices explicitly.

Illustrative Rust API:

```rust,ignore
let result = client
    .create_symmetric_key()
    .name("database-encryption")
    .algorithm(SymmetricAlgorithm::Aes)
    .length_bits(256)
    .usage(CryptographicUsage::ENCRYPT | CryptographicUsage::DECRYPT)
    .execute()?;
```

The exact signature is established by its approved implementation
specification. This example describes intent, not existing code.

### Typed protocol

Each operation has a distinct request and response type. Applications can
control headers, attributes, credentials, parameters, batch IDs, and
extensions. `Client::execute(request)` returns the complete typed outcome.
Its approved API must take a closed set of concrete typed KMIP requests: it
cannot accept the public generic `Item` tree, raw KMIP body bytes, or a
caller-implementable conversion trait as an alternate route to wire encoding.
The exact variants, signature, and per-call limit configuration belong to the
first client feature specification.

### Generic TTLV

Advanced users can build and inspect an in-memory ordered tree of KMIP items.
Each item has an allocation-checked Tag, including accepted extension tags,
and one of the eleven supported KMIP 2.1 typed Value variants; an unknown Item
Type code has no Value representation. The model checks that the Value
determines the Item Type. It also preserves child order, raw Enumeration
values, bitmask bits, and exact Big Integer Item Value octets. The tree has
KMIPKit's 64-level Structure limit.

The generic TTLV layer does not establish wire or protocol validity. It does
not store original framing, encoded lengths, or padding bytes, and it does not
validate schema-specific field order, cardinality, required fields, or
operation semantics. The KMIP 2.1 message layer validates the Request/Response
Message envelopes, common headers, batch items, result relationships, and
Message Extension shapes while retaining the source tree. It leaves operation
payload contents generic. The public `kmipkit-ttlv` codec surface provides the
bounded decoder for framing, exact wire lengths, endianness, padding, and
configured resource limits; it exposes no byte-producing encoder. KMIPKIT-0005
implements and tests a private writer, but adds no `Client::execute`, permit
type/constructor, or production callsite.
The first client feature/spec owns the execute API, its private permit type and
constructor, the sole production mint/callsite, and an exact-one audit. That
execute path must accept only a closed typed request input. The delegated
approval of ADR-0012, the feature specification, and the enforceable
permit/request boundary is recorded in
`specs/005-ttlv-wire-codec/approval-record.md`. KMIPKIT-0005 still adds no
production callsite. The first client feature PR must include the sole
production callsite and its owner-through-transport
integration test together. CI must pass that test against the candidate
callsite before merge, enablement, or release; until then, the release branch
must have neither the callsite nor a secret-bearing send. Schema validation
for known KMIP Structures and operation rules belongs
in the protocol/client layer before transmission. There is no public
`encode(&Item)` API, and the decoder does not retain original bytes for
re-emission. See the
[generic value-model specification](../../specs/004-generic-ttlv-model/spec.md)
for the model's exact scope and constraints.

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

```text
OperationOutcome<T>
  Completed(T)
  Pending(PendingOperation<T>)
```

`PendingOperation` exposes its correlation value, explicit poll and cancel,
result processing, and an optional blocking `wait` helper with configured
interval and deadline. No background task or implicit wait is started.

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

In that version, `Vec::zeroize` clears its initialized elements and sets the
length to zero; it does not guarantee wiping spare or otherwise uninitialized
allocation capacity. `String::zeroize` delegates to its initialized backing
vector contents. For the private outbound owner implemented by KMIPKIT-0005,
the guarantee is limited to zeroizing the initialized encoded byte range
before deallocation/owner drop.
Spare capacity is outside the guarantee unless explicitly initialized and its
cleanup is verified. This is not a guarantee that every process copy of a
value has been erased. Caller-side copies, buffers left by reallocations before
ownership transfer, copies deliberately made from borrowed views, temporary
stack or register copies, and copies retained by Java, Python, or another
runtime are outside this Rust model's guarantee.

The current policy in `AGENTS.md` §8 prohibits serialization of credentials,
private keys, secret key material, OTPs, tickets, and raw KMIP bodies. KMIPKIT-0005
FR-013 and accepted ADR-0012 define a narrow policy for temporary outbound
TTLV generated solely for a caller-requested typed operation. KMIPKIT-0005 may
implement and test a private encoder, but it adds no permit or production
callsite. The first client feature/spec owns the closed typed `Client::execute`
API, its execute-owned permit type/private constructor and sole production
mint/callsite, plus the exact-one audit. Bytes must be held in a private
zeroizing KMIPKit-owned buffer through the transport write; before owner
deallocation/drop, zeroize the initialized encoded byte range. Spare or
uninitialized `Vec` capacity is outside the guarantee unless explicitly
initialized and its cleanup is verified. The accepted policy has no
public `encode(&Item)` API. The delegated approval of ADR-0012, the KMIPKIT-0005
feature specification, and the enforceable private boundary is recorded in
`specs/005-ttlv-wire-codec/approval-record.md`. The first client feature PR must include its sole production
callsite and owner-through-transport integration test together. CI must pass
the test against the candidate callsite before merge, enablement, or release.
Until then, the release branch must have neither the callsite nor a secret-bearing send. If review rejects that
boundary or it cannot be enforced, do not implement a production secret-bearing
request path. The policy does not authorize diagnostics, general-purpose
serialization, logging, formatting, error inclusion, persistence, or arbitrary
inbound raw-byte retention or re-emission. ADR-0011 addresses received Reserved
Tags and does not authorize wire encoding.

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
