//! Typed KMIP 2.1 Process request and response view.

use std::fmt;

use kmipkit_ttlv::{Structure, StructureView};

use crate::asynchronous::{
    AsynchronousOperationError, SecretBytes, correlation_tag, is_pending, item, operation_result,
    structure,
};
use crate::{KmipOperationResult, ProtocolError, ResponseBatchItemView};

const PROCESS_OPERATION: u32 = 0x0000_003A;

/// A typed Process request for a previously Pending operation.
#[derive(Clone)]
pub struct ProcessRequest {
    asynchronous_correlation_value: SecretBytes,
}

impl ProcessRequest {
    /// Creates a Process request using the exact server-issued correlation bytes.
    #[must_use]
    pub fn new(asynchronous_correlation_value: &[u8]) -> Self {
        Self {
            asynchronous_correlation_value: SecretBytes::new(asynchronous_correlation_value),
        }
    }

    /// Builds the Process Request Payload from §6.1.39, Table 278.
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

impl fmt::Debug for ProcessRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProcessRequest")
            .field("asynchronous_correlation_value", &"[REDACTED]")
            .finish()
    }
}

/// A Process operation result. Every non-Failure response carries an empty
/// Response Payload Structure per §6.1.39, Table 279, and §8.6, Table 399;
/// response data stays borrowed from the source.
pub struct ProcessResponse<'a> {
    item: ResponseBatchItemView<'a>,
    result: KmipOperationResult,
}

impl<'a> ProcessResponse<'a> {
    /// Converts one validated Process response item.
    ///
    /// Process may itself be asynchronous when the request permits it. A
    /// Pending result retains the generic response shape and the correlation
    /// value required by §8.6, Table 399.
    ///
    /// # Errors
    ///
    /// Returns [`AsynchronousOperationError`] for a wrong operation, missing
    /// Pending correlation value, a missing required non-Failure payload, or a
    /// nonempty non-Failure payload.
    pub fn try_from_response_item(
        item: ResponseBatchItemView<'a>,
    ) -> Result<Self, AsynchronousOperationError> {
        if item.operation() != Some(PROCESS_OPERATION) {
            return Err(AsynchronousOperationError::UnexpectedOperation);
        }
        let result = operation_result(item)?;
        if is_pending(result.status()) && item.with_asynchronous_correlation_value(|_| ()).is_none()
        {
            return Err(AsynchronousOperationError::MissingAsynchronousCorrelationValue);
        }
        if result.status().known_name() != Some("Operation Failed") {
            let members = item
                .with_response_payload(|payload| payload.children().len())
                .ok_or(AsynchronousOperationError::MissingResponsePayload)?;
            if members != 0 {
                return Err(AsynchronousOperationError::MalformedResponsePayload);
            }
        }
        Ok(Self { item, result })
    }

    /// Returns the complete Process operation result.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }

    /// Returns whether Process is Pending and requires another caller action.
    #[must_use]
    pub const fn is_pending(&self) -> bool {
        is_pending(self.result.status())
    }

    /// Returns the number of members in a present Process Response Payload.
    #[must_use]
    pub fn payload_member_count(&self) -> Option<usize> {
        self.item
            .with_response_payload(|payload| payload.children().len())
    }

    /// Lends the response's Asynchronous Correlation Value to a callback.
    pub fn with_asynchronous_correlation_value<R>(
        &self,
        callback: impl for<'b> FnOnce(&'b [u8]) -> R,
    ) -> Option<R> {
        self.item.with_asynchronous_correlation_value(callback)
    }

    /// Lends the generic Response Payload to a callback.
    pub fn with_response_payload<R>(
        &self,
        callback: impl for<'b> FnOnce(StructureView<'b>) -> R,
    ) -> Option<R> {
        self.item.with_response_payload(callback)
    }
}

impl fmt::Debug for ProcessResponse<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProcessResponse")
            .field("result", &self.result)
            .field("correlation", &"[REDACTED]")
            .field("payload_member_count", &self.payload_member_count())
            .finish()
    }
}
