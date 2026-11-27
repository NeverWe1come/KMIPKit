//! Fluent builders for KMIP 2.1 asynchronous operation control requests.

use kmipkit_protocol::{
    CancelRequest, KmipOperationResult, PollRequest, ProcessRequest, QueryAsyncRequestsRequest,
    SecretBytes,
};
use kmipkit_ttlv::codec::CodecLimits;

use crate::{
    Client, ClientBatchItem, ClientBatchItemResponse, ClientBatchOutcome, ClientError,
    ClientRequest,
};

use super::execution::{FluentBuildError, FluentBuilderExecution, unwrap_typed_outcome};

fn unwrap_kmi_result_or_server_err(
    outcome: Result<ClientBatchItemResponse, ClientError>,
    extract: impl FnOnce(ClientBatchOutcome) -> Option<KmipOperationResult>,
) -> Result<KmipOperationResult, ClientError> {
    let item = outcome?;
    unwrap_typed_outcome(item, extract, |result| result)
}

/// A fluent builder for KMIP Cancel requests (§6.1.5, Table 176).
pub struct CancelBuilder<'a> {
    execution: FluentBuilderExecution<'a>,
    correlation_value: SecretBytes,
}

impl<'a> CancelBuilder<'a> {
    pub(super) fn new(client: &'a mut Client, correlation_value: SecretBytes) -> Self {
        Self {
            execution: FluentBuilderExecution::new(client),
            correlation_value,
        }
    }

    fluent_option_field_setters!(
        "Sets exchange-local options without changing client policy.";
        execution.options;
        execution.asynchronous_indicator;
        const
    );

    /// Validates the typed Cancel request locally (always valid).
    ///
    /// # Errors
    ///
    /// This method is infallible and always returns `Ok(())`.
    pub const fn validate(&self) -> Result<(), FluentBuildError> {
        Ok(())
    }

    /// Builds the typed [`CancelRequest`] without sending it.
    #[must_use]
    pub fn build(self) -> CancelRequest {
        self.correlation_value.with_bytes(CancelRequest::new)
    }

    /// Converts Cancel to a typed [`ClientBatchItem`].
    #[must_use]
    pub fn to_batch_item(self) -> ClientBatchItem {
        ClientBatchItem::new(ClientRequest::Cancel(self.build()))
    }

    /// Executes one Cancel exchange using the client's configured [`CodecLimits`].
    ///
    /// # Errors
    ///
    /// Returns a validation, protocol, transport, `ServerResult`, or `Pending` error.
    pub fn execute(self) -> Result<KmipOperationResult, ClientError> {
        let outcome = self.execute_outcome();
        unwrap_kmi_result_or_server_err(outcome, |item| match item {
            ClientBatchOutcome::Cancel(result) => Some(result),
            _ => None,
        })
    }

    /// Executes one Cancel exchange and returns the raw [`ClientBatchItemResponse`].
    ///
    /// # Errors
    ///
    /// Returns exchange or decoding failures.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let Self {
            execution,
            correlation_value,
        } = self;
        let request = correlation_value.with_bytes(CancelRequest::new);
        execution.run_outcome(ClientRequest::Cancel(request))
    }

    /// Executes one Cancel exchange with explicit [`CodecLimits`].
    ///
    /// # Errors
    ///
    /// Returns exchange or decoding failures.
    pub fn execute_outcome_with_limits(
        self,
        limits: &CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let Self {
            execution,
            correlation_value,
        } = self;
        let request = correlation_value.with_bytes(CancelRequest::new);
        execution.run(ClientRequest::Cancel(request), limits)
    }
}

/// A fluent builder for KMIP Poll requests (§6.1.38, Table 276).
pub struct PollBuilder<'a> {
    execution: FluentBuilderExecution<'a>,
    correlation_value: SecretBytes,
}

impl<'a> PollBuilder<'a> {
    pub(super) fn new(client: &'a mut Client, correlation_value: SecretBytes) -> Self {
        Self {
            execution: FluentBuilderExecution::new(client),
            correlation_value,
        }
    }

    fluent_option_field_setters!(
        "Sets exchange-local options without changing client policy.";
        execution.options;
        execution.asynchronous_indicator;
        const
    );

    /// Validates the typed Poll request locally (always valid).
    ///
    /// # Errors
    ///
    /// This method is infallible and always returns `Ok(())`.
    pub const fn validate(&self) -> Result<(), FluentBuildError> {
        Ok(())
    }

    /// Builds the typed [`PollRequest`] without sending it.
    #[must_use]
    pub fn build(self) -> PollRequest {
        self.correlation_value.with_bytes(PollRequest::new)
    }

    /// Converts Poll to a typed [`ClientBatchItem`].
    #[must_use]
    pub fn to_batch_item(self) -> ClientBatchItem {
        ClientBatchItem::new(ClientRequest::Poll(self.build()))
    }

    /// Executes one Poll exchange using the client's configured [`CodecLimits`].
    ///
    /// # Errors
    ///
    /// Returns a validation, protocol, transport, `ServerResult`, or `Pending` error.
    pub fn execute(self) -> Result<KmipOperationResult, ClientError> {
        let outcome = self.execute_outcome();
        unwrap_kmi_result_or_server_err(outcome, |item| match item {
            ClientBatchOutcome::Poll(result) => Some(result),
            _ => None,
        })
    }

    /// Executes one Poll exchange and returns the raw [`ClientBatchItemResponse`].
    ///
    /// # Errors
    ///
    /// Returns exchange or decoding failures.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let Self {
            execution,
            correlation_value,
        } = self;
        let request = correlation_value.with_bytes(PollRequest::new);
        execution.run_outcome(ClientRequest::Poll(request))
    }

    /// Executes one Poll exchange with explicit [`CodecLimits`].
    ///
    /// # Errors
    ///
    /// Returns exchange or decoding failures.
    pub fn execute_outcome_with_limits(
        self,
        limits: &CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let Self {
            execution,
            correlation_value,
        } = self;
        let request = correlation_value.with_bytes(PollRequest::new);
        execution.run(ClientRequest::Poll(request), limits)
    }
}

/// A fluent builder for KMIP Process requests (§6.1.39, Table 278).
pub struct ProcessBuilder<'a> {
    execution: FluentBuilderExecution<'a>,
    correlation_value: SecretBytes,
}

impl<'a> ProcessBuilder<'a> {
    pub(super) fn new(client: &'a mut Client, correlation_value: SecretBytes) -> Self {
        Self {
            execution: FluentBuilderExecution::new(client),
            correlation_value,
        }
    }

    fluent_option_field_setters!(
        "Sets exchange-local options without changing client policy.";
        execution.options;
        execution.asynchronous_indicator;
        const
    );

    /// Validates the typed Process request locally (always valid).
    ///
    /// # Errors
    ///
    /// This method is infallible and always returns `Ok(())`.
    pub const fn validate(&self) -> Result<(), FluentBuildError> {
        Ok(())
    }

    /// Builds the typed [`ProcessRequest`] without sending it.
    #[must_use]
    pub fn build(self) -> ProcessRequest {
        self.correlation_value.with_bytes(ProcessRequest::new)
    }

    /// Converts Process to a typed [`ClientBatchItem`].
    #[must_use]
    pub fn to_batch_item(self) -> ClientBatchItem {
        ClientBatchItem::new(ClientRequest::Process(self.build()))
    }

    /// Executes one Process exchange using the client's configured [`CodecLimits`].
    ///
    /// # Errors
    ///
    /// Returns a validation, protocol, transport, `ServerResult`, or `Pending` error.
    pub fn execute(self) -> Result<KmipOperationResult, ClientError> {
        let outcome = self.execute_outcome();
        unwrap_kmi_result_or_server_err(outcome, |item| match item {
            ClientBatchOutcome::Process(result) => Some(result),
            _ => None,
        })
    }

    /// Executes one Process exchange and returns the raw [`ClientBatchItemResponse`].
    ///
    /// # Errors
    ///
    /// Returns exchange or decoding failures.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let Self {
            execution,
            correlation_value,
        } = self;
        let request = correlation_value.with_bytes(ProcessRequest::new);
        execution.run_outcome(ClientRequest::Process(request))
    }

    /// Executes one Process exchange with explicit [`CodecLimits`].
    ///
    /// # Errors
    ///
    /// Returns exchange or decoding failures.
    pub fn execute_outcome_with_limits(
        self,
        limits: &CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let Self {
            execution,
            correlation_value,
        } = self;
        let request = correlation_value.with_bytes(ProcessRequest::new);
        execution.run(ClientRequest::Process(request), limits)
    }
}

/// A fluent builder for KMIP Query Asynchronous Requests payloads (§6.1.41, Table 285).
pub struct QueryAsyncRequestsBuilder<'a> {
    execution: FluentBuilderExecution<'a>,
    correlation_values: Option<Vec<SecretBytes>>,
    operations: Option<Vec<u32>>,
}

impl<'a> QueryAsyncRequestsBuilder<'a> {
    pub(super) fn new(client: &'a mut Client) -> Self {
        Self {
            execution: FluentBuilderExecution::new(client),
            correlation_values: None,
            operations: None,
        }
    }

    /// Adds one Asynchronous Correlation Value to the optional filter.
    #[must_use]
    pub fn correlation_value(mut self, value: SecretBytes) -> Self {
        self.correlation_values
            .get_or_insert_with(Vec::new)
            .push(value);
        self
    }

    /// Sets the optional Asynchronous Correlation Values filter list.
    #[must_use]
    pub fn correlation_values(mut self, values: Vec<SecretBytes>) -> Self {
        self.correlation_values = Some(values);
        self
    }

    /// Adds one raw KMIP Operation code to the optional filter.
    #[must_use]
    pub fn operation(mut self, operation: u32) -> Self {
        self.operations.get_or_insert_with(Vec::new).push(operation);
        self
    }

    /// Sets the optional Operation code filter list.
    #[must_use]
    pub fn operations(mut self, operations: Vec<u32>) -> Self {
        self.operations = Some(operations);
        self
    }

    fluent_option_field_setters!(
        "Sets exchange-local options without changing client policy.";
        execution.options;
        execution.asynchronous_indicator;
        const
    );

    /// Validates the typed Query Asynchronous Requests payload locally (always valid).
    ///
    /// # Errors
    ///
    /// This method is infallible and always returns `Ok(())`.
    pub const fn validate(&self) -> Result<(), FluentBuildError> {
        Ok(())
    }

    /// Builds the typed [`QueryAsyncRequestsRequest`] without sending it.
    #[must_use]
    pub fn build(self) -> QueryAsyncRequestsRequest {
        build_query_async_request(self.correlation_values, self.operations)
    }

    /// Converts Query Asynchronous Requests to a typed [`ClientBatchItem`].
    #[must_use]
    pub fn to_batch_item(self) -> ClientBatchItem {
        ClientBatchItem::new(ClientRequest::QueryAsynchronousRequests(self.build()))
    }

    /// Executes one Query Asynchronous Requests exchange using configured [`CodecLimits`].
    ///
    /// # Errors
    ///
    /// Returns a validation, protocol, transport, `ServerResult`, or `Pending` error.
    pub fn execute(self) -> Result<KmipOperationResult, ClientError> {
        let outcome = self.execute_outcome();
        unwrap_kmi_result_or_server_err(outcome, |item| match item {
            ClientBatchOutcome::QueryAsyncRequests(result) => Some(result),
            _ => None,
        })
    }

    /// Executes one Query Asynchronous Requests exchange and returns [`ClientBatchItemResponse`].
    ///
    /// # Errors
    ///
    /// Returns exchange or decoding failures.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let Self {
            execution,
            correlation_values,
            operations,
        } = self;
        execution.run_outcome(ClientRequest::QueryAsynchronousRequests(
            build_query_async_request(correlation_values, operations),
        ))
    }

    /// Executes one Query Asynchronous Requests exchange with explicit [`CodecLimits`].
    ///
    /// # Errors
    ///
    /// Returns exchange or decoding failures.
    pub fn execute_outcome_with_limits(
        self,
        limits: &CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let Self {
            execution,
            correlation_values,
            operations,
        } = self;
        execution.run(
            ClientRequest::QueryAsynchronousRequests(build_query_async_request(
                correlation_values,
                operations,
            )),
            limits,
        )
    }
}

fn build_query_async_request(
    correlation_values: Option<Vec<SecretBytes>>,
    operations: Option<Vec<u32>>,
) -> QueryAsyncRequestsRequest {
    let mut request = QueryAsyncRequestsRequest::new();
    if let Some(values) = correlation_values {
        let raw_values: Vec<Vec<u8>> = values
            .into_iter()
            .map(|secret| secret.with_bytes(<[u8]>::to_vec))
            .collect();
        request = request.with_correlation_values(raw_values);
    }
    if let Some(ops) = operations {
        request = request.with_operations(ops);
    }
    request
}
