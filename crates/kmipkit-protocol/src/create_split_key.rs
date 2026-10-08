//! Typed KMIP 2.1 Create Split Key request and response payloads.

use std::error::Error;
use std::fmt;

use kmipkit_ttlv::{Item, ModelError, RawTag, Structure, StructureView, Value, ValueView};

use crate::{
    AttributeSet, KmipOperationResult, ObjectType, ProtocolCauseCategory, ProtocolError,
    ProtocolErrorKind, ResponseBatchItemView, ResultMessage, ResultValidationError,
    UniqueIdentifier,
};

const CREATE_SPLIT_KEY_OPERATION: u32 = 0x0000_0003;
const SUCCESS: u32 = 0;

const OBJECT_TYPE: u32 = 0x0042_0057;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const SPLIT_KEY_PARTS: u32 = 0x0042_008b;
const SPLIT_KEY_THRESHOLD: u32 = 0x0042_008c;
const SPLIT_KEY_METHOD: u32 = 0x0042_008a;
const PRIME_FIELD_SIZE: u32 = 0x0042_0062;
const ATTRIBUTES: u32 = 0x0042_0125;
const PROTECTION_STORAGE_MASKS: u32 = 0x0042_015f;

/// A KMIP Split Key Method Enumeration, retaining unknown future values.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SplitKeyMethod(u32);

impl SplitKeyMethod {
    /// Splits the secret with XOR.
    pub const XOR: Self = Self(1);

    /// Splits the secret with polynomial sharing over GF(2¹⁶).
    pub const POLYNOMIAL_SHARING_GF_2_16: Self = Self(2);

    /// Splits the secret with polynomial sharing over the caller-selected prime field.
    pub const POLYNOMIAL_SHARING_PRIME_FIELD: Self = Self(3);

    /// Splits the secret with polynomial sharing over GF(2⁸).
    pub const POLYNOMIAL_SHARING_GF_2_8: Self = Self(4);

    /// Creates a Split Key Method from its raw unsigned Enumeration value.
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

/// A typed KMIP 2.1 Create Split Key request payload.
///
/// Required split parameters and Attributes are caller supplied. Optional
/// input identifiers, Prime Field Size, and Protection Storage Masks retain
/// their presence independently. `KMIPKit` applies FR-015 by requiring an
/// explicit Prime Field Size for Polynomial Sharing Prime Field requests.
pub struct CreateSplitKeyRequest {
    object_type: ObjectType,
    unique_identifier: Option<UniqueIdentifier>,
    split_key_parts: i32,
    split_key_threshold: i32,
    split_key_method: SplitKeyMethod,
    prime_field_size: Option<Vec<u8>>,
    attributes: AttributeSet,
    protection_storage_masks: Option<Structure>,
}

impl CreateSplitKeyRequest {
    /// Creates a request with explicit required fields and Attributes.
    #[must_use]
    pub const fn new(
        object_type: ObjectType,
        split_key_parts: i32,
        split_key_threshold: i32,
        split_key_method: SplitKeyMethod,
        attributes: AttributeSet,
    ) -> Self {
        Self {
            object_type,
            unique_identifier: None,
            split_key_parts,
            split_key_threshold,
            split_key_method,
            prime_field_size: None,
            attributes,
            protection_storage_masks: None,
        }
    }

    /// Sets the optional Unique Identifier of the input key to split.
    #[must_use]
    pub fn with_unique_identifier(mut self, identifier: UniqueIdentifier) -> Self {
        self.unique_identifier = Some(identifier);
        self
    }

    /// Sets the optional Prime Field Size as the exact Big Integer bytes.
    #[must_use]
    pub fn with_prime_field_size(mut self, value: Vec<u8>) -> Self {
        self.prime_field_size = Some(value);
        self
    }

    /// Replaces the caller-supplied direct Object Attribute items.
    #[must_use]
    pub fn with_attributes(mut self, attributes: AttributeSet) -> Self {
        self.attributes = attributes;
        self
    }

    /// Sets the optional generic Protection Storage Masks Structure.
    #[must_use]
    pub fn with_protection_storage_masks(mut self, masks: Structure) -> Self {
        self.protection_storage_masks = Some(masks);
        self
    }

    /// Returns the selected Object Type, including its raw Enumeration value.
    #[must_use]
    pub const fn object_type(&self) -> ObjectType {
        self.object_type
    }

    /// Returns the optional input key identifier.
    #[must_use]
    pub const fn unique_identifier(&self) -> Option<&UniqueIdentifier> {
        self.unique_identifier.as_ref()
    }

    /// Returns the caller-supplied number of parts.
    #[must_use]
    pub const fn split_key_parts(&self) -> i32 {
        self.split_key_parts
    }

    /// Returns the caller-supplied reconstruction threshold.
    #[must_use]
    pub const fn split_key_threshold(&self) -> i32 {
        self.split_key_threshold
    }

    /// Returns the selected raw Split Key Method.
    #[must_use]
    pub const fn split_key_method(&self) -> SplitKeyMethod {
        self.split_key_method
    }

    /// Returns the optional exact Big Integer bytes for Prime Field Size.
    #[must_use]
    pub fn prime_field_size(&self) -> Option<&[u8]> {
        self.prime_field_size.as_deref()
    }

    /// Returns the caller-supplied direct Object Attribute items as request
    /// values. The client does not claim these override the attributes of an
    /// input key identified by `Unique Identifier`.
    #[must_use]
    pub const fn attributes(&self) -> &AttributeSet {
        &self.attributes
    }

    /// Returns the optional generic Protection Storage Masks Structure.
    #[must_use]
    pub const fn protection_storage_masks(&self) -> Option<&Structure> {
        self.protection_storage_masks.as_ref()
    }

    /// Encodes the ordered Create Split Key request payload.
    ///
    /// The required Attributes Structure is emitted even when it has no
    /// children. No split method, parameter, attribute, or storage mask is
    /// selected or synthesized.
    ///
    /// # Errors
    ///
    /// Returns a sanitized protocol error for the FR-015 client-policy
    /// violation or an invalid generic TTLV model value.
    pub fn into_ttlv_payload(self) -> Result<Structure, ProtocolError> {
        self.validate_client_policy().map_err(|error| {
            ProtocolError::new(
                ProtocolErrorKind::InvalidValue,
                ProtocolCauseCategory::InvalidValue,
                error,
            )
        })?;

        let mut payload = Structure::new();
        push(
            &mut payload,
            OBJECT_TYPE,
            Value::enumeration(self.object_type.raw()),
        )?;
        if let Some(identifier) = self.unique_identifier {
            push(
                &mut payload,
                UNIQUE_IDENTIFIER,
                unique_identifier_value(identifier),
            )?;
        }
        push(
            &mut payload,
            SPLIT_KEY_PARTS,
            Value::integer(self.split_key_parts),
        )?;
        push(
            &mut payload,
            SPLIT_KEY_THRESHOLD,
            Value::integer(self.split_key_threshold),
        )?;
        push(
            &mut payload,
            SPLIT_KEY_METHOD,
            Value::enumeration(self.split_key_method.raw()),
        )?;
        if let Some(prime_field_size) = self.prime_field_size {
            push(
                &mut payload,
                PRIME_FIELD_SIZE,
                Value::big_integer(prime_field_size),
            )?;
        }

        let mut attributes = Structure::new();
        for attribute in self.attributes.into_items() {
            attributes.try_push(attribute).map_err(model_error)?;
        }
        push(&mut payload, ATTRIBUTES, Value::structure(attributes))?;
        if let Some(masks) = self.protection_storage_masks {
            push(
                &mut payload,
                PROTECTION_STORAGE_MASKS,
                Value::structure(masks),
            )?;
        }
        Ok(payload)
    }

    fn validate_client_policy(&self) -> Result<(), CreateSplitKeyError> {
        if self.split_key_method == SplitKeyMethod::POLYNOMIAL_SHARING_PRIME_FIELD
            && self.prime_field_size.is_none()
        {
            Err(CreateSplitKeyError::PolynomialMethodRequiresPrimeFieldSize)
        } else {
            Ok(())
        }
    }
}

impl fmt::Debug for CreateSplitKeyRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CreateSplitKeyRequest")
            .field("object_type", &self.object_type.raw())
            .field("has_unique_identifier", &self.unique_identifier.is_some())
            .field("split_key_parts", &self.split_key_parts)
            .field("split_key_threshold", &self.split_key_threshold)
            .field("split_key_method", &self.split_key_method.raw())
            .field("has_prime_field_size", &self.prime_field_size.is_some())
            .field("attribute_count", &self.attributes.len())
            .field(
                "has_protection_storage_masks",
                &self.protection_storage_masks.is_some(),
            )
            .finish()
    }
}

/// A typed Create Split Key response and operation result.
#[derive(Clone, Eq, PartialEq)]
pub struct CreateSplitKeyResponse {
    result: KmipOperationResult,
    unique_identifiers: Vec<UniqueIdentifier>,
}

impl CreateSplitKeyResponse {
    /// Converts one validated Create Split Key response batch item.
    ///
    /// A successful result requires one or more Unique Identifier fields and
    /// retains repeated values in their original wire order. Failure results
    /// preserve the shared KMIP result without success payload fields.
    ///
    /// # Errors
    ///
    /// Returns [`CreateSplitKeyError`] for a different operation, malformed
    /// result, or missing or incorrectly typed successful response fields.
    pub fn try_from_response_item(
        item: ResponseBatchItemView<'_>,
    ) -> Result<Self, CreateSplitKeyError> {
        if item.operation() != Some(CREATE_SPLIT_KEY_OPERATION) {
            return Err(CreateSplitKeyError::UnexpectedOperation);
        }
        let status = item
            .result_status()
            .ok_or(CreateSplitKeyError::MissingResultStatus)?;
        let result_message = item.with_result_message(|text| ResultMessage::new(text.to_owned()));
        let result = KmipOperationResult::new(status, item.result_reason(), result_message)
            .map_err(CreateSplitKeyError::InvalidOperationResult)?;

        if status.raw() != SUCCESS {
            return Ok(Self {
                result,
                unique_identifiers: Vec::new(),
            });
        }

        let unique_identifiers = item
            .with_response_payload(|payload| parse_success_payload(&payload))
            .ok_or(CreateSplitKeyError::MissingSuccessPayload)??;
        Ok(Self {
            result,
            unique_identifiers,
        })
    }

    /// Returns the exact KMIP operation result.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }

    /// Returns every created object's identifier in response wire order.
    #[must_use]
    pub fn unique_identifiers(&self) -> &[UniqueIdentifier] {
        &self.unique_identifiers
    }
}

impl fmt::Debug for CreateSplitKeyResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CreateSplitKeyResponse")
            .field("result", &self.result)
            .field("unique_identifier_count", &self.unique_identifiers.len())
            .finish()
    }
}

/// A payload-free error converting or validating Create Split Key values.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CreateSplitKeyError {
    /// The response item is not the client-to-server Create Split Key operation.
    UnexpectedOperation,
    /// The validated response item did not expose a Result Status.
    MissingResultStatus,
    /// The represented KMIP result violates the shared status/reason contract.
    InvalidOperationResult(ResultValidationError),
    /// A successful result has no Response Payload Structure.
    MissingSuccessPayload,
    /// A successful payload has no valid Unique Identifier field.
    MalformedSuccessPayload,
    /// `KMIPKit` FR-015 requires a caller value for the polynomial prime-field method.
    PolynomialMethodRequiresPrimeFieldSize,
}

impl fmt::Display for CreateSplitKeyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::UnexpectedOperation => "response item is not Create Split Key",
            Self::MissingResultStatus => "Create Split Key result status is missing",
            Self::InvalidOperationResult(cause) => {
                return write!(
                    formatter,
                    "Create Split Key operation result is invalid: {cause}"
                );
            }
            Self::MissingSuccessPayload => "successful Create Split Key result has no payload",
            Self::MalformedSuccessPayload => {
                "successful Create Split Key response payload is malformed"
            }
            Self::PolynomialMethodRequiresPrimeFieldSize => {
                "Create Split Key FR-015 requires explicit Prime Field Size"
            }
        };
        formatter.write_str(message)
    }
}

impl Error for CreateSplitKeyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidOperationResult(cause) => Some(cause),
            _ => None,
        }
    }
}

fn unique_identifier_value(identifier: UniqueIdentifier) -> Value {
    match identifier {
        UniqueIdentifier::TextString(value) => Value::text_string(value),
        UniqueIdentifier::Enumeration(value) => Value::enumeration(value),
        UniqueIdentifier::Integer(value) => Value::integer(value),
    }
}

fn parse_success_payload(
    payload: &StructureView<'_>,
) -> Result<Vec<UniqueIdentifier>, CreateSplitKeyError> {
    let mut unique_identifiers = Vec::new();
    for field in payload.children() {
        if field.tag().raw() != UNIQUE_IDENTIFIER {
            continue;
        }
        let identifier =
            parse_unique_identifier(field).ok_or(CreateSplitKeyError::MalformedSuccessPayload)?;
        unique_identifiers.push(identifier);
    }
    if unique_identifiers.is_empty() {
        return Err(CreateSplitKeyError::MalformedSuccessPayload);
    }
    Ok(unique_identifiers)
}

fn parse_unique_identifier(field: &Item) -> Option<UniqueIdentifier> {
    field.with_value(|value| match value {
        ValueView::TextString(value) => Some(UniqueIdentifier::TextString(value.to_owned())),
        ValueView::Enumeration(value) => Some(UniqueIdentifier::Enumeration(*value)),
        ValueView::Integer(value) => Some(UniqueIdentifier::Integer(*value)),
        _ => None,
    })
}

fn push(payload: &mut Structure, raw_tag: u32, value: Value) -> Result<(), ProtocolError> {
    let tag = RawTag::new(raw_tag)
        .and_then(|raw_tag| raw_tag.try_checked())
        .map_err(model_error)?;
    payload
        .try_push(Item::new(tag, value).map_err(model_error)?)
        .map_err(model_error)
}

fn model_error(error: ModelError) -> ProtocolError {
    ProtocolError::new(
        ProtocolErrorKind::InvalidValue,
        ProtocolCauseCategory::InvalidValue,
        error,
    )
}
