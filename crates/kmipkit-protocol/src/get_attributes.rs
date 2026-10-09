//! Typed KMIP v2.1 Get Attributes request and response payloads.

use std::collections::HashSet;
use std::error::Error;
use std::fmt;

use kmipkit_ttlv::{Item, ModelError, RawTag, Structure, StructureView, Tag, Value, ValueView};

use crate::{
    AttributeReference, AttributeSet, AttributeSetError, KmipOperationResult,
    ProtocolCauseCategory, ProtocolError, ProtocolErrorKind, ResponseBatchItemView, ResultMessage,
    ResultValidationError, attribute::copy_text_string,
};

const GET_ATTRIBUTES_OPERATION: u32 = 0x0000_000B;
const UNIQUE_IDENTIFIER_TAG: u32 = 0x0042_0094;
const ATTRIBUTES_TAG: u32 = 0x0042_0125;
const SUCCESS: u32 = 0;

/// A client-to-server Get Attributes request from OASIS KMIP v2.1 §6.1.20.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GetAttributesRequest {
    unique_identifier: Option<String>,
    attribute_references: Vec<AttributeReference>,
}

impl GetAttributesRequest {
    /// Creates a request while preserving the optional identifier and reference
    /// order. Table 223 permits no references to request all attributes.
    ///
    /// # Errors
    ///
    /// Returns [`GetAttributesError::DuplicateAttributeReference`] when the
    /// exact same name or tag reference is supplied more than once.
    pub fn try_new(
        unique_identifier: Option<String>,
        attribute_references: impl IntoIterator<Item = AttributeReference>,
    ) -> Result<Self, GetAttributesError> {
        let mut references = Vec::new();
        let mut seen = HashSet::new();

        for reference in attribute_references {
            if !seen.insert(reference.clone()) {
                return Err(GetAttributesError::DuplicateAttributeReference);
            }
            references.push(reference);
        }

        Ok(Self {
            unique_identifier,
            attribute_references: references,
        })
    }

    /// Builds the ordered Request Payload Structure defined in Table 223.
    ///
    /// # Errors
    ///
    /// Returns a payload-free protocol error if a tag-form Attribute Reference
    /// uses a reserved or unallocated tag, a name-form reference has an invalid
    /// Vendor Identification, or a fixed field tag or generic TTLV Structure
    /// cannot be represented.
    pub fn to_ttlv_payload(&self) -> Result<Structure, ProtocolError> {
        let mut payload = Structure::new();
        if let Some(unique_identifier) = &self.unique_identifier {
            payload
                .try_push(item(
                    UNIQUE_IDENTIFIER_TAG,
                    Value::text_string(unique_identifier.clone()),
                )?)
                .map_err(model_error)?;
        }

        for reference in &self.attribute_references {
            payload
                .try_push(reference.to_ttlv_item()?)
                .map_err(model_error)?;
        }

        Ok(payload)
    }
}

/// The typed result of one Get Attributes response batch item.
///
/// Successful responses contain the required Unique Identifier and the ordered
/// direct attribute Items in Table 224's Attributes Structure. Other KMIP
/// statuses retain the common result and have no typed response payload.
pub struct GetAttributesResponse {
    result: KmipOperationResult,
    unique_identifier: Option<String>,
    attributes: Option<AttributeSet>,
}

impl GetAttributesResponse {
    /// Converts one already validated KMIPKIT-0006 response item.
    ///
    /// Successful payload parsing follows OASIS KMIP v2.1 §6.1.20, Table 224,
    /// and preserves direct attribute multiplicity and wire order. The pinned
    /// prose says a no-match response contains only Unique Identifier, while
    /// Table 224 requires Attributes; `KMIPKit` follows the approved table-shaped
    /// contract and accepts its empty Attributes Structure without inventing
    /// Items.
    ///
    /// # Errors
    ///
    /// Returns a payload-free [`GetAttributesError`] for another operation,
    /// invalid result metadata, a missing successful payload, or a payload
    /// that violates Table 224 or the KMIPKIT-0014 `AttributeSet` contract.
    pub fn try_from_response_item(
        item: ResponseBatchItemView<'_>,
    ) -> Result<Self, GetAttributesError> {
        if item.operation() != Some(GET_ATTRIBUTES_OPERATION) {
            return Err(GetAttributesError::UnexpectedOperation);
        }

        let status = item
            .result_status()
            .ok_or(GetAttributesError::MissingResultStatus)?;
        let result_message = item.with_result_message(|text| ResultMessage::new(text.to_owned()));
        let result = KmipOperationResult::new(status, item.result_reason(), result_message)
            .map_err(GetAttributesError::InvalidOperationResult)?;

        if status.raw() != SUCCESS {
            return Ok(Self {
                result,
                unique_identifier: None,
                attributes: None,
            });
        }

        let (unique_identifier, attributes) = item
            .with_response_payload(|payload| parse_success_payload(&payload))
            .ok_or(GetAttributesError::MissingSuccessPayload)??;

        Ok(Self {
            result,
            unique_identifier: Some(unique_identifier),
            attributes: Some(attributes),
        })
    }

    /// Returns the complete KMIP status, reason, and optional Result Message.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }

    /// Returns the required successful-response Unique Identifier, if present.
    #[must_use]
    pub fn unique_identifier(&self) -> Option<&str> {
        self.unique_identifier.as_deref()
    }

    /// Returns the ordered direct attributes for a successful response.
    ///
    /// An empty set is returned when the server represents a no-match response
    /// using the Attributes field required by Table 224. Non-success results
    /// return `None`.
    #[must_use]
    pub const fn attributes(&self) -> Option<&AttributeSet> {
        self.attributes.as_ref()
    }
}

impl fmt::Debug for GetAttributesResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("GetAttributesResponse")
            .field("result", &self.result)
            .field("has_unique_identifier", &self.unique_identifier.is_some())
            .field(
                "attribute_count",
                &self.attributes.as_ref().map(AttributeSet::len),
            )
            .finish()
    }
}

/// A payload-free Get Attributes request or response conversion error.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GetAttributesError {
    /// The same Attribute Reference occurs more than once in the request.
    DuplicateAttributeReference,
    /// The validated response identifies a different operation.
    UnexpectedOperation,
    /// The validated response omitted Result Status.
    MissingResultStatus,
    /// The Result Status/Reason combination violates the common KMIP contract.
    InvalidOperationResult(ResultValidationError),
    /// A successful response omitted its required Response Payload Structure.
    MissingSuccessPayload,
    /// A successful payload violates Table 224's fields, types, or order.
    MalformedSuccessPayload,
    /// A direct attribute value could not be copied into the owned `AttributeSet`.
    TtlvModel(ModelError),
    /// A returned Vendor Attribute does not satisfy KMIPKIT-0014 validation.
    InvalidAttributeSet(AttributeSetError),
}

impl fmt::Display for GetAttributesError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateAttributeReference => {
                formatter.write_str("Get Attributes request repeats an Attribute Reference")
            }
            Self::UnexpectedOperation => formatter.write_str("response item is not Get Attributes"),
            Self::MissingResultStatus => {
                formatter.write_str("Get Attributes result status is missing")
            }
            Self::InvalidOperationResult(cause) => {
                write!(
                    formatter,
                    "Get Attributes operation result is invalid: {cause}"
                )
            }
            Self::MissingSuccessPayload => {
                formatter.write_str("successful Get Attributes response has no payload")
            }
            Self::MalformedSuccessPayload => {
                formatter.write_str("Get Attributes response payload is malformed")
            }
            Self::TtlvModel(cause) => write!(formatter, "Get Attributes item is invalid: {cause}"),
            Self::InvalidAttributeSet(cause) => {
                write!(
                    formatter,
                    "Get Attributes contains an invalid attribute: {cause}"
                )
            }
        }
    }
}

impl Error for GetAttributesError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidOperationResult(cause) => Some(cause),
            Self::TtlvModel(cause) => Some(cause),
            Self::InvalidAttributeSet(cause) => Some(cause),
            _ => None,
        }
    }
}

fn parse_success_payload(
    payload: &StructureView<'_>,
) -> Result<(String, AttributeSet), GetAttributesError> {
    let fields = payload.children();
    if fields.len() != 2
        || fields[0].tag().raw() != UNIQUE_IDENTIFIER_TAG
        || fields[1].tag().raw() != ATTRIBUTES_TAG
    {
        return Err(GetAttributesError::MalformedSuccessPayload);
    }

    let unique_identifier = fields[0]
        .with_value(|value| copy_text_string(&value))
        .ok_or(GetAttributesError::MalformedSuccessPayload)?;

    let attributes = fields[1]
        .with_value(|value| match value {
            ValueView::Structure(structure) => Some(parse_attributes(&structure)),
            _ => None,
        })
        .ok_or(GetAttributesError::MalformedSuccessPayload)??;

    Ok((unique_identifier, attributes))
}

fn parse_attributes(structure: &StructureView<'_>) -> Result<AttributeSet, GetAttributesError> {
    let mut items = Vec::with_capacity(structure.children().len());
    for field in structure.children() {
        let value = field.with_value(clone_value)?;
        let item = Item::new(field.tag(), value).map_err(GetAttributesError::TtlvModel)?;
        items.push(item);
    }

    AttributeSet::try_new(items).map_err(GetAttributesError::InvalidAttributeSet)
}

fn clone_value(value: ValueView<'_>) -> Result<Value, GetAttributesError> {
    match value {
        ValueView::Structure(structure) => {
            let mut children = Structure::new();
            for child in structure.children() {
                let value = child.with_value(clone_value)?;
                let item = Item::new(child.tag(), value).map_err(GetAttributesError::TtlvModel)?;
                children
                    .try_push(item)
                    .map_err(GetAttributesError::TtlvModel)?;
            }
            Ok(Value::structure(children))
        }
        ValueView::Integer(value) => Ok(Value::integer(*value)),
        ValueView::LongInteger(value) => Ok(Value::long_integer(*value)),
        ValueView::BigInteger(value) => Ok(Value::big_integer(value.to_vec())),
        ValueView::Enumeration(value) => Ok(Value::enumeration(*value)),
        ValueView::Boolean(value) => Ok(Value::boolean(*value)),
        ValueView::TextString(value) => Ok(Value::text_string(value.to_owned())),
        ValueView::ByteString(value) => Ok(Value::byte_string(value.to_vec())),
        ValueView::DateTime(value) => Ok(Value::date_time(*value)),
        ValueView::Interval(value) => Ok(Value::interval(*value)),
        ValueView::DateTimeExtended(value) => Ok(Value::date_time_extended(*value)),
        _ => Err(GetAttributesError::MalformedSuccessPayload),
    }
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
