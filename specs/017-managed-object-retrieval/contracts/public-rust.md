# Public Rust Contract: Get and Locate

**Feature**: KMIPKIT-0017<br>
**Status**: Proposed; release-contract refresh pending review before implementation.<br>
**Compatibility**: Additive APIs in kmipkit-protocol and kmipkit-client. No new dependency or unsafe code.

## Request models

- GetRequest::new() creates an empty request. Consuming builders set optional Unique Identifier, Key Format Type, Key Wrap Type, Key Compression Type, and Key Wrapping Specification. Accessors preserve each field's absence.
- LocateRequest::new(attributes: AttributeSet) creates the required Attributes Structure. Consuming builders set optional Maximum Items, Offset Items, raw-preserving StorageStatusMask, and Object Group Member. Accessors preserve explicit zero, unknown mask bits, and an empty Attributes Structure.
- StorageStatusMask::from_raw(u32) and raw() -> u32 preserve the Integer bit pattern, including bit values unknown to this release. Encoding and decoding reinterpret the `u32`/signed `i32` bit pattern so `0x8000_0000` round-trips without numeric range conversion.
- Enumeration selectors follow the raw-preserving ObjectType pattern with from_raw(u32) and raw() -> u32.
- Payload conversion follows exact Table 220 and Table 247 order and returns sanitized ProtocolError values.

## Typed response models

- GetResponse::try_from_response_item(ResponseBatchItemView<'_>) validates the successful Table 221 envelope and owns the complete opaque generic Any Object item. It exposes result(), object_type(), unique_identifier(), and object() -> Option<&Item>. The owned item remains redacted and zeroizing under the existing TTLV contract; caller-created clones are outside that guarantee.
- LocateResponse::try_from_response_item(ResponseBatchItemView<'_>) validates Table 248 forms and exposes result(), located_items(), and unique_identifiers(). The identifier slice preserves wire order and repetitions.
- Error outcomes retain KmipOperationResult but have no success-only fields. Unknown result and object Enumeration values stay accessible as raw values.
- Invalid responses return sanitized schema errors without raw body or value formatting.

## Client execution contract<br>

- Add ClientRequest::Get(GetRequest) and ClientRequest::Locate(LocateRequest).
- Add ClientOperation::Get and ClientOperation::Locate.
- Add GetCompleted and LocateCompleted variants to ClientBatchOutcome plus get_response() and locate_response() accessors; a mismatched operation or Pending result returns None. Add Get and Locate to the private response reference mapping and `ClientResponseView::get()` / `locate()` so the typed response representation is accessible through `ClientBatchOutcome::response()` and `PendingOutcome::response()` whenever the common response path provides one. Keep ClientOperationOutcome for the separate asynchronous-operation API.
- Client::execute and execute_with_options continue to accept ClientBatch and CodecLimits and return ClientBatchResponse. Each request-order item exposes its typed ClientBatchOutcome. They preserve one exchange per submitted batch, request/result order, Pending results, and RequestDeliveryState; they never retry, poll automatically, split batches, search local state, or infer ID Placeholder state.

Method signatures and response ownership follow the current non-exhaustive `ClientRequest`, `ClientOperation`, and `ClientBatchOutcome` surfaces, plus the `ResponseBatchItemView` pattern in `execute.rs`. `ResponseBatchItemView::with_ttlv` lends the full ordered response tree only for the callback lifetime. `GetResponse` owns its secret-bearing Any Object rather than returning that borrowed view; add a fallible `Item::try_clone()` to deep-copy only the object item, preserving repeated/unknown descendants, redacted formatting, and zeroization. `ClientResponseView::get()` and `locate()` expose the typed response representation when available, including the shared Pending representation; direct `ClientBatchOutcome::get_response()` and `locate_response()` remain completed-only. Pending outcomes retain the common result and required correlation value and are never interpreted as operation completion. Locate criteria use the current validated `AttributeSet`: empty criteria are valid, while populated values pass through `try_new`/`try_push`, including catalogued TTLV-type and Vendor Attribute shape/order checks. Any needed public-contract change must return to this specification before code continues.

## Security and errors

Object contents, Key Wrapping Specification values, raw KMIP bodies, and credentials never appear in debug/error formatting. KMIPKit-owned TTLV payloads are zeroized on drop; copies retained by a caller are outside this guarantee. Transport/decode errors preserve the existing not-sent, possibly-sent, and receiving-response delivery classification.
