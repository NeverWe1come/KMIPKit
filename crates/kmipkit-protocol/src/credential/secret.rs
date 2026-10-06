//! Redacted owners for caller-provided credential text and bytes.

use std::fmt;

use kmipkit_ttlv::Value;
use zeroize::Zeroize;

/// Owns sensitive UTF-8 text and zeroizes it when dropped.
pub struct SecretText(Secret<String>);

/// Owns sensitive bytes and zeroizes them when dropped.
pub struct SecretBytes(Secret<Vec<u8>>);

struct Secret<T: Zeroize>(T);

pub(super) fn format_debug_struct(
    formatter: &mut fmt::Formatter<'_>,
    name: &str,
    fields: impl FnOnce(&mut fmt::DebugStruct<'_, '_>) -> fmt::Result,
) -> fmt::Result {
    let mut debug = formatter.debug_struct(name);
    fields(&mut debug)?;
    debug.finish_non_exhaustive()
}

pub(super) fn format_redacted_display(
    formatter: &mut fmt::Formatter<'_>,
    name: &str,
) -> fmt::Result {
    write!(formatter, "KMIP {name} (redacted)")
}

impl<T: Zeroize> Drop for Secret<T> {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl<T: Zeroize + Default> Secret<T> {
    fn take(&mut self) -> T {
        std::mem::take(&mut self.0)
    }
}

impl SecretText {
    /// Takes ownership of caller-provided UTF-8 text without cloning it.
    #[must_use]
    pub fn new(value: String) -> Self {
        Self(Secret(value))
    }

    /// Lends the text for the duration of `callback`.
    ///
    /// The callback can deliberately copy the contents; any such copy is
    /// outside `KMIPKit`'s zeroization guarantee.
    pub fn with_str<R>(&self, callback: impl for<'a> FnOnce(&'a str) -> R) -> R {
        callback(&self.0.0)
    }

    /// Moves the owned allocation into a redacted, zeroizing TTLV Text String.
    #[must_use]
    pub fn into_ttlv_value(mut self) -> Value {
        Value::text_string(self.0.take())
    }
}

impl SecretBytes {
    /// Takes ownership of caller-provided bytes without cloning them.
    #[must_use]
    pub fn new(value: Vec<u8>) -> Self {
        Self(Secret(value))
    }

    /// Lends the bytes for the duration of `callback`.
    ///
    /// The callback can deliberately copy the contents; any such copy is
    /// outside `KMIPKit`'s zeroization guarantee.
    pub fn with_bytes<R>(&self, callback: impl for<'a> FnOnce(&'a [u8]) -> R) -> R {
        callback(&self.0.0)
    }

    /// Moves the owned allocation into a redacted, zeroizing TTLV Byte String.
    #[must_use]
    pub fn into_ttlv_value(mut self) -> Value {
        Value::byte_string(self.0.take())
    }
}

impl fmt::Debug for SecretText {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SecretText([REDACTED])")
    }
}

impl fmt::Display for SecretText {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("[REDACTED]")
    }
}

impl fmt::Debug for SecretBytes {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SecretBytes([REDACTED])")
    }
}

impl fmt::Display for SecretBytes {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("[REDACTED]")
    }
}

#[cfg(test)]
#[path = "../../tests/unit/credential_secret_tests.rs"]
mod tests;
