//! Lossless typed grouping for the direct KMIP Object Attribute items used by
//! operation attribute structures.

use std::collections::HashSet;
use std::error::Error;
use std::fmt::{self, Debug, Display};

use kmipkit_ttlv::{Item, ModelError, StructureView, Value, ValueView};

use crate::{ProtocolCauseCategory, ProtocolError, ProtocolErrorKind};

const VENDOR_ATTRIBUTE_TAG: u32 = 0x0042_0008;
const VENDOR_IDENTIFICATION_TAG: u32 = 0x0042_009D;
const ATTRIBUTE_NAME_TAG: u32 = 0x0042_000A;
const ATTRIBUTE_VALUE_TAG: u32 = 0x0042_000B;
const CERTIFICATE_TYPE_TAG: u32 = 0x0042_001D;
const OBJECT_TYPE_TAG: u32 = 0x0042_0057;
const KEY_BLOCK_TAG: u32 = 0x0042_0040;
const KEY_VALUE_TAG: u32 = 0x0042_0045;
const ATTRIBUTES_STRUCTURE_TAG: u32 = 0x0042_0125;
const CERTIFICATE_TYPE_X509: u32 = 1;
const OBJECT_TYPE_PGP_KEY: u32 = 9;
const OBJECT_TYPE_CERTIFICATE: u32 = 1;

/// A client-initiated KMIP operation that can mutate an object attribute.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClientAttributeMutation {
    /// Add a distinct attribute value.
    Add,
    /// Adjust an existing attribute value.
    Adjust,
    /// Delete an attribute value.
    Delete,
    /// Modify an existing attribute value.
    Modify,
    /// Set an attribute value.
    Set,
}

impl ClientAttributeMutation {
    const fn operation_name(self) -> &'static str {
        match self {
            Self::Add => "Add Attribute",
            Self::Adjust => "Adjust Attribute",
            Self::Delete => "Delete Attribute",
            Self::Modify => "Modify Attribute",
            Self::Set => "Set Attribute",
        }
    }
}

/// Returns whether the generated source-backed policy unconditionally
/// prohibits a client mutation of a catalogued standard attribute.
///
/// Unknown tags return `false`; `KMIPKit` does not infer policy for unrecognized
/// standard, extension, or vendor attributes. Qualified attribute rules that
/// require remote object state remain the server's decision.
#[must_use]
pub fn client_attribute_mutation_is_prohibited(
    tag: u32,
    mutation: ClientAttributeMutation,
) -> bool {
    let Some(policy) = crate::attribute_policy::ATTRIBUTE_POLICIES
        .iter()
        .find(|policy| policy.tag == tag)
    else {
        return false;
    };

    let client_capability = match mutation {
        ClientAttributeMutation::Delete => policy.source_deletable_by_client,
        ClientAttributeMutation::Add
        | ClientAttributeMutation::Adjust
        | ClientAttributeMutation::Modify
        | ClientAttributeMutation::Set => policy.source_modifiable_by_client,
    };
    source_says_no(client_capability)
        || policy.source_operation_restrictions.iter().any(|rule| {
            let text = rule.source_text;
            (text.contains("SHALL NOT") || text.contains("MUST NOT"))
                && text.contains(mutation.operation_name())
        })
}

/// Returns whether the generated §4.60 policy prohibits the supplied client
/// mutation for this Vendor Identification.
#[must_use]
pub fn client_vendor_attribute_mutation_is_prohibited(
    vendor_identification: &str,
    mutation: ClientAttributeMutation,
) -> bool {
    let policy = crate::attribute_policy::VENDOR_ATTRIBUTE_POLICY;
    policy.matches_vendor_identification(vendor_identification)
        && policy
            .prohibited_client_operations
            .contains(&mutation.operation_name())
}

fn source_says_no(value: &str) -> bool {
    value == "No" || value.starts_with("No,") || value.starts_with("No ")
}

pub(crate) fn copy_text_string(value: &ValueView<'_>) -> Option<String> {
    match value {
        ValueView::TextString(text) => Some((*text).to_owned()),
        _ => None,
    }
}

/// An ordered collection of direct §4 Object Attribute TTLV items.
///
/// Items retain their original tag, typed value, repetitions, and insertion
/// order as required by OASIS KMIP v2.1 §§5.1–5.4, Tables 157–160. Recognized
/// attributes with catalogued encodings are checked against their permitted
/// TTLV item types. Unknown assigned and extension tags remain generic items;
/// this type does not infer attribute names or synthesize tags. A Table 150
/// Vendor Attribute remains a distinct structure and is checked for its
/// required fields and source order.
///
/// For the standard Name attribute, callers are responsible for choosing a
/// Name Value that is unique within the target key management server and for
/// following any server-specific naming rules communicated through a mechanism
/// not specified by KMIP v2.1.
/// `KMIPKit` preserves the supplied Name and cannot detect collisions with
/// existing or concurrent server objects; global uniqueness is not required
/// by KMIP v2.1 §4.32.
///
/// When an attribute permits multiple instances, callers SHOULD supply
/// different values for those instances as recommended by KMIP v2.1 §4.
/// `KMIPKit` preserves the caller's values and order without deduplicating or
/// rewriting them; choosing distinct values remains the caller's
/// responsibility.
pub struct AttributeSet {
    items: Vec<Item>,
}

/// One direct object-attribute Item used to select an existing attribute value.
///
/// OASIS KMIP v2.1 §5.6, Table 162 defines Current Attribute as a Structure
/// containing exactly one direct §4 attribute Item. Its tag identifies the
/// attribute; the Item value is kept as generic TTLV.
pub struct CurrentAttribute {
    item: Item,
}

impl CurrentAttribute {
    /// Wraps one direct generic TTLV Item as a Current Attribute.
    #[must_use]
    pub fn new(item: Item) -> Self {
        Self { item }
    }

    /// Borrows the exact direct attribute Item supplied to [`Self::new`].
    #[must_use]
    pub const fn item(&self) -> &Item {
        &self.item
    }
}

impl Debug for CurrentAttribute {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CurrentAttribute")
            .field("item", &"[REDACTED]")
            .finish()
    }
}

/// One direct object-attribute Item supplied as a new attribute value.
///
/// OASIS KMIP v2.1 §5.7, Table 163 defines New Attribute as a Structure
/// containing exactly one direct §4 attribute Item. Its tag identifies the
/// attribute; the Item value is kept as generic TTLV.
pub struct NewAttribute {
    item: Item,
}

impl NewAttribute {
    /// Wraps one direct generic TTLV Item as a New Attribute.
    #[must_use]
    pub fn new(item: Item) -> Self {
        Self { item }
    }

    /// Borrows the exact direct attribute Item supplied to [`Self::new`].
    #[must_use]
    pub const fn item(&self) -> &Item {
        &self.item
    }
}

impl Debug for NewAttribute {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("NewAttribute")
            .field("item", &"[REDACTED]")
            .finish()
    }
}

/// Copies one generic TTLV Item without changing its tag or value.
///
/// Request payload builders borrow their typed request, so they need an owned
/// copy of the direct Item when constructing the generic TTLV tree.
pub(crate) fn clone_item(item: &Item) -> Result<Item, ProtocolError> {
    let value = item.with_value(clone_value)?;
    Item::new(item.tag(), value).map_err(model_error)
}

fn clone_value(value: ValueView<'_>) -> Result<Value, ProtocolError> {
    match value {
        ValueView::Structure(structure) => {
            let mut children = kmipkit_ttlv::Structure::new();
            for child in structure.children() {
                children.try_push(clone_item(child)?).map_err(model_error)?;
            }
            Ok(Value::structure(children))
        }
        ValueView::Integer(value) => Ok(Value::integer(*value)),
        ValueView::LongInteger(value) => Ok(Value::long_integer(*value)),
        ValueView::BigInteger(value) => Ok(Value::big_integer(value.to_vec())),
        ValueView::Enumeration(value) => Ok(Value::enumeration(*value)),
        ValueView::Boolean(value) => Ok(Value::boolean(*value)),
        ValueView::TextString(value) => Ok(Value::text_string((*value).to_owned())),
        ValueView::ByteString(value) => Ok(Value::byte_string(value.to_vec())),
        ValueView::DateTime(value) => Ok(Value::date_time(*value)),
        ValueView::Interval(value) => Ok(Value::interval(*value)),
        ValueView::DateTimeExtended(value) => Ok(Value::date_time_extended(*value)),
        _ => Err(ProtocolError::categorized(
            ProtocolErrorKind::InvalidValue,
            ProtocolCauseCategory::InvalidValue,
        )),
    }
}

fn model_error(error: ModelError) -> ProtocolError {
    ProtocolError::new(
        ProtocolErrorKind::InvalidValue,
        ProtocolCauseCategory::InvalidValue,
        error,
    )
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
    /// A recognized attribute's TTLV type is outside the catalog encoding.
    AttributeTtlvTypeMismatch,
    /// A direct attribute set contains multiple instances of a source-defined single-instance attribute.
    DuplicateSingleInstanceAttribute,
    /// A conditional source multiplicity rule cannot be decided from the available object context.
    UndeterminedAttributeCardinality,
    /// The required Vendor Attribute members violate §4.60 Table 150 order,
    /// applying the field order rule in §8 and the Structure encoding rule in §10.1.2.
    VendorAttributeFieldOrder,
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
            Self::AttributeTtlvTypeMismatch => {
                "attribute TTLV type does not match its catalog encoding"
            }
            Self::DuplicateSingleInstanceAttribute => {
                "attribute set repeats a source-defined single-instance attribute"
            }
            Self::UndeterminedAttributeCardinality => {
                "conditional attribute multiplicity cannot be decided from available context"
            }
            Self::VendorAttributeFieldOrder => {
                "Vendor Attribute fields do not follow Table 150 order"
            }
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
    /// Repeated tags and values are retained. Recognized attribute types are
    /// checked against catalogued encodings. A Vendor Attribute is validated
    /// against the distinct structure and member order in §4.60, Table 150.
    ///
    /// # Errors
    ///
    /// Returns a payload-free [`AttributeSetError`] when a recognized
    /// attribute has the wrong TTLV type or a Vendor Attribute is malformed,
    /// including required-field or Table 150 member-order violations.
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
    /// Attribute structure, including Table 150 member order. Other generic
    /// attribute values remain unchanged. Recognized standard attributes must
    /// use their catalogued TTLV types.
    pub fn try_push(&mut self, item: Item) -> Result<(), AttributeSetError> {
        validate_catalogued_item_type(&item)?;
        if item.tag().raw() == VENDOR_ATTRIBUTE_TAG {
            item.with_value(|value| match value {
                ValueView::Structure(structure) => validate_vendor_attribute(&structure),
                _ => Err(AttributeSetError::VendorAttributeMustBeStructure),
            })?;
        }
        self.items.push(item);
        Ok(())
    }

    /// Checks source-defined attribute multiplicity when interpreting an
    /// operation's direct object attribute set.
    ///
    /// The generic `AttributeSet` remains lossless. Typed object operations and
    /// Get Attributes responses call this check; Locate predicates and
    /// arbitrary TTLV collections do not. Conditional multiplicity is only
    /// decidable with the relevant object context.
    pub(crate) fn validate_single_instance_cardinality(&self) -> Result<(), AttributeSetError> {
        validate_single_instance_attribute_items(&self.items, None, false)
    }

    pub(crate) fn validate_single_instance_cardinality_for_object_type(
        &self,
        object_type: u32,
    ) -> Result<(), AttributeSetError> {
        validate_single_instance_attribute_items(&self.items, Some(object_type), false)
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

pub(crate) fn validate_single_instance_attribute_items(
    items: &[Item],
    object_type: Option<u32>,
    reject_undetermined: bool,
) -> Result<(), AttributeSetError> {
    let certificate_type = enumeration_value(items, CERTIFICATE_TYPE_TAG);
    let object_type = object_type.or_else(|| enumeration_value(items, OBJECT_TYPE_TAG));
    let mut seen = HashSet::with_capacity(items.len());

    for item in items {
        let raw_tag = item.tag().raw();
        if !seen.insert(raw_tag) {
            let Some(policy) = crate::attribute_policy::ATTRIBUTE_POLICIES
                .iter()
                .find(|policy| policy.tag == raw_tag)
            else {
                continue;
            };

            let is_single_instance = match policy.multiplicity() {
                crate::attribute_policy::AttributeMultiplicity::Single => Some(true),
                crate::attribute_policy::AttributeMultiplicity::Multiple => Some(false),
                crate::attribute_policy::AttributeMultiplicity::PgpKeyOrX509Certificate => {
                    match (object_type, certificate_type) {
                        (Some(OBJECT_TYPE_PGP_KEY), None) => Some(false),
                        (Some(OBJECT_TYPE_CERTIFICATE), Some(CERTIFICATE_TYPE_X509)) => Some(true),
                        _ => None,
                    }
                }
            };
            match is_single_instance {
                Some(true) => return Err(AttributeSetError::DuplicateSingleInstanceAttribute),
                None if reject_undetermined => {
                    return Err(AttributeSetError::UndeterminedAttributeCardinality);
                }
                Some(false) | None => {}
            }
        }
    }
    Ok(())
}

pub(crate) fn validate_key_value_attribute_cardinality(
    object: &Item,
    object_type: Option<u32>,
    reject_undetermined: bool,
) -> Result<(), AttributeSetError> {
    object.with_value(|value| {
        let ValueView::Structure(object_fields) = value else {
            return Ok(());
        };
        let Some(key_block) = unique_structure_field(object_fields.children(), KEY_BLOCK_TAG)?
        else {
            return Ok(());
        };
        key_block.with_value(|value| {
            let ValueView::Structure(key_block_fields) = value else {
                return Err(AttributeSetError::UndeterminedAttributeCardinality);
            };
            let Some(key_value) =
                unique_structure_field(key_block_fields.children(), KEY_VALUE_TAG)?
            else {
                return Ok(());
            };
            key_value.with_value(|value| {
                let ValueView::Structure(key_value_fields) = value else {
                    return Ok(());
                };
                let Some(attributes) =
                    unique_structure_field(key_value_fields.children(), ATTRIBUTES_STRUCTURE_TAG)?
                else {
                    return Ok(());
                };
                attributes.with_value(|value| {
                    let ValueView::Structure(attributes) = value else {
                        return Err(AttributeSetError::UndeterminedAttributeCardinality);
                    };
                    validate_single_instance_attribute_items(
                        attributes.children(),
                        object_type,
                        reject_undetermined,
                    )
                })
            })
        })
    })
}

fn unique_structure_field(
    fields: &[Item],
    raw_tag: u32,
) -> Result<Option<&Item>, AttributeSetError> {
    let mut matches = fields.iter().filter(|field| field.tag().raw() == raw_tag);
    match (matches.next(), matches.next()) {
        (Some(field), None) => Ok(Some(field)),
        (None, None) => Ok(None),
        _ => Err(AttributeSetError::UndeterminedAttributeCardinality),
    }
}

fn enumeration_value(items: &[Item], tag: u32) -> Option<u32> {
    let mut values = items
        .iter()
        .filter(|item| item.tag().raw() == tag)
        .map(|item| {
            item.with_value(|value| match value {
                ValueView::Enumeration(value) => Some(*value),
                _ => None,
            })
        });
    let value = values.next()??;
    values.next().is_none().then_some(value)
}

fn validate_catalogued_item_type(item: &Item) -> Result<(), AttributeSetError> {
    if crate::attribute_types_generated::expected_types(item.tag().raw())
        .is_some_and(|expected| !expected.contains(&item.item_type()))
    {
        Err(AttributeSetError::AttributeTtlvTypeMismatch)
    } else {
        Ok(())
    }
}

impl Default for AttributeSet {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) fn validate_vendor_attribute(
    structure: &StructureView<'_>,
) -> Result<(), AttributeSetError> {
    let mut vendor_identification = false;
    let mut attribute_name = false;
    let mut attribute_value = false;
    let mut last_required_field_rank = 0;

    for field in structure.children() {
        match field.tag().raw() {
            VENDOR_IDENTIFICATION_TAG => {
                if vendor_identification {
                    return Err(AttributeSetError::DuplicateVendorIdentification);
                }
                ensure_required_field_order(&mut last_required_field_rank, 1)?;
                vendor_identification = true;
                validate_vendor_identification(field)?;
            }
            ATTRIBUTE_NAME_TAG => {
                if attribute_name {
                    return Err(AttributeSetError::DuplicateAttributeName);
                }
                ensure_required_field_order(&mut last_required_field_rank, 2)?;
                attribute_name = true;
                validate_attribute_name(field)?;
            }
            ATTRIBUTE_VALUE_TAG => {
                if attribute_value {
                    return Err(AttributeSetError::DuplicateAttributeValue);
                }
                ensure_required_field_order(&mut last_required_field_rank, 3)?;
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

fn ensure_required_field_order(
    last_required_field_rank: &mut u8,
    current_field_rank: u8,
) -> Result<(), AttributeSetError> {
    if current_field_rank < *last_required_field_rank {
        return Err(AttributeSetError::VendorAttributeFieldOrder);
    }
    *last_required_field_rank = current_field_rank;
    Ok(())
}

fn validate_vendor_identification(field: &Item) -> Result<(), AttributeSetError> {
    let valid_identifier =
        field.with_value(|value| match value {
            ValueView::TextString(identifier) => Some(identifier.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '_' | '.')
            })),
            _ => None,
        });
    match valid_identifier {
        Some(true) => Ok(()),
        Some(false) => Err(AttributeSetError::InvalidVendorIdentification),
        None => Err(AttributeSetError::VendorIdentificationMustBeTextString),
    }
}

fn validate_attribute_name(field: &Item) -> Result<(), AttributeSetError> {
    let is_text = field.with_value(|value| matches!(value, ValueView::TextString(_)));
    if is_text {
        Ok(())
    } else {
        Err(AttributeSetError::AttributeNameMustBeTextString)
    }
}
