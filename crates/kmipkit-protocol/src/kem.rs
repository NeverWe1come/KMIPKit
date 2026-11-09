//! KMIP 3.0 KEM (Key Encapsulation Mechanism) and Deactivate operation models (KMIPKIT-0071).

use kmipkit_ttlv::{Item, RawTag, Structure, Value};

/// TTLV Tag for Unique Identifier (0x00420094).
pub const UNIQUE_IDENTIFIER_TAG: u32 = 0x0042_0094;
/// TTLV Tag for Data (0x004200C2), used for encapsulated output.
pub const DATA_TAG: u32 = 0x0042_00C2;

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
///
/// The response carries the newly created Secret Data object's identifier and
/// the encapsulated output as `Data`; the shared secret remains in that
/// server-managed object. See KMIP 3.0 §6.1.22, Table 318.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncapsulateResponse {
    unique_identifier: String,
    data: Vec<u8>,
}

impl EncapsulateResponse {
    #[must_use]
    pub fn new(unique_identifier: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            unique_identifier: unique_identifier.into(),
            data,
        }
    }

    #[must_use]
    pub fn unique_identifier(&self) -> &str {
        &self.unique_identifier
    }

    #[must_use]
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Serializes to TTLV response payload structure.
    ///
    /// # Errors
    ///
    /// Returns an error if structure push fails.
    pub fn into_ttlv_payload(self) -> Result<Structure, String> {
        uid_bytes_payload(self.unique_identifier, DATA_TAG, self.data)
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
        uid_bytes_payload(self.unique_identifier, DATA_TAG, self.ciphertext)
    }
}

/// A KMIP 3.0 Decapsulate response (PQC KEM).
///
/// The response carries only the identifier of the newly created Secret Data
/// object. Its shared secret is not returned in the response payload. See
/// KMIP 3.0 §6.1.15, Table 297.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecapsulateResponse {
    unique_identifier: String,
}

impl DecapsulateResponse {
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
        single_uid_payload(self.unique_identifier)
    }
}
