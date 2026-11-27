//! Safe client errors that distinguish local failures from server results.

use std::error::Error;
use std::fmt;
use std::sync::Arc;

use kmipkit_protocol::{
    KmipOperationResult, ProfileValidationError, ProtocolCauseCategory, ProtocolError,
    ResponseMessage, ResultReason, ResultStatus,
};
use kmipkit_transport::{RequestDeliveryState, TransportCauseCategory, TransportError};
use kmipkit_ttlv::StructureView;
use zeroize::Zeroizing;

/// A redacted copy of a server-issued Asynchronous Correlation Value.
///
/// The wrapped bytes live in zeroizing memory and are never formatted in
/// diagnostics or `Debug` output.
#[derive(Clone, Eq, PartialEq)]
pub struct RedactedCorrelation(Arc<Zeroizing<Vec<u8>>>);

impl RedactedCorrelation {
    pub(crate) fn new(bytes: Vec<u8>) -> Self {
        Self(Arc::new(Zeroizing::new(bytes)))
    }

    /// Borrows the exact correlation bytes needed to Poll or Cancel.
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl fmt::Debug for RedactedCorrelation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RedactedCorrelation([REDACTED])")
    }
}

/// An owned generic TTLV response retained for Unique Identifier shape errors in signature and verification responses.
///
/// The response is shared by cloned errors, zeroizes its payloads when the
/// final owner is dropped, and is never included in formatted diagnostics.
pub struct ClientErrorResponseTtlv {
    response: Arc<ResponseMessage>,
}

impl ClientErrorResponseTtlv {
    pub(crate) fn new(response: ResponseMessage) -> Self {
        Self {
            response: Arc::new(response),
        }
    }

    /// Lends the complete ordered generic TTLV response for callback-scoped access.
    ///
    /// Use [`StructureView::try_clone`] inside the callback only when the view
    /// must outlive it. The owned copy zeroizes its payloads when dropped.
    pub fn with_ttlv<R>(&self, callback: impl for<'a> FnOnce(StructureView<'a>) -> R) -> R {
        self.response.with_ttlv(callback)
    }
}

impl Clone for ClientErrorResponseTtlv {
    fn clone(&self) -> Self {
        Self {
            response: Arc::clone(&self.response),
        }
    }
}

impl fmt::Debug for ClientErrorResponseTtlv {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ClientErrorResponseTtlv")
            .field("retained", &true)
            .finish()
    }
}

impl PartialEq for ClientErrorResponseTtlv {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.response, &other.response)
    }
}

impl Eq for ClientErrorResponseTtlv {}

/// The safe layer that produced a client-visible outcome.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClientErrorCategory {
    /// Local application input failed validation.
    Validation,
    /// A protocol value could not be represented or processed.
    Protocol,
    /// A transport operation failed.
    Transport,
    /// The server returned a complete KMIP operation result.
    ServerResult,
}

/// A safe cause category retained by a local client error.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClientCauseCategory {
    /// A local application value failed validation.
    InvalidInput,
    /// A protocol-processing cause.
    Protocol(ProtocolCauseCategory),
    /// A transport cause.
    Transport(TransportCauseCategory),
    /// A validation cause without a more specific safe category.
    Other,
}

impl fmt::Display for ClientCauseCategory {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput => formatter.write_str("invalid input"),
            Self::Protocol(category) => write!(formatter, "protocol: {category}"),
            Self::Transport(category) => write!(formatter, "transport: {category}"),
            Self::Other => formatter.write_str("other validation cause"),
        }
    }
}

impl Error for ClientCauseCategory {}

/// A client outcome containing safe local failure metadata or a server result.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClientError {
    /// A local validation failure with delivery evidence; arbitrary source text is discarded.
    Validation {
        /// The safe cause category.
        cause: ClientCauseCategory,
        /// The strongest available request delivery evidence.
        delivery_state: RequestDeliveryState,
    },
    /// A protocol-processing failure with required delivery evidence and sanitized cause.
    Protocol {
        /// The sanitized protocol cause.
        error: ProtocolError,
        /// The strongest available request delivery evidence.
        delivery_state: RequestDeliveryState,
    },
    /// An approved cryptographic response-shape failure that retains the generic response.
    ProtocolResponse {
        /// The sanitized protocol cause.
        error: ProtocolError,
        /// The strongest available request delivery evidence.
        delivery_state: RequestDeliveryState,
        /// The response message for explicit inspection of its malformed required Unique Identifier.
        response_ttlv: ClientErrorResponseTtlv,
    },
    /// A profile-specific validation failure with stable source IDs and delivery evidence.
    ProfileValidation {
        /// The redacted profile, requirement, and source-clause failure.
        error: ProfileValidationError,
        /// The strongest available request delivery evidence.
        delivery_state: RequestDeliveryState,
    },
    /// A transport failure with sanitized cause and delivery evidence.
    Transport(TransportError),
    /// A complete server result, with no local delivery-failure state.
    ServerResult(KmipOperationResult),
    /// The server accepted the request asynchronously (`ResultStatus::Pending`)
    /// and returned an Asynchronous Correlation Value for explicit `Poll`/`Cancel`.
    Pending {
        /// The complete server result (`status = Pending`).
        result: KmipOperationResult,
        /// The redacted Asynchronous Correlation Value returned by the server.
        correlation_value: RedactedCorrelation,
    },
}

impl ClientError {
    /// Creates a validation failure with delivery evidence and immediately discards its source.
    pub fn validation<E>(
        cause: ClientCauseCategory,
        delivery_state: RequestDeliveryState,
        source: E,
    ) -> Self
    where
        E: Error + 'static,
    {
        drop(source);
        Self::Validation {
            cause,
            delivery_state,
        }
    }

    /// Wraps a protocol failure with delivery evidence and no discarded source text.
    #[must_use]
    pub const fn protocol(error: ProtocolError, delivery_state: RequestDeliveryState) -> Self {
        Self::Protocol {
            error,
            delivery_state,
        }
    }

    /// Wraps an approved Unique Identifier response-shape failure while preserving its message.
    #[must_use]
    pub(crate) const fn protocol_response(
        error: ProtocolError,
        delivery_state: RequestDeliveryState,
        response_ttlv: ClientErrorResponseTtlv,
    ) -> Self {
        Self::ProtocolResponse {
            error,
            delivery_state,
            response_ttlv,
        }
    }

    /// Wraps a source-backed profile failure without losing its stable IDs.
    #[must_use]
    pub const fn profile_validation(
        error: ProfileValidationError,
        delivery_state: RequestDeliveryState,
    ) -> Self {
        Self::ProfileValidation {
            error,
            delivery_state,
        }
    }

    /// Returns the structured profile validation failure, when present.
    #[must_use]
    pub const fn profile_validation_error(&self) -> Option<&ProfileValidationError> {
        match self {
            Self::ProfileValidation { error, .. } => Some(error),
            Self::Validation { .. }
            | Self::Protocol { .. }
            | Self::ProtocolResponse { .. }
            | Self::Transport(_)
            | Self::ServerResult(_)
            | Self::Pending { .. } => None,
        }
    }

    /// Returns the generic response retained for an approved malformed required
    /// Unique Identifier in a successful MAC, MAC Verify, Sign, or Signature
    /// Verify response for verification response parsing.
    ///
    /// This is `None` for failures that occur before a complete response is
    /// decoded and for validation or transport failures unrelated to response
    /// processing. Profile validation failures do not retain the raw response
    /// body. Any retained response is never included in diagnostics.
    #[must_use]
    pub const fn response_ttlv(&self) -> Option<&ClientErrorResponseTtlv> {
        match self {
            Self::ProtocolResponse { response_ttlv, .. } => Some(response_ttlv),
            Self::Validation { .. }
            | Self::Protocol { .. }
            | Self::Transport(_)
            | Self::ProfileValidation { .. }
            | Self::ServerResult(_)
            | Self::Pending { .. } => None,
        }
    }

    /// Wraps a transport failure and its delivery evidence.
    #[must_use]
    pub const fn transport(error: TransportError) -> Self {
        Self::Transport(error)
    }

    /// Represents a complete KMIP server result.
    #[must_use]
    pub const fn server_result(result: KmipOperationResult) -> Self {
        Self::ServerResult(result)
    }

    /// Represents an asynchronous `Pending` outcome returned where a completed
    /// result was expected.
    #[must_use]
    pub const fn pending(
        result: KmipOperationResult,
        correlation_value: RedactedCorrelation,
    ) -> Self {
        Self::Pending {
            result,
            correlation_value,
        }
    }

    /// Returns `true` when the server returned `ResultStatus::Pending`.
    #[must_use]
    pub const fn is_pending(&self) -> bool {
        matches!(self, Self::Pending { .. })
    }

    /// Returns `true` when the server returned a non-pending server error result.
    #[must_use]
    pub const fn is_server_rejected(&self) -> bool {
        matches!(self, Self::ServerResult(_))
    }

    /// Borrows the Asynchronous Correlation Value when the server returned `Pending`.
    #[must_use]
    pub fn asynchronous_correlation_value(&self) -> Option<&[u8]> {
        match self {
            Self::Pending {
                correlation_value, ..
            } => Some(correlation_value.as_slice()),
            Self::Validation { .. }
            | Self::ProfileValidation { .. }
            | Self::Protocol { .. }
            | Self::ProtocolResponse { .. }
            | Self::Transport(_)
            | Self::ServerResult(_) => None,
        }
    }

    /// Returns the safe outcome category.
    #[must_use]
    pub const fn category(&self) -> ClientErrorCategory {
        match self {
            Self::Validation { .. } | Self::ProfileValidation { .. } => {
                ClientErrorCategory::Validation
            }
            Self::Protocol { .. } | Self::ProtocolResponse { .. } => ClientErrorCategory::Protocol,
            Self::Transport(_) => ClientErrorCategory::Transport,
            Self::ServerResult(_) | Self::Pending { .. } => ClientErrorCategory::ServerResult,
        }
    }

    /// Returns the safe cause category, or `None` for a complete server result.
    #[must_use]
    pub const fn cause_category(&self) -> Option<ClientCauseCategory> {
        match self {
            Self::Validation { cause, .. } => Some(*cause),
            Self::ProfileValidation { .. } => Some(ClientCauseCategory::Other),
            Self::Protocol { error, .. } | Self::ProtocolResponse { error, .. } => {
                Some(ClientCauseCategory::Protocol(error.cause_category()))
            }
            Self::Transport(error) => Some(ClientCauseCategory::Transport(error.cause_category())),
            Self::ServerResult(_) | Self::Pending { .. } => None,
        }
    }

    /// Returns delivery evidence for a local failure, or `None` for a complete server result.
    #[must_use]
    pub const fn delivery_state(&self) -> Option<RequestDeliveryState> {
        match self {
            Self::Validation { delivery_state, .. }
            | Self::Protocol { delivery_state, .. }
            | Self::ProtocolResponse { delivery_state, .. }
            | Self::ProfileValidation { delivery_state, .. } => Some(*delivery_state),
            Self::Transport(error) => Some(error.delivery_state()),
            Self::ServerResult(_) | Self::Pending { .. } => None,
        }
    }

    /// Returns the server result for a server-result or pending outcome.
    #[must_use]
    pub const fn server_operation_result(&self) -> Option<&KmipOperationResult> {
        match self {
            Self::ServerResult(result) | Self::Pending { result, .. } => Some(result),
            Self::Validation { .. }
            | Self::Protocol { .. }
            | Self::ProtocolResponse { .. }
            | Self::ProfileValidation { .. }
            | Self::Transport(_) => None,
        }
    }

    /// Returns the server's raw status for a server-result or pending outcome.
    #[must_use]
    pub const fn server_status(&self) -> Option<ResultStatus> {
        match self {
            Self::ServerResult(result) | Self::Pending { result, .. } => Some(result.status()),
            Self::Validation { .. }
            | Self::Protocol { .. }
            | Self::ProtocolResponse { .. }
            | Self::ProfileValidation { .. }
            | Self::Transport(_) => None,
        }
    }

    /// Returns the server's optional result reason for a server-result or pending outcome.
    #[must_use]
    pub const fn server_reason(&self) -> Option<ResultReason> {
        match self {
            Self::ServerResult(result) | Self::Pending { result, .. } => result.reason(),
            Self::Validation { .. }
            | Self::Protocol { .. }
            | Self::ProtocolResponse { .. }
            | Self::ProfileValidation { .. }
            | Self::Transport(_) => None,
        }
    }
}

impl fmt::Display for ClientError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation {
                cause,
                delivery_state,
            } => write!(
                formatter,
                "client validation failure ({cause}, {delivery_state:?})"
            ),
            Self::ProfileValidation {
                error,
                delivery_state,
            } => write!(formatter, "client {error} ({delivery_state:?})"),
            Self::Protocol {
                error,
                delivery_state,
            }
            | Self::ProtocolResponse {
                error,
                delivery_state,
                ..
            } => write!(
                formatter,
                "client protocol failure ({error}, {delivery_state:?})"
            ),
            Self::Transport(error) => write!(formatter, "client {error}"),
            Self::ServerResult(result) | Self::Pending { result, .. } => {
                write!(formatter, "server returned {result}")
            }
        }
    }
}

impl Error for ClientError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Validation { cause, .. } => Some(cause),
            Self::ProfileValidation { error, .. } => Some(error),
            Self::Protocol { error, .. } | Self::ProtocolResponse { error, .. } => Some(error),
            Self::Transport(error) => Some(error),
            Self::ServerResult(_) | Self::Pending { .. } => None,
        }
    }
}
