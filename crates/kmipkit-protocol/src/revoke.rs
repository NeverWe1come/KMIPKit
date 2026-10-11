//! Typed KMIP 2.1 Revoke request and response models.

use std::error::Error;
use std::fmt;

use kmipkit_ttlv::{Structure, StructureView, Value};

use crate::cryptographic_operation::{self as common, OwnedResponseTtlv};
use crate::{
    KmipOperationResult, ProtocolCauseCategory, ProtocolError, ProtocolErrorKind,
    ResponseBatchItemView, ResultMessage, ResultValidationError, UniqueIdentifier,
};

const OPERATION: u32 = 0x0000_0013;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const REVOCATION_REASON: u32 = 0x0042_0081;
const REVOCATION_MESSAGE: u32 = 0x0042_0080;
const REVOCATION_REASON_CODE: u32 = 0x0042_0082;
const COMPROMISE_OCCURRENCE_DATE: u32 = 0x0042_0021;
const SUCCESS: u32 = 0;
const KEY_COMPROMISE_REASON: u32 = 2;
const CA_COMPROMISE_REASON: u32 = 3;

/// A raw-preserving Revocation Reason Code from §11.48, Table 481.
///
/// Unknown and future extension values remain representable.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RevocationReason(u32);

impl RevocationReason {
    /// Unspecified reason (value `0x00000001`).
    pub const UNSPECIFIED: Self = Self(1);
    /// Key compromise (value `0x00000002`).
    pub const KEY_COMPROMISE: Self = Self(KEY_COMPROMISE_REASON);
    /// CA compromise (value `0x00000003`).
    pub const CA_COMPROMISE: Self = Self(CA_COMPROMISE_REASON);
    /// Affiliation changed (value `0x00000004`).
    pub const AFFILIATION_CHANGED: Self = Self(4);
    /// Superseded (value `0x00000005`).
    pub const SUPERSEDED: Self = Self(5);
    /// Cessation of operation (value `0x00000006`).
    pub const CESSATION_OF_OPERATION: Self = Self(6);
    /// Privilege withdrawn (value `0x00000007`).
    pub const PRIVILEGE_WITHDRAWN: Self = Self(7);

    /// Creates a reason from its raw unsigned Enumeration value.
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Returns the exact raw unsigned Enumeration value.
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// A KMIP 2.1 Revoke request from §6.1.44, Table 295.
///
/// The reason remains open to future and vendor values. `KMIPKit` validates the
/// local SHALL NOT on Compromise Occurrence Date; it leaves the SHOULD for
/// compromise dates to the caller and server policy.
pub struct RevokeRequest {
    unique_identifier: Option<UniqueIdentifier>,
    revocation_reason: RevocationReason,
    revocation_message: Option<String>,
    compromise_occurrence_date: Option<i64>,
}

impl RevokeRequest {
    /// Creates a Revoke request with the required Revocation Reason.
    #[must_use]
    pub const fn new(revocation_reason: RevocationReason) -> Self {
        Self {
            unique_identifier: None,
            revocation_reason,
            revocation_message: None,
            compromise_occurrence_date: None,
        }
    }

    /// Sets the optional object identifier. Omission leaves ID Placeholder use explicit.
    #[must_use]
    pub fn with_unique_identifier(mut self, unique_identifier: UniqueIdentifier) -> Self {
        self.unique_identifier = Some(unique_identifier);
        self
    }

    /// Sets the optional audit-only Revocation Message from §4.47, Table 123.
    #[must_use]
    pub fn with_revocation_message(mut self, message: String) -> Self {
        self.revocation_message = Some(message);
        self
    }

    /// Sets the optional raw KMIP Date-Time compromise occurrence value.
    #[must_use]
    pub const fn with_compromise_occurrence_date(mut self, date_time: i64) -> Self {
        self.compromise_occurrence_date = Some(date_time);
        self
    }

    /// Returns the optional identifier in its original wire form.
    #[must_use]
    pub const fn unique_identifier(&self) -> Option<&UniqueIdentifier> {
        self.unique_identifier.as_ref()
    }

    /// Returns the required raw-preserving Revocation Reason.
    #[must_use]
    pub const fn revocation_reason(&self) -> RevocationReason {
        self.revocation_reason
    }

    /// Returns the optional audit-only Revocation Message.
    #[must_use]
    pub fn revocation_message(&self) -> Option<&str> {
        self.revocation_message.as_deref()
    }

    /// Returns the optional raw KMIP Date-Time value.
    #[must_use]
    pub const fn compromise_occurrence_date(&self) -> Option<i64> {
        self.compromise_occurrence_date
    }

    /// Builds the ordered Request Payload from §6.1.44, Table 295.
    ///
    /// # Errors
    ///
    /// Returns a sanitized protocol error when Compromise Occurrence Date is
    /// supplied for a reason other than Key Compromise or CA Compromise, or
    /// when the resulting TTLV model cannot be represented.
    pub fn into_ttlv_payload(self) -> Result<Structure, ProtocolError> {
        if self.compromise_occurrence_date.is_some()
            && !matches!(
                self.revocation_reason.raw(),
                KEY_COMPROMISE_REASON | CA_COMPROMISE_REASON
            )
        {
            return Err(invalid_request());
        }

        let mut payload = Structure::new();
        if let Some(identifier) = self.unique_identifier {
            common::push(
                &mut payload,
                UNIQUE_IDENTIFIER,
                common::unique_identifier_value(identifier),
            )?;
        }
        let mut reason = Structure::new();
        common::push(
            &mut reason,
            REVOCATION_REASON_CODE,
            Value::enumeration(self.revocation_reason.raw()),
        )?;
        if let Some(message) = self.revocation_message {
            common::push(&mut reason, REVOCATION_MESSAGE, Value::text_string(message))?;
        }
        common::push(&mut payload, REVOCATION_REASON, Value::structure(reason))?;
        if let Some(date_time) = self.compromise_occurrence_date {
            common::push(
                &mut payload,
                COMPROMISE_OCCURRENCE_DATE,
                Value::date_time(date_time),
            )?;
        }
        Ok(payload)
    }
}

impl fmt::Debug for RevokeRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RevokeRequest")
            .field("has_unique_identifier", &self.unique_identifier.is_some())
            .field("revocation_reason", &self.revocation_reason)
            .field("has_revocation_message", &self.revocation_message.is_some())
            .field(
                "compromise_occurrence_date",
                &self.compromise_occurrence_date,
            )
            .finish()
    }
}

/// A typed Revoke result from §6.1.44, Table 296.
#[derive(Debug)]
pub struct RevokeResponse {
    result: KmipOperationResult,
    response_ttlv: OwnedResponseTtlv,
    unique_identifier: Option<UniqueIdentifier>,
}

impl RevokeResponse {
    /// Converts one already validated Revoke response batch item.
    ///
    /// A successful result requires exactly one well-formed Unique Identifier.
    /// Non-success results preserve their common status, reason, and message.
    ///
    /// # Errors
    ///
    /// Returns [`RevokeError`] for a different operation, malformed result,
    /// or malformed successful payload. Error text never contains payload data.
    pub fn try_from_response_item(item: ResponseBatchItemView<'_>) -> Result<Self, RevokeError> {
        if item.operation() != Some(OPERATION) {
            return Err(RevokeError::UnexpectedOperation);
        }
        let status = item
            .result_status()
            .ok_or(RevokeError::MissingResultStatus)?;
        let result_message = item.with_result_message(|text| ResultMessage::new(text.to_owned()));
        let result = KmipOperationResult::new(status, item.result_reason(), result_message)
            .map_err(RevokeError::InvalidOperationResult)?;
        let response_ttlv = common::clone_response_item(item, "Revoke")
            .map_err(|_| RevokeError::MalformedResponseItem)?;

        let unique_identifier = if status.raw() == SUCCESS {
            Some(
                item.with_response_payload(|payload| parse_success_identifier(&payload))
                    .ok_or(RevokeError::MalformedSuccessPayload)??,
            )
        } else {
            None
        };

        Ok(Self {
            result,
            response_ttlv,
            unique_identifier,
        })
    }

    /// Returns the exact KMIP operation result.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }

    /// Returns the server-confirmed Unique Identifier on success.
    #[must_use]
    pub const fn unique_identifier(&self) -> Option<&UniqueIdentifier> {
        self.unique_identifier.as_ref()
    }

    /// Lends the complete ordered generic response batch item.
    pub fn with_ttlv<R>(&self, callback: impl for<'a> FnOnce(StructureView<'a>) -> R) -> R {
        self.response_ttlv.with_ttlv(callback)
    }
}

/// A payload-free error converting a validated Revoke response item.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RevokeError {
    /// The response item is not the client-to-server Revoke operation.
    UnexpectedOperation,
    /// The validated response item did not expose a Result Status.
    MissingResultStatus,
    /// The represented KMIP result violates the shared result contract.
    InvalidOperationResult(ResultValidationError),
    /// The generic response item could not be retained.
    MalformedResponseItem,
    /// A successful response omitted, duplicated, or mistyped its identifier.
    MalformedSuccessPayload,
}

impl fmt::Display for RevokeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedOperation => formatter.write_str("response item is not Revoke"),
            Self::MissingResultStatus => formatter.write_str("Revoke result status is missing"),
            Self::InvalidOperationResult(cause) => {
                write!(formatter, "Revoke operation result is invalid: {cause}")
            }
            Self::MalformedResponseItem => formatter.write_str("Revoke response item is malformed"),
            Self::MalformedSuccessPayload => {
                formatter.write_str("successful Revoke response payload is malformed")
            }
        }
    }
}

impl Error for RevokeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidOperationResult(cause) => Some(cause),
            _ => None,
        }
    }
}

fn parse_success_identifier(payload: &StructureView<'_>) -> Result<UniqueIdentifier, RevokeError> {
    crate::lifecycle::successful_response_identifier(payload)
        .ok_or(RevokeError::MalformedSuccessPayload)
}

fn invalid_request() -> ProtocolError {
    ProtocolError::categorized(
        ProtocolErrorKind::InvalidInput,
        ProtocolCauseCategory::InvalidValue,
    )
}
