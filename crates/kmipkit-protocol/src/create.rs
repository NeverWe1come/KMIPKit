//! Typed KMIP 2.1 Create request and response payloads.

use std::error::Error;
use std::fmt;

use kmipkit_ttlv::{Item, ModelError, RawTag, Structure, StructureView, Tag, Value, ValueView};

use crate::{
    AttributeSet, KmipOperationResult, ProtocolCauseCategory, ProtocolError, ProtocolErrorKind,
    ResponseBatchItemView, ResultMessage, ResultValidationError,
};

const CREATE_OPERATION: u32 = 0x0000_0001;
const OBJECT_TYPE: u32 = 0x0042_0057;
const ATTRIBUTES: u32 = 0x0042_0125; // §5.1 Attributes Structure, not the Attribute tag.
const PROTECTION_STORAGE_MASKS: u32 = 0x0042_015f;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const SUCCESS: u32 = 0;

/// A KMIP Object Type Enumeration, retaining values unknown to this release.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ObjectType(u32);

impl ObjectType {
    /// Creates an Object Type from its raw unsigned Enumeration value.
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Returns the exact raw unsigned Enumeration value.
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// A Table 187 Unique Identifier value in its original TTLV wire form.
///
/// Its [`Debug`] representation redacts the wire value because identifiers can
/// appear in lifecycle operation requests and responses.
#[non_exhaustive]
#[derive(Clone, Eq, PartialEq)]
pub enum UniqueIdentifier {
    /// A KMIP Text String identifier.
    TextString(String),
    /// A raw unsigned KMIP Enumeration identifier, including unknown values.
    Enumeration(u32),
    /// A signed KMIP Integer identifier.
    Integer(i32),
}

impl fmt::Debug for UniqueIdentifier {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("UniqueIdentifier([REDACTED])")
    }
}

/// A typed KMIP 2.1 Create request payload.
///
/// Object Type remains a raw-value wrapper so future Object Type values are
/// preserved. Attributes are always represented by the required outer
/// Structure, including when the direct attribute set is empty. Optional
/// Protection Storage Masks are retained as a generic ordered Structure.
pub struct CreateRequest {
    object_type: ObjectType,
    attributes: AttributeSet,
    protection_storage_masks: Option<Structure>,
}

impl CreateRequest {
    /// Creates a request with explicit Object Type and Attributes.
    #[must_use]
    pub const fn new(object_type: ObjectType, attributes: AttributeSet) -> Self {
        Self {
            object_type,
            attributes,
            protection_storage_masks: None,
        }
    }

    /// Sets the optional generic Protection Storage Masks Structure.
    #[must_use]
    pub fn with_protection_storage_masks(mut self, masks: Structure) -> Self {
        self.protection_storage_masks = Some(masks);
        self
    }

    /// Returns the selected Object Type, including its raw Enumeration value.
    #[must_use]
    pub const fn object_type(&self) -> ObjectType {
        self.object_type
    }

    /// Returns the direct ordered attributes supplied by the caller.
    #[must_use]
    pub const fn attributes(&self) -> &AttributeSet {
        &self.attributes
    }

    /// Returns the optional generic Protection Storage Masks Structure.
    #[must_use]
    pub const fn protection_storage_masks(&self) -> Option<&Structure> {
        self.protection_storage_masks.as_ref()
    }

    /// Builds the ordered generic Structure for the Create request payload.
    ///
    /// The common message model owns the enclosing Request Payload and Batch
    /// Item. No cryptographic attribute, parameter, or storage mask is
    /// synthesized.
    ///
    /// # Errors
    ///
    /// Returns a sanitized protocol error if an internally defined tag or
    /// nested generic Structure violates the TTLV model constraints.
    pub fn into_ttlv_payload(self) -> Result<Structure, ProtocolError> {
        let mut attributes = Structure::new();
        for attribute in self.attributes.into_items() {
            attributes.try_push(attribute).map_err(model_error)?;
        }

        let mut payload = Structure::new();
        payload
            .try_push(item(
                OBJECT_TYPE,
                Value::enumeration(self.object_type.raw()),
            )?)
            .map_err(model_error)?;
        payload
            .try_push(item(ATTRIBUTES, Value::structure(attributes))?)
            .map_err(model_error)?;
        if let Some(masks) = self.protection_storage_masks {
            payload
                .try_push(item(PROTECTION_STORAGE_MASKS, Value::structure(masks))?)
                .map_err(model_error)?;
        }
        Ok(payload)
    }
}

/// A typed Create result for one validated KMIP response batch item.
///
/// Successful responses expose the exact Object Type and Unique Identifier
/// representation from Table 187. Non-success results retain the shared
/// [`KmipOperationResult`] and have no successful payload fields. The source
/// [`crate::ResponseMessage`] remains the owner of the complete generic tree.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateResponse {
    result: KmipOperationResult,
    object_type: Option<ObjectType>,
    unique_identifier: Option<UniqueIdentifier>,
}

impl CreateResponse {
    /// Converts one already validated KMIP response batch item.
    ///
    /// A successful Create result requires exactly one Object Type Enumeration
    /// and one Unique Identifier in a permitted Table 187 wire form. The
    /// borrowed source tree is not consumed or normalized.
    ///
    /// # Errors
    ///
    /// Returns [`CreateError`] for a different operation, invalid result,
    /// absent success payload, or malformed required payload field. Errors do
    /// not include server-provided field values.
    pub fn try_from_response_item(item: ResponseBatchItemView<'_>) -> Result<Self, CreateError> {
        if item.operation() != Some(CREATE_OPERATION) {
            return Err(CreateError::UnexpectedOperation);
        }
        let status = item
            .result_status()
            .ok_or(CreateError::MissingResultStatus)?;
        let result_message = item.with_result_message(|text| ResultMessage::new(text.to_owned()));
        let result = KmipOperationResult::new(status, item.result_reason(), result_message)
            .map_err(CreateError::InvalidOperationResult)?;

        if status.raw() != SUCCESS {
            return Ok(Self {
                result,
                object_type: None,
                unique_identifier: None,
            });
        }

        let (object_type, unique_identifier) = item
            .with_response_payload(|payload| parse_success_payload(&payload))
            .ok_or(CreateError::MissingSuccessPayload)??;
        Ok(Self {
            result,
            object_type: Some(object_type),
            unique_identifier: Some(unique_identifier),
        })
    }

    /// Returns the exact KMIP operation result.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }

    /// Returns the Object Type for a successful result.
    #[must_use]
    pub const fn object_type(&self) -> Option<ObjectType> {
        self.object_type
    }

    /// Returns the Unique Identifier in the exact Table 187 wire form.
    #[must_use]
    pub const fn unique_identifier(&self) -> Option<&UniqueIdentifier> {
        self.unique_identifier.as_ref()
    }
}

/// A payload-free error converting a validated Create response item.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CreateError {
    /// The response item is not the client-to-server Create operation.
    UnexpectedOperation,
    /// The validated response item did not expose a Result Status.
    MissingResultStatus,
    /// The represented KMIP result violates the shared status/reason contract.
    InvalidOperationResult(ResultValidationError),
    /// A successful Create response omitted its Response Payload Structure.
    MissingSuccessPayload,
    /// A success payload omitted, duplicated, or mistyped a required field.
    MalformedSuccessPayload,
}

impl fmt::Display for CreateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::UnexpectedOperation => "response item is not Create",
            Self::MissingResultStatus => "Create result status is missing",
            Self::InvalidOperationResult(cause) => {
                return write!(formatter, "Create operation result is invalid: {cause}");
            }
            Self::MissingSuccessPayload => "successful Create response has no payload",
            Self::MalformedSuccessPayload => "successful Create response payload is malformed",
        };
        formatter.write_str(message)
    }
}

impl Error for CreateError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidOperationResult(cause) => Some(cause),
            _ => None,
        }
    }
}

fn parse_success_payload(
    payload: &StructureView<'_>,
) -> Result<(ObjectType, UniqueIdentifier), CreateError> {
    let mut object_type = None;
    let mut unique_identifier = None;

    for field in payload.children() {
        match field.tag().raw() {
            OBJECT_TYPE => {
                if object_type.is_some() {
                    return Err(CreateError::MalformedSuccessPayload);
                }
                object_type = parse_object_type(field);
                if object_type.is_none() {
                    return Err(CreateError::MalformedSuccessPayload);
                }
            }
            UNIQUE_IDENTIFIER => {
                if unique_identifier.is_some() {
                    return Err(CreateError::MalformedSuccessPayload);
                }
                unique_identifier = parse_unique_identifier(field);
                if unique_identifier.is_none() {
                    return Err(CreateError::MalformedSuccessPayload);
                }
            }
            // The validated source ResponseMessage remains available to callers
            // for exact generic inspection of extension or future fields.
            _ => {}
        }
    }

    match (object_type, unique_identifier) {
        (Some(object_type), Some(unique_identifier)) => Ok((object_type, unique_identifier)),
        _ => Err(CreateError::MalformedSuccessPayload),
    }
}

fn parse_object_type(field: &Item) -> Option<ObjectType> {
    field.with_value(|value| match value {
        ValueView::Enumeration(raw) => Some(ObjectType::from_raw(*raw)),
        _ => None,
    })
}

fn parse_unique_identifier(field: &Item) -> Option<UniqueIdentifier> {
    field.with_value(|value| match value {
        ValueView::TextString(value) => Some(UniqueIdentifier::TextString(value.to_owned())),
        ValueView::Enumeration(value) => Some(UniqueIdentifier::Enumeration(*value)),
        ValueView::Integer(value) => Some(UniqueIdentifier::Integer(*value)),
        _ => None,
    })
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
