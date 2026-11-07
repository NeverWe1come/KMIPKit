//! KMIP 3.0 KEM (Key Encapsulation Mechanism) and Deactivate operation models (KMIPKIT-0071).

use kmipkit_ttlv::{Item, RawTag, Structure, Value};

/// TTLV Tag for Unique Identifier (0x00420094).
pub const UNIQUE_IDENTIFIER_TAG: u32 = 0x0042_0094;
/// TTLV Tag for Ciphertext (0x004200C2).
pub const CIPHERTEXT_TAG: u32 = 0x0042_00C2;
/// TTLV Tag for Shared Secret (0x00420043).
pub const SHARED_SECRET_TAG: u32 = 0x0042_0043;

fn ttlv_item(raw_tag: u32, value: Value) -> Result<Item, String> {
    let tag = RawTag::new(raw_tag)
        .map_err(|e| e.to_string())?
        .try_checked()
        .map_err(|e| e.to_string())?;
    Item::new(tag, value).map_err(|e| e.to_string())
}

/// A KMIP 3.0 Deactivate request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeactivateRequest {
    unique_identifier: String,
}

impl DeactivateRequest {
    #[must_use]
    pub fn new(unique_identifier: impl Into<String>) -> Self {
        Self {
            unique_identifier: unique_identifier.into(),
        }
    }

    #[must_use]
    pub fn unique_identifier(&self) -> &str {
        &self.unique_identifier
    }

    /// Serializes to TTLV request payload structure.
    ///
    /// # Errors
    ///
    /// Returns an error if structure push fails.
    pub fn into_ttlv_payload(self) -> Result<Structure, String> {
        let mut s = Structure::new();
        s.try_push(ttlv_item(
            UNIQUE_IDENTIFIER_TAG,
            Value::text_string(self.unique_identifier),
        )?)
        .map_err(|e| e.to_string())?;
        Ok(s)
    }
}

/// A KMIP 3.0 Deactivate response.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeactivateResponse {
    unique_identifier: String,
}

impl DeactivateResponse {
    #[must_use]
    pub fn new(unique_identifier: impl Into<String>) -> Self {
        Self {
            unique_identifier: unique_identifier.into(),
        }
    }

    #[must_use]
    pub fn unique_identifier(&self) -> &str {
        &self.unique_identifier
    }

    /// Serializes to TTLV response payload structure.
    ///
    /// # Errors
    ///
    /// Returns an error if structure push fails.
    pub fn into_ttlv_payload(self) -> Result<Structure, String> {
        let mut s = Structure::new();
        s.try_push(ttlv_item(
            UNIQUE_IDENTIFIER_TAG,
            Value::text_string(self.unique_identifier),
        )?)
        .map_err(|e| e.to_string())?;
        Ok(s)
    }
}

/// A KMIP 3.0 Encapsulate request (PQC KEM).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncapsulateRequest {
    unique_identifier: String,
}

impl EncapsulateRequest {
    #[must_use]
    pub fn new(unique_identifier: impl Into<String>) -> Self {
        Self {
            unique_identifier: unique_identifier.into(),
        }
    }

    #[must_use]
    pub fn unique_identifier(&self) -> &str {
        &self.unique_identifier
    }

    /// Serializes to TTLV request payload structure.
    ///
    /// # Errors
    ///
    /// Returns an error if structure push fails.
    pub fn into_ttlv_payload(self) -> Result<Structure, String> {
        let mut s = Structure::new();
        s.try_push(ttlv_item(
            UNIQUE_IDENTIFIER_TAG,
            Value::text_string(self.unique_identifier),
        )?)
        .map_err(|e| e.to_string())?;
        Ok(s)
    }
}

/// A KMIP 3.0 Encapsulate response (PQC KEM).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncapsulateResponse {
    unique_identifier: String,
    ciphertext: Vec<u8>,
    shared_secret: Vec<u8>,
}

impl EncapsulateResponse {
    #[must_use]
    pub fn new(
        unique_identifier: impl Into<String>,
        ciphertext: Vec<u8>,
        shared_secret: Vec<u8>,
    ) -> Self {
        Self {
            unique_identifier: unique_identifier.into(),
            ciphertext,
            shared_secret,
        }
    }

    #[must_use]
    pub fn unique_identifier(&self) -> &str {
        &self.unique_identifier
    }

    #[must_use]
    pub fn ciphertext(&self) -> &[u8] {
        &self.ciphertext
    }

    #[must_use]
    pub fn shared_secret(&self) -> &[u8] {
        &self.shared_secret
    }

    /// Serializes to TTLV response payload structure.
    ///
    /// # Errors
    ///
    /// Returns an error if structure push fails.
    pub fn into_ttlv_payload(self) -> Result<Structure, String> {
        let mut s = Structure::new();
        s.try_push(ttlv_item(
            UNIQUE_IDENTIFIER_TAG,
            Value::text_string(self.unique_identifier),
        )?)
        .map_err(|e| e.to_string())?;
        s.try_push(ttlv_item(
            CIPHERTEXT_TAG,
            Value::byte_string(self.ciphertext),
        )?)
        .map_err(|e| e.to_string())?;
        s.try_push(ttlv_item(
            SHARED_SECRET_TAG,
            Value::byte_string(self.shared_secret),
        )?)
        .map_err(|e| e.to_string())?;
        Ok(s)
    }
}

/// A KMIP 3.0 Decapsulate request (PQC KEM).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecapsulateRequest {
    unique_identifier: String,
    ciphertext: Vec<u8>,
}

impl DecapsulateRequest {
    #[must_use]
    pub fn new(unique_identifier: impl Into<String>, ciphertext: Vec<u8>) -> Self {
        Self {
            unique_identifier: unique_identifier.into(),
            ciphertext,
        }
    }

    #[must_use]
    pub fn unique_identifier(&self) -> &str {
        &self.unique_identifier
    }

    #[must_use]
    pub fn ciphertext(&self) -> &[u8] {
        &self.ciphertext
    }

    /// Serializes to TTLV request payload structure.
    ///
    /// # Errors
    ///
    /// Returns an error if structure push fails.
    pub fn into_ttlv_payload(self) -> Result<Structure, String> {
        let mut s = Structure::new();
        s.try_push(ttlv_item(
            UNIQUE_IDENTIFIER_TAG,
            Value::text_string(self.unique_identifier),
        )?)
        .map_err(|e| e.to_string())?;
        s.try_push(ttlv_item(
            CIPHERTEXT_TAG,
            Value::byte_string(self.ciphertext),
        )?)
        .map_err(|e| e.to_string())?;
        Ok(s)
    }
}

/// A KMIP 3.0 Decapsulate response (PQC KEM).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecapsulateResponse {
    unique_identifier: String,
    shared_secret: Vec<u8>,
}

impl DecapsulateResponse {
    #[must_use]
    pub fn new(unique_identifier: impl Into<String>, shared_secret: Vec<u8>) -> Self {
        Self {
            unique_identifier: unique_identifier.into(),
            shared_secret,
        }
    }

    #[must_use]
    pub fn unique_identifier(&self) -> &str {
        &self.unique_identifier
    }

    #[must_use]
    pub fn shared_secret(&self) -> &[u8] {
        &self.shared_secret
    }

    /// Serializes to TTLV response payload structure.
    ///
    /// # Errors
    ///
    /// Returns an error if structure push fails.
    pub fn into_ttlv_payload(self) -> Result<Structure, String> {
        let mut s = Structure::new();
        s.try_push(ttlv_item(
            UNIQUE_IDENTIFIER_TAG,
            Value::text_string(self.unique_identifier),
        )?)
        .map_err(|e| e.to_string())?;
        s.try_push(ttlv_item(
            SHARED_SECRET_TAG,
            Value::byte_string(self.shared_secret),
        )?)
        .map_err(|e| e.to_string())?;
        Ok(s)
    }
}
