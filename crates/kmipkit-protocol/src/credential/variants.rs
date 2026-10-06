//! Table-validated KMIP Credential Value variants.

use std::fmt;

use kmipkit_ttlv::{Structure, StructureView};

use super::{
    CredentialType, CredentialValidationError, CredentialValidationErrorKind, validation,
    validation::{FieldKind, FieldRule},
};
use super::{attestation::AttestationCredential, hashed_password::HashedPasswordCredential};

const USERNAME: u32 = 0x0042_0099;
const PASSWORD: u32 = 0x0042_00A1;
const DEVICE_IDENTIFIER: u32 = 0x0042_00A2;
const DEVICE_SERIAL_NUMBER: u32 = 0x0042_00B0;
const NETWORK_IDENTIFIER: u32 = 0x0042_00AB;
const MACHINE_IDENTIFIER: u32 = 0x0042_00A9;
const MEDIA_IDENTIFIER: u32 = 0x0042_00AA;
const ONE_TIME_PASSWORD: u32 = 0x0042_0156;
const TICKET: u32 = 0x0042_0149;
const TICKET_TYPE: u32 = 0x0042_014A;
const TICKET_VALUE: u32 = 0x0042_014B;

/// A username-and-password Credential Value retaining its exact TTLV tree.
pub struct UsernameAndPasswordCredential {
    tree: Structure,
}

/// A Device Credential Value retaining its exact TTLV tree.
pub struct DeviceCredential {
    tree: Structure,
}

/// A one-time-password Credential Value retaining its exact TTLV tree.
pub struct OneTimePasswordCredential {
    tree: Structure,
}

/// A Ticket Credential Value retaining its exact TTLV tree.
pub struct TicketCredential {
    tree: Structure,
}

/// An opaque TTLV Structure for extension and unknown Credential Types.
pub struct OpaqueTtlv {
    tree: Structure,
}

macro_rules! impl_tree_value {
    ($type:ty, $label:literal) => {
        impl $type {
            fn from_tree(tree: Structure) -> Self {
                Self { tree }
            }

            /// Lends the original ordered TTLV Structure for callback-scoped access.
            pub fn with_ttlv<R>(&self, callback: impl for<'a> FnOnce(StructureView<'a>) -> R) -> R {
                callback(self.tree.view())
            }

            /// Returns the original ordered TTLV Structure without rebuilding it.
            #[must_use]
            pub fn into_ttlv(self) -> Structure {
                self.tree
            }
        }

        impl fmt::Debug for $type {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter
                    .debug_struct($label)
                    .field("field_count", &self.tree.view().children().len())
                    .finish_non_exhaustive()
            }
        }

        impl fmt::Display for $type {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(formatter, "KMIP {} (redacted)", $label)
            }
        }
    };
}

impl_tree_value!(
    UsernameAndPasswordCredential,
    "UsernameAndPasswordCredential"
);
impl_tree_value!(DeviceCredential, "DeviceCredential");
impl_tree_value!(OneTimePasswordCredential, "OneTimePasswordCredential");
impl_tree_value!(TicketCredential, "TicketCredential");
impl_tree_value!(OpaqueTtlv, "OpaqueTtlv");

/// A known or opaque KMIP Credential Value with its original ordered TTLV tree.
#[non_exhaustive]
pub enum CredentialValue {
    /// Username and optional Password (KMIP 2.1 §9.11, Table 411).
    UsernameAndPassword(UsernameAndPasswordCredential),
    /// Device fields (KMIP 2.1 §9.11, Table 412).
    Device(DeviceCredential),
    /// Attestation (KMIP 2.1 §9.11, Table 413).
    Attestation(AttestationCredential),
    /// Username, optional Password, and One Time Password (Table 414).
    OneTimePassword(OneTimePasswordCredential),
    /// Caller-supplied hashed-password fields (Table 415).
    HashedPassword(HashedPasswordCredential),
    /// A nested Ticket structure (KMIP 2.1 §7.39, §9.11, Table 416).
    Ticket(TicketCredential),
    /// Opaque payload for the assigned Extensions Credential Type.
    Extensions(OpaqueTtlv),
    /// Opaque payload for an unassigned or future Credential Type.
    Unknown {
        /// Original raw Credential Type value.
        credential_type: u32,
        /// Original Credential Value tree.
        value: OpaqueTtlv,
    },
}

impl CredentialValue {
    /// Validates a known Credential Value schema and takes ownership of its TTLV tree.
    ///
    /// Unknown fields and their order remain in the retained tree. Extension
    /// and unknown Credential Types are preserved as opaque structures.
    ///
    /// # Errors
    ///
    /// Returns a payload-free error when a known Credential Value violates a
    /// required field, singleton, or Item Type rule.
    pub fn try_from_ttlv(
        credential_type: CredentialType,
        tree: Structure,
    ) -> Result<Self, CredentialValidationError> {
        match credential_type.raw() {
            1 => {
                validate(&tree, USERNAME_PASSWORD_RULES, &[])?;
                Ok(Self::UsernameAndPassword(
                    UsernameAndPasswordCredential::from_tree(tree),
                ))
            }
            2 => {
                validate(&tree, DEVICE_RULES, DEVICE_IDENTIFIER_GROUP)?;
                Ok(Self::Device(DeviceCredential::from_tree(tree)))
            }
            3 => Ok(Self::Attestation(AttestationCredential::try_from_ttlv(
                tree,
            )?)),
            4 => {
                validate(&tree, OTP_RULES, &[])?;
                Ok(Self::OneTimePassword(OneTimePasswordCredential::from_tree(
                    tree,
                )))
            }
            5 => Ok(Self::HashedPassword(
                HashedPasswordCredential::try_from_ttlv(tree)?,
            )),
            6 => {
                validate(&tree, TICKET_RULES, &[])?;
                validate_ticket(&tree)?;
                Ok(Self::Ticket(TicketCredential::from_tree(tree)))
            }
            raw if (0x8000_0000..=0x8FFF_FFFF).contains(&raw) => {
                Ok(Self::Extensions(OpaqueTtlv::from_tree(tree)))
            }
            raw => Ok(Self::Unknown {
                credential_type: raw,
                value: OpaqueTtlv::from_tree(tree),
            }),
        }
    }

    /// Returns the retained Credential Value TTLV tree without rebuilding it.
    #[must_use]
    pub fn into_ttlv(self) -> Structure {
        match self {
            Self::UsernameAndPassword(value) => value.into_ttlv(),
            Self::Device(value) => value.into_ttlv(),
            Self::Attestation(value) => value.into_ttlv(),
            Self::OneTimePassword(value) => value.into_ttlv(),
            Self::HashedPassword(value) => value.into_ttlv(),
            Self::Ticket(value) => value.into_ttlv(),
            Self::Extensions(value) | Self::Unknown { value, .. } => value.into_ttlv(),
        }
    }
}

impl fmt::Debug for CredentialValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (variant, field_count) = match self {
            Self::UsernameAndPassword(value) => {
                ("UsernameAndPassword", value.tree.view().children().len())
            }
            Self::Device(value) => ("Device", value.tree.view().children().len()),
            Self::Attestation(value) => ("Attestation", value.field_count()),
            Self::OneTimePassword(value) => ("OneTimePassword", value.tree.view().children().len()),
            Self::HashedPassword(value) => ("HashedPassword", value.field_count()),
            Self::Ticket(value) => ("Ticket", value.tree.view().children().len()),
            Self::Extensions(value) => ("Extensions", value.tree.view().children().len()),
            Self::Unknown { value, .. } => ("Unknown", value.tree.view().children().len()),
        };
        formatter
            .debug_struct("CredentialValue")
            .field("variant", &variant)
            .field("field_count", &field_count)
            .finish_non_exhaustive()
    }
}

impl fmt::Display for CredentialValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("KMIP Credential Value (redacted)")
    }
}

const USERNAME_PASSWORD_RULES: &[FieldRule] = &[
    FieldRule {
        tag: USERNAME,
        kind: FieldKind::TextString,
        required: true,
    },
    FieldRule {
        tag: PASSWORD,
        kind: FieldKind::TextString,
        required: false,
    },
];

const DEVICE_RULES: &[FieldRule] = &[
    FieldRule {
        tag: DEVICE_SERIAL_NUMBER,
        kind: FieldKind::TextString,
        required: false,
    },
    FieldRule {
        tag: PASSWORD,
        kind: FieldKind::TextString,
        required: false,
    },
    FieldRule {
        tag: DEVICE_IDENTIFIER,
        kind: FieldKind::TextString,
        required: false,
    },
    FieldRule {
        tag: NETWORK_IDENTIFIER,
        kind: FieldKind::TextString,
        required: false,
    },
    FieldRule {
        tag: MACHINE_IDENTIFIER,
        kind: FieldKind::TextString,
        required: false,
    },
    FieldRule {
        tag: MEDIA_IDENTIFIER,
        kind: FieldKind::TextString,
        required: false,
    },
];

const DEVICE_IDENTIFIER_GROUP: &[u32] = &[
    DEVICE_SERIAL_NUMBER,
    NETWORK_IDENTIFIER,
    MACHINE_IDENTIFIER,
    MEDIA_IDENTIFIER,
];

const OTP_RULES: &[FieldRule] = &[
    FieldRule {
        tag: USERNAME,
        kind: FieldKind::TextString,
        required: true,
    },
    FieldRule {
        tag: PASSWORD,
        kind: FieldKind::TextString,
        required: false,
    },
    FieldRule {
        tag: ONE_TIME_PASSWORD,
        kind: FieldKind::TextString,
        required: true,
    },
];

const TICKET_RULES: &[FieldRule] = &[FieldRule {
    tag: TICKET,
    kind: FieldKind::Structure,
    required: true,
}];

const TICKET_INNER_RULES: &[FieldRule] = &[
    FieldRule {
        tag: TICKET_TYPE,
        kind: FieldKind::Enumeration,
        required: true,
    },
    FieldRule {
        tag: TICKET_VALUE,
        kind: FieldKind::ByteString,
        required: true,
    },
];

fn validate(
    tree: &Structure,
    rules: &[FieldRule],
    at_least_one: &[u32],
) -> Result<(), CredentialValidationError> {
    validation::validate_fields(&tree.view(), rules, at_least_one)
}

fn validate_ticket(tree: &Structure) -> Result<(), CredentialValidationError> {
    for field in tree.view().children() {
        if field.tag().raw() != TICKET {
            continue;
        }
        let result = field.with_value(|value| match value {
            kmipkit_ttlv::ValueView::Structure(ticket) => {
                validation::validate_fields(&ticket, TICKET_INNER_RULES, &[])
            }
            _ => Err(CredentialValidationError::new(
                CredentialValidationErrorKind::WrongFieldType,
            )),
        });
        result?;
    }
    Ok(())
}
