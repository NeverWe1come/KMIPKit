//! Typed KMIP 2.1 Server-to-Client Discover Versions request and response payloads (§6.2.1, Tables 343, 344).

use std::{error::Error, fmt};

use kmipkit_ttlv::{ModelError, Structure, Value, ValueView};

use crate::asynchronous::item;
use crate::{
    ProtocolCauseCategory, ProtocolError, ProtocolErrorKind, ProtocolVersion, RequestBatchItemView,
};

/// The KMIP operation code for Discover Versions (`0x0000_001E`).
pub const DISCOVER_VERSIONS_OPERATION: u32 = 0x0000_001E;
const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;

/// A server-to-client Discover Versions request (§6.2.1).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ServerDiscoverVersionsRequest {
    protocol_versions: Vec<ProtocolVersion>,
}

impl ServerDiscoverVersionsRequest {
    /// Creates a request from a list of protocol versions in server preference order.
    #[must_use]
    pub const fn new(protocol_versions: Vec<ProtocolVersion>) -> Self {
        Self { protocol_versions }
    }

    /// Returns the optional protocol versions requested by the server.
    #[must_use]
    pub fn protocol_versions(&self) -> &[ProtocolVersion] {
        &self.protocol_versions
    }

    /// Parses a server-to-client Discover Versions request from a request batch item.
    ///
    /// # Errors
    ///
    /// Returns `ServerDiscoverVersionsError` if the operation does not match or fields are invalid.
    pub fn try_from_request_item(
        item: RequestBatchItemView<'_>,
    ) -> Result<Self, ServerDiscoverVersionsError> {
        if item.operation() != Some(DISCOVER_VERSIONS_OPERATION) {
            return Err(ServerDiscoverVersionsError::UnexpectedOperation);
        }

        let versions = item
            .with_request_payload(|payload| {
                let mut versions = Vec::new();
                for field in payload.children() {
                    if field.tag().raw() == PROTOCOL_VERSION {
                        let pv = field.with_value(|v| match v {
                            ValueView::Structure(s) => parse_protocol_version(&s),
                            _ => None,
                        })?;
                        versions.push(pv);
                    }
                }
                Some(versions)
            })
            .ok_or(ServerDiscoverVersionsError::MissingPayload)?
            .ok_or(ServerDiscoverVersionsError::MalformedProtocolVersion)?;

        Ok(Self {
            protocol_versions: versions,
        })
    }
}

/// A server-to-client Discover Versions response payload (§6.2.1).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ServerDiscoverVersionsResponse {
    protocol_versions: Vec<ProtocolVersion>,
}

impl ServerDiscoverVersionsResponse {
    /// Creates a response containing the accepted protocol versions.
    #[must_use]
    pub const fn new(protocol_versions: Vec<ProtocolVersion>) -> Self {
        Self { protocol_versions }
    }

    /// Negotiates the version list according to OASIS KMIP 2.1 §6.2.1:
    /// - If the request contains versions: returns only the intersection, ordered by client preference.
    /// - If the request omitted versions: returns all versions supported by the client.
    /// - If no intersection: returns an empty list.
    #[must_use]
    pub fn negotiate(
        request: &ServerDiscoverVersionsRequest,
        client_supported: &[ProtocolVersion],
    ) -> Self {
        if request.protocol_versions.is_empty() {
            Self {
                protocol_versions: client_supported.to_vec(),
            }
        } else {
            let intersection: Vec<ProtocolVersion> = client_supported
                .iter()
                .filter(|v| request.protocol_versions.contains(v))
                .copied()
                .collect();
            Self {
                protocol_versions: intersection,
            }
        }
    }

    /// Returns the negotiated protocol versions in client preference order.
    #[must_use]
    pub fn protocol_versions(&self) -> &[ProtocolVersion] {
        &self.protocol_versions
    }

    /// Encodes the response payload into an ordered Structure.
    ///
    /// # Errors
    ///
    /// Returns a `ProtocolError` if TTLV structure construction fails.
    pub fn to_ttlv_payload(&self) -> Result<Structure, ProtocolError> {
        let mut structure = Structure::new();
        for version in &self.protocol_versions {
            let mut version_fields = Structure::new();
            version_fields
                .try_push(item(
                    PROTOCOL_VERSION_MAJOR,
                    Value::integer(version.major()),
                )?)
                .map_err(protocol_model_error)?;
            version_fields
                .try_push(item(
                    PROTOCOL_VERSION_MINOR,
                    Value::integer(version.minor()),
                )?)
                .map_err(protocol_model_error)?;
            structure
                .try_push(item(PROTOCOL_VERSION, Value::structure(version_fields))?)
                .map_err(protocol_model_error)?;
        }
        Ok(structure)
    }
}

fn parse_protocol_version(structure: &kmipkit_ttlv::StructureView<'_>) -> Option<ProtocolVersion> {
    let mut major = None;
    let mut minor = None;

    for field in structure.children() {
        match field.tag().raw() {
            PROTOCOL_VERSION_MAJOR => {
                if let Some(val) = field.with_value(|v| match v {
                    ValueView::Integer(i) => Some(*i),
                    _ => None,
                }) {
                    major = Some(val);
                }
            }
            PROTOCOL_VERSION_MINOR => {
                if let Some(val) = field.with_value(|v| match v {
                    ValueView::Integer(i) => Some(*i),
                    _ => None,
                }) {
                    minor = Some(val);
                }
            }
            _ => {}
        }
    }

    match (major, minor) {
        (Some(maj), Some(min)) => Some(ProtocolVersion::from_raw(maj, min)),
        _ => None,
    }
}

/// Errors returned while processing Server-to-Client Discover Versions requests.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ServerDiscoverVersionsError {
    /// Operation code in item did not match Discover Versions.
    UnexpectedOperation,
    /// Request batch item was missing a request payload.
    MissingPayload,
    /// Protocol Version structure inside payload was malformed.
    MalformedProtocolVersion,
}

impl fmt::Display for ServerDiscoverVersionsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedOperation => write!(f, "item operation was not Discover Versions"),
            Self::MissingPayload => write!(f, "request item missing payload"),
            Self::MalformedProtocolVersion => write!(f, "malformed Protocol Version in payload"),
        }
    }
}

impl Error for ServerDiscoverVersionsError {}

fn protocol_model_error(error: ModelError) -> ProtocolError {
    ProtocolError::new(
        ProtocolErrorKind::InvalidValue,
        ProtocolCauseCategory::InvalidValue,
        error,
    )
}
