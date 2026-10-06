//! Typed KMIP 2.1 Poll request and borrowed response view.

use std::fmt;

use kmipkit_ttlv::Structure;

use crate::asynchronous::{
    AsynchronousOperationError, SecretBytes, correlation_tag, is_failure, is_pending, item,
    operation_result, structure,
};
use crate::{KmipOperationResult, ProtocolError, ResponseBatchItemView};

const POLL_OPERATION: u32 = 0x0000_001A;

/// A typed Poll request for one previously Pending operation.
///
/// The correlation bytes are opaque and are kept in zeroizing storage. The
/// request is one-shot; creating or encoding it never initiates another Poll.
///
/// ```
/// use kmipkit_protocol::PollRequest;
///
/// let correlation = [0x00, 0xff, 0x80];
/// let request = PollRequest::new(&correlation);
/// let payload = request
///     .to_ttlv_payload()
///     .expect("the Poll fields are allocated by KMIP 2.1 Table 276");
/// assert_eq!(request.asynchronous_correlation_value(), correlation);
/// assert_eq!(payload.view().children().len(), 1);
/// ```
#[derive(Clone)]
pub struct PollRequest {
    asynchronous_correlation_value: SecretBytes,
}

impl PollRequest {
    /// Creates a Poll request from the exact server-issued correlation bytes.
    #[must_use]
    pub fn new(asynchronous_correlation_value: &[u8]) -> Self {
        Self {
            asynchronous_correlation_value: SecretBytes::new(asynchronous_correlation_value),
        }
    }

    /// Returns a borrowed view of the exact correlation bytes supplied to `new`.
    #[must_use]
    pub fn asynchronous_correlation_value(&self) -> &[u8] {
        self.asynchronous_correlation_value.as_slice()
    }

    /// Builds the ordered Poll Request Payload from §6.1.38, Table 276.
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

impl fmt::Debug for PollRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PollRequest")
            .field("asynchronous_correlation_value", &"[REDACTED]")
            .finish()
    }
}

/// A Poll result, retaining the original operation's response payload as
/// generic TTLV and lending Pending correlation bytes only to a callback.
pub struct PollResponse<'a> {
    item: ResponseBatchItemView<'a>,
    result: KmipOperationResult,
}

impl<'a> PollResponse<'a> {
    /// Converts one validated Poll response batch item.
    ///
    /// A Pending Poll response uses §6.1.38's special no-payload shape and
    /// requires the correlation field required by §8.6, Table 399. A terminal
    /// response keeps the original operation payload generic.
    ///
    /// # Errors
    ///
    /// Returns [`AsynchronousOperationError`] when the item names a different
    /// operation, omits Pending correlation bytes, or violates Poll response
    /// payload requirements.
    pub fn try_from_response_item(
        item: ResponseBatchItemView<'a>,
    ) -> Result<Self, AsynchronousOperationError> {
        if item.operation() != Some(POLL_OPERATION) {
            return Err(AsynchronousOperationError::UnexpectedOperation);
        }
        let result = operation_result(item)?;
        if is_pending(result.status()) {
            if item.with_asynchronous_correlation_value(|_| ()).is_none() {
                return Err(AsynchronousOperationError::MissingAsynchronousCorrelationValue);
            }
            if item.with_response_payload(|_| ()).is_some() {
                return Err(AsynchronousOperationError::UnexpectedResponsePayload);
            }
        } else if !is_failure(result.status()) && item.with_response_payload(|_| ()).is_none() {
            return Err(AsynchronousOperationError::MissingResponsePayload);
        }
        Ok(Self { item, result })
    }

    /// Returns the complete original-operation result.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }

    /// Returns whether the original operation remains Pending.
    #[must_use]
    pub const fn is_pending(&self) -> bool {
        is_pending(self.result.status())
    }

    /// Lends the response's Asynchronous Correlation Value to a callback.
    pub fn with_asynchronous_correlation_value<R>(
        &self,
        callback: impl for<'b> FnOnce(&'b [u8]) -> R,
    ) -> Option<R> {
        self.item.with_asynchronous_correlation_value(callback)
    }

    /// Lends the generic original-operation Response Payload to a callback.
    pub fn with_response_payload<R>(
        &self,
        callback: impl for<'b> FnOnce(kmipkit_ttlv::StructureView<'b>) -> R,
    ) -> Option<R> {
        self.item.with_response_payload(callback)
    }
}

impl fmt::Debug for PollResponse<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PollResponse")
            .field("result", &self.result)
            .field("pending_correlation", &"[REDACTED]")
            .field(
                "has_response_payload",
                &self.item.with_response_payload(|_| ()).is_some(),
            )
            .finish()
    }
}
