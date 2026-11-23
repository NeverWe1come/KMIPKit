//! Typed KMIP 2.0, 2.1, and 3.0 draft Server-to-Client Query payloads (§6.2.4).

use std::{error::Error, fmt, sync::Arc};

use kmipkit_ttlv::{ItemType, KmipTag, ModelError, Structure, Value, ValueView};

use crate::asynchronous::item;
use crate::enums::OperationCode;
use crate::extension::{self, ExtensionInformation};
use crate::{
    ProtocolCauseCategory, ProtocolError, ProtocolErrorKind, ProtocolVersion, QueryFunction,
    RequestBatchItemView,
};

/// The KMIP operation code for Query (`0x0000_0018`).
pub const QUERY_OPERATION: u32 = OperationCode::Query.as_u32();
const QUERY_FUNCTION_TAG: u32 = KmipTag::QueryFunction.as_u32();
const OPERATION_TAG: u32 = KmipTag::Operation.as_u32();
const OBJECT_TYPE_TAG: u32 = KmipTag::ObjectType.as_u32();
const SERVER_INFORMATION_TAG: u32 = KmipTag::ServerInformation.as_u32();
const VENDOR_IDENTIFICATION_TAG: u32 = KmipTag::VendorIdentification.as_u32();
const EXTENSION_INFORMATION_TAG: u32 = KmipTag::ExtensionInformation.as_u32();

#[derive(Clone, Debug)]
struct CapabilityField {
    function: QueryFunction,
    item: Arc<kmipkit_ttlv::Item>,
}

impl PartialEq for CapabilityField {
    fn eq(&self, other: &Self) -> bool {
        self.function == other.function
            && self.item.tag() == other.item.tag()
            && crate::create_key_pair::values_equal(&self.item, &other.item)
    }
}

impl Eq for CapabilityField {}

/// Server/Client capabilities queried via §6.2.4.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ServerCapabilities {
    operations: Vec<u32>,
    object_types: Vec<u32>,
    vendor_identification: Option<String>,
    extension_information: Vec<ExtensionInformation>,
    legacy_extension_tags: Vec<u32>,
    additional_fields: Vec<CapabilityField>,
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

    /// Replaces the advertised extension metadata with an ordered list.
    pub fn set_extension_information(&mut self, information: Vec<ExtensionInformation>) {
        self.extension_information = information;
        self.legacy_extension_tags.clear();
    }

    /// Advertises server information when requested by the peer.
    ///
    /// # Errors
    /// Returns an error if the value cannot be represented as TTLV.
    pub fn set_server_information(&mut self, value: Structure) -> Result<(), ProtocolError> {
        let item = item(KmipTag::ServerInformation.as_u32(), Value::structure(value))?;
        self.additional_fields
            .retain(|f| f.function != QueryFunction::SERVER_INFORMATION);
        self.additional_fields.push(CapabilityField {
            function: QueryFunction::SERVER_INFORMATION,
            item: Arc::new(item),
        });
        Ok(())
    }

    /// Advertises attestation type when requested by the peer.
    ///
    /// # Errors
    /// Returns an error if the value cannot be represented as TTLV.
    pub fn add_attestation_type(&mut self, value: u32) -> Result<(), ProtocolError> {
        let item = item(KmipTag::AttestationType.as_u32(), Value::enumeration(value))?;
        self.additional_fields.push(CapabilityField {
            function: QueryFunction::ATTESTATION_TYPES,
            item: Arc::new(item),
        });
        Ok(())
    }

    /// Advertises rng parameters when requested by the peer.
    ///
    /// # Errors
    /// Returns an error if the value cannot be represented as TTLV.
    pub fn add_rng_parameters(&mut self, value: Structure) -> Result<(), ProtocolError> {
        let item = item(KmipTag::RNGParameters.as_u32(), Value::structure(value))?;
        self.additional_fields.push(CapabilityField {
            function: QueryFunction::RNGS,
            item: Arc::new(item),
        });
        Ok(())
    }

    /// Advertises profile information when requested by the peer.
    ///
    /// # Errors
    /// Returns an error if the value cannot be represented as TTLV.
    pub fn add_profile_information(&mut self, value: Structure) -> Result<(), ProtocolError> {
        let item = item(
            KmipTag::ProfileInformation.as_u32(),
            Value::structure(value),
        )?;
        self.additional_fields.push(CapabilityField {
            function: QueryFunction::PROFILES,
            item: Arc::new(item),
        });
        Ok(())
    }

    /// Advertises validation information when requested by the peer.
    ///
    /// # Errors
    /// Returns an error if the value cannot be represented as TTLV.
    pub fn add_validation_information(&mut self, value: Structure) -> Result<(), ProtocolError> {
        let item = item(
            KmipTag::ValidationInformation.as_u32(),
            Value::structure(value),
        )?;
        self.additional_fields.push(CapabilityField {
            function: QueryFunction::VALIDATIONS,
            item: Arc::new(item),
        });
        Ok(())
    }

    /// Advertises capability information when requested by the peer.
    ///
    /// # Errors
    /// Returns an error if the value cannot be represented as TTLV.
    pub fn add_capability_information(&mut self, value: Structure) -> Result<(), ProtocolError> {
        let item = item(
            KmipTag::CapabilityInformation.as_u32(),
            Value::structure(value),
        )?;
        self.additional_fields.push(CapabilityField {
            function: QueryFunction::CAPABILITIES,
            item: Arc::new(item),
        });
        Ok(())
    }

    /// Advertises client registration method when requested by the peer.
    ///
    /// # Errors
    /// Returns an error if the value cannot be represented as TTLV.
    pub fn add_client_registration_method(&mut self, value: u32) -> Result<(), ProtocolError> {
        let item = item(
            KmipTag::ClientRegistrationMethod.as_u32(),
            Value::enumeration(value),
        )?;
        self.additional_fields.push(CapabilityField {
            function: QueryFunction::CLIENT_REGISTRATION_METHODS,
            item: Arc::new(item),
        });
        Ok(())
    }

    /// Reads additional Query capabilities from their named, standard TTLV fields.
    ///
    /// # Errors
    /// Returns an error for an unexpected field, duplicate singleton, or incorrect type.
    pub fn from_query_metadata(metadata: Structure) -> Result<Self, ProtocolError> {
        let mut result = Self::default();
        for field in metadata.view().children() {
            let function = match KmipTag::from_u32(field.tag().raw()) {
                Some(KmipTag::ServerInformation) => QueryFunction::SERVER_INFORMATION,
                Some(KmipTag::AttestationType) => QueryFunction::ATTESTATION_TYPES,
                Some(KmipTag::RNGParameters) => QueryFunction::RNGS,
                Some(KmipTag::ProfileInformation) => QueryFunction::PROFILES,
                Some(KmipTag::ValidationInformation) => QueryFunction::VALIDATIONS,
                Some(KmipTag::CapabilityInformation) => QueryFunction::CAPABILITIES,
                Some(KmipTag::ClientRegistrationMethod) => {
                    QueryFunction::CLIENT_REGISTRATION_METHODS
                }
                _ => {
                    return Err(invalid_extension_information(
                        "unsupported Query capability",
                    ));
                }
            };
            let enumeration = matches!(
                function,
                QueryFunction::ATTESTATION_TYPES | QueryFunction::CLIENT_REGISTRATION_METHODS
            );
            if !field.with_value(|value| {
                if enumeration {
                    matches!(value, ValueView::Enumeration(_))
                } else {
                    matches!(value, ValueView::Structure(_))
                }
            }) {
                return Err(invalid_extension_information(
                    "incorrect Query capability type",
                ));
            }
            if function == QueryFunction::SERVER_INFORMATION
                && result
                    .additional_fields
                    .iter()
                    .any(|value| value.function == function)
            {
                return Err(invalid_extension_information("duplicate Query capability"));
            }
            result.additional_fields.push(CapabilityField {
                function,
                item: Arc::new(field.try_clone().map_err(protocol_model_error)?),
            });
        }
        Ok(result)
    }

    /// Legacy setter retained for source compatibility.
    ///
    /// A non-empty list lacks the metadata required to answer either extension query.
    /// Use [`Self::set_extension_information`] to advertise extensions.
    #[deprecated(note = "use set_extension_information with KMIP Extension Information records")]
    pub fn set_extension_tags(&mut self, tags: Vec<u32>) {
        self.extension_information.clear();
        self.legacy_extension_tags = tags;
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
                        let raw = field.with_value(|value| match value {
                            ValueView::Enumeration(raw) => Some(*raw),
                            _ => None,
                        })?;
                        if !matches!(raw, 1..=3 | 5..=12 | 0x8000_0000..=u32::MAX) {
                            return None;
                        }
                        functions.push(QueryFunction::from_raw(raw));
                    }
                }
                (!functions.is_empty()).then_some(functions)
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
    query_server_information: bool,
    vendor_identification: Option<String>,
    fields: Vec<CapabilityField>,
    extensions: Vec<ExtensionInformation>,
    extension_query: Option<ExtensionQuery>,
    has_legacy_extension_tags: bool,
}

#[derive(Clone, Copy, Debug)]
enum ExtensionQuery {
    List,
    Map,
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
        let query_server_information = request
            .query_functions
            .contains(&QueryFunction::SERVER_INFORMATION);
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
                QueryFunction::EXTENSION_LIST => {
                    has_ext_list = true;
                }
                QueryFunction::EXTENSION_MAP => {
                    has_ext_map = true;
                }
                _ => {}
            }
        }

        let extension_query = if has_ext_map {
            Some(ExtensionQuery::Map)
        } else if has_ext_list {
            Some(ExtensionQuery::List)
        } else {
            None
        };

        Self {
            operations,
            object_types,
            query_server_information,
            vendor_identification: capabilities.vendor_identification.clone(),
            fields: capabilities
                .additional_fields
                .iter()
                .filter(|f| request.query_functions.contains(&f.function))
                .cloned()
                .collect(),
            extensions: extension_query
                .map_or_else(Vec::new, |_| capabilities.extension_information.clone()),
            extension_query,
            has_legacy_extension_tags: !capabilities.legacy_extension_tags.is_empty(),
        }
    }

    /// Adds requested capabilities for which the handler did not supply an explicit value.
    pub fn supply_missing_metadata(
        &mut self,
        request: &ServerQueryRequest,
        capabilities: &ServerCapabilities,
    ) {
        let mut fields = capabilities
            .additional_fields
            .iter()
            .filter(|field| {
                request.query_functions.contains(&field.function)
                    && !self
                        .fields
                        .iter()
                        .any(|current| current.function == field.function)
            })
            .cloned()
            .collect::<Vec<_>>();
        self.fields.append(&mut fields);
    }

    /// Encodes the response payload using KMIP 2.1.
    ///
    /// # Errors
    ///
    /// Returns a protocol error if response metadata is invalid or encoding fails.
    pub fn to_ttlv_payload(&self) -> Result<Structure, ProtocolError> {
        self.to_ttlv_payload_for_version(ProtocolVersion::V2_1)
    }

    /// Encodes a response for the request's protocol version.
    ///
    /// # Errors
    ///
    /// Returns a protocol error if metadata is missing, a mapped type is unavailable
    /// in the selected version, or structure construction fails.
    pub fn to_ttlv_payload_for_version(
        &self,
        version: ProtocolVersion,
    ) -> Result<Structure, ProtocolError> {
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

        if self.query_server_information {
            let vendor = self.vendor_identification.as_ref().ok_or_else(|| {
                ProtocolError::categorized(
                    ProtocolErrorKind::InvalidInput,
                    ProtocolCauseCategory::InvalidValue,
                )
            })?;
            structure
                .try_push(item(
                    VENDOR_IDENTIFICATION_TAG,
                    Value::text_string(vendor.clone()),
                )?)
                .map_err(protocol_model_error)?;
            let server = if let Some(info) = self
                .fields
                .iter()
                .find(|field| field.function == QueryFunction::SERVER_INFORMATION)
            {
                info.item.try_clone().map_err(protocol_model_error)?
            } else {
                item(SERVER_INFORMATION_TAG, Value::structure(Structure::new()))?
            };
            structure.try_push(server).map_err(protocol_model_error)?;
        }

        if let Some(query) = self.extension_query {
            if self.has_legacy_extension_tags {
                return Err(invalid_extension_information(
                    "extension names and types must be supplied as metadata",
                ));
            }
            for info in &self.extensions {
                let info = match query {
                    ExtensionQuery::List => extension::extension_information(info.name())?,
                    ExtensionQuery::Map => {
                        validate_extension_mapping(info, version)?;
                        info.clone()
                    }
                };
                let fields = extension::to_ttlv(info)?;
                structure
                    .try_push(item(EXTENSION_INFORMATION_TAG, Value::structure(fields))?)
                    .map_err(protocol_model_error)?;
            }
        }

        // Table order is independent of the order in which capabilities were configured.
        for function in [
            QueryFunction::ATTESTATION_TYPES,
            QueryFunction::RNGS,
            QueryFunction::PROFILES,
            QueryFunction::VALIDATIONS,
            QueryFunction::CAPABILITIES,
            QueryFunction::CLIENT_REGISTRATION_METHODS,
        ] {
            for field in self
                .fields
                .iter()
                .filter(|field| field.function == function)
            {
                structure
                    .try_push(field.item.try_clone().map_err(protocol_model_error)?)
                    .map_err(protocol_model_error)?;
            }
        }
        Ok(structure)
    }
}

fn validate_extension_mapping(
    information: &ExtensionInformation,
    version: ProtocolVersion,
) -> Result<(), ProtocolError> {
    if !information
        .tag()
        .is_some_and(|tag| (0x0054_0000..=0x0054_FFFF).contains(&tag))
    {
        return Err(invalid_extension_information(
            "extension map requires a tag in the extension range",
        ));
    }
    match information.item_type() {
        None => Err(invalid_extension_information(
            "extension map requires an item type",
        )),
        Some(ItemType::Identifier | ItemType::Reference | ItemType::NameReference)
            if version < ProtocolVersion::V3_0 =>
        {
            Err(invalid_extension_information(
                "extension item type requires KMIP 3.0",
            ))
        }
        Some(_) => Ok(()),
    }
}

fn invalid_extension_information(message: &'static str) -> ProtocolError {
    ProtocolError::new(
        ProtocolErrorKind::InvalidSchema,
        ProtocolCauseCategory::InvalidValue,
        std::io::Error::new(std::io::ErrorKind::InvalidInput, message),
    )
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

pub(crate) fn protocol_model_error(error: ModelError) -> ProtocolError {
    ProtocolError::new(
        ProtocolErrorKind::InvalidValue,
        ProtocolCauseCategory::InvalidValue,
        error,
    )
}
