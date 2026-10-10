# Public Rust Contract: KMIP 2.1 Hash, MAC, and Signature Operations

This design contract follows the existing public request/result conventions in `kmipkit-protocol` and `kmipkit-client`. It is additive. Public types require rustdoc, redacted formatting, and future-compatible values.

## Protocol request types

Each operation has a dedicated request type, a payload conversion method, a dedicated response type, and a sanitized operation-specific model error:

```rust
HashRequest / HashResponse / HashError
MacRequest / MacResponse / MacError
MacVerifyRequest / MacVerifyResponse / MacVerifyError
SignRequest / SignResponse / SignError
SignatureVerifyRequest / SignatureVerifyResponse / SignatureVerifyError
```

Verification requests expose `verification_response_context()` so callers
performing direct protocol conversion can carry the original request's part
shape to response validation. The context is `SinglePart` for unframed requests
and an Init=true/Final=true request without Correlation Value, `MultipartFinal`
for a multipart request with Correlation Value and Final Indicator=true, and
`MultipartNonFinal` for an initial or continuing multipart request that is not
marked final.

Request constructors require only operation fields that the OASIS tables require in every request. Optional fields are set explicitly through consuming `with_*` methods. Fields whose requiredness depends on single-part versus multi-part use remain optional in the stored request and are checked together by the shared request-part validation before serialization.

| Request type | Always required constructor input | Optional fields |
| --- | --- | --- |
| `HashRequest` | Cryptographic Parameters | Data, Correlation Value, Init Indicator, Final Indicator |
| `MacRequest` | None | Unique Identifier, Cryptographic Parameters, Data, Correlation Value, Init Indicator, Final Indicator |
| `MacVerifyRequest` | None | Unique Identifier, Cryptographic Parameters, original Data, MAC Data, Correlation Value, Init Indicator, Final Indicator |
| `SignRequest` | None | Unique Identifier, Cryptographic Parameters, Data, Digested Data, Correlation Value, Init Indicator, Final Indicator |
| `SignatureVerifyRequest` | None | Unique Identifier, Cryptographic Parameters, original Data, Digested Data, Signature Data, Correlation Value, Init Indicator, Final Indicator |

`HashRequest` and every cryptographic request preserve caller-supplied generic Cryptographic Parameters without narrowing unknown members. Data uses the KMIPKIT-0019 `OperationData`. MAC Data, Signature Data, and Digested Data are opaque byte strings. All unique identifiers and multipart values use their existing shared types.

## Protocol response types

`try_from_response_item(ResponseBatchItemView)` validates the operation code and shared result and, for verification responses, assumes the single-part context. Verification responses also provide `try_from_response_item_with_context(item, context)` for explicit multipart response validation. The client derives this context from each original request and carries it through batch response association, including reordered replies. On a non-success KMIP result, the typed response exposes the result and does not fabricate success fields. On success it validates and exposes payload fields by the response table, including exactly one well-formed Unique Identifier for MAC, MAC Verify, Sign, and Signature Verify. Missing, duplicate, or malformed identifiers produce a sanitized typed model error; generic response access remains available. Each response retains access to the generic source item through the common response view so unknown valid fields remain inspectable.

| Response type | Success fields |
| --- | --- |
| `HashResponse` | optional Data bytes; optional Correlation Value |
| `MacResponse` | required Unique Identifier; optional MAC Data bytes; optional Correlation Value |
| `MacVerifyResponse` | required Unique Identifier; optional open Validity Indicator; optional Correlation Value |
| `SignResponse` | required Unique Identifier; optional Signature Data bytes; optional Correlation Value |
| `SignatureVerifyResponse` | required Unique Identifier; optional open Validity Indicator; optional recovered Data bytes; optional Correlation Value |

For successful single-part verification responses, Validity Indicator is required. For final multipart verification responses the parser accepts either presence or absence while `KMIPKIT-DISC-048` remains open. It rejects that field in a non-final typed response because both conflicting sources say it is absent there; the original generic response remains available to inspect.

## Synchronous client operations

The closed `ClientRequest` admits each of the five request types. The client exposes one convenience method per operation, following the existing transport-options pattern:

```rust
client.hash(request, limits)
client.mac(request, limits)
client.mac_verify(request, limits)
client.sign(request, limits)
client.signature_verify(request, limits)
```

Each method returns the existing `ClientBatchItemResponse`; callers may inspect its common operation result and convert its response item through the operation-specific response type. Options-bearing variants follow existing method naming and use the same one-exchange path. Use `Client::execute` with an explicit `ClientBatch` when batch options or multiple request items are needed.

## Error and security contract

- Local request-model errors identify the missing field or shape rule and carry no field value.
- Wrong response operation, malformed success payload, missing required response fields, invalid field type, and disputed multipart shape use sanitized typed model errors.
- `ClientError` retains the existing request-delivery evidence (`NotSent`, `PossiblySent`, or `ResponseStarted`) and transport/protocol cause category.
- Invalid and Unknown verification values are exposed as operation results, not raised as exceptions solely because of their value.
- Debug and Display redact every input/output byte string and Cryptographic Parameters value. Errors never print raw KMIP structures or values.
- Operation calls perform no local hashing, MAC, signing, signature verification, retry, poll, hidden multipart continuation, or ID Placeholder lookup.
