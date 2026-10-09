//! Typed KMIP 2.1 Ping request and response payloads.

use std::{error::Error, fmt};

use kmipkit_ttlv::Structure;

use crate::{
    KmipOperationResult, ProtocolError, ResponseBatchItemView, ResultMessage, ResultValidationError,
};

const PING_OPERATION: u32 = 0x0000_003b;
const SUCCESS: u32 = 0;
const PENDING: u32 = 2;

/// A client-to-server Ping request from OASIS KMIP v2.1 §6.1.36.
///
/// Ping has an empty Request Payload Structure.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PingRequest {
    _private: (),
}

impl PingRequest {
    /// Creates an empty Ping request.
    #[must_use]
    pub const fn new() -> Self {
        Self { _private: () }
    }

    /// Builds the empty Request Payload Structure defined by §6.1.36, Table 271.
    ///
    /// # Errors
    ///
    /// This method has an error return for consistency with operation payload
    /// conversion; an empty structure is always representable.
    pub fn to_ttlv_payload(&self) -> Result<Structure, ProtocolError> {
        Ok(Structure::new())
    }
}

/// The typed result of one client-to-server Ping response.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PingResponse {
    result: KmipOperationResult,
}

impl PingResponse {
    /// Converts one validated KMIP Ping response item.
    ///
    /// Successful Ping responses require the empty Response Payload Structure
    /// from §6.1.36, Table 272. Failure responses retain the common result and
    /// do not carry a success payload.
    ///
    /// # Errors
    ///
    /// Returns [`PingError`] when operation, result metadata, or payload shape
    /// is invalid.
    pub fn try_from_response_item(item: ResponseBatchItemView<'_>) -> Result<Self, PingError> {
        if item.operation() != Some(PING_OPERATION) {
            return Err(PingError::UnexpectedOperation);
        }
        let status = item.result_status().ok_or(PingError::MissingResultStatus)?;
        if status.raw() == PENDING {
            return Err(PingError::PendingNotSupported);
        }
        let result_message = item.with_result_message(|text| ResultMessage::new(text.to_owned()));
        let result = KmipOperationResult::new(status, item.result_reason(), result_message)
            .map_err(PingError::InvalidOperationResult)?;

        if status.raw() == SUCCESS {
            let member_count = item
                .with_response_payload(|payload| payload.children().len())
                .ok_or(PingError::MissingResponsePayload)?;
            if member_count != 0 {
                return Err(PingError::MalformedResponsePayload);
            }
        } else if item.with_response_payload(|_| ()).is_some() {
            return Err(PingError::UnexpectedResponsePayload);
        }

        Ok(Self { result })
    }

    /// Returns the complete KMIP result, including raw status and optional
    /// reason and redacted Result Message.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }
}

/// A payload-free Ping response conversion error.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PingError {
    /// The response item identifies an operation other than Ping.
    UnexpectedOperation,
    /// The validated response item is missing Result Status.
    MissingResultStatus,
    /// Ping returned Operation Pending, which has no asynchronous contract.
    PendingNotSupported,
    /// The common result metadata is inconsistent.
    InvalidOperationResult(ResultValidationError),
    /// A successful Ping response omitted its empty Response Payload Structure.
    MissingResponsePayload,
    /// A successful Ping response payload contains one or more fields.
    MalformedResponsePayload,
    /// A failure response unexpectedly carries a Response Payload.
    UnexpectedResponsePayload,
}

impl fmt::Display for PingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::UnexpectedOperation => "response item is not Ping",
            Self::MissingResultStatus => "Ping result status is missing",
            Self::PendingNotSupported => "Ping does not support an Operation Pending result",
            Self::InvalidOperationResult(error) => {
                return write!(formatter, "invalid Ping operation result: {error}");
            }
            Self::MissingResponsePayload => "successful Ping response payload is missing",
            Self::MalformedResponsePayload => "successful Ping response payload is not empty",
            Self::UnexpectedResponsePayload => "failed Ping response has a payload",
        };
        formatter.write_str(message)
    }
}

impl Error for PingError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidOperationResult(error) => Some(error),
            Self::UnexpectedOperation
            | Self::MissingResultStatus
            | Self::PendingNotSupported
            | Self::MissingResponsePayload
            | Self::MalformedResponsePayload
            | Self::UnexpectedResponsePayload => None,
        }
    }
}
