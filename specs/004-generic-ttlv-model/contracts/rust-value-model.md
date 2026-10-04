# Rust Value Model Contract

**Status**: Proposed public contract. Exact signatures are finalized only in the implementation PR after the specification and ADR-0010 are approved.

## Construction and inspection

- `RawTag::new(u32) -> Result<RawTag, ModelError>` retains an unsigned 24-bit value and rejects larger values at runtime.
- `RawTag::try_checked(&self) -> Result<Tag, ModelError>` uses the generated KMIP 2.1 allocation table. It borrows rather than consumes the raw tag, so a failed check leaves the `RawTag` available.
- `Tag` has no unchecked public constructor; its observable raw value is unchanged.
- `Value` is an opaque public struct with constructors for each item type: `structure(Structure)`, `integer(i32)`, `long_integer(i64)`, `big_integer(Vec<u8>)`, `enumeration(u32)`, `boolean(bool)`, `text_string(String)`, `byte_string(Vec<u8>)`, `date_time(i64)`, `interval(u32)`, and `date_time_extended(i64)`. Constructors take ownership of payloads, place each value in a stable boxed secret allocation, and do not expose the wrapper.
- `Item::new(Tag, Value) -> Result<Item, ModelError>` creates a typed item. `item_type()` is derived from the `Value` representation and cannot mismatch.
- `Structure::new()` creates an empty ordered collection. `try_push(Item)` appends a child unless doing so would exceed the 64-level nesting bound. Failed insertion leaves the existing structure unchanged.
- `Item::with_value(|view| ...)` lends a read-only `ValueView<'_>` only for the callback. The callback result cannot retain references borrowed from the view. `StructureView::children()` exposes borrowed child items in order; payloads remain wrapped and require their own explicit callback exposure.
- `ValueView<'a>` has variants `Structure(StructureView<'a>)`, `Integer(&'a i32)`, `LongInteger(&'a i64)`, `BigInteger(&'a [u8])`, `Enumeration(&'a u32)`, `Boolean(&'a bool)`, `TextString(&'a str)`, `ByteString(&'a [u8])`, `DateTime(&'a i64)`, `Interval(&'a u32)`, and `DateTimeExtended(&'a i64)`. It does not implement payload-revealing `Debug`, `Display`, `Clone`, or `serde::Serialize`. `StructureView` follows the same restrictions.
- Public owning `Value`, `Item`, and `Structure` do not implement `Clone` or `Copy`; explicit copies require the caller to copy data observed through a borrowed view.

## Observable guarantees

| Surface | Contract |
|---|---|
| Tag bits | `RawTag` preserves all 24 bits; checked tags retain the same bits. |
| Type | A value representation determines its KMIP Item Type. |
| Unknown values | Unknown Enumeration values and integer bit patterns remain unchanged. |
| Text/bytes | Unicode text is not normalized; bytes are not transformed. |
| Big Integer | Exact Item Value octets are retained, including sign-extension octets; empty in-memory octets do not imply wire validity. |
| Structure | Child order, duplicate tags, and nesting through 64 Structure levels are retained. A deeper insertion fails without modifying the prior tree. |
| Diagnostics | `Debug`, optional `Display`, and model errors never disclose payload values. `Value`, `ValueView`, `StructureView`, `Item`, and `Structure` do not implement `serde::Serialize`. |
| Zeroization | The proposed `zeroize` 1.9.0 safe Drop path clears current KMIPKit-owned payload storage, including full String/Vec backing capacity and nested values. Secret payloads do not reside inline in the growable Structure child vector. Caller copies, previous allocations, temporary stack/register values, and managed-runtime copies are excluded. |
| Validity | The contract covers an in-memory model and tag allocation only, not TTLV wire validity or KMIP operation semantics. |

## Compatibility and dependency direction

This feature is within the Rust core and does not expose a C ABI or language binding. New public Rust types require rustdoc. `kmipkit-ttlv` must keep `#![forbid(unsafe_code)]` and must not depend on protocol/client crates. Higher layers may convert local model errors into the shared error contract established by KMIPKIT-0003.
