//! Typed KMIP 2.1 Delegated Login operation models.
//!
//! Request and response fields follow OASIS KMIP Specification v2.1 §6.1.12,
//! Tables 199–201. Returned Ticket values follow §7.39, Table 391.

use std::{error::Error, fmt};

use kmipkit_ttlv::{Structure, Value, ValueView};

use crate::cryptographic_operation;
use crate::{
    KmipOperationResult, ProtocolError, ResponseBatchItemView, ResultMessage,
    ResultValidationError, Ticket,
};

const OPERATION: u32 = 0x0000_002F;
const LEASE_TIME: u32 = 0x0042_0049;
const REQUEST_COUNT: u32 = 0x0042_014C;
const USAGE_LIMITS: u32 = 0x0042_0095;
const RIGHTS: u32 = 0x0042_014D;
const TICKET: u32 = 0x0042_0149;
const SUCCESS: u32 = 0;

enum LeaseTime {
    Interval(u32),
    DateTime(i64),
}

/// A typed client-to-server Delegated Login request from §6.1.12, Table 199.
///
/// Rights is required. Lease Time, Request Count, and Usage Limits remain
/// absent unless the caller explicitly supplies them. Rights and Usage Limits
/// preserve their structurally valid generic TTLV values.
pub struct DelegatedLoginRequest {
    lease_time: Option<LeaseTime>,
    request_count: Option<i32>,
    usage_limits: Option<Structure>,
    rights: Structure,
}

impl DelegatedLoginRequest {
    /// Creates a request with the required Rights structure.
    #[must_use]
    pub fn new(rights: Structure) -> Self {
        Self {
            lease_time: None,
            request_count: None,
            usage_limits: None,
            rights,
        }
    }

    /// Supplies Lease Time using the Interval form from §6.1.12, Table 199.
    #[must_use]
    pub fn with_lease_time_interval(mut self, seconds: u32) -> Self {
        self.lease_time = Some(LeaseTime::Interval(seconds));
        self
    }

    /// Supplies Lease Time using the Date Time form from §6.1.12, Table 199.
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

    /// Encodes the Table 199 Request Payload in protocol field order.
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
            push(&mut payload, LEASE_TIME, value)?;
        }
        if let Some(count) = self.request_count {
            push(&mut payload, REQUEST_COUNT, Value::integer(count))?;
        }
        if let Some(usage_limits) = self.usage_limits {
            push(&mut payload, USAGE_LIMITS, Value::structure(usage_limits))?;
        }
        push(&mut payload, RIGHTS, Value::structure(self.rights))?;
        Ok(payload)
    }
}

impl fmt::Debug for DelegatedLoginRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DelegatedLoginRequest")
            .field("lease_time", &self.lease_time.is_some())
            .field("request_count", &self.request_count)
            .field("usage_limits", &self.usage_limits.is_some())
            .field("rights", &"[PRESERVED TTLV]")
            .finish()
    }
}

fn push(payload: &mut Structure, raw_tag: u32, value: Value) -> Result<(), ProtocolError> {
    cryptographic_operation::push(payload, raw_tag, value)
}

/// A typed Delegated Login response from §6.1.12, Table 200, or a preserved
/// operation failure from Table 201.
#[derive(Debug)]
pub struct DelegatedLoginResponse {
    result: KmipOperationResult,
    ticket: Option<Ticket>,
}

impl DelegatedLoginResponse {
    /// Converts one validated Delegated Login response batch item.
    ///
    /// A successful response requires exactly one valid Ticket. Failure and
    /// Pending responses retain their shared result and expose no Ticket.
    /// Unknown values inside the Ticket remain preserved by its typed model;
    /// the complete generic response remains available from the source
    /// [`crate::ResponseMessage`].
    ///
    /// # Errors
    ///
    /// Returns [`DelegatedLoginError`] for another operation, invalid result
    /// metadata, or a malformed successful Ticket payload. Diagnostics never
    /// contain Ticket Value bytes.
    pub fn try_from_response_item(
        item: ResponseBatchItemView<'_>,
    ) -> Result<Self, DelegatedLoginError> {
        if item.operation() != Some(OPERATION) {
            return Err(DelegatedLoginError::UnexpectedOperation);
        }
        let status = item
            .result_status()
            .ok_or(DelegatedLoginError::MissingResultStatus)?;
        let result_message = item.with_result_message(|text| ResultMessage::new(text.to_owned()));
        let result = KmipOperationResult::new(status, item.result_reason(), result_message)
            .map_err(DelegatedLoginError::InvalidOperationResult)?;
        let ticket = if status.raw() == SUCCESS {
            let payload = item
                .with_response_payload(|payload| parse_ticket(&payload))
                .ok_or(DelegatedLoginError::MalformedSuccessPayload)??;
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

fn parse_ticket(payload: &kmipkit_ttlv::StructureView<'_>) -> Result<Ticket, DelegatedLoginError> {
    let mut ticket = None;
    for field in payload.children() {
        if field.tag().raw() != TICKET {
            continue;
        }
        if ticket.is_some() {
            return Err(DelegatedLoginError::MalformedSuccessPayload);
        }
        let tree = field.with_value(|value| match value {
            ValueView::Structure(structure) => structure.try_clone().ok(),
            _ => None,
        });
        let tree = tree.ok_or(DelegatedLoginError::MalformedSuccessPayload)?;
        ticket = Some(
            Ticket::try_from_ttlv(tree)
                .map_err(|_| DelegatedLoginError::MalformedSuccessPayload)?,
        );
    }
    ticket.ok_or(DelegatedLoginError::MalformedSuccessPayload)
}

/// Payload-free error converting a Delegated Login response batch item.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DelegatedLoginError {
    /// The response item is not the client-to-server Delegated Login operation.
    UnexpectedOperation,
    /// The validated response item did not expose Result Status.
    MissingResultStatus,
    /// The represented KMIP result violates the shared status/reason contract.
    InvalidOperationResult(ResultValidationError),
    /// A successful response omitted or malformed its required Ticket.
    MalformedSuccessPayload,
}

impl fmt::Display for DelegatedLoginError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedOperation => {
                formatter.write_str("response item is not Delegated Login")
            }
            Self::MissingResultStatus => {
                formatter.write_str("Delegated Login result status is missing")
            }
            Self::InvalidOperationResult(cause) => {
                write!(
                    formatter,
                    "Delegated Login operation result is invalid: {cause}"
                )
            }
            Self::MalformedSuccessPayload => {
                formatter.write_str("successful Delegated Login Ticket payload is malformed")
            }
        }
    }
}

impl Error for DelegatedLoginError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidOperationResult(cause) => Some(cause),
            _ => None,
        }
    }
}
