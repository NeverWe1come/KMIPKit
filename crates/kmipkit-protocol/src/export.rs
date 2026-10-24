//! Typed KMIP 2.1 Export request and response payloads.

use std::error::Error;
use std::fmt;

use kmipkit_ttlv::{Item, ItemType, Structure, StructureView, ValueView};

use crate::cryptographic_operation::{self as common, OwnedResponseTtlv};
use crate::{
    GetRequest, KeyCompressionType, KeyFormatType, KeyWrapType, KmipOperationResult, ObjectType,
    ProtocolError, ResponseBatchItemView, ResultValidationError, UniqueIdentifier,
};

const OPERATION: u32 = 0x0000_002B;
const SUCCESS: u32 = 0;
const OBJECT_TYPE: u32 = 0x0042_0057;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const ATTRIBUTES: u32 = 0x0042_0125;
const SYMMETRIC_KEY: u32 = 0x0042_008F;
const PUBLIC_KEY: u32 = 0x0042_006D;
const PRIVATE_KEY: u32 = 0x0042_0064;
const SPLIT_KEY: u32 = 0x0042_0089;
const SECRET_DATA: u32 = 0x0042_0085;
const CERTIFICATE: u32 = 0x0042_0013;
const CERTIFICATE_REQUEST: u32 = 0x0042_0018;
const OPAQUE_OBJECT: u32 = 0x0042_005B;
const PGP_KEY: u32 = 0x0042_00BD;

/// A typed KMIP 2.1 Export request from §6.1.18 Table 217.
///
/// Its optional selectors match the Get request. The request preserves omitted
/// fields so server-side ID Placeholder behavior remains available.
pub struct ExportRequest {
    selectors: GetRequest,
}

impl ExportRequest {
    /// Creates an Export request with all optional fields omitted.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            selectors: GetRequest::new(),
        }
    }

    /// Sets the optional Unique Identifier in its original KMIP wire form.
    #[must_use]
    pub fn with_unique_identifier(mut self, identifier: UniqueIdentifier) -> Self {
        self.selectors = self.selectors.with_unique_identifier(identifier);
        self
    }

    /// Returns the optional Unique Identifier in its original KMIP wire form.
    #[must_use]
    pub const fn unique_identifier(&self) -> Option<&UniqueIdentifier> {
        self.selectors.unique_identifier()
    }

    /// Sets the optional Key Format Type, including unknown future values.
    #[must_use]
    pub fn with_key_format_type(mut self, value: KeyFormatType) -> Self {
        self.selectors = self.selectors.with_key_format_type(value);
        self
    }

    /// Sets the optional Key Wrap Type, including unknown future values.
    #[must_use]
    pub fn with_key_wrap_type(mut self, value: KeyWrapType) -> Self {
        self.selectors = self.selectors.with_key_wrap_type(value);
        self
    }

    /// Sets the optional Key Compression Type, including unknown future values.
    #[must_use]
    pub fn with_key_compression_type(mut self, value: KeyCompressionType) -> Self {
        self.selectors = self.selectors.with_key_compression_type(value);
        self
    }

    /// Sets the optional caller-supplied Key Wrapping Specification.
    #[must_use]
    pub fn with_key_wrapping_specification(mut self, value: Structure) -> Self {
        self.selectors = self.selectors.with_key_wrapping_specification(value);
        self
    }

    /// Encodes the caller-selected Table 217 request fields in wire order.
    ///
    /// `KMIPKit` does not perform wrapping, compression, or format conversion.
    ///
    /// # Errors
    /// Returns a payload-free protocol error if the selected fields violate
    /// TTLV model constraints.
    pub fn into_ttlv_payload(self) -> Result<Structure, ProtocolError> {
        self.selectors.to_ttlv_payload()
    }
}

impl Default for ExportRequest {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for ExportRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ExportRequest([REDACTED])")
    }
}

/// A typed Export result from §6.1.18 Tables 218–219.
///
/// Successful object and attribute values remain opaque TTLV. KMIPKit-owned
/// copies are zeroized when dropped; caller-created copies are outside that
/// guarantee.
pub struct ExportResponse {
    result: KmipOperationResult,
    object_type: Option<ObjectType>,
    unique_identifier: Option<UniqueIdentifier>,
    attributes: Option<Item>,
    object: Option<Item>,
    response_ttlv: OwnedResponseTtlv,
}

impl ExportResponse {
    /// Converts one validated Export result and preserves the complete response item.
    ///
    /// # Errors
    /// Returns a payload-free [`ExportError`] for an unexpected operation,
    /// invalid common result fields, or malformed successful payload.
    pub fn try_from_response_item(item: ResponseBatchItemView<'_>) -> Result<Self, ExportError> {
        if item.operation() != Some(OPERATION) {
            return Err(ExportError::UnexpectedOperation);
        }
        let status = item
            .result_status()
            .ok_or(ExportError::MissingResultStatus)?;
        let result = KmipOperationResult::new(
            status,
            item.result_reason(),
            item.with_result_message(|message| crate::ResultMessage::new(message.to_owned())),
        )
        .map_err(ExportError::InvalidOperationResult)?;
        let response_ttlv = common::clone_response_item(item, "Export")
            .map_err(|_| ExportError::MalformedResponseItem)?;
        if status.raw() != SUCCESS {
            return Ok(Self {
                result,
                object_type: None,
                unique_identifier: None,
                attributes: None,
                object: None,
                response_ttlv,
            });
        }

        let (object_type, unique_identifier, attributes, object) = item
            .with_response_payload(|payload| parse_success_payload(&payload))
            .ok_or(ExportError::MissingSuccessPayload)??;
        Ok(Self {
            result,
            object_type: Some(object_type),
            unique_identifier: Some(unique_identifier),
            attributes: Some(attributes),
            object: Some(object),
            response_ttlv,
        })
    }

    /// Returns the exact KMIP status, reason, and optional Result Message.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }

    /// Returns the Object Type on success, including unrecognized values.
    #[must_use]
    pub const fn object_type(&self) -> Option<ObjectType> {
        self.object_type
    }

    /// Returns the object's Unique Identifier in its original wire form.
    #[must_use]
    pub const fn unique_identifier(&self) -> Option<&UniqueIdentifier> {
        self.unique_identifier.as_ref()
    }

    /// Returns the opaque Attributes item on success.
    #[must_use]
    pub const fn attributes(&self) -> Option<&Item> {
        self.attributes.as_ref()
    }

    /// Returns the opaque Any Object item on success.
    #[must_use]
    pub const fn object(&self) -> Option<&Item> {
        self.object.as_ref()
    }

    /// Lends the complete ordered response item, including unknown fields.
    pub fn with_ttlv<R>(&self, callback: impl for<'a> FnOnce(StructureView<'a>) -> R) -> R {
        self.response_ttlv.with_ttlv(callback)
    }
}

impl fmt::Debug for ExportResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ExportResponse")
            .field("result", &self.result)
            .field("object_type", &self.object_type)
            .field("has_unique_identifier", &self.unique_identifier.is_some())
            .field("has_attributes", &self.attributes.is_some())
            .field("has_object", &self.object.is_some())
            .field("payload", &"[REDACTED]")
            .field("response_ttlv", &"[REDACTED]")
            .finish()
    }
}

/// A payload-free Export request or response conversion error.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExportError {
    /// The response item identifies another operation.
    UnexpectedOperation,
    /// The validated response omitted Result Status.
    MissingResultStatus,
    /// The common KMIP result fields are inconsistent.
    InvalidOperationResult(ResultValidationError),
    /// A successful result omitted its Response Payload.
    MissingSuccessPayload,
    /// A successful payload omitted, duplicated, or mistyped a required field.
    MalformedSuccessPayload,
    /// The full response item could not be retained.
    MalformedResponseItem,
}

impl fmt::Display for ExportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedOperation => formatter.write_str("response item is not Export"),
            Self::MissingResultStatus => formatter.write_str("Export result status is missing"),
            Self::InvalidOperationResult(error) => {
                write!(formatter, "Export result is invalid: {error}")
            }
            Self::MissingSuccessPayload => {
                formatter.write_str("successful Export result has no payload")
            }
            Self::MalformedSuccessPayload => {
                formatter.write_str("Export response payload is malformed")
            }
            Self::MalformedResponseItem => formatter.write_str("Export response item is malformed"),
        }
    }
}

impl Error for ExportError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidOperationResult(error) => Some(error),
            _ => None,
        }
    }
}

fn parse_success_payload(
    payload: &StructureView<'_>,
) -> Result<(ObjectType, UniqueIdentifier, Item, Item), ExportError> {
    let mut object_type = None;
    let mut unique_identifier = None;
    let mut attributes = None;
    let mut object = None;
    let mut unknown_structures = Vec::new();
    for field in payload.children() {
        match field.tag().raw() {
            OBJECT_TYPE => {
                let value = field
                    .with_value(|value| match value {
                        ValueView::Enumeration(raw) => Some(ObjectType::from_raw(*raw)),
                        _ => None,
                    })
                    .ok_or(ExportError::MalformedSuccessPayload)?;
                set_once(&mut object_type, value)?;
            }
            UNIQUE_IDENTIFIER => {
                let value = field
                    .with_value(|value| parse_unique_identifier(&value))
                    .ok_or(ExportError::MalformedSuccessPayload)?;
                set_once(&mut unique_identifier, value)?;
            }
            ATTRIBUTES => {
                if field.item_type() != ItemType::Structure {
                    return Err(ExportError::MalformedSuccessPayload);
                }
                set_once(&mut attributes, clone_field(field)?)?;
            }
            raw if is_object_tag(raw) => {
                if field.item_type() != ItemType::Structure {
                    return Err(ExportError::MalformedSuccessPayload);
                }
                set_once(&mut object, clone_field(field)?)?;
            }
            _ if field.item_type() == ItemType::Structure => unknown_structures.push(field),
            _ => {}
        }
    }
    if object.is_none() && unknown_structures.len() == 1 {
        object = Some(
            unknown_structures[0]
                .try_clone()
                .map_err(|_| ExportError::MalformedSuccessPayload)?,
        );
    }
    Ok((
        object_type.ok_or(ExportError::MalformedSuccessPayload)?,
        unique_identifier.ok_or(ExportError::MalformedSuccessPayload)?,
        attributes.ok_or(ExportError::MalformedSuccessPayload)?,
        object.ok_or(ExportError::MalformedSuccessPayload)?,
    ))
}

fn set_once<T>(slot: &mut Option<T>, value: T) -> Result<(), ExportError> {
    if slot.is_some() {
        return Err(ExportError::MalformedSuccessPayload);
    }
    *slot = Some(value);
    Ok(())
}

fn clone_field(field: &Item) -> Result<Item, ExportError> {
    field
        .try_clone()
        .map_err(|_| ExportError::MalformedSuccessPayload)
}

fn is_object_tag(raw: u32) -> bool {
    matches!(
        raw,
        CERTIFICATE
            | CERTIFICATE_REQUEST
            | SYMMETRIC_KEY
            | PUBLIC_KEY
            | PRIVATE_KEY
            | SPLIT_KEY
            | SECRET_DATA
            | OPAQUE_OBJECT
            | PGP_KEY
    )
}

fn parse_unique_identifier(value: &ValueView<'_>) -> Option<UniqueIdentifier> {
    match value {
        ValueView::TextString(value) => Some(UniqueIdentifier::TextString((*value).to_owned())),
        ValueView::Enumeration(value) => Some(UniqueIdentifier::Enumeration(**value)),
        ValueView::Integer(value) => Some(UniqueIdentifier::Integer(**value)),
        _ => None,
    }
}
