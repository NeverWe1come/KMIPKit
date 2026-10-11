//! Typed KMIP 2.1 Locate request and response payloads.

use std::error::Error;
use std::fmt;

use kmipkit_ttlv::{Item, ModelError, RawTag, Structure, StructureView, Tag, Value, ValueView};

use crate::{
    AttributeSet, KmipOperationResult, ProtocolCauseCategory, ProtocolError, ProtocolErrorKind,
    ResponseBatchItemView, ResultMessage, ResultValidationError, UniqueIdentifier,
};

const LOCATE_OPERATION: u32 = 0x0000_0008;
const MAXIMUM_ITEMS: u32 = 0x0042_004F;
const OFFSET_ITEMS: u32 = 0x0042_00D4;
const STORAGE_STATUS_MASK: u32 = 0x0042_008E;
const OBJECT_GROUP_MEMBER: u32 = 0x0042_00AC;
const ATTRIBUTES: u32 = 0x0042_0125;
const LOCATED_ITEMS: u32 = 0x0042_00D5;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const SUCCESS: u32 = 0;

/// A KMIP Storage Status Mask that preserves all raw bits, including extensions.
///
/// OASIS KMIP v2.1 §12.3 assigns the on-line, archival, and destroyed bits.
/// This wrapper does not validate or discard other bits.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct StorageStatusMask(u32);

impl StorageStatusMask {
    /// Creates a Storage Status Mask from its exact raw 32-bit pattern.
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Returns the exact raw 32-bit pattern.
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// A KMIP Object Group Member Enumeration that preserves unrecognized values.
///
/// OASIS KMIP v2.1 §11.33 defines the assigned values. Locate forwards the
/// caller's selector; it does not choose or create a group member locally.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ObjectGroupMember(u32);

impl ObjectGroupMember {
    /// Creates an Object Group Member from its raw unsigned Enumeration value.
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

/// A typed KMIP 2.1 Locate request payload from §6.1.28, Table 247.
///
/// The required Attributes Structure is retained even when empty. The client
/// sends criteria to the server and does not evaluate them against local state.
pub struct LocateRequest {
    maximum_items: Option<i32>,
    offset_items: Option<i32>,
    storage_status_mask: Option<StorageStatusMask>,
    object_group_member: Option<ObjectGroupMember>,
    attributes: AttributeSet,
}

impl LocateRequest {
    /// Creates a Locate request with its required Attributes Structure.
    #[must_use]
    pub const fn new(attributes: AttributeSet) -> Self {
        Self {
            maximum_items: None,
            offset_items: None,
            storage_status_mask: None,
            object_group_member: None,
            attributes,
        }
    }

    /// Sets the optional Maximum Items Integer without changing its value.
    #[must_use]
    pub const fn with_maximum_items(mut self, maximum_items: i32) -> Self {
        self.maximum_items = Some(maximum_items);
        self
    }

    /// Sets the optional Offset Items Integer, preserving an explicit zero.
    #[must_use]
    pub const fn with_offset_items(mut self, offset_items: i32) -> Self {
        self.offset_items = Some(offset_items);
        self
    }

    /// Sets the optional Storage Status Mask, including unknown raw bits.
    #[must_use]
    pub const fn with_storage_status_mask(mut self, mask: StorageStatusMask) -> Self {
        self.storage_status_mask = Some(mask);
        self
    }

    /// Sets the optional Object Group Member Enumeration, including future values.
    #[must_use]
    pub const fn with_object_group_member(mut self, member: ObjectGroupMember) -> Self {
        self.object_group_member = Some(member);
        self
    }

    /// Returns the optional Maximum Items Integer.
    #[must_use]
    pub const fn maximum_items(&self) -> Option<i32> {
        self.maximum_items
    }

    /// Returns the optional Offset Items Integer, including explicit zero.
    #[must_use]
    pub const fn offset_items(&self) -> Option<i32> {
        self.offset_items
    }

    /// Returns the optional raw-preserving Storage Status Mask.
    #[must_use]
    pub const fn storage_status_mask(&self) -> Option<StorageStatusMask> {
        self.storage_status_mask
    }

    /// Returns the optional raw-preserving Object Group Member value.
    #[must_use]
    pub const fn object_group_member(&self) -> Option<ObjectGroupMember> {
        self.object_group_member
    }

    /// Returns the required ordered direct-item Attributes Structure contents.
    #[must_use]
    pub const fn attributes(&self) -> &AttributeSet {
        &self.attributes
    }

    /// Builds the ordered Request Payload Structure from Table 247.
    ///
    /// This consumes the validated criteria and moves each Attribute item into
    /// the required outer Attributes Structure without changing its value,
    /// repetition, or order.
    ///
    /// # Errors
    ///
    /// Returns a sanitized protocol error if a tag or Structure violates the
    /// generic TTLV model constraints.
    pub fn to_ttlv_payload(self) -> Result<Structure, ProtocolError> {
        let mut payload = Structure::new();
        if let Some(maximum_items) = self.maximum_items {
            payload
                .try_push(item(MAXIMUM_ITEMS, Value::integer(maximum_items))?)
                .map_err(protocol_model_error)?;
        }
        if let Some(offset_items) = self.offset_items {
            payload
                .try_push(item(OFFSET_ITEMS, Value::integer(offset_items))?)
                .map_err(protocol_model_error)?;
        }
        if let Some(mask) = self.storage_status_mask {
            payload
                .try_push(item(
                    STORAGE_STATUS_MASK,
                    Value::integer(mask.raw().cast_signed()),
                )?)
                .map_err(protocol_model_error)?;
        }
        if let Some(member) = self.object_group_member {
            payload
                .try_push(item(OBJECT_GROUP_MEMBER, Value::enumeration(member.raw()))?)
                .map_err(protocol_model_error)?;
        }

        let mut attributes = Structure::new();
        for attribute in self.attributes.into_items() {
            attributes
                .try_push(attribute)
                .map_err(protocol_model_error)?;
        }
        payload
            .try_push(item(ATTRIBUTES, Value::structure(attributes))?)
            .map_err(protocol_model_error)?;
        Ok(payload)
    }
}

impl fmt::Debug for LocateRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("LocateRequest([REDACTED])")
    }
}

/// The typed result of one KMIP 2.1 Locate response from §6.1.28,
/// Tables 248–249.
pub struct LocateResponse {
    result: KmipOperationResult,
    located_items: Option<i32>,
    unique_identifiers: Vec<UniqueIdentifier>,
}

impl LocateResponse {
    /// Converts one already validated KMIP Locate response batch item.
    ///
    /// Successful results retain optional Located Items and every Unique
    /// Identifier in wire order. Non-success results retain only the common
    /// operation result, including any raw-preserving Result Reason.
    ///
    /// # Errors
    ///
    /// Returns a payload-free [`LocateError`] for a different operation,
    /// invalid result metadata, a missing success payload, or a malformed
    /// Table 248 field.
    pub fn try_from_response_item(item: ResponseBatchItemView<'_>) -> Result<Self, LocateError> {
        if item.operation() != Some(LOCATE_OPERATION) {
            return Err(LocateError::UnexpectedOperation);
        }
        let status = item
            .result_status()
            .ok_or(LocateError::MissingResultStatus)?;
        let result_message = item.with_result_message(|text| ResultMessage::new(text.to_owned()));
        let result = KmipOperationResult::new(status, item.result_reason(), result_message)
            .map_err(LocateError::InvalidOperationResult)?;

        if status.raw() != SUCCESS {
            return Ok(Self {
                result,
                located_items: None,
                unique_identifiers: Vec::new(),
            });
        }

        let (located_items, unique_identifiers) = item
            .with_response_payload(|payload| parse_success_payload(&payload))
            .ok_or(LocateError::MissingSuccessPayload)??;
        Ok(Self {
            result,
            located_items,
            unique_identifiers,
        })
    }

    /// Returns the complete KMIP result, including status and optional reason
    /// and redacted Result Message.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }

    /// Returns the optional Located Items count without converting its value.
    #[must_use]
    pub const fn located_items(&self) -> Option<i32> {
        self.located_items
    }

    /// Returns all response Unique Identifiers in their original wire order.
    ///
    /// Repeated and duplicate identifiers are retained as received.
    #[must_use]
    pub fn unique_identifiers(&self) -> &[UniqueIdentifier] {
        &self.unique_identifiers
    }
}

impl fmt::Debug for LocateResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LocateResponse")
            .field("result", &self.result)
            .field("located_items", &self.located_items)
            .field("unique_identifiers", &self.unique_identifiers)
            .finish()
    }
}

/// A payload-free Locate response conversion error.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LocateError {
    /// The response identifies a different operation.
    UnexpectedOperation,
    /// The validated response omitted Result Status.
    MissingResultStatus,
    /// The Result Status/Reason combination violates the common KMIP contract.
    InvalidOperationResult(ResultValidationError),
    /// A successful response omitted its Response Payload Structure.
    MissingSuccessPayload,
    /// A successful payload contains a duplicate, mistyped, or out-of-order field.
    MalformedSuccessPayload,
}

impl fmt::Display for LocateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedOperation => formatter.write_str("response item is not Locate"),
            Self::MissingResultStatus => formatter.write_str("Locate result status is missing"),
            Self::InvalidOperationResult(cause) => {
                write!(formatter, "Locate operation result is invalid: {cause}")
            }
            Self::MissingSuccessPayload => {
                formatter.write_str("successful Locate response has no payload")
            }
            Self::MalformedSuccessPayload => {
                formatter.write_str("Locate response payload is malformed")
            }
        }
    }
}

impl Error for LocateError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidOperationResult(cause) => Some(cause),
            _ => None,
        }
    }
}

fn parse_success_payload(
    payload: &StructureView<'_>,
) -> Result<(Option<i32>, Vec<UniqueIdentifier>), LocateError> {
    let mut located_items = None;
    let mut unique_identifiers = Vec::new();
    let mut saw_unique_identifier = false;

    for field in payload.children() {
        match field.tag().raw() {
            LOCATED_ITEMS => {
                if located_items.is_some() || saw_unique_identifier {
                    return Err(LocateError::MalformedSuccessPayload);
                }
                located_items = field.with_value(|value| match value {
                    ValueView::Integer(value) => Some(*value),
                    _ => None,
                });
                if located_items.is_none() {
                    return Err(LocateError::MalformedSuccessPayload);
                }
            }
            UNIQUE_IDENTIFIER => {
                saw_unique_identifier = true;
                let identifier = field
                    .with_value(|value| parse_unique_identifier(&value))
                    .ok_or(LocateError::MalformedSuccessPayload)?;
                unique_identifiers.push(identifier);
            }
            _ => return Err(LocateError::MalformedSuccessPayload),
        }
    }

    Ok((located_items, unique_identifiers))
}

fn parse_unique_identifier(value: &ValueView<'_>) -> Option<UniqueIdentifier> {
    match value {
        ValueView::TextString(value) => Some(UniqueIdentifier::TextString((*value).to_owned())),
        ValueView::Enumeration(value) => Some(UniqueIdentifier::Enumeration(**value)),
        ValueView::Integer(value) => Some(UniqueIdentifier::Integer(**value)),
        _ => None,
    }
}

fn item(raw_tag: u32, value: Value) -> Result<Item, ProtocolError> {
    Item::new(tag(raw_tag)?, value).map_err(protocol_model_error)
}

fn tag(raw_tag: u32) -> Result<Tag, ProtocolError> {
    RawTag::new(raw_tag)
        .and_then(|raw| raw.try_checked())
        .map_err(protocol_model_error)
}

fn protocol_model_error(error: ModelError) -> ProtocolError {
    ProtocolError::new(
        ProtocolErrorKind::InvalidValue,
        ProtocolCauseCategory::InvalidValue,
        error,
    )
}
