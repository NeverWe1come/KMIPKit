//! Typed KMIP 2.1 Log request and response payloads.
//!
//! Fields follow OASIS KMIP Specification v2.1 §6.1.29, Tables 250–252.
//! Section 7.19 defines Log Message; approved DISC-017 assigns Table 252 to Log.

use std::{error::Error, fmt};

use kmipkit_ttlv::{Structure, StructureView};

use crate::cryptographic_operation;

use crate::{
    KmipOperationResult, ProtocolError, ResponseBatchItemView, ResultMessage,
    ResultValidationError, SecretText,
};

const OPERATION: u32 = 0x0000_002c;

const SUCCESS: u32 = 0;
const LOG_MESSAGE: u32 = 0x0042_0141;
/// Required Log Message from section 6.1.29, Table 250.
///
/// The caller must not submit secrets to a server log. The complete text is
/// nevertheless redacted from every diagnostic and zeroized when dropped.
pub struct LogRequest {
    log_message: SecretText,
}

impl LogRequest {
    /// Creates a request with the caller-selected message.
    #[must_use]
    pub const fn new(log_message: SecretText) -> Self {
        Self { log_message }
    }

    /// Lends the exact message without cloning it.
    pub fn with_log_message<R>(&self, callback: impl for<'a> FnOnce(&'a str) -> R) -> R {
        self.log_message.with_str(callback)
    }

    /// Moves the required Table 250 Text String into its payload.
    ///
    /// # Errors
    ///
    /// Returns a sanitized error if the TTLV model cannot represent the field.
    pub fn into_ttlv_payload(self) -> Result<Structure, ProtocolError> {
        let mut payload = Structure::new();
        cryptographic_operation::push(
            &mut payload,
            LOG_MESSAGE,
            self.log_message.into_ttlv_value(),
        )?;
        Ok(payload)
    }
}

impl fmt::Debug for LogRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LogRequest")
            .field("log_message", &self.log_message)
            .finish()
    }
}

/// A typed Log result from §6.1.29, Tables 251–252.
pub struct LogResponse {
    result: KmipOperationResult,
    response_ttlv: cryptographic_operation::OwnedResponseTtlv,
}

impl LogResponse {
    /// Converts one validated Log response item.
    ///
    /// Success requires the empty Response Payload Structure from Table 251.
    /// Failure and Pending preserve the common result metadata.
    ///
    /// # Errors
    ///
    /// Returns [`LogError`] for another operation, inconsistent result
    /// metadata, or a missing/non-empty success payload.
    pub fn try_from_response_item(item: ResponseBatchItemView<'_>) -> Result<Self, LogError> {
        if item.operation() != Some(OPERATION) {
            return Err(LogError::UnexpectedOperation);
        }

        let result = log_operation_result(item)?;

        if result.status().raw() == SUCCESS {
            let member_count = item
                .with_response_payload(|payload| payload.children().len())
                .ok_or(LogError::MissingResponsePayload)?;

            if member_count != 0 {
                return Err(LogError::MalformedResponsePayload);
            }
        }

        let response_ttlv = cryptographic_operation::clone_response_item(item, "Log")
            .map_err(|_| LogError::MalformedResponseItem)?;

        Ok(Self {
            result,
            response_ttlv,
        })
    }
    /// Returns the complete KMIP result, including status and optional reason.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }
    /// Lends the complete generic response, preserving unknown fields and order.
    pub fn with_ttlv<R>(&self, callback: impl for<'a> FnOnce(StructureView<'a>) -> R) -> R {
        self.response_ttlv.with_ttlv(callback)
    }
}

fn log_operation_result(item: ResponseBatchItemView<'_>) -> Result<KmipOperationResult, LogError> {
    let status = item.result_status().ok_or(LogError::MissingResultStatus)?;

    let result_message = item.with_result_message(|text| ResultMessage::new(text.to_owned()));

    KmipOperationResult::new(status, item.result_reason(), result_message)
        .map_err(LogError::InvalidOperationResult)
}
/// A payload-free error converting an Log response batch item.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LogError {
    /// The response item identifies an operation other than Log.
    UnexpectedOperation,
    /// The validated response item is missing Result Status.
    MissingResultStatus,
    /// The common result metadata is inconsistent.
    InvalidOperationResult(ResultValidationError),
    /// A successful Log response omitted its empty Response Payload.
    MissingResponsePayload,
    /// A successful Log response contains a non-empty Response Payload.
    MalformedResponsePayload,
    /// The generic response item cannot be retained.
    MalformedResponseItem,
}

impl fmt::Display for LogError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedOperation => formatter.write_str("response item is not Log"),

            Self::MissingResultStatus => formatter.write_str("Log result status is missing"),

            Self::InvalidOperationResult(error) => {
                write!(formatter, "Log operation result is invalid: {error}")
            }

            Self::MissingResponsePayload => {
                formatter.write_str("successful Log response payload is missing")
            }

            Self::MalformedResponseItem => formatter.write_str("Log response item is malformed"),

            Self::MalformedResponsePayload => {
                formatter.write_str("successful Log response payload is not empty")
            }
        }
    }
}

impl Error for LogError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidOperationResult(error) => Some(error),

            Self::UnexpectedOperation
            | Self::MissingResultStatus
            | Self::MissingResponsePayload
            | Self::MalformedResponsePayload
            | Self::MalformedResponseItem => None,
        }
    }
}

impl fmt::Debug for LogResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LogResponse")
            .field("result", &self.result)
            .field("response_ttlv", &"[REDACTED]")
            .finish()
    }
}
