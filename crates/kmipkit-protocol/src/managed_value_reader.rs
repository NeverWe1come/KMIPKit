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
        _ => ManagedFieldValue::Unknown(item_type),
    }
}

#[cfg(test)]
mod tests {
    use kmipkit_ttlv::{ItemType, RawTag, Structure, Value, ValueView};

    use super::{ManagedValueKind, ManagedValueReader};
    use crate::{Certificate, KeyFormatType, ObjectType};

    fn tag(raw: u32) -> kmipkit_ttlv::Tag {
        RawTag::new(raw)
            .expect("fixture tag fits")
            .try_checked()
            .expect("fixture tag is allocated")
    }

    #[test]
    fn typed_reader_keeps_open_context_repeats_and_source_order() {
        let field_tag = 0x0054_0001;
        let mut structure = Structure::new();
        structure
            .try_push(
                kmipkit_ttlv::Item::new(tag(field_tag), Value::enumeration(0x8000_0101))
                    .expect("first vendor value"),
            )
            .expect("first vendor field fits");
        structure
            .try_push(
                kmipkit_ttlv::Item::new(tag(field_tag), Value::enumeration(0x8000_0102))
                    .expect("repeated vendor value"),
            )
            .expect("repeated vendor field fits");
        let item = kmipkit_ttlv::Item::new(tag(0x0054_0002), Value::structure(structure))
            .expect("vendor object");
        let reader = ManagedValueReader::try_from_item(
            ManagedValueKind::ManagedObject(ObjectType::from_raw(0x8000_0001)),
            item,
        )
        .expect("future object context stays open");

        assert!(reader.has_field(field_tag));
        assert_eq!(reader.field_type(field_tag), Some(ItemType::Enumeration));
        assert_eq!(reader.field_enumeration(field_tag), Some(0x8000_0101));
        assert_eq!(reader.field_integer(field_tag), None);

        let item = reader.into_item();
        item.with_value(|value| {
            let ValueView::Structure(structure) = value else {
                panic!("vendor object remains a Structure")
            };
            assert_eq!(
                structure
                    .children()
                    .iter()
                    .map(|child| child.tag().raw())
                    .collect::<Vec<_>>(),
                [field_tag, field_tag]
            );
        });
    }

    #[test]
    fn typed_reader_zeroizes_and_redacts_copied_secret_bytes() {
        let item = Certificate::try_new(0x8000_0042, b"secret-canary".to_vec())
            .expect("certificate")
            .into_item();
        let reader = ManagedValueReader::try_from_item(
            ManagedValueKind::ManagedObject(ObjectType::from_raw(1)),
            item,
        )
        .expect("certificate model validates");
        let value = reader
            .read_field(0x0042_001e)
            .expect("certificate bytes are present");
        assert_eq!(value.item_type(), ItemType::ByteString);
        assert_eq!(value.bytes(), Some(b"secret-canary".as_slice()));
        assert!(!format!("{value:?}").contains("secret-canary"));
        assert!(!format!("{reader:?}").contains("secret-canary"));
        assert!(!reader.has_field(0x0054_0001));
    }

    #[test]
    fn typed_reader_returns_owned_nested_item_with_repeated_vendor_children() {
        let vendor_tag = 0x0054_0001;
        let mut attributes = Structure::new();
        for value in [b"first".as_slice(), b"second".as_slice()] {
            attributes
                .try_push(
                    kmipkit_ttlv::Item::new(tag(vendor_tag), Value::byte_string(value.to_vec()))
                        .expect("vendor child"),
                )
                .expect("vendor child fits");
        }
        let attributes = kmipkit_ttlv::Item::new(tag(0x0042_0125), Value::structure(attributes))
            .expect("Attributes");
        let mut key_value = Structure::new();
        key_value
            .try_push(
                crate::KeyMaterial::byte_string(1, b"key-data".to_vec())
                    .expect("Key Material")
                    .into_item(),
            )
            .expect("Key Material child fits");
        key_value
            .try_push(attributes)
            .expect("Attributes child fits");
        let item = kmipkit_ttlv::Item::new(tag(0x0042_0045), Value::structure(key_value))
            .expect("Key Value");
        let reader = ManagedValueReader::try_from_item(
            ManagedValueKind::KeyValue(KeyFormatType::from_raw(1)),
            item,
        )
        .expect("Key Value validates");

        let nested = reader
            .read_field_item(0x0042_0125)
            .expect("nested item clone succeeds")
            .expect("Attributes is present");
        assert_eq!(nested.tag().raw(), 0x0042_0125);
        nested.with_value(|value| {
            let ValueView::Structure(structure) = value else {
                panic!("Attributes remains a Structure")
            };
            assert_eq!(
                structure
                    .children()
                    .iter()
                    .map(|child| child.tag().raw())
                    .collect::<Vec<_>>(),
                [vendor_tag, vendor_tag]
            );
        });
        assert!(
            reader
                .read_field_item(0x0054_0002)
                .expect("missing lookup succeeds")
                .is_none()
        );
    }

    #[test]
    fn typed_reader_rejects_known_object_mismatch_and_preserves_future_format() {
        let certificate = Certificate::try_new(1, vec![1])
            .expect("certificate")
            .into_item();
        assert!(
            ManagedValueReader::try_from_item(
                ManagedValueKind::ManagedObject(ObjectType::from_raw(8)),
                certificate,
            )
            .is_err()
        );

        let key_material = crate::KeyMaterial::byte_string(0x8000_0001, b"future".to_vec())
            .expect("open Key Format Type")
            .into_item();
        let reader = ManagedValueReader::try_from_item(
            ManagedValueKind::KeyMaterial(KeyFormatType::from_raw(0x8000_0001)),
            key_material,
        )
        .expect("future byte-string format remains open");
        assert_eq!(
            reader.kind(),
            ManagedValueKind::KeyMaterial(KeyFormatType::from_raw(0x8000_0001))
        );
        assert_eq!(reader.field_type(0x0042_0043), None);
        assert_eq!(reader.into_item().item_type(), ItemType::ByteString);
    }
}
