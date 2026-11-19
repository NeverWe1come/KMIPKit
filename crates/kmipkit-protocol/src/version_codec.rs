//! Version-aware encoding and decoding boundary context.

use kmipkit_ttlv::Structure;

use crate::{ProtocolError, ProtocolVersion};

/// Context parameters governing version-aware serialization and deserialization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CodecContext {
    /// The effective protocol version negotiated or selected for this message.
    pub version: ProtocolVersion,
}

impl CodecContext {
    /// Creates a codec context with the specified effective protocol version.
    #[must_use]
    pub const fn new(version: ProtocolVersion) -> Self {
        Self { version }
    }
}

/// A protocol structure capable of version-aware wire representation.
pub trait VersionedCodec: Sized {
    /// Encodes this item into a generic TTLV Structure adapted to the version in `context`.
    ///
    /// # Errors
    ///
    /// Returns [`ProtocolError`] if the structure cannot be represented under the effective version.
    fn to_versioned_ttlv(&self, context: CodecContext) -> Result<Structure, ProtocolError>;
}
