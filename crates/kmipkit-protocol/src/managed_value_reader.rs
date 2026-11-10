//! Format-aware field reads over typed managed-object and key structures.

use std::fmt;

use kmipkit_ttlv::{Item, ItemType, ValueView};
use zeroize::Zeroizing;

use crate::{
    KeyBlock, KeyFormatType, KeyMaterial, KeyValue, KeyWrappingData, ManagedObject,
    ManagedObjectError, ObjectType,
};

/// The typed structure used to validate a managed-field read.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ManagedValueKind {
    /// One of the nine known or a future generic managed objects.
    ManagedObject(ObjectType),
    /// KMIP Key Block.
    KeyBlock,
    /// Plaintext Key Value interpreted with its enclosing open Key Format Type.
    KeyValue(KeyFormatType),
    /// Key Material interpreted with its open Key Format Type.
    KeyMaterial(KeyFormatType),
    /// KMIP Key Wrapping Data.
    KeyWrappingData,
}

/// A value copied from one managed-field read. Byte payloads wipe this owned copy on drop.
#[non_exhaustive]
pub enum ManagedFieldValue {
    /// A nested Structure field.
    Structure,
    /// A signed 32-bit Integer field.
    Integer(i32),
    /// A signed 64-bit Integer field.
    LongInteger(i64),
    /// Exact Big Integer octets in a zeroizing owner.
    BigInteger(Zeroizing<Vec<u8>>),
    /// An open unsigned Enumeration value.
    Enumeration(u32),
    /// A Boolean field.
    Boolean(bool),
    /// UTF-8 Text String octets in a zeroizing owner.
    TextString(Zeroizing<Vec<u8>>),
    /// Byte String octets in a zeroizing owner.
    ByteString(Zeroizing<Vec<u8>>),
    /// A signed Date Time value.
    DateTime(i64),
    /// An unsigned Interval value.
    Interval(u32),
    /// A signed extended Date Time value.
    DateTimeExtended(i64),
    /// A future TTLV value type retained only through the source Item.
    Unknown(ItemType),
}

impl ManagedFieldValue {
    /// Returns the TTLV type of this copied field value.
    #[must_use]
    pub const fn item_type(&self) -> ItemType {
        match self {
            Self::Structure => ItemType::Structure,
            Self::Integer(_) => ItemType::Integer,
            Self::LongInteger(_) => ItemType::LongInteger,
            Self::BigInteger(_) => ItemType::BigInteger,
            Self::Enumeration(_) => ItemType::Enumeration,
            Self::Boolean(_) => ItemType::Boolean,
            Self::TextString(_) => ItemType::TextString,
            Self::ByteString(_) => ItemType::ByteString,
            Self::DateTime(_) => ItemType::DateTime,
            Self::Interval(_) => ItemType::Interval,
            Self::DateTimeExtended(_) => ItemType::DateTimeExtended,
            Self::Unknown(item_type) => *item_type,
        }
    }

    /// Returns the open raw Enumeration value, if this field is an Enumeration.
    #[must_use]
    pub const fn enumeration(&self) -> Option<u32> {
        match self {
            Self::Enumeration(value) => Some(*value),
            _ => None,
        }
    }

    /// Returns the Integer value, if this field is an Integer.
    #[must_use]
    pub const fn integer(&self) -> Option<i32> {
        match self {
            Self::Integer(value) => Some(*value),
            _ => None,
        }
    }

    /// Lends Byte String or Big Integer octets without making another copy.
    #[must_use]
    pub fn bytes(&self) -> Option<&[u8]> {
        match self {
            Self::BigInteger(value) | Self::ByteString(value) => Some(value),
            _ => None,
        }
    }
}

impl fmt::Debug for ManagedFieldValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Structure => formatter.write_str("Structure"),
            Self::Integer(value) => formatter.debug_tuple("Integer").field(value).finish(),
            Self::LongInteger(value) => formatter.debug_tuple("LongInteger").field(value).finish(),
            Self::BigInteger(_) => formatter.write_str("BigInteger([REDACTED])"),
            Self::Enumeration(value) => formatter.debug_tuple("Enumeration").field(value).finish(),
            Self::Boolean(value) => formatter.debug_tuple("Boolean").field(value).finish(),
            Self::TextString(_) => formatter.write_str("TextString([REDACTED])"),
            Self::ByteString(_) => formatter.write_str("ByteString([REDACTED])"),
            Self::DateTime(value) => formatter.debug_tuple("DateTime").field(value).finish(),
            Self::Interval(value) => formatter.debug_tuple("Interval").field(value).finish(),
            Self::DateTimeExtended(value) => formatter
                .debug_tuple("DateTimeExtended")
                .field(value)
                .finish(),
            Self::Unknown(item_type) => formatter.debug_tuple("Unknown").field(item_type).finish(),
        }
    }
}

/// A typed, lossless owner for validating and reading direct managed fields.
pub struct ManagedValueReader {
    kind: ManagedValueKind,
    item: Item,
}

impl ManagedValueReader {
    /// Validates an Item using the selected managed-object or key-structure model.
    ///
    /// Unknown Object Types and Key Format Types remain open and retain their
    /// generic TTLV representation.
    ///
    /// # Errors
    /// Returns a sanitized model error when the selected model rejects the Item.
    pub fn try_from_item(kind: ManagedValueKind, item: Item) -> Result<Self, ManagedObjectError> {
        let item = match kind {
            ManagedValueKind::ManagedObject(object_type) => {
                ManagedObject::try_from_item(object_type, item)?.into_item()
            }
            ManagedValueKind::KeyBlock => KeyBlock::try_from_item(item)?.try_into_item()?,
            ManagedValueKind::KeyValue(format) => {
                KeyValue::try_from_item(item, format)?.into_item()
            }
            ManagedValueKind::KeyMaterial(format) => {
                KeyMaterial::try_from_item(item, format)?.into_item()
            }
            ManagedValueKind::KeyWrappingData => {
                KeyWrappingData::try_from_item(item)?.into_item()?
            }
        };
        Ok(Self { kind, item })
    }

    /// Returns the typed context used to validate this Item.
    #[must_use]
    pub const fn kind(&self) -> ManagedValueKind {
        self.kind
    }

    /// Consumes this reader into the exact source-preserving TTLV Item.
    #[must_use]
    pub fn into_item(self) -> Item {
        self.item
    }

    /// Reads the first direct field with this raw tag as a typed value.
    ///
    /// Repeated fields, unknown tags, and source order remain unchanged in the
    /// retained Item; the reader returns only the first matching field. Secret
    /// byte values are copied into a zeroizing Rust owner.
    #[must_use]
    pub fn read_field(&self, raw_tag: u32) -> Option<ManagedFieldValue> {
        self.item.with_value(|value| {
            let ValueView::Structure(structure) = value else {
                return None;
            };
            structure
                .children()
                .iter()
                .find(|field| field.tag().raw() == raw_tag)
                .map(|field| {
                    let item_type = field.item_type();
                    field.with_value(|value| copy_field_value(item_type, &value))
                })
        })
    }

    /// Returns an independently owned copy of the first direct child with this raw tag.
    ///
    /// The selected model is validated when the reader is constructed. The returned Item keeps
    /// its complete subtree, including unknown descendants and their original order.
    ///
    /// # Errors
    ///
    /// Returns [`ManagedObjectError::TtlvModelLimit`] when copying the selected subtree would
    /// exceed the TTLV model's limits.
    pub fn read_field_item(&self, raw_tag: u32) -> Result<Option<Item>, ManagedObjectError> {
        self.item.with_value(|value| {
            let ValueView::Structure(structure) = value else {
                return Ok(None);
            };
            structure
                .children()
                .iter()
                .find(|field| field.tag().raw() == raw_tag)
                .map(Item::try_clone)
                .transpose()
                .map_err(ManagedObjectError::from)
        })
    }

    /// Reports whether a direct field with this raw tag is present.
    #[must_use]
    pub fn has_field(&self, raw_tag: u32) -> bool {
        self.read_field(raw_tag).is_some()
    }

    /// Returns the direct field's TTLV type when present.
    #[must_use]
    pub fn field_type(&self, raw_tag: u32) -> Option<ItemType> {
        self.read_field(raw_tag).map(|value| value.item_type())
    }

    /// Returns the open raw Enumeration value when the field has that type.
    #[must_use]
    pub fn field_enumeration(&self, raw_tag: u32) -> Option<u32> {
        self.read_field(raw_tag)?.enumeration()
    }

    /// Returns the Integer value when the field has that type.
    #[must_use]
    pub fn field_integer(&self, raw_tag: u32) -> Option<i32> {
        self.read_field(raw_tag)?.integer()
    }

    /// Lends Byte String or Big Integer octets for the first matching field.
    pub fn with_field_bytes<R>(
        &self,
        raw_tag: u32,
        callback: impl FnOnce(&[u8]) -> R,
    ) -> Option<R> {
        let field = self.read_field(raw_tag)?;
        Some(callback(field.bytes()?))
    }
}

impl fmt::Debug for ManagedValueReader {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ManagedValueReader")
            .field("kind", &self.kind)
            .field("item_tag", &self.item.tag().raw())
            .field("item_type", &self.item.item_type())
            .finish()
    }
}

fn copy_field_value(item_type: ItemType, value: &ValueView<'_>) -> ManagedFieldValue {
    match value {
        ValueView::Structure(_) => ManagedFieldValue::Structure,
        ValueView::Integer(value) => ManagedFieldValue::Integer(**value),
        ValueView::LongInteger(value) => ManagedFieldValue::LongInteger(**value),
        ValueView::BigInteger(value) => {
            ManagedFieldValue::BigInteger(Zeroizing::new(value.to_vec()))
        }
        ValueView::Enumeration(value) => ManagedFieldValue::Enumeration(**value),
        ValueView::Boolean(value) => ManagedFieldValue::Boolean(**value),
        ValueView::TextString(value) => {
            ManagedFieldValue::TextString(Zeroizing::new(value.as_bytes().to_vec()))
        }
        ValueView::ByteString(value) => {
            ManagedFieldValue::ByteString(Zeroizing::new(value.to_vec()))
        }
        ValueView::DateTime(value) => ManagedFieldValue::DateTime(**value),
        ValueView::Interval(value) => ManagedFieldValue::Interval(**value),
        ValueView::DateTimeExtended(value) => ManagedFieldValue::DateTimeExtended(**value),
        _ => ManagedFieldValue::Unknown(item_type), // COVERAGE-EXCLUDE: ValueView currently has no unknown variant; retain this arm for future TTLV types.
    }
}

#[cfg(test)]
#[path = "../tests/support/managed_value_reader_unit.rs"]
mod tests;
