//! Raw-preserving KMIP protocol version values.

use std::fmt;

/// A KMIP Protocol Version Major/Minor pair without client policy.
///
/// The message model retains signed TTLV Integer values exactly. Version
/// acceptance and catalog policy belong to client execution and version gate,
/// not this type.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProtocolVersion {
    major: i32,
    minor: i32,
}

impl ProtocolVersion {
    /// OASIS KMIP 2.0 Standard (October 2019).
    pub const V2_0: Self = Self::from_raw(2, 0);

    /// OASIS KMIP 2.1 Standard (December 2020) — canonical default.
    pub const V2_1: Self = Self::from_raw(2, 1);

    /// OASIS KMIP 3.0 Draft/Standard.
    pub const V3_0: Self = Self::from_raw(3, 0);

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

    /// Returns whether this protocol version is recognized and supported by
    /// the compiled normative catalog.
    ///
    /// Non-catalogued versions (e.g. legacy 1.x or future unmapped versions)
    /// return `false`.
    #[must_use]
    pub const fn is_catalogued(self) -> bool {
        matches!((self.major, self.minor), (2 | 3, 0) | (2, 1))
    }
}

impl fmt::Display for ProtocolVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}", self.major, self.minor)
    }
}
