//! Typed KMIP 2.1 Key Block and Key Value models (§§3.1–3.2, Tables 12–13).
#![allow(clippy::missing_errors_doc)] // All fallible APIs return one payload-free schema error documented on ManagedObjectError.

use std::fmt;

use kmipkit_ttlv::{Item, ItemType, StructureView, Value, ValueView};

use crate::{AttributeSet, KeyCompressionType, KeyFormatType, ManagedObjectError};

use crate::key_material::KeyMaterial;
use crate::key_wrapping::KeyWrappingData;
use crate::managed_object::{checked_tag, make_item};

const KEY_BLOCK: u32 = 0x0042_0040;
const KEY_COMPRESSION_TYPE: u32 = 0x0042_0041;
const KEY_FORMAT_TYPE: u32 = 0x0042_0042;
const KEY_VALUE: u32 = 0x0042_0045;
const CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;
const CRYPTOGRAPHIC_LENGTH: u32 = 0x0042_002A;
const KEY_WRAPPING_DATA: u32 = 0x0042_0046;
const ATTRIBUTES: u32 = 0x0042_0125;

/// A Key Value encoded as plaintext Structure or wrapped Byte String.
pub enum KeyBlockValue {
    /// Plaintext Key Value containing Key Material and optional Attributes.
    Plaintext(KeyValue),
    /// Wrapped key bytes, as required by KMIP §3.1.2.
    Wrapped(Vec<u8>),
}

/// A typed Key Block builder that emits members in Table 12 order.
pub struct KeyBlock {
    key_format_type: KeyFormatType,
    key_compression_type: Option<KeyCompressionType>,
    key_value: Option<KeyBlockValue>,
    cryptographic_algorithm: Option<u32>,
    cryptographic_length: Option<i32>,
    key_wrapping_data: Option<KeyWrappingData>,
    raw_item: Option<Item>,
}

#[allow(clippy::missing_fields_in_debug)] // KeyBlock intentionally reports secret-presence metadata without formatting key or wrapping bytes.
impl fmt::Debug for KeyBlock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("KeyBlock")
            .field("key_format_type", &self.key_format_type.raw())
            .field("has_key_value", &self.key_value.is_some())
            .field(
                "has_cryptographic_properties",
                &self.has_cryptographic_properties(),
            )
            .field("has_key_wrapping_data", &self.key_wrapping_data.is_some())
            .finish()
    }
}

impl KeyBlock {
    /// Creates a Key Block builder for the given open Key Format Type value.
    pub fn try_new(key_format_type: KeyFormatType) -> Result<Self, ManagedObjectError> {
        Ok(Self {
            key_format_type,
            key_compression_type: None,
            key_value: None,
            cryptographic_algorithm: None,
            cryptographic_length: None,
            key_wrapping_data: None,
            raw_item: None,
        })
    }

    /// Reads a source-ordered Key Block while retaining its complete TTLV tree.
    #[allow(clippy::too_many_lines)] // The source-ordered Table 12 parser validates one complete Key Block pass.
    pub fn try_from_item(item: Item) -> Result<Self, ManagedObjectError> {
        if item.tag().raw() != KEY_BLOCK || item.item_type() != ItemType::Structure {
            return Err(ManagedObjectError::InvalidStructure);
        }
        let mut present = [false; 6];
        let mut last_position = None;
        let mut key_format_type = None;
        let mut key_compression_type = None;
        let mut cryptographic_algorithm = None;
        let mut cryptographic_length = None;
        let mut value_type = None;
        let mut wrapping = false;
        item.with_value(|value| {
            let ValueView::Structure(structure) = value else {
                return Err(ManagedObjectError::InvalidStructure);
            };
            for child in structure.children() {
                let position = match child.tag().raw() {
                    KEY_FORMAT_TYPE => 0,
                    KEY_COMPRESSION_TYPE => 1,
                    KEY_VALUE => 2,
                    CRYPTOGRAPHIC_ALGORITHM => 3,
                    CRYPTOGRAPHIC_LENGTH => 4,
                    KEY_WRAPPING_DATA => 5,
                    _ => continue,
                };
                if present[position] {
                    return Err(ManagedObjectError::DuplicateField);
                }
                if last_position.is_some_and(|last| position < last) {
                    return Err(ManagedObjectError::IncorrectFieldOrder);
                }
                let valid = match (position, child.item_type()) {
                    (0, ItemType::Enumeration) => {
                        key_format_type = child.with_value(|value| match value {
                            ValueView::Enumeration(raw) => Some(KeyFormatType::from_raw(*raw)),
                            _ => None,
                        });
                        true
                    }
                    (1, ItemType::Enumeration) => {
                        key_compression_type = child.with_value(|value| match value {
                            ValueView::Enumeration(raw) => {
                                Some(crate::KeyCompressionType::from_raw(*raw))
                            }
                            _ => None,
                        });
                        true
                    }
                    (2, ItemType::Structure | ItemType::ByteString) => {
                        value_type = Some(child.item_type());
                        true
                    }
                    (3, ItemType::Enumeration) => {
                        cryptographic_algorithm = child.with_value(|value| match value {
                            ValueView::Enumeration(raw) => Some(*raw),
                            _ => None,
                        });
                        true
                    }
                    (4, ItemType::Integer) => {
                        cryptographic_length = child.with_value(|value| match value {
                            ValueView::Integer(raw) => Some(*raw),
                            _ => None,
                        });
                        true
                    }
                    (5, ItemType::Structure) => {
                        let wrapping_item = child.try_clone().map_err(ManagedObjectError::from)?;
                        KeyWrappingData::try_from_item(wrapping_item)?;
                        wrapping = true;
                        true
                    }
                    _ => false,
                };
                if !valid {
                    return Err(ManagedObjectError::IncorrectFieldType);
                }
                present[position] = true;
                last_position = Some(position);
            }
            Ok(())
        })?;
        if !present[0] {
            return Err(ManagedObjectError::MissingRequiredField);
        }
        if present[3] != present[4]
            || value_type.is_some_and(|kind| (kind == ItemType::ByteString) != wrapping)
            || (value_type.is_none() && wrapping)
        {
            return Err(ManagedObjectError::InvalidFieldCombination);
        }
        let format = key_format_type.ok_or(ManagedObjectError::IncorrectFieldType)?;
        if value_type == Some(ItemType::Structure) {
            let key_value = item.with_value(|value| match value {
                ValueView::Structure(block) => block
                    .children()
                    .iter()
                    .find(|child| child.tag().raw() == KEY_VALUE)
                    .and_then(|key_value| key_value.try_clone().ok()),
                _ => None,
            });
            let key_value = key_value.ok_or(ManagedObjectError::MissingRequiredField)?;
            KeyValue::try_from_item(key_value, format)?;
        }
        Ok(Self {
            key_format_type: format,
            key_compression_type,
            key_value: None,
            cryptographic_algorithm,
            cryptographic_length,
            key_wrapping_data: None,
            raw_item: Some(item),
        })
    }

    /// Returns the source-preserving Key Format Type.
    #[must_use]
    pub const fn key_format_type(&self) -> KeyFormatType {
        self.key_format_type
    }

    /// Returns the Key Value TTLV type, when one is present.
    #[must_use]
    pub fn key_value_type(&self) -> Option<ItemType> {
        if let Some(item) = &self.raw_item {
            return item.with_value(|value| match value {
                ValueView::Structure(structure) => structure
                    .children()
                    .iter()
                    .find(|child| child.tag().raw() == KEY_VALUE)
                    .map(Item::item_type),
                _ => None,
            });
        }
        self.key_value.as_ref().map(|value| match value {
            KeyBlockValue::Plaintext(_) => ItemType::Structure,
            KeyBlockValue::Wrapped(_) => ItemType::ByteString,
        })
    }

    /// Lends a parsed Key Block's Structure for callback-scoped inspection.
    pub fn with_raw<R>(
        &self,
        callback: impl for<'a> FnOnce(StructureView<'a>) -> R,
    ) -> Result<R, ManagedObjectError> {
        let item = self
            .raw_item
            .as_ref()
            .ok_or(ManagedObjectError::InvalidStructure)?;
        item.with_value(|value| match value {
            ValueView::Structure(structure) => Ok(callback(structure)),
            _ => Err(ManagedObjectError::InvalidStructure),
        })
    }

    /// Sets the optional compression type.
    #[must_use]
    pub fn with_key_compression_type(mut self, value: KeyCompressionType) -> Self {
        self.key_compression_type = Some(value);
        self
    }

    /// Sets the optional plaintext or wrapped Key Value.
    #[must_use]
    pub fn with_key_value(mut self, value: KeyBlockValue) -> Self {
        self.key_value = Some(value);
        self
    }

    /// Sets the optional Algorithm and Length fields, which must be paired.
    #[must_use]
    pub fn with_cryptographic_properties(
        mut self,
        algorithm: Option<u32>,
        length: Option<i32>,
    ) -> Self {
        self.cryptographic_algorithm = algorithm;
        self.cryptographic_length = length;
        self
    }

    /// Sets Key Wrapping Data.
    #[must_use]
    pub fn with_key_wrapping_data(mut self, value: KeyWrappingData) -> Self {
        self.key_wrapping_data = Some(value);
        self
    }

    /// Reports whether Algorithm or Length was supplied.
    #[must_use]
    pub const fn has_cryptographic_properties(&self) -> bool {
        self.cryptographic_algorithm.is_some() || self.cryptographic_length.is_some()
    }

    /// Consumes the builder into a source-ordered Key Block Item.
    ///
    /// # Errors
    /// Returns a sanitized error when paired fields or wrapping conditions fail.
    pub fn try_into_item(self) -> Result<Item, ManagedObjectError> {
        if let Some(item) = self.raw_item {
            return Ok(item);
        }
        if self.cryptographic_algorithm.is_some() != self.cryptographic_length.is_some() {
            return Err(ManagedObjectError::InvalidFieldCombination);
        }
        let wrapped = matches!(self.key_value.as_ref(), Some(KeyBlockValue::Wrapped(_)));
        if wrapped != self.key_wrapping_data.is_some() {
            return Err(ManagedObjectError::InvalidFieldCombination);
        }

        let mut fields = vec![make_item(
            KEY_FORMAT_TYPE,
            Value::enumeration(self.key_format_type.raw()),
        )?];
        if let Some(compression) = self.key_compression_type {
            fields.push(make_item(
                KEY_COMPRESSION_TYPE,
                Value::enumeration(compression.raw()),
            )?);
        }
        if let Some(value) = self.key_value {
            fields.push(match value {
                KeyBlockValue::Plaintext(value) => {
                    if !key_format_accepts_material(
                        self.key_format_type.raw(),
                        value.key_format_type(),
                        value.key_material_type(),
                    ) {
                        return Err(ManagedObjectError::IncorrectFieldType);
                    }
                    value.into_item()
                }
                KeyBlockValue::Wrapped(bytes) => make_item(KEY_VALUE, Value::byte_string(bytes))?,
            });
        }
        if let Some(algorithm) = self.cryptographic_algorithm {
            fields.push(make_item(
                CRYPTOGRAPHIC_ALGORITHM,
                Value::enumeration(algorithm),
            )?);
        }
        if let Some(length) = self.cryptographic_length {
            fields.push(make_item(CRYPTOGRAPHIC_LENGTH, Value::integer(length))?);
        }
        if let Some(wrapping) = self.key_wrapping_data {
            fields.push(wrapping.try_into_item()?);
        }
        let mut structure = kmipkit_ttlv::Structure::new();
        for field in fields {
            structure
                .try_push(field)
                .map_err(ManagedObjectError::from)?;
        }
        Item::new(checked_tag(KEY_BLOCK)?, Value::structure(structure))
            .map_err(ManagedObjectError::from)
    }
}

/// A plaintext Key Value Structure containing Key Material and optional Attributes.
pub struct KeyValue {
    item: Item,
    key_format_type: u32,
}

impl fmt::Debug for KeyValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("KeyValue")
            .field("key_format_type", &self.key_format_type)
            .field("item", &self.item)
            .finish()
    }
}

impl KeyValue {
    /// Creates a Key Value with required Key Material and optional attributes.
    pub fn try_new(
        key_material: KeyMaterial,
        attributes: Option<AttributeSet>,
    ) -> Result<Self, ManagedObjectError> {
        let key_format_type = key_material.format_raw();
        let key_material_item = key_material.into_item();
        let mut fields = vec![key_material_item];
        if let Some(attributes) = attributes {
            let mut structure = kmipkit_ttlv::Structure::new();
            for item in attributes.into_items() {
                structure.try_push(item).map_err(ManagedObjectError::from)?;
            }
            fields.push(make_item(ATTRIBUTES, Value::structure(structure))?);
        }
        let mut structure = kmipkit_ttlv::Structure::new();
        for field in fields {
            structure
                .try_push(field)
                .map_err(ManagedObjectError::from)?;
        }
        let item = Item::new(checked_tag(KEY_VALUE)?, Value::structure(structure))
            .map_err(ManagedObjectError::from)?;
        Ok(Self {
            item,
            key_format_type,
        })
    }

    /// Reads a plaintext Key Value Structure without dropping unknown children.
    pub fn try_from_item(
        item: Item,
        key_format_type: KeyFormatType,
    ) -> Result<Self, ManagedObjectError> {
        if item.tag().raw() != KEY_VALUE || item.item_type() != ItemType::Structure {
            return Err(ManagedObjectError::InvalidStructure);
        }
        let mut found_material = false;
        let mut found_attributes = false;
        item.with_value(|value| {
            let ValueView::Structure(structure) = value else {
                return Err(ManagedObjectError::InvalidStructure);
            };
            let mut seen_material = false;
            for child in structure.children() {
                match child.tag().raw() {
                    0x0042_0043 => {
                        if found_material
                            || found_attributes
                            || !matches!(
                                child.item_type(),
                                ItemType::ByteString | ItemType::Structure
                            )
                        {
                            return Err(ManagedObjectError::IncorrectFieldOrder);
                        }
                        found_material = true;
                        seen_material = true;
                    }
                    ATTRIBUTES => {
                        if found_attributes
                            || !seen_material
                            || child.item_type() != ItemType::Structure
                        {
                            return Err(ManagedObjectError::IncorrectFieldOrder);
                        }
                        found_attributes = true;
                    }
                    _ => {}
                }
            }
            if found_material {
                Ok(())
            } else {
                Err(ManagedObjectError::MissingRequiredField)
            }
        })?;
        if !found_material {
            return Err(ManagedObjectError::MissingRequiredField);
        }
        let material = item
            .with_value(|value| match value {
                ValueView::Structure(structure) => structure
                    .children()
                    .iter()
                    .find(|child| child.tag().raw() == 0x0042_0043)
                    .and_then(|child| child.try_clone().ok()),
                _ => None,
            })
            .ok_or(ManagedObjectError::MissingRequiredField)?;
        KeyMaterial::try_from_item(material, key_format_type)?;
        Ok(Self {
            item,
            key_format_type: key_format_type.raw(),
        })
    }

    /// Lends the Key Value Structure for callback-scoped navigation.
    pub fn with_raw<R>(
        &self,
        callback: impl for<'a> FnOnce(StructureView<'a>) -> R,
    ) -> Result<R, ManagedObjectError> {
        self.item.with_value(|value| match value {
            ValueView::Structure(structure) => Ok(callback(structure)),
            _ => Err(ManagedObjectError::InvalidStructure),
        })
    }

    fn key_format_type(&self) -> u32 {
        self.key_format_type
    }

    fn key_material_type(&self) -> ItemType {
        self.item.with_value(|value| match value {
            kmipkit_ttlv::ValueView::Structure(structure) => structure
                .children()
                .first()
                .map_or(ItemType::Structure, Item::item_type),
            _ => ItemType::Structure,
        })
    }

    /// Consumes the plaintext Key Value into its TTLV Item.
    #[must_use]
    pub fn into_item(self) -> Item {
        self.item
    }
}

fn key_format_accepts_material(format: u32, material_format: u32, item_type: ItemType) -> bool {
    if (1..=15).contains(&format) && format != material_format {
        return false;
    }
    if (1..=6).contains(&format) {
        item_type == ItemType::ByteString
    } else if (7..=15).contains(&format) {
        item_type == ItemType::Structure
    } else {
        true
    }
}
