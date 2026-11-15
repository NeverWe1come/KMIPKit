//! Typed KMIP 2.1 Ticket values from §7.39, Table 391.
//!
//! The model keeps the complete ordered TTLV tree so unrecognized children
//! survive parsing and re-encoding. Ticket Value bytes are held both in the
//! zeroizing generic TTLV value and in a callback-only `SecretBytes` view; both
//! owned allocations are zeroized when the Ticket is dropped.

use std::{error::Error, fmt};

use kmipkit_ttlv::{Item, RawTag, Structure, Tag, Value, ValueView};

use crate::SecretBytes;

const TICKET_TYPE_TAG: u32 = 0x0042_014A;
const TICKET_VALUE_TAG: u32 = 0x0042_014B;

/// A KMIP Ticket Type Enumeration that preserves future and vendor values.
///
/// OASIS KMIP 2.1 §11.57, Table 488 assigns Login (`1`) and the `8XXXXXXX`
/// extension range. Other raw values remain representable for lossless parsing.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TicketType(u32);

impl TicketType {
    /// Login Ticket Type (`1`), from OASIS KMIP 2.1 §11.57, Table 488.
    pub const LOGIN: Self = Self(1);

    /// Creates a Ticket Type from its raw unsigned Enumeration value.
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Returns the exact raw unsigned Enumeration value.
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }

    /// Returns the assigned Ticket Type name, including the extension range.
    #[must_use]
    pub const fn known_name(self) -> Option<&'static str> {
        match self.0 {
            1 => Some("Login"),
            0x8000_0000..=0x8FFF_FFFF => Some("Extensions"),
            _ => None,
        }
    }
}

/// A payload-free category for an invalid KMIP Ticket structure.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TicketValidationErrorKind {
    /// Ticket Type or Ticket Value appeared more than once.
    DuplicateField,
    /// Ticket Type or Ticket Value was absent.
    MissingField,
    /// A required Ticket field used the wrong TTLV Item Type.
    WrongFieldType,
    /// Ticket Value preceded Ticket Type in the known-field order.
    FieldOutOfOrder,
    /// A generic TTLV item or structure could not be constructed.
    InvalidTtlvStructure,
}

impl fmt::Display for TicketValidationErrorKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateField => formatter.write_str("duplicate KMIP Ticket field"),
            Self::MissingField => formatter.write_str("missing required KMIP Ticket field"),
            Self::WrongFieldType => {
                formatter.write_str("KMIP Ticket field has an invalid Item Type")
            }
            Self::FieldOutOfOrder => formatter.write_str("KMIP Ticket fields are out of order"),
            Self::InvalidTtlvStructure => {
                formatter.write_str("invalid generic KMIP TTLV structure")
            }
        }
    }
}

/// A sanitized Ticket validation failure that never retains payload bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TicketValidationError {
    kind: TicketValidationErrorKind,
}

impl TicketValidationError {
    const fn new(kind: TicketValidationErrorKind) -> Self {
        Self { kind }
    }

    /// Returns the payload-free validation category.
    #[must_use]
    pub const fn kind(self) -> TicketValidationErrorKind {
        self.kind
    }
}

impl fmt::Display for TicketValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(formatter)
    }
}

impl Error for TicketValidationError {}

/// A secret-safe KMIP Ticket retaining its complete ordered TTLV Structure.
///
/// Ticket Value is available only through a callback. Unknown allocated tags,
/// extension values, and original child order remain intact in the retained
/// Structure. `KMIPKit` does not install a Ticket as implicit client state.
pub struct Ticket {
    tree: Structure,
    kind: TicketType,
    value: SecretBytes,
}

impl Ticket {
    /// Creates a Ticket from an open Ticket Type and caller-owned secret bytes.
    ///
    /// The supplied secret bytes are copied into the retained TTLV tree so the
    /// model can preserve the original wire Structure while exposing a
    /// callback-scoped secret accessor. Both allocations are zeroized on drop.
    ///
    /// # Errors
    ///
    /// Returns a payload-free error only if the fixed KMIP fields cannot be
    /// represented by the generic TTLV model.
    pub fn new(
        ticket_type: TicketType,
        ticket_value: SecretBytes,
    ) -> Result<Self, TicketValidationError> {
        let mut tree = Structure::new();
        push_item(
            &mut tree,
            TICKET_TYPE_TAG,
            Value::enumeration(ticket_type.raw()),
        )?;
        let value_copy = ticket_value.with_bytes(<[u8]>::to_vec);
        push_item(&mut tree, TICKET_VALUE_TAG, Value::byte_string(value_copy))?;
        Ok(Self {
            tree,
            kind: ticket_type,
            value: ticket_value,
        })
    }

    /// Validates and takes ownership of a Ticket TTLV Structure.
    ///
    /// Ticket Type and Ticket Value must each occur once, use Enumeration and
    /// Byte String respectively, and appear in Table 391 order. Other children
    /// are retained without interpretation.
    ///
    /// # Errors
    ///
    /// Returns a sanitized error when a required field is missing, repeated,
    /// out of order, or uses an incompatible Item Type.
    pub fn try_from_ttlv(tree: Structure) -> Result<Self, TicketValidationError> {
        let mut ticket_type = None;
        let mut ticket_value = None;
        let mut last_known_field = None;

        for field in tree.view().children() {
            let index = match field.tag().raw() {
                TICKET_TYPE_TAG => 0,
                TICKET_VALUE_TAG => 1,
                _ => continue,
            };
            if last_known_field.is_some_and(|previous| index < previous) {
                return Err(TicketValidationError::new(
                    TicketValidationErrorKind::FieldOutOfOrder,
                ));
            }
            last_known_field = Some(index);

            if index == 0 {
                if ticket_type.is_some() {
                    return Err(TicketValidationError::new(
                        TicketValidationErrorKind::DuplicateField,
                    ));
                }
                ticket_type = Some(read_ticket_type(field)?);
            } else {
                if ticket_value.is_some() {
                    return Err(TicketValidationError::new(
                        TicketValidationErrorKind::DuplicateField,
                    ));
                }
                ticket_value = Some(read_ticket_value(field)?);
            }
        }

        let Some(ticket_type) = ticket_type else {
            return Err(TicketValidationError::new(
                TicketValidationErrorKind::MissingField,
            ));
        };
        let Some(ticket_value) = ticket_value else {
            return Err(TicketValidationError::new(
                TicketValidationErrorKind::MissingField,
            ));
        };

        Ok(Self {
            tree,
            kind: ticket_type,
            value: ticket_value,
        })
    }

    /// Returns the exact raw-preserving Ticket Type.
    #[must_use]
    pub const fn ticket_type(&self) -> TicketType {
        self.kind
    }

    /// Lends the Ticket Value bytes only for the duration of `callback`.
    ///
    /// The callback may copy the bytes; copies made by the caller are outside
    /// `KMIPKit`'s zeroization guarantee.
    pub fn with_ticket_value<R>(&self, callback: impl for<'a> FnOnce(&'a [u8]) -> R) -> R {
        self.value.with_bytes(callback)
    }

    /// Lends the original ordered TTLV Structure for callback-scoped access.
    pub fn with_ttlv<R>(
        &self,
        callback: impl for<'a> FnOnce(kmipkit_ttlv::StructureView<'a>) -> R,
    ) -> R {
        callback(self.tree.view())
    }

    /// Returns the original ordered TTLV Structure without rebuilding it.
    #[must_use]
    pub fn into_ttlv(self) -> Structure {
        self.tree
    }
}

impl fmt::Debug for Ticket {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Ticket")
            .field("ticket_type", &self.kind.raw())
            .field("ticket_value", &"[REDACTED]")
            .field("field_count", &self.tree.view().children().len())
            .finish_non_exhaustive()
    }
}

impl fmt::Display for Ticket {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("KMIP Ticket (redacted)")
    }
}

fn push_item(
    tree: &mut Structure,
    raw_tag: u32,
    value: Value,
) -> Result<(), TicketValidationError> {
    let tag = checked_tag(raw_tag)?;
    let item = Item::new(tag, value)
        .map_err(|_| TicketValidationError::new(TicketValidationErrorKind::InvalidTtlvStructure))?;
    tree.try_push(item)
        .map_err(|_| TicketValidationError::new(TicketValidationErrorKind::InvalidTtlvStructure))
}

fn read_ticket_type(field: &Item) -> Result<TicketType, TicketValidationError> {
    field
        .with_value(|value| match value {
            ValueView::Enumeration(raw) => Some(TicketType::from_raw(*raw)),
            _ => None,
        })
        .ok_or_else(|| TicketValidationError::new(TicketValidationErrorKind::WrongFieldType))
}

fn read_ticket_value(field: &Item) -> Result<SecretBytes, TicketValidationError> {
    field
        .with_value(|value| match value {
            ValueView::ByteString(bytes) => Some(SecretBytes::new(bytes.to_vec())),
            _ => None,
        })
        .ok_or_else(|| TicketValidationError::new(TicketValidationErrorKind::WrongFieldType))
}

fn checked_tag(raw_tag: u32) -> Result<Tag, TicketValidationError> {
    RawTag::new(raw_tag)
        .and_then(|tag| tag.try_checked())
        .map_err(|_| TicketValidationError::new(TicketValidationErrorKind::InvalidTtlvStructure))
}
