//! Typed KMIP 2.1 Modify Attribute request and response models.

use std::error::Error;
use std::fmt;

use kmipkit_ttlv::{Structure, StructureView, Value};

use crate::asynchronous::{is_success, item, model_error};
use crate::attribute::{clone_item, copy_text_string};
use crate::{
    CurrentAttribute, KmipOperationResult, NewAttribute, ProtocolError, ResponseBatchItemView,
    ResultMessage, ResultValidationError,
};

const MODIFY_ATTRIBUTE_OPERATION: u32 = 0x0000_000E;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const CURRENT_ATTRIBUTE: u32 = 0x0042_013C;
const NEW_ATTRIBUTE: u32 = 0x0042_013D;

/// A client-to-server Modify Attribute request from OASIS KMIP v2.1 §6.1.34.
///
/// Table 265 permits optional Unique Identifier and Current Attribute fields
/// followed by required New Attribute. Omission of Current Attribute is
/// retained for server-side instance selection.
#[derive(Debug)]
pub struct ModifyAttributeRequest {
    unique_identifier: Option<String>,
    current_attribute: Option<CurrentAttribute>,
    new_attribute: NewAttribute,
}

impl ModifyAttributeRequest {
    /// Creates a Modify Attribute request preserving its optional selector and
    /// required caller-supplied replacement.
    #[must_use]
    pub fn new(
        unique_identifier: Option<String>,
        current_attribute: Option<CurrentAttribute>,
        new_attribute: NewAttribute,
    ) -> Self {
        Self {
            unique_identifier,
            current_attribute,
            new_attribute,
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

    /// Returns the required caller-supplied New Attribute.
    #[must_use]
    pub const fn new_attribute(&self) -> &NewAttribute {
        &self.new_attribute
    }

    /// Builds the ordered Request Payload Structure from §6.1.34, Table 265.
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
        if let Some(current_attribute) = &self.current_attribute {
            let mut attribute = Structure::new();
            attribute
                .try_push(clone_item(current_attribute.item())?)
                .map_err(model_error)?;
            payload
                .try_push(item(CURRENT_ATTRIBUTE, Value::structure(attribute))?)
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

/// A typed Modify Attribute response preserving the common result and
/// successful Unique Identifier.
#[derive(Debug)]
pub struct ModifyAttributeResponse {
    result: KmipOperationResult,
    unique_identifier: Option<String>,
}

impl ModifyAttributeResponse {
    /// Converts one validated Modify Attribute response batch item.
    ///
    /// Successful payloads follow OASIS KMIP v2.1 §6.1.34, Table 266 and
    /// contain exactly the required Unique Identifier. Non-success results
    /// preserve Result Status, optional Result Reason, and optional Result
    /// Message.
    ///
    /// # Errors
    ///
    /// Returns [`ModifyAttributeError`] for another operation, invalid common
    /// result metadata, a missing successful payload, or a malformed payload.
    pub fn try_from_response_item(
        item: ResponseBatchItemView<'_>,
    ) -> Result<Self, ModifyAttributeError> {
        if item.operation() != Some(MODIFY_ATTRIBUTE_OPERATION) {
            return Err(ModifyAttributeError::UnexpectedOperation);
        }

        let status = item
            .result_status()
            .ok_or(ModifyAttributeError::MissingResultStatus)?;
        let message = item.with_result_message(|text| ResultMessage::new(text.to_owned()));
        let result = KmipOperationResult::new(status, item.result_reason(), message)
            .map_err(ModifyAttributeError::InvalidOperationResult)?;

        if !is_success(status) {
            return Ok(Self {
                result,
                unique_identifier: None,
            });
        }

        let unique_identifier = item
            .with_response_payload(|payload| parse_success_payload(&payload))
            .ok_or(ModifyAttributeError::MissingSuccessPayload)??;

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

/// A payload-free Modify Attribute response conversion error.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModifyAttributeError {
    /// The response item identifies a different KMIP operation.
    UnexpectedOperation,
    /// The validated response item does not expose Result Status.
    MissingResultStatus,
    /// Result Status, Result Reason, and Result Message violate the common contract.
    InvalidOperationResult(ResultValidationError),
    /// A successful response omitted its Response Payload.
    MissingSuccessPayload,
    /// A successful response payload does not match Table 266.
    MalformedSuccessPayload,
}

impl fmt::Display for ModifyAttributeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedOperation => {
                formatter.write_str("response item is not Modify Attribute")
            }
            Self::MissingResultStatus => {
                formatter.write_str("Modify Attribute result status is missing")
            }
            Self::InvalidOperationResult(cause) => {
                write!(
                    formatter,
                    "Modify Attribute operation result is invalid: {cause}"
                )
            }
            Self::MissingSuccessPayload => {
                formatter.write_str("successful Modify Attribute response has no payload")
            }
            Self::MalformedSuccessPayload => {
                formatter.write_str("Modify Attribute response payload is malformed")
            }
        }
    }
}

impl Error for ModifyAttributeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidOperationResult(cause) => Some(cause),
            _ => None,
        }
    }
}

fn parse_success_payload(payload: &StructureView<'_>) -> Result<String, ModifyAttributeError> {
    let fields = payload.children();
    if fields.len() != 1 {
        return Err(ModifyAttributeError::MalformedSuccessPayload);
    }
    let field = fields
        .first()
        .ok_or(ModifyAttributeError::MalformedSuccessPayload)?;
    if field.tag().raw() != UNIQUE_IDENTIFIER {
        return Err(ModifyAttributeError::MalformedSuccessPayload);
    }
    field
        .with_value(|value| copy_text_string(&value))
        .ok_or(ModifyAttributeError::MalformedSuccessPayload)
}
