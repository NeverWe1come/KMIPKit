//! Shared helpers for client-initiated KMIP asynchronous operations.

use std::error::Error;
use std::fmt;

use kmipkit_ttlv::{Item, ModelError, RawTag, Structure, Tag, Value};
use zeroize::Zeroizing;

use crate::{
    KmipOperationResult, ProtocolCauseCategory, ProtocolError, ProtocolErrorKind,
    ResponseBatchItemView, ResultStatus, ResultValidationError,
};

const ASYNCHRONOUS_CORRELATION_VALUE: u32 = 0x0042_0006;
const SUCCESS: u32 = 0;
const OPERATION_PENDING: u32 = 2;

/// KMIPKit-owned asynchronous correlation bytes, redacted from diagnostics and
/// zeroized when dropped.
#[derive(Clone)]
pub(crate) struct SecretBytes(Zeroizing<Vec<u8>>);

impl SecretBytes {
    pub(crate) fn new(bytes: &[u8]) -> Self {
        Self(Zeroizing::new(bytes.to_vec()))
    }

    pub(crate) fn from_vec(bytes: Vec<u8>) -> Self {
        Self(Zeroizing::new(bytes))
    }

    pub(crate) fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }

    pub(crate) fn into_value(self) -> Value {
        // `Value::ByteString` zeroizes its owned allocation when dropped.
        let mut bytes = self.0;
        Value::byte_string(std::mem::take(&mut *bytes))
    }
}

impl fmt::Debug for SecretBytes {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SecretBytes([REDACTED])")
    }
}

/// A safe failure converting a validated asynchronous-operation response.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AsynchronousOperationError {
    /// The response item names a different KMIP operation.
    UnexpectedOperation,
    /// A validated response item does not expose Result Status.
    MissingResultStatus,
    /// The represented status/reason/message combination is inconsistent.
    InvalidOperationResult(ResultValidationError),
    /// A required Asynchronous Correlation Value is absent or has the wrong type.
    MissingAsynchronousCorrelationValue,
    /// A required Response Payload is absent.
    MissingResponsePayload,
    /// A response payload is present where the operation forbids it.
    UnexpectedResponsePayload,
    /// A required operation-specific payload field is absent, duplicated, or malformed.
    MalformedResponsePayload,
    /// A response status is forbidden by the operation's synchronous contract.
    ForbiddenResultStatus,
}

impl fmt::Display for AsynchronousOperationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::UnexpectedOperation => "response item is for a different operation",
            Self::MissingResultStatus => "asynchronous operation result status is missing",
            Self::InvalidOperationResult(cause) => {
                return write!(
                    formatter,
                    "asynchronous operation result is invalid: {cause}"
                );
            }
            Self::MissingAsynchronousCorrelationValue => {
                "required asynchronous correlation value is missing or malformed"
            }
            Self::MissingResponsePayload => "required asynchronous response payload is missing",
            Self::UnexpectedResponsePayload => {
                "asynchronous response payload is forbidden for this result"
            }
            Self::MalformedResponsePayload => "asynchronous response payload is malformed",
            Self::ForbiddenResultStatus => "result status is forbidden for this operation",
        };
        formatter.write_str(message)
    }
}

impl Error for AsynchronousOperationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidOperationResult(cause) => Some(cause),
            _ => None,
        }
    }
}

pub(crate) fn operation_result(
    item: ResponseBatchItemView<'_>,
) -> Result<KmipOperationResult, AsynchronousOperationError> {
    crate::result::parse_operation_result(item).map_err(|error| match error {
        crate::result::OperationResultParseError::MissingResultStatus => {
            AsynchronousOperationError::MissingResultStatus
        }
        crate::result::OperationResultParseError::Invalid(cause) => {
            AsynchronousOperationError::InvalidOperationResult(cause)
        }
    })
}

pub(crate) fn item(raw_tag: u32, value: Value) -> Result<Item, ProtocolError> {
    Item::new(tag(raw_tag)?, value).map_err(model_error)
}

pub(crate) fn structure(items: impl IntoIterator<Item = Item>) -> Result<Structure, ProtocolError> {
    let mut structure = Structure::new();
    for item in items {
        structure.try_push(item).map_err(model_error)?;
    }
    Ok(structure)
}

pub(crate) fn tag(raw_tag: u32) -> Result<Tag, ProtocolError> {
    RawTag::new(raw_tag)
        .and_then(|raw| raw.try_checked())
        .map_err(model_error)
}

pub(crate) fn model_error(error: ModelError) -> ProtocolError {
    ProtocolError::new(
        ProtocolErrorKind::InvalidValue,
        ProtocolCauseCategory::InvalidValue,
        error,
    )
}

pub(crate) const fn is_success(status: ResultStatus) -> bool {
    status.raw() == SUCCESS
}

pub(crate) fn is_failure(status: ResultStatus) -> bool {
    status.known_name() == Some("Operation Failed")
}

pub(crate) const fn is_pending(status: ResultStatus) -> bool {
    status.raw() == OPERATION_PENDING
}

pub(crate) const fn correlation_tag() -> u32 {
    ASYNCHRONOUS_CORRELATION_VALUE
}
