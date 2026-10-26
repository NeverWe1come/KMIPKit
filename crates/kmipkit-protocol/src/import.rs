//! Typed KMIP 2.1 Import request and response payloads.

use std::error::Error;
use std::fmt;

use kmipkit_ttlv::{
    Item, ItemType, ModelError, RawTag, Structure, StructureView, Tag, Value, ValueView,
};

use crate::cryptographic_operation::{self as common, OwnedResponseTtlv};
use crate::{
    AttributeSet, KeyWrapType, KmipOperationResult, ObjectType, ProtocolCauseCategory,
    ProtocolError, ProtocolErrorKind, ResponseBatchItemView, ResultMessage, ResultValidationError,
    UniqueIdentifier,
};

const OPERATION: u32 = 0x0000_002A;
const SUCCESS: u32 = 0;
const SYMMETRIC_KEY: u32 = 2;
const PUBLIC_KEY: u32 = 3;
const PRIVATE_KEY: u32 = 4;
const SPLIT_KEY: u32 = 5;
const PGP_KEY: u32 = 9;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const OBJECT_TYPE: u32 = 0x0042_0057;
const KEY_BLOCK: u32 = 0x0042_0040;
const KEY_WRAPPING_DATA: u32 = 0x0042_0046;
const REPLACE_EXISTING: u32 = 0x0042_0124;
const KEY_WRAP_TYPE: u32 = 0x0042_00F8;
const ATTRIBUTES: u32 = 0x0042_0125;

/// A typed KMIP 2.1 Import request from §6.1.25, Table 238.
///
/// The caller supplies the identifier, object type, Attributes, and complete
/// Any Object. An optional Key Wrap Type is caller-selected for a wrapped key;
/// `KMIPKit` preserves the provided object representation and performs no
/// local unwrap or import. For caller-supplied wrapped Key Values, the caller
/// is responsible for providing IV/Counter/Nonce when the selected wrapping
/// method requires it (KMIP Specification v2.1 §3.3, Table 14), and for
/// TTLV-encoding the Key Value Structure before wrapping it when Encoding
/// Option is absent (§3.3, Table 15). `KMIPKit` does not infer referenced-key
/// parameters or inspect wrapped plaintext or ciphertext.
pub struct ImportRequest {
    unique_identifier: UniqueIdentifier,
    object_type: ObjectType,
    replace_existing: Option<bool>,
    key_wrap_type: Option<KeyWrapType>,
    attributes: AttributeSet,
    object: Item,
}

impl ImportRequest {
    /// Creates an Import request with all required caller-provided values.
    #[must_use]
    pub const fn new(
        unique_identifier: UniqueIdentifier,
        object_type: ObjectType,
        attributes: AttributeSet,
        object: Item,
    ) -> Self {
        Self {
            unique_identifier,
            object_type,
            replace_existing: None,
            key_wrap_type: None,
            attributes,
            object,
        }
    }

    /// Sets whether a pre-existing object with this identifier is replaced.
    ///
    /// Leave this absent when replacement is not the caller's explicit intent.
    #[must_use]
    pub const fn with_replace_existing(mut self, value: bool) -> Self {
        self.replace_existing = Some(value);
        self
    }

    /// Sets the optional Key Wrap Type for a caller-provided wrapped key.
    ///
    /// The raw Enumeration is retained, including values introduced by future
    /// KMIP versions or vendors.
    #[must_use]
    pub const fn with_key_wrap_type(mut self, value: KeyWrapType) -> Self {
        self.key_wrap_type = Some(value);
        self
    }

    /// Returns the caller-supplied Unique Identifier.
    #[must_use]
    pub const fn unique_identifier(&self) -> &UniqueIdentifier {
        &self.unique_identifier
    }

    /// Returns the raw-preserving Object Type.
    #[must_use]
    pub const fn object_type(&self) -> ObjectType {
        self.object_type
    }

    /// Returns the explicit Replace Existing choice, if set.
    #[must_use]
    pub const fn replace_existing(&self) -> Option<bool> {
        self.replace_existing
    }

    /// Returns the explicit Key Wrap Type choice, if set.
    #[must_use]
    pub const fn key_wrap_type(&self) -> Option<KeyWrapType> {
        self.key_wrap_type
    }

    /// Returns the exact direct Attributes supplied by the caller.
    #[must_use]
    pub const fn attributes(&self) -> &AttributeSet {
        &self.attributes
    }

    /// Returns the supplied Any Object without exposing payload in formatting.
    #[must_use]
    pub const fn object(&self) -> &Item {
        &self.object
    }

    /// Encodes the ordered Table 238 fields without changing supplied data.
    ///
    /// # Errors
    /// Returns a payload-free protocol error when the Any Object is not a
    /// Structure, Key Wrap Type does not match the wrapped-key state, or a
    /// TTLV model limit is exceeded.
    pub fn into_ttlv_payload(self) -> Result<Structure, ProtocolError> {
        if self.object.item_type() != ItemType::Structure
            || !key_wrap_type_matches_object(
                self.object_type,
                &self.object,
                self.key_wrap_type.is_some(),
            )
        {
            return Err(invalid_request());
        }

        let mut payload = Structure::new();
        push(
            &mut payload,
            UNIQUE_IDENTIFIER,
            common::unique_identifier_value(self.unique_identifier),
        )?;
        push(
            &mut payload,
            OBJECT_TYPE,
            Value::enumeration(self.object_type.raw()),
        )?;
        append_optional_fields(&mut payload, self.replace_existing, self.key_wrap_type)?;
        append_attributes(&mut payload, self.attributes)?;
        payload.try_push(self.object).map_err(model_error)?;
        Ok(payload)
    }
}

fn append_optional_fields(
    payload: &mut Structure,
    replace_existing: Option<bool>,
    key_wrap_type: Option<KeyWrapType>,
) -> Result<(), ProtocolError> {
    if let Some(value) = replace_existing {
        push(payload, REPLACE_EXISTING, Value::boolean(value))?;
    }
    if let Some(value) = key_wrap_type {
        push(payload, KEY_WRAP_TYPE, Value::enumeration(value.raw()))?;
    }
    Ok(())
}

fn append_attributes(
    payload: &mut Structure,
    attributes: AttributeSet,
) -> Result<(), ProtocolError> {
    let mut fields = Structure::new();
    for attribute in attributes.into_items() {
        fields.try_push(attribute).map_err(model_error)?;
    }
    push(payload, ATTRIBUTES, Value::structure(fields))
}

fn key_wrap_type_matches_object(
    object_type: ObjectType,
    object: &Item,
    key_wrap_type_is_present: bool,
) -> bool {
    let is_key_object = matches!(
        object_type.raw(),
        SYMMETRIC_KEY | PUBLIC_KEY | PRIVATE_KEY | SPLIT_KEY | PGP_KEY
    );
    if !is_key_object {
        return !key_wrap_type_is_present;
    }
    key_wrap_type_is_present == key_block_has(object, KEY_WRAPPING_DATA)
}

fn key_block_has(object: &Item, raw_tag: u32) -> bool {
    object.with_value(|value| {
        let ValueView::Structure(object) = value else {
            return false;
        };
        object.children().iter().any(|field| {
            field.tag().raw() == KEY_BLOCK
                && field.with_value(|value| match value {
                    ValueView::Structure(block) => block
                        .children()
                        .iter()
                        .any(|entry| entry.tag().raw() == raw_tag),
                    _ => false,
                })
        })
    })
}

impl fmt::Debug for ImportRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ImportRequest")
            .field("unique_identifier", &"[REDACTED]")
            .field("object_type", &self.object_type)
            .field("replace_existing", &self.replace_existing)
            .field("key_wrap_type", &self.key_wrap_type)
            .field("attribute_count", &self.attributes.len())
            .field("object", &"[REDACTED]")
            .finish()
    }
}

/// A typed Import result from §6.1.25, Tables 239–240.
///
/// The response retains its full generic TTLV item for inspection. `KMIPKit`-
/// owned response copies are zeroized when dropped.
pub struct ImportResponse {
    result: KmipOperationResult,
    unique_identifier: Option<UniqueIdentifier>,
    response_ttlv: OwnedResponseTtlv,
}

impl ImportResponse {
    /// Converts one validated Import result and retains its complete response item.
    ///
    /// # Errors
    /// Returns a payload-free [`ImportError`] for an unexpected operation,
    /// invalid common result fields, or malformed successful payload.
    pub fn try_from_response_item(item: ResponseBatchItemView<'_>) -> Result<Self, ImportError> {
        if item.operation() != Some(OPERATION) {
            return Err(ImportError::UnexpectedOperation);
        }
        let status = item
            .result_status()
            .ok_or(ImportError::MissingResultStatus)?;
        let result = KmipOperationResult::new(
            status,
            item.result_reason(),
            item.with_result_message(|message| ResultMessage::new(message.to_owned())),
        )
        .map_err(ImportError::InvalidOperationResult)?;
        let response_ttlv = common::clone_response_item(item, "Import")
            .map_err(|_| ImportError::MalformedResponseItem)?;
        let unique_identifier = if status.raw() == SUCCESS {
            Some(
                item.with_response_payload(|payload| parse_success_payload(&payload))
                    .ok_or(ImportError::MissingSuccessPayload)??,
            )
        } else {
            None
        };
        Ok(Self {
            result,
            unique_identifier,
            response_ttlv,
        })
    }

    /// Returns the exact KMIP status, reason, and optional result message.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }

    /// Returns the resulting object's Unique Identifier on success.
    #[must_use]
    pub const fn unique_identifier(&self) -> Option<&UniqueIdentifier> {
        self.unique_identifier.as_ref()
    }

    /// Lends the complete ordered response item, including unknown fields.
    pub fn with_ttlv<R>(&self, callback: impl for<'a> FnOnce(StructureView<'a>) -> R) -> R {
        self.response_ttlv.with_ttlv(callback)
    }
}

impl fmt::Debug for ImportResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ImportResponse")
            .field("result", &self.result)
            .field("has_unique_identifier", &self.unique_identifier.is_some())
            .field("response_ttlv", &"[REDACTED]")
            .finish()
    }
}

/// A payload-free Import request or response conversion error.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImportError {
    /// The response item identifies another operation.
    UnexpectedOperation,
    /// The validated response omitted Result Status.
    MissingResultStatus,
    /// The common KMIP result fields are inconsistent.
    InvalidOperationResult(ResultValidationError),
    /// A successful response omitted its Response Payload.
    MissingSuccessPayload,
    /// The success payload omitted, duplicated, or mistyped its identifier.
    MalformedSuccessPayload,
    /// The complete response item could not be retained.
    MalformedResponseItem,
}

impl fmt::Display for ImportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedOperation => formatter.write_str("response item is not Import"),
            Self::MissingResultStatus => formatter.write_str("Import result status is missing"),
            Self::InvalidOperationResult(error) => {
                write!(formatter, "Import result is invalid: {error}")
            }
            Self::MissingSuccessPayload => {
                formatter.write_str("successful Import result has no payload")
            }
            Self::MalformedSuccessPayload => {
                formatter.write_str("Import success payload is malformed")
            }
            Self::MalformedResponseItem => formatter.write_str("Import response item is malformed"),
        }
    }
}

impl Error for ImportError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidOperationResult(error) => Some(error),
            _ => None,
        }
    }
}

fn parse_success_payload(payload: &StructureView<'_>) -> Result<UniqueIdentifier, ImportError> {
    let mut unique_identifier = None;
    for field in payload.children() {
        if field.tag().raw() != UNIQUE_IDENTIFIER {
            continue;
        }
        if unique_identifier.is_some() {
            return Err(ImportError::MalformedSuccessPayload);
        }
        unique_identifier = field.with_value(|value| parse_unique_identifier(&value));
        if unique_identifier.is_none() {
            return Err(ImportError::MalformedSuccessPayload);
        }
    }
    unique_identifier.ok_or(ImportError::MalformedSuccessPayload)
}

fn parse_unique_identifier(value: &ValueView<'_>) -> Option<UniqueIdentifier> {
    match value {
        ValueView::TextString(value) => Some(UniqueIdentifier::TextString((*value).to_owned())),
        ValueView::Enumeration(value) => Some(UniqueIdentifier::Enumeration(**value)),
        ValueView::Integer(value) => Some(UniqueIdentifier::Integer(**value)),
        _ => None,
    }
}

fn push(payload: &mut Structure, raw_tag: u32, value: Value) -> Result<(), ProtocolError> {
    payload
        .try_push(Item::new(tag(raw_tag)?, value).map_err(model_error)?)
        .map_err(model_error)
}

fn tag(raw: u32) -> Result<Tag, ProtocolError> {
    RawTag::new(raw)
        .and_then(|raw| raw.try_checked())
        .map_err(model_error)
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
