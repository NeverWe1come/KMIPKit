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
const SUCCESS: u32 = 0;

/// A KMIP 2.1 Register request from §6.1.43, Table 291.
///
/// The caller supplies the object type, its direct Attributes, and the full
/// object item. `KMIPKit` preserves their order and unknown values. It checks
/// only the source-defined local Table 293 attribute requirements; the server
/// remains authoritative for object validity, policy, and registration.
pub struct RegisterRequest {
    object_type: ObjectType,
    attributes: AttributeSet,
    object: Item,
    protection_storage_masks: Option<Structure>,
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
        }
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

    /// Builds the ordered Request Payload from §6.1.43, Table 291.
    ///
    /// # Errors
    ///
    /// Returns a sanitized error if the object is not a Structure, a Table 293
    /// required attribute is absent, paired key attributes disagree, or the
    /// generic TTLV model cannot represent the payload.
    pub fn into_ttlv_payload(self) -> Result<Structure, ProtocolError> {
        if !matches!(self.object.item_type(), kmipkit_ttlv::ItemType::Structure)
            || !attributes_satisfy_register_requirements(
                self.object_type,
                &self.attributes,
                &self.object,
            )
        {
            return Err(invalid_request());
        }

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
) -> bool {
    let has_algorithm = has_attribute(attributes, CRYPTOGRAPHIC_ALGORITHM);
    let has_length = has_attribute(attributes, CRYPTOGRAPHIC_LENGTH);
    let has_usage_mask = has_attribute(attributes, CRYPTOGRAPHIC_USAGE_MASK);

    match object_type.raw() {
        // Certificate Length is required for a Certificate. Table 293 does
        // not apply the managed-cryptographic-object requirements to it.
        1 => has_attribute(attributes, CERTIFICATE_LENGTH),
        // Table 293 applies to managed cryptographic objects. Secret Data
        // still requires a Usage Mask, but Algorithm and Length do not apply.
        2 | 3 | 4 | 5 | 7 | 9 => {
            if !has_usage_mask {
                return false;
            }
            if object_type.raw() == 7 {
                return true;
            }
            let algorithm_available =
                has_algorithm || key_block_has(object, CRYPTOGRAPHIC_ALGORITHM);
            let length_available = has_length || key_block_has(object, CRYPTOGRAPHIC_LENGTH);
            algorithm_available && length_available
        }
        // Table 293's certificate-specific signature algorithm may be omitted
        // when it is encoded in the supplied Certificate object. KMIPKit does
        // not parse certificate bytes locally.
        _ => true,
    }
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
                    ValueView::Structure(block) => block.children().iter().any(|entry| {
                        entry.tag().raw() == raw_tag
                            || (entry.tag().raw() == KEY_VALUE
                                && key_value_has_attribute(entry, raw_tag))
                    }),
                    _ => false,
                })
        })
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
                    ValueView::Structure(attributes) => attributes
                        .children()
                        .iter()
                        .any(|attribute| attribute.tag().raw() == raw_tag),
                    _ => false,
                })
        })
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
