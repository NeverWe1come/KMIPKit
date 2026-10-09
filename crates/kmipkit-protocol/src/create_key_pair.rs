//! Typed KMIP 2.1 Create Key Pair request and response payloads.

use std::error::Error;
use std::fmt;

use kmipkit_ttlv::{Item, ModelError, RawTag, Structure, StructureView, Tag, Value, ValueView};

use crate::{
    AttributeSet, KmipOperationResult, ProtocolCauseCategory, ProtocolError, ProtocolErrorKind,
    ResponseBatchItemView, ResultMessage, ResultValidationError, UniqueIdentifier,
};

const CREATE_KEY_PAIR_OPERATION: u32 = 0x0000_0002;
const SUCCESS: u32 = 0;

const COMMON_ATTRIBUTES: u32 = 0x0042_0126;
const PRIVATE_KEY_ATTRIBUTES: u32 = 0x0042_0127;
const PUBLIC_KEY_ATTRIBUTES: u32 = 0x0042_0128;
const COMMON_PROTECTION_STORAGE_MASKS: u32 = 0x0042_0163;
const PRIVATE_PROTECTION_STORAGE_MASKS: u32 = 0x0042_0164;
const PUBLIC_PROTECTION_STORAGE_MASKS: u32 = 0x0042_0165;

const CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;
const CRYPTOGRAPHIC_DOMAIN_PARAMETERS: u32 = 0x0042_0029;
const CRYPTOGRAPHIC_LENGTH: u32 = 0x0042_002a;
const CRYPTOGRAPHIC_PARAMETERS: u32 = 0x0042_002b;
const TABLE_191_ATTRIBUTES: [u32; 4] = [
    CRYPTOGRAPHIC_ALGORITHM,
    CRYPTOGRAPHIC_LENGTH,
    CRYPTOGRAPHIC_DOMAIN_PARAMETERS,
    CRYPTOGRAPHIC_PARAMETERS,
];

const PRIVATE_KEY_UNIQUE_IDENTIFIER: u32 = 0x0042_0066;
const PUBLIC_KEY_UNIQUE_IDENTIFIER: u32 = 0x0042_006f;

/// A typed KMIP 2.1 Create Key Pair request payload.
///
/// The six optional Table 189 groups remain independent. Attribute items keep
/// their tags, values, repetitions, and group-local order; the server applies
/// the Table 189 multi-instance union semantics.
pub struct CreateKeyPairRequest {
    common_attributes: Option<AttributeSet>,
    private_key_attributes: Option<AttributeSet>,
    public_key_attributes: Option<AttributeSet>,
    common_protection_storage_masks: Option<Structure>,
    private_protection_storage_masks: Option<Structure>,
    public_protection_storage_masks: Option<Structure>,
}

impl CreateKeyPairRequest {
    /// Creates a request with all optional Table 189 groups absent.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            common_attributes: None,
            private_key_attributes: None,
            public_key_attributes: None,
            common_protection_storage_masks: None,
            private_protection_storage_masks: None,
            public_protection_storage_masks: None,
        }
    }

    /// Sets the optional Common Attributes Structure.
    #[must_use]
    pub fn with_common_attributes(mut self, attributes: AttributeSet) -> Self {
        self.common_attributes = Some(attributes);
        self
    }

    /// Sets the optional Private Key Attributes Structure.
    #[must_use]
    pub fn with_private_key_attributes(mut self, attributes: AttributeSet) -> Self {
        self.private_key_attributes = Some(attributes);
        self
    }

    /// Sets the optional Public Key Attributes Structure.
    #[must_use]
    pub fn with_public_key_attributes(mut self, attributes: AttributeSet) -> Self {
        self.public_key_attributes = Some(attributes);
        self
    }

    /// Sets the optional Common Protection Storage Masks Structure.
    #[must_use]
    pub fn with_common_protection_storage_masks(mut self, masks: Structure) -> Self {
        self.common_protection_storage_masks = Some(masks);
        self
    }

    /// Sets the optional Private Protection Storage Masks Structure.
    #[must_use]
    pub fn with_private_protection_storage_masks(mut self, masks: Structure) -> Self {
        self.private_protection_storage_masks = Some(masks);
        self
    }

    /// Sets the optional Public Protection Storage Masks Structure.
    #[must_use]
    pub fn with_public_protection_storage_masks(mut self, masks: Structure) -> Self {
        self.public_protection_storage_masks = Some(masks);
        self
    }

    /// Returns whether Common Attributes is present.
    #[must_use]
    pub const fn has_common_attributes(&self) -> bool {
        self.common_attributes.is_some()
    }

    /// Returns whether Private Key Attributes is present.
    #[must_use]
    pub const fn has_private_key_attributes(&self) -> bool {
        self.private_key_attributes.is_some()
    }

    /// Returns whether Public Key Attributes is present.
    #[must_use]
    pub const fn has_public_key_attributes(&self) -> bool {
        self.public_key_attributes.is_some()
    }

    /// Encodes the ordered Create Key Pair payload.
    ///
    /// For each Table 191 attribute, the effective value for each key is its
    /// key-specific item when supplied, otherwise the Common Attributes item.
    /// The resulting values must either both be absent or both be present and
    /// equal. Values are compared by exact generic TTLV type, content, and
    /// Structure child order; no value is copied between groups.
    ///
    /// # Errors
    ///
    /// Returns a sanitized protocol error for an inconsistent Table 191 value
    /// or a generic TTLV model constraint failure.
    pub fn into_ttlv_payload(self) -> Result<Structure, ProtocolError> {
        validate_table_191(
            self.common_attributes.as_ref(),
            self.private_key_attributes.as_ref(),
            self.public_key_attributes.as_ref(),
        )
        .map_err(|error| {
            ProtocolError::new(
                ProtocolErrorKind::InvalidValue,
                ProtocolCauseCategory::InvalidValue,
                error,
            )
        })?;

        let mut payload = Structure::new();
        push_attribute_group(&mut payload, COMMON_ATTRIBUTES, self.common_attributes)?;
        push_attribute_group(
            &mut payload,
            PRIVATE_KEY_ATTRIBUTES,
            self.private_key_attributes,
        )?;
        push_attribute_group(
            &mut payload,
            PUBLIC_KEY_ATTRIBUTES,
            self.public_key_attributes,
        )?;
        push_structure_group(
            &mut payload,
            COMMON_PROTECTION_STORAGE_MASKS,
            self.common_protection_storage_masks,
        )?;
        push_structure_group(
            &mut payload,
            PRIVATE_PROTECTION_STORAGE_MASKS,
            self.private_protection_storage_masks,
        )?;
        push_structure_group(
            &mut payload,
            PUBLIC_PROTECTION_STORAGE_MASKS,
            self.public_protection_storage_masks,
        )?;
        Ok(payload)
    }
}

impl Default for CreateKeyPairRequest {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for CreateKeyPairRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CreateKeyPairRequest")
            .field("has_common_attributes", &self.common_attributes.is_some())
            .field(
                "has_private_key_attributes",
                &self.private_key_attributes.is_some(),
            )
            .field(
                "has_public_key_attributes",
                &self.public_key_attributes.is_some(),
            )
            .field(
                "has_common_protection_storage_masks",
                &self.common_protection_storage_masks.is_some(),
            )
            .field(
                "has_private_protection_storage_masks",
                &self.private_protection_storage_masks.is_some(),
            )
            .field(
                "has_public_protection_storage_masks",
                &self.public_protection_storage_masks.is_some(),
            )
            .finish()
    }
}

/// A typed Create Key Pair response payload and operation result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateKeyPairResponse {
    result: KmipOperationResult,
    private_key_unique_identifier: Option<UniqueIdentifier>,
    public_key_unique_identifier: Option<UniqueIdentifier>,
}

impl CreateKeyPairResponse {
    /// Converts one validated Create Key Pair response batch item.
    ///
    /// A successful result requires exactly one private-key and one public-key
    /// Unique Identifier. Failure results retain the shared KMIP result and do
    /// not expose success payload fields.
    ///
    /// # Errors
    ///
    /// Returns [`CreateKeyPairError`] for a different operation, malformed
    /// result, or missing, duplicated, or incorrectly typed success fields.
    pub fn try_from_response_item(
        item: ResponseBatchItemView<'_>,
    ) -> Result<Self, CreateKeyPairError> {
        if item.operation() != Some(CREATE_KEY_PAIR_OPERATION) {
            return Err(CreateKeyPairError::UnexpectedOperation);
        }
        let status = item
            .result_status()
            .ok_or(CreateKeyPairError::MissingResultStatus)?;
        let result_message = item.with_result_message(|text| ResultMessage::new(text.to_owned()));
        let result = KmipOperationResult::new(status, item.result_reason(), result_message)
            .map_err(CreateKeyPairError::InvalidOperationResult)?;

        if status.raw() != SUCCESS {
            return Ok(Self {
                result,
                private_key_unique_identifier: None,
                public_key_unique_identifier: None,
            });
        }

        let (private_key_unique_identifier, public_key_unique_identifier) = item
            .with_response_payload(|payload| parse_success_payload(&payload))
            .ok_or(CreateKeyPairError::MissingSuccessPayload)??;
        Ok(Self {
            result,
            private_key_unique_identifier: Some(private_key_unique_identifier),
            public_key_unique_identifier: Some(public_key_unique_identifier),
        })
    }

    /// Returns the exact KMIP operation result.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }

    /// Returns the private-key Unique Identifier on success.
    #[must_use]
    pub const fn private_key_unique_identifier(&self) -> Option<&UniqueIdentifier> {
        self.private_key_unique_identifier.as_ref()
    }

    /// Returns the public-key Unique Identifier on success.
    #[must_use]
    pub const fn public_key_unique_identifier(&self) -> Option<&UniqueIdentifier> {
        self.public_key_unique_identifier.as_ref()
    }
}

/// A payload-free error converting a validated Create Key Pair response item.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CreateKeyPairError {
    /// The response item is not the client-to-server Create Key Pair operation.
    UnexpectedOperation,
    /// The validated response item did not expose a Result Status.
    MissingResultStatus,
    /// The represented KMIP result violates the shared status/reason contract.
    InvalidOperationResult(ResultValidationError),
    /// A successful result has no Response Payload Structure.
    MissingSuccessPayload,
    /// A successful payload has missing, duplicate, or mistyped identifiers.
    MalformedSuccessPayload,
}

impl fmt::Display for CreateKeyPairError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::UnexpectedOperation => "response item is not Create Key Pair",
            Self::MissingResultStatus => "Create Key Pair result status is missing",
            Self::InvalidOperationResult(cause) => {
                return write!(
                    formatter,
                    "Create Key Pair operation result is invalid: {cause}"
                );
            }
            Self::MissingSuccessPayload => "successful Create Key Pair result has no payload",
            Self::MalformedSuccessPayload => {
                "successful Create Key Pair response payload is malformed"
            }
        };
        formatter.write_str(message)
    }
}

impl Error for CreateKeyPairError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidOperationResult(cause) => Some(cause),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Table191Mismatch {
    DuplicateOrDifferentEffectiveValue,
}

impl fmt::Display for Table191Mismatch {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateOrDifferentEffectiveValue => {
                formatter.write_str("Create Key Pair Table 191 values are inconsistent")
            }
        }
    }
}

impl Error for Table191Mismatch {}

fn validate_table_191(
    common: Option<&AttributeSet>,
    private: Option<&AttributeSet>,
    public: Option<&AttributeSet>,
) -> Result<(), Table191Mismatch> {
    for raw_tag in TABLE_191_ATTRIBUTES {
        let common_value = unique_attribute(common, raw_tag)?;
        let private_value = unique_attribute(private, raw_tag)?.or(common_value);
        let public_value = unique_attribute(public, raw_tag)?.or(common_value);
        match (private_value, public_value) {
            (None, None) => {}
            (Some(private), Some(public)) if values_equal(private, public) => {}
            _ => return Err(Table191Mismatch::DuplicateOrDifferentEffectiveValue),
        }
    }
    Ok(())
}

fn unique_attribute(
    attributes: Option<&AttributeSet>,
    raw_tag: u32,
) -> Result<Option<&Item>, Table191Mismatch> {
    let mut matching = attributes
        .into_iter()
        .flat_map(AttributeSet::as_items)
        .filter(|item| item.tag().raw() == raw_tag);
    let value = matching.next();
    if matching.next().is_some() {
        return Err(Table191Mismatch::DuplicateOrDifferentEffectiveValue);
    }
    Ok(value)
}

fn values_equal(left: &Item, right: &Item) -> bool {
    left.with_value(|left_value| {
        right.with_value(|right_value| value_views_equal(left_value, right_value))
    })
}

fn items_equal(left: &Item, right: &Item) -> bool {
    left.tag().raw() == right.tag().raw() && values_equal(left, right)
}

fn value_views_equal(left: ValueView<'_>, right: ValueView<'_>) -> bool {
    match (left, right) {
        (ValueView::Structure(left), ValueView::Structure(right)) => {
            structures_equal(&left, &right)
        }
        (ValueView::Integer(left), ValueView::Integer(right)) => left == right,
        (ValueView::LongInteger(left), ValueView::LongInteger(right))
        | (ValueView::DateTime(left), ValueView::DateTime(right))
        | (ValueView::DateTimeExtended(left), ValueView::DateTimeExtended(right)) => left == right,
        (ValueView::BigInteger(left), ValueView::BigInteger(right))
        | (ValueView::ByteString(left), ValueView::ByteString(right)) => left == right,
        (ValueView::Enumeration(left), ValueView::Enumeration(right))
        | (ValueView::Interval(left), ValueView::Interval(right)) => left == right,
        (ValueView::Boolean(left), ValueView::Boolean(right)) => left == right,
        (ValueView::TextString(left), ValueView::TextString(right)) => left == right,
        _ => false,
    }
}

fn structures_equal(left: &StructureView<'_>, right: &StructureView<'_>) -> bool {
    let left_items = left.children();
    let right_items = right.children();
    left_items.len() == right_items.len()
        && left_items
            .iter()
            .zip(right_items)
            .all(|(left, right)| items_equal(left, right))
}

fn parse_success_payload(
    payload: &StructureView<'_>,
) -> Result<(UniqueIdentifier, UniqueIdentifier), CreateKeyPairError> {
    let mut private_key_unique_identifier = None;
    let mut public_key_unique_identifier = None;
    for field in payload.children() {
        match field.tag().raw() {
            PRIVATE_KEY_UNIQUE_IDENTIFIER => {
                if private_key_unique_identifier.is_some() {
                    return Err(CreateKeyPairError::MalformedSuccessPayload);
                }
                private_key_unique_identifier = parse_unique_identifier(field);
                if private_key_unique_identifier.is_none() {
                    return Err(CreateKeyPairError::MalformedSuccessPayload);
                }
            }
            PUBLIC_KEY_UNIQUE_IDENTIFIER => {
                if public_key_unique_identifier.is_some() {
                    return Err(CreateKeyPairError::MalformedSuccessPayload);
                }
                public_key_unique_identifier = parse_unique_identifier(field);
                if public_key_unique_identifier.is_none() {
                    return Err(CreateKeyPairError::MalformedSuccessPayload);
                }
            }
            _ => {}
        }
    }
    match (private_key_unique_identifier, public_key_unique_identifier) {
        (Some(private), Some(public)) => Ok((private, public)),
        _ => Err(CreateKeyPairError::MalformedSuccessPayload),
    }
}

fn parse_unique_identifier(field: &Item) -> Option<UniqueIdentifier> {
    field.with_value(|value| match value {
        ValueView::TextString(value) => Some(UniqueIdentifier::TextString(value.to_owned())),
        ValueView::Enumeration(value) => Some(UniqueIdentifier::Enumeration(*value)),
        ValueView::Integer(value) => Some(UniqueIdentifier::Integer(*value)),
        _ => None,
    })
}

fn push_attribute_group(
    payload: &mut Structure,
    raw_tag: u32,
    attributes: Option<AttributeSet>,
) -> Result<(), ProtocolError> {
    let Some(attributes) = attributes else {
        return Ok(());
    };
    let mut group = Structure::new();
    for attribute in attributes.into_items() {
        group.try_push(attribute).map_err(model_error)?;
    }
    push(payload, raw_tag, Value::structure(group))
}

fn push_structure_group(
    payload: &mut Structure,
    raw_tag: u32,
    group: Option<Structure>,
) -> Result<(), ProtocolError> {
    if let Some(group) = group {
        push(payload, raw_tag, Value::structure(group))?;
    }
    Ok(())
}

fn push(structure: &mut Structure, raw_tag: u32, value: Value) -> Result<(), ProtocolError> {
    structure
        .try_push(item(raw_tag, value)?)
        .map_err(model_error)
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
