//! Typed KMIP v2.1 Obtain Lease request and response payloads.
//!
//! OASIS KMIP Specification v2.1 §6.1.35, Tables 268–270 defines Obtain
//! Lease. The returned values use §4.29 Date-Time and §4.30 Interval.

use std::error::Error;
use std::fmt;

use kmipkit_ttlv::{Structure, StructureView, ValueView};

use crate::cryptographic_operation::{self as common, OwnedResponseTtlv};
use crate::{
    KmipOperationResult, ProtocolError, ResponseBatchItemView, ResultMessage,
    ResultValidationError, UniqueIdentifier,
};

const OPERATION: u32 = 0x0000_0010;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const LEASE_TIME: u32 = 0x0042_0049;
const LAST_CHANGE_DATE: u32 = 0x0042_0048;
const SUCCESS: u32 = 0;

/// A typed KMIP v2.1 Obtain Lease request from §6.1.35, Table 268.
///
/// When the optional Unique Identifier is omitted in a batch, the server uses
/// the ID Placeholder value as the identifier.
pub struct ObtainLeaseRequest {
    unique_identifier: Option<UniqueIdentifier>,
}

impl ObtainLeaseRequest {
    /// Creates an Obtain Lease request with its optional identifier omitted.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            unique_identifier: None,
        }
    }

    /// Selects the object by its explicit Unique Identifier.
    #[must_use]
    pub fn with_unique_identifier(mut self, value: UniqueIdentifier) -> Self {
        self.unique_identifier = Some(value);
        self
    }

    /// Returns the optional explicit Unique Identifier.
    #[must_use]
    pub const fn unique_identifier(&self) -> Option<&UniqueIdentifier> {
        self.unique_identifier.as_ref()
    }

    /// Builds the Request Payload Structure from §6.1.35, Table 268.
    ///
    /// # Errors
    ///
    /// Returns a sanitized protocol error if the generic TTLV model cannot
    /// represent the payload.
    pub fn into_ttlv_payload(self) -> Result<Structure, ProtocolError> {
        let mut payload = Structure::new();
        if let Some(identifier) = self.unique_identifier {
            common::push(
                &mut payload,
                UNIQUE_IDENTIFIER,
                common::unique_identifier_value(identifier),
            )?;
        }
        Ok(payload)
    }
}

impl Default for ObtainLeaseRequest {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for ObtainLeaseRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ObtainLeaseRequest")
            .field("has_unique_identifier", &self.unique_identifier.is_some())
            .finish()
    }
}

/// A typed Obtain Lease response from §6.1.35, Table 269.
///
/// Successful responses expose the object identifier, exact lease interval,
/// and last change time. No local timer, cache, or lease state is maintained.
#[derive(Debug)]
pub struct ObtainLeaseResponse {
    result: KmipOperationResult,
    response_ttlv: OwnedResponseTtlv,
    unique_identifier: Option<UniqueIdentifier>,
    lease_time: Option<u32>,
    last_change_date: Option<i64>,
}

impl ObtainLeaseResponse {
    /// Converts one already validated Obtain Lease response batch item.
    ///
    /// Successful responses require exactly one well-formed instance of each
    /// Table 269 field. Non-success results keep their common result; unknown
    /// fields remain available through the generic response item.
    ///
    /// # Errors
    ///
    /// Returns [`ObtainLeaseError`] for another operation, malformed result
    /// metadata, or a malformed successful payload.
    pub fn try_from_response_item(
        item: ResponseBatchItemView<'_>,
    ) -> Result<Self, ObtainLeaseError> {
        if item.operation() != Some(OPERATION) {
            return Err(ObtainLeaseError::UnexpectedOperation);
        }
        let status = item
            .result_status()
            .ok_or(ObtainLeaseError::MissingResultStatus)?;
        let result_message = item.with_result_message(|text| ResultMessage::new(text.to_owned()));
        let result = KmipOperationResult::new(status, item.result_reason(), result_message)
            .map_err(ObtainLeaseError::InvalidOperationResult)?;
        let response_ttlv = common::clone_response_item(item, "Obtain Lease")
            .map_err(|_| ObtainLeaseError::MalformedResponseItem)?;

        let (unique_identifier, lease_time, last_change_date) = if status.raw() == SUCCESS {
            let parsed_payload = item
                .with_response_payload(|payload| parse_success_payload(&payload))
                .ok_or(ObtainLeaseError::MalformedSuccessPayload)??;
            (
                Some(parsed_payload.unique_identifier),
                Some(parsed_payload.lease_time),
                Some(parsed_payload.last_change_date),
            )
        } else {
            (None, None, None)
        };

        Ok(Self {
            result,
            response_ttlv,
            unique_identifier,
            lease_time,
            last_change_date,
        })
    }

    /// Returns the exact KMIP operation result.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }

    /// Returns the Unique Identifier included on a successful response.
    #[must_use]
    pub const fn unique_identifier(&self) -> Option<&UniqueIdentifier> {
        self.unique_identifier.as_ref()
    }

    /// Returns the exact Lease Time interval in seconds, including zero.
    #[must_use]
    pub const fn lease_time(&self) -> Option<u32> {
        self.lease_time
    }

    /// Returns the Last Change Date as KMIP Date-Time seconds since the epoch.
    #[must_use]
    pub const fn last_change_date(&self) -> Option<i64> {
        self.last_change_date
    }

    /// Lends the complete ordered generic response batch item.
    pub fn with_ttlv<R>(&self, callback: impl for<'a> FnOnce(StructureView<'a>) -> R) -> R {
        self.response_ttlv.with_ttlv(callback)
    }
}

/// A payload-free error converting a validated Obtain Lease response item.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObtainLeaseError {
    /// The response item is not the client-to-server Obtain Lease operation.
    UnexpectedOperation,
    /// The validated response item did not expose a Result Status.
    MissingResultStatus,
    /// The represented KMIP result violates the shared result contract.
    InvalidOperationResult(ResultValidationError),
    /// The generic response item could not be retained.
    MalformedResponseItem,
    /// A successful response omitted, duplicated, or mistyped a required field.
    MalformedSuccessPayload,
}

impl fmt::Display for ObtainLeaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedOperation => formatter.write_str("response item is not Obtain Lease"),
            Self::MissingResultStatus => {
                formatter.write_str("Obtain Lease result status is missing")
            }
            Self::InvalidOperationResult(cause) => {
                write!(
                    formatter,
                    "Obtain Lease operation result is invalid: {cause}"
                )
            }
            Self::MalformedResponseItem => {
                formatter.write_str("Obtain Lease response item is malformed")
            }
            Self::MalformedSuccessPayload => {
                formatter.write_str("successful Obtain Lease response payload is malformed")
            }
        }
    }
}

impl Error for ObtainLeaseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidOperationResult(cause) => Some(cause),
            _ => None,
        }
    }
}

struct ParsedSuccessPayload {
    unique_identifier: UniqueIdentifier,
    lease_time: u32,
    last_change_date: i64,
}

fn parse_success_payload(
    payload: &StructureView<'_>,
) -> Result<ParsedSuccessPayload, ObtainLeaseError> {
    let unique_identifier = super::lifecycle::successful_response_identifier(payload)
        .ok_or(ObtainLeaseError::MalformedSuccessPayload)?;
    let mut lease_time = None;
    let mut last_change_date = None;

    for field in payload.children() {
        match field.tag().raw() {
            LEASE_TIME => {
                if lease_time.is_some() {
                    return Err(ObtainLeaseError::MalformedSuccessPayload);
                }
                lease_time = Some(
                    field
                        .with_value(|value| match value {
                            ValueView::Interval(value) => Some(*value),
                            _ => None,
                        })
                        .ok_or(ObtainLeaseError::MalformedSuccessPayload)?,
                );
            }
            LAST_CHANGE_DATE => {
                if last_change_date.is_some() {
                    return Err(ObtainLeaseError::MalformedSuccessPayload);
                }
                last_change_date = Some(
                    field
                        .with_value(|value| match value {
                            ValueView::DateTime(value) => Some(*value),
                            _ => None,
                        })
                        .ok_or(ObtainLeaseError::MalformedSuccessPayload)?,
                );
            }
            _ => {}
        }
    }

    Ok(ParsedSuccessPayload {
        unique_identifier,
        lease_time: lease_time.ok_or(ObtainLeaseError::MalformedSuccessPayload)?,
        last_change_date: last_change_date.ok_or(ObtainLeaseError::MalformedSuccessPayload)?,
    })
}
