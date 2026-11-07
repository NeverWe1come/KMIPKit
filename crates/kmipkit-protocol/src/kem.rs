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

fn single_uid_payload(uid: String) -> Result<Structure, String> {
    let mut s = Structure::new();
    s.try_push(ttlv_item(UNIQUE_IDENTIFIER_TAG, Value::text_string(uid))?)
        .map_err(|e| e.to_string())?;
    Ok(s)
}

fn uid_bytes_payload(uid: String, tag: u32, bytes: Vec<u8>) -> Result<Structure, String> {
    let mut s = single_uid_payload(uid)?;
    s.try_push(ttlv_item(tag, Value::byte_string(bytes))?)
        .map_err(|e| e.to_string())?;
    Ok(s)
}

macro_rules! define_uid_message {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct $name {
            unique_identifier: String,
        }

        impl $name {
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

            /// Serializes to TTLV payload structure.
            ///
            /// # Errors
            ///
            /// Returns an error if structure push fails.
            pub fn into_ttlv_payload(self) -> Result<Structure, String> {
                single_uid_payload(self.unique_identifier)
            }
        }
    };
}

define_uid_message!(DeactivateRequest, "A KMIP 3.0 Deactivate request.");
define_uid_message!(DeactivateResponse, "A KMIP 3.0 Deactivate response.");
define_uid_message!(
    EncapsulateRequest,
    "A KMIP 3.0 Encapsulate request (PQC KEM)."
);

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
        let mut s = uid_bytes_payload(self.unique_identifier, CIPHERTEXT_TAG, self.ciphertext)?;
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
        uid_bytes_payload(self.unique_identifier, CIPHERTEXT_TAG, self.ciphertext)
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
        uid_bytes_payload(
            self.unique_identifier,
            SHARED_SECRET_TAG,
            self.shared_secret,
        )
    }
}
