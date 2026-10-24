//! Typed KMIP 2.1 Set Defaults request and response payloads.
//!
//! OASIS KMIP Specification v2.1 §6.1.53, Tables 328–330 defines the
//! operation; §7.11 defines Defaults Information.

use std::{error::Error, fmt};

use kmipkit_ttlv::{Structure, StructureView, Value};

use crate::cryptographic_operation::{self as common, OwnedResponseTtlv};
use crate::{
    KmipOperationResult, ProtocolError, ResponseBatchItemView, ResultMessage, ResultValidationError,
};

const SET_DEFAULTS_OPERATION: u32 = 0x0000_0036;
const DEFAULTS_INFORMATION_TAG: u32 = 0x0042_0152;
const SUCCESS: u32 = 0;

/// A client-to-server Set Defaults request from §6.1.53, Table 328.
pub struct SetDefaultsRequest {
    defaults_information: Option<Structure>,
}

impl SetDefaultsRequest {
    /// Creates a request that omits Defaults Information.
    ///
    /// OASIS Table 328 defines omission as a request to remove all Object
    /// Defaults from the server.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            defaults_information: None,
        }
    }

    /// Supplies Defaults Information, including an explicitly empty Structure.
    #[must_use]
    pub fn with_defaults_information(mut self, information: Structure) -> Self {
        self.defaults_information = Some(information);
        self
    }

    /// Lends the optional Defaults Information structure.
    #[must_use]
    pub fn defaults_information(&self) -> Option<StructureView<'_>> {
        self.defaults_information.as_ref().map(Structure::view)
    }

    /// Builds the Request Payload Structure defined by Table 328.
    ///
    /// # Errors
    ///
    /// Returns a sanitized error if the supplied Defaults Information cannot
    /// be copied into the outgoing TTLV payload.
    pub fn to_ttlv_payload(&self) -> Result<Structure, ProtocolError> {
        let mut payload = Structure::new();
        if let Some(information) = &self.defaults_information {
            let information = information
                .view()
                .try_clone()
                .map_err(|_| common::request_error())?;
            common::push(
                &mut payload,
                DEFAULTS_INFORMATION_TAG,
                Value::structure(information),
            )?;
        }
        Ok(payload)
    }
}

impl Default for SetDefaultsRequest {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for SetDefaultsRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SetDefaultsRequest")
            .field(
                "has_defaults_information",
                &self.defaults_information.is_some(),
            )
            .field("defaults_information", &"[REDACTED]")
            .finish()
    }
}

/// A Set Defaults result from §6.1.53, Tables 329–330.
pub struct SetDefaultsResponse {
    result: KmipOperationResult,
    response_ttlv: Option<OwnedResponseTtlv>,
}

impl SetDefaultsResponse {
    /// Converts one validated Set Defaults response batch item.
    ///
    /// On Success, Table 329 defines an empty operation-specific response
    /// payload. Structurally valid unknown fields are retained for forward and
    /// vendor compatibility; the complete response item is available through
    /// [`Self::with_ttlv`]. Failure and Pending retain only common result
    /// metadata and do not retain response-body TTLV.
    ///
    /// # Errors
    ///
    /// Returns a payload-free error for an unexpected operation, invalid
    /// common result, missing successful Response Payload, or malformed TTLV.
    pub fn try_from_response_item(
        item: ResponseBatchItemView<'_>,
    ) -> Result<Self, SetDefaultsError> {
        if item.operation() != Some(SET_DEFAULTS_OPERATION) {
            return Err(SetDefaultsError::UnexpectedOperation);
        }
        let status = item
            .result_status()
            .ok_or(SetDefaultsError::MissingResultStatus)?;
        let result_message = item.with_result_message(|text| ResultMessage::new(text.to_owned()));
        let result = KmipOperationResult::new(status, item.result_reason(), result_message)
            .map_err(SetDefaultsError::InvalidOperationResult)?;
        let response_ttlv = if status.raw() == SUCCESS {
            if item.with_response_payload(|_| ()).is_none() {
                return Err(SetDefaultsError::MissingSuccessPayload); // COVERAGE-EXCLUDE: KMIP v2.1 §8.6 Table 399 rejects Success without a Response Payload before this mapper.
            }
            Some(
                common::clone_response_item(item, "Set Defaults")
                    .map_err(|_| SetDefaultsError::MalformedResponseItem)?,
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

    /// Lends the complete ordered response item, including unknown fields, on
    /// Success. Failure and Pending expose common result metadata only.
    pub fn with_ttlv<R>(&self, callback: impl for<'a> FnOnce(StructureView<'a>) -> R) -> Option<R> {
        self.response_ttlv
            .as_ref()
            .map(|response_ttlv| response_ttlv.with_ttlv(callback))
    }
}

impl fmt::Debug for SetDefaultsResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SetDefaultsResponse")
            .field("result", &self.result)
            .field("response_ttlv", &"[REDACTED]")
            .finish()
    }
}

/// A payload-free Set Defaults response conversion error.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SetDefaultsError {
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

impl fmt::Display for SetDefaultsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedOperation => formatter.write_str("response item is not Set Defaults"),
            Self::MissingResultStatus => {
                formatter.write_str("Set Defaults result status is missing")
            }
            Self::InvalidOperationResult(error) => {
                write!(formatter, "invalid Set Defaults result: {error}")
            }
            Self::MissingSuccessPayload => {
                formatter.write_str("successful Set Defaults payload is missing")
            }
            Self::MalformedResponseItem => {
                formatter.write_str("Set Defaults response item is malformed")
            }
        }
    }
}

impl Error for SetDefaultsError {
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
