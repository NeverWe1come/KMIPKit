//! Lossless typed grouping for the direct KMIP Object Attribute items used by
//! operation attribute structures.

use std::error::Error;
use std::fmt::{self, Debug, Display};

use kmipkit_ttlv::{Item, StructureView, ValueView};

const VENDOR_ATTRIBUTE_TAG: u32 = 0x0042_0008;
const VENDOR_IDENTIFICATION_TAG: u32 = 0x0042_009D;
const ATTRIBUTE_NAME_TAG: u32 = 0x0042_000A;
const ATTRIBUTE_VALUE_TAG: u32 = 0x0042_000B;

/// An ordered collection of direct §4 Object Attribute TTLV items.
///
/// Items retain their original tag, typed value, repetitions, and insertion
/// order as required by OASIS KMIP v2.1 §§5.1–5.4, Tables 157–160. Unknown
/// assigned and extension tags remain generic items; this type does not infer
/// attribute names or synthesize tags. A Table 150 Vendor Attribute remains a
/// distinct structure and is checked for its required fields.
pub struct AttributeSet {
    items: Vec<Item>,
}

/// A payload-free error describing an invalid Vendor Attribute structure.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttributeSetError {
    /// The Vendor Attribute item is not a Structure.
    VendorAttributeMustBeStructure,
    /// The Vendor Attribute does not contain Vendor Identification.
    MissingVendorIdentification,
    /// Vendor Identification occurs more than once.
    DuplicateVendorIdentification,
    /// Vendor Identification is not a Text String.
    VendorIdentificationMustBeTextString,
    /// Vendor Identification contains a character outside `[A-Za-z0-9_.]`.
    InvalidVendorIdentification,
    /// The Vendor Attribute does not contain Attribute Name.
    MissingAttributeName,
    /// Attribute Name occurs more than once.
    DuplicateAttributeName,
    /// Attribute Name is not a Text String.
    AttributeNameMustBeTextString,
    /// The Vendor Attribute does not contain Attribute Value.
    MissingAttributeValue,
    /// Attribute Value occurs more than once.
    DuplicateAttributeValue,
}

impl Display for AttributeSetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::VendorAttributeMustBeStructure => "Vendor Attribute must be a Structure",
            Self::MissingVendorIdentification => {
                "Vendor Attribute is missing Vendor Identification"
            }
            Self::DuplicateVendorIdentification => "Vendor Identification is repeated",
            Self::VendorIdentificationMustBeTextString => {
                "Vendor Identification must be a Text String"
            }
            Self::InvalidVendorIdentification => "Vendor Identification has invalid characters",
            Self::MissingAttributeName => "Vendor Attribute is missing Attribute Name",
            Self::DuplicateAttributeName => "Attribute Name is repeated",
            Self::AttributeNameMustBeTextString => "Attribute Name must be a Text String",
            Self::MissingAttributeValue => "Vendor Attribute is missing Attribute Value",
            Self::DuplicateAttributeValue => "Attribute Value is repeated",
        };
        formatter.write_str(message)
    }
}

impl Error for AttributeSetError {}

impl Debug for AttributeSet {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AttributeSet")
            .field("item_count", &self.items.len())
            .finish()
    }
}

impl AttributeSet {
    /// Creates an empty attribute set.
    #[must_use]
    pub const fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Creates a set from direct attribute items in their original order.
    ///
    /// Repeated tags and generic values are retained. A Vendor Attribute is
    /// validated against the distinct structure in §4.60, Table 150.
    ///
    /// # Errors
    ///
    /// Returns a payload-free [`AttributeSetError`] when a Vendor Attribute
    /// has an invalid structure or required field.
    pub fn try_new(items: impl IntoIterator<Item = Item>) -> Result<Self, AttributeSetError> {
        let mut set = Self::new();
        for item in items {
            set.try_push(item)?;
        }
        Ok(set)
    }

    /// Appends one direct attribute item while preserving its original tag.
    ///
    /// # Errors
    ///
    /// Returns a payload-free [`AttributeSetError`] for an invalid Vendor
    /// Attribute structure. Other generic attribute values remain unchanged.
    pub fn try_push(&mut self, item: Item) -> Result<(), AttributeSetError> {
        if item.tag().raw() == VENDOR_ATTRIBUTE_TAG {
            item.with_value(|value| match value {
                ValueView::Structure(structure) => validate_vendor_attribute(&structure),
                _ => Err(AttributeSetError::VendorAttributeMustBeStructure),
            })?;
        }
        self.items.push(item);
        Ok(())
    }

    /// Returns the direct items in wire order.
    #[must_use]
    pub fn as_items(&self) -> &[Item] {
        &self.items
    }

    /// Returns the number of direct attribute items.
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns whether this set contains no direct attribute items.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Consumes the set and returns its direct items in wire order.
    #[must_use]
    pub fn into_items(self) -> Vec<Item> {
        self.items
    }
}

impl Default for AttributeSet {
    fn default() -> Self {
        Self::new()
    }
}

fn validate_vendor_attribute(structure: &StructureView<'_>) -> Result<(), AttributeSetError> {
    let mut vendor_identification = false;
    let mut attribute_name = false;
    let mut attribute_value = false;

    for field in structure.children() {
        match field.tag().raw() {
            VENDOR_IDENTIFICATION_TAG => {
                if vendor_identification {
                    return Err(AttributeSetError::DuplicateVendorIdentification);
                }
                vendor_identification = true;
                let valid_identifier = field.with_value(|value| match value {
                    ValueView::TextString(identifier) => {
                        Some(identifier.chars().all(|character| {
                            character.is_ascii_alphanumeric() || matches!(character, '_' | '.')
                        }))
                    }
                    _ => None,
                });
                match valid_identifier {
                    Some(true) => {}
                    Some(false) => return Err(AttributeSetError::InvalidVendorIdentification),
                    None => return Err(AttributeSetError::VendorIdentificationMustBeTextString),
                }
            }
            ATTRIBUTE_NAME_TAG => {
                if attribute_name {
                    return Err(AttributeSetError::DuplicateAttributeName);
                }
                attribute_name = true;
                let is_text = field.with_value(|value| matches!(value, ValueView::TextString(_)));
                if !is_text {
                    return Err(AttributeSetError::AttributeNameMustBeTextString);
                }
            }
            ATTRIBUTE_VALUE_TAG => {
                if attribute_value {
                    return Err(AttributeSetError::DuplicateAttributeValue);
                }
                attribute_value = true;
            }
            _ => {}
        }
    }

    if !vendor_identification {
        return Err(AttributeSetError::MissingVendorIdentification);
    }
    if !attribute_name {
        return Err(AttributeSetError::MissingAttributeName);
    }
    if !attribute_value {
        return Err(AttributeSetError::MissingAttributeValue);
    }
    Ok(())
}
