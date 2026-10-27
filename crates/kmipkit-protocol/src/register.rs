//! Typed KMIP 2.1 Register request and response models.

use std::error::Error;
use std::fmt;

use kmipkit_ttlv::{Item, ModelError, RawTag, Structure, StructureView, Tag, Value, ValueView};

use crate::cryptographic_operation::{self as common, OwnedResponseTtlv};
use crate::{
    AttributeSet, KmipOperationResult, ObjectType, ProtocolCauseCategory, ProtocolError,
    ProtocolErrorKind, ResponseBatchItemView, ResultMessage, ResultValidationError,
    UniqueIdentifier,
};

const OPERATION: u32 = 0x0000_0003;
const OBJECT_TYPE: u32 = 0x0042_0057;
const ATTRIBUTES: u32 = 0x0042_0125;
const PROTECTION_STORAGE_MASKS: u32 = 0x0042_015f;
const CERTIFICATE_LENGTH: u32 = 0x0042_00ad;
const CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;
const CRYPTOGRAPHIC_LENGTH: u32 = 0x0042_002a;
const CRYPTOGRAPHIC_USAGE_MASK: u32 = 0x0042_002c;
const KEY_BLOCK: u32 = 0x0042_0040;
const KEY_VALUE: u32 = 0x0042_0045;
const OPAQUE_OBJECT: u32 = 0x0042_005b;
const OPAQUE_DATA_TYPE: u32 = 0x0042_0059;
const OPAQUE_DATA_VALUE: u32 = 0x0042_005a;
const SUCCESS: u32 = 0;

/// Local attribute-handling context for an Opaque Object in a Register request.
///
/// The selected context is client metadata and is never encoded in the KMIP
/// request payload.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum OpaqueObjectContext {
    /// Treat the Opaque Object as generic opaque data without requiring crypto attributes.
    Generic,
    /// Apply Managed Cryptographic Object Register attribute requirements.
    ManagedCryptographicObject,
}

/// A KMIP 2.1 Register request from §6.1.43, Table 291.
///
/// The caller supplies the object type, its direct Attributes, and the full
/// object item. `KMIPKit` preserves their order and unknown values, rejects
/// repeated source-defined single-instance attributes, and checks the local
/// Table 293 attribute requirements; the server remains authoritative for
/// object validity, policy, and registration.
///
/// For caller-supplied wrapped Key Values, the caller is responsible for
/// providing IV/Counter/Nonce when the selected wrapping method requires it
/// (KMIP Specification v2.1 §3.3, Table 14), and for TTLV-encoding the Key
/// Value Structure before wrapping it when Encoding Option is absent (§3.3,
/// Table 15). `KMIPKit` does not infer referenced-key parameters or inspect
/// wrapped plaintext or ciphertext.
pub struct RegisterRequest {
    object_type: ObjectType,
    attributes: AttributeSet,
    object: Item,
    protection_storage_masks: Option<Structure>,
    opaque_object_context: Option<OpaqueObjectContext>,
}

impl RegisterRequest {
    /// Creates a Register request with explicit object type, attributes, and object.
    #[must_use]
    pub const fn new(object_type: ObjectType, attributes: AttributeSet, object: Item) -> Self {
        Self {
            object_type,
            attributes,
            object,
            protection_storage_masks: None,
            opaque_object_context: None,
        }
    }

    /// Selects local attribute handling for an Opaque Object.
    ///
    /// Supplying a context for any other Object Type makes this request
    /// invalid. The context is never added to the KMIP wire payload.
    #[must_use]
    pub fn with_opaque_object_context(mut self, context: OpaqueObjectContext) -> Self {
        self.opaque_object_context = Some(context);
        self
    }

    /// Sets the optional generic Protection Storage Masks Structure.
    #[must_use]
    pub fn with_protection_storage_masks(mut self, masks: Structure) -> Self {
        self.protection_storage_masks = Some(masks);
        self
    }

    /// Returns the raw-preserving Object Type value.
    #[must_use]
    pub const fn object_type(&self) -> ObjectType {
        self.object_type
    }

    /// Returns the direct ordered attributes supplied by the caller.
    #[must_use]
    pub const fn attributes(&self) -> &AttributeSet {
        &self.attributes
    }

    /// Returns the source object item without exposing its payload in formatting.
    #[must_use]
    pub const fn object(&self) -> &Item {
        &self.object
    }

    /// Validates the caller-provided Register object and attributes in place.
    ///
    /// This check performs no allocation and does not inspect opaque payload
    /// bytes.
    ///
    /// # Errors
    ///
    /// Returns a sanitized error if the object is not a Structure, a Table 4
    /// or Table 293 requirement is not satisfied, or the selected opaque
    /// context does not apply to the Object Type.
    #[doc(hidden)]
    pub fn validate_local_shape(&self) -> Result<(), ProtocolError> {
        if self
            .attributes
            .validate_single_instance_cardinality_for_object_type(self.object_type.raw())
            .is_err()
            || crate::attribute::validate_key_value_attribute_cardinality(
                &self.object,
                Some(self.object_type.raw()),
                false,
            )
            .is_err()
            || !matches!(self.object.item_type(), kmipkit_ttlv::ItemType::Structure)
            || !attributes_satisfy_register_requirements(
                self.object_type,
                &self.attributes,
                &self.object,
                self.opaque_object_context,
            )
        {
            return Err(invalid_request());
        }

        Ok(())
    }

    /// Builds the ordered Request Payload from §6.1.43, Table 291.
    ///
    /// # Errors
    ///
    /// Returns a sanitized error if the object is not a Structure, a Table 293
    /// required attribute is absent, paired key attributes disagree, or the
    /// generic TTLV model cannot represent the payload.
    pub fn into_ttlv_payload(self) -> Result<Structure, ProtocolError> {
        self.validate_local_shape()?;

        let mut attributes = Structure::new();
        for attribute in self.attributes.into_items() {
            attributes.try_push(attribute).map_err(model_error)?;
        }

        let mut payload = Structure::new();
        push(
            &mut payload,
            OBJECT_TYPE,
            Value::enumeration(self.object_type.raw()),
        )?;
        push(&mut payload, ATTRIBUTES, Value::structure(attributes))?;
        payload.try_push(self.object).map_err(model_error)?;
        if let Some(masks) = self.protection_storage_masks {
            push(
                &mut payload,
                PROTECTION_STORAGE_MASKS,
                Value::structure(masks),
            )?;
        }
        Ok(payload)
    }
}

// The context is local validation metadata and intentionally stays out of Debug output.
#[allow(clippy::missing_fields_in_debug)]
impl fmt::Debug for RegisterRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RegisterRequest")
            .field("object_type", &self.object_type)
            .field("attribute_count", &self.attributes.len())
            .field("object_tag", &self.object.tag())
            .field("has_object_payload", &true)
            .field(
                "has_protection_storage_masks",
                &self.protection_storage_masks.is_some(),
            )
            .finish()
    }
}

/// A typed Register result from §6.1.43, Table 292.
///
/// The required Unique Identifier remains in its original KMIP wire form.
/// The complete generic response item is retained for callback-scoped
/// inspection, including unknown or future fields.
#[derive(Debug)]
pub struct RegisterResponse {
    result: KmipOperationResult,
    response_ttlv: OwnedResponseTtlv,
    unique_identifier: Option<UniqueIdentifier>,
}

impl RegisterResponse {
    /// Converts one already validated Register response batch item.
    ///
    /// A successful result requires exactly one well-formed Unique Identifier.
    /// Non-success results preserve their common status, reason, and message.
    ///
    /// # Errors
    ///
    /// Returns [`RegisterError`] for a different operation, malformed result,
    /// or malformed successful payload. Error text never contains payload data.
    pub fn try_from_response_item(item: ResponseBatchItemView<'_>) -> Result<Self, RegisterError> {
        if item.operation() != Some(OPERATION) {
            return Err(RegisterError::UnexpectedOperation);
        }
        let status = item
            .result_status()
            .ok_or(RegisterError::MissingResultStatus)?;
        let result_message = item.with_result_message(|text| ResultMessage::new(text.to_owned()));
        let result = KmipOperationResult::new(status, item.result_reason(), result_message)
            .map_err(RegisterError::InvalidOperationResult)?;
        let response_ttlv = common::clone_response_item(item, "Register")
            .map_err(|_| RegisterError::MalformedResponseItem)?;

        let unique_identifier = if status.raw() == SUCCESS {
            Some(
                item.with_response_payload(|payload| parse_success_identifier(&payload))
                    .ok_or(RegisterError::MalformedSuccessPayload)??,
            )
        } else {
            None
        };

        Ok(Self {
            result,
            response_ttlv,
            unique_identifier,
        })
    }

    /// Returns the exact KMIP operation result.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }

    /// Returns the server-assigned Unique Identifier on success.
    #[must_use]
    pub const fn unique_identifier(&self) -> Option<&UniqueIdentifier> {
        self.unique_identifier.as_ref()
    }

    /// Lends the complete ordered generic response batch item.
    pub fn with_ttlv<R>(&self, callback: impl for<'a> FnOnce(StructureView<'a>) -> R) -> R {
        self.response_ttlv.with_ttlv(callback)
    }
}

/// A payload-free error converting a validated Register response item.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegisterError {
    /// The response item is not the client-to-server Register operation.
    UnexpectedOperation,
    /// The validated response item did not expose a Result Status.
    MissingResultStatus,
    /// The represented KMIP result violates the shared result contract.
    InvalidOperationResult(ResultValidationError),
    /// The generic response item could not be retained.
    MalformedResponseItem,
    /// A successful response omitted, duplicated, or mistyped its identifier.
    MalformedSuccessPayload,
}

impl fmt::Display for RegisterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedOperation => formatter.write_str("response item is not Register"),
            Self::MissingResultStatus => formatter.write_str("Register result status is missing"),
            Self::InvalidOperationResult(cause) => {
                write!(formatter, "Register operation result is invalid: {cause}")
            }
            Self::MalformedResponseItem => {
                formatter.write_str("Register response item is malformed")
            }
            Self::MalformedSuccessPayload => {
                formatter.write_str("successful Register response payload is malformed")
            }
        }
    }
}

impl Error for RegisterError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidOperationResult(cause) => Some(cause),
            _ => None,
        }
    }
}

fn attributes_satisfy_register_requirements(
    object_type: ObjectType,
    attributes: &AttributeSet,
    object: &Item,
    opaque_object_context: Option<OpaqueObjectContext>,
) -> bool {
    if opaque_object_context.is_some() && object_type.raw() != 8 {
        return false;
    }

    match object_type.raw() {
        // Certificate has its own Certificate Length requirement and also
        // follows Table 293's Managed Cryptographic Object attribute pairing.
        1 => {
            has_attribute(attributes, CERTIFICATE_LENGTH)
                && cryptographic_algorithm_and_length_outer_pair_is_present(attributes)
        }
        // Opaque Object has its own Table 4 Structure, independent of whether
        // the caller selects generic or managed-cryptographic context.
        8 => {
            opaque_object_has_required_fields(object)
                && match opaque_object_context {
                    Some(OpaqueObjectContext::ManagedCryptographicObject) => {
                        opaque_managed_crypto_attributes_satisfy_register_requirements(attributes)
                    }
                    Some(OpaqueObjectContext::Generic) | None => true,
                }
        }
        // Table 293 applies to managed cryptographic objects. Secret Data
        // still requires a Usage Mask, but Algorithm and Length do not apply.
        2 | 3 | 4 | 5 | 7 | 9 => {
            managed_crypto_attributes_satisfy_register_requirements(object_type, attributes, object)
        }
        // Certificate Request is a Managed Cryptographic Object under §2.2.
        10 => cryptographic_algorithm_and_length_outer_pair_is_present(attributes),
        // Table 293's certificate-specific signature algorithm may be omitted
        // when it is encoded in the supplied Certificate object. KMIPKit does
        // not parse certificate bytes locally.
        _ => true,
    }
}

fn opaque_managed_crypto_attributes_satisfy_register_requirements(
    attributes: &AttributeSet,
) -> bool {
    has_attribute(attributes, CRYPTOGRAPHIC_USAGE_MASK)
        && cryptographic_algorithm_and_length_satisfy_register_requirements(attributes, None)
}

fn managed_crypto_attributes_satisfy_register_requirements(
    object_type: ObjectType,
    attributes: &AttributeSet,
    object: &Item,
) -> bool {
    if !has_attribute(attributes, CRYPTOGRAPHIC_USAGE_MASK) {
        return false;
    }
    if object_type.raw() == 7 {
        return true;
    }

    cryptographic_algorithm_and_length_satisfy_register_requirements(attributes, Some(object))
}

fn cryptographic_algorithm_and_length_satisfy_register_requirements(
    attributes: &AttributeSet,
    object: Option<&Item>,
) -> bool {
    let has_algorithm = has_attribute(attributes, CRYPTOGRAPHIC_ALGORITHM);

    // §6.1.43 Table 293 couples the outer Register Attributes. Key Block data
    // can satisfy both omitted values, but cannot supply one side of an outer
    // pair whose other side is present in Register Attributes.
    if !cryptographic_algorithm_and_length_outer_pair_is_satisfied(attributes) {
        return false;
    }

    has_algorithm
        || object.is_some_and(|object| {
            key_block_has(object, CRYPTOGRAPHIC_ALGORITHM)
                && key_block_has(object, CRYPTOGRAPHIC_LENGTH)
        })
}

fn cryptographic_algorithm_and_length_outer_pair_is_satisfied(attributes: &AttributeSet) -> bool {
    has_attribute(attributes, CRYPTOGRAPHIC_ALGORITHM)
        == has_attribute(attributes, CRYPTOGRAPHIC_LENGTH)
}

fn cryptographic_algorithm_and_length_outer_pair_is_present(attributes: &AttributeSet) -> bool {
    has_attribute(attributes, CRYPTOGRAPHIC_ALGORITHM)
        && has_attribute(attributes, CRYPTOGRAPHIC_LENGTH)
}

fn opaque_object_has_required_fields(object: &Item) -> bool {
    if object.tag().raw() != OPAQUE_OBJECT {
        return false;
    }

    object.with_value(|value| {
        let ValueView::Structure(fields) = value else {
            return false;
        };

        opaque_object_fields_have_required_fields(&fields)
    })
}

fn opaque_object_fields_have_required_fields(fields: &StructureView<'_>) -> bool {
    let mut has_data_type = false;
    let mut has_data_value = false;
    let fields_have_valid_types = fields
        .children()
        .iter()
        .all(|field| match field.tag().raw() {
            OPAQUE_DATA_TYPE => {
                has_data_type = true;
                field.with_value(|value| matches!(value, ValueView::Enumeration(_)))
            }
            OPAQUE_DATA_VALUE => {
                has_data_value = true;
                field.with_value(|value| matches!(value, ValueView::ByteString(_)))
            }
            _ => true,
        });

    fields_have_valid_types && has_data_type && has_data_value
}

fn has_attribute(attributes: &AttributeSet, raw_tag: u32) -> bool {
    attributes
        .as_items()
        .iter()
        .any(|attribute| attribute.tag().raw() == raw_tag)
}

fn key_block_has(object: &Item, raw_tag: u32) -> bool {
    object.with_value(|value| {
        let ValueView::Structure(object) = value else {
            return false;
        };
        object.children().iter().any(|field| {
            field.tag().raw() == KEY_BLOCK
                && field.with_value(|value| match value {
                    ValueView::Structure(block) => key_block_contains_attribute(&block, raw_tag),
                    _ => false,
                })
        })
    })
}

fn key_block_contains_attribute(block: &StructureView<'_>, raw_tag: u32) -> bool {
    block.children().iter().any(|entry| {
        if entry.tag().raw() == raw_tag {
            return cryptographic_metadata_has_expected_type(entry, raw_tag);
        }
        entry.tag().raw() == KEY_VALUE && key_value_has_attribute(entry, raw_tag)
    })
}

fn key_value_has_attribute(key_value: &Item, raw_tag: u32) -> bool {
    key_value.with_value(|value| {
        let ValueView::Structure(key_value) = value else {
            return false;
        };
        key_value.children().iter().any(|field| {
            field.tag().raw() == ATTRIBUTES
                && field.with_value(|value| match value {
                    ValueView::Structure(attributes) => {
                        attributes.children().iter().any(|attribute| {
                            attribute.tag().raw() == raw_tag
                                && cryptographic_metadata_has_expected_type(attribute, raw_tag)
                        })
                    }
                    _ => false,
                })
        })
    })
}

fn cryptographic_metadata_has_expected_type(attribute: &Item, raw_tag: u32) -> bool {
    attribute.with_value(|value| match raw_tag {
        CRYPTOGRAPHIC_ALGORITHM => matches!(value, ValueView::Enumeration(_)),
        CRYPTOGRAPHIC_LENGTH => matches!(value, ValueView::Integer(_)),
        _ => false,
    })
}

fn parse_success_identifier(
    payload: &StructureView<'_>,
) -> Result<UniqueIdentifier, RegisterError> {
    crate::lifecycle::successful_response_identifier(payload)
        .ok_or(RegisterError::MalformedSuccessPayload)
}

fn push(payload: &mut Structure, raw_tag: u32, value: Value) -> Result<(), ProtocolError> {
    let item = Item::new(tag(raw_tag)?, value).map_err(model_error)?;
    payload.try_push(item).map_err(model_error)
}

fn tag(raw_tag: u32) -> Result<Tag, ProtocolError> {
    RawTag::new(raw_tag)
        .and_then(|raw| raw.try_checked())
        .map_err(model_error)
}

fn model_error(error: ModelError) -> ProtocolError {
    ProtocolError::new(
        ProtocolErrorKind::InvalidValue,
        ProtocolCauseCategory::InvalidValue,
        error,
    )
}

fn invalid_request() -> ProtocolError {
    ProtocolError::categorized(
        ProtocolErrorKind::InvalidInput,
        ProtocolCauseCategory::InvalidValue,
    )
}
