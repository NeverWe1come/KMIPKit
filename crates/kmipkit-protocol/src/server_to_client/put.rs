//! Typed KMIP 2.1 Server-to-Client Put request and response payloads (§6.2.3, Tables 342, 343, 346).

use std::{error::Error, fmt};

use kmipkit_ttlv::{Item, Structure, ValueView};

use crate::{ProtocolError, RequestBatchItemView};

/// The KMIP operation code for Put (`0x0000_001C`).
pub const PUT_OPERATION: u32 = 0x0000_001C;
const PUT_FUNCTION: u32 = 0x0042_0070;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const REPLACED_UNIQUE_IDENTIFIER: u32 = 0x0042_0076;
const ATTRIBUTES: u32 = 0x0042_0125;

const ENUMERATION_EXTENSION_MIN: u32 = 0x8000_0000;
const ENUMERATION_EXTENSION_MAX: u32 = 0x8FFF_FFFF;

/// An open KMIP Put Function enumeration value (§11.41, Table 473).
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct PutFunction(u32);

impl PutFunction {
    /// New object (`0x0000_0001`).
    pub const NEW: Self = Self(1);
    /// Replacement object (`0x0000_0002`).
    pub const REPLACE: Self = Self(2);

    /// Constructs a Put Function from an arbitrary unsigned 32-bit integer.
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Returns the raw unsigned 32-bit integer.
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }

    /// Returns true if this value is an assigned standard Put Function.
    #[must_use]
    pub const fn is_known(self) -> bool {
        matches!(self.0, 1 | 2)
    }

    /// Returns true if this value is in the extension range.
    #[must_use]
    pub const fn is_extension(self) -> bool {
        self.0 >= ENUMERATION_EXTENSION_MIN && self.0 <= ENUMERATION_EXTENSION_MAX
    }

    /// Returns the normative wire name for known values.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self.0 {
            1 => "New",
            2 => "Replace",
            _ => "Unknown",
        }
    }
}

impl fmt::Debug for PutFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PutFunction({}: {})", self.0, self.wire_name())
    }
}

impl fmt::Display for PutFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.wire_name())
    }
}

/// A server-to-client Put request (§6.2.3).
#[derive(Debug)]
pub struct ServerPutRequest {
    put_function: PutFunction,
    unique_identifier: String,
    replaced_unique_identifier: Option<String>,
    object_item: Item,
    attributes: Option<Structure>,
}

impl ServerPutRequest {
    /// Creates a new Server Put request.
    #[must_use]
    pub const fn new(
        put_function: PutFunction,
        unique_identifier: String,
        replaced_unique_identifier: Option<String>,
        object_item: Item,
        attributes: Option<Structure>,
    ) -> Self {
        Self {
            put_function,
            unique_identifier,
            replaced_unique_identifier,
            object_item,
            attributes,
        }
    }

    /// Returns the indicated Put Function.
    #[must_use]
    pub const fn put_function(&self) -> PutFunction {
        self.put_function
    }

    /// Returns the Unique Identifier of the object being pushed.
    #[must_use]
    pub fn unique_identifier(&self) -> &str {
        &self.unique_identifier
    }

    /// Returns the Replaced Unique Identifier if Put Function is Replace.
    #[must_use]
    pub fn replaced_unique_identifier(&self) -> Option<&str> {
        self.replaced_unique_identifier.as_deref()
    }

    /// Returns the Managed Object item being pushed to the client.
    #[must_use]
    pub const fn object_item(&self) -> &Item {
        &self.object_item
    }

    /// Returns optional additional attributes pushed with the object.
    #[must_use]
    pub const fn attributes(&self) -> Option<&Structure> {
        self.attributes.as_ref()
    }

    /// Computes the effective Put Function per OASIS KMIP 2.1 §6.2.3:
    /// "In case the object with the Replaced Unique Identifier does not exist at
    /// the client, the client SHALL interpret this as if the Put Function contained
    /// the value New."
    #[must_use]
    pub fn effective_put_function(&self, replaced_object_exists_locally: bool) -> PutFunction {
        if self.put_function == PutFunction::REPLACE && !replaced_object_exists_locally {
            PutFunction::NEW
        } else {
            self.put_function
        }
    }

    /// Parses a server-to-client Put request from a request batch item.
    ///
    /// # Errors
    ///
    /// Returns `ServerPutError` if required fields are missing or payload is malformed.
    pub fn try_from_request_item(item: RequestBatchItemView<'_>) -> Result<Self, ServerPutError> {
        if item.operation() != Some(PUT_OPERATION) {
            return Err(ServerPutError::UnexpectedOperation);
        }

        let parsed = item
            .with_request_payload(|payload| {
                let mut put_fn = None;
                let mut uid = None;
                let mut replaced_uid = None;
                let mut object_item = None;
                let mut attrs = None;

                for field in payload.children() {
                    match field.tag().raw() {
                        PUT_FUNCTION => {
                            if let Some(raw) = field.with_value(|v| match v {
                                ValueView::Enumeration(e) => Some(*e),
                                _ => None,
                            }) {
                                put_fn = Some(PutFunction::from_raw(raw));
                            }
                        }
                        UNIQUE_IDENTIFIER => {
                            if let Some(text) = field.with_value(|v| match v {
                                ValueView::TextString(s) => Some(s.to_string()),
                                _ => None,
                            }) {
                                uid = Some(text);
                            }
                        }
                        REPLACED_UNIQUE_IDENTIFIER => {
                            if let Some(text) = field.with_value(|v| match v {
                                ValueView::TextString(s) => Some(s.to_string()),
                                _ => None,
                            }) {
                                replaced_uid = Some(text);
                            }
                        }
                        ATTRIBUTES => {
                            attrs = field.with_value(|v| match v {
                                ValueView::Structure(st) => st.try_clone().ok(),
                                _ => None,
                            });
                        }
                        _ => {
                            // Any other item is the Managed Object
                            if object_item.is_none() {
                                let cloned = field.try_clone().ok()?;
                                object_item = Some(cloned);
                            }
                        }
                    }
                }
                Some((put_fn, uid, replaced_uid, object_item, attrs))
            })
            .ok_or(ServerPutError::MissingPayload)?
            .ok_or(ServerPutError::MalformedPayload)?;

        let put_function = parsed.0.unwrap_or(PutFunction::NEW);
        let unique_identifier = parsed.1.ok_or(ServerPutError::MissingUniqueIdentifier)?;
        let replaced_unique_identifier = parsed.2;
        let object_item = parsed.3.ok_or(ServerPutError::MissingManagedObject)?;
        let attributes = parsed.4;

        if put_function == PutFunction::REPLACE && replaced_unique_identifier.is_none() {
            return Err(ServerPutError::MissingReplacedUniqueIdentifier);
        }

        Ok(Self {
            put_function,
            unique_identifier,
            replaced_unique_identifier,
            object_item,
            attributes,
        })
    }
}

/// A server-to-client Put response payload (§6.2.3, Table 343).
///
/// Contains no payload items ("Response Message containing no payload").
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ServerPutResponse {
    _private: (),
}

impl ServerPutResponse {
    /// Creates a new empty Put response.
    #[must_use]
    pub const fn new() -> Self {
        Self { _private: () }
    }

    /// Encodes the response payload (empty structure).
    ///
    /// # Errors
    ///
    /// Returns a `ProtocolError` if structure construction fails.
    pub fn to_ttlv_payload(&self) -> Result<Structure, ProtocolError> {
        Ok(Structure::new())
    }
}

/// Errors returned while processing Server-to-Client Put requests.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ServerPutError {
    /// Operation code in item did not match Put.
    UnexpectedOperation,
    /// Request batch item was missing a request payload.
    MissingPayload,
    /// Request payload was missing the required Unique Identifier.
    MissingUniqueIdentifier,
    /// Put Function was Replace, but Replaced Unique Identifier was omitted.
    MissingReplacedUniqueIdentifier,
    /// Request payload was missing the required Managed Object.
    MissingManagedObject,
    /// Request payload structure was malformed.
    MalformedPayload,
}

impl fmt::Display for ServerPutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedOperation => write!(f, "item operation was not Put"),
            Self::MissingPayload => write!(f, "request item missing payload"),
            Self::MissingUniqueIdentifier => {
                write!(f, "Put request omitted required Unique Identifier")
            }
            Self::MissingReplacedUniqueIdentifier => {
                write!(
                    f,
                    "Put Function Replace requires Replaced Unique Identifier"
                )
            }
            Self::MissingManagedObject => write!(f, "Put request omitted required Managed Object"),
            Self::MalformedPayload => write!(f, "malformed Put payload"),
        }
    }
}

impl Error for ServerPutError {}
