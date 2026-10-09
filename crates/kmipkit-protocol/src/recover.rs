//! Typed KMIP v2.1 Recover request and response payloads.

use std::error::Error;
use std::fmt;

use kmipkit_ttlv::Structure;

use crate::{
    KmipOperationResult, ProtocolError, ResponseBatchItemView, ResultMessage,
    ResultValidationError, UniqueIdentifier,
};

const RECOVER_OPERATION: u32 = 0x0000_002A;
const SUCCESS: u32 = 0;

/// A typed KMIP 2.1 Recover request payload from §6.1.42, Table 288.
///
/// The optional Unique Identifier is preserved exactly as supplied. Omission
/// is not replaced with a generated value, and the client does not infer local
/// object state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoverRequest {
    unique_identifier: Option<UniqueIdentifier>,
}

impl RecoverRequest {
    /// Creates a Recover request while preserving the optional identifier.
    #[must_use]
    pub const fn new(unique_identifier: Option<UniqueIdentifier>) -> Self {
        Self { unique_identifier }
    }

    /// Returns the optional Unique Identifier in its original wire form.
    #[must_use]
    pub const fn unique_identifier(&self) -> Option<&UniqueIdentifier> {
        self.unique_identifier.as_ref()
    }

    /// Builds the ordered Request Payload Structure defined by Table 288.
    ///
    /// # Errors
    ///
    /// Returns a sanitized protocol error if the fixed tag or resulting TTLV
    /// item cannot be represented.
    pub fn to_ttlv_payload(&self) -> Result<Structure, ProtocolError> {
        super::lifecycle::request_payload(self.unique_identifier.as_ref())
    }
}

/// A typed Recover result for one validated KMIP response batch item.
///
/// A successful response exposes the required Unique Identifier from
/// §6.1.42, Table 289; §4.58 Tables 145–146 define permitted Unique Identifier
/// encodings, and §11.56 Table 487 assigns tag 0x420094 to Unique Identifier.
/// Non-success results retain the shared KMIP operation result and have no
/// typed success identifier. A later Get remains a separate caller action.
/// The source [`crate::ResponseMessage`] retains the complete generic payload,
/// including unknown or future fields.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoverResponse {
    result: KmipOperationResult,
    unique_identifier: Option<UniqueIdentifier>,
}

impl RecoverResponse {
    /// Converts one already validated KMIP response batch item.
    ///
    /// The item must identify the client-to-server Recover operation (`0x2A`).
    /// Successful payload parsing requires exactly one Unique Identifier in a
    /// form permitted by §4.58 Tables 145–146. Unknown fields remain available
    /// from the original response message.
    ///
    /// # Errors
    ///
    /// Returns [`RecoverError`] for another operation, invalid result
    /// metadata, or a malformed required success payload. Errors do not
    /// include server-provided payload values.
    pub fn try_from_response_item(item: ResponseBatchItemView<'_>) -> Result<Self, RecoverError> {
        if item.operation() != Some(RECOVER_OPERATION) {
            return Err(RecoverError::UnexpectedOperation);
        }

        let status = item
            .result_status()
            .ok_or(RecoverError::MissingResultStatus)?;
        let result_message = item.with_result_message(|text| ResultMessage::new(text.to_owned()));
        let result = KmipOperationResult::new(status, item.result_reason(), result_message)
            .map_err(RecoverError::InvalidOperationResult)?;

        if status.raw() != SUCCESS {
            return Ok(Self {
                result,
                unique_identifier: None,
            });
        }

        let unique_identifier = item
            .with_response_payload(|payload| {
                super::lifecycle::successful_response_identifier(&payload)
                    .ok_or(RecoverError::MalformedSuccessPayload)
            })
            .ok_or(RecoverError::MalformedSuccessPayload)??;
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

/// A payload-free error converting a validated Recover response item.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoverError {
    /// The response item is not the client-to-server Recover operation.
    UnexpectedOperation,
    /// The validated response item did not expose a Result Status.
    MissingResultStatus,
    /// The represented KMIP result violates the shared status/reason contract.
    InvalidOperationResult(ResultValidationError),
    /// A successful response omitted or malformed its required identifier.
    MalformedSuccessPayload,
}

impl fmt::Display for RecoverError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedOperation => formatter.write_str("response item is not Recover"),
            Self::MissingResultStatus => formatter.write_str("Recover result status is missing"),
            Self::InvalidOperationResult(cause) => {
                write!(formatter, "Recover operation result is invalid: {cause}")
            }
            Self::MalformedSuccessPayload => {
                formatter.write_str("successful Recover response payload is malformed")
            }
        }
    }
}

impl Error for RecoverError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidOperationResult(cause) => Some(cause),
            _ => None,
        }
    }
}
