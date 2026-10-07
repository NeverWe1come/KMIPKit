//! Safe, categorized errors for local protocol processing.

use std::error::Error;
use std::fmt;

/// A safe protocol failure category that contains no caller-provided text.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProtocolErrorKind {
    /// An extension identity is invalid or violates a local identity invariant.
    InvalidIdentity,
    /// A registry key conflicts with another declared key.
    DuplicateKey,
    /// A declared compatibility range does not support this client.
    CompatibilityMismatch,
    /// An extension schema or value does not satisfy its declared rules.
    InvalidSchema,
    /// A configured or encountered resource limit has been exceeded.
    ResourceLimit,
    /// A public operation received an invalid local input.
    InvalidInput,
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
            Self::InvalidIdentity => formatter.write_str("invalid extension identity"),
            Self::DuplicateKey => formatter.write_str("duplicate extension key"),
            Self::CompatibilityMismatch => formatter.write_str("extension compatibility mismatch"),
            Self::InvalidSchema => formatter.write_str("invalid extension schema"),
            Self::ResourceLimit => formatter.write_str("extension resource limit exceeded"),
            Self::InvalidInput => formatter.write_str("invalid protocol input"),
            Self::InvalidValue => formatter.write_str("invalid protocol value"),
            Self::MalformedMessage => formatter.write_str("malformed protocol message"),
            Self::UnsupportedValue => formatter.write_str("unsupported protocol value"),
        }
    }
}

/// A safe cause category retained after an arbitrary source is discarded.
#[non_exhaustive]
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
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtocolError {
    kind: ProtocolErrorKind,
    cause: ProtocolCauseCategory,
    tag_path: Option<Vec<kmipkit_ttlv::Tag>>,
}

impl ProtocolError {
    /// Sanitizes and discards `source`, retaining only safe categories.
    pub fn new<E>(kind: ProtocolErrorKind, cause: ProtocolCauseCategory, source: E) -> Self
    where
        E: Error + 'static,
    {
        drop(source);
        Self {
            kind,
            cause,
            tag_path: None,
        }
    }

    /// Creates an error with a safe structural tag path and no untrusted source.
    pub(crate) fn with_tag_path(
        kind: ProtocolErrorKind,
        cause: ProtocolCauseCategory,
        tag_path: Vec<kmipkit_ttlv::Tag>,
    ) -> Self {
        Self {
            kind,
            cause,
            tag_path: Some(tag_path),
        }
    }

    /// Creates a categorized error without retaining any caller-provided text.
    pub(crate) const fn categorized(kind: ProtocolErrorKind, cause: ProtocolCauseCategory) -> Self {
        Self {
            kind,
            cause,
            tag_path: None,
        }
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
        write!(formatter, "{}", self.kind)?;
        if let Some(tag_path) = &self.tag_path {
            formatter.write_str(" at tag path ")?;
            fmt::Debug::fmt(tag_path, formatter)?;
        }
        write!(formatter, " ({})", self.cause)
    }
}

impl Error for ProtocolError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.cause)
    }
}
