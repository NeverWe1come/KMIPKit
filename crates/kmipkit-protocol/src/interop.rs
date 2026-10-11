//! Typed KMIP 2.1 Interop request and response payloads.
//!
//! Fields follow OASIS KMIP Specification v2.1 §6.1.26, Tables 241–243.
//! The request fields are defined by sections 7.15 and 7.16.

use std::{error::Error, fmt};

use kmipkit_ttlv::{Item, Structure, StructureView, Value};

use crate::cryptographic_operation;

use crate::{
    KmipOperationResult, ProtocolError, ResponseBatchItemView, ResultMessage,
    ResultValidationError, SecretText,
};

const OPERATION: u32 = 0x0000_0034;

const SUCCESS: u32 = 0;
const INTEROP_FUNCTION: u32 = 0x0042_0160;
const INTEROP_IDENTIFIER: u32 = 0x0042_0161;
/// Required Interop fields from §6.1.26, Table 241.
///
/// The production-server restriction applies to servers. This client does
/// not predict whether a server offers Interop.
pub struct InteropRequest {
    interop_function: u32,
    interop_identifier: SecretText,
    extensions: Vec<Item>,
}

impl InteropRequest {
    /// Creates a request with an open Enumeration and the required identifier.
    #[must_use]
    pub const fn new(interop_function: u32, interop_identifier: SecretText) -> Self {
        Self {
            interop_function,
            interop_identifier,
            extensions: Vec::new(),
        }
    }
    /// Returns the exact caller-selected Interop Function Enumeration.
    #[must_use]
    pub const fn interop_function(&self) -> u32 {
        self.interop_function
    }
    /// Lends the required Interop Identifier without cloning it.
    pub fn with_interop_identifier<R>(&self, callback: impl for<'a> FnOnce(&'a str) -> R) -> R {
        self.interop_identifier.with_str(callback)
    }
    /// Appends an unknown operation-payload extension in caller-selected order.
    ///
    /// Table-defined fields must use the typed builders; using an extension
    /// to override them fails locally while all other checked tags remain open.
    #[must_use]
    pub fn with_extension(mut self, extension: Item) -> Self {
        self.extensions.push(extension);
        self
    }

    /// Moves the required Table 241 fields into their ordered payload.
    ///
    /// # Errors
    ///
    /// Returns a sanitized error if the TTLV model cannot represent the fields.
    pub fn into_ttlv_payload(self) -> Result<Structure, ProtocolError> {
        let mut payload = Structure::new();

        cryptographic_operation::push(
            &mut payload,
            INTEROP_FUNCTION,
            Value::enumeration(self.interop_function),
        )?;

        cryptographic_operation::push(
            &mut payload,
            INTEROP_IDENTIFIER,
            self.interop_identifier.into_ttlv_value(),
        )?;

        crate::diagnostics::append_extensions(
            &mut payload,
            self.extensions,
            &[INTEROP_FUNCTION, INTEROP_IDENTIFIER],
        )?;
        Ok(payload)
    }
}

impl fmt::Debug for InteropRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("InteropRequest")
            .field("interop_function", &self.interop_function)
            .field("interop_identifier", &self.interop_identifier)
            .field("extensions", &"[REDACTED]")
            .finish()
    }
}
/// A typed Interop result from §6.1.26, Tables 242–243.
pub struct InteropResponse {
    result: KmipOperationResult,
    response_ttlv: cryptographic_operation::OwnedResponseTtlv,
}

impl InteropResponse {
    /// Converts one validated Interop response item.
    ///
    /// Success requires the empty Response Payload Structure from Table 242.
    /// Failure and Pending preserve the common result metadata.
    ///
    /// # Errors
    ///
    /// Returns [`InteropError`] for another operation, inconsistent result
    /// metadata, or a missing/non-empty success payload.
    pub fn try_from_response_item(item: ResponseBatchItemView<'_>) -> Result<Self, InteropError> {
        if item.operation() != Some(OPERATION) {
            return Err(InteropError::UnexpectedOperation);
        }

        let result = interop_operation_result(item)?;

        if result.status().raw() == SUCCESS {
            let member_count = item
                .with_response_payload(|payload| payload.children().len())
                .ok_or(InteropError::MissingResponsePayload)?;

            if member_count != 0 {
                return Err(InteropError::MalformedResponsePayload);
            }
        }

        let response_ttlv = cryptographic_operation::clone_response_item(item, "Interop")
            .map_err(|_| InteropError::MalformedResponseItem)?;

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

fn interop_operation_result(
    item: ResponseBatchItemView<'_>,
) -> Result<KmipOperationResult, InteropError> {
    let status = item
        .result_status()
        .ok_or(InteropError::MissingResultStatus)?;

    let result_message = item.with_result_message(|text| ResultMessage::new(text.to_owned()));

    KmipOperationResult::new(status, item.result_reason(), result_message)
        .map_err(InteropError::InvalidOperationResult)
}
/// A payload-free error converting an Interop response batch item.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InteropError {
    /// The response item identifies an operation other than Interop.
    UnexpectedOperation,
    /// The validated response item is missing Result Status.
    MissingResultStatus,
    /// The common result metadata is inconsistent.
    InvalidOperationResult(ResultValidationError),
    /// A successful Interop response omitted its empty Response Payload.
    MissingResponsePayload,
    /// A successful Interop response contains a non-empty Response Payload.
    MalformedResponsePayload,
    /// The generic response item cannot be retained.
    MalformedResponseItem,
}

impl fmt::Display for InteropError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedOperation => formatter.write_str("response item is not Interop"),

            Self::MissingResultStatus => formatter.write_str("Interop result status is missing"),

            Self::InvalidOperationResult(error) => {
                write!(formatter, "Interop operation result is invalid: {error}")
            }

            Self::MissingResponsePayload => {
                formatter.write_str("successful Interop response payload is missing")
            }

            Self::MalformedResponseItem => {
                formatter.write_str("Interop response item is malformed")
            }

            Self::MalformedResponsePayload => {
                formatter.write_str("successful Interop response payload is not empty")
            }
        }
    }
}

impl Error for InteropError {
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

impl fmt::Debug for InteropResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("InteropResponse")
            .field("result", &self.result)
            .field("response_ttlv", &"[REDACTED]")
            .finish()
    }
}
