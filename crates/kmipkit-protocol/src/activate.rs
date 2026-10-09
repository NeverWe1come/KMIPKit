//! Typed KMIP v2.1 Activate request and response payloads.

use std::error::Error;
use std::fmt;

use kmipkit_ttlv::{Item, ModelError, RawTag, Structure, Tag, Value, ValueView};

use crate::{
    KmipOperationResult, ProtocolCauseCategory, ProtocolError, ProtocolErrorKind,
    ResponseBatchItemView, ResultMessage, ResultValidationError, UniqueIdentifier,
};

const ACTIVATE_OPERATION: u32 = 0x0000_0012;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const SUCCESS: u32 = 0;

/// A typed KMIP 2.1 Activate request payload from §6.1.1, Table 164.
///
/// The optional Unique Identifier is preserved exactly as supplied. Omission
/// is not replaced with a generated value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivateRequest {
    unique_identifier: Option<UniqueIdentifier>,
}

impl ActivateRequest {
    /// Creates an Activate request while preserving the optional identifier.
    #[must_use]
    pub const fn new(unique_identifier: Option<UniqueIdentifier>) -> Self {
        Self { unique_identifier }
    }

    /// Returns the optional Unique Identifier in its original wire form.
    #[must_use]
    pub const fn unique_identifier(&self) -> Option<&UniqueIdentifier> {
        self.unique_identifier.as_ref()
    }

    /// Builds the ordered Request Payload Structure defined by Table 164.
    ///
    /// # Errors
    ///
    /// Returns a sanitized protocol error if the fixed tag or resulting TTLV
    /// item cannot be represented.
    pub fn to_ttlv_payload(&self) -> Result<Structure, ProtocolError> {
        let mut payload = Structure::new();
        if let Some(identifier) = &self.unique_identifier {
            let value = match identifier {
                UniqueIdentifier::TextString(value) => Value::text_string(value.clone()),
                UniqueIdentifier::Enumeration(value) => Value::enumeration(*value),
                UniqueIdentifier::Integer(value) => Value::integer(*value),
            };
            payload
                .try_push(item(UNIQUE_IDENTIFIER, value)?)
                .map_err(model_error)?;
        }
        Ok(payload)
    }
}

/// A typed Activate result for one validated KMIP response batch item.
///
/// A successful response exposes the required Unique Identifier in its exact
/// §6.1.1, Table 165 wire form. Non-success results retain the shared KMIP
/// operation result and have no typed success identifier. The source
/// [`crate::ResponseMessage`] retains the complete generic payload, including
/// unknown or future fields.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivateResponse {
    result: KmipOperationResult,
    unique_identifier: Option<UniqueIdentifier>,
}

impl ActivateResponse {
    /// Converts one already validated KMIP response batch item.
    ///
    /// The item must identify the client-to-server Activate operation (`0x12`).
    /// Successful payload parsing requires exactly one Unique Identifier in a
    /// form permitted by §11.56. Unknown fields remain available from the
    /// original response message.
    ///
    /// # Errors
    ///
    /// Returns [`ActivateError`] for another operation, invalid result
    /// metadata, or a malformed required success payload. Errors do not
    /// include server-provided payload values.
    pub fn try_from_response_item(item: ResponseBatchItemView<'_>) -> Result<Self, ActivateError> {
        if item.operation() != Some(ACTIVATE_OPERATION) {
            return Err(ActivateError::UnexpectedOperation);
        }

        let status = item
            .result_status()
            .ok_or(ActivateError::MissingResultStatus)?;
        let result_message = item.with_result_message(|text| ResultMessage::new(text.to_owned()));
        let result = KmipOperationResult::new(status, item.result_reason(), result_message)
            .map_err(ActivateError::InvalidOperationResult)?;

        if status.raw() != SUCCESS {
            return Ok(Self {
                result,
                unique_identifier: None,
            });
        }

        let unique_identifier = item
            .with_response_payload(|payload| parse_success_payload(&payload))
            .ok_or(ActivateError::MalformedSuccessPayload)??;
        Ok(Self {
            result,
            unique_identifier: Some(unique_identifier),
        })
    }

    /// Returns the exact KMIP operation result.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }

    /// Returns the required successful-response Unique Identifier, if present.
    #[must_use]
    pub const fn unique_identifier(&self) -> Option<&UniqueIdentifier> {
        self.unique_identifier.as_ref()
    }
}

/// A payload-free error converting a validated Activate response item.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivateError {
    /// The response item is not the client-to-server Activate operation.
    UnexpectedOperation,
    /// The validated response item did not expose a Result Status.
    MissingResultStatus,
    /// The represented KMIP result violates the shared status/reason contract.
    InvalidOperationResult(ResultValidationError),
    /// A successful response omitted or malformed its required identifier.
    MalformedSuccessPayload,
}

impl fmt::Display for ActivateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedOperation => formatter.write_str("response item is not Activate"),
            Self::MissingResultStatus => formatter.write_str("Activate result status is missing"),
            Self::InvalidOperationResult(cause) => {
                write!(formatter, "Activate operation result is invalid: {cause}")
            }
            Self::MalformedSuccessPayload => {
                formatter.write_str("successful Activate response payload is malformed")
            }
        }
    }
}

impl Error for ActivateError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidOperationResult(cause) => Some(cause),
            _ => None,
        }
    }
}

fn parse_success_payload(
    payload: &kmipkit_ttlv::StructureView<'_>,
) -> Result<UniqueIdentifier, ActivateError> {
    let mut unique_identifier = None;
    for field in payload.children() {
        if field.tag().raw() == UNIQUE_IDENTIFIER {
            if unique_identifier.is_some() {
                return Err(ActivateError::MalformedSuccessPayload);
            }
            unique_identifier = parse_unique_identifier(field);
            if unique_identifier.is_none() {
                return Err(ActivateError::MalformedSuccessPayload);
            }
        }
    }

    unique_identifier.ok_or(ActivateError::MalformedSuccessPayload)
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
