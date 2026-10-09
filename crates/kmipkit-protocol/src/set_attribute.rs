//! Typed KMIP 2.1 Set Attribute request and response models.

use std::error::Error;
use std::fmt;

use kmipkit_ttlv::{Structure, StructureView, Value};

use crate::asynchronous::{is_success, item, model_error};
use crate::attribute::{clone_item, copy_text_string};
use crate::{
    KmipOperationResult, NewAttribute, ProtocolError, ResponseBatchItemView, ResultMessage,
    ResultValidationError,
};

const SET_ATTRIBUTE_OPERATION: u32 = 0x0000_0031;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const NEW_ATTRIBUTE: u32 = 0x0042_013D;

/// A client-to-server Set Attribute request from OASIS KMIP v2.1 §6.1.51.
///
/// Table 322 permits an optional Unique Identifier and requires one New
/// Attribute. The request preserves server-side instance cardinality
/// semantics and does not construct remote object state.
#[derive(Debug)]
pub struct SetAttributeRequest {
    unique_identifier: Option<String>,
    new_attribute: NewAttribute,
}

impl SetAttributeRequest {
    /// Creates a Set Attribute request preserving the optional identifier and
    /// caller-supplied New Attribute.
    #[must_use]
    pub fn new(unique_identifier: Option<String>, new_attribute: NewAttribute) -> Self {
        Self {
            unique_identifier,
            new_attribute,
        }
    }

    /// Returns the optional exact Unique Identifier.
    #[must_use]
    pub fn unique_identifier(&self) -> Option<&str> {
        self.unique_identifier.as_deref()
    }

    /// Returns the caller-supplied New Attribute.
    #[must_use]
    pub const fn new_attribute(&self) -> &NewAttribute {
        &self.new_attribute
    }

    /// Builds the ordered Request Payload Structure from §6.1.51, Table 322.
    ///
    /// # Errors
    ///
    /// Returns a sanitized protocol error if a fixed field tag, copied generic
    /// Item, or enclosing TTLV Structure cannot be represented.
    pub fn to_ttlv_payload(&self) -> Result<Structure, ProtocolError> {
        let mut payload = Structure::new();
        if let Some(unique_identifier) = &self.unique_identifier {
            payload
                .try_push(item(
                    UNIQUE_IDENTIFIER,
                    Value::text_string(unique_identifier.clone()),
                )?)
                .map_err(model_error)?;
        }

        let mut attribute = Structure::new();
        attribute
            .try_push(clone_item(self.new_attribute.item())?)
            .map_err(model_error)?;
        payload
            .try_push(item(NEW_ATTRIBUTE, Value::structure(attribute))?)
            .map_err(model_error)?;
        Ok(payload)
    }
}

/// A typed Set Attribute response preserving the common result and successful
/// Unique Identifier.
#[derive(Debug)]
pub struct SetAttributeResponse {
    result: KmipOperationResult,
    unique_identifier: Option<String>,
}

impl SetAttributeResponse {
    /// Converts one validated Set Attribute response batch item.
    ///
    /// Successful payloads follow OASIS KMIP v2.1 §6.1.51, Table 323 and
    /// contain exactly the required Unique Identifier. Non-success results
    /// preserve Result Status, optional Result Reason, and optional Result
    /// Message.
    ///
    /// # Errors
    ///
    /// Returns [`SetAttributeError`] for another operation, invalid common
    /// result metadata, a missing successful payload, or a malformed payload.
    pub fn try_from_response_item(
        item: ResponseBatchItemView<'_>,
    ) -> Result<Self, SetAttributeError> {
        if item.operation() != Some(SET_ATTRIBUTE_OPERATION) {
            return Err(SetAttributeError::UnexpectedOperation);
        }

        let status = item
            .result_status()
            .ok_or(SetAttributeError::MissingResultStatus)?;
        let message = item.with_result_message(|text| ResultMessage::new(text.to_owned()));
        let result = KmipOperationResult::new(status, item.result_reason(), message)
            .map_err(SetAttributeError::InvalidOperationResult)?;

        if !is_success(status) {
            return Ok(Self {
                result,
                unique_identifier: None,
            });
        }

        let unique_identifier = item
            .with_response_payload(|payload| parse_success_payload(&payload))
            .ok_or(SetAttributeError::MissingSuccessPayload)??;

        Ok(Self {
            result,
            unique_identifier: Some(unique_identifier),
        })
    }

    /// Returns the complete server-reported KMIP operation result.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }

    /// Returns the successful response's exact Unique Identifier, if present.
    ///
    /// Non-success results return `None`.
    #[must_use]
    pub fn unique_identifier(&self) -> Option<&str> {
        self.unique_identifier.as_deref()
    }
}

/// A payload-free Set Attribute response conversion error.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SetAttributeError {
    /// The response item identifies a different KMIP operation.
    UnexpectedOperation,
    /// The validated response item does not expose Result Status.
    MissingResultStatus,
    /// Result Status, Result Reason, and Result Message violate the common contract.
    InvalidOperationResult(ResultValidationError),
    /// A successful response omitted its Response Payload.
    MissingSuccessPayload,
    /// A successful response payload does not match Table 323.
    MalformedSuccessPayload,
}

impl fmt::Display for SetAttributeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedOperation => formatter.write_str("response item is not Set Attribute"),
            Self::MissingResultStatus => {
                formatter.write_str("Set Attribute result status is missing")
            }
            Self::InvalidOperationResult(cause) => {
                write!(
                    formatter,
                    "Set Attribute operation result is invalid: {cause}"
                )
            }
            Self::MissingSuccessPayload => {
                formatter.write_str("successful Set Attribute response has no payload")
            }
            Self::MalformedSuccessPayload => {
                formatter.write_str("Set Attribute response payload is malformed")
            }
        }
    }
}

impl Error for SetAttributeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidOperationResult(cause) => Some(cause),
            _ => None,
        }
    }
}

fn parse_success_payload(payload: &StructureView<'_>) -> Result<String, SetAttributeError> {
    let fields = payload.children();
    if fields.len() != 1 {
        return Err(SetAttributeError::MalformedSuccessPayload);
    }
    let field = fields
        .first()
        .ok_or(SetAttributeError::MalformedSuccessPayload)?;
    if field.tag().raw() != UNIQUE_IDENTIFIER {
        return Err(SetAttributeError::MalformedSuccessPayload);
    }
    field
        .with_value(|value| copy_text_string(&value))
        .ok_or(SetAttributeError::MalformedSuccessPayload)
}
