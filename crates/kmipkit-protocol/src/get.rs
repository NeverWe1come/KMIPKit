//! Typed KMIP 2.1 Get request and response payloads.

use std::error::Error;
use std::fmt;

use kmipkit_ttlv::{Item, ModelError, RawTag, Structure, StructureView, Tag, Value, ValueView};

use crate::{
    KmipOperationResult, ObjectType, ProtocolCauseCategory, ProtocolError, ProtocolErrorKind,
    ResponseBatchItemView, ResultMessage, ResultValidationError, UniqueIdentifier,
};

const GET_OPERATION: u32 = 0x0000_000A;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const KEY_FORMAT_TYPE: u32 = 0x0042_0042;
const KEY_WRAP_TYPE: u32 = 0x0042_00F8;
const KEY_COMPRESSION_TYPE: u32 = 0x0042_0041;
const KEY_WRAPPING_SPECIFICATION: u32 = 0x0042_0047;
const OBJECT_TYPE: u32 = 0x0042_0057;
const SUCCESS: u32 = 0;

/// A KMIP Key Format Type Enumeration that preserves unrecognized values.
///
/// Values are defined by OASIS KMIP v2.1 §11.25, Table 461.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct KeyFormatType(u32);

impl KeyFormatType {
    /// Creates a Key Format Type from its raw unsigned Enumeration value.
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

/// A KMIP Key Wrap Type Enumeration that preserves unrecognized values.
///
/// Values are defined by OASIS KMIP v2.1 §11.29.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct KeyWrapType(u32);

impl KeyWrapType {
    /// Creates a Key Wrap Type from its raw unsigned Enumeration value.
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

/// A KMIP Key Compression Type Enumeration that preserves unrecognized values.
///
/// Values are defined by OASIS KMIP v2.1 §11.24, Table 459.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct KeyCompressionType(u32);

impl KeyCompressionType {
    /// Creates a Key Compression Type from its raw unsigned Enumeration value.
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

/// A typed KMIP 2.1 Get request payload from §6.1.19, Table 220.
///
/// Every selector is optional. An omitted Unique Identifier remains omitted so
/// a caller can use a server-side ID Placeholder when the protocol context
/// provides one.
pub struct GetRequest {
    unique_identifier: Option<UniqueIdentifier>,
    key_format_type: Option<KeyFormatType>,
    key_wrap_type: Option<KeyWrapType>,
    key_compression_type: Option<KeyCompressionType>,
    key_wrapping_specification: Option<Structure>,
}

impl GetRequest {
    /// Creates a Get request with all optional fields absent.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            unique_identifier: None,
            key_format_type: None,
            key_wrap_type: None,
            key_compression_type: None,
            key_wrapping_specification: None,
        }
    }

    /// Sets the optional Unique Identifier in its original KMIP wire form.
    #[must_use]
    pub fn with_unique_identifier(mut self, unique_identifier: UniqueIdentifier) -> Self {
        self.unique_identifier = Some(unique_identifier);
        self
    }

    /// Sets the optional Key Format Type, including a future raw value.
    #[must_use]
    pub const fn with_key_format_type(mut self, key_format_type: KeyFormatType) -> Self {
        self.key_format_type = Some(key_format_type);
        self
    }

    /// Sets the optional Key Wrap Type, including a future raw value.
    #[must_use]
    pub const fn with_key_wrap_type(mut self, key_wrap_type: KeyWrapType) -> Self {
        self.key_wrap_type = Some(key_wrap_type);
        self
    }

    /// Sets the optional Key Compression Type, including a future raw value.
    #[must_use]
    pub const fn with_key_compression_type(
        mut self,
        key_compression_type: KeyCompressionType,
    ) -> Self {
        self.key_compression_type = Some(key_compression_type);
        self
    }

    /// Sets the optional caller-supplied Key Wrapping Specification.
    #[must_use]
    pub fn with_key_wrapping_specification(mut self, specification: Structure) -> Self {
        self.key_wrapping_specification = Some(specification);
        self
    }

    /// Returns the optional Unique Identifier in its original KMIP wire form.
    #[must_use]
    pub const fn unique_identifier(&self) -> Option<&UniqueIdentifier> {
        self.unique_identifier.as_ref()
    }

    /// Returns the optional Key Format Type raw-value wrapper.
    #[must_use]
    pub const fn key_format_type(&self) -> Option<KeyFormatType> {
        self.key_format_type
    }

    /// Returns the optional Key Wrap Type raw-value wrapper.
    #[must_use]
    pub const fn key_wrap_type(&self) -> Option<KeyWrapType> {
        self.key_wrap_type
    }

    /// Returns the optional Key Compression Type raw-value wrapper.
    #[must_use]
    pub const fn key_compression_type(&self) -> Option<KeyCompressionType> {
        self.key_compression_type
    }

    /// Returns the optional caller-supplied Key Wrapping Specification.
    #[must_use]
    pub const fn key_wrapping_specification(&self) -> Option<&Structure> {
        self.key_wrapping_specification.as_ref()
    }

    /// Builds the ordered Request Payload Structure from Table 220.
    ///
    /// This consumes the request so a Key Wrapping Specification can be moved
    /// into the returned zeroizing TTLV tree without an extra secret copy. No
    /// field or cryptographic option is synthesized.
    ///
    /// # Errors
    ///
    /// Returns a sanitized protocol error if a field tag or nested Structure
    /// violates the TTLV model constraints.
    pub fn to_ttlv_payload(self) -> Result<Structure, ProtocolError> {
        let mut payload = Structure::new();
        if let Some(unique_identifier) = self.unique_identifier {
            let value = match unique_identifier {
                UniqueIdentifier::TextString(value) => Value::text_string(value),
                UniqueIdentifier::Enumeration(value) => Value::enumeration(value),
                UniqueIdentifier::Integer(value) => Value::integer(value),
            };
            payload
                .try_push(item(UNIQUE_IDENTIFIER, value)?)
                .map_err(protocol_model_error)?;
        }
        if let Some(key_format_type) = self.key_format_type {
            payload
                .try_push(item(
                    KEY_FORMAT_TYPE,
                    Value::enumeration(key_format_type.raw()),
                )?)
                .map_err(protocol_model_error)?;
        }
        if let Some(key_wrap_type) = self.key_wrap_type {
            payload
                .try_push(item(
                    KEY_WRAP_TYPE,
                    Value::enumeration(key_wrap_type.raw()),
                )?)
                .map_err(protocol_model_error)?;
        }
        if let Some(key_compression_type) = self.key_compression_type {
            payload
                .try_push(item(
                    KEY_COMPRESSION_TYPE,
                    Value::enumeration(key_compression_type.raw()),
                )?)
                .map_err(protocol_model_error)?;
        }
        if let Some(specification) = self.key_wrapping_specification {
            payload
                .try_push(item(
                    KEY_WRAPPING_SPECIFICATION,
                    Value::structure(specification),
                )?)
                .map_err(protocol_model_error)?;
        }
        Ok(payload)
    }
}

impl Default for GetRequest {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for GetRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("GetRequest([REDACTED])")
    }
}

/// The typed result of one KMIP 2.1 Get response from §6.1.19, Tables 221–222.
///
/// A successful response owns the complete generic Any Object item. That item
/// preserves its original tag and descendant order, redacts its values in
/// formatting, and zeroizes KMIPKit-owned payload memory when dropped.
pub struct GetResponse {
    result: KmipOperationResult,
    object_type: Option<ObjectType>,
    unique_identifier: Option<UniqueIdentifier>,
    object: Option<Item>,
}

impl GetResponse {
    /// Converts one already validated KMIP Get response batch item.
    ///
    /// A successful result requires exactly one Object Type Enumeration, one
    /// Unique Identifier in a permitted wire form, and one Structure-valued
    /// Any Object. Other statuses retain only the common operation result.
    /// The Any Object is copied with [`Item::try_clone`] while its
    /// callback-scoped response-tree view is valid.
    ///
    /// # Errors
    ///
    /// Returns a payload-free [`GetError`] for a different operation, invalid
    /// result metadata, a missing success payload, or malformed required
    /// fields. Errors do not include server field values or object contents.
    pub fn try_from_response_item(item: ResponseBatchItemView<'_>) -> Result<Self, GetError> {
        if item.operation() != Some(GET_OPERATION) {
            return Err(GetError::UnexpectedOperation);
        }
        let status = item.result_status().ok_or(GetError::MissingResultStatus)?;
        let result_message = item.with_result_message(|text| ResultMessage::new(text.to_owned()));
        let result = KmipOperationResult::new(status, item.result_reason(), result_message)
            .map_err(GetError::InvalidOperationResult)?;

        if status.raw() != SUCCESS {
            return Ok(Self {
                result,
                object_type: None,
                unique_identifier: None,
                object: None,
            });
        }

        let (object_type, unique_identifier, object) = item
            .with_response_payload(|payload| parse_success_payload(&payload))
            .ok_or(GetError::MissingSuccessPayload)??;
        Ok(Self {
            result,
            object_type: Some(object_type),
            unique_identifier: Some(unique_identifier),
            object: Some(object),
        })
    }

    /// Returns the complete KMIP result, including raw status and optional
    /// reason and redacted Result Message.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }

    /// Returns the response Object Type for a successful Get result.
    #[must_use]
    pub const fn object_type(&self) -> Option<ObjectType> {
        self.object_type
    }

    /// Returns the successful-response Unique Identifier in its original
    /// KMIP wire form.
    #[must_use]
    pub const fn unique_identifier(&self) -> Option<&UniqueIdentifier> {
        self.unique_identifier.as_ref()
    }

    /// Returns the owned opaque Any Object item for a successful response.
    ///
    /// Its descendants remain ordered and unparsed. KMIPKit-owned contents are
    /// zeroized when this response is dropped; copies retained by a caller are
    /// outside that guarantee.
    #[must_use]
    pub const fn object(&self) -> Option<&Item> {
        self.object.as_ref()
    }
}

impl fmt::Debug for GetResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("GetResponse")
            .field("result", &self.result)
            .field("object_type", &self.object_type)
            .field("unique_identifier", &self.unique_identifier)
            .field("object", &self.object.as_ref().map(|_| "[REDACTED]"))
            .finish()
    }
}

/// A payload-free Get response conversion error.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GetError {
    /// The response identifies a different operation.
    UnexpectedOperation,
    /// The validated response omitted Result Status.
    MissingResultStatus,
    /// The Result Status/Reason combination violates the common KMIP contract.
    InvalidOperationResult(ResultValidationError),
    /// A successful response omitted its required Response Payload Structure.
    MissingSuccessPayload,
    /// A successful payload omits, duplicates, or mistypes a required field.
    MalformedSuccessPayload,
}

impl fmt::Display for GetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedOperation => formatter.write_str("response item is not Get"),
            Self::MissingResultStatus => formatter.write_str("Get result status is missing"),
            Self::InvalidOperationResult(cause) => {
                write!(formatter, "Get operation result is invalid: {cause}")
            }
            Self::MissingSuccessPayload => {
                formatter.write_str("successful Get response has no payload")
            }
            Self::MalformedSuccessPayload => {
                formatter.write_str("Get response payload is malformed")
            }
        }
    }
}

impl Error for GetError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidOperationResult(cause) => Some(cause),
            _ => None,
        }
    }
}

fn parse_success_payload(
    payload: &StructureView<'_>,
) -> Result<(ObjectType, UniqueIdentifier, Item), GetError> {
    let mut object_type = None;
    let mut unique_identifier = None;
    let mut object = None;

    for field in payload.children() {
        match field.tag().raw() {
            OBJECT_TYPE => {
                if object_type.is_some() {
                    return Err(GetError::MalformedSuccessPayload);
                }
                object_type = field.with_value(|value| match value {
                    ValueView::Enumeration(raw) => Some(ObjectType::from_raw(*raw)),
                    _ => None,
                });
                if object_type.is_none() {
                    return Err(GetError::MalformedSuccessPayload);
                }
            }
            UNIQUE_IDENTIFIER => {
                if unique_identifier.is_some() {
                    return Err(GetError::MalformedSuccessPayload);
                }
                unique_identifier = field.with_value(|value| parse_unique_identifier(&value));
                if unique_identifier.is_none() {
                    return Err(GetError::MalformedSuccessPayload);
                }
            }
            _ => {
                if object.is_some() || field.item_type() != kmipkit_ttlv::ItemType::Structure {
                    return Err(GetError::MalformedSuccessPayload);
                }
                object = Some(
                    field
                        .try_clone()
                        .map_err(|_| GetError::MalformedSuccessPayload)?,
                );
            }
        }
    }

    match (object_type, unique_identifier, object) {
        (Some(object_type), Some(unique_identifier), Some(object)) => {
            Ok((object_type, unique_identifier, object))
        }
        _ => Err(GetError::MalformedSuccessPayload),
    }
}

fn parse_unique_identifier(value: &ValueView<'_>) -> Option<UniqueIdentifier> {
    match value {
        ValueView::TextString(value) => Some(UniqueIdentifier::TextString((*value).to_owned())),
        ValueView::Enumeration(value) => Some(UniqueIdentifier::Enumeration(**value)),
        ValueView::Integer(value) => Some(UniqueIdentifier::Integer(**value)),
        _ => None,
    }
}

fn item(raw_tag: u32, value: Value) -> Result<Item, ProtocolError> {
    Item::new(tag(raw_tag)?, value).map_err(protocol_model_error)
}

fn tag(raw_tag: u32) -> Result<Tag, ProtocolError> {
    RawTag::new(raw_tag)
        .and_then(|raw| raw.try_checked())
        .map_err(protocol_model_error)
}

fn protocol_model_error(error: ModelError) -> ProtocolError {
    ProtocolError::new(
        ProtocolErrorKind::InvalidValue,
        ProtocolCauseCategory::InvalidValue,
        error,
    )
}
