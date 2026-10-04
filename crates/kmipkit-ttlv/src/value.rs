// Opaque, typed representations of KMIP TTLV Item Values.

use std::fmt::{self, Debug};

use crate::structure::{Structure, StructureView};
use zeroize::Zeroize;

/// The KMIP Item Type represented by a [`Value`].
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ItemType {
    /// An ordered collection of child items.
    Structure,
    /// A signed 32-bit integer.
    Integer,
    /// A signed 64-bit integer.
    LongInteger,
    /// Exact Big Integer Item Value octets.
    BigInteger,
    /// An unsigned 32-bit enumeration value.
    Enumeration,
    /// A Boolean value.
    Boolean,
    /// Unicode text without normalization.
    TextString,
    /// An arbitrary sequence of bytes.
    ByteString,
    /// A signed 64-bit Date Time value.
    DateTime,
    /// An unsigned 32-bit interval.
    Interval,
    /// A signed 64-bit Date Time Extended value.
    DateTimeExtended,
}

/// An opaque owning KMIP Item Value.
///
/// The value representation is boxed so payload addresses remain stable when
/// its containing item is moved as part of an ordered Structure. Payloads are
/// observable only through [`crate::Item::with_value`]. Dropping the value
/// zeroizes its currently owned payload storage, including the current backing
/// allocations for `BigInteger` and `ByteString` vectors and `TextString`
/// strings. The pinned `zeroize` 1.9.0 implementation clears initialized
/// elements and the full current `Vec` capacity; `String` delegates to its
/// backing vector. Payloads in nested Structures are zeroized recursively.
/// Moving a payload into this value does not clear earlier allocations or
/// copies made before ownership transfer. Callers can also make
/// copies from closure-scoped borrowed views, and those copies are outside this
/// guarantee. See the [public API guide](../../../docs/architecture/public-api.md#secrets)
/// for the complete limits and dependency reference.
pub struct Value {
    inner: Secret<ValueRepr>,
}

struct Secret<T: Zeroize> {
    // Keep payload bytes outside the growable Structure child vector.
    boxed: Box<T>,
}

impl<T: Zeroize> Secret<T> {
    fn new(value: T) -> Self {
        Self {
            boxed: Box::new(value),
        }
    }
}

impl<T: Zeroize> Drop for Secret<T> {
    fn drop(&mut self) {
        self.boxed.as_mut().zeroize();
    }
}

enum ValueRepr {
    Structure(Structure),
    Integer(i32),
    LongInteger(i64),
    BigInteger(Vec<u8>),
    Enumeration(u32),
    Boolean(bool),
    TextString(String),
    ByteString(Vec<u8>),
    DateTime(i64),
    Interval(u32),
    DateTimeExtended(i64),
}

impl Debug for Value {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let child_count = match self.inner.boxed.as_ref() {
            ValueRepr::Structure(structure) => Some(structure.as_view().children().len()),
            _ => None,
        };

        debug_value_metadata(formatter, "Value", self.item_type(), child_count)
    }
}

impl Debug for ValueView<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (item_type, child_count) = match self {
            Self::Structure(structure) => (ItemType::Structure, Some(structure.children().len())),
            Self::Integer(_) => (ItemType::Integer, None),
            Self::LongInteger(_) => (ItemType::LongInteger, None),
            Self::BigInteger(_) => (ItemType::BigInteger, None),
            Self::Enumeration(_) => (ItemType::Enumeration, None),
            Self::Boolean(_) => (ItemType::Boolean, None),
            Self::TextString(_) => (ItemType::TextString, None),
            Self::ByteString(_) => (ItemType::ByteString, None),
            Self::DateTime(_) => (ItemType::DateTime, None),
            Self::Interval(_) => (ItemType::Interval, None),
            Self::DateTimeExtended(_) => (ItemType::DateTimeExtended, None),
        };

        debug_value_metadata(formatter, "ValueView", item_type, child_count)
    }
}

fn debug_value_metadata(
    formatter: &mut fmt::Formatter<'_>,
    name: &str,
    item_type: ItemType,
    child_count: Option<usize>,
) -> fmt::Result {
    let mut debug = formatter.debug_struct(name);
    debug.field("item_type", &item_type);

    if let Some(child_count) = child_count {
        debug.field("child_count", &child_count);
    }

    debug.finish()
}

impl Zeroize for ValueRepr {
    fn zeroize(&mut self) {
        match self {
            Self::Structure(value) => value.zeroize_payloads(),
            Self::Integer(value) => value.zeroize(),
            Self::LongInteger(value) | Self::DateTime(value) | Self::DateTimeExtended(value) => {
                value.zeroize();
            }
            Self::BigInteger(value) | Self::ByteString(value) => value.zeroize(),
            Self::Enumeration(value) | Self::Interval(value) => value.zeroize(),
            Self::Boolean(value) => value.zeroize(),
            Self::TextString(value) => value.zeroize(),
        }
    }
}

/// A read-only, borrowed view of an Item Value.
///
/// Views are lent by [`crate::Item::with_value`] and cannot be returned from
/// that callback while retaining a borrow from the item.
#[non_exhaustive]
pub enum ValueView<'a> {
    /// A borrowed view of an ordered Structure.
    Structure(StructureView<'a>),
    /// A borrowed signed 32-bit integer.
    Integer(&'a i32),
    /// A borrowed signed 64-bit integer.
    LongInteger(&'a i64),
    /// Borrowed exact Big Integer Item Value octets.
    BigInteger(&'a [u8]),
    /// A borrowed unsigned 32-bit enumeration value.
    Enumeration(&'a u32),
    /// A borrowed Boolean value.
    Boolean(&'a bool),
    /// Borrowed Unicode text.
    TextString(&'a str),
    /// Borrowed arbitrary bytes.
    ByteString(&'a [u8]),
    /// A borrowed signed 64-bit Date Time value.
    DateTime(&'a i64),
    /// A borrowed unsigned 32-bit interval.
    Interval(&'a u32),
    /// A borrowed signed 64-bit Date Time Extended value.
    DateTimeExtended(&'a i64),
}

impl Value {
    /// Creates a Structure value.
    #[must_use]
    pub fn structure(value: Structure) -> Self {
        Self::new(ValueRepr::Structure(value))
    }

    /// Creates an Integer value without changing its signed 32-bit pattern.
    #[must_use]
    pub fn integer(value: i32) -> Self {
        Self::new(ValueRepr::Integer(value))
    }

    /// Creates a Long Integer value without changing its signed 64-bit pattern.
    #[must_use]
    pub fn long_integer(value: i64) -> Self {
        Self::new(ValueRepr::LongInteger(value))
    }

    /// Creates a Big Integer value from its exact Item Value octets.
    ///
    /// This in-memory constructor preserves empty octets and does not establish
    /// that a value is valid for TTLV wire encoding.
    #[must_use]
    pub fn big_integer(value: Vec<u8>) -> Self {
        Self::new(ValueRepr::BigInteger(value))
    }

    /// Creates an Enumeration value while preserving all 32 bits.
    #[must_use]
    pub fn enumeration(value: u32) -> Self {
        Self::new(ValueRepr::Enumeration(value))
    }

    /// Creates a Boolean value.
    #[must_use]
    pub fn boolean(value: bool) -> Self {
        Self::new(ValueRepr::Boolean(value))
    }

    /// Creates a Text String value without normalizing its Unicode text.
    #[must_use]
    pub fn text_string(value: String) -> Self {
        Self::new(ValueRepr::TextString(value))
    }

    /// Creates a Byte String value while preserving every byte.
    #[must_use]
    pub fn byte_string(value: Vec<u8>) -> Self {
        Self::new(ValueRepr::ByteString(value))
    }

    /// Creates a Date Time value without changing its signed 64-bit pattern.
    #[must_use]
    pub fn date_time(value: i64) -> Self {
        Self::new(ValueRepr::DateTime(value))
    }

    /// Creates an Interval value while preserving all 32 bits.
    #[must_use]
    pub fn interval(value: u32) -> Self {
        Self::new(ValueRepr::Interval(value))
    }

    /// Creates a Date Time Extended value without changing its signed 64-bit pattern.
    #[must_use]
    pub fn date_time_extended(value: i64) -> Self {
        Self::new(ValueRepr::DateTimeExtended(value))
    }

    fn new(value: ValueRepr) -> Self {
        Self {
            inner: Secret::new(value),
        }
    }

    pub(crate) fn item_type(&self) -> ItemType {
        match self.inner.boxed.as_ref() {
            ValueRepr::Structure(_) => ItemType::Structure,
            ValueRepr::Integer(_) => ItemType::Integer,
            ValueRepr::LongInteger(_) => ItemType::LongInteger,
            ValueRepr::BigInteger(_) => ItemType::BigInteger,
            ValueRepr::Enumeration(_) => ItemType::Enumeration,
            ValueRepr::Boolean(_) => ItemType::Boolean,
            ValueRepr::TextString(_) => ItemType::TextString,
            ValueRepr::ByteString(_) => ItemType::ByteString,
            ValueRepr::DateTime(_) => ItemType::DateTime,
            ValueRepr::Interval(_) => ItemType::Interval,
            ValueRepr::DateTimeExtended(_) => ItemType::DateTimeExtended,
        }
    }

    pub(crate) fn as_view(&self) -> ValueView<'_> {
        match self.inner.boxed.as_ref() {
            ValueRepr::Structure(value) => ValueView::Structure(value.as_view()),
            ValueRepr::Integer(value) => ValueView::Integer(value),
            ValueRepr::LongInteger(value) => ValueView::LongInteger(value),
            ValueRepr::BigInteger(value) => ValueView::BigInteger(value),
            ValueRepr::Enumeration(value) => ValueView::Enumeration(value),
            ValueRepr::Boolean(value) => ValueView::Boolean(value),
            ValueRepr::TextString(value) => ValueView::TextString(value),
            ValueRepr::ByteString(value) => ValueView::ByteString(value),
            ValueRepr::DateTime(value) => ValueView::DateTime(value),
            ValueRepr::Interval(value) => ValueView::Interval(value),
            ValueRepr::DateTimeExtended(value) => ValueView::DateTimeExtended(value),
        }
    }

    pub(crate) fn zeroize_payloads(&mut self) {
        self.inner.boxed.as_mut().zeroize();
    }
}
