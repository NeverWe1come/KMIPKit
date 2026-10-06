//! Raw-preserving Attestation Credential Value.

use std::fmt;

use kmipkit_ttlv::{Structure, StructureView};

/// An Attestation Credential Value retaining its exact source TTLV tree.
pub struct AttestationCredential {
    tree: Structure,
}

impl AttestationCredential {
    pub(super) fn from_unvalidated_tree(tree: Structure) -> Self {
        Self { tree }
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
