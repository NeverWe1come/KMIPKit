//! Request-delivery evidence and safe transport errors.

use std::error::Error;
use std::fmt;

/// The strongest evidence available about request transmission and response reception.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RequestDeliveryState {
    /// No request byte was sent.
    NotSent,
    /// Transmission began, but no response byte has been received.
    PossiblySent,
    /// At least one response byte arrived, but no complete result is available.
    ResponseStarted,
}

impl RequestDeliveryState {
    /// Returns the state before request transmission begins.
    #[must_use]
    pub const fn not_sent() -> Self {
        Self::NotSent
    }

    /// Advances to `PossiblySent` when request transmission begins.
    #[must_use]
    pub const fn write_started(self) -> Self {
        match self {
            Self::NotSent => Self::PossiblySent,
            Self::PossiblySent | Self::ResponseStarted => self,
        }
    }

    /// Advances after a read yields bytes; a zero-byte read adds no evidence.
    #[must_use]
    pub const fn response_bytes_received(self, byte_count: usize) -> Self {
        match (self, byte_count) {
            (Self::NotSent, _) | (_, 0) => self,
            (Self::PossiblySent | Self::ResponseStarted, _) => Self::ResponseStarted,
        }
    }
}

/// A safe transport cause category that contains no caller-provided text.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransportCauseCategory {
    /// An operating-system or stream I/O failure.
    Io,
    /// A TLS handshake, certificate, or record failure.
    Tls,
    /// An HTTP framing or status failure.
    Http,
    /// A timeout occurred.
    Timeout,
    /// The source does not fit another safe category.
    Other,
}

impl fmt::Display for TransportCauseCategory {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io => formatter.write_str("I/O failure"),
            Self::Tls => formatter.write_str("TLS failure"),
            Self::Http => formatter.write_str("HTTP failure"),
            Self::Timeout => formatter.write_str("timeout"),
            Self::Other => formatter.write_str("other transport cause"),
        }
    }
}

impl Error for TransportCauseCategory {}

/// A transport failure retaining only safe cause and delivery categories.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransportError {
    delivery_state: RequestDeliveryState,
    cause: TransportCauseCategory,
}

impl TransportError {
    /// Sanitizes and discards `source`, retaining its safe category and delivery state.
    pub fn new<E>(
        delivery_state: RequestDeliveryState,
        cause: TransportCauseCategory,
        source: E,
    ) -> Self
    where
        E: Error + 'static,
    {
        drop(source);
        Self {
            delivery_state,
            cause,
        }
    }

    /// Returns the strongest request-delivery evidence.
    #[must_use]
    pub const fn delivery_state(&self) -> RequestDeliveryState {
        self.delivery_state
    }

    /// Returns the safe transport cause category.
    #[must_use]
    pub const fn cause_category(&self) -> TransportCauseCategory {
        self.cause
    }
}

impl fmt::Display for TransportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "transport failure ({}, {:?})",
            self.cause, self.delivery_state
        )
    }
}

impl Error for TransportError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.cause)
    }
}
