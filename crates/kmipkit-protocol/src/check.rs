//! Typed KMIP v2.1 Check request and response payloads.
//!
//! OASIS KMIP Specification v2.1 §6.1.7, Tables 183–185 defines Check.
//! Value encodings follow §§4.17, 4.30, and 4.59.

use std::error::Error;
use std::fmt;

use kmipkit_ttlv::{Structure, StructureView, Value, ValueView};

use crate::cryptographic_operation::{self as common, OwnedResponseTtlv};
use crate::{
    KmipOperationResult, ProtocolError, ResponseBatchItemView, ResultMessage,
    ResultValidationError, UniqueIdentifier,
};

const OPERATION: u32 = 0x0000_0009;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const CRYPTOGRAPHIC_USAGE_MASK: u32 = 0x0042_002C;
const LEASE_TIME: u32 = 0x0042_0049;
const USAGE_LIMITS_COUNT: u32 = 0x0042_0096;
const SUCCESS: u32 = 0;

/// A typed KMIP v2.1 Check request payload from §6.1.7, Table 183.
///
/// All fields are optional. When Unique Identifier is omitted, an eligible
/// preceding request in an ordered batch supplies the server's ID Placeholder.
pub struct CheckRequest {
    unique_identifier: Option<UniqueIdentifier>,
    usage_limits_count: Option<i64>,
    cryptographic_usage_mask: Option<i32>,
    lease_time: Option<u32>,
}

impl CheckRequest {
    /// Creates a Check request with all optional fields omitted.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            unique_identifier: None,
            usage_limits_count: None,
            cryptographic_usage_mask: None,
            lease_time: None,
        }
    }

    /// Selects the object to check by its explicit Unique Identifier.
    #[must_use]
    pub fn with_unique_identifier(mut self, value: UniqueIdentifier) -> Self {
        self.unique_identifier = Some(value);
        self
    }

    /// Supplies the requested number of Usage Limits Units as a KMIP Long Integer.
    #[must_use]
    pub const fn with_usage_limits_count(mut self, value: i64) -> Self {
        self.usage_limits_count = Some(value);
        self
    }

    /// Supplies the raw Cryptographic Usage Mask Integer, including unknown bits.
    #[must_use]
    pub const fn with_cryptographic_usage_mask(mut self, value: i32) -> Self {
        self.cryptographic_usage_mask = Some(value);
        self
    }

    /// Supplies the requested Lease Time as a KMIP Interval.
    #[must_use]
    pub const fn with_lease_time(mut self, value: u32) -> Self {
        self.lease_time = Some(value);
        self
    }

    /// Returns the optional explicit Unique Identifier.
    #[must_use]
    pub const fn unique_identifier(&self) -> Option<&UniqueIdentifier> {
        self.unique_identifier.as_ref()
    }

    /// Builds the ordered Request Payload Structure from Table 183.
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
        if let Some(count) = self.usage_limits_count {
            common::push(&mut payload, USAGE_LIMITS_COUNT, Value::long_integer(count))?;
        }
        if let Some(mask) = self.cryptographic_usage_mask {
            common::push(&mut payload, CRYPTOGRAPHIC_USAGE_MASK, Value::integer(mask))?;
        }
        if let Some(lease_time) = self.lease_time {
            common::push(&mut payload, LEASE_TIME, Value::interval(lease_time))?;
        }
        Ok(payload)
    }
}

impl Default for CheckRequest {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for CheckRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CheckRequest")
            .field("has_unique_identifier", &self.unique_identifier.is_some())
            .field("usage_limits_count", &self.usage_limits_count)
            .field("cryptographic_usage_mask", &self.cryptographic_usage_mask)
            .field("lease_time", &self.lease_time)
            .finish()
    }
}

/// A typed Check response from §6.1.7, Table 184.
///
/// The complete generic response item is retained for callback-scoped
/// inspection, including unknown or future fields. A successful result
/// requires one Unique Identifier; policy values are optional.
#[derive(Debug)]
pub struct CheckResponse {
    result: KmipOperationResult,
    response_ttlv: OwnedResponseTtlv,
    unique_identifier: Option<UniqueIdentifier>,
    usage_limits_count: Option<i64>,
    cryptographic_usage_mask: Option<i32>,
    lease_time: Option<u32>,
}

impl CheckResponse {
    /// Converts one already validated Check response batch item.
    ///
    /// Known success and policy-denial payload fields are typed, while unknown
    /// fields remain available through the generic response item. Malformed or
    /// duplicate known fields are rejected without including server values.
    ///
    /// # Errors
    ///
    /// Returns [`CheckError`] for another operation, malformed result
    /// metadata or payload, or a successful response without its identifier.
    pub fn try_from_response_item(item: ResponseBatchItemView<'_>) -> Result<Self, CheckError> {
        if item.operation() != Some(OPERATION) {
            return Err(CheckError::UnexpectedOperation);
        }
        let status = item
            .result_status()
            .ok_or(CheckError::MissingResultStatus)?;
        let result_message = item.with_result_message(|text| ResultMessage::new(text.to_owned()));
        let result = KmipOperationResult::new(status, item.result_reason(), result_message)
            .map_err(CheckError::InvalidOperationResult)?;
        let response_ttlv = common::clone_response_item(item, "Check")
            .map_err(|_| CheckError::MalformedResponseItem)?;

        let is_success = status.raw() == SUCCESS;
        let parsed_payload =
            item.with_response_payload(|payload| parse_response_payload(&payload, is_success));
        let parsed_payload = match parsed_payload {
            Some(payload) => Some(payload?),
            None if is_success => return Err(CheckError::MalformedResponsePayload),
            None => None,
        };
        let (unique_identifier, usage_limits_count, cryptographic_usage_mask, lease_time) =
            match parsed_payload {
                Some(payload) => (
                    payload.unique_identifier,
                    payload.usage_limits_count,
                    payload.cryptographic_usage_mask,
                    payload.lease_time,
                ),
                None => (None, None, None, None),
            };

        Ok(Self {
            result,
            response_ttlv,
            unique_identifier,
            usage_limits_count,
            cryptographic_usage_mask,
            lease_time,
        })
    }

    /// Returns the exact KMIP operation result.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }

    /// Returns the Unique Identifier when the server confirms allowed use.
    #[must_use]
    pub const fn unique_identifier(&self) -> Option<&UniqueIdentifier> {
        self.unique_identifier.as_ref()
    }

    /// Returns the optional returned Usage Limits Count Long Integer.
    #[must_use]
    pub const fn usage_limits_count(&self) -> Option<i64> {
        self.usage_limits_count
    }

    /// Returns the optional raw Cryptographic Usage Mask Integer.
    #[must_use]
    pub const fn cryptographic_usage_mask(&self) -> Option<i32> {
        self.cryptographic_usage_mask
    }

    /// Returns the optional returned Lease Time Interval.
    #[must_use]
    pub const fn lease_time(&self) -> Option<u32> {
        self.lease_time
    }

    /// Lends the complete ordered generic response batch item.
    pub fn with_ttlv<R>(&self, callback: impl for<'a> FnOnce(StructureView<'a>) -> R) -> R {
        self.response_ttlv.with_ttlv(callback)
    }
}

/// A payload-free error converting a validated Check response item.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckError {
    /// The response item is not the client-to-server Check operation.
    UnexpectedOperation,
    /// The validated response item did not expose a Result Status.
    MissingResultStatus,
    /// The represented KMIP result violates the shared result contract.
    InvalidOperationResult(ResultValidationError),
    /// The generic response item could not be retained.
    MalformedResponseItem,
    /// A Check response payload is malformed or success omitted its identifier.
    MalformedResponsePayload,
}

impl fmt::Display for CheckError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedOperation => formatter.write_str("response item is not Check"),
            Self::MissingResultStatus => formatter.write_str("Check result status is missing"),
            Self::InvalidOperationResult(cause) => {
                write!(formatter, "Check operation result is invalid: {cause}")
            }
            Self::MalformedResponseItem => formatter.write_str("Check response item is malformed"),
            Self::MalformedResponsePayload => {
                formatter.write_str("Check response payload is malformed")
            }
        }
    }
}

impl Error for CheckError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidOperationResult(cause) => Some(cause),
            _ => None,
        }
    }
}

struct ParsedCheckResponsePayload {
    unique_identifier: Option<UniqueIdentifier>,
    usage_limits_count: Option<i64>,
    cryptographic_usage_mask: Option<i32>,
    lease_time: Option<u32>,
}

fn parse_response_payload(
    payload: &StructureView<'_>,
    require_identifier: bool,
) -> Result<ParsedCheckResponsePayload, CheckError> {
    let mut unique_identifier = None;
    let mut usage_limits_count = None;
    let mut cryptographic_usage_mask = None;
    let mut lease_time = None;

    for field in payload.children() {
        match field.tag().raw() {
            UNIQUE_IDENTIFIER => {
                if unique_identifier.is_some() {
                    return Err(CheckError::MalformedResponsePayload);
                }
                let identifier = super::lifecycle::parse_unique_identifier(field)
                    .ok_or(CheckError::MalformedResponsePayload)?;
                unique_identifier = Some(identifier);
            }
            USAGE_LIMITS_COUNT => {
                if usage_limits_count.is_some() {
                    return Err(CheckError::MalformedResponsePayload);
                }
                usage_limits_count = Some(
                    field
                        .with_value(|value| match value {
                            ValueView::LongInteger(value) => Some(*value),
                            _ => None,
                        })
                        .ok_or(CheckError::MalformedResponsePayload)?,
                );
            }
            CRYPTOGRAPHIC_USAGE_MASK => {
                if cryptographic_usage_mask.is_some() {
                    return Err(CheckError::MalformedResponsePayload);
                }
                cryptographic_usage_mask = Some(
                    field
                        .with_value(|value| match value {
                            ValueView::Integer(value) => Some(*value),
                            _ => None,
                        })
                        .ok_or(CheckError::MalformedResponsePayload)?,
                );
            }
            LEASE_TIME => {
                if lease_time.is_some() {
                    return Err(CheckError::MalformedResponsePayload);
                }
                lease_time = Some(
                    field
                        .with_value(|value| match value {
                            ValueView::Interval(value) => Some(*value),
                            _ => None,
                        })
                        .ok_or(CheckError::MalformedResponsePayload)?,
                );
            }
            _ => {}
        }
    }

    if require_identifier && unique_identifier.is_none() {
        return Err(CheckError::MalformedResponsePayload);
    }

    Ok(ParsedCheckResponsePayload {
        unique_identifier,
        usage_limits_count,
        cryptographic_usage_mask,
        lease_time,
    })
}
