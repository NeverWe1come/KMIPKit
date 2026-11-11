# Feature Specification: Fluent Create Split Key Builder (KMIPKIT-0099)

**Branch**: `feature/KMIPKIT-0099-fluent-create-split-key`  
**Base**: `feature/KMIPKIT-0097-fluent-object-api` at
`45749b62e355ceea1ac6c936ec4a65b085050d60`  
**Status**: Approved for implementation by the maintainer's direct request to
implement the complete KMIPKIT-0093 plan.
**Owning identifier**: KMIPKIT-0099

## Purpose

Add an idiomatic Rust request builder for the existing typed Create Split Key
operation. A caller will explicitly choose the object type, split method,
number of parts, threshold, and optional input values; validate locally;
convert to the existing typed batch item; or perform one exchange through the
current `Client`. The typed protocol request remains the authority for payload
encoding, Prime Field Size policy, Attribute cardinality, and response
decoding.

This is one Rust operation-object delivery under KMIPKIT-0093. It does not
claim that other operation objects or the C, Java, and Python facades are
complete.

## Normative and architectural sources

- OASIS KMIP Specification v2.1 §6.1.10 Tables 193–195: Create Split Key
  request fields, response identifiers, and result fields.
- OASIS KMIP Specification v2.1 §§4–5: Object Attributes and their TTLV
  representation.
- `specs/023-key-material-transfer/spec.md` and
  `crates/kmipkit-protocol/src/create_split_key.rs`: existing typed request,
  response, source-backed Attribute checks, and the explicit
  Prime Field Size client policy.
- `specs/093-fluent-complete-api/spec.md`: KMIPKIT-0093-FR-004 through FR-011,
  especially the closed typed-send boundary, batch conversion, explicit
  choices, operation results, and generated public API contract.
- ADR-0012 and ADR-0022: existing typed request execution boundary and
  supported protocol versions.

## Functional requirements

- **KMIPKIT-0099-FR-001**: `RequestBuilder::create_split_key()` MUST return a
  public `CreateSplitKeyBuilder`. Building requires explicit Object Type,
  Split Key Method, Split Key Parts, and Split Key Threshold. It MUST use the
  existing `CreateSplitKeyRequest` model and MUST NOT select or normalize
  these values.
- **KMIPKIT-0099-FR-002**: The builder MUST preserve an optional input Unique
  Identifier, exact caller-supplied Prime Field Size bytes, a prebuilt
  `AttributeSet`, and optional Protection Storage Masks. The request's
  Attributes Structure MUST remain present when empty, as required by the
  existing typed model. No input identifier or object state is inferred.
- **KMIPKIT-0099-FR-003**: The builder MUST provide typed and explicit Item
  attribute insertion, a Vendor Attribute constructor that forms the valid
  KMIP Vendor Attribute structure, and explicit final-position insertion in
  the direct Object Attributes list. Message Extension tags MUST be rejected
  as Object Attributes. The final set MUST use the existing `AttributeSet`
  and typed request validation; unrecognized tags MUST retain their explicit
  TTLV type and value.
- **KMIPKIT-0099-FR-004**: `validate(&self)` MUST build and validate through the
  same local path as `build_request()` without consuming the builder or
  invoking transport. The existing typed validator MUST enforce its
  Prime Field Size rule for Polynomial Sharing Prime Field and applicable
  Attribute cardinality rules. Invalid construction MUST return a
  payload-free `FluentBuildError` and MUST NOT be sent.
- **KMIPKIT-0099-FR-005**: `build`, `build_request`, `to_batch_item`, and
  `execute` MUST preserve every selected field and attribute order. `execute`
  MUST use the existing typed client path for one exchange at most and retain
  existing Success, Failure, Pending, correlation, and delivery-state
  behavior. It MUST NOT retry or Poll automatically.
- **KMIPKIT-0099-FR-006**: Exchange-local `RequestOptions` and the explicit
  Request Header Asynchronous Indicator MUST use the existing API boundaries.
  The operation remains on the existing KMIP 2.1 typed route; the builder
  MUST NOT add raw-message submission or select a protocol version.
- **KMIPKIT-0099-FR-007**: Rustdoc, English and Spanish usage guidance,
  requirement traceability, and exact Rust public API declarations MUST be
  updated with the implementation. Generated files MUST be produced by the
  pinned repository tools; existing declarations MUST remain unchanged.

## API contract

The builder uses these semantic methods:

```rust
client.request()
    .create_split_key()
    .object_type(ObjectType::from_raw(
        kmipkit_protocol::named_values_generated::values::enumeration::object_type::SYMMETRIC_KEY,
    ))
    .method(SplitKeyMethod::XOR)
    .parts(3)
    .threshold(2)
    .unique_identifier("input-key".into())
    .attributes(attributes)
    .validate()?;
```

`prime_field_size(Vec<u8>)`, `protection_storage_masks(Structure)`,
`attribute(Attribute<D>)`, `attribute_item(Item)`, `vendor_attribute(...)`,
and `attribute_at(FluentAttributePosition, Item)` expose explicit optional or
expert inputs. Appended attributes retain call order; positioned items use a
zero-based position in the final direct Object Attributes list. The typed
response accessor continues to expose all returned Unique Identifiers in wire
order.

## Explicit limits

- This slice adds only the Rust Create Split Key builder.
- It does not implement split cryptography or choose split parameters.
- It does not add a protocol model, raw message send API, new codec, or
  transport path.
- It does not fill absent response fields, retry, Poll, or infer the server's
  object state or capabilities.
- C, Java, and Python parity is delivered in later KMIPKIT-0093 slices.

## Public Rust trait inventory

The exact Rust standard trait implementations (`Any`, `Borrow`, `BorrowMut`,
`From`, `Into`, `TryFrom`, and `TryInto`) for the public builder type are
language-level behavior, not separate KMIPKit product capabilities. The
manifest may exclude only those exact compiler-visible paths for the root and
module re-exports; the builder and its named methods remain explicit
declarations. This exact Rust standard-trait behavior is excluded from the
separate fluent product capability inventory.
