//! Raw-preserving server-sourced Attestation Nonce.

use std::fmt;

use kmipkit_ttlv::Structure;

use super::CredentialValidationError;

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
    /// missing, duplicated, or has the wrong TTLV Item Type.
    pub fn try_from_ttlv(tree: Structure) -> Result<Self, CredentialValidationError> {
        Ok(Self { tree })
    }

    /// Returns the original ordered TTLV Structure without rebuilding it.
    #[must_use]
    pub fn into_ttlv(self) -> Structure {
        self.tree
    }
}

impl fmt::Debug for Nonce {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Nonce")
            .field("field_count", &self.tree.view().children().len())
            .finish_non_exhaustive()
    }
}

impl fmt::Display for Nonce {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("KMIP Attestation Nonce (redacted)")
    }
}
