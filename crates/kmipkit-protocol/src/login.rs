//! Typed KMIP 2.1 Login operation models.
//!
//! Request and response fields follow OASIS KMIP Specification v2.1 §6.1.30,
//! Tables 253–255. Returned Ticket values follow §7.39, Table 391.

use std::{error::Error, fmt};

use kmipkit_ttlv::{Structure, Value, ValueView};

use crate::cryptographic_operation;
use crate::{
    KmipOperationResult, ProtocolError, ResponseBatchItemView, ResultMessage,
    ResultValidationError, Ticket,
};

const OPERATION: u32 = 0x0000_002D;
const LEASE_TIME: u32 = 0x0042_0049;
const REQUEST_COUNT: u32 = 0x0042_014C;
const USAGE_LIMITS: u32 = 0x0042_0095;
const TICKET: u32 = 0x0042_0149;
const SUCCESS: u32 = 0;

enum LeaseTime {
    Interval(u32),
    DateTime(i64),
}

/// A typed client-to-server Login request from §6.1.30, Table 253.
///
/// Lease Time, Request Count, and Usage Limits are optional. Constructing an
/// empty request preserves their absence; no local defaults are synthesized.
pub struct LoginRequest {
    lease_time: Option<LeaseTime>,
    request_count: Option<i32>,
    usage_limits: Option<Structure>,
}

impl LoginRequest {
    /// Creates an empty Login request with all optional fields absent.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            lease_time: None,
            request_count: None,
            usage_limits: None,
        }
    }

    /// Supplies Lease Time using the Interval form from §6.1.30, Table 253.
    #[must_use]
    pub fn with_lease_time_interval(mut self, seconds: u32) -> Self {
        self.lease_time = Some(LeaseTime::Interval(seconds));
        self
    }

    /// Supplies Lease Time using the Date Time form from §6.1.30, Table 253.
    #[must_use]
    pub fn with_lease_time_date_time(mut self, unix_seconds: i64) -> Self {
        self.lease_time = Some(LeaseTime::DateTime(unix_seconds));
        self
    }

    /// Supplies the optional Request Count Integer.
    #[must_use]
    pub const fn with_request_count(mut self, count: i32) -> Self {
        self.request_count = Some(count);
        self
    }

    /// Supplies the optional generic Usage Limits structure.
    #[must_use]
    pub fn with_usage_limits(mut self, usage_limits: Structure) -> Self {
        self.usage_limits = Some(usage_limits);
        self
    }

    /// Encodes the Table 253 Request Payload in protocol field order.
    ///
    /// # Errors
    ///
    /// Returns a sanitized protocol error if a supplied nested structure
    /// exceeds the generic TTLV model's representable limits.
    pub fn into_ttlv_payload(self) -> Result<Structure, ProtocolError> {
        let mut payload = Structure::new();
        if let Some(lease_time) = self.lease_time {
            let value = match lease_time {
                LeaseTime::Interval(seconds) => Value::interval(seconds),
                LeaseTime::DateTime(unix_seconds) => Value::date_time(unix_seconds),
            };
            cryptographic_operation::push(&mut payload, LEASE_TIME, value)?;
        }
        if let Some(count) = self.request_count {
            cryptographic_operation::push(&mut payload, REQUEST_COUNT, Value::integer(count))?;
        }
        if let Some(usage_limits) = self.usage_limits {
            cryptographic_operation::push(
                &mut payload,
                USAGE_LIMITS,
                Value::structure(usage_limits),
            )?;
        }
        Ok(payload)
    }
}

impl Default for LoginRequest {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for LoginRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LoginRequest")
            .field("lease_time", &self.lease_time.is_some())
            .field("request_count", &self.request_count)
            .field("usage_limits", &self.usage_limits.is_some())
            .finish()
    }
}

/// A typed Login response from §6.1.30, Table 254, or a preserved failure.
#[derive(Debug)]
pub struct LoginResponse {
    result: KmipOperationResult,
    ticket: Option<Ticket>,
}

impl LoginResponse {
    /// Converts one validated Login response batch item.
    ///
    /// A successful response requires exactly one valid Ticket. Failure and
    /// Pending responses retain their shared result and expose no Ticket.
    /// Unknown values inside the Ticket remain preserved by its typed model.
    ///
    /// # Errors
    ///
    /// Returns [`LoginError`] for another operation, invalid result metadata,
    /// or a malformed successful Ticket payload. Diagnostics never contain
    /// Ticket Value bytes.
    pub fn try_from_response_item(item: ResponseBatchItemView<'_>) -> Result<Self, LoginError> {
        if item.operation() != Some(OPERATION) {
            return Err(LoginError::UnexpectedOperation);
        }
        let status = item
            .result_status()
            .ok_or(LoginError::MissingResultStatus)?;
        let result_message = item.with_result_message(|text| ResultMessage::new(text.to_owned()));
        let result = KmipOperationResult::new(status, item.result_reason(), result_message)
            .map_err(LoginError::InvalidOperationResult)?;
        let ticket = if status.raw() == SUCCESS {
            let payload = item
                .with_response_payload(|payload| parse_ticket(&payload))
                .ok_or(LoginError::MalformedSuccessPayload)??;
            Some(payload)
        } else {
            None
        };
        Ok(Self { result, ticket })
    }

    /// Returns the exact KMIP operation result.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }

    /// Returns the required Ticket for success, if this result is successful.
    #[must_use]
    pub const fn ticket(&self) -> Option<&Ticket> {
        self.ticket.as_ref()
    }
}

fn parse_ticket(payload: &kmipkit_ttlv::StructureView<'_>) -> Result<Ticket, LoginError> {
    let mut ticket = None;
    for field in payload.children() {
        if field.tag().raw() != TICKET {
            continue;
        }
        if ticket.is_some() {
            return Err(LoginError::MalformedSuccessPayload);
        }
        let tree = field.with_value(|value| match value {
            ValueView::Structure(structure) => structure.try_clone().ok(),
            _ => None,
        });
        let tree = tree.ok_or(LoginError::MalformedSuccessPayload)?;
        ticket =
            Some(Ticket::try_from_ttlv(tree).map_err(|_| LoginError::MalformedSuccessPayload)?);
    }
    ticket.ok_or(LoginError::MalformedSuccessPayload)
}

/// Payload-free error converting a Login response batch item.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoginError {
    /// The response item is not the client-to-server Login operation.
    UnexpectedOperation,
    /// The validated response item did not expose Result Status.
    MissingResultStatus,
    /// The represented KMIP result violates the shared status/reason contract.
    InvalidOperationResult(ResultValidationError),
    /// A successful response omitted or malformed its required Ticket.
    MalformedSuccessPayload,
}

impl fmt::Display for LoginError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedOperation => formatter.write_str("response item is not Login"),
            Self::MissingResultStatus => formatter.write_str("Login result status is missing"),
            Self::InvalidOperationResult(cause) => {
                write!(formatter, "Login operation result is invalid: {cause}")
            }
            Self::MalformedSuccessPayload => {
                formatter.write_str("successful Login Ticket payload is malformed")
            }
        }
    }
}

impl Error for LoginError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidOperationResult(cause) => Some(cause),
            _ => None,
        }
    }
}
