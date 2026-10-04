# Architecture overview

## System shape

```mermaid
flowchart TB
    Java[Java API] --> JNI[JNI bridge]
    Python[Python API] --> CFFI[CFFI bridge]
    C[C API] --> ABI[Stable C ABI]
    JNI --> ABI
    CFFI --> ABI
    RustUser[Rust application] --> Facade[kmipkit facade]
    ABI --> Facade
    Facade --> Client[kmipkit-client]
    Client --> Protocol[kmipkit-protocol]
    Client --> Transport[kmipkit-transport]
    Protocol --> TTLV[kmipkit-ttlv]
    Transport --> Raw[Raw TLS 1.3]
    Transport --> HTTPS[HTTPS / HTTP 1.1]
    Raw --> Server[KMIP server]
    HTTPS --> Server
```

One Rust implementation owns protocol semantics. Language adapters translate
idioms, resources, exceptions, and types; they do not reimplement KMIP.

## Workspace layout

```text
crates/
  kmipkit-ttlv/          Generic tree, encoder, and decoder
  kmipkit-protocol/      KMIP models, validation, catalog output
  kmipkit-transport/     Raw TLS and HTTPS transport implementations
  kmipkit-client/        Synchronous orchestration and connection lifecycle
  kmipkit/               Supported Rust facade and high-level API
  kmipkit-ffi/           C ABI; the only crate allowed to contain unsafe
  kmipkit-test-support/  Fakes, fixtures, and test PKI; not published
bindings/
  java/
  python/
specification/
  oasis/                  Immutable upstream sources
  catalog/                Reviewed machine-readable normative catalog
  compliance/             Requirement traceability and profile matrices
tools/
  specgen/                Deterministic source generation
  xtask/                  Repository automation
docs/
fuzz/
```

The five runtime Rust crates are published with synchronized versions. The
facade is the normal entry point; lower crates are stable public APIs for
advanced integration.

## Layer responsibilities

### `kmipkit-ttlv`

- Implemented: in-memory typed value model, catalog-checked tags, ordered generic tree, and bounded Structure depth.
- Planned: canonical encoder, strict decoder and resource limits, and incremental framing primitives for transports.
- No KMIP operation semantics and no I/O.

### `kmipkit-protocol`

- Generated KMIP constants and known values.
- Managed objects, attributes, messages, and per-operation types.
- Structural and normative validation.
- Profile validation when explicitly requested.
- Conversion to and from generic TTLV.
- Unknown extension preservation.

### `kmipkit-transport`

- Synchronous message transport trait.
- `TcpStream + rustls` raw TLS.
- `reqwest::blocking + rustls` HTTPS.
- Timeouts, delivery state, connection invalidation, and redacted events.
- No operation-specific decisions.

### `kmipkit-client`

- Client lifecycle and one reusable serialized connection.
- Request header construction and response correlation.
- Batch execution and per-item outcomes.
- Pending operation handles and explicit polling/cancellation.
- No automatic retry, failover, or capability discovery.

### `kmipkit`

- High level builders for every client operation.
- Public configuration, result, error, and secret types.
- Reexports of supported advanced protocol and TTLV APIs.
- Stable user-facing Rust documentation.

### `kmipkit-ffi`

- Panic containment.
- Opaque handles and fixed width ABI types.
- Thread-local detailed last error plus stable error codes.
- Memory and lifecycle functions.
- Generated operation builders and accessors.

## Message flow

1. A high level builder creates a typed request.
2. Protocol validation applies invariant rules and any explicitly selected
   profile.
3. The request converts to generic TTLV and encodes canonically.
4. The selected transport sends one complete bounded message.
5. The decoder validates TTLV before typed conversion.
6. The client validates version, correlation, batch count, and operation
   relationships.
7. A single operation returns a typed result, pending handle, or detailed KMIP
   error. A batch returns all per-item outcomes.

## Dependency rules

- Dependencies point downward through the layers shown above.
- TTLV cannot depend on protocol or transport.
- Protocol cannot perform network I/O.
- Transport cannot depend on high level operations.
- Bindings cannot contain KMIP business rules.
- Test support cannot become a runtime dependency.
- Vendor extensions cannot access transport internals.

## Future server-initiated operations

The codec and transport model direction and role explicitly. Version 1.0 only
executes client initiated requests. Version 1.1 adds listeners, dispatch, and
handlers without changing the core TTLV model.
