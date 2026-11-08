//! Typed KMIP 2.1 Set Endpoint Role request and response payloads (§6.1.54, Tables 331–333; §6.2.5, Tables 349–351).

use std::{error::Error, fmt};

use kmipkit_ttlv::{ModelError, Structure, Value, ValueView};

use crate::asynchronous::item;
use crate::{
    KmipOperationResult, ProtocolCauseCategory, ProtocolError, ProtocolErrorKind,
    RequestBatchItemView, ResponseBatchItemView, ResultMessage, ResultReason, ResultStatus,
    ResultValidationError,
};

/// The KMIP operation code for Set Endpoint Role (`0x0000_0032`).
pub const SET_ENDPOINT_ROLE_OPERATION: u32 = 0x0000_0032;
/// The KMIP tag for Endpoint Role (`0x0042_0151`).
pub const ENDPOINT_ROLE_TAG: u32 = 0x0042_0151;
/// The KMIP tag for Vendor Identification (`0x0042_009D`).
pub const VENDOR_IDENTIFICATION_TAG: u32 = 0x0042_009D;

const ENUMERATION_EXTENSION_MIN: u32 = 0x8000_0000;
const ENUMERATION_EXTENSION_MAX: u32 = 0x8FFF_FFFF;
const SUCCESS: u32 = 0;

/// An open KMIP Endpoint Role enumeration value (§11.19, Table 451).
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct EndpointRole(u32);

impl EndpointRole {
    /// Client endpoint role (`0x0000_0001`).
    pub const CLIENT: Self = Self(1);
    /// Server endpoint role (`0x0000_0002`).
    pub const SERVER: Self = Self(2);

    /// Constructs an Endpoint Role from an arbitrary unsigned 32-bit integer.
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Returns the raw unsigned 32-bit integer.
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }

    /// Returns true if this value is an assigned standard role.
    #[must_use]
    pub const fn is_known(self) -> bool {
        matches!(self.0, 1 | 2)
    }

    /// Returns true if this value is in the extension range (`0x8000_0000..=0x8FFF_FFFF`).
    #[must_use]
    pub const fn is_extension(self) -> bool {
        self.0 >= ENUMERATION_EXTENSION_MIN && self.0 <= ENUMERATION_EXTENSION_MAX
    }

    /// Returns the normative wire name for known roles.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self.0 {
            1 => "Client",
            2 => "Server",
            _ => "Unknown",
        }
    }

    /// Parses an Endpoint Role from its exact wire name.
    #[must_use]
    pub fn from_wire_name(name: &str) -> Option<Self> {
        match name {
            "Client" => Some(Self::CLIENT),
            "Server" => Some(Self::SERVER),
            _ => None,
        }
    }
}

impl fmt::Debug for EndpointRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EndpointRole({}: {})", self.0, self.wire_name())
    }
}

impl fmt::Display for EndpointRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.wire_name())
    }
}

/// A typed KMIP 2.1 Set Endpoint Role request (§6.1.54, Table 331; §6.2.5, Table 349).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SetEndpointRoleRequest {
    endpoint_role: EndpointRole,
    vendor_identification: Option<String>,
}

fn parse_role_fields(
    payload: &kmipkit_ttlv::StructureView<'_>,
) -> (Option<EndpointRole>, Option<String>) {
    let mut endpoint_role = None;
    let mut vendor_identification = None;
    for field in payload.children() {
        match field.tag().raw() {
            ENDPOINT_ROLE_TAG => {
                if let Some(raw) = field.with_value(|v| match v {
                    ValueView::Enumeration(e) => Some(*e),
                    _ => None,
                }) {
                    endpoint_role = Some(EndpointRole::from_raw(raw));
                }
            }
            VENDOR_IDENTIFICATION_TAG => {
                if let Some(text) = field.with_value(|v| match v {
                    ValueView::TextString(s) => Some(s.to_string()),
                    _ => None,
                }) {
                    vendor_identification = Some(text);
                }
            }
            _ => {}
        }
    }
    (endpoint_role, vendor_identification)
}

impl SetEndpointRoleRequest {
    /// Creates a new Set Endpoint Role request for the specified role.
    #[must_use]
    pub const fn new(endpoint_role: EndpointRole) -> Self {
        Self {
            endpoint_role,
            vendor_identification: None,
        }
    }

    /// Sets the optional Vendor Identification text string.
    #[must_use]
    pub fn with_vendor_identification(mut self, vendor_id: impl Into<String>) -> Self {
        self.vendor_identification = Some(vendor_id.into());
        self
    }

    /// Returns the target Endpoint Role.
    #[must_use]
    pub const fn endpoint_role(&self) -> EndpointRole {
        self.endpoint_role
    }

    /// Returns the optional Vendor Identification.
    #[must_use]
    pub fn vendor_identification(&self) -> Option<&str> {
        self.vendor_identification.as_deref()
    }

    /// Encodes the request payload into an ordered Structure.
    ///
    /// # Errors
    ///
    /// Returns a `ProtocolError` if TTLV structure construction fails.
    pub fn to_ttlv_payload(&self) -> Result<Structure, ProtocolError> {
        let mut structure = Structure::new();
        structure
            .try_push(item(
                ENDPOINT_ROLE_TAG,
                Value::enumeration(self.endpoint_role.raw()),
            )?)
            .map_err(protocol_model_error)?;
        if let Some(ref vendor_id) = self.vendor_identification {
            structure
                .try_push(item(
                    VENDOR_IDENTIFICATION_TAG,
                    Value::text_string(vendor_id.clone()),
                )?)
                .map_err(protocol_model_error)?;
        }
        Ok(structure)
    }

    /// Parses a Set Endpoint Role request from a validated request batch item view (§6.2.5).
    ///
    /// # Errors
    ///
    /// Returns `SetEndpointRoleError` if the batch item is malformed or missing fields.
    pub fn try_from_request_item(
        item: RequestBatchItemView<'_>,
    ) -> Result<Self, SetEndpointRoleError> {
        if item.operation() != Some(SET_ENDPOINT_ROLE_OPERATION) {
            return Err(SetEndpointRoleError::UnexpectedOperation);
        }
        let (endpoint_role, vendor_identification) = item
            .with_request_payload(|payload| Some(parse_role_fields(&payload)))
            .ok_or(SetEndpointRoleError::MissingPayload)?
            .ok_or(SetEndpointRoleError::InvalidPayloadStructure)?;

        let endpoint_role = endpoint_role.ok_or(SetEndpointRoleError::MissingRequiredField)?;

        Ok(Self {
            endpoint_role,
            vendor_identification,
        })
    }
}

/// A typed Set Endpoint Role response item (§6.1.54, Table 332; §6.2.5, Table 350).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SetEndpointRoleResponse {
    result: KmipOperationResult,
    endpoint_role: Option<EndpointRole>,
    vendor_identification: Option<String>,
}

impl SetEndpointRoleResponse {
    /// Creates a successful response item.
    ///
    /// # Panics
    ///
    /// Panics if the internal success result construction fails.
    #[must_use]
    pub fn success(endpoint_role: EndpointRole, vendor_identification: Option<String>) -> Self {
        Self {
            result: KmipOperationResult::new(ResultStatus::from_raw(SUCCESS), None, None)
                .expect("success result without reason is valid"),
            endpoint_role: Some(endpoint_role),
            vendor_identification,
        }
    }

    /// Encodes the response payload into an ordered Structure.
    ///
    /// # Errors
    ///
    /// Returns a `ProtocolError` if TTLV structure construction fails.
    pub fn to_ttlv_payload(&self) -> Result<Structure, ProtocolError> {
        let mut structure = Structure::new();
        if let Some(role) = self.endpoint_role {
            structure
                .try_push(item(ENDPOINT_ROLE_TAG, Value::enumeration(role.raw()))?)
                .map_err(protocol_model_error)?;
        }
        if let Some(ref vendor_id) = self.vendor_identification {
            structure
                .try_push(item(
                    VENDOR_IDENTIFICATION_TAG,
                    Value::text_string(vendor_id.clone()),
                )?)
                .map_err(protocol_model_error)?;
        }
        Ok(structure)
    }

    /// Parses a Set Endpoint Role response from a validated response batch item view.
    ///
    /// # Errors
    ///
    /// Returns `SetEndpointRoleError` if the response batch item structure is malformed.
    pub fn try_from_response_item(
        item: ResponseBatchItemView<'_>,
    ) -> Result<Self, SetEndpointRoleError> {
        if item.operation() != Some(SET_ENDPOINT_ROLE_OPERATION) {
            return Err(SetEndpointRoleError::UnexpectedOperation);
        }
        let status = item
            .result_status()
            .ok_or(SetEndpointRoleError::MissingResultStatus)?;
        let result_message = item.with_result_message(|text| ResultMessage::new(text.to_owned()));
        let result = operation_result(status, item.result_reason(), result_message)?;

        if status.raw() != SUCCESS {
            return Ok(Self {
                result,
                endpoint_role: None,
                vendor_identification: None,
            });
        }

        let (endpoint_role, vendor_identification) = item
            .with_response_payload(|payload| Some(parse_role_fields(&payload)))
            .ok_or(SetEndpointRoleError::MissingPayload)?
            .ok_or(SetEndpointRoleError::InvalidPayloadStructure)?;

        let endpoint_role = endpoint_role.ok_or(SetEndpointRoleError::MissingRequiredField)?;

        Ok(Self {
            result,
            endpoint_role: Some(endpoint_role),
            vendor_identification,
        })
    }

    /// Returns the common KMIP operation result metadata.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }

    /// Returns the accepted Endpoint Role if the operation succeeded.
    #[must_use]
    pub const fn endpoint_role(&self) -> Option<EndpointRole> {
        self.endpoint_role
    }

    /// Returns the accepted Vendor Identification if present.
    #[must_use]
    pub fn vendor_identification(&self) -> Option<&str> {
        self.vendor_identification.as_deref()
    }

    /// Unwraps the accepted Endpoint Role, or returns an error if rejected.
    ///
    /// # Errors
    ///
    /// Returns `SetEndpointRoleError` if the server rejected the operation or fields were missing.
    pub fn into_result(self) -> Result<EndpointRole, SetEndpointRoleError> {
        if self.result.status().raw() == SUCCESS {
            self.endpoint_role
                .ok_or(SetEndpointRoleError::MissingRequiredField)
        } else {
            Err(SetEndpointRoleError::ServerRejected(self.result))
        }
    }
}

/// Errors returned while parsing or evaluating Set Endpoint Role operations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SetEndpointRoleError {
    /// The response batch item operation did not match Set Endpoint Role.
    UnexpectedOperation,
    /// The response batch item was missing a Result Status.
    MissingResultStatus,
    /// Invalid combination of Result Status and Result Reason.
    InvalidResultCombination(ResultValidationError),
    /// Successful operation response was missing its payload.
    MissingPayload,
    /// Malformed field type in payload structure.
    InvalidPayloadStructure,
    /// Missing required Endpoint Role field in payload.
    MissingRequiredField,
    /// The server rejected the Set Endpoint Role operation.
    ServerRejected(KmipOperationResult),
}

impl fmt::Display for SetEndpointRoleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedOperation => write!(f, "item operation was not Set Endpoint Role"),
            Self::MissingResultStatus => write!(f, "response item is missing a result status"),
            Self::InvalidResultCombination(err) => write!(f, "invalid result status/reason: {err}"),
            Self::MissingPayload => {
                write!(f, "successful Set Endpoint Role response omitted payload")
            }
            Self::InvalidPayloadStructure => {
                write!(f, "malformed Set Endpoint Role payload structure")
            }
            Self::MissingRequiredField => {
                write!(f, "response payload missing required Endpoint Role field")
            }
            Self::ServerRejected(res) => {
                write!(f, "server rejected Set Endpoint Role: {:?}", res.status())
            }
        }
    }
}

impl Error for SetEndpointRoleError {}

pub(crate) fn protocol_model_error(error: ModelError) -> ProtocolError {
    ProtocolError::new(
        ProtocolErrorKind::InvalidValue,
        ProtocolCauseCategory::InvalidValue,
        error,
    )
}

pub(crate) fn operation_result(
    status: ResultStatus,
    reason: Option<ResultReason>,
    message: Option<ResultMessage>,
) -> Result<KmipOperationResult, SetEndpointRoleError> {
    KmipOperationResult::new(status, reason, message)
        .map_err(SetEndpointRoleError::InvalidResultCombination)
}
