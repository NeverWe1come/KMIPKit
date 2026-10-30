//! Typed KMIP 2.1 Join Split Key request and response payloads.

use std::error::Error;
use std::fmt;

use kmipkit_ttlv::{ModelError, Structure, StructureView, Value, ValueView};

use crate::cryptographic_operation::{self as common, OwnedResponseTtlv};
use crate::{
    AttributeSet, KmipOperationResult, ObjectType, ProtocolCauseCategory, ProtocolError,
    ProtocolErrorKind, ResponseBatchItemView, ResultMessage, ResultValidationError,
    UniqueIdentifier,
};

const OPERATION: u32 = 0x0000_0029; // KMIP v2.1 §11.36, Table 470.
const OBJECT_TYPE: u32 = 0x0042_0057;
const SECRET_DATA_TYPE: u32 = 0x0042_0086;
const ATTRIBUTES: u32 = 0x0042_0125;
const PROTECTION_STORAGE_MASKS: u32 = 0x0042_015F;

/// One transmitted Split Key identifier and its caller-known threshold context.
///
/// The threshold is local validation metadata and is never encoded. It remains
/// optional while a caller assembles references so an incomplete context can
/// be reported before transmission.
pub struct SplitKeyReference {
    unique_identifier: UniqueIdentifier,
    split_key_threshold: Option<u32>,
}

impl SplitKeyReference {
    /// Starts a reference with the exact identifier to transmit.
    #[must_use]
    pub const fn new(unique_identifier: UniqueIdentifier) -> Self {
        Self {
            unique_identifier,
            split_key_threshold: None,
        }
    }

    /// Sets the caller-known Split Key Threshold for this identifier occurrence.
    #[must_use]
    pub const fn with_split_key_threshold(mut self, value: u32) -> Self {
        self.split_key_threshold = Some(value);
        self
    }

    /// Returns the supplied identifier in its original KMIP value form.
    #[must_use]
    pub const fn unique_identifier(&self) -> &UniqueIdentifier {
        &self.unique_identifier
    }

    /// Returns this reference's local threshold context, if supplied.
    #[must_use]
    pub const fn split_key_threshold(&self) -> Option<u32> {
        self.split_key_threshold
    }
}

impl fmt::Debug for SplitKeyReference {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SplitKeyReference")
            .field("unique_identifier", &"[REDACTED]")
            .field("split_key_threshold", &self.split_key_threshold)
            .finish()
    }
}

/// A typed KMIP 2.1 Join Split Key request from §6.1.27, Table 244.
///
/// Each identifier occurrence is paired with the caller-known threshold of
/// that Split Key. The context is validation-only and is not serialized.
pub struct JoinSplitKeyRequest {
    object_type: ObjectType,
    references: Vec<SplitKeyReference>,
    secret_data_type: Option<u32>,
    attributes: Option<AttributeSet>,
    protection_storage_masks: Option<Structure>,
}

impl JoinSplitKeyRequest {
    /// Creates a request with an explicit Object Type and ordered references.
    #[must_use]
    pub const fn new(object_type: ObjectType, references: Vec<SplitKeyReference>) -> Self {
        Self {
            object_type,
            references,
            secret_data_type: None,
            attributes: None,
            protection_storage_masks: None,
        }
    }

    /// Sets the optional Secret Data Type raw Enumeration value.
    #[must_use]
    pub const fn with_secret_data_type(mut self, value: u32) -> Self {
        self.secret_data_type = Some(value);
        self
    }

    /// Sets optional caller-provided object Attributes.
    #[must_use]
    pub fn with_attributes(mut self, value: AttributeSet) -> Self {
        self.attributes = Some(value);
        self
    }

    /// Sets optional generic Protection Storage Masks.
    #[must_use]
    pub fn with_protection_storage_masks(mut self, value: Structure) -> Self {
        self.protection_storage_masks = Some(value);
        self
    }

    /// Returns the selected raw-preserving Object Type.
    #[must_use]
    pub const fn object_type(&self) -> ObjectType {
        self.object_type
    }

    /// Returns each reference and its caller-supplied local threshold context.
    #[must_use]
    pub fn references(&self) -> &[SplitKeyReference] {
        &self.references
    }

    /// Returns the optional raw Secret Data Type Enumeration.
    #[must_use]
    pub const fn secret_data_type(&self) -> Option<u32> {
        self.secret_data_type
    }

    /// Returns optional caller-provided Attributes.
    #[must_use]
    pub const fn attributes(&self) -> Option<&AttributeSet> {
        self.attributes.as_ref()
    }

    /// Returns optional Protection Storage Masks.
    #[must_use]
    pub const fn protection_storage_masks(&self) -> Option<&Structure> {
        self.protection_storage_masks.as_ref()
    }

    /// Encodes Table 244 fields, preserving identifier order and repetitions.
    ///
    /// The caller-known Split Key Threshold for each reference is used only
    /// to validate that the number of transmitted identifier entries is at
    /// least the maximum threshold. No identifier lookup or threshold equality
    /// check occurs.
    ///
    /// # Errors
    /// Returns a payload-free protocol error for source-defined Attribute
    /// cardinality violations, an empty reference list, missing per-entry
    /// threshold context, insufficient identifier count, or a TTLV model
    /// limit.
    pub fn into_ttlv_payload(self) -> Result<Structure, ProtocolError> {
        if let Some(attributes) = self.attributes.as_ref() {
            crate::attribute::validate_single_instance_attribute_items(
                attributes.as_items(),
                Some(self.object_type.raw()),
                true,
            )
            .map_err(|_| common::request_error())?;
        }

        let maximum_threshold = validate_references(&self.references)?;
        let entry_count = self.references.len();
        if usize::try_from(maximum_threshold).map_or(true, |required| entry_count < required) {
            return Err(common::request_error());
        }

        let mut payload = Structure::new();
        common::push(
            &mut payload,
            OBJECT_TYPE,
            Value::enumeration(self.object_type.raw()),
        )?;
        for reference in self.references {
            common::push(
                &mut payload,
                common::UNIQUE_IDENTIFIER,
                common::unique_identifier_value(reference.unique_identifier),
            )?;
        }
        if let Some(value) = self.secret_data_type {
            common::push(&mut payload, SECRET_DATA_TYPE, Value::enumeration(value))?;
        }
        if let Some(attributes) = self.attributes {
            let mut fields = Structure::new();
            for attribute in attributes.into_items() {
                fields.try_push(attribute).map_err(model_error)?;
            }
            common::push(&mut payload, ATTRIBUTES, Value::structure(fields))?;
        }
        if let Some(masks) = self.protection_storage_masks {
            common::push(
                &mut payload,
                PROTECTION_STORAGE_MASKS,
                Value::structure(masks),
            )?;
        }
        Ok(payload)
    }
}

impl fmt::Debug for JoinSplitKeyRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("JoinSplitKeyRequest")
            .field("object_type", &self.object_type)
            .field("references", &self.references)
            .field("secret_data_type", &self.secret_data_type)
            .field(
                "attribute_count",
                &self.attributes.as_ref().map(AttributeSet::len),
            )
            .field(
                "has_protection_storage_masks",
                &self.protection_storage_masks.is_some(),
            )
            .finish()
    }
}

/// A typed Join Split Key result from §6.1.27, Tables 245–246.
pub struct JoinSplitKeyResponse {
    result: KmipOperationResult,
    unique_identifier: Option<UniqueIdentifier>,
    response_ttlv: OwnedResponseTtlv,
}

impl JoinSplitKeyResponse {
    /// Converts one validated Join Split Key response item and retains its TTLV.
    ///
    /// # Errors
    /// Returns a payload-free [`JoinSplitKeyError`] for a different operation,
    /// invalid common result fields, or malformed successful payload.
    pub fn try_from_response_item(
        item: ResponseBatchItemView<'_>,
    ) -> Result<Self, JoinSplitKeyError> {
        if item.operation() != Some(OPERATION) {
            return Err(JoinSplitKeyError::UnexpectedOperation);
        }
        let status = item
            .result_status()
            .ok_or(JoinSplitKeyError::MissingResultStatus)?;
        let result = KmipOperationResult::new(
            status,
            item.result_reason(),
            item.with_result_message(|message| ResultMessage::new(message.to_owned())),
        )
        .map_err(JoinSplitKeyError::InvalidOperationResult)?;
        let response_ttlv = common::clone_response_item(item, "Join Split Key")
            .map_err(|_| JoinSplitKeyError::MalformedResponseItem)?;
        let unique_identifier = if status.raw() == common::SUCCESS {
            Some(
                item.with_response_payload(|payload| parse_success_payload(&payload))
                    .ok_or(JoinSplitKeyError::MissingSuccessPayload)??,
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

    /// Returns the joined object's Unique Identifier on success.
    #[must_use]
    pub const fn unique_identifier(&self) -> Option<&UniqueIdentifier> {
        self.unique_identifier.as_ref()
    }

    /// Lends the complete ordered response item, including unknown fields.
    pub fn with_ttlv<R>(&self, callback: impl for<'a> FnOnce(StructureView<'a>) -> R) -> R {
        self.response_ttlv.with_ttlv(callback)
    }
}

impl fmt::Debug for JoinSplitKeyResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("JoinSplitKeyResponse")
            .field("result", &self.result)
            .field("has_unique_identifier", &self.unique_identifier.is_some())
            .field("response_ttlv", &"[REDACTED]")
            .finish()
    }
}

/// A payload-free Join Split Key response conversion error.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JoinSplitKeyError {
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

impl fmt::Display for JoinSplitKeyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedOperation => formatter.write_str("response item is not Join Split Key"),
            Self::MissingResultStatus => {
                formatter.write_str("Join Split Key result status is missing")
            }
            Self::InvalidOperationResult(error) => {
                write!(formatter, "Join Split Key result is invalid: {error}")
            }
            Self::MissingSuccessPayload => {
                formatter.write_str("successful Join Split Key result has no payload")
            }
            Self::MalformedSuccessPayload => {
                formatter.write_str("Join Split Key success payload is malformed")
            }
            Self::MalformedResponseItem => {
                formatter.write_str("Join Split Key response item is malformed")
            }
        }
    }
}

impl Error for JoinSplitKeyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidOperationResult(error) => Some(error),
            _ => None,
        }
    }
}

fn validate_references(references: &[SplitKeyReference]) -> Result<u32, ProtocolError> {
    if references.is_empty() {
        return Err(common::request_error());
    }
    let mut maximum_threshold = 0;
    for reference in references {
        let Some(threshold) = reference.split_key_threshold else {
            return Err(common::request_error());
        };
        maximum_threshold = maximum_threshold.max(threshold);
    }
    Ok(maximum_threshold)
}

fn parse_success_payload(
    payload: &StructureView<'_>,
) -> Result<UniqueIdentifier, JoinSplitKeyError> {
    let mut unique_identifier = None;
    for field in payload.children() {
        if field.tag().raw() != common::UNIQUE_IDENTIFIER {
            continue;
        }
        if unique_identifier.is_some() {
            return Err(JoinSplitKeyError::MalformedSuccessPayload);
        }
        unique_identifier = field.with_value(|value| parse_unique_identifier(&value));
        if unique_identifier.is_none() {
            return Err(JoinSplitKeyError::MalformedSuccessPayload);
        }
    }
    unique_identifier.ok_or(JoinSplitKeyError::MalformedSuccessPayload)
}

fn parse_unique_identifier(value: &ValueView<'_>) -> Option<UniqueIdentifier> {
    match value {
        ValueView::TextString(value) => Some(UniqueIdentifier::TextString((*value).to_owned())),
        ValueView::Enumeration(value) => Some(UniqueIdentifier::Enumeration(**value)),
        ValueView::Integer(value) => Some(UniqueIdentifier::Integer(**value)),
        _ => None,
    }
}

fn model_error(error: ModelError) -> ProtocolError {
    ProtocolError::new(
        ProtocolErrorKind::InvalidValue,
        ProtocolCauseCategory::InvalidValue,
        error,
    )
}
