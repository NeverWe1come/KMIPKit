//! Raw-preserving Attestation Credential Value.

use std::fmt;

use kmipkit_ttlv::{Structure, StructureView, ValueView};

use super::validation::{FieldKind, FieldRule};
use super::{CredentialValidationError, CredentialValidationErrorKind, nonce::Nonce, validation};

const NONCE: u32 = 0x0042_00C8;
const ATTESTATION_TYPE: u32 = 0x0042_00C7;
const ATTESTATION_MEASUREMENT: u32 = 0x0042_00CB;
const ATTESTATION_ASSERTION: u32 = 0x0042_00CC;

const ATTESTATION_RULES: &[FieldRule] = &[
    FieldRule {
        tag: NONCE,
        kind: FieldKind::Structure,
        required: true,
    },
    FieldRule {
        tag: ATTESTATION_TYPE,
        kind: FieldKind::Enumeration,
        required: true,
    },
    FieldRule {
        tag: ATTESTATION_MEASUREMENT,
        kind: FieldKind::ByteString,
        required: false,
    },
    FieldRule {
        tag: ATTESTATION_ASSERTION,
        kind: FieldKind::ByteString,
        required: false,
    },
];

const ATTESTATION_EVIDENCE_GROUP: &[u32] = &[ATTESTATION_MEASUREMENT, ATTESTATION_ASSERTION];

/// An Attestation Credential Value retaining its exact source TTLV tree.
pub struct AttestationCredential {
    tree: Structure,
}

impl AttestationCredential {
    /// Validates the Attestation structure and its server-sourced Nonce.
    ///
    /// Both Measurement and Assertion may be present; at least one is
    /// required. Unknown Attestation Type enumeration values are retained.
    ///
    /// # Errors
    ///
    /// Returns a sanitized error for missing, duplicated, incorrectly typed,
    /// or noncanonically ordered known members, including malformed Nonce
    /// fields.
    pub fn try_from_ttlv(tree: Structure) -> Result<Self, CredentialValidationError> {
        validation::validate_fields(&tree.view(), ATTESTATION_RULES, ATTESTATION_EVIDENCE_GROUP)?;
        validate_nonce(&tree)?;
        Ok(Self { tree })
    }

    pub(super) fn field_count(&self) -> usize {
        self.tree.view().children().len()
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

fn validate_nonce(tree: &Structure) -> Result<(), CredentialValidationError> {
    for field in tree.view().children() {
        if field.tag().raw() != NONCE {
            continue;
        }
        field.with_value(|value| match value {
            ValueView::Structure(nonce) => Nonce::validate_view(&nonce),
            _ => Err(CredentialValidationError::new(
                CredentialValidationErrorKind::WrongFieldType,
            )),
        })?;
    }
    Ok(())
}

impl fmt::Debug for AttestationCredential {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AttestationCredential")
            .field("field_count", &self.field_count())
            .finish_non_exhaustive()
    }
}

impl fmt::Display for AttestationCredential {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("KMIP Attestation Credential (redacted)")
    }
}
