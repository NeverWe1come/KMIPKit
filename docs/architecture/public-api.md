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

Stable categories are configuration, validation, codec, transport, TLS, HTTP,
KMIP response, extension, unavailable feature, and internal panic. Errors
preserve source chains, operation and batch context, delivery state, and KMIP
result fields while redacting sensitive values.

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
