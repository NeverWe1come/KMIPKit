//! Redacted request Data values shared by the Encrypt and Decrypt models.

use std::fmt;

use kmipkit_ttlv::Value;

use crate::SecretBytes;

/// One of the three request Data encodings defined by KMIP 2.1 §7.9.
///
/// Byte strings retain the protocol crate's zeroizing owner. Formatting
/// redacts every variant because request Data may contain plaintext,
/// ciphertext, or other sensitive operation material.
#[non_exhaustive]
pub enum OperationData {
    /// A KMIP Byte String, owned and zeroized through [`SecretBytes`].
    ByteString(SecretBytes),
    /// A raw unsigned KMIP Enumeration, including values unknown to this release.
    Enumeration(u32),
    /// A signed KMIP Integer.
    Integer(i32),
}

impl OperationData {
    /// Moves this value into the corresponding owned TTLV value without
    /// changing its Item Type or value.
    #[must_use]
    pub fn into_ttlv_value(self) -> Value {
        match self {
            Self::ByteString(value) => value.into_ttlv_value(),
            Self::Enumeration(value) => Value::enumeration(value),
            Self::Integer(value) => Value::integer(value),
        }
    }
}

impl fmt::Debug for OperationData {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("OperationData([REDACTED])")
    }
}

impl fmt::Display for OperationData {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("KMIP operation data ([REDACTED])")
    }
}
