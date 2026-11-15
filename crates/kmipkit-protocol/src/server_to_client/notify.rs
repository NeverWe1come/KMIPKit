//! Typed KMIP 2.1 Server-to-Client Notify request and response payloads (§6.2.2, Tables 338, 339, 345).

use std::{error::Error, fmt};

use kmipkit_ttlv::{Structure, ValueView};

use crate::{ProtocolError, RequestBatchItemView};

/// The KMIP operation code for Notify (`0x0000_001B`).
pub const NOTIFY_OPERATION: u32 = 0x0000_001B;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const ATTRIBUTES: u32 = 0x0042_0125;
const DELETED_ATTRIBUTE_REFERENCE: u32 = 0x0042_0126;

/// A server-to-client Notify request (§6.2.2).
#[derive(Debug)]
pub struct ServerNotifyRequest {
    unique_identifier: String,
    attributes: Option<Structure>,
    deleted_attribute_references: Vec<Structure>,
}

impl ServerNotifyRequest {
    /// Creates a new Server Notify request.
    #[must_use]
    pub const fn new(
        unique_identifier: String,
        attributes: Option<Structure>,
        deleted_attribute_references: Vec<Structure>,
    ) -> Self {
        Self {
            unique_identifier,
            attributes,
            deleted_attribute_references,
        }
    }

    /// Returns the Unique Identifier of the object to which the notification applies.
    #[must_use]
    pub fn unique_identifier(&self) -> &str {
        &self.unique_identifier
    }

    /// Returns the optional changed attributes.
    #[must_use]
    pub const fn attributes(&self) -> Option<&Structure> {
        self.attributes.as_ref()
    }

    /// Returns the deleted attribute references.
    #[must_use]
    pub fn deleted_attribute_references(&self) -> &[Structure] {
        &self.deleted_attribute_references
    }

    /// Parses a server-to-client Notify request from a request batch item.
    ///
    /// # Errors
    ///
    /// Returns `ServerNotifyError` if the operation is wrong, Unique Identifier is missing, or payload is malformed.
    pub fn try_from_request_item(
        item: RequestBatchItemView<'_>,
    ) -> Result<Self, ServerNotifyError> {
        if item.operation() != Some(NOTIFY_OPERATION) {
            return Err(ServerNotifyError::UnexpectedOperation);
        }

        let parsed = item
            .with_request_payload(|payload| {
                let mut uid = None;
                let mut attrs = None;
                let mut deleted_refs = Vec::new();

                for field in payload.children() {
                    match field.tag().raw() {
                        UNIQUE_IDENTIFIER => {
                            if let Some(text) = field.with_value(|v| match v {
                                ValueView::TextString(s) => Some(s.to_string()),
                                _ => None,
                            }) {
                                uid = Some(text);
                            }
                        }
                        ATTRIBUTES => {
                            attrs = field.with_value(|v| match v {
                                ValueView::Structure(st) => st.try_clone().ok(),
                                _ => None,
                            });
                        }
                        DELETED_ATTRIBUTE_REFERENCE => {
                            if let Some(r) = field.with_value(|v| match v {
                                ValueView::Structure(st) => st.try_clone().ok(),
                                _ => None,
                            }) {
                                deleted_refs.push(r);
                            }
                        }
                        _ => {}
                    }
                }
                Some((uid, attrs, deleted_refs))
            })
            .ok_or(ServerNotifyError::MissingPayload)?
            .ok_or(ServerNotifyError::MalformedPayload)?;

        let unique_identifier = parsed.0.ok_or(ServerNotifyError::MissingUniqueIdentifier)?;

        Ok(Self {
            unique_identifier,
            attributes: parsed.1,
            deleted_attribute_references: parsed.2,
        })
    }
}

/// A server-to-client Notify response payload (§6.2.2, Table 339).
///
/// Contains no payload items ("Response containing no payload").
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ServerNotifyResponse {
    _private: (),
}

impl ServerNotifyResponse {
    /// Creates a new empty Notify response.
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

/// Errors returned while processing Server-to-Client Notify requests.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ServerNotifyError {
    /// Operation code in item did not match Notify.
    UnexpectedOperation,
    /// Request batch item was missing a request payload.
    MissingPayload,
    /// Request payload was missing the required Unique Identifier.
    MissingUniqueIdentifier,
    /// Request payload structure was malformed.
    MalformedPayload,
}

impl fmt::Display for ServerNotifyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedOperation => write!(f, "item operation was not Notify"),
            Self::MissingPayload => write!(f, "request item missing payload"),
            Self::MissingUniqueIdentifier => {
                write!(f, "Notify request omitted required Unique Identifier")
            }
            Self::MalformedPayload => write!(f, "malformed Notify payload"),
        }
    }
}

impl Error for ServerNotifyError {}
