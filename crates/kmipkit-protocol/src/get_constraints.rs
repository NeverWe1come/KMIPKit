//! Typed KMIP 2.1 Get Constraints request and response payloads.

use std::{error::Error, fmt};

use kmipkit_ttlv::{ModelError, Structure, ValueView};

use crate::{
    KmipOperationResult, ProtocolError, ResponseBatchItemView, ResultMessage, ResultValidationError,
};

const GET_CONSTRAINTS_OPERATION: u32 = 0x0000_0038;
const CONSTRAINTS_TAG: u32 = 0x0042_0168;
const SUCCESS: u32 = 0;

/// An empty Get Constraints request from OASIS KMIP v2.1 §6.1.22, Table 229.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GetConstraintsRequest;

impl GetConstraintsRequest {
    /// Creates an empty Get Constraints request.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Builds the empty Request Payload Structure.
    ///
    /// # Errors
    ///
    /// The fallible signature matches other typed request builders. An empty
    /// Structure is always representable.
    pub fn to_ttlv_payload(&self) -> Result<Structure, ProtocolError> {
        Ok(Structure::new())
    }
}

/// A Get Constraints operation result from OASIS KMIP v2.1 §6.1.22, Tables 230–231.
pub struct GetConstraintsResponse {
    result: KmipOperationResult,
    constraints: Option<Structure>,
}

impl GetConstraintsResponse {
    /// Converts one validated response batch item.
    ///
    /// # Errors
    ///
    /// Returns a sanitized error for an invalid operation, result, or success
    /// payload. Unknown nested Constraints members remain ordered and intact.
    pub fn try_from_response_item(
        item: ResponseBatchItemView<'_>,
    ) -> Result<Self, GetConstraintsError> {
        if item.operation() != Some(GET_CONSTRAINTS_OPERATION) {
            return Err(GetConstraintsError::UnexpectedOperation);
        }
        let status = item
            .result_status()
            .ok_or(GetConstraintsError::MissingResultStatus)?;
        let result_message = item.with_result_message(|text| ResultMessage::new(text.to_owned()));
        let result = KmipOperationResult::new(status, item.result_reason(), result_message)
            .map_err(GetConstraintsError::InvalidOperationResult)?;
        if status.raw() != SUCCESS {
            return Ok(Self {
                result,
                constraints: None,
            });
        }
        let constraints = item
            .with_response_payload(|payload| {
                let [field] = payload.children() else {
                    return Err(GetConstraintsError::MalformedSuccessPayload);
                };
                if field.tag().raw() != CONSTRAINTS_TAG {
                    return Err(GetConstraintsError::MalformedSuccessPayload);
                }
                field.with_value(|value| match value {
                    ValueView::Structure(structure) => structure
                        .try_clone()
                        .map_err(GetConstraintsError::TtlvModel),
                    _ => Err(GetConstraintsError::MalformedSuccessPayload),
                })
            })
            .ok_or(GetConstraintsError::MissingSuccessPayload)??;
        Ok(Self {
            result,
            constraints: Some(constraints),
        })
    }

    /// Returns the complete KMIP result.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }

    /// Returns the ordered Constraints Structure for a successful response.
    #[must_use]
    pub const fn constraints(&self) -> Option<&Structure> {
        self.constraints.as_ref()
    }
}

impl fmt::Debug for GetConstraintsResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("GetConstraintsResponse")
            .field("result", &self.result)
            .field("has_constraints", &self.constraints.is_some())
            .finish()
    }
}

/// A payload-free Get Constraints response conversion error.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GetConstraintsError {
    /// The response names another operation.
    UnexpectedOperation,
    /// The response has no Result Status.
    MissingResultStatus,
    /// The common result fields are inconsistent.
    InvalidOperationResult(ResultValidationError),
    /// A successful response omitted its Response Payload.
    MissingSuccessPayload,
    /// A successful payload lacks exactly one Constraints Structure.
    MalformedSuccessPayload,
    /// The generic TTLV Structure could not be copied safely.
    TtlvModel(ModelError),
}

impl fmt::Display for GetConstraintsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedOperation => {
                formatter.write_str("response item is not Get Constraints")
            }
            Self::MissingResultStatus => {
                formatter.write_str("Get Constraints result status is missing")
            }
            Self::InvalidOperationResult(error) => {
                write!(formatter, "invalid Get Constraints result: {error}")
            }
            Self::MissingSuccessPayload => {
                formatter.write_str("successful Get Constraints payload is missing")
            }
            Self::MalformedSuccessPayload => {
                formatter.write_str("successful Get Constraints payload is malformed")
            }
            Self::TtlvModel(error) => {
                write!(formatter, "invalid Get Constraints TTLV model: {error}")
            }
        }
    }
}

impl Error for GetConstraintsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidOperationResult(error) => Some(error),
            Self::TtlvModel(error) => Some(error),
            Self::UnexpectedOperation
            | Self::MissingResultStatus
            | Self::MissingSuccessPayload
            | Self::MalformedSuccessPayload => None,
        }
    }
}
