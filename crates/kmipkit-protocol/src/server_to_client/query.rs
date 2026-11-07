//! Typed KMIP 2.1 Server-to-Client Query request and response payloads (§6.2.4, Tables 347, 348).

use std::{error::Error, fmt};

use kmipkit_ttlv::{ModelError, Structure, Value, ValueView};

use crate::asynchronous::item;
use crate::{
    ProtocolCauseCategory, ProtocolError, ProtocolErrorKind, QueryFunction, RequestBatchItemView,
};

/// The KMIP operation code for Query (`0x0000_0018`).
pub const QUERY_OPERATION: u32 = 0x0000_0018;
const QUERY_FUNCTION_TAG: u32 = 0x0042_0074;
const OPERATION_TAG: u32 = 0x0042_005C;
const OBJECT_TYPE_TAG: u32 = 0x0042_0057;
const SERVER_INFORMATION_TAG: u32 = 0x0042_0088;
const VENDOR_IDENTIFICATION_TAG: u32 = 0x0042_009D;
const EXTENSION_LIST_TAG: u32 = 0x0042_00A3;
const EXTENSION_MAP_TAG: u32 = 0x0042_00A4;

/// Server/Client capabilities queried via §6.2.4.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ServerCapabilities {
    operations: Vec<u32>,
    object_types: Vec<u32>,
    vendor_identification: Option<String>,
    extension_tags: Vec<u32>,
}

impl ServerCapabilities {
    /// Creates a new empty capabilities set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the vendor identification string.
    pub fn set_vendor_identification(&mut self, id: impl Into<String>) {
        self.vendor_identification = Some(id.into());
    }

    /// Sets the supported operations list.
    pub fn set_operations(&mut self, operations: Vec<u32>) {
        self.operations = operations;
    }

    /// Sets the supported object types list.
    pub fn set_object_types(&mut self, object_types: Vec<u32>) {
        self.object_types = object_types;
    }

    /// Sets the supported extension tags list.
    pub fn set_extension_tags(&mut self, tags: Vec<u32>) {
        self.extension_tags = tags;
    }

    /// Returns the vendor identification string, if configured.
    #[must_use]
    pub fn vendor_identification(&self) -> Option<&str> {
        self.vendor_identification.as_deref()
    }
}

/// A server-to-client Query request (§6.2.4).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServerQueryRequest {
    query_functions: Vec<QueryFunction>,
}

impl ServerQueryRequest {
    /// Creates a new Query request with the specified query functions.
    #[must_use]
    pub const fn new(query_functions: Vec<QueryFunction>) -> Self {
        Self { query_functions }
    }

    /// Returns the requested query functions.
    #[must_use]
    pub fn query_functions(&self) -> &[QueryFunction] {
        &self.query_functions
    }

    /// Parses a server-to-client Query request from a request batch item.
    ///
    /// # Errors
    ///
    /// Returns `ServerQueryError` if the operation is invalid or payload is malformed.
    pub fn try_from_request_item(item: RequestBatchItemView<'_>) -> Result<Self, ServerQueryError> {
        if item.operation() != Some(QUERY_OPERATION) {
            return Err(ServerQueryError::UnexpectedOperation);
        }

        let query_functions = item
            .with_request_payload(|payload| {
                let mut functions = Vec::new();
                for field in payload.children() {
                    if field.tag().raw() == QUERY_FUNCTION_TAG {
                        let raw = field.with_value(|v| match v {
                            ValueView::Enumeration(e) => Some(*e),
                            _ => None,
                        });
                        if let Some(r) = raw {
                            functions.push(QueryFunction::from_raw(r));
                        }
                    }
                }
                Some(functions)
            })
            .ok_or(ServerQueryError::MissingPayload)?
            .ok_or(ServerQueryError::MalformedPayload)?;

        Ok(Self { query_functions })
    }
}

/// A server-to-client Query response payload (§6.2.4).
#[derive(Debug)]
pub struct ServerQueryResponse {
    operations: Vec<u32>,
    object_types: Vec<u32>,
    server_information: Option<Structure>,
    extension_map: Vec<u32>,
    extension_list: Vec<u32>,
}

impl ServerQueryResponse {
    /// Generates a Query response based on the request and local capabilities.
    ///
    /// Normative Rule (§6.2.4): "If both Query Extension List and Query Extension
    /// Map are specified in the request, then only the response to Query Extension
    /// Map SHALL be returned and the Query Extension List SHALL be ignored."
    #[must_use]
    pub fn respond(request: &ServerQueryRequest, capabilities: &ServerCapabilities) -> Self {
        let mut operations = Vec::new();
        let mut object_types = Vec::new();
        let mut server_information = None;
        let mut has_ext_list = false;
        let mut has_ext_map = false;

        for qf in &request.query_functions {
            match *qf {
                QueryFunction::OPERATIONS => {
                    operations.clone_from(&capabilities.operations);
                }
                QueryFunction::OBJECTS => {
                    object_types.clone_from(&capabilities.object_types);
                }
                QueryFunction::SERVER_INFORMATION => {
                    if let Some(ref vendor_id) = capabilities.vendor_identification {
                        let mut s = Structure::new();
                        if let Ok(it) = item(
                            VENDOR_IDENTIFICATION_TAG,
                            Value::text_string(vendor_id.clone()),
                        ) {
                            let _ = s.try_push(it);
                        }
                        server_information = Some(s);
                    }
                }
                QueryFunction::EXTENSION_LIST => {
                    has_ext_list = true;
                }
                QueryFunction::EXTENSION_MAP => {
                    has_ext_map = true;
                }
                _ => {}
            }
        }

        let (extension_map, extension_list) = if has_ext_map {
            // Extension Map takes precedence over Extension List
            (capabilities.extension_tags.clone(), Vec::new())
        } else if has_ext_list {
            (Vec::new(), capabilities.extension_tags.clone())
        } else {
            (Vec::new(), Vec::new())
        };

        Self {
            operations,
            object_types,
            server_information,
            extension_map,
            extension_list,
        }
    }

    /// Encodes the response payload into an ordered Structure.
    ///
    /// # Errors
    ///
    /// Returns a `ProtocolError` if structure construction fails.
    pub fn to_ttlv_payload(&self) -> Result<Structure, ProtocolError> {
        let mut structure = Structure::new();

        for op in &self.operations {
            structure
                .try_push(item(OPERATION_TAG, Value::enumeration(*op))?)
                .map_err(protocol_model_error)?;
        }

        for obj in &self.object_types {
            structure
                .try_push(item(OBJECT_TYPE_TAG, Value::enumeration(*obj))?)
                .map_err(protocol_model_error)?;
        }

        if let Some(ref info) = self.server_information {
            let cloned_info = info.view().try_clone().map_err(protocol_model_error)?;
            structure
                .try_push(item(SERVER_INFORMATION_TAG, Value::structure(cloned_info))?)
                .map_err(protocol_model_error)?;
        }

        // Extension Map
        for ext in &self.extension_map {
            let val = i32::try_from(*ext).unwrap_or(0);
            structure
                .try_push(item(EXTENSION_MAP_TAG, Value::integer(val))?)
                .map_err(protocol_model_error)?;
        }

        // Extension List (only if Extension Map was not used)
        for ext in &self.extension_list {
            let val = i32::try_from(*ext).unwrap_or(0);
            structure
                .try_push(item(EXTENSION_LIST_TAG, Value::integer(val))?)
                .map_err(protocol_model_error)?;
        }

        Ok(structure)
    }
}

/// Errors returned while processing Server-to-Client Query requests.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ServerQueryError {
    /// Operation code in item did not match Query.
    UnexpectedOperation,
    /// Request batch item was missing a request payload.
    MissingPayload,
    /// Request payload structure was malformed.
    MalformedPayload,
}

impl fmt::Display for ServerQueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedOperation => write!(f, "item operation was not Query"),
            Self::MissingPayload => write!(f, "request item missing payload"),
            Self::MalformedPayload => write!(f, "malformed Query payload"),
        }
    }
}

impl Error for ServerQueryError {}

fn protocol_model_error(error: ModelError) -> ProtocolError {
    ProtocolError::new(
        ProtocolErrorKind::InvalidValue,
        ProtocolCauseCategory::InvalidValue,
        error,
    )
}
