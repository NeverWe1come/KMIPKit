# KMIPKIT-0048 Public Rust API Contract

The exact names below are the implementation contract for this specification.
Generated definitions live in `kmipkit::protocol::attribute::typed`; the
shared wrapper is `kmipkit::protocol::attribute::Attribute<D>`.

```rust
pub trait AttributeDefinition: sealed::Sealed {
    type Value;
    type ValueView<'a>;

    const TAG: u32;

    fn into_ttlv(value: Self::Value) -> kmipkit_ttlv::Value;
    fn value_view<'a>(
        value: kmipkit_ttlv::ValueView<'a>,
    ) -> Result<Self::ValueView<'a>, AttributeValueError>;
}

pub struct Attribute<D: AttributeDefinition> { /* owned Item */ }
pub struct AttributeView<'a, D: AttributeDefinition> { /* borrowed Item */ }

impl<D: AttributeDefinition> Attribute<D> {
    pub fn new(value: D::Value) -> Result<Self, AttributeValueError>;
    pub fn from_item(item: Item) -> Result<Self, AttributeValueError>;
    pub fn as_item(&self) -> &Item;
    pub fn with_value<R>(
        &self,
        callback: impl for<'a> FnOnce(D::ValueView<'a>) -> R,
    ) -> Result<R, AttributeValueError>;
}

impl AttributeSet {
    pub fn try_push_typed<D: AttributeDefinition>(
        &mut self,
        attribute: Attribute<D>,
    ) -> Result<(), ProtocolError>;
    pub fn typed<D: AttributeDefinition>(
        &self,
    ) -> impl Iterator<Item = AttributeView<'_, D>>;
}

impl CurrentAttribute {
    pub fn from_typed<D: AttributeDefinition>(attribute: Attribute<D>) -> Self;
    pub fn typed<D: AttributeDefinition>(
        &self,
    ) -> Result<AttributeView<'_, D>, AttributeValueError>;
}

impl NewAttribute {
    pub fn from_typed<D: AttributeDefinition>(attribute: Attribute<D>) -> Self;
    pub fn typed<D: AttributeDefinition>(
        &self,
    ) -> Result<AttributeView<'_, D>, AttributeValueError>;
}
```

`AttributeDefinition` is sealed so callers cannot assign standard tags or
encodings to arbitrary marker types. Generic `Item` APIs remain the extension
path. `AttributeView::with_value` uses the same scoped callback contract as
`Item::with_value`; references into text, bytes, and Structures cannot outlive
the callback. Scalar views copy only fixed-size values. No view clones an
attribute payload. `Debug` implementations for owned and borrowed typed
attributes redact their values.

Owned/view mapping is fixed by OASIS encoding:

| TTLV source category | Owned value | Callback view |
|---|---|---|
| Boolean | `bool` | `bool` |
| Byte String | `Vec<u8>` | `&[u8]` |
| Date-Time | `i64` | `i64` |
| Enumeration | `u32` | `u32` |
| Integer | `i32` | `i32` |
| Interval | `u32` | `u32` |
| Text String | `String` | `&str` |
| Structure / RNG Parameters | `Structure` | `StructureView<'a>` |
| Unique Identifier | `UniqueIdentifierValue` | `UniqueIdentifierView<'a>` |

`UniqueIdentifierValue` and its borrowed view retain the source-permitted Text
String, Enumeration, and Integer alternatives without normalizing them. An
invalid tag or value type returns a fixed, redacted `AttributeValueError`.
