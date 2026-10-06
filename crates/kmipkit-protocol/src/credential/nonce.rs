//! Raw-preserving server-sourced Attestation Nonce.

use std::fmt;

use kmipkit_ttlv::{Structure, StructureView};

use super::secret;
use super::validation::{FieldKind, FieldRule};
use super::{CredentialValidationError, validation};

const NONCE_ID: u32 = 0x0042_00C9;
const NONCE_VALUE: u32 = 0x0042_00CA;

const NONCE_RULES: &[FieldRule] = &[
    FieldRule {
        tag: NONCE_ID,
        kind: FieldKind::ByteString,
        required: true,
    },
    FieldRule {
        tag: NONCE_VALUE,
        kind: FieldKind::ByteString,
        required: true,
    },
];

/// A server-sourced Attestation Nonce retaining its exact TTLV bytes.
pub struct Nonce {
    tree: Structure,
}

impl Nonce {
    /// Takes ownership of a Nonce TTLV Structure.
    ///
    /// Full member validation is implemented with the Attestation schema.
    ///
    /// # Errors
    ///
    /// Returns a payload-free validation error when a required member is
    /// missing, duplicated, out of order, or has the wrong TTLV Item Type.
    pub fn try_from_ttlv(tree: Structure) -> Result<Self, CredentialValidationError> {
        Self::validate_view(&tree.view())?;
        Ok(Self { tree })
    }

    pub(super) fn validate_view(view: &StructureView<'_>) -> Result<(), CredentialValidationError> {
        validation::validate_fields(view, NONCE_RULES, &[])
    }

    /// Lends the original Nonce structure for callback-scoped byte access.
    pub fn with_ttlv<R>(&self, callback: impl for<'a> FnOnce(StructureView<'a>) -> R) -> R {
        callback(self.tree.view())
    }

    /// Returns the original ordered TTLV Structure without rebuilding it.
    #[must_use]
    pub fn into_ttlv(self) -> Structure {
        self.tree
    }
}

impl fmt::Debug for Nonce {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        secret::format_debug_struct(formatter, "Nonce", |debug| {
            debug.field("field_count", &self.tree.view().children().len());
            Ok(())
        })
    }
}

impl fmt::Display for Nonce {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        secret::format_redacted_display(formatter, "Attestation Nonce")
    }
}
