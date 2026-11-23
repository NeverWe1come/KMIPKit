//! Typed KMIP 2.1 Server-to-Client Notify request and response payloads (§6.2.2, Tables 338, 339, 345).

use std::{error::Error, fmt};

use kmipkit_ttlv::{KmipTag, Structure, ValueView};

use crate::enums::OperationCode;
use crate::{ProtocolError, RequestBatchItemView};

/// The KMIP operation code for Notify (`0x0000_001B`).
pub const NOTIFY_OPERATION: u32 = OperationCode::Notify.as_u32();
const UNIQUE_IDENTIFIER: u32 = KmipTag::UniqueIdentifier.as_u32();
const ATTRIBUTES: u32 = KmipTag::Attributes.as_u32();
const DELETED_ATTRIBUTE_REFERENCE: u32 = KmipTag::AttributeReference.as_u32();

/// A server-to-client Notify request (§6.2.2).
#[derive(Debug)]
pub struct ServerNotifyRequest {
    unique_identifier: super::ServerIdentifier,
    attributes: Option<Structure>,
    deleted_attribute_references: Vec<DeletedAttributeReference>,
}

/// Reference to a deleted standard or vendor attribute in wire order.
#[derive(Debug)]
pub enum DeletedAttributeReference {
    /// Numeric tag of a standard attribute.
    Standard(u32),
    /// Vendor identification and attribute name.
    Vendor(Structure),
}

impl ServerNotifyRequest {
    /// Creates a new Server Notify request.
    #[must_use]
    pub fn new(
        unique_identifier: String,
        attributes: Option<Structure>,
        deleted_attribute_references: Vec<Structure>,
    ) -> Self {
        Self {
            unique_identifier: super::ServerIdentifier::new(unique_identifier.into()),
            attributes,
            deleted_attribute_references: deleted_attribute_references
                .into_iter()
                .map(DeletedAttributeReference::Vendor)
                .collect(),
        }
    }

    /// Returns the Unique Identifier of the object to which the notification applies.
    #[must_use]
    pub fn unique_identifier(&self) -> &str {
        self.unique_identifier.text()
    }

    /// Returns the original identifier and TTLV type.
    pub fn identifier(&self) -> &crate::UniqueIdentifier {
        &self.unique_identifier.value
    }

    /// Returns the optional changed attributes.
    #[must_use]
    pub const fn attributes(&self) -> Option<&Structure> {
        self.attributes.as_ref()
    }

    /// Returns the deleted attribute references.
    #[must_use]
    pub fn deleted_attribute_references(&self) -> &[DeletedAttributeReference] {
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
                        UNIQUE_IDENTIFIER if uid.is_some() => return None,
                        UNIQUE_IDENTIFIER => {
                            uid = Some(super::ServerIdentifier::new(
                                field.with_value(crate::UniqueIdentifier::from_value)?,
                            ));
                        }
                        ATTRIBUTES if attrs.is_some() => return None,
                        ATTRIBUTES => {
                            attrs = Some(field.as_structure()?.try_clone().ok()?);
                        }
                        DELETED_ATTRIBUTE_REFERENCE => {
                            let reference = field.with_value(|value| match value {
                                ValueView::Enumeration(tag) => {
                                    Some(DeletedAttributeReference::Standard(*tag))
                                }
                                ValueView::Structure(fields) => fields
                                    .try_clone()
                                    .ok()
                                    .map(DeletedAttributeReference::Vendor),
                                _ => None,
                            })?;
                            deleted_refs.push(reference);
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
