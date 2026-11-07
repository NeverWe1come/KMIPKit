//! KMIP 3.0 Object Class model (KMIPKIT-0071).

/// An OASIS KMIP 3.0 Object Class (§5.2).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum ObjectClass {
    /// A cryptographic key, certificate, or secret object.
    CryptographicObject = 1,
    /// A system object such as User, Group, or Credential.
    SystemObject = 2,
    /// A metadata or policy object.
    MetadataObject = 3,
}

impl ObjectClass {
    /// Creates an Object Class from raw value if valid.
    #[must_use]
    pub const fn from_raw(raw: u32) -> Option<Self> {
        match raw {
            1 => Some(Self::CryptographicObject),
            2 => Some(Self::SystemObject),
            3 => Some(Self::MetadataObject),
            _ => None,
        }
    }

    /// Returns the raw integer value of this Object Class.
    #[must_use]
    pub const fn to_raw(self) -> u32 {
        self as u32
    }
}
