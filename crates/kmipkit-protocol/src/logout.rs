//! Typed KMIP 2.1 Logout request and response payloads.
//!
//! Fields follow OASIS KMIP Specification v2.1 §6.1.31, Tables 256–258.
//! The request Ticket is defined by §7.39, Table 391.

use std::{error::Error, fmt};

use kmipkit_ttlv::{Structure, Value};

use crate::cryptographic_operation;
use crate::{
    KmipOperationResult, ProtocolError, ResponseBatchItemView, ResultMessage,
    ResultValidationError, Ticket,
};

const OPERATION: u32 = 0x0000_002E;
const TICKET: u32 = 0x0042_0149;
const SUCCESS: u32 = 0;

/// A typed client-to-server Logout request from §6.1.31, Table 256.
///
/// A caller must supply the exact Ticket to invalidate. The Ticket remains
/// secret-safe in diagnostics and is consumed when building the request.
pub struct LogoutRequest {
    ticket: Ticket,
}

impl LogoutRequest {
    /// Creates a Logout request with the required caller-selected Ticket.
    #[must_use]
    pub const fn new(ticket: Ticket) -> Self {
        Self { ticket }
    }

    /// Encodes the required Table 256 Ticket without interpreting its value.
    ///
    /// # Errors
    ///
    /// Returns a sanitized protocol error if the nested Ticket exceeds TTLV
    /// model limits.
    pub fn into_ttlv_payload(self) -> Result<Structure, ProtocolError> {
        let mut payload = Structure::new();
        cryptographic_operation::push(
            &mut payload,
            TICKET,
            Value::structure(self.ticket.into_ttlv()),
        )?;
        Ok(payload)
    }
}

impl fmt::Debug for LogoutRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LogoutRequest")
            .field("ticket", &self.ticket)
            .finish()
    }
}

/// A typed Logout result from §6.1.31, Tables 257–258.
#[derive(Debug)]
pub struct LogoutResponse {
    result: KmipOperationResult,
}

impl LogoutResponse {
    /// Converts one validated Logout response item.
    ///
    /// Success requires the empty Response Payload Structure from Table 257.
    /// Failure and Pending preserve the common result metadata.
    ///
    /// # Errors
    ///
    /// Returns [`LogoutError`] for another operation, inconsistent result
    /// metadata, or a missing/non-empty success payload.
    pub fn try_from_response_item(item: ResponseBatchItemView<'_>) -> Result<Self, LogoutError> {
        if item.operation() != Some(OPERATION) {
            return Err(LogoutError::UnexpectedOperation);
        }
        let status = item
            .result_status()
            .ok_or(LogoutError::MissingResultStatus)?;
        let result_message = item.with_result_message(|text| ResultMessage::new(text.to_owned()));
        let result = KmipOperationResult::new(status, item.result_reason(), result_message)
            .map_err(LogoutError::InvalidOperationResult)?;

        if status.raw() == SUCCESS {
            let member_count = item
                .with_response_payload(|payload| payload.children().len())
                .ok_or(LogoutError::MissingResponsePayload)?;
            if member_count != 0 {
                return Err(LogoutError::MalformedResponsePayload);
            }
        }

        Ok(Self { result })
    }

    /// Returns the complete KMIP result, including status and optional reason.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }
}

/// A payload-free error converting a Logout response batch item.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LogoutError {
    /// The response item identifies an operation other than Logout.
    UnexpectedOperation,
    /// The validated response item is missing Result Status.
    MissingResultStatus,
    /// The common result metadata is inconsistent.
    InvalidOperationResult(ResultValidationError),
    /// A successful Logout response omitted its empty Response Payload.
    MissingResponsePayload,
    /// A successful Logout response contains a non-empty Response Payload.
    MalformedResponsePayload,
}

impl fmt::Display for LogoutError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedOperation => formatter.write_str("response item is not Logout"),
            Self::MissingResultStatus => formatter.write_str("Logout result status is missing"),
            Self::InvalidOperationResult(error) => {
                write!(formatter, "Logout operation result is invalid: {error}")
            }
            Self::MissingResponsePayload => {
                formatter.write_str("successful Logout response payload is missing")
            }
            Self::MalformedResponsePayload => {
                formatter.write_str("successful Logout response payload is not empty")
            }
        }
    }
}

impl Error for LogoutError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidOperationResult(error) => Some(error),
            Self::UnexpectedOperation
            | Self::MissingResultStatus
            | Self::MissingResponsePayload
            | Self::MalformedResponsePayload => None,
        }
    }
}
