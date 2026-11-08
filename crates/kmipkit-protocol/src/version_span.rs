//! Normative validity span metadata for protocol elements across KMIP versions.

use crate::ProtocolVersion;

/// The normative lifespan of a KMIP protocol element across versions.
///
/// An element is supported if `version >= introduced`, has not been removed,
/// and is catalogued.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VersionSpan {
    /// The earliest KMIP specification version where this element was defined.
    pub introduced: ProtocolVersion,
    /// The KMIP specification version where this element was marked deprecated, if any.
    pub deprecated: Option<ProtocolVersion>,
    /// The KMIP specification version where this element was formally removed, if any.
    pub removed: Option<ProtocolVersion>,
}

impl VersionSpan {
    /// Creates a span introduced in `introduced` that has not been deprecated or removed.
    #[must_use]
    pub const fn since(introduced: ProtocolVersion) -> Self {
        Self {
            introduced,
            deprecated: None,
            removed: None,
        }
    }

    /// Returns whether this element is valid and supported in the given protocol version.
    #[must_use]
    pub const fn is_supported_in(&self, version: ProtocolVersion) -> bool {
        if (version.major() < self.introduced.major())
            || (version.major() == self.introduced.major()
                && version.minor() < self.introduced.minor())
        {
            return false;
        }

        if let Some(removed) = self.removed
            && ((version.major() > removed.major())
                || (version.major() == removed.major() && version.minor() >= removed.minor()))
        {
            return false;
        }

        true
    }
}
