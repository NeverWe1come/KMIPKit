//! Typed KMIP 2.1 Query Asynchronous Requests request and generic response.

use std::fmt;

use kmipkit_ttlv::{KmipTag, Structure, StructureView, Value, ValueView};

use crate::asynchronous::{
    AsynchronousOperationError, SecretBytes, item, operation_result, structure,
};
use crate::enums::OperationCode;
use crate::{
    KmipOperationResult, ProtocolError, ResponseBatchItemView, ResultReason, ResultStatus,
};

const QUERY_ASYNCHRONOUS_REQUESTS_OPERATION: u32 = OperationCode::QueryAsynchronousRequests.as_u32();
const ASYNCHRONOUS_CORRELATION_VALUES: u32 = KmipTag::AsynchronousCorrelationValues.as_u32();
const ASYNCHRONOUS_REQUEST: u32 = KmipTag::AsynchronousRequest.as_u32();
const ASYNCHRONOUS_CORRELATION_VALUE: u32 = KmipTag::AsynchronousCorrelationValue.as_u32();
const SUBMISSION_DATE: u32 = KmipTag::SubmissionDate.as_u32();
const PROCESSING_STAGE: u32 = KmipTag::ProcessingStage.as_u32();
const RESULT_STATUS: u32 = KmipTag::ResultStatus.as_u32();
const RESULT_REASON: u32 = KmipTag::ResultReason.as_u32();
const RESULT_MESSAGE: u32 = KmipTag::ResultMessage.as_u32();
const BATCH_ITEM: u32 = KmipTag::BatchItem.as_u32();
const OPERATIONS: u32 = KmipTag::Operations.as_u32();
const OPERATION: u32 = KmipTag::Operation.as_u32();

/// A typed Asynchronous Request entry from KMIP v2.1 §6.1.41, Table 286.
#[derive(Debug)]
pub struct AsynchronousRequestEntry {
    /// Asynchronous Correlation Value bytes.
    pub asynchronous_correlation_value: Vec<u8>,
    /// Operation enumeration code.
    pub operation: u32,
    /// Submission Date timestamp.
    pub submission_date: i64,
    /// Processing Stage enumeration code.
    pub processing_stage: u32,
    /// Optional Result Status.
    pub result_status: Option<ResultStatus>,
    /// Optional Result Reason.
    pub result_reason: Option<ResultReason>,
    /// Optional Result Message.
    pub result_message: Option<String>,
    /// Optional Batch Item structure.
    pub batch_item: Option<Structure>,
}

/// A typed Query Asynchronous Requests payload with optional lossless filters.
///
/// Correlation filter copies are zeroizing and are redacted from Debug output.
/// The unresolved `KMIPKIT-DISC-039` response schema is deliberately not modeled.
#[derive(Clone, Debug, Default)]
pub struct QueryAsyncRequestsRequest {
    correlation_values: Option<Vec<SecretBytes>>,
    operations: Option<Vec<u32>>,
}

impl QueryAsyncRequestsRequest {
    /// Creates a Query request with both optional filter structures absent.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            correlation_values: None,
            operations: None,
        }
    }

    /// Sets the optional correlation filter, preserving order, duplicates, and
    /// arbitrary binary values. The values are moved into zeroizing storage. An
    /// empty iterator encodes a present-empty filter.
    #[must_use]
    pub fn with_correlation_values<I>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = Vec<u8>>,
    {
        self.correlation_values = Some(values.into_iter().map(SecretBytes::from_vec).collect());
        self
    }

    /// Sets the optional Operation filter, retaining order, repetitions, and
    /// unknown extension values exactly as supplied.
    #[must_use]
    pub fn with_operations<I>(mut self, operations: I) -> Self
    where
        I: IntoIterator<Item = u32>,
    {
        self.operations = Some(operations.into_iter().collect());
        self
    }

    /// Builds the optional filter structures from §6.1.41, Table 285.
    ///
    /// # Errors
    ///
    /// Returns a sanitized model error if an internally defined TTLV field
    /// cannot be represented.
    pub fn to_ttlv_payload(&self) -> Result<Structure, ProtocolError> {
        let mut fields = Vec::new();
        if let Some(values) = &self.correlation_values {
            let entries = values.iter().map(|value| {
                item(
                    crate::asynchronous::correlation_tag(),
                    value.clone().into_value(),
                )
            });
            fields.push(item(
                ASYNCHRONOUS_CORRELATION_VALUES,
                Value::structure(structure(entries.collect::<Result<Vec<_>, _>>()?)?),
            )?);
        }
        if let Some(operations) = &self.operations {
            let entries = operations
                .iter()
                .map(|operation| item(OPERATION, Value::enumeration(*operation)));
            fields.push(item(
                OPERATIONS,
                Value::structure(structure(entries.collect::<Result<Vec<_>, _>>()?)?),
            )?);
        }
        structure(fields)
    }
}

/// A typed Query Asynchronous Requests result with a generic, borrowed payload.
///
/// The response mapping remains generic while `KMIPKIT-DISC-039` is open; this
/// type assigns no meaning to the Table 286 payload structure.
pub struct QueryAsyncRequestsResponse<'a> {
    item: ResponseBatchItemView<'a>,
    result: KmipOperationResult,
}

impl<'a> QueryAsyncRequestsResponse<'a> {
    /// Converts one validated Query Asynchronous Requests response item without
    /// interpreting its response payload.
    ///
    /// # Errors
    ///
    /// Returns [`AsynchronousOperationError`] when the item names another
    /// operation or its represented operation result is invalid.
    pub fn try_from_response_item(
        item: ResponseBatchItemView<'a>,
    ) -> Result<Self, AsynchronousOperationError> {
        if item.operation() != Some(QUERY_ASYNCHRONOUS_REQUESTS_OPERATION) {
            return Err(AsynchronousOperationError::UnexpectedOperation);
        }
        let result = operation_result(item)?;
        Ok(Self { item, result })
    }

    /// Returns the complete Query operation result.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }

    /// Lends the uninterpreted generic Response Payload to a callback.
    pub fn with_response_payload<R>(
        &self,
        callback: impl for<'b> FnOnce(StructureView<'b>) -> R,
    ) -> Option<R> {
        self.item.with_response_payload(callback)
    }

    /// Parses typed Asynchronous Request entries from the Response Payload.
    ///
    /// # Errors
    ///
    /// Returns [`AsynchronousOperationError::MalformedResponsePayload`] when an
    /// Asynchronous Request structure is missing required fields or uses invalid types.
    pub fn asynchronous_requests(
        &self,
    ) -> Result<Vec<AsynchronousRequestEntry>, AsynchronousOperationError> {
        let Some(parsed) = self.item.with_response_payload(|payload| {
            let mut entries = Vec::new();
            for child in payload.children() {
                if child.tag().raw() != ASYNCHRONOUS_REQUEST {
                    continue;
                }
                let mut entry_result = None;
                child.with_value(|view| {
                    let ValueView::Structure(req_struct) = view else {
                        entry_result =
                            Some(Err(AsynchronousOperationError::MalformedResponsePayload));
                        return;
                    };
                    entry_result = Some(parse_asynchronous_request_entry(&req_struct));
                });
                entries.push(
                    entry_result
                        .unwrap_or(Err(AsynchronousOperationError::MalformedResponsePayload))?,
                );
            }
            Ok(entries)
        }) else {
            return Ok(Vec::new());
        };
        parsed
    }
}

fn parse_asynchronous_request_entry(
    req_struct: &StructureView<'_>,
) -> Result<AsynchronousRequestEntry, AsynchronousOperationError> {
    let mut correlation: Option<Vec<u8>> = None;
    let mut operation: Option<u32> = None;
    let mut submission_date: Option<i64> = None;
    let mut processing_stage: Option<u32> = None;
    let mut result_status: Option<ResultStatus> = None;
    let mut result_reason: Option<ResultReason> = None;
    let mut result_message: Option<String> = None;
    let mut batch_item: Option<Structure> = None;
    let mut malformed = false;

    for field in req_struct.children() {
        field.with_value(|value| match field.tag().raw() {
            ASYNCHRONOUS_CORRELATION_VALUE if correlation.is_none() => {
                if let ValueView::ByteString(bytes) = value {
                    correlation = Some(bytes.to_vec());
                } else {
                    malformed = true;
                }
            }
            OPERATION if operation.is_none() => {
                if let ValueView::Enumeration(raw) = value {
                    operation = Some(*raw);
                } else {
                    malformed = true;
                }
            }
            SUBMISSION_DATE if submission_date.is_none() => {
                if let ValueView::DateTime(dt) = value {
                    submission_date = Some(*dt);
                } else {
                    malformed = true;
                }
            }
            PROCESSING_STAGE if processing_stage.is_none() => {
                if let ValueView::Enumeration(raw) = value {
                    processing_stage = Some(*raw);
                } else {
                    malformed = true;
                }
            }
            RESULT_STATUS if result_status.is_none() => {
                if let ValueView::Enumeration(raw) = value {
                    result_status = Some(ResultStatus::from_raw(*raw));
                } else {
                    malformed = true;
                }
            }
            RESULT_REASON if result_reason.is_none() => {
                if let ValueView::Enumeration(raw) = value {
                    result_reason = Some(ResultReason::from_raw(*raw));
                } else {
                    malformed = true;
                }
            }
            RESULT_MESSAGE if result_message.is_none() => {
                if let ValueView::TextString(text) = value {
                    result_message = Some(text.to_owned());
                } else {
                    malformed = true;
                }
            }
            BATCH_ITEM if batch_item.is_none() => {
                if let ValueView::Structure(s) = value {
                    match s.try_clone() {
                        Ok(cloned) => batch_item = Some(cloned),
                        Err(_) => malformed = true,
                    }
                } else {
                    malformed = true;
                }
            }
            _ => {}
        });
    }

    if malformed {
        return Err(AsynchronousOperationError::MalformedResponsePayload);
    }

    Ok(AsynchronousRequestEntry {
        asynchronous_correlation_value: correlation
            .ok_or(AsynchronousOperationError::MalformedResponsePayload)?,
        operation: operation.ok_or(AsynchronousOperationError::MalformedResponsePayload)?,
        submission_date: submission_date
            .ok_or(AsynchronousOperationError::MalformedResponsePayload)?,
        processing_stage: processing_stage
            .ok_or(AsynchronousOperationError::MalformedResponsePayload)?,
        result_status,
        result_reason,
        result_message,
        batch_item,
    })
}

impl fmt::Debug for QueryAsyncRequestsResponse<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("QueryAsyncRequestsResponse")
            .field("result", &self.result)
            .field(
                "has_response_payload",
                &self.item.with_response_payload(|_| ()).is_some(),
            )
            .finish()
    }
}
