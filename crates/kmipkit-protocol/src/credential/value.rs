//! Raw-preserving Credential discriminator and TTLV value.

use std::fmt;

use kmipkit_ttlv::{Structure, StructureView, ValueView};

use super::{CredentialValidationError, CredentialValidationErrorKind};

const CREDENTIAL_TYPE: u32 = 0x0042_0024;
const CREDENTIAL_VALUE: u32 = 0x0042_0025;

/// A Credential Type, retaining standardized, extension-range, and future values.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CredentialType {
    /// Username and Password, value `1`.
    UsernameAndPassword,
    /// Device, value `2`.
    Device,
    /// Attestation, value `3`.
    Attestation,
    /// One Time Password, value `4`.
    OneTimePassword,
    /// Hashed Password, value `5`.
    HashedPassword,
    /// Ticket, value `6`.
    Ticket,
    /// Extensions, values in the OASIS `8XXXXXXX` range.
    Extensions(u32),
    /// An unassigned raw enumeration value.
    Unknown(u32),
}

impl CredentialType {
    /// Preserves a raw Credential Type enumeration as a known, extension, or future value.
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        match raw {
            1 => Self::UsernameAndPassword,
            2 => Self::Device,
            3 => Self::Attestation,
            4 => Self::OneTimePassword,
            5 => Self::HashedPassword,
            6 => Self::Ticket,
            0x8000_0000..=0x8FFF_FFFF => Self::Extensions(raw),
            value => Self::Unknown(value),
        }
    }

    /// Returns the original unsigned 32-bit Credential Type value.
    #[must_use]
    pub const fn raw(self) -> u32 {
        match self {
            Self::UsernameAndPassword => 1,
            Self::Device => 2,
            Self::Attestation => 3,
            Self::OneTimePassword => 4,
            Self::HashedPassword => 5,
            Self::Ticket => 6,
            Self::Extensions(raw) | Self::Unknown(raw) => raw,
        }
    }
}

/// A Credential retaining its complete ordered generic TTLV value.
pub struct Credential {
    tree: Structure,
    credential_type: CredentialType,
}

impl Credential {
    /// Returns the raw Credential Type value without interpreting its Credential Value.
    #[must_use]
    pub const fn credential_type(&self) -> CredentialType {
        self.credential_type
    }

    /// Returns the original raw Credential Type enumeration.
    #[must_use]
    pub const fn credential_type_raw(&self) -> u32 {
        self.credential_type.raw()
    }

    /// Lends the original ordered generic Credential structure for callback-scoped access.
    pub fn with_ttlv<R>(&self, callback: impl for<'a> FnOnce(StructureView<'a>) -> R) -> R {
        callback(self.tree.view())
    }

    pub(super) fn into_tree(self) -> Structure {
        self.tree
    }

    pub(super) fn credential_type_from_view(
        view: &StructureView<'_>,
    ) -> Result<CredentialType, CredentialValidationError> {
        let mut credential_type = None;
        let mut credential_value_seen = false;

        for field in view.children() {
            match field.tag().raw() {
                CREDENTIAL_TYPE => {
                    if credential_type.is_some() {
                        return Err(CredentialValidationError::new(
                            CredentialValidationErrorKind::DuplicateField,
                        ));
                    }
                    if credential_value_seen {
                        return Err(CredentialValidationError::new(
                            CredentialValidationErrorKind::FieldOutOfOrder,
                        ));
                    }
                    let raw = field.with_value(|value| match value {
                        ValueView::Enumeration(raw) => Some(*raw),
                        _ => None,
                    });
                    let Some(raw) = raw else {
                        return Err(CredentialValidationError::new(
                            CredentialValidationErrorKind::WrongFieldType,
                        ));
                    };
                    credential_type = Some(CredentialType::from_raw(raw));
                }
                CREDENTIAL_VALUE => {
                    if credential_value_seen {
                        return Err(CredentialValidationError::new(
                            CredentialValidationErrorKind::DuplicateField,
                        ));
                    }
                    if credential_type.is_none() {
                        return Err(CredentialValidationError::new(
                            CredentialValidationErrorKind::FieldOutOfOrder,
                        ));
                    }
                    credential_value_seen = true;
                }
                _ => {}
            }
        }

        let Some(credential_type) = credential_type else {
            return Err(CredentialValidationError::new(
                CredentialValidationErrorKind::MissingField,
            ));
        };
        if !credential_value_seen {
            return Err(CredentialValidationError::new(
                CredentialValidationErrorKind::MissingField,
            ));
        }
        Ok(credential_type)
    }
}

impl fmt::Debug for Credential {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Credential")
            .field("credential_type", &self.credential_type)
            .field("field_count", &self.tree.view().children().len())
            .finish_non_exhaustive()
    }
}

impl fmt::Display for Credential {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "KMIP Credential (type {})",
            self.credential_type.raw()
        )
    }
}
