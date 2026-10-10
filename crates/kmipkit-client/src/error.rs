//! Safe client errors that distinguish local failures from server results.

use std::error::Error;
use std::fmt;
use std::sync::Arc;

use kmipkit_protocol::{
    KmipOperationResult, ProtocolCauseCategory, ProtocolError, ResponseMessage, ResultStatus,
};
use kmipkit_transport::{RequestDeliveryState, TransportCauseCategory, TransportError};
use kmipkit_ttlv::StructureView;

/// An owned generic TTLV response retained when response validation fails.
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
    /// A response validation failure that retains the complete generic response.
    ProtocolResponse {
        /// The sanitized protocol cause.
        error: ProtocolError,
        /// The strongest available request delivery evidence.
        delivery_state: RequestDeliveryState,
        /// The complete generic response message for explicit inspection.
        response_ttlv: ClientErrorResponseTtlv,
    },
    /// A transport failure with sanitized cause and delivery evidence.
    Transport(TransportError),
    /// A complete server result, with no local delivery-failure state.
    ServerResult(KmipOperationResult),
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

    /// Wraps a typed response failure while preserving its generic batch item.
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

    /// Returns the complete generic response when response validation failed.
    ///
    /// This is `None` for failures that occur before a complete response is
    /// decoded and for validation or transport failures unrelated to response
    /// processing. The retained response is not included in diagnostics.
    #[must_use]
    pub const fn response_ttlv(&self) -> Option<&ClientErrorResponseTtlv> {
        match self {
            Self::ProtocolResponse { response_ttlv, .. } => Some(response_ttlv),
            Self::Validation { .. }
            | Self::Protocol { .. }
            | Self::Transport(_)
            | Self::ServerResult(_) => None,
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

    /// Returns the safe outcome category.
    #[must_use]
    pub const fn category(&self) -> ClientErrorCategory {
        match self {
            Self::Validation { .. } => ClientErrorCategory::Validation,
            Self::Protocol { .. } | Self::ProtocolResponse { .. } => ClientErrorCategory::Protocol,
            Self::Transport(_) => ClientErrorCategory::Transport,
            Self::ServerResult(_) => ClientErrorCategory::ServerResult,
        }
    }

    /// Returns the safe cause category, or `None` for a complete server result.
    #[must_use]
    pub const fn cause_category(&self) -> Option<ClientCauseCategory> {
        match self {
            Self::Validation { cause, .. } => Some(*cause),
            Self::Protocol { error, .. } | Self::ProtocolResponse { error, .. } => {
                Some(ClientCauseCategory::Protocol(error.cause_category()))
            }
            Self::Transport(error) => Some(ClientCauseCategory::Transport(error.cause_category())),
            Self::ServerResult(_) => None,
        }
    }

    /// Returns delivery evidence for a local failure, or `None` for a complete server result.
    #[must_use]
    pub const fn delivery_state(&self) -> Option<RequestDeliveryState> {
        match self {
            Self::Validation { delivery_state, .. }
            | Self::Protocol { delivery_state, .. }
            | Self::ProtocolResponse { delivery_state, .. } => Some(*delivery_state),
            Self::Transport(error) => Some(error.delivery_state()),
            Self::ServerResult(_) => None,
        }
    }

    /// Returns the server result for a server-result outcome.
    #[must_use]
    pub const fn server_operation_result(&self) -> Option<&KmipOperationResult> {
        match self {
            Self::ServerResult(result) => Some(result),
            Self::Validation { .. }
            | Self::Protocol { .. }
            | Self::ProtocolResponse { .. }
            | Self::Transport(_) => None,
        }
    }

    /// Returns the server's raw status for a server-result outcome.
    #[must_use]
    pub const fn server_status(&self) -> Option<ResultStatus> {
        match self {
            Self::ServerResult(result) => Some(result.status()),
            Self::Validation { .. }
            | Self::Protocol { .. }
            | Self::ProtocolResponse { .. }
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
            Self::ServerResult(result) => write!(formatter, "server returned {result}"),
        }
    }
}

impl Error for ClientError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Validation { cause, .. } => Some(cause),
            Self::Protocol { error, .. } | Self::ProtocolResponse { error, .. } => Some(error),
            Self::Transport(error) => Some(error),
            Self::ServerResult(_) => None,
        }
    }
}
