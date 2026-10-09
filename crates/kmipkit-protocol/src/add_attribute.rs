//! Typed KMIP 2.1 Add Attribute request and response models.

use std::error::Error;
use std::fmt;

use kmipkit_ttlv::{Structure, StructureView, Value};

use crate::asynchronous::{is_success, item, model_error};
use crate::attribute::{clone_item, copy_text_string};
use crate::{
    KmipOperationResult, NewAttribute, ProtocolError, ResponseBatchItemView, ResultMessage,
    ResultValidationError,
};

const ADD_ATTRIBUTE_OPERATION: u32 = 0x0000_000D;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const NEW_ATTRIBUTE: u32 = 0x0042_013D;

/// A client-to-server Add Attribute request from OASIS KMIP v2.1 §6.1.2.
///
/// Table 167 permits an optional Unique Identifier and requires one New
/// Attribute. The supplied attribute remains one direct generic TTLV Item.
#[derive(Debug)]
pub struct AddAttributeRequest {
    unique_identifier: Option<String>,
    new_attribute: NewAttribute,
}

impl AddAttributeRequest {
    /// Creates an Add Attribute request preserving the optional identifier and
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

    /// Builds the ordered Request Payload Structure from §6.1.2, Table 167.
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

/// A typed Add Attribute response preserving the common result and successful
/// Unique Identifier.
#[derive(Debug)]
pub struct AddAttributeResponse {
    result: KmipOperationResult,
    unique_identifier: Option<String>,
}

impl AddAttributeResponse {
    /// Converts one validated Add Attribute response batch item.
    ///
    /// Successful payloads follow OASIS KMIP v2.1 §6.1.2, Table 168 and contain
    /// exactly the required Unique Identifier. Non-success results preserve
    /// Result Status, optional Result Reason, and optional Result Message.
    ///
    /// # Errors
    ///
    /// Returns [`AddAttributeError`] for a different operation, invalid common
    /// result metadata, a missing successful payload, or a malformed payload.
    pub fn try_from_response_item(
        item: ResponseBatchItemView<'_>,
    ) -> Result<Self, AddAttributeError> {
        if item.operation() != Some(ADD_ATTRIBUTE_OPERATION) {
            return Err(AddAttributeError::UnexpectedOperation);
        }

        let status = item
            .result_status()
            .ok_or(AddAttributeError::MissingResultStatus)?;
        let message = item.with_result_message(|text| ResultMessage::new(text.to_owned()));
        let result = KmipOperationResult::new(status, item.result_reason(), message)
            .map_err(AddAttributeError::InvalidOperationResult)?;

        if !is_success(status) {
            return Ok(Self {
                result,
                unique_identifier: None,
            });
        }

        let unique_identifier = item
            .with_response_payload(|payload| parse_success_payload(&payload))
            .ok_or(AddAttributeError::MissingSuccessPayload)??;

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

/// A payload-free Add Attribute response conversion error.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AddAttributeError {
    /// The response item identifies a different KMIP operation.
    UnexpectedOperation,
    /// The validated response item does not expose Result Status.
    MissingResultStatus,
    /// Result Status, Result Reason, and Result Message violate the common contract.
    InvalidOperationResult(ResultValidationError),
    /// A successful response omitted its Response Payload.
    MissingSuccessPayload,
    /// A successful response payload does not match Table 168.
    MalformedSuccessPayload,
}

impl fmt::Display for AddAttributeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedOperation => formatter.write_str("response item is not Add Attribute"),
            Self::MissingResultStatus => {
                formatter.write_str("Add Attribute result status is missing")
            }
            Self::InvalidOperationResult(cause) => {
                write!(
                    formatter,
                    "Add Attribute operation result is invalid: {cause}"
                )
            }
            Self::MissingSuccessPayload => {
                formatter.write_str("successful Add Attribute response has no payload")
            }
            Self::MalformedSuccessPayload => {
                formatter.write_str("Add Attribute response payload is malformed")
            }
        }
    }
}

impl Error for AddAttributeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidOperationResult(cause) => Some(cause),
            _ => None,
        }
    }
}

fn parse_success_payload(payload: &StructureView<'_>) -> Result<String, AddAttributeError> {
    let fields = payload.children();
    if fields.len() != 1 {
        return Err(AddAttributeError::MalformedSuccessPayload);
    }
    let field = fields
        .first()
        .ok_or(AddAttributeError::MalformedSuccessPayload)?;
    if field.tag().raw() != UNIQUE_IDENTIFIER {
        return Err(AddAttributeError::MalformedSuccessPayload);
    }
    field
        .with_value(|value| copy_text_string(&value))
        .ok_or(AddAttributeError::MalformedSuccessPayload)
}
