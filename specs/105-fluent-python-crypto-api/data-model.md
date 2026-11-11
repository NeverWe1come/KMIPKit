# KMIPKIT-0105 Data Model

## Python operation builder

An operation builder is a scoped owner for one Rust C ABI
`kmipkit_crypto_operation_builder_t`.

| Field | Meaning | Constraint |
|---|---|---|
| Operation kind | One of Encrypt, Decrypt, Sign, Signature Verify, MAC, MAC Verify, Hash | Fixed by constructor; never caller-selected through a numeric value |
| Client | Optional configured KMIP client used by `execute()` | Required at execution; may be supplied by `Client.request()` |
| Cryptographic Parameters | Typed KMIP object or explicitly ordered TTLV Structure | Caller-selected; no algorithm or mode is inferred |
| Operation inputs | Byte values, typed Data, key identifier, or multipart fields | Exactly those accepted by the operation's typed builder |
| RequestOptions | Exchange-local timeout configuration | Independent from Request Header fields |
| Asynchronous Indicator | Optional uint32 Request Header value | Caller-supplied and validated before execution |
| Native handle | Opaque Rust-owned request builder | Released by `close()`, context exit, or finalizer |

## Lifecycle

`open -> configured -> build/execute -> closed`. `build()` does not consume the
builder. `execute()` performs one exchange and does not retry or poll. Any
native operation after `closed` raises the package's existing closed-handle
error. Context manager exit releases the handle even when the body raises.

## Data ownership

Python input buffers are staged for a synchronous FFI call. KMIPKit clears
mutable staging arrays it allocated in all exit paths. The Rust builder owns
and zeroizes its copy. Caller-owned bytes and copies retained by the Python
runtime are outside KMIPKit's memory control.
