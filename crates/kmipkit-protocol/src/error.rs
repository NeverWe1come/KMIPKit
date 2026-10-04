//! Safe, categorized errors for local protocol processing.

use std::error::Error;
use std::fmt;

/// A safe protocol failure category that contains no caller-provided text.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProtocolErrorKind {
    /// A represented KMIP value violates a local invariant.
    InvalidValue,
    /// A message could not be processed as a complete protocol value.
    MalformedMessage,
    /// The value uses a protocol feature unavailable to this implementation.
    UnsupportedValue,
}

impl fmt::Display for ProtocolErrorKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidValue => formatter.write_str("invalid protocol value"),
            Self::MalformedMessage => formatter.write_str("malformed protocol message"),
            Self::UnsupportedValue => formatter.write_str("unsupported protocol value"),
        }
    }
}

/// A safe cause category retained after an arbitrary source is discarded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProtocolCauseCategory {
    /// A decoded or supplied value failed validation.
    InvalidValue,
    /// A protocol encoding could not be processed.
    InvalidEncoding,
    /// The source does not fit another safe category.
    Other,
}

impl fmt::Display for ProtocolCauseCategory {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidValue => formatter.write_str("invalid value"),
            Self::InvalidEncoding => formatter.write_str("invalid encoding"),
            Self::Other => formatter.write_str("other protocol cause"),
        }
    }
}

impl Error for ProtocolCauseCategory {}

/// A protocol-processing error whose source text and payload are discarded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProtocolError {
    kind: ProtocolErrorKind,
    cause: ProtocolCauseCategory,
}

impl ProtocolError {
    /// Sanitizes and discards `source`, retaining only safe categories.
    pub fn new<E>(kind: ProtocolErrorKind, cause: ProtocolCauseCategory, source: E) -> Self
    where
        E: Error + 'static,
    {
        drop(source);
        Self { kind, cause }
    }

    /// Returns the safe protocol error category.
    #[must_use]
    pub const fn kind(&self) -> ProtocolErrorKind {
        self.kind
    }

    /// Returns the safe cause category retained after source sanitization.
    #[must_use]
    pub const fn cause_category(&self) -> ProtocolCauseCategory {
        self.cause
    }
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} ({})", self.kind, self.cause)
    }
}

impl Error for ProtocolError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.cause)
    }
}
