//! Typed KMIP 2.1 Set Constraints request and response payloads.
//!
//! OASIS KMIP Specification v2.1 §6.1.52, Tables 325–327 defines the
//! operation; §7.7 defines the Constraints Structure.

use std::{error::Error, fmt};

use kmipkit_ttlv::{Structure, StructureView, Value};

use crate::cryptographic_operation::{self as common, OwnedResponseTtlv};
use crate::{
    KmipOperationResult, ProtocolError, ResponseBatchItemView, ResultMessage, ResultValidationError,
};

const SET_CONSTRAINTS_OPERATION: u32 = 0x0000_0037;
const CONSTRAINTS_TAG: u32 = 0x0042_0168;
const SUCCESS: u32 = 0;

/// A client-to-server Set Constraints request from §6.1.52, Table 325.
pub struct SetConstraintsRequest {
    constraints: Structure,
}

impl SetConstraintsRequest {
    /// Creates a request with the caller-supplied ordered Constraints structure.
    #[must_use]
    pub const fn new(constraints: Structure) -> Self {
        Self { constraints }
    }

    /// Lends the ordered Constraints structure supplied for this request.
    #[must_use]
    pub fn constraints(&self) -> StructureView<'_> {
        self.constraints.view()
    }

    /// Builds the Request Payload Structure defined by Table 325.
    ///
    /// # Errors
    ///
    /// Returns a sanitized error if the Constraints structure cannot be copied
    /// into the outgoing TTLV payload.
    pub fn to_ttlv_payload(&self) -> Result<Structure, ProtocolError> {
        let constraints = self
            .constraints
            .view()
            .try_clone()
            .map_err(|_| common::request_error())?;
        let mut payload = Structure::new();
        common::push(&mut payload, CONSTRAINTS_TAG, Value::structure(constraints))?;
        Ok(payload)
    }
}

impl fmt::Debug for SetConstraintsRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SetConstraintsRequest")
            .field("constraints", &"[REDACTED]")
            .finish()
    }
}

/// A Set Constraints result from §6.1.52, Tables 326–327.
pub struct SetConstraintsResponse {
    result: KmipOperationResult,
    response_ttlv: Option<OwnedResponseTtlv>,
}

impl SetConstraintsResponse {
    /// Converts one validated Set Constraints response batch item.
    ///
    /// On Success, Table 326 defines an empty operation-specific response
    /// payload. Structurally valid unknown success fields are retained for
    /// forward and vendor compatibility; Failure and Pending results expose
    /// only their typed common result and do not retain generic response TTLV.
    ///
    /// # Errors
    ///
    /// Returns a payload-free error for an unexpected operation, invalid
    /// common result, missing successful Response Payload, or malformed TTLV.
    pub fn try_from_response_item(
        item: ResponseBatchItemView<'_>,
    ) -> Result<Self, SetConstraintsError> {
        if item.operation() != Some(SET_CONSTRAINTS_OPERATION) {
            return Err(SetConstraintsError::UnexpectedOperation);
        }
        let status = item
            .result_status()
            .ok_or(SetConstraintsError::MissingResultStatus)?;
        let result_message = item.with_result_message(|text| ResultMessage::new(text.to_owned()));
        let result = KmipOperationResult::new(status, item.result_reason(), result_message)
            .map_err(SetConstraintsError::InvalidOperationResult)?;
        if status.raw() == SUCCESS && item.with_response_payload(|_| ()).is_none() {
            return Err(SetConstraintsError::MissingSuccessPayload);
        }
        let response_ttlv = if status.raw() == SUCCESS {
            Some(
                common::clone_response_item(item, "Set Constraints")
                    .map_err(|_| SetConstraintsError::MalformedResponseItem)?,
            )
        } else {
            None
        };

        Ok(Self {
            result,
            response_ttlv,
        })
    }

    /// Returns the complete KMIP status, reason, and optional Result Message.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }

    /// Lends the complete ordered success response item, including unknown fields.
    ///
    /// Failure and Pending results expose their typed common result but do not
    /// retain or expose generic response TTLV.
    pub fn with_ttlv<R>(&self, callback: impl for<'a> FnOnce(StructureView<'a>) -> R) -> Option<R> {
        self.response_ttlv
            .as_ref()
            .map(|response_ttlv| response_ttlv.with_ttlv(callback))
    }
}

impl fmt::Debug for SetConstraintsResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SetConstraintsResponse")
            .field("result", &self.result)
            .field("response_ttlv", &"[REDACTED]")
            .finish()
    }
}

/// A payload-free Set Constraints response conversion error.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SetConstraintsError {
    /// The response names another operation.
    UnexpectedOperation,
    /// The response has no Result Status.
    MissingResultStatus,
    /// The common result fields are inconsistent.
    InvalidOperationResult(ResultValidationError),
    /// A successful response omitted its Response Payload.
    MissingSuccessPayload,
    /// The generic response item could not be retained.
    MalformedResponseItem,
}

impl fmt::Display for SetConstraintsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedOperation => {
                formatter.write_str("response item is not Set Constraints")
            }
            Self::MissingResultStatus => {
                formatter.write_str("Set Constraints result status is missing")
            }
            Self::InvalidOperationResult(error) => {
                write!(formatter, "invalid Set Constraints result: {error}")
            }
            Self::MissingSuccessPayload => {
                formatter.write_str("successful Set Constraints payload is missing")
            }
            Self::MalformedResponseItem => {
                formatter.write_str("Set Constraints response item is malformed")
            }
        }
    }
}

impl Error for SetConstraintsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidOperationResult(error) => Some(error),
            Self::UnexpectedOperation
            | Self::MissingResultStatus
            | Self::MissingSuccessPayload
            | Self::MalformedResponseItem => None,
        }
    }
}
