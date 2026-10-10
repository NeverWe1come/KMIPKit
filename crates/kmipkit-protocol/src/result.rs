//! Lossless values for KMIP operation results.

mod generated {
    include!("result_values_generated.rs");
}

use std::error::Error;
use std::fmt;

use crate::ResponseBatchItemView;

/// A server-reported KMIP Result Status, retaining unknown 32-bit values.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ResultStatus(u32);

impl ResultStatus {
    /// Creates a status from its raw KMIP Enumeration value.
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Returns the exact raw KMIP Enumeration value.
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }

    /// Returns the assigned catalog name, or `None` for an unknown value.
    #[must_use]
    pub fn known_name(self) -> Option<&'static str> {
        generated::RESULT_STATUSES
            .iter()
            .find_map(|(value, name)| (*value == self).then_some(*name))
    }

    /// Returns all assigned KMIP 2.1 status values from the normative catalog.
    #[must_use]
    pub const fn known_values() -> &'static [(Self, &'static str)] {
        generated::RESULT_STATUSES
    }

    fn is_success(self) -> bool {
        self.known_name() == Some("Success")
    }

    fn is_failure(self) -> bool {
        self.known_name() == Some("Operation Failed")
    }
}

/// A server-reported KMIP Result Reason, retaining unknown 32-bit values.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ResultReason(u32);

impl ResultReason {
    /// Creates a reason from its raw KMIP Enumeration value.
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Returns the exact raw KMIP Enumeration value.
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }

    /// Returns the assigned catalog name, or `None` for an unknown value.
    #[must_use]
    pub fn known_name(self) -> Option<&'static str> {
        generated::RESULT_REASONS
            .iter()
            .find_map(|(value, name)| (*value == self).then_some(*name))
    }

    /// Returns all assigned KMIP 2.1 reason values from the normative catalog.
    #[must_use]
    pub const fn known_values() -> &'static [(Self, &'static str)] {
        generated::RESULT_REASONS
    }
}

/// Optional, untrusted Result Message text returned by a KMIP server.
#[derive(Clone, Eq, PartialEq)]
pub struct ResultMessage(String);

impl ResultMessage {
    /// Creates a Result Message from valid UTF-8 text.
    #[must_use]
    pub fn new(text: String) -> Self {
        Self(text)
    }

    /// Returns the exact message text for explicit application inspection.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the UTF-8 bytes of the exact message text.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl fmt::Debug for ResultMessage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ResultMessage([REDACTED])")
    }
}

/// A complete server-reported result for one KMIP operation.
#[derive(Clone, Eq, PartialEq)]
pub struct KmipOperationResult {
    status: ResultStatus,
    reason: Option<ResultReason>,
    message: Option<ResultMessage>,
}

impl KmipOperationResult {
    /// Creates a result and enforces the KMIP Success/Failure reason invariant.
    ///
    /// A Failure status requires a reason and a Success status forbids one.
    /// Other known and unknown statuses impose no reason-presence rule here.
    ///
    /// # Errors
    ///
    /// Returns [`ResultValidationError`] when a Success/Failure reason-presence
    /// invariant is violated.
    pub fn new(
        status: ResultStatus,
        reason: Option<ResultReason>,
        message: Option<ResultMessage>,
    ) -> Result<Self, ResultValidationError> {
        if status.is_failure() && reason.is_none() {
            return Err(ResultValidationError::FailureRequiresReason);
        }
        if status.is_success() && reason.is_some() {
            return Err(ResultValidationError::SuccessForbidsReason);
        }
        Ok(Self {
            status,
            reason,
            message,
        })
    }

    /// Returns the exact status value.
    #[must_use]
    pub const fn status(&self) -> ResultStatus {
        self.status
    }

    /// Returns the optional exact reason value.
    #[must_use]
    pub const fn reason(&self) -> Option<ResultReason> {
        self.reason
    }

    /// Returns the optional untrusted message for explicit inspection.
    #[must_use]
    pub const fn message(&self) -> Option<&ResultMessage> {
        self.message.as_ref()
    }
}

/// A missing or inconsistent common result on a validated response item.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OperationResultParseError {
    MissingResultStatus,
    Invalid(ResultValidationError),
}

/// Parses the shared Result Status, Result Reason, and Result Message fields.
///
/// Operation-specific response converters map this error into their own
/// payload-free error type. The parsed result retains unknown status/reason
/// values and leaves Pending available for explicit outcome routing.
pub(crate) fn parse_operation_result(
    item: ResponseBatchItemView<'_>,
) -> Result<KmipOperationResult, OperationResultParseError> {
    let status = item
        .result_status()
        .ok_or(OperationResultParseError::MissingResultStatus)?;
    let reason = item.result_reason();
    let message = item.with_result_message(|text| ResultMessage::new(text.to_owned()));

    KmipOperationResult::new(status, reason, message).map_err(OperationResultParseError::Invalid)
}

impl fmt::Debug for KmipOperationResult {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("KmipOperationResult")
            .field("status", &self.status)
            .field("reason", &self.reason)
            .field("message", &self.message)
            .finish()
    }
}

impl fmt::Display for KmipOperationResult {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "KMIP operation result status {}",
            self.status.raw()
        )?;
        if let Some(reason) = self.reason {
            write!(formatter, " with reason {}", reason.raw())?;
        }
        Ok(())
    }
}

/// A rejected status/reason combination in a represented operation result.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResultValidationError {
    /// A Failure status was represented without a Result Reason.
    FailureRequiresReason,
    /// A Success status was represented with a Result Reason.
    SuccessForbidsReason,
}

impl fmt::Display for ResultValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FailureRequiresReason => formatter.write_str("Failure requires a Result Reason"),
            Self::SuccessForbidsReason => formatter.write_str("Success forbids a Result Reason"),
        }
    }
}

impl Error for ResultValidationError {}
