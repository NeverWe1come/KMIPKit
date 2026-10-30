//! Typed KMIP 2.1 Derive Key request and response payloads.

use std::error::Error;
use std::fmt;

use kmipkit_ttlv::{Item, ModelError, RawTag, Structure, StructureView, Value, ValueView};

use crate::cryptographic_operation::{self as common, OwnedResponseTtlv};
use crate::{
    AttributeSet, KmipOperationResult, ObjectType, ProtocolCauseCategory, ProtocolError,
    ProtocolErrorKind, ResponseBatchItemView, ResultMessage, ResultValidationError, SecretBytes,
    UniqueIdentifier,
};

const OPERATION: u32 = 0x0000_0005;
const SUCCESS: u32 = 0;
const ID_PLACEHOLDER: u32 = 1;
const OBJECT_TYPE: u32 = 0x0042_0057;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const DERIVATION_METHOD: u32 = 0x0042_0031;
const DERIVATION_PARAMETERS: u32 = 0x0042_0032;
const CRYPTOGRAPHIC_PARAMETERS: u32 = 0x0042_002B;
const INITIALIZATION_VECTOR: u32 = 0x0042_003A;
const DERIVATION_DATA: u32 = 0x0042_0030;
const ITERATION_COUNT: u32 = 0x0042_003C;
const SALT: u32 = 0x0042_0084;
const ATTRIBUTES: u32 = 0x0042_0125;
const CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;
const CRYPTOGRAPHIC_LENGTH: u32 = 0x0042_002A;
const HASHING_ALGORITHM: u32 = 0x0042_0038;
const DERIVATION_METHOD_PBKDF2: u32 = 1;
const DERIVATION_METHOD_HASH: u32 = 2;

/// A KMIP Derivation Method Enumeration that retains unknown future values.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DerivationMethod(u32);

impl DerivationMethod {
    /// Creates a Derivation Method from its raw unsigned Enumeration value.
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Returns the exact raw unsigned Enumeration value.
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// Caller-selected §7.12 Derivation Parameters.
///
/// Secret byte-string members use [`SecretBytes`] so `KMIPKit` zeroizes its
/// owned copies. Unknown members may be retained as generic TTLV extensions.
pub struct DerivationParameters {
    cryptographic_parameters: Option<Structure>,
    initialization_vector: Option<SecretBytes>,
    derivation_data: Vec<SecretBytes>,
    salt: Option<SecretBytes>,
    iteration_count: Option<i32>,
    extensions: Structure,
}

impl DerivationParameters {
    /// Creates empty caller-controlled derivation parameters.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            cryptographic_parameters: None,
            initialization_vector: None,
            derivation_data: Vec::new(),
            salt: None,
            iteration_count: None,
            extensions: Structure::new(),
        }
    }

    /// Supplies explicit Cryptographic Parameters for the selected method.
    #[must_use]
    pub fn with_cryptographic_parameters(mut self, value: Structure) -> Self {
        self.cryptographic_parameters = Some(value);
        self
    }

    /// Supplies an explicit Initialization Vector.
    #[must_use]
    pub fn with_initialization_vector(mut self, value: SecretBytes) -> Self {
        self.initialization_vector = Some(value);
        self
    }

    /// Appends one caller-selected Derivation Data occurrence in wire order.
    #[must_use]
    pub fn with_derivation_data(mut self, value: SecretBytes) -> Self {
        self.derivation_data.push(value);
        self
    }

    /// Supplies the PBKDF2 salt or method-specific salt.
    #[must_use]
    pub fn with_salt(mut self, value: SecretBytes) -> Self {
        self.salt = Some(value);
        self
    }

    /// Supplies the PBKDF2 iteration count.
    #[must_use]
    pub const fn with_iteration_count(mut self, value: i32) -> Self {
        self.iteration_count = Some(value);
        self
    }

    /// Appends an unknown or vendor Derivation Parameters member without
    /// changing its tag, TTLV value, or relative order among extensions.
    ///
    /// # Errors
    /// Returns a sanitized protocol error if the generic structure exceeds
    /// TTLV model limits.
    pub fn with_extension(mut self, item: Item) -> Result<Self, ProtocolError> {
        self.extensions.try_push(item).map_err(model_error)?;
        Ok(self)
    }

    fn into_ttlv_structure(self) -> Result<Structure, ProtocolError> {
        let mut parameters = Structure::new();
        if let Some(value) = self.cryptographic_parameters {
            push(
                &mut parameters,
                CRYPTOGRAPHIC_PARAMETERS,
                Value::structure(value),
            )?;
        }
        if let Some(value) = self.initialization_vector {
            push(
                &mut parameters,
                INITIALIZATION_VECTOR,
                value.into_ttlv_value(),
            )?;
        }
        for value in self.derivation_data {
            push(&mut parameters, DERIVATION_DATA, value.into_ttlv_value())?;
        }
        if let Some(value) = self.salt {
            push(&mut parameters, SALT, value.into_ttlv_value())?;
        }
        if let Some(value) = self.iteration_count {
            push(&mut parameters, ITERATION_COUNT, Value::integer(value))?;
        }
        for item in self.extensions.view().children() {
            parameters
                .try_push(clone_item(item)?)
                .map_err(model_error)?;
        }
        Ok(parameters)
    }
}

impl Default for DerivationParameters {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for DerivationParameters {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DerivationParameters")
            .field(
                "has_cryptographic_parameters",
                &self.cryptographic_parameters.is_some(),
            )
            .field(
                "has_initialization_vector",
                &self.initialization_vector.is_some(),
            )
            .field("derivation_data_count", &self.derivation_data.len())
            .field("has_salt", &self.salt.is_some())
            .field("has_iteration_count", &self.iteration_count.is_some())
            .field("extension_count", &self.extensions.view().children().len())
            .finish()
    }
}

/// A typed KMIP 2.1 Derive Key request from §6.1.14 Table 205.
pub struct DeriveKeyRequest {
    object_type: ObjectType,
    unique_identifiers: Vec<UniqueIdentifier>,
    derivation_method: DerivationMethod,
    derivation_parameters: DerivationParameters,
    attributes: AttributeSet,
}

impl DeriveKeyRequest {
    /// Creates a request from caller-selected inputs, method, parameters, and
    /// desired output attributes.
    #[must_use]
    pub fn new(
        object_type: ObjectType,
        unique_identifiers: Vec<UniqueIdentifier>,
        derivation_method: DerivationMethod,
        derivation_parameters: DerivationParameters,
        attributes: AttributeSet,
    ) -> Self {
        Self {
            object_type,
            unique_identifiers,
            derivation_method,
            derivation_parameters,
            attributes,
        }
    }

    /// Encodes the ordered Table 205 request payload.
    ///
    /// The source object's remote Cryptographic Usage Mask and method-specific
    /// server policy cannot be queried from local request data; the server
    /// remains authoritative. `KMIPKit` does enforce the locally knowable
    /// required fields and the Derive Key ID Placeholder prohibition.
    ///
    /// # Errors
    /// Returns a payload-free protocol error for source-defined Attribute
    /// cardinality violations, missing required identifiers, an ID Placeholder
    /// identifier, missing required output attributes, or invalid TTLV
    /// structure.
    pub fn into_ttlv_payload(self) -> Result<Structure, ProtocolError> {
        if crate::attribute::validate_single_instance_attribute_items(
            self.attributes.as_items(),
            Some(self.object_type.raw()),
            true,
        )
        .is_err()
            || self.unique_identifiers.is_empty()
            || self.unique_identifiers.iter().any(is_id_placeholder)
            || !has_required_output_attributes(self.object_type, &self.attributes)
            || !self
                .derivation_parameters
                .valid_for_method(self.derivation_method)
        {
            return Err(invalid_request());
        }

        let mut payload = Structure::new();
        push(
            &mut payload,
            OBJECT_TYPE,
            Value::enumeration(self.object_type.raw()),
        )?;
        for identifier in self.unique_identifiers {
            push(
                &mut payload,
                UNIQUE_IDENTIFIER,
                unique_identifier_value(identifier),
            )?;
        }
        push(
            &mut payload,
            DERIVATION_METHOD,
            Value::enumeration(self.derivation_method.raw()),
        )?;
        push(
            &mut payload,
            DERIVATION_PARAMETERS,
            Value::structure(self.derivation_parameters.into_ttlv_structure()?),
        )?;
        let mut attributes = Structure::new();
        for attribute in self.attributes.into_items() {
            attributes.try_push(attribute).map_err(model_error)?;
        }
        push(&mut payload, ATTRIBUTES, Value::structure(attributes))?;
        Ok(payload)
    }
}

impl fmt::Debug for DeriveKeyRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DeriveKeyRequest")
            .field("object_type", &self.object_type.raw())
            .field("unique_identifier_count", &self.unique_identifiers.len())
            .field("derivation_method", &self.derivation_method.raw())
            .field("derivation_parameters", &self.derivation_parameters)
            .field("attribute_count", &self.attributes.len())
            .finish()
    }
}

impl DerivationParameters {
    fn valid_for_method(&self, method: DerivationMethod) -> bool {
        match method.raw() {
            // §7.12 Table 364 requires both Salt and Iteration Count for PBKDF2.
            DERIVATION_METHOD_PBKDF2 => self.salt.is_some() && self.iteration_count.is_some(),
            // §7.12 requires an explicit Hashing Algorithm for HASH derivation.
            DERIVATION_METHOD_HASH => self.has_hashing_algorithm(),
            _ => true,
        }
    }

    fn has_hashing_algorithm(&self) -> bool {
        self.cryptographic_parameters
            .as_ref()
            .is_some_and(|parameters| {
                parameters.view().children().iter().any(|field| {
                    field.tag().raw() == HASHING_ALGORITHM
                        && field.with_value(|value| matches!(value, ValueView::Enumeration(_)))
                })
            })
    }
}

/// A typed Derive Key response and common KMIP operation result.
#[derive(Debug)]
pub struct DeriveKeyResponse {
    result: KmipOperationResult,
    unique_identifier: Option<UniqueIdentifier>,
    response_ttlv: OwnedResponseTtlv,
}

impl DeriveKeyResponse {
    /// Converts one validated Derive Key response batch item from §6.1.14
    /// Tables 206–207.
    ///
    /// # Errors
    /// Returns [`DeriveKeyError`] for a different operation, invalid common
    /// result fields, or a malformed successful response payload.
    pub fn try_from_response_item(item: ResponseBatchItemView<'_>) -> Result<Self, DeriveKeyError> {
        if item.operation() != Some(OPERATION) {
            return Err(DeriveKeyError::UnexpectedOperation);
        }
        let status = item
            .result_status()
            .ok_or(DeriveKeyError::MissingResultStatus)?;
        let message = item.with_result_message(|text| ResultMessage::new(text.to_owned()));
        let result = KmipOperationResult::new(status, item.result_reason(), message)
            .map_err(DeriveKeyError::InvalidOperationResult)?;
        let response_ttlv = common::clone_response_item(item, "Derive Key")
            .map_err(|_| DeriveKeyError::MalformedResponseItem)?;
        if status.raw() != SUCCESS {
            return Ok(Self {
                result,
                unique_identifier: None,
                response_ttlv,
            });
        }
        let unique_identifier = item
            .with_response_payload(|payload| parse_success_payload(&payload))
            .ok_or(DeriveKeyError::MissingSuccessPayload)??;
        Ok(Self {
            result,
            unique_identifier: Some(unique_identifier),
            response_ttlv,
        })
    }

    /// Returns the exact server-reported KMIP operation result.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }

    /// Returns the identifier of the derived object on success.
    #[must_use]
    pub const fn unique_identifier(&self) -> Option<&UniqueIdentifier> {
        self.unique_identifier.as_ref()
    }

    /// Lends the complete ordered generic response item, including unknown
    /// tags and future fields, for callback-scoped inspection.
    pub fn with_ttlv<R>(&self, callback: impl for<'a> FnOnce(StructureView<'a>) -> R) -> R {
        self.response_ttlv.with_ttlv(callback)
    }
}

/// A payload-free error converting a Derive Key response.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeriveKeyError {
    /// The response item is not Derive Key.
    UnexpectedOperation,
    /// The validated response item omitted Result Status.
    MissingResultStatus,
    /// The common KMIP result fields are inconsistent.
    InvalidOperationResult(ResultValidationError),
    /// A success result omitted its Response Payload.
    MissingSuccessPayload,
    /// A success payload omitted, duplicated, or mistyped Unique Identifier.
    MalformedSuccessPayload,
    /// The complete generic response item could not be retained.
    MalformedResponseItem,
}

impl fmt::Display for DeriveKeyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedOperation => formatter.write_str("response item is not Derive Key"),
            Self::MissingResultStatus => formatter.write_str("Derive Key result status is missing"),
            Self::InvalidOperationResult(cause) => {
                write!(formatter, "Derive Key result is invalid: {cause}")
            }
            Self::MissingSuccessPayload => {
                formatter.write_str("successful Derive Key result has no payload")
            }
            Self::MalformedSuccessPayload => {
                formatter.write_str("successful Derive Key response payload is malformed")
            }
            Self::MalformedResponseItem => {
                formatter.write_str("Derive Key response item is malformed")
            }
        }
    }
}

impl Error for DeriveKeyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidOperationResult(cause) => Some(cause),
            _ => None,
        }
    }
}

fn parse_success_payload(payload: &StructureView<'_>) -> Result<UniqueIdentifier, DeriveKeyError> {
    let mut identifier = None;
    for field in payload.children() {
        if field.tag().raw() == UNIQUE_IDENTIFIER {
            if identifier.is_some() {
                return Err(DeriveKeyError::MalformedSuccessPayload);
            }
            identifier = field.with_value(|value| parse_unique_identifier(&value));
            if identifier.is_none() {
                return Err(DeriveKeyError::MalformedSuccessPayload);
            }
        }
    }
    identifier.ok_or(DeriveKeyError::MalformedSuccessPayload)
}

fn parse_unique_identifier(value: &ValueView<'_>) -> Option<UniqueIdentifier> {
    match value {
        ValueView::TextString(value) => Some(UniqueIdentifier::TextString((*value).to_owned())),
        ValueView::Enumeration(value) => Some(UniqueIdentifier::Enumeration(**value)),
        ValueView::Integer(value) => Some(UniqueIdentifier::Integer(**value)),
        _ => None,
    }
}

fn has_required_output_attributes(object_type: ObjectType, attributes: &AttributeSet) -> bool {
    let mut length_count = 0;
    let mut length_is_positive = false;
    let mut algorithm_count = 0;
    for item in attributes.as_items() {
        match item.tag().raw() {
            CRYPTOGRAPHIC_LENGTH => {
                length_count += 1;
                length_is_positive = item
                    .with_value(|value| matches!(value, ValueView::Integer(value) if *value > 0));
            }
            CRYPTOGRAPHIC_ALGORITHM => {
                algorithm_count += 1;
                if !item.with_value(|value| matches!(value, ValueView::Enumeration(_))) {
                    return false;
                }
            }
            _ => {}
        }
    }
    length_count == 1
        && length_is_positive
        && (matches!(object_type.raw(), 1..=3) == (algorithm_count == 1))
        && algorithm_count <= 1
}

fn is_id_placeholder(identifier: &UniqueIdentifier) -> bool {
    matches!(identifier, UniqueIdentifier::Enumeration(value) if *value == ID_PLACEHOLDER)
}

fn unique_identifier_value(identifier: UniqueIdentifier) -> Value {
    match identifier {
        UniqueIdentifier::TextString(value) => Value::text_string(value),
        UniqueIdentifier::Enumeration(value) => Value::enumeration(value),
        UniqueIdentifier::Integer(value) => Value::integer(value),
    }
}

fn push(payload: &mut Structure, raw_tag: u32, value: Value) -> Result<(), ProtocolError> {
    let tag = RawTag::new(raw_tag)
        .and_then(|raw| raw.try_checked())
        .map_err(model_error)?;
    let item = Item::new(tag, value).map_err(model_error)?;
    payload.try_push(item).map_err(model_error)
}

fn invalid_request() -> ProtocolError {
    ProtocolError::categorized(
        ProtocolErrorKind::InvalidValue,
        ProtocolCauseCategory::InvalidValue,
    )
}

fn model_error(error: ModelError) -> ProtocolError {
    ProtocolError::new(
        ProtocolErrorKind::InvalidValue,
        ProtocolCauseCategory::InvalidValue,
        error,
    )
}

fn clone_item(item: &Item) -> Result<Item, ProtocolError> {
    item.with_value(|value| match value {
        ValueView::Structure(value) => {
            let mut structure = Structure::new();
            for child in value.children() {
                structure
                    .try_push(clone_item(child)?)
                    .map_err(model_error)?;
            }
            Ok(Value::structure(structure))
        }
        ValueView::Integer(value) => Ok(Value::integer(*value)),
        ValueView::LongInteger(value) => Ok(Value::long_integer(*value)),
        ValueView::BigInteger(value) => Ok(Value::big_integer(value.to_vec())),
        ValueView::Enumeration(value) => Ok(Value::enumeration(*value)),
        ValueView::Boolean(value) => Ok(Value::boolean(*value)),
        ValueView::TextString(value) => Ok(Value::text_string((*value).to_owned())),
        ValueView::ByteString(value) => Ok(Value::byte_string(value.to_vec())),
        ValueView::DateTime(value) => Ok(Value::date_time(*value)),
        ValueView::Interval(value) => Ok(Value::interval(*value)),
        ValueView::DateTimeExtended(value) => Ok(Value::date_time_extended(*value)),
        _ => Err(invalid_request()),
    })
    .and_then(|value| Item::new(item.tag(), value).map_err(model_error))
}
