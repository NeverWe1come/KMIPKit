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
    let operation = view
        .children()
        .iter()
        .find(|child| child.tag().raw() == OPERATION)
        .and_then(enumeration_value);
    // §6.1.38 gives Poll's still-Pending response a specific no-payload shape;
    // §8.6/Table 399 still requires its Asynchronous Correlation Value.
    let poll_pending_without_payload =
        is_pending && operation == Some(0x0000_001A) && !response_payload;

    if (is_success || is_pending) && result_message {
        return Err(error(
            MessageValidationErrorKind::InvalidResult,
            Some(top_index),
            None,
        ));
    }
    if (is_failure == response_payload || (!is_failure && !response_payload))
        && !poll_pending_without_payload
    {
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
include!("../../tests/support/message_validation_unit.rs");
