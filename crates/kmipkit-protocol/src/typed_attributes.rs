//! Typed, catalog-backed views and construction for direct KMIP attributes.

use std::error::Error;
use std::fmt::{self, Debug, Display};
use std::marker::PhantomData;

use kmipkit_ttlv::{Item, RawTag, Value, ValueView};

/// A sealed description of one standard direct KMIP attribute.
pub trait AttributeDefinition: sealed::Sealed + 'static {
    /// The owned value accepted by [`Attribute::new`].
    type Value;

    /// The scoped borrowed value returned by [`Attribute::with_value`].
    type ValueView<'a>
    where
        Self: 'a;

    /// The assigned KMIP 2.1 tag for this attribute.
    const TAG: u32;

    /// Encodes the value using the catalogued TTLV Item Type.
    fn into_ttlv(value: Self::Value) -> Value;

    /// Interprets one value view using the catalogued TTLV Item Type.
    fn value_view<'a>(value: ValueView<'a>) -> Result<Self::ValueView<'a>, AttributeValueError>
    where
        Self: 'a;
}

mod sealed {
    pub trait Sealed {}
}

/// A redacted error returned when typed attribute construction or reading fails.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttributeValueError {
    /// The marker's catalogued tag could not be constructed as an allocated tag.
    InvalidTag,
    /// The supplied item has a different tag from the marker's tag.
    TagMismatch,
    /// The supplied item does not use a TTLV type permitted by the marker.
    ValueTypeMismatch,
    /// The generic TTLV item model rejected construction.
    ItemConstruction,
}

impl Display for AttributeValueError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidTag => "typed attribute tag is not allocated",
            Self::TagMismatch => "typed attribute item has a different tag",
            Self::ValueTypeMismatch => "typed attribute item has an unexpected value type",
            Self::ItemConstruction => "typed attribute item could not be constructed",
        };
        formatter.write_str(message)
    }
}

impl Error for AttributeValueError {}

/// An owned direct KMIP attribute paired with its catalogued marker type.
pub struct Attribute<D: AttributeDefinition> {
    item: Item,
    marker: PhantomData<fn() -> D>,
}

impl<D: AttributeDefinition> Attribute<D> {
    /// Creates a direct attribute Item using its catalogued tag and encoding.
    ///
    /// # Errors
    ///
    /// Returns a redacted [`AttributeValueError`] if the generated tag is not
    /// accepted by the local tag catalog or the TTLV item model rejects the
    /// value.
    pub fn new(value: D::Value) -> Result<Self, AttributeValueError> {
        let tag = RawTag::new(D::TAG)
            .and_then(|tag| tag.try_checked())
            .map_err(|_| AttributeValueError::InvalidTag)?;
        let item = Item::new(tag, D::into_ttlv(value))
            .map_err(|_| AttributeValueError::ItemConstruction)?;
        Ok(Self {
            item,
            marker: PhantomData,
        })
    }

    /// Consumes and validates a generic Item as this typed attribute.
    ///
    /// # Errors
    ///
    /// Returns a redacted error if the tag or value type does not match the
    /// marker.
    pub fn from_item(item: Item) -> Result<Self, AttributeValueError> {
        validate_item::<D>(&item)?;
        Ok(Self {
            item,
            marker: PhantomData,
        })
    }

    /// Borrows the underlying generic Item without changing its representation.
    #[must_use]
    pub const fn as_item(&self) -> &Item {
        &self.item
    }

    /// Consumes this typed wrapper and returns its generic Item.
    #[must_use]
    pub fn into_item(self) -> Item {
        self.item
    }

    /// Borrows the value for the duration of `callback`.
    ///
    /// References into text, bytes, and Structures cannot outlive the callback.
    ///
    /// # Errors
    ///
    /// Returns a redacted error if an invariant of the owned Item does not
    /// match the typed marker.
    pub fn with_value<R>(
        &self,
        callback: impl for<'a> FnOnce(D::ValueView<'a>) -> R,
    ) -> Result<R, AttributeValueError> {
        self.item
            .with_value(|value| D::value_view(value).map(callback))
    }
}

impl<D: AttributeDefinition> Debug for Attribute<D> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Attribute")
            .field("tag", &self.item.tag())
            .field("value", &"[REDACTED]")
            .finish()
    }
}

/// A borrowed direct attribute Item paired with its catalogued marker type.
pub struct AttributeView<'a, D: AttributeDefinition> {
    item: &'a Item,
    marker: PhantomData<fn() -> D>,
}

impl<'a, D: AttributeDefinition> AttributeView<'a, D> {
    pub(crate) fn try_from_item(item: &'a Item) -> Result<Self, AttributeValueError> {
        validate_item::<D>(item)?;
        Ok(Self::from_catalogued_item(item))
    }

    // AttributeSet validates the catalogued TTLV type before storing Items.
    // Its private storage and checked insertion make matching tags safe here.
    pub(crate) const fn from_catalogued_item(item: &'a Item) -> Self {
        Self {
            item,
            marker: PhantomData,
        }
    }

    /// Borrows the underlying generic Item.
    #[must_use]
    pub const fn as_item(&self) -> &'a Item {
        self.item
    }

    /// Borrows the value for the duration of `callback`.
    ///
    /// # Errors
    ///
    /// Returns a redacted error if the Item's value type does not match the
    /// marker.
    pub fn with_value<R>(
        &self,
        callback: impl for<'value> FnOnce(D::ValueView<'value>) -> R,
    ) -> Result<R, AttributeValueError> {
        self.item
            .with_value(|value| D::value_view(value).map(callback))
    }
}

impl<D: AttributeDefinition> Debug for AttributeView<'_, D> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AttributeView")
            .field("tag", &self.item.tag())
            .field("value", &"[REDACTED]")
            .finish()
    }
}

fn validate_item<D: AttributeDefinition>(item: &Item) -> Result<(), AttributeValueError> {
    if item.tag().raw() != D::TAG {
        return Err(AttributeValueError::TagMismatch);
    }
    item.with_value(|value| D::value_view(value).map(drop))
}

/// One source-permitted value representation for the Unique Identifier attribute.
pub enum UniqueIdentifierValue {
    /// A Unicode Text String identifier.
    TextString(String),
    /// An Enumeration identifier, including values unknown to this version.
    Enumeration(u32),
    /// A signed Integer identifier.
    Integer(i32),
}

impl Clone for UniqueIdentifierValue {
    fn clone(&self) -> Self {
        match self {
            Self::TextString(value) => Self::TextString(value.clone()),
            Self::Enumeration(value) => Self::Enumeration(*value),
            Self::Integer(value) => Self::Integer(*value),
        }
    }
}

impl PartialEq for UniqueIdentifierValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::TextString(left), Self::TextString(right)) => left == right,
            (Self::Enumeration(left), Self::Enumeration(right)) => left == right,
            (Self::Integer(left), Self::Integer(right)) => left == right,
            _ => false,
        }
    }
}

impl Eq for UniqueIdentifierValue {}

impl Debug for UniqueIdentifierValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TextString(_) => {
                formatter.write_str("UniqueIdentifierValue::TextString([REDACTED])")
            }
            Self::Enumeration(_) => {
                formatter.write_str("UniqueIdentifierValue::Enumeration([REDACTED])")
            }
            Self::Integer(_) => formatter.write_str("UniqueIdentifierValue::Integer([REDACTED])"),
        }
    }
}

/// A borrowed view of a source-permitted Unique Identifier value.
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum UniqueIdentifierView<'a> {
    /// A borrowed Unicode Text String identifier.
    TextString(&'a str),
    /// An Enumeration identifier, including values unknown to this version.
    Enumeration(u32),
    /// A signed Integer identifier.
    Integer(i32),
}

impl Debug for UniqueIdentifierView<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TextString(_) => {
                formatter.write_str("UniqueIdentifierView::TextString([REDACTED])")
            }
            Self::Enumeration(_) => {
                formatter.write_str("UniqueIdentifierView::Enumeration([REDACTED])")
            }
            Self::Integer(_) => formatter.write_str("UniqueIdentifierView::Integer([REDACTED])"),
        }
    }
}

#[path = "typed_attributes_generated.rs"]
mod generated;

pub use generated::typed;

#[cfg(test)]
pub(crate) use generated::ATTRIBUTE_DEFINITIONS;
