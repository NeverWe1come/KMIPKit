//! Raw-preserving KMIP 3.0 Object Class values.

/// A KMIP Object Class Enumeration value.
///
/// KMIP 3.0 §11.34, Table 581 assigns `User` and `System`. Unknown values are
/// retained so future editions and vendor extensions round-trip unchanged.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ObjectClass(u32);

impl ObjectClass {
    /// KMIP 3.0 User Object Class (§11.34, Table 581).
    pub const USER: Self = Self(1);
    /// KMIP 3.0 System Object Class (§11.34, Table 581).
    pub const SYSTEM: Self = Self(2);

    /// KMIP 3.0 User Object Class alias.
    #[allow(non_upper_case_globals)]
    pub const User: Self = Self::USER;
    /// KMIP 3.0 System Object Class alias.
    #[allow(non_upper_case_globals)]
    pub const System: Self = Self::SYSTEM;

    /// Creates an Object Class while preserving its raw Enumeration value.
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Returns the exact raw Enumeration value.
    #[must_use]
    pub const fn to_raw(self) -> u32 {
        self.0
    }
}
