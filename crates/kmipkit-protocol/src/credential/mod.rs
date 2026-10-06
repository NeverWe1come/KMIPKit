//! Lossless typed KMIP Credential and Authentication values.

mod authentication;
mod value;

pub use authentication::{Authentication, CredentialView};
pub use value::{Credential, CredentialType};

use std::error::Error;
use std::fmt;

/// A payload-free category for an invalid Credential or Authentication value.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CredentialValidationErrorKind {
    /// Authentication contained no Credential structures.
    EmptyAuthentication,
    /// A Credential field appeared more than once.
    DuplicateField,
    /// A required Credential field was absent.
    MissingField,
    /// A Credential field used an unexpected TTLV Item Type.
    WrongFieldType,
    /// Credential Type appeared after Credential Value.
    FieldOutOfOrder,
    /// A KMIPKit-owned generic TTLV model operation failed.
    InvalidTtlvStructure,
}

impl fmt::Display for CredentialValidationErrorKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyAuthentication => {
                formatter.write_str("Authentication must contain a Credential")
            }
            Self::DuplicateField => formatter.write_str("duplicate Credential field"),
            Self::MissingField => formatter.write_str("missing required Credential field"),
            Self::WrongFieldType => {
                formatter.write_str("Credential field has an invalid Item Type")
            }
            Self::FieldOutOfOrder => formatter.write_str("Credential fields are out of order"),
            Self::InvalidTtlvStructure => formatter.write_str("invalid generic TTLV structure"),
        }
    }
}

/// A sanitized validation failure that never stores or formats credential data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CredentialValidationError {
    kind: CredentialValidationErrorKind,
}

impl CredentialValidationError {
    pub(super) const fn new(kind: CredentialValidationErrorKind) -> Self {
        Self { kind }
    }

    /// Returns the payload-free validation category.
    #[must_use]
    pub const fn kind(self) -> CredentialValidationErrorKind {
        self.kind
    }
}

impl fmt::Display for CredentialValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(formatter)
    }
}

impl Error for CredentialValidationError {}
