//! Lossless KMIP Attribute Reference values shared by attribute operations.

use kmipkit_ttlv::{Item, ModelError, RawTag, Structure, StructureView, Tag, Value, ValueView};

use crate::{ProtocolCauseCategory, ProtocolError, ProtocolErrorKind};

const ATTRIBUTE_REFERENCE_TAG: u32 = 0x0042_013B;
const VENDOR_IDENTIFICATION_TAG: u32 = 0x0042_009D;
const ATTRIBUTE_NAME_TAG: u32 = 0x0042_000A;

/// A KMIP v2.1 §5.5 Attribute Reference, represented by name or raw tag.
///
/// Tag references retain the complete raw Enumeration value, including values
/// that are not assigned by the pinned KMIP version. Name references retain
/// the exact Vendor Identification and Attribute Name text.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct AttributeReference {
    kind: AttributeReferenceKind,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum AttributeReferenceKind {
    Tag(u32),
    Name {
        vendor_identification: String,
        attribute_name: String,
    },
}

impl AttributeReference {
    /// Creates an Attribute Reference that carries the exact raw tag value.
    #[must_use]
    pub const fn tag(raw_tag: u32) -> Self {
        Self {
            kind: AttributeReferenceKind::Tag(raw_tag),
        }
    }

    /// Creates a name-form Attribute Reference from exact source text.
    #[must_use]
    pub fn name(
        vendor_identification: impl Into<String>,
        attribute_name: impl Into<String>,
    ) -> Self {
        Self {
            kind: AttributeReferenceKind::Name {
                vendor_identification: vendor_identification.into(),
                attribute_name: attribute_name.into(),
            },
        }
    }

    /// Returns the exact raw tag Enumeration carried by a tag-form reference.
    ///
    /// Name-form references return `None`.
    #[must_use]
    pub const fn tag_value(&self) -> Option<u32> {
        match &self.kind {
            AttributeReferenceKind::Tag(raw_tag) => Some(*raw_tag),
            AttributeReferenceKind::Name { .. } => None,
        }
    }

    /// Returns the name-form Vendor Identification and Attribute Name.
    ///
    /// The returned tuple is ordered as `(vendor_identification, attribute_name)`.
    /// Tag-form references return `None`.
    #[must_use]
    pub fn name_parts(&self) -> Option<(&str, &str)> {
        match &self.kind {
            AttributeReferenceKind::Name {
                vendor_identification,
                attribute_name,
            } => Some((vendor_identification, attribute_name)),
            AttributeReferenceKind::Tag(_) => None,
        }
    }

    /// Encodes this reference as the complete outer §5.5 Attribute Reference
    /// Item, using either its Enumeration (Tag) or Structure form.
    ///
    /// # Errors
    ///
    /// Returns a sanitized protocol error if the fixed KMIP tags or generic
    /// TTLV model reject this internally defined structure.
    pub(crate) fn to_ttlv_item(&self) -> Result<Item, ProtocolError> {
        let value = match &self.kind {
            AttributeReferenceKind::Tag(raw_tag) => {
                // Outbound tag identities must pass the accepted §11.56
                // allocation policy; the decoder below keeps raw values.
                tag(*raw_tag)?;
                Value::enumeration(*raw_tag)
            }
            AttributeReferenceKind::Name {
                vendor_identification,
                attribute_name,
            } => {
                if !valid_vendor_identification(vendor_identification) {
                    return Err(malformed_reference());
                }
                let mut fields = Structure::new();
                fields
                    .try_push(item(
                        VENDOR_IDENTIFICATION_TAG,
                        Value::text_string(vendor_identification.clone()),
                    )?)
                    .map_err(model_error)?;
                fields
                    .try_push(item(
                        ATTRIBUTE_NAME_TAG,
                        Value::text_string(attribute_name.clone()),
                    )?)
                    .map_err(model_error)?;
                Value::structure(fields)
            }
        };

        item(ATTRIBUTE_REFERENCE_TAG, value)
    }

    /// Parses one complete outer §5.5 Attribute Reference Item.
    ///
    /// Unknown Enumeration values are retained as raw tag values. The caller's
    /// validated message remains the owner of the original generic TTLV tree.
    ///
    /// # Errors
    ///
    /// Returns a payload-free error if the outer tag, Item Type, or required
    /// name-form fields do not match Table 161.
    pub(crate) fn try_from_ttlv_item(item: &Item) -> Result<Self, ProtocolError> {
        if item.tag().raw() != ATTRIBUTE_REFERENCE_TAG {
            return Err(malformed_reference());
        }

        item.with_value(|value| match value {
            ValueView::Enumeration(raw_tag) => Ok(Self::tag(*raw_tag)),
            ValueView::Structure(structure) => parse_name_reference(&structure),
            _ => Err(malformed_reference()),
        })
    }
}

fn parse_name_reference(
    structure: &StructureView<'_>,
) -> Result<AttributeReference, ProtocolError> {
    let fields = structure.children();
    if fields.len() != 2
        || fields[0].tag().raw() != VENDOR_IDENTIFICATION_TAG
        || fields[1].tag().raw() != ATTRIBUTE_NAME_TAG
    {
        return Err(malformed_reference());
    }

    let vendor_identification = fields[0]
        .with_value(|value| match value {
            ValueView::TextString(text) => Some(text.to_owned()),
            _ => None,
        })
        .ok_or_else(malformed_reference)?;
    let attribute_name = fields[1]
        .with_value(|value| match value {
            ValueView::TextString(text) => Some(text.to_owned()),
            _ => None,
        })
        .ok_or_else(malformed_reference)?;

    if !valid_vendor_identification(&vendor_identification) {
        return Err(malformed_reference());
    }

    Ok(AttributeReference::name(
        vendor_identification,
        attribute_name,
    ))
}

fn valid_vendor_identification(value: &str) -> bool {
    value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '.'))
}

fn item(raw_tag: u32, value: Value) -> Result<Item, ProtocolError> {
    Item::new(tag(raw_tag)?, value).map_err(model_error)
}

fn tag(raw_tag: u32) -> Result<Tag, ProtocolError> {
    RawTag::new(raw_tag)
        .and_then(|raw| raw.try_checked())
        .map_err(model_error)
}

fn model_error(error: ModelError) -> ProtocolError {
    ProtocolError::new(
        ProtocolErrorKind::InvalidValue,
        ProtocolCauseCategory::InvalidValue,
        error,
    )
}

fn malformed_reference() -> ProtocolError {
    ProtocolError::categorized(
        ProtocolErrorKind::InvalidValue,
        ProtocolCauseCategory::InvalidValue,
    )
}
