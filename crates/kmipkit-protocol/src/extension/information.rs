//! Lossless local model of the KMIP Extension Information structure.

use kmipkit_ttlv::{Item, ItemType, RawTag, Structure, Value};

use crate::{ProtocolCauseCategory, ProtocolError, ProtocolErrorKind};

use super::categorized_error;
use super::definition::ExtensionDefinition;

const MAX_TEXT_BYTES_PER_FIELD: usize = 4_096;
const EXTENSION_NAME_TAG: u32 = 0x0042_00A5;
const EXTENSION_TAG_TAG: u32 = 0x0042_00A6;
const EXTENSION_TYPE_TAG: u32 = 0x0042_00A7;
const EXTENSION_ENUMERATION_TAG: u32 = 0x0042_0129;
const EXTENSION_ATTRIBUTE_TAG: u32 = 0x0042_012A;
const EXTENSION_PARENT_STRUCTURE_TAG: u32 = 0x0042_012B;
const EXTENSION_DESCRIPTION_TAG: u32 = 0x0042_012C;

/// KMIP §7.13/Table 365 metadata for a vendor extension.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExtensionInformation {
    pub(crate) name: String,
    pub(crate) tag: Option<u32>,
    pub(crate) item_type: Option<ItemType>,
    pub(crate) enumeration: Option<u32>,
    pub(crate) attribute: Option<bool>,
    pub(crate) parent_structure_tag: Option<u32>,
    pub(crate) description: Option<String>,
}

/// Creates Extension Information with its required Extension Name.
///
/// # Errors
///
/// Returns `InvalidIdentity` when the name is empty and `ResourceLimit` when
/// it exceeds the 4,096-byte metadata-field limit.
pub fn extension_information(name: &str) -> Result<ExtensionInformation, ProtocolError> {
    if name.is_empty() {
        return Err(categorized_error(ProtocolErrorKind::InvalidIdentity));
    }
    if name.len() > MAX_TEXT_BYTES_PER_FIELD {
        return Err(categorized_error(ProtocolErrorKind::ResourceLimit));
    }
    Ok(ExtensionInformation {
        name: name.to_owned(),
        tag: None,
        item_type: None,
        enumeration: None,
        attribute: None,
        parent_structure_tag: None,
        description: None,
    })
}

/// Adds an Extension Tag, represented as a non-negative KMIP Integer.
///
/// # Errors
///
/// Returns `InvalidSchema` when `tag` exceeds the 24-bit KMIP Tag width.
pub fn with_tag(
    mut information: ExtensionInformation,
    tag: u32,
) -> Result<ExtensionInformation, ProtocolError> {
    if tag > 0x00FF_FFFF {
        return Err(categorized_error(ProtocolErrorKind::InvalidSchema));
    }
    information.tag = Some(tag);
    Ok(information)
}

/// Adds an Extension Type containing the selected TTLV Item Type value.
///
/// # Errors
///
/// Returns `InvalidSchema` when the Item Type cannot be represented in the
/// Extension Type Enumeration.
pub fn with_type(
    mut information: ExtensionInformation,
    item_type: ItemType,
) -> Result<ExtensionInformation, ProtocolError> {
    item_type_value(item_type)?;
    information.item_type = Some(item_type);
    Ok(information)
}

/// Adds an Extension Enumeration, represented as a non-negative KMIP Integer.
///
/// # Errors
///
/// Returns `InvalidSchema` when `enumeration` exceeds the signed KMIP Integer
/// range.
pub fn with_enumeration(
    mut information: ExtensionInformation,
    enumeration: u32,
) -> Result<ExtensionInformation, ProtocolError> {
    if i32::try_from(enumeration).is_err() {
        return Err(categorized_error(ProtocolErrorKind::InvalidSchema));
    }
    information.enumeration = Some(enumeration);
    Ok(information)
}

/// Adds the Extension Attribute Boolean.
///
/// # Errors
///
/// This constructor currently has no failure condition and returns `Ok`.
pub fn with_attribute(
    mut information: ExtensionInformation,
    attribute: bool,
) -> Result<ExtensionInformation, ProtocolError> {
    information.attribute = Some(attribute);
    Ok(information)
}

/// Adds an Extension Parent Structure Tag as a non-negative KMIP Integer.
///
/// # Errors
///
/// Returns `InvalidSchema` when `parent_structure_tag` exceeds the 24-bit KMIP
/// Tag width.
pub fn with_parent_structure_tag(
    mut information: ExtensionInformation,
    parent_structure_tag: u32,
) -> Result<ExtensionInformation, ProtocolError> {
    if parent_structure_tag > 0x00FF_FFFF {
        return Err(categorized_error(ProtocolErrorKind::InvalidSchema));
    }
    information.parent_structure_tag = Some(parent_structure_tag);
    Ok(information)
}

/// Adds optional descriptive metadata.
///
/// # Errors
///
/// Returns `ResourceLimit` when the description exceeds the 4,096-byte field
/// limit.
pub fn with_description(
    mut information: ExtensionInformation,
    description: &str,
) -> Result<ExtensionInformation, ProtocolError> {
    if description.len() > MAX_TEXT_BYTES_PER_FIELD {
        return Err(categorized_error(ProtocolErrorKind::ResourceLimit));
    }
    information.description = Some(description.to_owned());
    Ok(information)
}

/// Converts this model into the ordered KMIP §7.13/Table 365 Structure.
///
/// Known fields are emitted in table order. The returned generic Structure
/// retains the exact represented values and remains subject to the existing
/// TTLV zeroization contract.
///
/// # Errors
///
/// Returns a payload-free protocol error if an Extension Information field
/// cannot be represented by the generic TTLV model.
pub fn to_ttlv(information: ExtensionInformation) -> Result<Structure, ProtocolError> {
    let mut structure = Structure::new();
    push_field(
        &mut structure,
        EXTENSION_NAME_TAG,
        Value::text_string(information.name),
    )?;
    if let Some(tag) = information.tag {
        push_field(
            &mut structure,
            EXTENSION_TAG_TAG,
            Value::integer(
                i32::try_from(tag)
                    .map_err(|_| categorized_error(ProtocolErrorKind::InvalidSchema))?,
            ),
        )?;
    }
    if let Some(item_type) = information.item_type {
        push_field(
            &mut structure,
            EXTENSION_TYPE_TAG,
            Value::enumeration(item_type_value(item_type)?),
        )?;
    }
    if let Some(enumeration) = information.enumeration {
        push_field(
            &mut structure,
            EXTENSION_ENUMERATION_TAG,
            Value::integer(
                i32::try_from(enumeration)
                    .map_err(|_| categorized_error(ProtocolErrorKind::InvalidSchema))?,
            ),
        )?;
    }
    if let Some(attribute) = information.attribute {
        push_field(
            &mut structure,
            EXTENSION_ATTRIBUTE_TAG,
            Value::boolean(attribute),
        )?;
    }
    if let Some(parent_tag) = information.parent_structure_tag {
        push_field(
            &mut structure,
            EXTENSION_PARENT_STRUCTURE_TAG,
            Value::integer(
                i32::try_from(parent_tag)
                    .map_err(|_| categorized_error(ProtocolErrorKind::InvalidSchema))?,
            ),
        )?;
    }
    if let Some(description) = information.description {
        push_field(
            &mut structure,
            EXTENSION_DESCRIPTION_TAG,
            Value::text_string(description),
        )?;
    }
    Ok(structure)
}

fn push_field(structure: &mut Structure, raw_tag: u32, value: Value) -> Result<(), ProtocolError> {
    let tag = RawTag::new(raw_tag)
        .and_then(|raw| raw.try_checked())
        .map_err(model_error)?;
    let item = Item::new(tag, value).map_err(model_error)?;
    structure.try_push(item).map_err(model_error)
}

fn model_error(error: kmipkit_ttlv::ModelError) -> ProtocolError {
    ProtocolError::new(
        ProtocolErrorKind::InvalidSchema,
        ProtocolCauseCategory::InvalidValue,
        error,
    )
}

fn item_type_value(item_type: ItemType) -> Result<u32, ProtocolError> {
    match item_type {
        ItemType::Structure => Ok(0x01),
        ItemType::Integer => Ok(0x02),
        ItemType::LongInteger => Ok(0x03),
        ItemType::BigInteger => Ok(0x04),
        ItemType::Enumeration => Ok(0x05),
        ItemType::Boolean => Ok(0x06),
        ItemType::TextString => Ok(0x07),
        ItemType::ByteString => Ok(0x08),
        ItemType::DateTime => Ok(0x09),
        ItemType::Interval => Ok(0x0A),
        ItemType::DateTimeExtended => Ok(0x0B),
        _ => Err(categorized_error(ProtocolErrorKind::InvalidSchema)),
    }
}

/// Attaches optional Extension Information metadata to a definition.
///
/// # Errors
///
/// This constructor currently has no failure condition and returns `Ok`.
pub fn with_information(
    mut definition: ExtensionDefinition,
    information: ExtensionInformation,
) -> Result<ExtensionDefinition, ProtocolError> {
    definition.information = Some(information);
    Ok(definition)
}

/// Returns a copy of a definition's Extension Information metadata, if present.
#[must_use]
pub fn information(definition: &ExtensionDefinition) -> Option<ExtensionInformation> {
    definition.information.clone()
}

impl ExtensionInformation {
    /// Returns the required Extension Name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the optional Extension Tag.
    #[must_use]
    pub const fn tag(&self) -> Option<u32> {
        self.tag
    }

    /// Returns the optional Extension Type.
    #[must_use]
    pub const fn item_type(&self) -> Option<ItemType> {
        self.item_type
    }

    /// Returns the optional Extension Enumeration.
    #[must_use]
    pub const fn enumeration(&self) -> Option<u32> {
        self.enumeration
    }

    /// Returns the optional Extension Attribute value.
    #[must_use]
    pub const fn attribute(&self) -> Option<bool> {
        self.attribute
    }

    /// Returns the optional Extension Parent Structure Tag.
    #[must_use]
    pub const fn parent_structure_tag(&self) -> Option<u32> {
        self.parent_structure_tag
    }

    /// Returns the optional Extension Description.
    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}
