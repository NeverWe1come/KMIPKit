# Rust API Contract: KMIPKIT-0050

## Typed Cryptographic Parameters

- `CryptographicParameters::try_from_ttlv(Structure)` validates and retains the supplied tree.
- `CryptographicParametersBuilder::new()` starts an empty caller-selected value. `with_*` methods replace the corresponding singleton; `with_extension(Item)` retains a non-owned extension member; `build()` emits known fields in Table 59 order and returns a payload-free validation error on invalid input.
- Typed getters expose every known member. Enumeration getters return the exact raw `u32`; no conversion rejects future or vendor values.
- `with_ttlv` lends the original Structure through a callback. `into_ttlv` consumes the wrapper and returns that Structure.
- Operation request types keep their existing `with_cryptographic_parameters(Structure)` method and gain additive `with_typed_cryptographic_parameters(CryptographicParameters)` methods.

## Request Authentication

- `ClientBatch::with_authentication(Authentication) -> ClientBatch` sets the optional Request Header field for that one message.
- `ClientBatch::new(...)` and `ClientBatch::from_items(...)` leave it absent.
- Authentication is encoded once in Request Header order and applies to every item in the batch.
- The typed Authentication constructor rejects empty or malformed generic structures before they can be attached to a batch. Any later preflight failure occurs before I/O and carries `RequestDeliveryState::NotSent`.
- The Request Header writer encodes Authentication after Attestation Type and before Batch Error Continuation Option, matching KMIP v2.1 §8.2 Table 395; the field is emitted once for the entire message.
- No credential field is added to `ClientConfiguration` or `Client`.

## Direct Client Methods

Each direct method has a default-options form and a `_with_options` form, forwards the existing typed request to one-item batch execution, and returns a `ClientBatchItemResponse`. It propagates any returned Pending status and correlation value without polling. The caller uses `ClientBatch` when selecting the Request Header Asynchronous Indicator or Authentication; this preserves access to shared batch-level fields without duplicating them across all direct method signatures.
