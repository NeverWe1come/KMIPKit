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

### Generic TTLV

Advanced users can build and inspect an in-memory ordered tree of KMIP items.
Each item has an allocation-checked Tag, including accepted extension tags,
and one of the eleven supported KMIP 2.1 typed Value variants; an unknown Item
Type code has no Value representation. The model checks that the Value
determines the Item Type. It also preserves child order, raw Enumeration
values, bitmask bits, and exact Big Integer Item Value octets. The tree has
KMIPKit's 64-level Structure limit.

The model is not a TTLV wire message and does not establish wire or protocol
validity. It does not store original framing, encoded lengths, or padding
bytes, and it does not validate schema-specific field order, cardinality,
required fields, or operation semantics. The planned TTLV codec will handle
framing, exact wire lengths, endianness, padding, and configured decoder
resource limits. Schema validation for known KMIP Structures and operation
rules belongs in the protocol/client layer before transmission. Re-encoding a
model makes no promise to reproduce the original input bytes. See the
[generic value-model specification](../../specs/004-generic-ttlv-model/spec.md)
for the model's exact scope and constraints.

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

In that version, `Vec::zeroize` clears initialized elements, sets the length to
zero, and zeroizes the entire current allocation capacity. `String::zeroize`
delegates to its backing vector. This describes the current storage owned by
KMIPKit when it is dropped; it is not a guarantee that every process copy of a
value has been erased. Caller-side copies, buffers left by reallocations before
ownership transfer, copies deliberately made from borrowed views, temporary
stack or register copies, and copies retained by Java, Python, or another
runtime are outside this Rust model's guarantee.

The current policy in `AGENTS.md` §8 prohibits serialization of credentials,
private keys, secret key material, OTPs, tickets, and raw KMIP bodies. KMIPKIT-0005
FR-013 and Proposed ADR-0012 only propose a narrow exception for temporary
outbound TTLV generated solely to carry an explicitly caller-requested KMIP
operation, held in a zeroizing KMIPKit-owned buffer through the transport write.
That exception is not in force unless a human accepts ADR-0012 and approves the
KMIPKIT-0005 feature specification. It does not authorize diagnostics,
general-purpose serialization, logging, formatting, error inclusion,
persistence, or arbitrary inbound raw-byte retention or re-emission. ADR-0011
addresses received Reserved Tags and does not authorize wire encoding.

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
