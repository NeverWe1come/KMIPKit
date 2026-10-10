//! Typed KMIP v2.1 Activate request and response payloads.

use std::error::Error;
use std::fmt;

use kmipkit_ttlv::Structure;

use crate::{
    KmipOperationResult, ProtocolError, ResponseBatchItemView, ResultMessage,
    ResultValidationError, UniqueIdentifier,
};

const ACTIVATE_OPERATION: u32 = 0x0000_0012;
const SUCCESS: u32 = 0;

/// A typed KMIP 2.1 Activate request payload from §6.1.1, Table 164.
///
/// The optional Unique Identifier is preserved exactly as supplied. Omission
/// is not replaced with a generated value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivateRequest {
    unique_identifier: Option<UniqueIdentifier>,
}

impl ActivateRequest {
    /// Creates an Activate request while preserving the optional identifier.
    #[must_use]
    pub const fn new(unique_identifier: Option<UniqueIdentifier>) -> Self {
        Self { unique_identifier }
    }

    /// Returns the optional Unique Identifier in its original wire form.
    #[must_use]
    pub const fn unique_identifier(&self) -> Option<&UniqueIdentifier> {
        self.unique_identifier.as_ref()
    }

    /// Builds the ordered Request Payload Structure defined by Table 164.
    ///
    /// # Errors
    ///
    /// Returns a sanitized protocol error if the fixed tag or resulting TTLV
    /// item cannot be represented.
    pub fn to_ttlv_payload(&self) -> Result<Structure, ProtocolError> {
        super::lifecycle::request_payload(self.unique_identifier.as_ref())
    }
}

/// A typed Activate result for one validated KMIP response batch item.
///
/// A successful response exposes the required Unique Identifier from
/// §6.1.1, Table 165; §4.58 Tables 145–146 define permitted Unique Identifier
/// encodings, and §11.56 Table 487 assigns tag 0x420094 to Unique Identifier.
/// Non-success results retain the shared KMIP operation result and have no
/// typed success identifier. The source
/// [`crate::ResponseMessage`] retains the complete generic payload, including
/// unknown or future fields.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivateResponse {
    result: KmipOperationResult,
    unique_identifier: Option<UniqueIdentifier>,
}

impl ActivateResponse {
    /// Converts one already validated KMIP response batch item.
    ///
    /// The item must identify the client-to-server Activate operation (`0x12`).
    /// Successful payload parsing requires exactly one Unique Identifier in a
    /// form permitted by §4.58 Tables 145–146. Unknown fields remain available
    /// from the original response message.
    ///
    /// # Errors
    ///
    /// Returns [`ActivateError`] for another operation, invalid result
    /// metadata, or a malformed required success payload. Errors do not
    /// include server-provided payload values.
    pub fn try_from_response_item(item: ResponseBatchItemView<'_>) -> Result<Self, ActivateError> {
        if item.operation() != Some(ACTIVATE_OPERATION) {
            return Err(ActivateError::UnexpectedOperation);
        }

        let status = item
            .result_status()
            .ok_or(ActivateError::MissingResultStatus)?;
        let result_message = item.with_result_message(|text| ResultMessage::new(text.to_owned()));
        let result = KmipOperationResult::new(status, item.result_reason(), result_message)
            .map_err(ActivateError::InvalidOperationResult)?;

        if status.raw() != SUCCESS {
            return Ok(Self {
                result,
                unique_identifier: None,
            });
        }

        let unique_identifier = item
            .with_response_payload(|payload| {
                super::lifecycle::successful_response_identifier(&payload)
                    .ok_or(ActivateError::MalformedSuccessPayload)
            })
            .ok_or(ActivateError::MalformedSuccessPayload)??;
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

/// A payload-free error converting a validated Activate response item.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivateError {
    /// The response item is not the client-to-server Activate operation.
    UnexpectedOperation,
    /// The validated response item did not expose a Result Status.
    MissingResultStatus,
    /// The represented KMIP result violates the shared status/reason contract.
    InvalidOperationResult(ResultValidationError),
    /// A successful response omitted or malformed its required identifier.
    MalformedSuccessPayload,
}

impl fmt::Display for ActivateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedOperation => formatter.write_str("response item is not Activate"),
            Self::MissingResultStatus => formatter.write_str("Activate result status is missing"),
            Self::InvalidOperationResult(cause) => {
                write!(formatter, "Activate operation result is invalid: {cause}")
            }
            Self::MalformedSuccessPayload => {
                formatter.write_str("successful Activate response payload is malformed")
            }
        }
    }
}

impl Error for ActivateError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidOperationResult(cause) => Some(cause),
            _ => None,
        }
    }
}
