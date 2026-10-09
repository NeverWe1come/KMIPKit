//! Typed KMIP 2.1 Query request and response payloads.

use std::{error::Error, fmt};

use kmipkit_ttlv::{Item, ModelError, RawTag, Structure, StructureView, Tag, Value, ValueView};

use crate::{
    KmipOperationResult, ProtocolCauseCategory, ProtocolError, ProtocolErrorKind,
    ResponseBatchItemView, ResultMessage, ResultValidationError,
};

const QUERY_OPERATION: u32 = 0x0000_0018;
const QUERY_FUNCTION: u32 = 0x0042_0074;
const OBJECT_GROUPS: u32 = 0x0042_0166;
const OBJECT_GROUP: u32 = 0x0042_0056;
const SUCCESS: u32 = 0;
const PENDING: u32 = 2;
const ENUMERATION_EXTENSION_MIN: u32 = 0x8000_0000;
const ENUMERATION_EXTENSION_MAX: u32 = 0x8fff_ffff;

const OPERATION: u32 = 0x0042_005c;
const OBJECT_TYPE: u32 = 0x0042_0057;
const VENDOR_IDENTIFICATION: u32 = 0x0042_009d;
const SERVER_INFORMATION: u32 = 0x0042_0088;
const APPLICATION_NAMESPACE: u32 = 0x0042_0003;
const EXTENSION_INFORMATION: u32 = 0x0042_00a4;
const ATTESTATION_TYPE: u32 = 0x0042_00c7;
const RNG_PARAMETERS: u32 = 0x0042_00d9;
const PROFILE_INFORMATION: u32 = 0x0042_00eb;
const VALIDATION_INFORMATION: u32 = 0x0042_00df;
const CAPABILITY_INFORMATION: u32 = 0x0042_00f7;
const CLIENT_REGISTRATION_METHOD: u32 = 0x0042_00f6;
const DEFAULTS_INFORMATION: u32 = 0x0042_0152;
const PROTECTION_STORAGE_MASKS: u32 = 0x0042_015f;

/// An open KMIP Query Function enumeration value from §11.44, Table 476.
///
/// Unknown values remain representable. Query request encoding accepts the
/// fourteen assigned standard values and the KMIP Enumeration extension range.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct QueryFunction(u32);

impl QueryFunction {
    /// Query Operations (1).
    pub const OPERATIONS: Self = Self(1);
    /// Query Objects (2).
    pub const OBJECTS: Self = Self(2);
    /// Query Server Information (3).
    pub const SERVER_INFORMATION: Self = Self(3);
    /// Query Application Namespaces (4).
    pub const APPLICATION_NAMESPACES: Self = Self(4);
    /// Query Extension List (5).
    pub const EXTENSION_LIST: Self = Self(5);
    /// Query Extension Map (6).
    pub const EXTENSION_MAP: Self = Self(6);
    /// Query Attestation Types (7).
    pub const ATTESTATION_TYPES: Self = Self(7);
    /// Query RNGs (8).
    pub const RNGS: Self = Self(8);
    /// Query Validations (9).
    pub const VALIDATIONS: Self = Self(9);
    /// Query Profiles (10).
    pub const PROFILES: Self = Self(10);
    /// Query Capabilities (11).
    pub const CAPABILITIES: Self = Self(11);
    /// Query Client Registration Methods (12).
    pub const CLIENT_REGISTRATION_METHODS: Self = Self(12);
    /// Query Defaults Information (13).
    pub const DEFAULTS_INFORMATION: Self = Self(13);
    /// Query Storage Protection Masks (14).
    pub const STORAGE_PROTECTION_MASKS: Self = Self(14);

    /// Constructs a Query Function from its exact raw Enumeration value.
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Returns the exact raw unsigned Enumeration value.
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }

    const fn is_valid_request_value(self) -> bool {
        (self.0 >= 1 && self.0 <= 14)
            || (self.0 >= ENUMERATION_EXTENSION_MIN && self.0 <= ENUMERATION_EXTENSION_MAX)
    }
}

impl fmt::Debug for QueryFunction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("QueryFunction")
            .field(&self.0)
            .finish()
    }
}

/// A typed Query request payload from §6.1.40, Table 282.
pub struct QueryRequest {
    functions: Vec<QueryFunction>,
    object_groups: Option<Vec<String>>,
}

impl QueryRequest {
    /// Creates a Query request with the supplied ordered Query Functions.
    ///
    /// The list is validated when encoded so an empty request fails through
    /// the normal client execution path before transmission.
    #[must_use]
    pub fn new(functions: impl IntoIterator<Item = QueryFunction>) -> Self {
        Self {
            functions: functions.into_iter().collect(),
            object_groups: None,
        }
    }

    /// Sets the optional Object Groups structure, preserving empty input,
    /// caller order, and duplicate Object Group text values.
    #[must_use]
    pub fn with_object_groups<I, S>(mut self, object_groups: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.object_groups = Some(object_groups.into_iter().map(Into::into).collect());
        self
    }

    /// Returns the ordered Query Functions supplied by the caller.
    #[must_use]
    pub fn functions(&self) -> &[QueryFunction] {
        &self.functions
    }

    /// Returns the optional ordered Object Group text values.
    #[must_use]
    pub fn object_groups(&self) -> Option<&[String]> {
        self.object_groups.as_deref()
    }

    /// Builds the Request Payload Structure from §6.1.40, Table 282.
    ///
    /// # Errors
    ///
    /// Returns a sanitized error for an empty function list, a reserved Query
    /// Function value, or an invalid local TTLV model.
    pub fn to_ttlv_payload(&self) -> Result<Structure, ProtocolError> {
        if self.functions.is_empty()
            || self
                .functions
                .iter()
                .any(|function| !function.is_valid_request_value())
        {
            return Err(ProtocolError::categorized(
                ProtocolErrorKind::InvalidInput,
                ProtocolCauseCategory::InvalidValue,
            ));
        }

        let mut payload = Structure::new();
        for function in &self.functions {
            payload
                .try_push(item(QUERY_FUNCTION, Value::enumeration(function.raw()))?)
                .map_err(model_error)?;
        }
        if let Some(object_groups) = &self.object_groups {
            let mut groups = Structure::new();
            for group in object_groups {
                groups
                    .try_push(item(OBJECT_GROUP, Value::text_string(group.clone()))?)
                    .map_err(model_error)?;
            }
            payload
                .try_push(item(OBJECT_GROUPS, Value::structure(groups))?)
                .map_err(model_error)?;
        }
        Ok(payload)
    }
}

impl fmt::Debug for QueryRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("QueryRequest")
            .field("function_count", &self.functions.len())
            .field("object_groups_present", &self.object_groups.is_some())
            .field(
                "object_group_count",
                &self.object_groups.as_ref().map(Vec::len),
            )
            .finish()
    }
}

/// A typed Table 283 Query response member.
///
/// Enumeration and text fields are exposed as values. Complex and unknown
/// members retain their complete TTLV Item trees.
pub enum QueryResponseField {
    /// One supported or unknown Operation enumeration value.
    Operation(u32),
    /// One supported or unknown Object Type enumeration value.
    ObjectType(u32),
    /// The optional Vendor Identification text value.
    VendorIdentification(String),
    /// Vendor-specific Server Information structure.
    ServerInformation(Item),
    /// One Application Namespace text value.
    ApplicationNamespace(String),
    /// One Extension Information structure.
    ExtensionInformation(Item),
    /// One supported or unknown Attestation Type enumeration value.
    AttestationType(u32),
    /// One RNG Parameters structure.
    RngParameters(Item),
    /// One Profile Information structure.
    ProfileInformation(Item),
    /// One Validation Information structure.
    ValidationInformation(Item),
    /// One Capability Information structure.
    CapabilityInformation(Item),
    /// One supported or unknown Client Registration Method enumeration value.
    ClientRegistrationMethod(u32),
    /// Defaults Information structure.
    DefaultsInformation(Item),
    /// Required Protection Storage Masks structure in structured responses.
    ProtectionStorageMasks(Item),
    /// A structurally valid unknown or extension response Item.
    Unknown(Item),
}

impl fmt::Debug for QueryResponseField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Operation(_) => "Operation",
            Self::ObjectType(_) => "ObjectType",
            Self::VendorIdentification(_) => "VendorIdentification",
            Self::ServerInformation(_) => "ServerInformation",
            Self::ApplicationNamespace(_) => "ApplicationNamespace",
            Self::ExtensionInformation(_) => "ExtensionInformation",
            Self::AttestationType(_) => "AttestationType",
            Self::RngParameters(_) => "RngParameters",
            Self::ProfileInformation(_) => "ProfileInformation",
            Self::ValidationInformation(_) => "ValidationInformation",
            Self::CapabilityInformation(_) => "CapabilityInformation",
            Self::ClientRegistrationMethod(_) => "ClientRegistrationMethod",
            Self::DefaultsInformation(_) => "DefaultsInformation",
            Self::ProtectionStorageMasks(_) => "ProtectionStorageMasks",
            Self::Unknown(_) => "Unknown",
        };
        formatter.write_str(name)
    }
}

impl QueryResponseField {
    /// Returns the KMIP tag identifying this response member.
    #[must_use]
    pub const fn tag(&self) -> u32 {
        match self {
            Self::Operation(_) => OPERATION,
            Self::ObjectType(_) => OBJECT_TYPE,
            Self::VendorIdentification(_) => VENDOR_IDENTIFICATION,
            Self::ServerInformation(_) => SERVER_INFORMATION,
            Self::ApplicationNamespace(_) => APPLICATION_NAMESPACE,
            Self::ExtensionInformation(_) => EXTENSION_INFORMATION,
            Self::AttestationType(_) => ATTESTATION_TYPE,
            Self::RngParameters(_) => RNG_PARAMETERS,
            Self::ProfileInformation(_) => PROFILE_INFORMATION,
            Self::ValidationInformation(_) => VALIDATION_INFORMATION,
            Self::CapabilityInformation(_) => CAPABILITY_INFORMATION,
            Self::ClientRegistrationMethod(_) => CLIENT_REGISTRATION_METHOD,
            Self::DefaultsInformation(_) => DEFAULTS_INFORMATION,
            Self::ProtectionStorageMasks(_) => PROTECTION_STORAGE_MASKS,
            Self::Unknown(item) => item.tag().raw(),
        }
    }

    /// Lends a complex or unknown Item tree for callback-scoped use.
    pub fn with_ttlv<R>(&self, callback: impl FnOnce(&Item) -> R) -> Option<R> {
        match self {
            Self::ServerInformation(item)
            | Self::ExtensionInformation(item)
            | Self::RngParameters(item)
            | Self::ProfileInformation(item)
            | Self::ValidationInformation(item)
            | Self::CapabilityInformation(item)
            | Self::DefaultsInformation(item)
            | Self::ProtectionStorageMasks(item)
            | Self::Unknown(item) => Some(callback(item)),
            Self::Operation(_)
            | Self::ObjectType(_)
            | Self::VendorIdentification(_)
            | Self::ApplicationNamespace(_)
            | Self::AttestationType(_)
            | Self::ClientRegistrationMethod(_) => None,
        }
    }
}

/// The typed result of one KMIP Query response.
pub struct QueryResponse {
    result: KmipOperationResult,
    empty_payload: bool,
    fields: Vec<QueryResponseField>,
}

impl QueryResponse {
    /// Converts one validated Query response batch item.
    ///
    /// The empty successful payload and structured Table 283 form are both
    /// accepted, retaining the source discrepancy `KMIPKIT-DISC-047` without
    /// making a server-conformance determination.
    ///
    /// # Errors
    ///
    /// Returns [`QueryError`] for invalid common result metadata, operation,
    /// payload member types/cardinalities, or required structured fields.
    pub fn try_from_response_item(item: ResponseBatchItemView<'_>) -> Result<Self, QueryError> {
        if item.operation() != Some(QUERY_OPERATION) {
            return Err(QueryError::UnexpectedOperation);
        }
        let status = item
            .result_status()
            .ok_or(QueryError::MissingResultStatus)?;
        if status.raw() == PENDING {
            return Err(QueryError::PendingNotSupported);
        }
        let result_message = item.with_result_message(|text| ResultMessage::new(text.to_owned()));
        let result = KmipOperationResult::new(status, item.result_reason(), result_message)
            .map_err(QueryError::InvalidOperationResult)?;

        if status.raw() != SUCCESS {
            if item.with_response_payload(|_| ()).is_some() {
                return Err(QueryError::UnexpectedResponsePayload);
            }
            return Ok(Self {
                result,
                empty_payload: false,
                fields: Vec::new(),
            });
        }

        let parsed = item
            .with_response_payload(parse_response_payload)
            .ok_or(QueryError::MissingResponsePayload)??;
        Ok(Self {
            result,
            empty_payload: parsed.empty_payload,
            fields: parsed.fields,
        })
    }

    /// Returns the complete KMIP operation result.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }

    /// Returns whether the server used the empty successful Response Payload form.
    #[must_use]
    pub const fn is_empty_payload(&self) -> bool {
        self.empty_payload
    }

    /// Returns all structured response members in their original wire order.
    #[must_use]
    pub fn response_fields(&self) -> &[QueryResponseField] {
        &self.fields
    }

    /// Returns all Operation values in their original order.
    pub fn operations(&self) -> impl Iterator<Item = u32> + '_ {
        self.fields.iter().filter_map(|field| match field {
            QueryResponseField::Operation(value) => Some(*value),
            _ => None,
        })
    }

    /// Returns all Object Type values in their original order.
    pub fn object_types(&self) -> impl Iterator<Item = u32> + '_ {
        self.fields.iter().filter_map(|field| match field {
            QueryResponseField::ObjectType(value) => Some(*value),
            _ => None,
        })
    }

    /// Returns optional Vendor Identification.
    #[must_use]
    pub fn vendor_identification(&self) -> Option<&str> {
        self.fields.iter().find_map(|field| match field {
            QueryResponseField::VendorIdentification(value) => Some(value.as_str()),
            _ => None,
        })
    }

    /// Returns optional vendor-specific Server Information.
    #[must_use]
    pub fn server_information(&self) -> Option<&Item> {
        self.fields.iter().find_map(|field| match field {
            QueryResponseField::ServerInformation(value) => Some(value),
            _ => None,
        })
    }

    /// Returns Application Namespace values in their original order.
    pub fn application_namespaces(&self) -> impl Iterator<Item = &str> + '_ {
        self.fields.iter().filter_map(|field| match field {
            QueryResponseField::ApplicationNamespace(value) => Some(value.as_str()),
            _ => None,
        })
    }

    /// Returns Extension Information structures in their original order.
    pub fn extension_information(&self) -> impl Iterator<Item = &Item> + '_ {
        self.fields.iter().filter_map(|field| match field {
            QueryResponseField::ExtensionInformation(value) => Some(value),
            _ => None,
        })
    }

    /// Returns Attestation Type values in their original order.
    pub fn attestation_types(&self) -> impl Iterator<Item = u32> + '_ {
        self.fields.iter().filter_map(|field| match field {
            QueryResponseField::AttestationType(value) => Some(*value),
            _ => None,
        })
    }

    /// Returns RNG Parameters structures in their original order.
    pub fn rng_parameters(&self) -> impl Iterator<Item = &Item> + '_ {
        self.fields.iter().filter_map(|field| match field {
            QueryResponseField::RngParameters(value) => Some(value),
            _ => None,
        })
    }

    /// Returns Profile Information structures in their original order.
    pub fn profile_information(&self) -> impl Iterator<Item = &Item> + '_ {
        self.fields.iter().filter_map(|field| match field {
            QueryResponseField::ProfileInformation(value) => Some(value),
            _ => None,
        })
    }

    /// Returns Validation Information structures in their original order.
    pub fn validation_information(&self) -> impl Iterator<Item = &Item> + '_ {
        self.fields.iter().filter_map(|field| match field {
            QueryResponseField::ValidationInformation(value) => Some(value),
            _ => None,
        })
    }

    /// Returns Capability Information structures in their original order.
    pub fn capability_information(&self) -> impl Iterator<Item = &Item> + '_ {
        self.fields.iter().filter_map(|field| match field {
            QueryResponseField::CapabilityInformation(value) => Some(value),
            _ => None,
        })
    }

    /// Returns Client Registration Method values in their original order.
    pub fn client_registration_methods(&self) -> impl Iterator<Item = u32> + '_ {
        self.fields.iter().filter_map(|field| match field {
            QueryResponseField::ClientRegistrationMethod(value) => Some(*value),
            _ => None,
        })
    }

    /// Returns optional Defaults Information.
    #[must_use]
    pub fn defaults_information(&self) -> Option<&Item> {
        self.fields.iter().find_map(|field| match field {
            QueryResponseField::DefaultsInformation(value) => Some(value),
            _ => None,
        })
    }

    /// Returns optional Protection Storage Masks. Structured success requires it.
    #[must_use]
    pub fn protection_storage_masks(&self) -> Option<&Item> {
        self.fields.iter().find_map(|field| match field {
            QueryResponseField::ProtectionStorageMasks(value) => Some(value),
            _ => None,
        })
    }

    /// Returns unknown and extension Items in their original order.
    pub fn unknown_items(&self) -> impl Iterator<Item = &Item> + '_ {
        self.fields.iter().filter_map(|field| match field {
            QueryResponseField::Unknown(value) => Some(value),
            _ => None,
        })
    }
}

impl fmt::Debug for QueryResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("QueryResponse")
            .field("result", &self.result)
            .field("empty_payload", &self.empty_payload)
            .field("response_field_count", &self.fields.len())
            .finish()
    }
}

struct ParsedResponsePayload {
    empty_payload: bool,
    fields: Vec<QueryResponseField>,
}

fn parse_response_payload(payload: StructureView<'_>) -> Result<ParsedResponsePayload, QueryError> {
    if payload.children().is_empty() {
        return Ok(ParsedResponsePayload {
            empty_payload: true,
            fields: Vec::new(),
        });
    }

    let mut counts = [0_usize; 14];
    let mut fields = Vec::with_capacity(payload.children().len());
    for field in payload.children() {
        let tag = field.tag().raw();
        let Some(index) = query_response_field_index(tag) else {
            let value = field.with_value(clone_value)?;
            fields.push(QueryResponseField::Unknown(
                Item::new(field.tag(), value).map_err(QueryError::TtlvModel)?,
            ));
            continue;
        };
        counts[index] += 1;
        if is_singleton_response_field(tag) && counts[index] > 1 {
            return Err(QueryError::RepeatedSingletonField);
        }

        let parsed = field.with_value(|value| parse_known_response_field(tag, value))?;
        fields.push(parsed);
    }

    if counts[13] == 0 {
        return Err(QueryError::MissingProtectionStorageMasks);
    }
    Ok(ParsedResponsePayload {
        empty_payload: false,
        fields,
    })
}

fn parse_known_response_field(
    tag: u32,
    value: ValueView<'_>,
) -> Result<QueryResponseField, QueryError> {
    let invalid = || QueryError::MalformedResponsePayload;
    let enumeration = || match value {
        ValueView::Enumeration(value) => Ok(*value),
        _ => Err(invalid()),
    };
    let text = || match value {
        ValueView::TextString(value) => Ok(value.to_owned()),
        _ => Err(invalid()),
    };
    let structure = || match value {
        ValueView::Structure(_) => Ok(()),
        _ => Err(invalid()),
    };

    match tag {
        OPERATION => Ok(QueryResponseField::Operation(enumeration()?)),
        OBJECT_TYPE => Ok(QueryResponseField::ObjectType(enumeration()?)),
        VENDOR_IDENTIFICATION => Ok(QueryResponseField::VendorIdentification(text()?)),
        SERVER_INFORMATION => Ok(QueryResponseField::ServerInformation(clone_item(
            tag, value,
        )?)),
        APPLICATION_NAMESPACE => Ok(QueryResponseField::ApplicationNamespace(text()?)),
        EXTENSION_INFORMATION => {
            structure()?;
            Ok(QueryResponseField::ExtensionInformation(clone_item(
                tag, value,
            )?))
        }
        ATTESTATION_TYPE => Ok(QueryResponseField::AttestationType(enumeration()?)),
        RNG_PARAMETERS => {
            structure()?;
            Ok(QueryResponseField::RngParameters(clone_item(tag, value)?))
        }
        PROFILE_INFORMATION => {
            structure()?;
            Ok(QueryResponseField::ProfileInformation(clone_item(
                tag, value,
            )?))
        }
        VALIDATION_INFORMATION => {
            structure()?;
            Ok(QueryResponseField::ValidationInformation(clone_item(
                tag, value,
            )?))
        }
        CAPABILITY_INFORMATION => {
            structure()?;
            Ok(QueryResponseField::CapabilityInformation(clone_item(
                tag, value,
            )?))
        }
        CLIENT_REGISTRATION_METHOD => {
            Ok(QueryResponseField::ClientRegistrationMethod(enumeration()?))
        }
        DEFAULTS_INFORMATION => {
            structure()?;
            Ok(QueryResponseField::DefaultsInformation(clone_item(
                tag, value,
            )?))
        }
        PROTECTION_STORAGE_MASKS => {
            structure()?;
            Ok(QueryResponseField::ProtectionStorageMasks(clone_item(
                tag, value,
            )?))
        }
        _ => unreachable!("only assigned response field tags are parsed"),
    }
}

fn query_response_field_index(tag: u32) -> Option<usize> {
    Some(match tag {
        OPERATION => 0,
        OBJECT_TYPE => 1,
        VENDOR_IDENTIFICATION => 2,
        SERVER_INFORMATION => 3,
        APPLICATION_NAMESPACE => 4,
        EXTENSION_INFORMATION => 5,
        ATTESTATION_TYPE => 6,
        RNG_PARAMETERS => 7,
        PROFILE_INFORMATION => 8,
        VALIDATION_INFORMATION => 9,
        CAPABILITY_INFORMATION => 10,
        CLIENT_REGISTRATION_METHOD => 11,
        DEFAULTS_INFORMATION => 12,
        PROTECTION_STORAGE_MASKS => 13,
        _ => return None,
    })
}

const fn is_singleton_response_field(tag: u32) -> bool {
    matches!(
        tag,
        VENDOR_IDENTIFICATION
            | SERVER_INFORMATION
            | DEFAULTS_INFORMATION
            | PROTECTION_STORAGE_MASKS
    )
}

fn clone_item(tag: u32, value: ValueView<'_>) -> Result<Item, QueryError> {
    let value = clone_value(value)?;
    Item::new(checked_tag(tag)?, value).map_err(QueryError::TtlvModel)
}

fn clone_value(value: ValueView<'_>) -> Result<Value, QueryError> {
    match value {
        ValueView::Structure(structure) => {
            let mut children = Structure::new();
            for child in structure.children() {
                let value = child.with_value(clone_value)?;
                let item = Item::new(child.tag(), value).map_err(QueryError::TtlvModel)?;
                children.try_push(item).map_err(QueryError::TtlvModel)?;
            }
            Ok(Value::structure(children))
        }
        ValueView::Integer(value) => Ok(Value::integer(*value)),
        ValueView::LongInteger(value) => Ok(Value::long_integer(*value)),
        ValueView::BigInteger(value) => Ok(Value::big_integer(value.to_vec())),
        ValueView::Enumeration(value) => Ok(Value::enumeration(*value)),
        ValueView::Boolean(value) => Ok(Value::boolean(*value)),
        ValueView::TextString(value) => Ok(Value::text_string(value.to_owned())),
        ValueView::ByteString(value) => Ok(Value::byte_string(value.to_vec())),
        ValueView::DateTime(value) => Ok(Value::date_time(*value)),
        ValueView::Interval(value) => Ok(Value::interval(*value)),
        ValueView::DateTimeExtended(value) => Ok(Value::date_time_extended(*value)),
        _ => Err(QueryError::MalformedResponsePayload),
    }
}

fn item(raw_tag: u32, value: Value) -> Result<Item, ProtocolError> {
    let tag = RawTag::new(raw_tag)
        .and_then(|raw| raw.try_checked())
        .map_err(model_error)?;
    Item::new(tag, value).map_err(model_error)
}

fn checked_tag(raw_tag: u32) -> Result<Tag, QueryError> {
    RawTag::new(raw_tag)
        .and_then(|raw| raw.try_checked())
        .map_err(QueryError::TtlvModel)
}

fn model_error(error: ModelError) -> ProtocolError {
    ProtocolError::new(
        ProtocolErrorKind::InvalidValue,
        ProtocolCauseCategory::InvalidValue,
        error,
    )
}

/// A sanitized Query request or response conversion error.
#[non_exhaustive]
#[derive(Debug)]
pub enum QueryError {
    /// The response item identifies an operation other than Query.
    UnexpectedOperation,
    /// The response item is missing Result Status.
    MissingResultStatus,
    /// Query returned Operation Pending, which has no asynchronous contract.
    PendingNotSupported,
    /// Common KMIP result metadata is inconsistent.
    InvalidOperationResult(ResultValidationError),
    /// A successful Query response omitted its Response Payload Structure.
    MissingResponsePayload,
    /// A failed Query response unexpectedly includes a Response Payload.
    UnexpectedResponsePayload,
    /// A known response member has an invalid Item Type or shape.
    MalformedResponsePayload,
    /// A structured successful response omitted its Table 283 required member.
    MissingProtectionStorageMasks,
    /// A singleton Table 283 member occurred more than once.
    RepeatedSingletonField,
    /// A nested TTLV value could not be represented by the local model.
    TtlvModel(ModelError),
}

impl fmt::Display for QueryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::UnexpectedOperation => "response item is not Query",
            Self::MissingResultStatus => "Query result status is missing",
            Self::PendingNotSupported => "Query does not support an Operation Pending result",
            Self::InvalidOperationResult(error) => {
                return write!(formatter, "invalid Query operation result: {error}");
            }
            Self::MissingResponsePayload => "successful Query response payload is missing",
            Self::UnexpectedResponsePayload => "failed Query response has a payload",
            Self::MalformedResponsePayload => "Query response payload is malformed",
            Self::MissingProtectionStorageMasks => {
                "structured Query response is missing Protection Storage Masks"
            }
            Self::RepeatedSingletonField => "Query response repeats a singleton member",
            Self::TtlvModel(_) => "Query TTLV model is invalid",
        };
        formatter.write_str(message)
    }
}

impl Error for QueryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidOperationResult(error) => Some(error),
            Self::TtlvModel(error) => Some(error),
            Self::UnexpectedOperation
            | Self::MissingResultStatus
            | Self::PendingNotSupported
            | Self::MissingResponsePayload
            | Self::UnexpectedResponsePayload
            | Self::MalformedResponsePayload
            | Self::MissingProtectionStorageMasks
            | Self::RepeatedSingletonField => None,
        }
    }
}
