//! Raw-preserving KMIP protocol version values.

/// A KMIP Protocol Version Major/Minor pair without client policy.
///
/// The message model retains signed TTLV Integer values exactly. Version
/// acceptance and the `KMIPKit` 1.0 requirement for 2.1 belong to client
/// execution, not this type.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ProtocolVersion {
    major: i32,
    minor: i32,
}

impl ProtocolVersion {
    /// Creates a version pair from its exact signed Integer values.
    #[must_use]
    pub const fn from_raw(major: i32, minor: i32) -> Self {
        Self { major, minor }
    }

    /// Returns the exact Protocol Version Major value.
    #[must_use]
    pub const fn major(self) -> i32 {
        self.major
    }

    /// Returns the exact Protocol Version Minor value.
    #[must_use]
    pub const fn minor(self) -> i32 {
        self.minor
    }
}
