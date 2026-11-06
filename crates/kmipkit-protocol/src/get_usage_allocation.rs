//! Typed KMIP 2.1 Get Usage Allocation request and response payloads.
//!
//! OASIS KMIP Specification v2.1 §6.1.23, Tables 232–234 defines the
//! operation; §7.40, Table 392 defines Usage Limits Count as a Long Integer.

use std::{error::Error, fmt};

use kmipkit_ttlv::{Structure, StructureView, Value};

use crate::cryptographic_operation::{self as common, OwnedResponseTtlv};
use crate::{
    KmipOperationResult, ProtocolError, ResponseBatchItemView, ResultMessage,
    ResultValidationError, UniqueIdentifier,
};

const GET_USAGE_ALLOCATION_OPERATION: u32 = 0x0000_0011;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const USAGE_LIMITS_COUNT: u32 = 0x0042_0096;
const SUCCESS: u32 = 0;

/// A typed Get Usage Allocation request payload from §6.1.23, Table 232.
pub struct GetUsageAllocationRequest {
    unique_identifier: Option<UniqueIdentifier>,
    usage_limits_count: i64,
}

impl GetUsageAllocationRequest {
    /// Creates a request for the supplied number of Usage Limits Units.
    ///
    /// The caller may omit the Unique Identifier when using the KMIP ID
    /// Placeholder established by the surrounding batch context.
    #[must_use]
    pub const fn new(usage_limits_count: i64) -> Self {
        Self {
            unique_identifier: None,
            usage_limits_count,
        }
    }

    /// Selects the object whose protection usage should be allocated.
    #[must_use]
    pub fn with_unique_identifier(mut self, unique_identifier: UniqueIdentifier) -> Self {
        self.unique_identifier = Some(unique_identifier);
        self
    }

    /// Returns the optional explicit object identifier.
    #[must_use]
    pub const fn unique_identifier(&self) -> Option<&UniqueIdentifier> {
        self.unique_identifier.as_ref()
    }

    /// Returns the exact caller-requested Usage Limits Count.
    #[must_use]
    pub const fn usage_limits_count(&self) -> i64 {
        self.usage_limits_count
    }

    /// Builds the ordered Request Payload Structure from Table 232.
    ///
    /// # Errors
    ///
    /// Returns a sanitized protocol error if the TTLV model cannot represent
    /// the request payload.
    pub fn to_ttlv_payload(&self) -> Result<Structure, ProtocolError> {
        let mut payload = Structure::new();
        if let Some(identifier) = &self.unique_identifier {
            common::push(
                &mut payload,
                UNIQUE_IDENTIFIER,
                common::unique_identifier_value(identifier.clone()),
            )?;
        }
        common::push(
            &mut payload,
            USAGE_LIMITS_COUNT,
            Value::long_integer(self.usage_limits_count),
        )?;
        Ok(payload)
    }
}

/// A typed Get Usage Allocation result from §6.1.23, Tables 233–234.
pub struct GetUsageAllocationResponse {
    result: KmipOperationResult,
    unique_identifier: Option<UniqueIdentifier>,
    response_ttlv: Option<OwnedResponseTtlv>,
}

impl GetUsageAllocationResponse {
    /// Converts one validated Get Usage Allocation response batch item.
    ///
    /// Successful responses require exactly one valid Unique Identifier;
    /// unrelated fields remain available in the retained generic response.
    ///
    /// # Errors
    ///
    /// Returns a sanitized error for an unexpected operation, invalid common
    /// result, or a malformed successful payload.
    pub fn try_from_response_item(
        item: ResponseBatchItemView<'_>,
    ) -> Result<Self, GetUsageAllocationError> {
        if item.operation() != Some(GET_USAGE_ALLOCATION_OPERATION) {
            return Err(GetUsageAllocationError::UnexpectedOperation);
        }
        let status = item
            .result_status()
            .ok_or(GetUsageAllocationError::MissingResultStatus)?;
        let result_message = item.with_result_message(|text| ResultMessage::new(text.to_owned()));
        let result = KmipOperationResult::new(status, item.result_reason(), result_message)
            .map_err(GetUsageAllocationError::InvalidOperationResult)?;
        let (unique_identifier, response_ttlv) = if status.raw() == SUCCESS {
            let response_identifier = item
                .with_response_payload(|payload| {
                    crate::lifecycle::successful_response_identifier(&payload)
                })
                .ok_or(GetUsageAllocationError::MissingSuccessPayload)?;
            let unique_identifier =
                response_identifier.ok_or(GetUsageAllocationError::MalformedSuccessPayload)?;
            let response_ttlv = common::clone_response_item(item, "Get Usage Allocation")
                .map_err(|_| GetUsageAllocationError::MalformedResponseItem)?;
            (Some(unique_identifier), Some(response_ttlv))
        } else {
            (None, None)
        };

        Ok(Self {
            result,
            unique_identifier,
            response_ttlv,
        })
    }

    /// Returns the complete KMIP status, reason, and optional Result Message.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }

    /// Returns the server-resolved Unique Identifier on success.
    #[must_use]
    pub const fn unique_identifier(&self) -> Option<&UniqueIdentifier> {
        self.unique_identifier.as_ref()
    }

    /// Lends the complete ordered response item on success, including unknown fields.
    ///
    /// Pending and Failure results do not retain or expose generic response TTLV.
    pub fn with_ttlv<R>(&self, callback: impl for<'a> FnOnce(StructureView<'a>) -> R) -> Option<R> {
        self.response_ttlv
            .as_ref()
            .map(|response_ttlv| response_ttlv.with_ttlv(callback))
    }
}

impl fmt::Debug for GetUsageAllocationRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("GetUsageAllocationRequest")
            .field("unique_identifier", &self.unique_identifier)
            .field("usage_limits_count", &self.usage_limits_count)
            .finish()
    }
}

impl fmt::Debug for GetUsageAllocationResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("GetUsageAllocationResponse")
            .field("result", &self.result)
            .field("has_unique_identifier", &self.unique_identifier.is_some())
            .field("response_ttlv", &"[REDACTED]")
            .finish()
    }
}

/// A payload-free Get Usage Allocation response conversion error.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GetUsageAllocationError {
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
    /// The successful payload lacks exactly one valid Unique Identifier.
    MalformedSuccessPayload,
}

impl fmt::Display for GetUsageAllocationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedOperation => {
                formatter.write_str("response item is not Get Usage Allocation")
            }
            Self::MissingResultStatus => {
                formatter.write_str("Get Usage Allocation result status is missing")
            }
            Self::InvalidOperationResult(error) => {
                write!(formatter, "invalid Get Usage Allocation result: {error}")
            }
            Self::MissingSuccessPayload => {
                formatter.write_str("successful Get Usage Allocation payload is missing")
            }
            Self::MalformedResponseItem => {
                formatter.write_str("Get Usage Allocation response item is malformed")
            }
            Self::MalformedSuccessPayload => {
                formatter.write_str("successful Get Usage Allocation payload is malformed")
            }
        }
    }
}

impl Error for GetUsageAllocationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidOperationResult(error) => Some(error),
            Self::UnexpectedOperation
            | Self::MissingResultStatus
            | Self::MissingSuccessPayload
            | Self::MalformedResponseItem
            | Self::MalformedSuccessPayload => None,
        }
    }
}
