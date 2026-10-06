//! Typed KMIP 2.1 Cancel request and response views.

use std::fmt;

use kmipkit_ttlv::{Structure, StructureView, ValueView};

use crate::asynchronous::{
    AsynchronousOperationError, SecretBytes, correlation_tag, is_pending, is_success, item,
    operation_result, structure,
};
use crate::{KmipOperationResult, ProtocolError, ResponseBatchItemView};

const CANCEL_OPERATION: u32 = 0x0000_0019;
const CANCELLATION_RESULT: u32 = 0x0042_0012;

/// A lossless Cancellation Result Enumeration from OASIS KMIP v2.1 §11.7,
/// Tables 437–438.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CancellationResult {
    /// The asynchronous operation was canceled (value 1).
    Canceled,
    /// The operation could not be canceled (value 2).
    UnableToCancel,
    /// The operation had already completed (value 3).
    Completed,
    /// The operation failed (value 4).
    Failed,
    /// The operation is unavailable (value 5).
    Unavailable,
    /// An extension or future value not assigned by KMIP 2.1.
    Unknown(u32),
}

impl CancellationResult {
    /// Converts a raw KMIP Enumeration while preserving unassigned values.
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        match raw {
            1 => Self::Canceled,
            2 => Self::UnableToCancel,
            3 => Self::Completed,
            4 => Self::Failed,
            5 => Self::Unavailable,
            _ => Self::Unknown(raw),
        }
    }

    /// Returns the exact KMIP Enumeration value.
    #[must_use]
    pub const fn raw(self) -> u32 {
        match self {
            Self::Canceled => 1,
            Self::UnableToCancel => 2,
            Self::Completed => 3,
            Self::Failed => 4,
            Self::Unavailable => 5,
            Self::Unknown(raw) => raw,
        }
    }
}

/// A typed Cancel request for a previously Pending operation.
#[derive(Clone)]
pub struct CancelRequest {
    asynchronous_correlation_value: SecretBytes,
}

impl CancelRequest {
    /// Creates a Cancel request using the exact server-issued correlation bytes.
    #[must_use]
    pub fn new(asynchronous_correlation_value: &[u8]) -> Self {
        Self {
            asynchronous_correlation_value: SecretBytes::new(asynchronous_correlation_value),
        }
    }

    /// Builds the ordered Cancel Request Payload from §6.1.5, Table 176.
    ///
    /// # Errors
    ///
    /// Returns a sanitized model error if an internally defined TTLV field
    /// cannot be represented.
    pub fn to_ttlv_payload(&self) -> Result<Structure, ProtocolError> {
        structure([item(
            correlation_tag(),
            self.asynchronous_correlation_value.clone().into_value(),
        )?])
    }
}

impl fmt::Debug for CancelRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CancelRequest")
            .field("asynchronous_correlation_value", &"[REDACTED]")
            .finish()
    }
}

/// A typed Cancel result. Successful responses expose the echoed correlation
/// value through a callback and preserve all assigned and unknown result values.
pub struct CancelResponse<'a> {
    item: ResponseBatchItemView<'a>,
    result: KmipOperationResult,
    cancellation_result: Option<CancellationResult>,
}

impl<'a> CancelResponse<'a> {
    /// Converts one validated Cancel response item.
    ///
    /// The §6.1.5 response is synchronous. A Pending status is rejected even
    /// when the original operation allowed asynchronous processing. Successful
    /// payloads must contain the ordered Table 177 echo and Cancellation Result.
    ///
    /// # Errors
    ///
    /// Returns [`AsynchronousOperationError`] for a wrong operation, invalid
    /// result, forbidden Pending status, or malformed successful payload.
    pub fn try_from_response_item(
        item: ResponseBatchItemView<'a>,
    ) -> Result<Self, AsynchronousOperationError> {
        if item.operation() != Some(CANCEL_OPERATION) {
            return Err(AsynchronousOperationError::UnexpectedOperation);
        }
        let result = operation_result(item)?;
        if is_pending(result.status()) {
            return Err(AsynchronousOperationError::ForbiddenResultStatus);
        }
        let cancellation_result = if is_success(result.status()) {
            let parsed = item
                .with_response_payload(|payload| parse_success_payload(&payload))
                .ok_or(AsynchronousOperationError::MissingResponsePayload)??;
            Some(parsed)
        } else {
            None
        };
        Ok(Self {
            item,
            result,
            cancellation_result,
        })
    }

    /// Returns the complete Cancel operation result.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }

    /// Lends the Table 177 echoed correlation value to a callback when present.
    pub fn with_asynchronous_correlation_value<R>(
        &self,
        callback: impl for<'b> FnOnce(&'b [u8]) -> R,
    ) -> Option<R> {
        self.item
            .with_response_payload(|payload| {
                payload
                    .children()
                    .iter()
                    .find(|child| child.tag().raw() == correlation_tag())
                    .and_then(|child| {
                        child.with_value(|value| match value {
                            ValueView::ByteString(bytes) => Some(callback(bytes)),
                            _ => None,
                        })
                    })
            })
            .flatten()
    }

    /// Returns the known or preserved raw Cancellation Result for success.
    #[must_use]
    pub const fn cancellation_result(&self) -> Option<CancellationResult> {
        self.cancellation_result
    }
}

impl fmt::Debug for CancelResponse<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CancelResponse")
            .field("result", &self.result)
            .field("echoed_correlation", &"[REDACTED]")
            .field("cancellation_result", &self.cancellation_result)
            .finish()
    }
}

fn parse_success_payload(
    payload: &StructureView<'_>,
) -> Result<CancellationResult, AsynchronousOperationError> {
    let mut correlation_index = None;
    let mut result_index = None;
    let mut cancellation_result = None;

    for (index, child) in payload.children().iter().enumerate() {
        match child.tag().raw() {
            tag if tag == correlation_tag() => {
                if correlation_index.is_some()
                    || !child.with_value(|value| matches!(value, ValueView::ByteString(_)))
                {
                    return Err(AsynchronousOperationError::MalformedResponsePayload);
                }
                correlation_index = Some(index);
            }
            CANCELLATION_RESULT => {
                if result_index.is_some() {
                    return Err(AsynchronousOperationError::MalformedResponsePayload);
                }
                cancellation_result = child.with_value(|value| match value {
                    ValueView::Enumeration(raw) => Some(CancellationResult::from_raw(*raw)),
                    _ => None,
                });
                if cancellation_result.is_none() {
                    return Err(AsynchronousOperationError::MalformedResponsePayload);
                }
                result_index = Some(index);
            }
            _ => {}
        }
    }

    match (correlation_index, result_index, cancellation_result) {
        (Some(correlation), Some(result), Some(value)) if correlation < result => Ok(value),
        _ => Err(AsynchronousOperationError::MalformedResponsePayload),
    }
}
