//! Typed KMIP 2.1 Delete Attribute request and response models.

use std::error::Error;
use std::fmt;

use kmipkit_ttlv::{Structure, StructureView, Value};

use crate::asynchronous::{is_success, item, model_error};
use crate::attribute::{clone_item, copy_text_string};
use crate::{
    AttributeReference, CurrentAttribute, KmipOperationResult, ProtocolError,
    ResponseBatchItemView, ResultMessage, ResultValidationError,
};

const DELETE_ATTRIBUTE_OPERATION: u32 = 0x0000_000F;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const CURRENT_ATTRIBUTE: u32 = 0x0042_013C;

/// A client-to-server Delete Attribute request from OASIS KMIP v2.1 §6.1.13.
///
/// Table 202 permits the Unique Identifier, Current Attribute, and Attribute
/// Reference to be omitted independently. This model preserves the supplied
/// selectors without inferring remote object state or deletion semantics.
#[derive(Debug)]
pub struct DeleteAttributeRequest {
    unique_identifier: Option<String>,
    current_attribute: Option<CurrentAttribute>,
    attribute_reference: Option<AttributeReference>,
}

impl DeleteAttributeRequest {
    /// Creates a Delete Attribute request preserving each optional field.
    #[must_use]
    pub fn new(
        unique_identifier: Option<String>,
        current_attribute: Option<CurrentAttribute>,
        attribute_reference: Option<AttributeReference>,
    ) -> Self {
        Self {
            unique_identifier,
            current_attribute,
            attribute_reference,
        }
    }

    /// Returns the optional exact Unique Identifier.
    #[must_use]
    pub fn unique_identifier(&self) -> Option<&str> {
        self.unique_identifier.as_deref()
    }

    /// Returns the optional caller-supplied Current Attribute selector.
    #[must_use]
    pub const fn current_attribute(&self) -> Option<&CurrentAttribute> {
        self.current_attribute.as_ref()
    }

    /// Returns the optional caller-supplied Attribute Reference selector.
    #[must_use]
    pub const fn attribute_reference(&self) -> Option<&AttributeReference> {
        self.attribute_reference.as_ref()
    }

    /// Builds the ordered Request Payload Structure from §6.1.13, Table 202.
    ///
    /// # Errors
    ///
    /// Returns a sanitized protocol error if a reference, copied generic Item,
    /// fixed field tag, or enclosing TTLV Structure cannot be represented.
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
        if let Some(current_attribute) = &self.current_attribute {
            let mut attribute = Structure::new();
            attribute
                .try_push(clone_item(current_attribute.item())?)
                .map_err(model_error)?;
            payload
                .try_push(item(CURRENT_ATTRIBUTE, Value::structure(attribute))?)
                .map_err(model_error)?;
        }
        if let Some(attribute_reference) = &self.attribute_reference {
            payload
                .try_push(attribute_reference.to_ttlv_item()?)
                .map_err(model_error)?;
        }
        Ok(payload)
    }
}

/// A typed Delete Attribute response preserving the common result and
/// successful Unique Identifier.
#[derive(Debug)]
pub struct DeleteAttributeResponse {
    result: KmipOperationResult,
    unique_identifier: Option<String>,
}

impl DeleteAttributeResponse {
    /// Converts one validated Delete Attribute response batch item.
    ///
    /// Successful payloads follow OASIS KMIP v2.1 §6.1.13, Table 203 and
    /// contain exactly the required Unique Identifier. Non-success results
    /// preserve Result Status, optional Result Reason, and optional Result
    /// Message.
    ///
    /// # Errors
    ///
    /// Returns [`DeleteAttributeError`] for another operation, invalid common
    /// result metadata, a missing successful payload, or a malformed payload.
    pub fn try_from_response_item(
        item: ResponseBatchItemView<'_>,
    ) -> Result<Self, DeleteAttributeError> {
        if item.operation() != Some(DELETE_ATTRIBUTE_OPERATION) {
            return Err(DeleteAttributeError::UnexpectedOperation);
        }

        let status = item
            .result_status()
            .ok_or(DeleteAttributeError::MissingResultStatus)?;
        let message = item.with_result_message(|text| ResultMessage::new(text.to_owned()));
        let result = KmipOperationResult::new(status, item.result_reason(), message)
            .map_err(DeleteAttributeError::InvalidOperationResult)?;

        if !is_success(status) {
            return Ok(Self {
                result,
                unique_identifier: None,
            });
        }

        let unique_identifier = item
            .with_response_payload(|payload| parse_success_payload(&payload))
            .ok_or(DeleteAttributeError::MissingSuccessPayload)??;

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

/// A payload-free Delete Attribute response conversion error.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeleteAttributeError {
    /// The response item identifies a different KMIP operation.
    UnexpectedOperation,
    /// The validated response item does not expose Result Status.
    MissingResultStatus,
    /// Result Status, Result Reason, and Result Message violate the common contract.
    InvalidOperationResult(ResultValidationError),
    /// A successful response omitted its Response Payload.
    MissingSuccessPayload,
    /// A successful response payload does not match Table 203.
    MalformedSuccessPayload,
}

impl fmt::Display for DeleteAttributeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedOperation => {
                formatter.write_str("response item is not Delete Attribute")
            }
            Self::MissingResultStatus => {
                formatter.write_str("Delete Attribute result status is missing")
            }
            Self::InvalidOperationResult(cause) => {
                write!(
                    formatter,
                    "Delete Attribute operation result is invalid: {cause}"
                )
            }
            Self::MissingSuccessPayload => {
                formatter.write_str("successful Delete Attribute response has no payload")
            }
            Self::MalformedSuccessPayload => {
                formatter.write_str("Delete Attribute response payload is malformed")
            }
        }
    }
}

impl Error for DeleteAttributeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidOperationResult(cause) => Some(cause),
            _ => None,
        }
    }
}

fn parse_success_payload(payload: &StructureView<'_>) -> Result<String, DeleteAttributeError> {
    let fields = payload.children();
    if fields.len() != 1 {
        return Err(DeleteAttributeError::MalformedSuccessPayload);
    }
    let field = fields
        .first()
        .ok_or(DeleteAttributeError::MalformedSuccessPayload)?;
    if field.tag().raw() != UNIQUE_IDENTIFIER {
        return Err(DeleteAttributeError::MalformedSuccessPayload);
    }
    field
        .with_value(|value| copy_text_string(&value))
        .ok_or(DeleteAttributeError::MalformedSuccessPayload)
}
