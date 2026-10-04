//! Safe client errors that distinguish local failures from server results.

use std::error::Error;
use std::fmt;

use kmipkit_protocol::{KmipOperationResult, ProtocolCauseCategory, ProtocolError, ResultStatus};
use kmipkit_transport::{RequestDeliveryState, TransportCauseCategory, TransportError};

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
    /// A local validation failure; arbitrary source text is discarded.
    Validation {
        /// The safe cause category.
        cause: ClientCauseCategory,
        /// Request delivery evidence when this error concerns a request.
        delivery_state: Option<RequestDeliveryState>,
    },
    /// A protocol-processing failure with sanitized protocol cause.
    Protocol {
        /// The sanitized protocol cause.
        error: ProtocolError,
        /// Request delivery evidence when this error concerns a request.
        delivery_state: Option<RequestDeliveryState>,
    },
    /// A transport failure with sanitized cause and delivery evidence.
    Transport(TransportError),
    /// A complete server result, with no local delivery-failure state.
    ServerResult(KmipOperationResult),
}

impl ClientError {
    /// Creates a validation failure and immediately discards its source.
    pub fn validation<E>(
        cause: ClientCauseCategory,
        delivery_state: Option<RequestDeliveryState>,
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

    /// Wraps a protocol failure without restoring its discarded source text.
    #[must_use]
    pub const fn protocol(
        error: ProtocolError,
        delivery_state: Option<RequestDeliveryState>,
    ) -> Self {
        Self::Protocol {
            error,
            delivery_state,
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
            Self::Protocol { .. } => ClientErrorCategory::Protocol,
            Self::Transport(_) => ClientErrorCategory::Transport,
            Self::ServerResult(_) => ClientErrorCategory::ServerResult,
        }
    }

    /// Returns the safe cause category, or `None` for a complete server result.
    #[must_use]
    pub const fn cause_category(&self) -> Option<ClientCauseCategory> {
        match self {
            Self::Validation { cause, .. } => Some(*cause),
            Self::Protocol { error, .. } => {
                Some(ClientCauseCategory::Protocol(error.cause_category()))
            }
            Self::Transport(error) => Some(ClientCauseCategory::Transport(error.cause_category())),
            Self::ServerResult(_) => None,
        }
    }

    /// Returns local request-delivery evidence, if this is a local failure.
    #[must_use]
    pub const fn delivery_state(&self) -> Option<RequestDeliveryState> {
        match self {
            Self::Validation { delivery_state, .. } | Self::Protocol { delivery_state, .. } => {
                *delivery_state
            }
            Self::Transport(error) => Some(error.delivery_state()),
            Self::ServerResult(_) => None,
        }
    }

    /// Returns the server result for a server-result outcome.
    #[must_use]
    pub const fn server_operation_result(&self) -> Option<&KmipOperationResult> {
        match self {
            Self::ServerResult(result) => Some(result),
            Self::Validation { .. } | Self::Protocol { .. } | Self::Transport(_) => None,
        }
    }

    /// Returns the server's raw status for a server-result outcome.
    #[must_use]
    pub const fn server_status(&self) -> Option<ResultStatus> {
        match self {
            Self::ServerResult(result) => Some(result.status()),
            Self::Validation { .. } | Self::Protocol { .. } | Self::Transport(_) => None,
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
            Self::Protocol { error, .. } => Some(error),
            Self::Transport(error) => Some(error),
            Self::ServerResult(_) => None,
        }
    }
}
