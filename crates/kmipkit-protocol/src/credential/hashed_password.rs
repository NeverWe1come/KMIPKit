//! Raw-preserving Hashed Password Credential Value.

use std::fmt;

use kmipkit_ttlv::{Structure, StructureView, ValueView};

const HASHING_ALGORITHM: u32 = 0x0042_0038;
const DEFAULT_HASHING_ALGORITHM_SHA256: u32 = 6;

/// A caller-supplied Hashed Password Credential Value.
pub struct HashedPasswordCredential {
    tree: Structure,
}

impl HashedPasswordCredential {
    pub(super) fn from_unvalidated_tree(tree: Structure) -> Self {
        Self { tree }
    }

    pub(super) fn field_count(&self) -> usize {
        self.tree.view().children().len()
    }

    /// Returns the explicitly supplied raw Hashing Algorithm, if present.
    #[must_use]
    pub fn hashing_algorithm_raw(&self) -> Option<u32> {
        self.tree.view().children().iter().find_map(|field| {
            if field.tag().raw() != HASHING_ALGORITHM {
                return None;
            }
            field.with_value(|value| match value {
                ValueView::Enumeration(raw) => Some(*raw),
                _ => None,
            })
        })
    }

    /// Returns the effective Hashing Algorithm without inserting an absent field.
    #[must_use]
    pub fn effective_hashing_algorithm_raw(&self) -> u32 {
        self.hashing_algorithm_raw()
            .unwrap_or(DEFAULT_HASHING_ALGORITHM_SHA256)
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

impl fmt::Debug for HashedPasswordCredential {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HashedPasswordCredential")
            .field("field_count", &self.field_count())
            .finish_non_exhaustive()
    }
}

impl fmt::Display for HashedPasswordCredential {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("KMIP Hashed Password Credential (redacted)")
    }
}
