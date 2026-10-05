# Rust Contract: Message and Batch Model

This is the public API target for `kmipkit-protocol`; implementation may refine names only when the behavior and ownership contract remain unchanged.

## Owning message types

```rust
pub struct RequestMessage { /* validated, ordered generic Structure */ }
pub struct ResponseMessage { /* validated, ordered generic Structure */ }

impl RequestMessage {
    pub fn try_from_ttlv(tree: Structure) -> Result<Self, MessageValidationError>;
    pub fn header(&self) -> RequestHeaderView<'_>;
    pub fn batch_items(&self) -> impl ExactSizeIterator<Item = RequestBatchItemView<'_>> + '_;
    pub fn with_ttlv<R>(&self, f: impl for<'a> FnOnce(StructureView<'a>) -> R) -> R;
    pub fn into_ttlv(self) -> Structure;
}

impl ResponseMessage {
    pub fn try_from_ttlv(tree: Structure) -> Result<Self, MessageValidationError>;
    pub fn header(&self) -> ResponseHeaderView<'_>;
    pub fn batch_items(&self) -> impl ExactSizeIterator<Item = ResponseBatchItemView<'_>> + '_;
    pub fn with_ttlv<R>(&self, f: impl for<'a> FnOnce(StructureView<'a>) -> R) -> R;
    pub fn into_ttlv(self) -> Structure;
}
```

Both validated message types redact their inner tree from `Debug` and `Display`. Construction consumes the supplied tree. Successful construction stores it without cloning; failed construction drops it through the TTLV zeroizing owner. `into_ttlv` returns the same tree without reordering or copying.

## Typed views

`ProtocolVersion` is a small copied value containing raw signed major/minor integers. `RequestHeaderView::protocol_version()` returns it without exposing the backing tree. `RequestHeaderView::batch_count()` returns the validated item count.

- `ProtocolVersion` returns copied raw Major Version and Minor Version integer values.
- `RequestHeaderView` and `ResponseHeaderView` expose each in-scope 1.0 Table 395/398 field, optional presence, repeated-value order, and effective defaults where the specification defines one. Server Correlation Value in a client-to-server Request Header remains opaque in the generic tree and has no typed setter; the Response Header exposes it as server metadata.
- `RequestBatchItemView` and `ResponseBatchItemView` expose typed scalar fields and required/optional presence. Payloads, authentication, attestation, nonce, hashed password, and extension values are exposed only through callback-scoped `StructureView`/`ValueView` accessors.
- Raw Enumeration wrappers preserve unknown `u32` values. Named constants are convenience values, not a closed Rust enum.
- `MessageValidationError` is non-exhaustive, carries a stable category and safe field/index metadata only, and never retains a raw item, payload, tag string, credential, extension value, or Result Message.

## Generic TTLV addition

`kmipkit-ttlv::Structure` adds a safe borrowed view method returning `StructureView<'_>`. The view is tied to the source borrow, does not expose mutable access, and does not transfer or clone payload ownership. Existing child order and `Item::with_value` closure restrictions remain intact.

## Required invariants

The Request Message and Response Message envelope order is validated using their respective wrapper tables: the Request Header precedes one or more Request Batch Items per Table 394, and the Response Header precedes one or more Response Batch Items per Table 397. Request header and batch-item field order is validated against Tables 395–396; response header and batch-item field order is validated against Tables 398–399.

1. Request/response structures contain one header before at least one batch item; Batch Count equals the number of items.
2. Known field order is validated against Tables 395–399. Unknown allocation-valid fields and schema-authorized repeated fields stay in their original order; duplicate known singleton fields are rejected.
3. Multi-item requests carry one Unique Batch Item ID per request item; pairwise distinctness is a KMIPKit project invariant, not an OASIS requirement. Single-item requests may omit the ID.
4. Batch Order Option and Batch Error Continuation Option appear only on multi-item requests. `RequestMessage::try_from_ttlv` and request-model conversions preserve all raw Asynchronous Indicator and Batch Error Continuation values, including unassigned values, without applying outbound policy. Before emitting client requests, `KMIPKIT-0007-client-execution` validates Asynchronous Indicator values, accepting assigned values and the OASIS `0x80000000..=0x8FFFFFFF` extension allocation and rejecting values in neither set (Table 432; §9.2). It accepts assigned Batch Error Continuation values and rejects values outside the assigned values and extension allocation; acceptance or rejection of values within that allocation remains unresolved under `KMIPKIT-0007-OD-001` pending source/catalog-owner disposition (Table 435; §9.6). Generic TTLV and typed raw-preserving Enumeration wrappers retain unknown and vendor/future values. Effective defaults remain queryable without materializing an absent field.
5. Pending results carry an Asynchronous Correlation Value. `KMIPKIT-0007-client-execution` checks whether the associated request permitted async results; `KMIPKIT-0009-asynchronous-operations` uses the exact bytes in explicit Poll/Cancel requests. This model does not compare request and response or initiate follow-up operations.
6. Message Extensions require one correctly typed Vendor Identification, Criticality Indicator, and Vendor Extension in Table 418 order; Vendor Identification uses only `[A-Za-z0-9_.]`.
7. The immutable per-client registry in `docs/architecture/extensions.md`, not this parser, decides extension recognition. `KMIPKIT-0007-client-execution` enforces criticality as required by ADR-0007.
8. Response Result Message is absent for Success and Pending; it may appear for other statuses. Response Operation is required when specified by the paired request, which `KMIPKIT-0007-client-execution` validates.
9. Request batch items may contain repeated Message Extensions; response batch items may contain at most one optional Message Extension.
10. All error and formatting paths are payload-free and redacted.
