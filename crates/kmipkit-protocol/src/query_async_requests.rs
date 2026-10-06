//! Typed KMIP 2.1 Query Asynchronous Requests request and generic response.

use std::fmt;

use kmipkit_ttlv::{Structure, StructureView, Value};

use crate::asynchronous::{
    AsynchronousOperationError, SecretBytes, item, operation_result, structure,
};
use crate::{KmipOperationResult, ProtocolError, ResponseBatchItemView};

const QUERY_ASYNCHRONOUS_REQUESTS_OPERATION: u32 = 0x0000_0039;
const ASYNCHRONOUS_CORRELATION_VALUES: u32 = 0x0042_0176;
const OPERATIONS: u32 = 0x0042_014F;
const OPERATION: u32 = 0x0042_005C;

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
