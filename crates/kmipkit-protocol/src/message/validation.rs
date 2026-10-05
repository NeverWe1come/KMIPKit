//! Payload-free message shape validation.

use std::collections::HashSet;
use std::error::Error;
use std::fmt;

use kmipkit_ttlv::{Item, ItemType, StructureView, ValueView};

use crate::{KmipOperationResult, ResultReason, ResultStatus};

use super::version::ProtocolVersion;

const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
const ASYNCHRONOUS_INDICATOR: u32 = 0x0042_0007;
const ASYNCHRONOUS_CORRELATION_VALUE: u32 = 0x0042_0006;
const ATTESTATION_CAPABLE_INDICATOR: u32 = 0x0042_00D3;
const ATTESTATION_TYPE: u32 = 0x0042_00C7;
const AUTHENTICATION: u32 = 0x0042_000C;
const BATCH_COUNT: u32 = 0x0042_000D;
const BATCH_ERROR_CONTINUATION_OPTION: u32 = 0x0042_000E;
const BATCH_ITEM: u32 = 0x0042_000F;
const BATCH_ORDER_OPTION: u32 = 0x0042_0010;
const CLIENT_CORRELATION_VALUE: u32 = 0x0042_0105;
const CRITICALITY_INDICATOR: u32 = 0x0042_0026;
const EPHEMERAL: u32 = 0x0042_0154;
const MAXIMUM_RESPONSE_SIZE: u32 = 0x0042_0050;
const MESSAGE_EXTENSION: u32 = 0x0042_0051;
const NONCE: u32 = 0x0042_00C8;
const OPERATION: u32 = 0x0042_005C;
const REQUEST_HEADER: u32 = 0x0042_0077;
const REQUEST_PAYLOAD: u32 = 0x0042_0079;
const RESPONSE_HEADER: u32 = 0x0042_007A;
const RESPONSE_PAYLOAD: u32 = 0x0042_007C;
const RESULT_MESSAGE: u32 = 0x0042_007D;
const RESULT_REASON: u32 = 0x0042_007E;
const RESULT_STATUS: u32 = 0x0042_007F;
const SERVER_CORRELATION_VALUE: u32 = 0x0042_0106;
const SERVER_HASHED_PASSWORD: u32 = 0x0042_0155;
const TIME_STAMP: u32 = 0x0042_0092;
const UNIQUE_BATCH_ITEM_ID: u32 = 0x0042_0093;
const VENDOR_EXTENSION: u32 = 0x0042_009C;
const VENDOR_IDENTIFICATION: u32 = 0x0042_009D;

/// Safe category for a rejected request/response message structure.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MessageValidationErrorKind {
    /// A required message, header, batch item, or field is absent.
    MissingRequiredField,
    /// A singleton message/header/batch field appears more than once.
    DuplicateField,
    /// A known field appears before an earlier field in its KMIP table.
    FieldOutOfOrder,
    /// A known field has a TTLV Item Type different from its table type.
    WrongItemType,
    /// Batch Count is negative, zero, or otherwise outside the represented range.
    InvalidBatchCount,
    /// Batch Count does not equal the number of Batch Items.
    BatchCountMismatch,
    /// A batch option appears on a single-item request.
    InvalidSingleItemOption,
    /// A result's reason, message, payload, or asynchronous fields conflict.
    InvalidResult,
    /// A represented value violates a structural field constraint.
    InvalidFieldValue,
}

impl fmt::Display for MessageValidationErrorKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::MissingRequiredField => "required message field is missing",
            Self::DuplicateField => "message field is duplicated",
            Self::FieldOutOfOrder => "message field is out of order",
            Self::WrongItemType => "message field has the wrong TTLV type",
            Self::InvalidBatchCount => "message Batch Count is invalid",
            Self::BatchCountMismatch => "message Batch Count does not match its items",
            Self::InvalidSingleItemOption => "batch option is invalid for a single-item message",
            Self::InvalidResult => "response result fields are inconsistent",
            Self::InvalidFieldValue => "message field value is invalid",
        };
        formatter.write_str(message)
    }
}

/// A structural error that retains only a safe category and numeric indexes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MessageValidationError {
    kind: MessageValidationErrorKind,
    structure_index: Option<usize>,
    field_index: Option<usize>,
}

impl MessageValidationError {
    const fn new(
        kind: MessageValidationErrorKind,
        structure_index: Option<usize>,
        field_index: Option<usize>,
    ) -> Self {
        Self {
            kind,
            structure_index,
            field_index,
        }
    }

    /// Returns the safe structural failure category.
    #[must_use]
    pub const fn kind(self) -> MessageValidationErrorKind {
        self.kind
    }

    /// Returns the top-level Structure item index, when available.
    #[must_use]
    pub const fn structure_index(self) -> Option<usize> {
        self.structure_index
    }

    /// Returns the field index within the reported Structure, when available.
    #[must_use]
    pub const fn field_index(self) -> Option<usize> {
        self.field_index
    }
}

impl fmt::Display for MessageValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.kind)?;
        if let Some(structure_index) = self.structure_index {
            write!(formatter, " at structure item {structure_index}")?;
        }
        if let Some(field_index) = self.field_index {
            write!(formatter, ", field {field_index}")?;
        }
        Ok(())
    }
}

impl Error for MessageValidationError {}

/// Metadata retained by a successfully validated request.
#[derive(Clone, Copy)]
pub(super) struct ValidatedRequestHeader {
    pub(super) protocol_version: ProtocolVersion,
    pub(super) batch_count: i32,
}

/// Metadata retained by a successfully validated response.
#[derive(Clone, Copy)]
pub(super) struct ValidatedResponseHeader {
    pub(super) protocol_version: ProtocolVersion,
    pub(super) time_stamp: i64,
    pub(super) batch_count: i32,
}

/// Validated positions and the copied required header values for a request.
pub(super) struct ValidatedRequestMessage {
    pub(super) header_index: usize,
    pub(super) batch_indices: Vec<usize>,
    pub(super) header: ValidatedRequestHeader,
}

/// Validated positions and the copied required header values for a response.
pub(super) struct ValidatedResponseMessage {
    pub(super) header_index: usize,
    pub(super) batch_indices: Vec<usize>,
    pub(super) header: ValidatedResponseHeader,
}

#[derive(Clone, Copy)]
struct FieldSpec {
    tag: u32,
    item_type: ItemType,
    repeatable: bool,
}

const fn field(tag: u32, item_type: ItemType) -> FieldSpec {
    FieldSpec {
        tag,
        item_type,
        repeatable: false,
    }
}

const fn repeated_field(tag: u32, item_type: ItemType) -> FieldSpec {
    FieldSpec {
        tag,
        item_type,
        repeatable: true,
    }
}

const VERSION_FIELDS: &[FieldSpec] = &[
    field(PROTOCOL_VERSION_MAJOR, ItemType::Integer),
    field(PROTOCOL_VERSION_MINOR, ItemType::Integer),
];
const REQUEST_HEADER_FIELDS: &[FieldSpec] = &[
    field(PROTOCOL_VERSION, ItemType::Structure),
    field(MAXIMUM_RESPONSE_SIZE, ItemType::Integer),
    field(CLIENT_CORRELATION_VALUE, ItemType::TextString),
    field(SERVER_CORRELATION_VALUE, ItemType::TextString),
    field(ASYNCHRONOUS_INDICATOR, ItemType::Enumeration),
    field(ATTESTATION_CAPABLE_INDICATOR, ItemType::Boolean),
    repeated_field(ATTESTATION_TYPE, ItemType::Enumeration),
    field(AUTHENTICATION, ItemType::Structure),
    field(BATCH_ERROR_CONTINUATION_OPTION, ItemType::Enumeration),
    field(BATCH_ORDER_OPTION, ItemType::Boolean),
    field(TIME_STAMP, ItemType::DateTime),
    field(BATCH_COUNT, ItemType::Integer),
];
const RESPONSE_HEADER_FIELDS: &[FieldSpec] = &[
    field(PROTOCOL_VERSION, ItemType::Structure),
    field(TIME_STAMP, ItemType::DateTime),
    field(NONCE, ItemType::Structure),
    field(SERVER_HASHED_PASSWORD, ItemType::ByteString),
    repeated_field(ATTESTATION_TYPE, ItemType::Enumeration),
    field(CLIENT_CORRELATION_VALUE, ItemType::TextString),
    field(SERVER_CORRELATION_VALUE, ItemType::TextString),
    field(BATCH_COUNT, ItemType::Integer),
];
const REQUEST_BATCH_FIELDS: &[FieldSpec] = &[
    field(OPERATION, ItemType::Enumeration),
    field(EPHEMERAL, ItemType::Boolean),
    field(UNIQUE_BATCH_ITEM_ID, ItemType::ByteString),
    field(REQUEST_PAYLOAD, ItemType::Structure),
    repeated_field(MESSAGE_EXTENSION, ItemType::Structure),
];
const RESPONSE_BATCH_FIELDS: &[FieldSpec] = &[
    field(OPERATION, ItemType::Enumeration),
    field(UNIQUE_BATCH_ITEM_ID, ItemType::ByteString),
    field(RESULT_STATUS, ItemType::Enumeration),
    field(RESULT_REASON, ItemType::Enumeration),
    field(RESULT_MESSAGE, ItemType::TextString),
    field(ASYNCHRONOUS_CORRELATION_VALUE, ItemType::ByteString),
    field(RESPONSE_PAYLOAD, ItemType::Structure),
    field(MESSAGE_EXTENSION, ItemType::Structure),
];
const MESSAGE_EXTENSION_FIELDS: &[FieldSpec] = &[
    field(VENDOR_IDENTIFICATION, ItemType::TextString),
    field(CRITICALITY_INDICATOR, ItemType::Boolean),
    field(VENDOR_EXTENSION, ItemType::Structure),
];
pub(super) fn validate_request_message(
    root: &StructureView<'_>,
) -> Result<ValidatedRequestMessage, MessageValidationError> {
    let mut header: Option<(usize, ValidatedRequestHeader)> = None;
    let mut batch_indices = Vec::new();
    let mut request_ids = HashSet::<Vec<u8>>::new();

    for (index, child) in root.children().iter().enumerate() {
        match child.tag().raw() {
            REQUEST_HEADER => {
                if header.is_some() {
                    return Err(error(
                        MessageValidationErrorKind::DuplicateField,
                        Some(index),
                        None,
                    ));
                }
                if !batch_indices.is_empty() {
                    return Err(error(
                        MessageValidationErrorKind::FieldOutOfOrder,
                        Some(index),
                        None,
                    ));
                }
                let header_value = with_structure(child, index, None, |view| {
                    validate_request_header(&view, index)
                })?;
                header = Some((index, header_value));
            }
            RESPONSE_HEADER => {
                return Err(error(
                    MessageValidationErrorKind::InvalidFieldValue,
                    Some(index),
                    None,
                ));
            }
            BATCH_ITEM => {
                if header.is_none() {
                    return Err(error(
                        MessageValidationErrorKind::FieldOutOfOrder,
                        Some(index),
                        None,
                    ));
                }
                let batch_count = header.map_or(0, |(_, value)| value.batch_count);
                let item_id = with_structure(child, index, None, |view| {
                    validate_request_batch_item(&view, index, batch_count)
                })?;
                if let Some(item_id) = item_id
                    && !request_ids.insert(item_id)
                {
                    return Err(error(
                        MessageValidationErrorKind::InvalidFieldValue,
                        Some(index),
                        None,
                    ));
                }
                batch_indices.push(index);
            }
            _ => {}
        }
    }

    let Some((header_index, header)) = header else {
        return Err(error(
            MessageValidationErrorKind::MissingRequiredField,
            None,
            None,
        ));
    };
    if batch_indices.is_empty() {
        return Err(error(
            MessageValidationErrorKind::MissingRequiredField,
            Some(header_index),
            None,
        ));
    }
    if header.batch_count < 1 {
        return Err(error(
            MessageValidationErrorKind::InvalidBatchCount,
            Some(header_index),
            None,
        ));
    }
    if usize::try_from(header.batch_count).ok() != Some(batch_indices.len()) {
        return Err(error(
            MessageValidationErrorKind::BatchCountMismatch,
            Some(header_index),
            None,
        ));
    }

    Ok(ValidatedRequestMessage {
        header_index,
        batch_indices,
        header,
    })
}

pub(super) fn validate_response_message(
    root: &StructureView<'_>,
) -> Result<ValidatedResponseMessage, MessageValidationError> {
    let mut header: Option<(usize, ValidatedResponseHeader)> = None;
    let mut batch_indices = Vec::new();

    for (index, child) in root.children().iter().enumerate() {
        match child.tag().raw() {
            RESPONSE_HEADER => {
                if header.is_some() {
                    return Err(error(
                        MessageValidationErrorKind::DuplicateField,
                        Some(index),
                        None,
                    ));
                }
                if !batch_indices.is_empty() {
                    return Err(error(
                        MessageValidationErrorKind::FieldOutOfOrder,
                        Some(index),
                        None,
                    ));
                }
                let header_value = with_structure(child, index, None, |view| {
                    validate_response_header(&view, index)
                })?;
                header = Some((index, header_value));
            }
            REQUEST_HEADER => {
                return Err(error(
                    MessageValidationErrorKind::InvalidFieldValue,
                    Some(index),
                    None,
                ));
            }
            BATCH_ITEM => {
                if header.is_none() {
                    return Err(error(
                        MessageValidationErrorKind::FieldOutOfOrder,
                        Some(index),
                        None,
                    ));
                }
                with_structure(child, index, None, |view| {
                    validate_response_batch_item(&view, index)
                })?;
                batch_indices.push(index);
            }
            _ => {}
        }
    }

    let Some((header_index, header)) = header else {
        return Err(error(
            MessageValidationErrorKind::MissingRequiredField,
            None,
            None,
        ));
    };
    if batch_indices.is_empty() {
        return Err(error(
            MessageValidationErrorKind::MissingRequiredField,
            Some(header_index),
            None,
        ));
    }
    if header.batch_count < 1 {
        return Err(error(
            MessageValidationErrorKind::InvalidBatchCount,
            Some(header_index),
            None,
        ));
    }
    if usize::try_from(header.batch_count).ok() != Some(batch_indices.len()) {
        return Err(error(
            MessageValidationErrorKind::BatchCountMismatch,
            Some(header_index),
            None,
        ));
    }

    Ok(ValidatedResponseMessage {
        header_index,
        batch_indices,
        header,
    })
}

fn validate_request_header(
    view: &StructureView<'_>,
    top_index: usize,
) -> Result<ValidatedRequestHeader, MessageValidationError> {
    validate_fields(
        view.children(),
        REQUEST_HEADER_FIELDS,
        &[PROTOCOL_VERSION, BATCH_COUNT],
        Some(top_index),
    )?;
    let version_item = required_child(view.children(), PROTOCOL_VERSION, top_index)?;
    let protocol_version = with_structure(version_item, top_index, None, |view| {
        validate_protocol_version(&view)
    })?;
    let count_item = required_child(view.children(), BATCH_COUNT, top_index)?;
    let batch_count = integer_value(count_item).ok_or_else(|| {
        error(
            MessageValidationErrorKind::WrongItemType,
            Some(top_index),
            None,
        )
    })?;
    if batch_count < 1 {
        return Err(error(
            MessageValidationErrorKind::InvalidBatchCount,
            Some(top_index),
            None,
        ));
    }
    if batch_count == 1
        && view.children().iter().any(|item| {
            matches!(
                item.tag().raw(),
                BATCH_ORDER_OPTION | BATCH_ERROR_CONTINUATION_OPTION
            )
        })
    {
        return Err(error(
            MessageValidationErrorKind::InvalidSingleItemOption,
            Some(top_index),
            None,
        ));
    }
    Ok(ValidatedRequestHeader {
        protocol_version,
        batch_count,
    })
}

fn validate_response_header(
    view: &StructureView<'_>,
    top_index: usize,
) -> Result<ValidatedResponseHeader, MessageValidationError> {
    validate_fields(
        view.children(),
        RESPONSE_HEADER_FIELDS,
        &[PROTOCOL_VERSION, TIME_STAMP, BATCH_COUNT],
        Some(top_index),
    )?;
    let version_item = required_child(view.children(), PROTOCOL_VERSION, top_index)?;
    let protocol_version = with_structure(version_item, top_index, None, |view| {
        validate_protocol_version(&view)
    })?;
    let time_stamp = required_child(view.children(), TIME_STAMP, top_index)?
        .with_value(|value| match value {
            ValueView::DateTime(raw) => Some(*raw),
            _ => None,
        })
        .ok_or_else(|| {
            error(
                MessageValidationErrorKind::WrongItemType,
                Some(top_index),
                None,
            )
        })?;
    let batch_count = integer_value(required_child(view.children(), BATCH_COUNT, top_index)?)
        .ok_or_else(|| {
            error(
                MessageValidationErrorKind::WrongItemType,
                Some(top_index),
                None,
            )
        })?;
    if batch_count < 1 {
        return Err(error(
            MessageValidationErrorKind::InvalidBatchCount,
            Some(top_index),
            None,
        ));
    }
    Ok(ValidatedResponseHeader {
        protocol_version,
        time_stamp,
        batch_count,
    })
}

fn validate_protocol_version(
    view: &StructureView<'_>,
) -> Result<ProtocolVersion, MessageValidationError> {
    validate_fields(
        view.children(),
        VERSION_FIELDS,
        &[PROTOCOL_VERSION_MAJOR, PROTOCOL_VERSION_MINOR],
        None,
    )?;
    let major = integer_value(required_child(view.children(), PROTOCOL_VERSION_MAJOR, 0)?)
        .ok_or_else(|| error(MessageValidationErrorKind::WrongItemType, None, None))?;
    let minor = integer_value(required_child(view.children(), PROTOCOL_VERSION_MINOR, 0)?)
        .ok_or_else(|| error(MessageValidationErrorKind::WrongItemType, None, None))?;
    Ok(ProtocolVersion::from_raw(major, minor))
}

fn validate_request_batch_item(
    view: &StructureView<'_>,
    top_index: usize,
    batch_count: i32,
) -> Result<Option<Vec<u8>>, MessageValidationError> {
    validate_fields(
        view.children(),
        REQUEST_BATCH_FIELDS,
        &[OPERATION, REQUEST_PAYLOAD],
        Some(top_index),
    )?;
    let id = view
        .children()
        .iter()
        .find(|child| child.tag().raw() == UNIQUE_BATCH_ITEM_ID)
        .and_then(byte_string_value);
    if batch_count > 1 && id.is_none() {
        return Err(error(
            MessageValidationErrorKind::MissingRequiredField,
            Some(top_index),
            None,
        ));
    }
    for (index, child) in view.children().iter().enumerate() {
        if child.tag().raw() == MESSAGE_EXTENSION {
            with_structure(child, top_index, Some(index), |view| {
                validate_message_extension(&view)
            })?;
        }
    }
    Ok(id)
}

fn validate_response_batch_item(
    view: &StructureView<'_>,
    top_index: usize,
) -> Result<(), MessageValidationError> {
    validate_fields(
        view.children(),
        RESPONSE_BATCH_FIELDS,
        &[RESULT_STATUS],
        Some(top_index),
    )?;
    let status_value =
        enumeration_value(required_child(view.children(), RESULT_STATUS, top_index)?).ok_or_else(
            || {
                error(
                    MessageValidationErrorKind::WrongItemType,
                    Some(top_index),
                    None,
                )
            },
        )?;
    let status = ResultStatus::from_raw(status_value);
    let reason = view
        .children()
        .iter()
        .find(|child| child.tag().raw() == RESULT_REASON)
        .and_then(enumeration_value)
        .map(ResultReason::from_raw);
    KmipOperationResult::new(status, reason, None).map_err(|_| {
        error(
            MessageValidationErrorKind::InvalidResult,
            Some(top_index),
            None,
        )
    })?;

    let is_success = status.known_name() == Some("Success");
    let is_failure = status.known_name() == Some("Operation Failed");
    let is_pending = status.known_name() == Some("Operation Pending");
    let result_message = view
        .children()
        .iter()
        .any(|child| child.tag().raw() == RESULT_MESSAGE);
    let response_payload = view
        .children()
        .iter()
        .any(|child| child.tag().raw() == RESPONSE_PAYLOAD);
    let async_correlation = view
        .children()
        .iter()
        .any(|child| child.tag().raw() == ASYNCHRONOUS_CORRELATION_VALUE);

    if (is_success || is_pending) && result_message {
        return Err(error(
            MessageValidationErrorKind::InvalidResult,
            Some(top_index),
            None,
        ));
    }
    if is_failure == response_payload || (!is_failure && !response_payload) {
        return Err(error(
            MessageValidationErrorKind::InvalidResult,
            Some(top_index),
            None,
        ));
    }
    if is_pending && !async_correlation {
        return Err(error(
            MessageValidationErrorKind::MissingRequiredField,
            Some(top_index),
            None,
        ));
    }
    for (index, child) in view.children().iter().enumerate() {
        if child.tag().raw() == MESSAGE_EXTENSION {
            with_structure(child, top_index, Some(index), |view| {
                validate_message_extension(&view)
            })?;
        }
    }
    Ok(())
}

fn validate_message_extension(view: &StructureView<'_>) -> Result<(), MessageValidationError> {
    validate_fields(
        view.children(),
        MESSAGE_EXTENSION_FIELDS,
        &[
            VENDOR_IDENTIFICATION,
            CRITICALITY_INDICATOR,
            VENDOR_EXTENSION,
        ],
        None,
    )?;
    // §9.13 requires a value that identifies the vendor; empty text cannot do so.
    let vendor_is_valid = required_child(view.children(), VENDOR_IDENTIFICATION, 0)?
        .with_value(|value| match value {
            ValueView::TextString(value) => Some(
                !value.is_empty()
                    && value
                        .as_bytes()
                        .iter()
                        .all(|byte| byte.is_ascii_alphanumeric() || matches!(*byte, b'_' | b'.')),
            ),
            _ => None,
        })
        .ok_or_else(|| error(MessageValidationErrorKind::WrongItemType, None, None))?;
    if !vendor_is_valid {
        return Err(error(
            MessageValidationErrorKind::InvalidFieldValue,
            None,
            None,
        ));
    }
    Ok(())
}

fn validate_fields(
    children: &[Item],
    schema: &[FieldSpec],
    required_tags: &[u32],
    structure_index: Option<usize>,
) -> Result<(), MessageValidationError> {
    let mut previous_rank: Option<usize> = None;
    let mut seen = vec![false; schema.len()];
    for (field_index, child) in children.iter().enumerate() {
        let Some((rank, spec)) = schema
            .iter()
            .enumerate()
            .find(|(_, spec)| spec.tag == child.tag().raw())
        else {
            continue;
        };
        if child.item_type() != spec.item_type {
            return Err(error(
                MessageValidationErrorKind::WrongItemType,
                structure_index,
                Some(field_index),
            ));
        }
        if previous_rank.is_some_and(|previous| rank < previous) {
            return Err(error(
                MessageValidationErrorKind::FieldOutOfOrder,
                structure_index,
                Some(field_index),
            ));
        }
        if seen[rank] && !spec.repeatable {
            return Err(error(
                MessageValidationErrorKind::DuplicateField,
                structure_index,
                Some(field_index),
            ));
        }
        seen[rank] = true;
        previous_rank = Some(rank);
    }
    for tag in required_tags {
        if !children.iter().any(|child| child.tag().raw() == *tag) {
            return Err(error(
                MessageValidationErrorKind::MissingRequiredField,
                structure_index,
                None,
            ));
        }
    }
    Ok(())
}

fn required_child(
    children: &[Item],
    tag: u32,
    structure_index: usize,
) -> Result<&Item, MessageValidationError> {
    children
        .iter()
        .find(|child| child.tag().raw() == tag)
        .ok_or_else(|| {
            error(
                MessageValidationErrorKind::MissingRequiredField,
                Some(structure_index),
                None,
            )
        })
}

fn with_structure<R>(
    item: &Item,
    structure_index: usize,
    field_index: Option<usize>,
    callback: impl for<'a> FnOnce(StructureView<'a>) -> Result<R, MessageValidationError>,
) -> Result<R, MessageValidationError> {
    item.with_value(|value| match value {
        ValueView::Structure(view) => callback(view),
        _ => Err(error(
            MessageValidationErrorKind::WrongItemType,
            Some(structure_index),
            field_index,
        )),
    })
}

fn integer_value(item: &Item) -> Option<i32> {
    item.with_value(|value| match value {
        ValueView::Integer(raw) => Some(*raw),
        _ => None,
    })
}

fn enumeration_value(item: &Item) -> Option<u32> {
    item.with_value(|value| match value {
        ValueView::Enumeration(raw) => Some(*raw),
        _ => None,
    })
}

fn byte_string_value(item: &Item) -> Option<Vec<u8>> {
    item.with_value(|value| match value {
        ValueView::ByteString(raw) => Some(raw.to_vec()),
        _ => None,
    })
}

fn error(
    kind: MessageValidationErrorKind,
    structure_index: Option<usize>,
    field_index: Option<usize>,
) -> MessageValidationError {
    MessageValidationError::new(kind, structure_index, field_index)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kmipkit_ttlv::{RawTag, Structure, Tag, Value};

    fn tag(raw: u32) -> Tag {
        RawTag::new(raw)
            .expect("fixture tag fits")
            .try_checked()
            .expect("fixture tag is allocated")
    }

    fn item(raw: u32, value: Value) -> Item {
        Item::new(tag(raw), value).expect("fixture item is valid")
    }

    fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
        let mut result = Structure::new();
        for child in items {
            result.try_push(child).expect("fixture remains bounded");
        }
        result
    }

    fn version() -> Item {
        item(
            PROTOCOL_VERSION,
            Value::structure(structure([
                item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
                item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
            ])),
        )
    }

    fn request_header(count: i32) -> Structure {
        structure([version(), item(BATCH_COUNT, Value::integer(count))])
    }

    fn response_header(count: i32) -> Structure {
        structure([
            version(),
            item(TIME_STAMP, Value::date_time(1)),
            item(BATCH_COUNT, Value::integer(count)),
        ])
    }

    fn request_batch() -> Structure {
        structure([
            item(OPERATION, Value::enumeration(1)),
            item(UNIQUE_BATCH_ITEM_ID, Value::byte_string(vec![1])),
            item(REQUEST_PAYLOAD, Value::structure(Structure::new())),
        ])
    }

    fn response_batch(status: u32) -> Structure {
        let mut fields = vec![item(RESULT_STATUS, Value::enumeration(status))];
        match status {
            1 => fields.push(item(RESULT_REASON, Value::enumeration(1))),
            2 => {
                fields.push(item(
                    ASYNCHRONOUS_CORRELATION_VALUE,
                    Value::byte_string(vec![0xA5]),
                ));
                fields.push(item(RESPONSE_PAYLOAD, Value::structure(Structure::new())));
            }
            _ => fields.push(item(RESPONSE_PAYLOAD, Value::structure(Structure::new()))),
        }
        structure(fields)
    }

    fn message_extension() -> Item {
        item(
            MESSAGE_EXTENSION,
            Value::structure(structure([
                item(
                    VENDOR_IDENTIFICATION,
                    Value::text_string("Vendor".to_owned()),
                ),
                item(CRITICALITY_INDICATOR, Value::boolean(false)),
                item(VENDOR_EXTENSION, Value::structure(Structure::new())),
            ])),
        )
    }

    fn assert_kind<T>(
        result: Result<T, MessageValidationError>,
        expected: MessageValidationErrorKind,
    ) {
        assert_eq!(result.err().expect("validation fails").kind(), expected);
    }

    #[test]
    fn every_validation_error_kind_has_safe_display_text() {
        let cases = [
            (
                MessageValidationErrorKind::MissingRequiredField,
                "required message field is missing",
            ),
            (
                MessageValidationErrorKind::DuplicateField,
                "message field is duplicated",
            ),
            (
                MessageValidationErrorKind::FieldOutOfOrder,
                "message field is out of order",
            ),
            (
                MessageValidationErrorKind::WrongItemType,
                "message field has the wrong TTLV type",
            ),
            (
                MessageValidationErrorKind::InvalidBatchCount,
                "message Batch Count is invalid",
            ),
            (
                MessageValidationErrorKind::BatchCountMismatch,
                "message Batch Count does not match its items",
            ),
            (
                MessageValidationErrorKind::InvalidSingleItemOption,
                "batch option is invalid for a single-item message",
            ),
            (
                MessageValidationErrorKind::InvalidResult,
                "response result fields are inconsistent",
            ),
            (
                MessageValidationErrorKind::InvalidFieldValue,
                "message field value is invalid",
            ),
        ];
        for (kind, expected) in cases {
            assert_eq!(kind.to_string(), expected);
        }

        let error = MessageValidationError::new(
            MessageValidationErrorKind::WrongItemType,
            Some(3),
            Some(7),
        );
        assert_eq!(error.kind(), MessageValidationErrorKind::WrongItemType);
        assert_eq!(error.structure_index(), Some(3));
        assert_eq!(error.field_index(), Some(7));
        assert_eq!(
            error.to_string(),
            "message field has the wrong TTLV type at structure item 3, field 7"
        );
        let unindexed = MessageValidationError::new(
            MessageValidationErrorKind::MissingRequiredField,
            None,
            None,
        );
        assert_eq!(unindexed.structure_index(), None);
        assert_eq!(unindexed.field_index(), None);
        assert_eq!(unindexed.to_string(), "required message field is missing");
    }

    #[test]
    fn field_schema_checks_type_order_singletons_required_fields_and_repetition() {
        let schema = [
            field(OPERATION, ItemType::Enumeration),
            repeated_field(ATTESTATION_TYPE, ItemType::Enumeration),
            field(RESULT_STATUS, ItemType::Enumeration),
        ];
        let valid = [
            item(OPERATION, Value::enumeration(1)),
            item(ATTESTATION_TYPE, Value::enumeration(2)),
            item(ATTESTATION_TYPE, Value::enumeration(3)),
            item(
                RESULT_REASON,
                Value::text_string("unknown schema field".to_owned()),
            ),
            item(RESULT_STATUS, Value::enumeration(0)),
        ];
        assert!(validate_fields(&valid, &schema, &[OPERATION, RESULT_STATUS], Some(4)).is_ok());

        assert_kind(
            validate_fields(
                &[item(OPERATION, Value::text_string("wrong".to_owned()))],
                &schema,
                &[OPERATION],
                Some(4),
            ),
            MessageValidationErrorKind::WrongItemType,
        );
        assert_kind(
            validate_fields(
                &[
                    item(ATTESTATION_TYPE, Value::enumeration(2)),
                    item(OPERATION, Value::enumeration(1)),
                ],
                &schema,
                &[],
                Some(4),
            ),
            MessageValidationErrorKind::FieldOutOfOrder,
        );
        assert_kind(
            validate_fields(
                &[
                    item(OPERATION, Value::enumeration(1)),
                    item(OPERATION, Value::enumeration(2)),
                ],
                &schema,
                &[],
                Some(4),
            ),
            MessageValidationErrorKind::DuplicateField,
        );
        assert_kind(
            validate_fields(&[], &schema, &[RESULT_STATUS], Some(4)),
            MessageValidationErrorKind::MissingRequiredField,
        );

        let type_error = validate_fields(
            &[item(OPERATION, Value::text_string("wrong".to_owned()))],
            &schema,
            &[],
            Some(4),
        )
        .expect_err("the Operation item type is constrained");
        assert_eq!(type_error.structure_index(), Some(4));
        assert_eq!(type_error.field_index(), Some(0));
    }

    #[test]
    fn protocol_version_and_typed_value_helpers_cover_valid_and_invalid_values() {
        let valid = structure([
            item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
            item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
        ]);
        assert_eq!(
            validate_protocol_version(&valid.view())
                .expect("version valid")
                .major(),
            2
        );

        let missing_major = structure([item(PROTOCOL_VERSION_MINOR, Value::integer(1))]);
        assert_kind(
            validate_protocol_version(&missing_major.view()),
            MessageValidationErrorKind::MissingRequiredField,
        );
        let wrong_major = structure([
            item(PROTOCOL_VERSION_MAJOR, Value::text_string("2".to_owned())),
            item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
        ]);
        assert_kind(
            validate_protocol_version(&wrong_major.view()),
            MessageValidationErrorKind::WrongItemType,
        );
        let missing_minor = structure([item(PROTOCOL_VERSION_MAJOR, Value::integer(2))]);
        assert_kind(
            validate_protocol_version(&missing_minor.view()),
            MessageValidationErrorKind::MissingRequiredField,
        );

        let integer = item(OPERATION, Value::integer(7));
        assert_eq!(integer_value(&integer), Some(7));
        assert_eq!(enumeration_value(&integer), None);
        assert_eq!(byte_string_value(&integer), None);
        let enumeration = item(OPERATION, Value::enumeration(9));
        assert_eq!(enumeration_value(&enumeration), Some(9));
        assert_eq!(integer_value(&enumeration), None);
        let bytes = item(OPERATION, Value::byte_string(vec![1, 2]));
        assert_eq!(byte_string_value(&bytes), Some(vec![1, 2]));
    }

    #[test]
    fn private_field_helpers_return_categories_without_exposing_values() {
        let Err(missing) = required_child(&[], OPERATION, 6) else {
            panic!("required operation is missing");
        };
        assert_eq!(
            missing.kind(),
            MessageValidationErrorKind::MissingRequiredField
        );
        assert_eq!(missing.structure_index(), Some(6));
        assert_eq!(missing.field_index(), None);

        let not_a_structure = item(OPERATION, Value::integer(1));
        assert_kind(
            with_structure(&not_a_structure, 4, Some(2), |_| Ok(())),
            MessageValidationErrorKind::WrongItemType,
        );
    }

    #[test]
    fn request_header_validation_covers_required_fields_types_counts_and_options() {
        assert_eq!(
            validate_request_header(&request_header(2).view(), 0)
                .expect("valid request header")
                .batch_count,
            2
        );
        let missing_version = structure([item(BATCH_COUNT, Value::integer(1))]);
        assert_kind(
            validate_request_header(&missing_version.view(), 0),
            MessageValidationErrorKind::MissingRequiredField,
        );
        let missing_count = structure([version()]);
        assert_kind(
            validate_request_header(&missing_count.view(), 0),
            MessageValidationErrorKind::MissingRequiredField,
        );
        let wrong_count = structure([version(), item(BATCH_COUNT, Value::enumeration(1))]);
        assert_kind(
            validate_request_header(&wrong_count.view(), 0),
            MessageValidationErrorKind::WrongItemType,
        );
        let zero_count = request_header(0);
        assert_kind(
            validate_request_header(&zero_count.view(), 0),
            MessageValidationErrorKind::InvalidBatchCount,
        );
        let option = structure([
            version(),
            item(BATCH_ORDER_OPTION, Value::boolean(true)),
            item(BATCH_COUNT, Value::integer(1)),
        ]);
        assert_kind(
            validate_request_header(&option.view(), 0),
            MessageValidationErrorKind::InvalidSingleItemOption,
        );
        let duplicate = structure([
            version(),
            item(MAXIMUM_RESPONSE_SIZE, Value::integer(1)),
            item(MAXIMUM_RESPONSE_SIZE, Value::integer(2)),
            item(BATCH_COUNT, Value::integer(1)),
        ]);
        assert_kind(
            validate_request_header(&duplicate.view(), 0),
            MessageValidationErrorKind::DuplicateField,
        );
    }

    #[test]
    fn response_header_validation_covers_required_fields_types_and_counts() {
        assert_eq!(
            validate_response_header(&response_header(1).view(), 0)
                .expect("valid response header")
                .time_stamp,
            1
        );
        let missing_time = structure([version(), item(BATCH_COUNT, Value::integer(1))]);
        assert_kind(
            validate_response_header(&missing_time.view(), 0),
            MessageValidationErrorKind::MissingRequiredField,
        );
        let wrong_time = structure([
            version(),
            item(TIME_STAMP, Value::integer(1)),
            item(BATCH_COUNT, Value::integer(1)),
        ]);
        assert_kind(
            validate_response_header(&wrong_time.view(), 0),
            MessageValidationErrorKind::WrongItemType,
        );
        let wrong_count = structure([
            version(),
            item(TIME_STAMP, Value::date_time(1)),
            item(BATCH_COUNT, Value::boolean(true)),
        ]);
        assert_kind(
            validate_response_header(&wrong_count.view(), 0),
            MessageValidationErrorKind::WrongItemType,
        );
        let zero_count = response_header(0);
        assert_kind(
            validate_response_header(&zero_count.view(), 0),
            MessageValidationErrorKind::InvalidBatchCount,
        );
        let duplicate = structure([
            version(),
            item(TIME_STAMP, Value::date_time(1)),
            item(TIME_STAMP, Value::date_time(2)),
            item(BATCH_COUNT, Value::integer(1)),
        ]);
        assert_kind(
            validate_response_header(&duplicate.view(), 0),
            MessageValidationErrorKind::DuplicateField,
        );
    }

    #[test]
    fn request_batch_validation_covers_required_fields_ids_and_extensions() {
        let valid = request_batch();
        assert_eq!(
            validate_request_batch_item(&valid.view(), 1, 1).expect("batch valid"),
            Some(vec![1])
        );
        let identified = structure([
            item(OPERATION, Value::enumeration(1)),
            item(UNIQUE_BATCH_ITEM_ID, Value::byte_string(vec![3, 4])),
            item(REQUEST_PAYLOAD, Value::structure(Structure::new())),
            message_extension(),
        ]);
        assert_eq!(
            validate_request_batch_item(&identified.view(), 1, 2).expect("identified batch valid"),
            Some(vec![3, 4])
        );
        let missing_operation =
            structure([item(REQUEST_PAYLOAD, Value::structure(Structure::new()))]);
        assert_kind(
            validate_request_batch_item(&missing_operation.view(), 1, 1),
            MessageValidationErrorKind::MissingRequiredField,
        );
        let missing_payload = structure([item(OPERATION, Value::enumeration(1))]);
        assert_kind(
            validate_request_batch_item(&missing_payload.view(), 1, 1),
            MessageValidationErrorKind::MissingRequiredField,
        );
        let missing_id = structure([
            item(OPERATION, Value::enumeration(1)),
            item(REQUEST_PAYLOAD, Value::structure(Structure::new())),
        ]);
        assert_kind(
            validate_request_batch_item(&missing_id.view(), 1, 2),
            MessageValidationErrorKind::MissingRequiredField,
        );
        let wrong_id = structure([
            item(OPERATION, Value::enumeration(1)),
            item(UNIQUE_BATCH_ITEM_ID, Value::text_string("id".to_owned())),
            item(REQUEST_PAYLOAD, Value::structure(Structure::new())),
        ]);
        assert_kind(
            validate_request_batch_item(&wrong_id.view(), 1, 1),
            MessageValidationErrorKind::WrongItemType,
        );

        let malformed_extension = structure([
            item(OPERATION, Value::enumeration(1)),
            item(REQUEST_PAYLOAD, Value::structure(Structure::new())),
            item(MESSAGE_EXTENSION, Value::integer(2)),
        ]);
        assert_kind(
            validate_request_batch_item(&malformed_extension.view(), 1, 1),
            MessageValidationErrorKind::WrongItemType,
        );
    }

    #[test]
    fn response_batch_validation_covers_result_invariants_and_extensions() {
        assert!(validate_response_batch_item(&response_batch(0).view(), 1).is_ok());
        assert!(validate_response_batch_item(&response_batch(1).view(), 1).is_ok());
        assert!(validate_response_batch_item(&response_batch(2).view(), 1).is_ok());
        assert!(validate_response_batch_item(&response_batch(u32::MAX).view(), 1).is_ok());

        let missing_status = Structure::new();
        assert_kind(
            validate_response_batch_item(&missing_status.view(), 1),
            MessageValidationErrorKind::MissingRequiredField,
        );
        let wrong_status = structure([item(
            RESULT_STATUS,
            Value::text_string("success".to_owned()),
        )]);
        assert_kind(
            validate_response_batch_item(&wrong_status.view(), 1),
            MessageValidationErrorKind::WrongItemType,
        );
        let success_with_reason = structure([
            item(RESULT_STATUS, Value::enumeration(0)),
            item(RESULT_REASON, Value::enumeration(1)),
            item(RESPONSE_PAYLOAD, Value::structure(Structure::new())),
        ]);
        assert_kind(
            validate_response_batch_item(&success_with_reason.view(), 1),
            MessageValidationErrorKind::InvalidResult,
        );
        let failure_without_reason = structure([item(RESULT_STATUS, Value::enumeration(1))]);
        assert_kind(
            validate_response_batch_item(&failure_without_reason.view(), 1),
            MessageValidationErrorKind::InvalidResult,
        );
        let failure_with_payload = structure([
            item(RESULT_STATUS, Value::enumeration(1)),
            item(RESULT_REASON, Value::enumeration(1)),
            item(RESPONSE_PAYLOAD, Value::structure(Structure::new())),
        ]);
        assert_kind(
            validate_response_batch_item(&failure_with_payload.view(), 1),
            MessageValidationErrorKind::InvalidResult,
        );
        let success_with_message = structure([
            item(RESULT_STATUS, Value::enumeration(0)),
            item(RESULT_MESSAGE, Value::text_string("not allowed".to_owned())),
            item(RESPONSE_PAYLOAD, Value::structure(Structure::new())),
        ]);
        assert_kind(
            validate_response_batch_item(&success_with_message.view(), 1),
            MessageValidationErrorKind::InvalidResult,
        );
        let pending_without_correlation = structure([
            item(RESULT_STATUS, Value::enumeration(2)),
            item(RESPONSE_PAYLOAD, Value::structure(Structure::new())),
        ]);
        assert_kind(
            validate_response_batch_item(&pending_without_correlation.view(), 1),
            MessageValidationErrorKind::MissingRequiredField,
        );
        let extension_on_response = structure([
            item(RESULT_STATUS, Value::enumeration(0)),
            item(RESPONSE_PAYLOAD, Value::structure(Structure::new())),
            message_extension(),
        ]);
        assert!(validate_response_batch_item(&extension_on_response.view(), 1).is_ok());
        let bad_extension = structure([
            item(RESULT_STATUS, Value::enumeration(0)),
            item(RESPONSE_PAYLOAD, Value::structure(Structure::new())),
            item(MESSAGE_EXTENSION, Value::integer(1)),
        ]);
        assert_kind(
            validate_response_batch_item(&bad_extension.view(), 1),
            MessageValidationErrorKind::WrongItemType,
        );
    }

    #[test]
    fn message_extension_validation_checks_required_fields_order_and_values() {
        let valid = structure([
            item(
                VENDOR_IDENTIFICATION,
                Value::text_string("Vendor_1".to_owned()),
            ),
            item(CRITICALITY_INDICATOR, Value::boolean(false)),
            item(VENDOR_EXTENSION, Value::structure(Structure::new())),
        ]);
        assert!(validate_message_extension(&valid.view()).is_ok());

        let missing_vendor = structure([
            item(CRITICALITY_INDICATOR, Value::boolean(false)),
            item(VENDOR_EXTENSION, Value::structure(Structure::new())),
        ]);
        assert_kind(
            validate_message_extension(&missing_vendor.view()),
            MessageValidationErrorKind::MissingRequiredField,
        );
        let wrong_vendor_type = structure([
            item(VENDOR_IDENTIFICATION, Value::enumeration(1)),
            item(CRITICALITY_INDICATOR, Value::boolean(false)),
            item(VENDOR_EXTENSION, Value::structure(Structure::new())),
        ]);
        assert_kind(
            validate_message_extension(&wrong_vendor_type.view()),
            MessageValidationErrorKind::WrongItemType,
        );
        let wrong_criticality_type = structure([
            item(
                VENDOR_IDENTIFICATION,
                Value::text_string("Vendor".to_owned()),
            ),
            item(CRITICALITY_INDICATOR, Value::integer(0)),
            item(VENDOR_EXTENSION, Value::structure(Structure::new())),
        ]);
        assert_kind(
            validate_message_extension(&wrong_criticality_type.view()),
            MessageValidationErrorKind::WrongItemType,
        );
        let empty_vendor = structure([
            item(VENDOR_IDENTIFICATION, Value::text_string(String::new())),
            item(CRITICALITY_INDICATOR, Value::boolean(false)),
            item(VENDOR_EXTENSION, Value::structure(Structure::new())),
        ]);
        assert_kind(
            validate_message_extension(&empty_vendor.view()),
            MessageValidationErrorKind::InvalidFieldValue,
        );
        let invalid_characters = structure([
            item(
                VENDOR_IDENTIFICATION,
                Value::text_string("Vendor name".to_owned()),
            ),
            item(CRITICALITY_INDICATOR, Value::boolean(false)),
            item(VENDOR_EXTENSION, Value::structure(Structure::new())),
        ]);
        assert_kind(
            validate_message_extension(&invalid_characters.view()),
            MessageValidationErrorKind::InvalidFieldValue,
        );
    }

    #[test]
    fn request_envelope_validator_rejects_wrong_kinds_order_duplicates_and_absence() {
        let request = structure([
            item(REQUEST_HEADER, Value::structure(request_header(1))),
            item(BATCH_ITEM, Value::structure(request_batch())),
        ]);
        assert!(validate_request_message(&request.view()).is_ok());
        let no_header = Structure::new();
        assert_kind(
            validate_request_message(&no_header.view()),
            MessageValidationErrorKind::MissingRequiredField,
        );
        let no_items = structure([item(REQUEST_HEADER, Value::structure(request_header(1)))]);
        assert_kind(
            validate_request_message(&no_items.view()),
            MessageValidationErrorKind::MissingRequiredField,
        );
        let batch_first = structure([
            item(BATCH_ITEM, Value::structure(request_batch())),
            item(REQUEST_HEADER, Value::structure(request_header(1))),
        ]);
        assert_kind(
            validate_request_message(&batch_first.view()),
            MessageValidationErrorKind::FieldOutOfOrder,
        );
        let response_header_in_request =
            structure([item(RESPONSE_HEADER, Value::structure(response_header(1)))]);
        assert_kind(
            validate_request_message(&response_header_in_request.view()),
            MessageValidationErrorKind::InvalidFieldValue,
        );
        let wrong_header_type = structure([item(REQUEST_HEADER, Value::integer(1))]);
        assert_kind(
            validate_request_message(&wrong_header_type.view()),
            MessageValidationErrorKind::WrongItemType,
        );
        let duplicate_header = structure([
            item(REQUEST_HEADER, Value::structure(request_header(1))),
            item(REQUEST_HEADER, Value::structure(request_header(1))),
        ]);
        assert_kind(
            validate_request_message(&duplicate_header.view()),
            MessageValidationErrorKind::DuplicateField,
        );
        let header_after_item = structure([
            item(REQUEST_HEADER, Value::structure(request_header(1))),
            item(BATCH_ITEM, Value::structure(request_batch())),
            item(REQUEST_HEADER, Value::structure(request_header(1))),
        ]);
        assert_kind(
            validate_request_message(&header_after_item.view()),
            MessageValidationErrorKind::DuplicateField,
        );
    }

    #[test]
    fn response_envelope_validator_rejects_wrong_kinds_order_duplicates_and_absence() {
        let response = structure([
            item(RESPONSE_HEADER, Value::structure(response_header(1))),
            item(BATCH_ITEM, Value::structure(response_batch(0))),
        ]);
        assert!(validate_response_message(&response.view()).is_ok());
        let no_header = Structure::new();
        assert_kind(
            validate_response_message(&no_header.view()),
            MessageValidationErrorKind::MissingRequiredField,
        );
        let no_items = structure([item(RESPONSE_HEADER, Value::structure(response_header(1)))]);
        assert_kind(
            validate_response_message(&no_items.view()),
            MessageValidationErrorKind::MissingRequiredField,
        );
        let batch_first = structure([
            item(BATCH_ITEM, Value::structure(response_batch(0))),
            item(RESPONSE_HEADER, Value::structure(response_header(1))),
        ]);
        assert_kind(
            validate_response_message(&batch_first.view()),
            MessageValidationErrorKind::FieldOutOfOrder,
        );
        let request_header_in_response =
            structure([item(REQUEST_HEADER, Value::structure(request_header(1)))]);
        assert_kind(
            validate_response_message(&request_header_in_response.view()),
            MessageValidationErrorKind::InvalidFieldValue,
        );
        let wrong_header_type = structure([item(RESPONSE_HEADER, Value::integer(1))]);
        assert_kind(
            validate_response_message(&wrong_header_type.view()),
            MessageValidationErrorKind::WrongItemType,
        );
        let duplicate_header = structure([
            item(RESPONSE_HEADER, Value::structure(response_header(1))),
            item(RESPONSE_HEADER, Value::structure(response_header(1))),
        ]);
        assert_kind(
            validate_response_message(&duplicate_header.view()),
            MessageValidationErrorKind::DuplicateField,
        );
        let header_after_item = structure([
            item(RESPONSE_HEADER, Value::structure(response_header(1))),
            item(BATCH_ITEM, Value::structure(response_batch(0))),
            item(RESPONSE_HEADER, Value::structure(response_header(1))),
        ]);
        assert_kind(
            validate_response_message(&header_after_item.view()),
            MessageValidationErrorKind::DuplicateField,
        );
    }

    #[test]
    fn envelope_count_and_batch_structure_edges_have_safe_categories() {
        let wrong_batch = structure([
            item(REQUEST_HEADER, Value::structure(request_header(1))),
            item(BATCH_ITEM, Value::integer(1)),
        ]);
        assert_kind(
            validate_request_message(&wrong_batch.view()),
            MessageValidationErrorKind::WrongItemType,
        );
        let wrong_response_batch = structure([
            item(RESPONSE_HEADER, Value::structure(response_header(1))),
            item(BATCH_ITEM, Value::integer(1)),
        ]);
        assert_kind(
            validate_response_message(&wrong_response_batch.view()),
            MessageValidationErrorKind::WrongItemType,
        );
        let mismatch_request = structure([
            item(REQUEST_HEADER, Value::structure(request_header(2))),
            item(BATCH_ITEM, Value::structure(request_batch())),
        ]);
        assert_kind(
            validate_request_message(&mismatch_request.view()),
            MessageValidationErrorKind::BatchCountMismatch,
        );
        let mismatch_response = structure([
            item(RESPONSE_HEADER, Value::structure(response_header(2))),
            item(BATCH_ITEM, Value::structure(response_batch(0))),
        ]);
        assert_kind(
            validate_response_message(&mismatch_response.view()),
            MessageValidationErrorKind::BatchCountMismatch,
        );
        let empty_request = structure([item(REQUEST_HEADER, Value::structure(request_header(0)))]);
        assert_kind(
            validate_request_message(&empty_request.view()),
            MessageValidationErrorKind::InvalidBatchCount,
        );
        let empty_response =
            structure([item(RESPONSE_HEADER, Value::structure(response_header(0)))]);
        assert_kind(
            validate_response_message(&empty_response.view()),
            MessageValidationErrorKind::InvalidBatchCount,
        );
    }
}
