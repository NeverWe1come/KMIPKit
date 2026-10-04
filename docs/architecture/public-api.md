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

Advanced users can build and inspect structurally valid ordered TTLV nodes.
This level supports vendor and future values. It validates framing, types,
lengths, padding, and limits. Arbitrary malformed bytes are restricted to test
tools.

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

- Rust public enums are non-exhaustive and retain an unknown raw value.
- Bitmasks retain unknown bits.
- Tags retain their numeric value.
- Generic nodes retain field order.
- Java and Python avoid closed enums where they would lose data.
- Encoding a valid generic tree after decoding must reproduce the canonical
  input bytes.

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

The library clears its owned memory. It cannot clear copies created by the
caller or managed language runtime.

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
