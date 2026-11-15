//! Lossless conversions between typed credential wrappers and generic TTLV.

use kmipkit_ttlv::Structure;

use super::{Authentication, Credential, CredentialValidationError};

impl Credential {
    /// Validates the outer Credential fields and takes ownership of the original TTLV tree.
    ///
    /// Unknown children and the Credential Value subtree remain opaque and in
    /// their original order. Variant-specific validation is provided by the
    /// corresponding typed constructors.
    ///
    /// # Errors
    ///
    /// Returns a payload-free error if Credential Type or Credential Value is
    /// missing, duplicated, out of order, or uses an invalid TTLV Item Type.
    pub fn try_from_ttlv(tree: Structure) -> Result<Self, CredentialValidationError> {
        Self::from_tree(tree)
    }

    /// Returns the original ordered Credential TTLV tree without rebuilding or copying it.
    #[must_use]
    pub fn into_ttlv(self) -> Structure {
        self.into_tree()
    }
}

impl Authentication {
    /// Returns the original ordered Authentication TTLV tree without rebuilding or copying it.
    #[must_use]
    pub fn into_ttlv(self) -> Structure {
        self.into_tree()
    }
}
