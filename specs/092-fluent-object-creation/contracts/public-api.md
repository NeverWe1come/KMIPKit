# Public API Contract: Fluent Object Creation

This contract adds task-oriented entry points on the approved KMIPKIT-0091
foundation while preserving the existing typed request, batch, and execution
APIs.

## Rust

- `Client::request()` exposes builders for `create_key_pair()`,
  `create_split_key(...)`, and `register()`.
- Builders expose explicit semantic choices for algorithms/presets, key usages,
  split method/parts/threshold, object data, attributes, and optional storage
  controls. The common path uses named algorithm, usage, object-type,
  split-method, and key-format values rather than numeric protocol values.
- Each builder provides validation through `build()`, conversion to the
  existing typed batch-item path, and one-operation `execute()` through the
  current client methods.
- `with_options(RequestOptions)` applies only to the exchange. Authentication
  and shared Request Header changes remain explicit batch-level actions.
- Execute preserves typed Success, Pending, Failure, and delivery state. It
  does not call Poll or retry.

## C

- Expose task-level client functions for common Create Key Pair, Create Split
  Key, and Register inputs and opaque builders for advanced inputs.
- Any public input record is versioned and has fixed-width fields; handles
  follow the existing documented ownership and release rules.
- Return the stable `int32_t` status values and expose existing result/error
  snapshots. Do not add a second `kmipkit_error **`-style model.
- C callers provide semantic fields or typed object handles; the high-level
  builders do not require callers to create TTLV Request Messages or manually
  assemble operation payloads.

## Java and Python

- Java exposes equivalent task methods in `org.kmipkit.client` and builders in
  `org.kmipkit.fluent`; closeable/native ownership follows existing facade
  conventions.
- Python exposes equivalent named-argument Client methods and builders in
  `kmipkit.fluent`; `close()` and context-manager lifetimes follow existing
  conventions.
- Java/Python wrappers pass semantic values through the stable C ABI. They do
  not implement independent KMIP serialization, validation, or secret
  erasure.
- Documentation identifies which copies may remain under runtime management.

## Compatibility and Execution Boundary

- Existing symbols and package exports remain available.
- Fluent execution accepts only existing typed requests or typed batch items.
- No arbitrary TTLV Request Message, raw request bytes, or caller-supplied
  transport is accepted for execution.
- KMIP 2.1 remains the default effective version. Unsupported version and
  operation combinations fail before the request is written.
