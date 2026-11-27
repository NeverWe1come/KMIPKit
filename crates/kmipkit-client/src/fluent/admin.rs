//! Task-oriented, fluent builders over the existing typed KMIP requests.


use kmipkit_protocol::{
    CheckRequest, CheckResponse, DiscoverVersionsRequest, DiscoverVersionsResponse,
    GetConstraintsRequest, GetConstraintsResponse, GetUsageAllocationRequest,
    GetUsageAllocationResponse, ObtainLeaseRequest, ObtainLeaseResponse, PingRequest,
    PingResponse, ProtocolVersion, QueryFunction, QueryRequest, QueryResponse,
    RngRetrieveRequest, RngRetrieveResponse, RngSeedRequest, RngSeedResponse, SecretBytes,
    UniqueIdentifier,
};
use kmipkit_transport::RequestOptions;

use crate::{Client, ClientBatchItem, ClientBatchItemResponse, ClientBatchOutcome, ClientError, ClientRequest};

use super::execution::{
    FluentBuildError, FluentBuilderExecution, effective_protocol_version, execute_fluent_outcome, execute_fluent_request, fluent_client_validation_error, normalize_uid_for_version, unwrap_typed_outcome, validate_fluent_request,
};

/// A fluent builder for the empty Ping payload in OASIS KMIP v2.1 §6.1.36,
/// Table 271. Execution preserves the existing typed result and Pending outcome.
pub struct PingBuilder<'a> {
    execution: FluentBuilderExecution<'a>,
}

impl<'a> PingBuilder<'a> {
    pub(super) fn new(client: &'a mut Client) -> Self {
        Self {
            execution: FluentBuilderExecution::new(client),
        }
    }
}

impl PingBuilder<'_> {
    /// Sets exchange-local options without changing client policy.
    #[must_use]
    pub fn with_options(mut self, options: RequestOptions) -> Self {
        self.execution = self.execution.with_options(options);
        self
    }

    /// Sets the KMIP protocol version override for this request/exchange.
    #[must_use]
    pub fn with_protocol_version(mut self, version: ProtocolVersion) -> Self {
        self.execution = self.execution.with_protocol_version(version);
        self
    }

    /// Sets the KMIP protocol version override (alias for [`Self::with_protocol_version`]).
    #[must_use]
    pub fn with_kmip_version(self, version: ProtocolVersion) -> Self {
        self.with_protocol_version(version)
    }

    /// Sets the Request Header Asynchronous Indicator for this exchange.
    #[must_use]
    pub const fn with_asynchronous_indicator(mut self, value: u32) -> Self {
        self.execution = self.execution.with_asynchronous_indicator(value);
        self
    }

    /// Validates the typed empty payload without sending it.
    ///
    /// Protocol-version eligibility is checked by the client before sending.
    ///
    /// # Errors
    /// Returns [`FluentBuildError::InvalidRequest`] if typed validation fails.
    pub fn validate(&self) -> Result<(), FluentBuildError> {
        validate_fluent_request(&PingRequest::new())
    }

    /// Builds the empty typed Ping request without sending it.
    #[must_use]
    pub const fn build(self) -> PingRequest {
        PingRequest::new()
    }

    /// Builds the empty typed Ping request without sending it.
    ///
    /// Compatibility alias for [`Self::build`].
    ///
    /// # Errors
    ///
    /// This method is infallible and always returns `Ok`.
    pub fn build_request(self) -> Result<PingRequest, FluentBuildError> {
        Ok(self.build())
    }

    /// Converts the empty request to a typed batch item.
    ///
    /// Exchange options and the asynchronous indicator belong to the batch
    /// header and are not retained in an individual batch item.
    #[must_use]
    pub fn to_batch_item(self) -> ClientBatchItem {
        ClientBatchItem::new(ClientRequest::Ping(self.build()))
    }

    /// Executes at most one exchange using the client's configured [`CodecLimits`]
    /// and returns the unwrapped [`PingResponse`].
    ///
    /// # Errors
    /// Returns the local validation, protocol, transport, `ServerResult`, or `Pending` error.
    pub fn execute(self) -> Result<PingResponse, ClientError> {
        self.execution.run_typed(
            ClientRequest::Ping(PingRequest::new()),
            |outcome| match outcome {
                ClientBatchOutcome::Ping(resp) => Some(resp),
                _ => None,
            },
            PingResponse::result,
        )
    }

    /// Executes at most one exchange using the client's configured [`CodecLimits`]
    /// and returns the raw [`ClientBatchItemResponse`] envelope.
    ///
    /// # Errors
    /// Returns the existing sanitized validation, protocol, or transport error.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        self.execution
            .run_outcome(ClientRequest::Ping(PingRequest::new()))
    }

    /// Executes at most one exchange with explicit [`CodecLimits`] overrides.
    ///
    /// # Errors
    /// Returns the existing sanitized validation, protocol, or transport error.
    pub fn execute_outcome_with_limits(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        self.execution
            .run(ClientRequest::Ping(PingRequest::new()), limits)
    }
}

/// A fluent builder for the empty Get Constraints request in OASIS KMIP v2.1
/// §6.1.22, Table 229. Its response requires the typed Constraints payload.
pub struct GetConstraintsBuilder<'a> {
    execution: FluentBuilderExecution<'a>,
}

impl<'a> GetConstraintsBuilder<'a> {
    pub(super) fn new(client: &'a mut Client) -> Self {
        Self {
            execution: FluentBuilderExecution::new(client),
        }
    }

    /// Sets exchange-local options without changing client policy.
    #[must_use]
    pub fn with_options(mut self, options: RequestOptions) -> Self {
        self.execution = self.execution.with_options(options);
        self
    }

    /// Sets the KMIP protocol version override for this request/exchange.
    #[must_use]
    pub fn with_protocol_version(mut self, version: ProtocolVersion) -> Self {
        self.execution = self.execution.with_protocol_version(version);
        self
    }

    /// Sets the KMIP protocol version override (alias for [`Self::with_protocol_version`]).
    #[must_use]
    pub fn with_kmip_version(self, version: ProtocolVersion) -> Self {
        self.with_protocol_version(version)
    }

    /// Sets the Request Header Asynchronous Indicator for this exchange.
    #[must_use]
    pub const fn with_asynchronous_indicator(mut self, value: u32) -> Self {
        self.execution = self.execution.with_asynchronous_indicator(value);
        self
    }

    /// Validates the empty typed request without sending it.
    ///
    /// # Errors
    /// Returns [`FluentBuildError::InvalidRequest`] if typed validation fails.
    pub fn validate(&self) -> Result<(), FluentBuildError> {
        validate_fluent_request(&GetConstraintsRequest::new())
    }

    /// Builds the empty typed Get Constraints request without sending it.
    #[must_use]
    pub const fn build(self) -> GetConstraintsRequest {
        GetConstraintsRequest::new()
    }

    /// Builds the empty typed Get Constraints request without sending it.
    ///
    /// Compatibility alias for [`Self::build`].
    ///
    /// # Errors
    ///
    /// This method is infallible and always returns `Ok`.
    pub fn build_request(self) -> Result<GetConstraintsRequest, FluentBuildError> {
        Ok(self.build())
    }

    /// Converts the empty request to a typed batch item.
    ///
    /// Exchange options and the asynchronous indicator belong to the batch
    /// header and are not retained in an individual batch item.
    #[must_use]
    pub fn to_batch_item(self) -> ClientBatchItem {
        ClientBatchItem::new(ClientRequest::GetConstraints(self.build()))
    }

    /// Executes at most one exchange using configured [`CodecLimits`]
    /// and returns the unwrapped [`GetConstraintsResponse`].
    ///
    /// # Errors
    /// Returns the local validation, protocol, transport, `ServerResult`, or `Pending` error.
    pub fn execute(self) -> Result<GetConstraintsResponse, ClientError> {
        self.execution.run_typed(
            ClientRequest::get_constraints(),
            |outcome| match outcome {
                ClientBatchOutcome::GetConstraints(resp) => Some(resp),
                _ => None,
            },
            GetConstraintsResponse::result,
        )
    }

    /// Executes at most one exchange using configured [`CodecLimits`]
    /// and returns the raw [`ClientBatchItemResponse`] envelope.
    ///
    /// # Errors
    /// Returns the existing sanitized protocol or transport error.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        self.execution.run_outcome(ClientRequest::get_constraints())
    }

    /// Executes at most one exchange with explicit [`CodecLimits`] overrides.
    ///
    /// # Errors
    /// Returns the existing sanitized protocol or transport error with its
    /// request-delivery evidence.
    pub fn execute_outcome_with_limits(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        self.execution.run(ClientRequest::get_constraints(), limits)
    }
}

/// A fluent Get Usage Allocation builder for OASIS KMIP v2.1 §6.1.23,
/// Tables 232–234. It delegates request validation and protocol-version
/// eligibility to the existing typed client path.
pub struct GetUsageAllocationBuilder<'a> {
    execution: FluentBuilderExecution<'a>,
    unique_identifier: Option<UniqueIdentifier>,
    usage_limits_count: Option<i64>,
}

impl<'a> GetUsageAllocationBuilder<'a> {
    pub(super) fn new(client: &'a mut Client) -> Self {
        Self {
            execution: FluentBuilderExecution::new(client),
            unique_identifier: None,
            usage_limits_count: None,
        }
    }

    /// Selects the object whose protection usage should be allocated.
    ///
    /// Omit this when the KMIP ID Placeholder resolves the object from an
    /// earlier batch operation.
    #[must_use]
    pub fn unique_identifier(mut self, value: impl Into<UniqueIdentifier>) -> Self {
        self.unique_identifier = Some(value.into());
        self
    }

    /// Sets the caller-requested Usage Limits Count.
    #[must_use]
    pub const fn usage_limits_count(mut self, value: i64) -> Self {
        self.usage_limits_count = Some(value);
        self
    }

    /// Sets exchange-local options without changing client policy.
    #[must_use]
    pub fn with_options(mut self, options: RequestOptions) -> Self {
        self.execution = self.execution.with_options(options);
        self
    }

    /// Sets the KMIP protocol version override for this request/exchange.
    #[must_use]
    pub fn with_protocol_version(mut self, version: ProtocolVersion) -> Self {
        self.execution = self.execution.with_protocol_version(version);
        self
    }

    /// Sets the KMIP protocol version override (alias for [`Self::with_protocol_version`]).
    #[must_use]
    pub fn with_kmip_version(self, version: ProtocolVersion) -> Self {
        self.with_protocol_version(version)
    }

    /// Sets the Request Header Asynchronous Indicator for this exchange.
    #[must_use]
    pub const fn with_asynchronous_indicator(mut self, value: u32) -> Self {
        self.execution = self.execution.with_asynchronous_indicator(value);
        self
    }

    fn typed_request(&self) -> Result<GetUsageAllocationRequest, FluentBuildError> {
        let count = self
            .usage_limits_count
            .ok_or(FluentBuildError::MissingUsageLimitsCount)?;
        let mut request = GetUsageAllocationRequest::new(count);
        if let Some(identifier) = &self.unique_identifier {
            request = request.with_unique_identifier(identifier.clone());
        }
        validate_fluent_request(&request)?;
        Ok(request)
    }

    /// Validates required builder fields and the existing typed request rules.
    ///
    /// # Errors
    /// Returns [`FluentBuildError::MissingUsageLimitsCount`] when Usage Limits
    /// Count is absent, or [`FluentBuildError::InvalidRequest`] when the typed
    /// request rejects the supplied values.
    pub fn validate(&self) -> Result<(), FluentBuildError> {
        self.typed_request().map(drop)
    }

    /// Builds the existing typed Get Usage Allocation request.
    ///
    /// # Errors
    /// Returns a build error when a required field is absent or typed request
    /// validation fails.
    pub fn build(self) -> Result<GetUsageAllocationRequest, FluentBuildError> {
        self.typed_request()
    }

    /// Compatibility alias for [`Self::build`].
    ///
    /// # Errors
    ///
    /// Returns a build error when a required field is absent or typed request
    /// validation fails.
    pub fn build_request(self) -> Result<GetUsageAllocationRequest, FluentBuildError> {
        self.build()
    }

    /// Converts the request to a typed batch item without sending it.
    ///
    /// # Errors
    /// Returns a build error when a required field is absent or typed request
    /// validation fails.
    pub fn to_batch_item(self) -> Result<ClientBatchItem, FluentBuildError> {
        Ok(ClientBatchItem::new(ClientRequest::get_usage_allocation(
            self.build()?,
        )))
    }

    /// Executes at most one exchange through the existing typed client path.
    ///
    /// Pending remains an explicit outcome; this builder does not poll or
    /// retry. Protocol-version eligibility is checked by the client before
    /// sending.
    ///
    /// # Errors
    /// Returns the unwrapped [`GetUsageAllocationResponse`].
    ///
    /// # Errors
    /// Returns the validation, protocol, transport, `ServerResult`, or `Pending` error.
    pub fn execute(self) -> Result<GetUsageAllocationResponse, ClientError> {
        let mut request = self
            .typed_request()
            .map_err(fluent_client_validation_error)?;
        let ver = self.execution.protocol_version();
        if let Some(id) = self.unique_identifier {
            request = request.with_unique_identifier(normalize_uid_for_version(id, ver));
        }
        self.execution.run_typed(
            ClientRequest::get_usage_allocation(request),
            |outcome| match outcome {
                ClientBatchOutcome::GetUsageAllocation(resp) => Some(resp),
                _ => None,
            },
            GetUsageAllocationResponse::result,
        )
    }

    /// Returns the raw [`ClientBatchItemResponse`] envelope.
    ///
    /// # Errors
    /// Returns the existing sanitized validation, protocol, or transport error.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let request = self
            .typed_request()
            .map_err(fluent_client_validation_error)?;
        self.execution
            .run_outcome(ClientRequest::get_usage_allocation(request))
    }

    /// Executes at most one exchange with explicit [`CodecLimits`] overrides.
    ///
    /// # Errors
    /// Returns the existing sanitized validation, protocol, or transport error.
    pub fn execute_outcome_with_limits(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let request = self
            .typed_request()
            .map_err(fluent_client_validation_error)?;
        self.execution
            .run(ClientRequest::get_usage_allocation(request), limits)
    }
}

/// A fluent RNG Retrieve builder for OASIS KMIP v2.1 §6.1.49, Tables 316–318.
/// The returned Data remains in the existing zeroizing secret owner.
pub struct RngRetrieveBuilder<'a> {
    execution: FluentBuilderExecution<'a>,
    data_length: Option<i32>,
}

impl<'a> RngRetrieveBuilder<'a> {
    pub(super) fn new(client: &'a mut Client) -> Self {
        Self {
            execution: FluentBuilderExecution::new(client),
            data_length: None,
        }
    }

    /// Sets the caller-selected number of random output bytes to request.
    ///
    /// The value is passed to the existing typed request without adding a
    /// range rule beyond that request's validator.
    #[must_use]
    pub const fn data_length(mut self, value: i32) -> Self {
        self.data_length = Some(value);
        self
    }

    /// Sets exchange-local options without changing client policy.
    #[must_use]
    pub fn with_options(mut self, options: RequestOptions) -> Self {
        self.execution = self.execution.with_options(options);
        self
    }

    /// Sets the KMIP protocol version override for this request/exchange.
    #[must_use]
    pub fn with_protocol_version(mut self, version: ProtocolVersion) -> Self {
        self.execution = self.execution.with_protocol_version(version);
        self
    }

    /// Sets the KMIP protocol version override (alias for [`Self::with_protocol_version`]).
    #[must_use]
    pub fn with_kmip_version(self, version: ProtocolVersion) -> Self {
        self.with_protocol_version(version)
    }

    /// Sets the Request Header Asynchronous Indicator for this exchange.
    #[must_use]
    pub const fn with_asynchronous_indicator(mut self, value: u32) -> Self {
        self.execution = self.execution.with_asynchronous_indicator(value);
        self
    }

    fn typed_request(&self) -> Result<RngRetrieveRequest, FluentBuildError> {
        let data_length = self
            .data_length
            .ok_or(FluentBuildError::MissingRngRetrieveDataLength)?;
        let request = RngRetrieveRequest::new(data_length);
        validate_fluent_request(&request)?;
        Ok(request)
    }

    /// Validates the required Data Length and existing typed request rules.
    ///
    /// # Errors
    /// Returns [`FluentBuildError::MissingRngRetrieveDataLength`] when Data
    /// Length is absent, or [`FluentBuildError::InvalidRequest`] when the
    /// typed request rejects the supplied values.
    pub fn validate(&self) -> Result<(), FluentBuildError> {
        self.typed_request().map(drop)
    }

    /// Builds the existing typed RNG Retrieve request.
    ///
    /// # Errors
    /// Returns a build error when Data Length is absent or typed validation
    /// fails.
    pub fn build(self) -> Result<RngRetrieveRequest, FluentBuildError> {
        self.typed_request()
    }

    /// Converts the request to a typed batch item without sending it.
    ///
    /// # Errors
    /// Returns a build error when Data Length is absent or typed validation
    /// fails.
    pub fn to_batch_item(self) -> Result<ClientBatchItem, FluentBuildError> {
        Ok(ClientBatchItem::new(ClientRequest::rng_retrieve(
            self.build()?,
        )))
    }

    /// Executes at most one exchange through the existing typed client path.
    ///
    /// Pending remains explicit; this builder does not poll or retry.
    ///
    /// # Errors
    /// Returns the unwrapped [`RngRetrieveResponse`].
    ///
    /// # Errors
    /// Returns the validation, protocol, transport, `ServerResult`, or `Pending` error.
    pub fn execute(self) -> Result<RngRetrieveResponse, ClientError> {
        let request = self
            .typed_request()
            .map_err(fluent_client_validation_error)?;
        self.execution.run_typed(
            ClientRequest::rng_retrieve(request),
            |outcome| match outcome {
                ClientBatchOutcome::RngRetrieve(resp) => Some(resp),
                _ => None,
            },
            RngRetrieveResponse::result,
        )
    }

    /// Returns the raw [`ClientBatchItemResponse`] envelope.
    ///
    /// # Errors
    /// Returns the existing sanitized validation, protocol, or transport error.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let request = self
            .typed_request()
            .map_err(fluent_client_validation_error)?;
        self.execution
            .run_outcome(ClientRequest::rng_retrieve(request))
    }

    /// Executes at most one exchange with explicit [`CodecLimits`] overrides.
    ///
    /// # Errors
    /// Returns the existing sanitized validation, protocol, or transport error.
    pub fn execute_outcome_with_limits(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let request = self
            .typed_request()
            .map_err(fluent_client_validation_error)?;
        self.execution
            .run(ClientRequest::rng_retrieve(request), limits)
    }
}

/// A fluent RNG Seed builder for OASIS KMIP v2.1 §6.1.50, Tables 319–321.
/// Caller Data is kept in the existing zeroizing secret owner.
pub struct RngSeedBuilder<'a> {
    execution: FluentBuilderExecution<'a>,
    data: Option<SecretBytes>,
}

impl<'a> RngSeedBuilder<'a> {
    pub(super) fn new(client: &'a mut Client) -> Self {
        Self {
            execution: FluentBuilderExecution::new(client),
            data: None,
        }
    }

    /// Moves caller-supplied seed bytes into `KMIPKit`'s zeroizing secret owner.
    #[must_use]
    pub fn data(mut self, value: SecretBytes) -> Self {
        self.data = Some(value);
        self
    }

    /// Sets exchange-local options without changing client policy.
    #[must_use]
    pub fn with_options(mut self, options: RequestOptions) -> Self {
        self.execution = self.execution.with_options(options);
        self
    }

    /// Sets the KMIP protocol version override for this request/exchange.
    #[must_use]
    pub fn with_protocol_version(mut self, version: ProtocolVersion) -> Self {
        self.execution = self.execution.with_protocol_version(version);
        self
    }

    /// Sets the KMIP protocol version override (alias for [`Self::with_protocol_version`]).
    #[must_use]
    pub fn with_kmip_version(self, version: ProtocolVersion) -> Self {
        self.with_protocol_version(version)
    }

    /// Sets the Request Header Asynchronous Indicator for this exchange.
    #[must_use]
    pub const fn with_asynchronous_indicator(mut self, value: u32) -> Self {
        self.execution = self.execution.with_asynchronous_indicator(value);
        self
    }

    /// Validates that the required secret Data was supplied.
    ///
    /// # Errors
    /// Returns [`FluentBuildError::MissingRngSeedData`] when Data is absent.
    pub fn validate(&self) -> Result<(), FluentBuildError> {
        self.data
            .as_ref()
            .map(|_| ())
            .ok_or(FluentBuildError::MissingRngSeedData)
    }

    fn into_request_and_execution(
        self,
    ) -> Result<(RngSeedRequest, FluentBuilderExecution<'a>), FluentBuildError> {
        let data = self.data.ok_or(FluentBuildError::MissingRngSeedData)?;
        Ok((RngSeedRequest::new(data), self.execution))
    }

    /// Builds the existing typed RNG Seed request.
    ///
    /// # Errors
    /// Returns [`FluentBuildError::MissingRngSeedData`] when Data is absent.
    pub fn build(self) -> Result<RngSeedRequest, FluentBuildError> {
        self.into_request_and_execution()
            .map(|(request, _execution)| request)
    }

    /// Converts the request to a typed batch item without sending it.
    ///
    /// # Errors
    /// Returns [`FluentBuildError::MissingRngSeedData`] when Data is absent.
    pub fn to_batch_item(self) -> Result<ClientBatchItem, FluentBuildError> {
        Ok(ClientBatchItem::new(ClientRequest::rng_seed(self.build()?)))
    }

    /// Executes at most one exchange through the existing typed client path.
    ///
    /// Pending remains explicit; this builder does not poll or retry.
    ///
    /// # Errors
    /// Returns the unwrapped [`RngSeedResponse`].
    ///
    /// # Errors
    /// Returns the validation, protocol, transport, `ServerResult`, or `Pending` error.
    pub fn execute(self) -> Result<RngSeedResponse, ClientError> {
        let (request, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run_typed(
            ClientRequest::rng_seed(request),
            |outcome| match outcome {
                ClientBatchOutcome::RngSeed(resp) => Some(resp),
                _ => None,
            },
            RngSeedResponse::result,
        )
    }

    /// Returns the raw [`ClientBatchItemResponse`] envelope.
    ///
    /// # Errors
    /// Returns the existing sanitized validation, protocol, or transport error.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let (request, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run_outcome(ClientRequest::rng_seed(request))
    }

    /// Executes at most one exchange with explicit [`CodecLimits`] overrides.
    ///
    /// # Errors
    /// Returns the existing sanitized validation, protocol, or transport error.
    pub fn execute_outcome_with_limits(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let (request, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run(ClientRequest::rng_seed(request), limits)
    }
}

/// A fluent builder for typed Check requests.
pub struct CheckBuilder<'a> {
    client: &'a mut Client,
    unique_identifier: Option<UniqueIdentifier>,
    usage_limits_count: Option<i64>,
    cryptographic_usage_mask: Option<i32>,
    lease_time: Option<u32>,
    options: Option<RequestOptions>,
    asynchronous_indicator: Option<u32>,
}

impl<'a> CheckBuilder<'a> {
    pub(super) fn new(client: &'a mut Client) -> Self {
        Self {
            client,
            unique_identifier: None,
            usage_limits_count: None,
            cryptographic_usage_mask: None,
            lease_time: None,
            options: None,
            asynchronous_indicator: None,
        }
    }

    /// Sets the optional object identifier. Omit it only when the request is
    /// placed in an ordered batch whose preceding operation produces the
    /// intended ID Placeholder.
    #[must_use]
    pub fn unique_identifier(mut self, value: impl Into<UniqueIdentifier>) -> Self {
        self.unique_identifier = Some(value.into());
        self
    }

    /// Sets the optional requested Usage Limits count.
    #[must_use]
    pub const fn usage_limits_count(mut self, value: i64) -> Self {
        self.usage_limits_count = Some(value);
        self
    }

    /// Sets the optional raw Cryptographic Usage Mask, preserving unknown bits.
    #[must_use]
    pub const fn cryptographic_usage_mask(mut self, value: i32) -> Self {
        self.cryptographic_usage_mask = Some(value);
        self
    }

    /// Sets the optional requested Lease Time interval.
    #[must_use]
    pub const fn lease_time(mut self, value: u32) -> Self {
        self.lease_time = Some(value);
        self
    }

    fluent_option_field_setters!("Sets exchange-local options without changing client policy."; options; asynchronous_indicator; const);

    /// Validates the typed Check request without sending.
    ///
    /// # Errors
    ///
    /// Returns a sanitized construction error if request validation fails.
    pub fn validate(&self) -> Result<(), FluentBuildError> {
        validate_fluent_request(&check_request(
            self.unique_identifier.clone(),
            self.usage_limits_count,
            self.cryptographic_usage_mask,
            self.lease_time,
        ))
    }

    /// Builds the existing typed Check request without sending it.
    ///
    /// # Errors
    ///
    /// Returns a sanitized construction error if request validation fails.
    pub fn build(self) -> Result<CheckRequest, FluentBuildError> {
        self.build_request()
    }

    /// Compatibility alias for [`Self::build`].
    ///
    /// # Errors
    ///
    /// Returns a sanitized construction error if request validation fails.
    pub fn build_request(self) -> Result<CheckRequest, FluentBuildError> {
        let request = check_request(
            self.unique_identifier,
            self.usage_limits_count,
            self.cryptographic_usage_mask,
            self.lease_time,
        );
        validate_fluent_request(&request)?;
        Ok(request)
    }

    /// Converts Check to the existing typed client batch item.
    ///
    /// # Errors
    ///
    /// Returns a sanitized construction error if request validation fails.
    pub fn to_batch_item(self) -> Result<ClientBatchItem, FluentBuildError> {
        self.build_request()
            .map(|request| ClientBatchItem::new(ClientRequest::Check(request)))
    }

    /// Executes one Check exchange and returns the unwrapped [`CheckResponse`].
    ///
    /// # Errors
    ///
    /// Returns a validation, protocol, transport, `ServerResult`, or `Pending` error.
    pub fn execute(self) -> Result<CheckResponse, ClientError> {
        let item = self.execute_outcome()?;
        unwrap_typed_outcome(
            item,
            |outcome| match outcome {
                ClientBatchOutcome::Check(resp) => Some(resp),
                _ => None,
            },
            CheckResponse::result,
        )
    }

    /// Executes one Check exchange using the client's configured [`CodecLimits`]
    /// and returns the raw [`ClientBatchItemResponse`] envelope.
    ///
    /// # Errors
    ///
    /// Returns a not-sent validation error before transport or exchange failure.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let Self {
            client,
            unique_identifier,
            usage_limits_count,
            cryptographic_usage_mask,
            lease_time,
            options,
            asynchronous_indicator,
        } = self;
        let ver = effective_protocol_version(Some(client), options.as_ref());
        let unique_identifier =
            unique_identifier.map(|id| normalize_uid_for_version(id, ver));
        let request = check_request(
            unique_identifier,
            usage_limits_count,
            cryptographic_usage_mask,
            lease_time,
        );
        validate_fluent_request(&request).map_err(fluent_client_validation_error)?;
        execute_fluent_outcome(
            client,
            ClientRequest::Check(request),
            asynchronous_indicator,
            options,
        )
    }

    /// Executes one Check exchange with explicit [`CodecLimits`] overrides.
    ///
    /// # Errors
    ///
    /// Returns a not-sent validation error before transport if request
    /// validation fails, or the existing client error for exchange and decoding failures.
    pub fn execute_outcome_with_limits(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let Self {
            client,
            unique_identifier,
            usage_limits_count,
            cryptographic_usage_mask,
            lease_time,
            options,
            asynchronous_indicator,
        } = self;
        let request = check_request(
            unique_identifier,
            usage_limits_count,
            cryptographic_usage_mask,
            lease_time,
        );
        validate_fluent_request(&request).map_err(fluent_client_validation_error)?;
        execute_fluent_request(
            client,
            ClientRequest::Check(request),
            asynchronous_indicator,
            options,
            limits,
        )
    }
}

fn check_request(
    unique_identifier: Option<UniqueIdentifier>,
    usage_limits_count: Option<i64>,
    cryptographic_usage_mask: Option<i32>,
    lease_time: Option<u32>,
) -> CheckRequest {
    let mut request = CheckRequest::new();
    if let Some(unique_identifier) = unique_identifier {
        request = request.with_unique_identifier(unique_identifier);
    }
    if let Some(usage_limits_count) = usage_limits_count {
        request = request.with_usage_limits_count(usage_limits_count);
    }
    if let Some(cryptographic_usage_mask) = cryptographic_usage_mask {
        request = request.with_cryptographic_usage_mask(cryptographic_usage_mask);
    }
    if let Some(lease_time) = lease_time {
        request = request.with_lease_time(lease_time);
    }
    request
}

/// A fluent builder for typed Query requests.
pub struct QueryBuilder<'a> {
    client: &'a mut Client,
    functions: Vec<QueryFunction>,
    object_groups: Option<Vec<String>>,
    options: Option<RequestOptions>,
    asynchronous_indicator: Option<u32>,
}

impl<'a> QueryBuilder<'a> {
    pub(super) fn new(client: &'a mut Client) -> Self {
        Self {
            client,
            functions: Vec::new(),
            object_groups: None,
            options: None,
            asynchronous_indicator: None,
        }
    }

    /// Sets the ordered Query Functions explicitly.
    #[must_use]
    pub fn functions(mut self, value: Vec<QueryFunction>) -> Self {
        self.functions = value;
        self
    }

    /// Sets ordered Object Group text values without deduplication.
    #[must_use]
    pub fn object_groups(mut self, value: Vec<String>) -> Self {
        self.object_groups = Some(value);
        self
    }

    fluent_option_field_setters!("Sets exchange-local options without changing client policy."; options; asynchronous_indicator; const);

    /// Validates the Query functions and typed request without sending.
    ///
    /// # Errors
    ///
    /// Returns a sanitized construction error for an empty or invalid function
    /// list, or another typed request validation failure.
    pub fn validate(&self) -> Result<(), FluentBuildError> {
        let request = query_request(self.functions.clone(), self.object_groups.clone());
        validate_fluent_request(&request)
    }

    /// Builds the existing typed Query request without sending it.
    ///
    /// # Errors
    ///
    /// Returns a sanitized construction error for an empty or invalid function
    /// list, or another typed request validation failure.
    pub fn build(self) -> Result<QueryRequest, FluentBuildError> {
        self.build_request()
    }

    /// Compatibility alias for [`Self::build`].
    ///
    /// # Errors
    ///
    /// Returns a sanitized construction error for an empty or invalid function
    /// list, or another typed request validation failure.
    pub fn build_request(self) -> Result<QueryRequest, FluentBuildError> {
        let request = query_request(self.functions, self.object_groups);
        validate_fluent_request(&request)?;
        Ok(request)
    }

    /// Converts Query to the existing typed client batch item.
    ///
    /// # Errors
    ///
    /// Returns a sanitized construction error when the request is invalid.
    pub fn to_batch_item(self) -> Result<ClientBatchItem, FluentBuildError> {
        self.build_request()
            .map(|request| ClientBatchItem::new(ClientRequest::Query(request)))
    }

    /// Executes one typed Query exchange and returns the unwrapped [`QueryResponse`].
    ///
    /// # Errors
    ///
    /// Returns a validation, protocol, transport, `ServerResult`, or `Pending` error.
    pub fn execute(self) -> Result<QueryResponse, ClientError> {
        let item = self.execute_outcome()?;
        unwrap_typed_outcome(
            item,
            |outcome| match outcome {
                ClientBatchOutcome::Query(resp) => Some(resp),
                _ => None,
            },
            QueryResponse::result,
        )
    }

    /// Executes one typed Query exchange using configured [`CodecLimits`]
    /// and returns the raw [`ClientBatchItemResponse`] envelope.
    ///
    /// # Errors
    ///
    /// Returns a not-sent validation error or exchange/decoding failure.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let Self {
            client,
            functions,
            object_groups,
            options,
            asynchronous_indicator,
        } = self;
        let request = query_request(functions, object_groups);
        validate_fluent_request(&request).map_err(fluent_client_validation_error)?;
        execute_fluent_outcome(
            client,
            ClientRequest::Query(request),
            asynchronous_indicator,
            options,
        )
    }

    /// Executes one typed Query exchange with explicit [`CodecLimits`] overrides.
    ///
    /// # Errors
    ///
    /// Returns a not-sent validation error before transport if the request is
    /// invalid, or the existing client error for exchange and decoding failures.
    pub fn execute_outcome_with_limits(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let Self {
            client,
            functions,
            object_groups,
            options,
            asynchronous_indicator,
        } = self;
        let request = query_request(functions, object_groups);
        validate_fluent_request(&request).map_err(fluent_client_validation_error)?;
        execute_fluent_request(
            client,
            ClientRequest::Query(request),
            asynchronous_indicator,
            options,
            limits,
        )
    }
}

fn query_request(
    functions: Vec<QueryFunction>,
    object_groups: Option<Vec<String>>,
) -> QueryRequest {
    let request = QueryRequest::new(functions);
    match object_groups {
        Some(groups) => request.with_object_groups(groups),
        None => request,
    }
}

/// A fluent builder for typed Discover Versions requests.
pub struct DiscoverVersionsBuilder<'a> {
    client: &'a mut Client,
    protocol_versions: Option<Vec<ProtocolVersion>>,
    options: Option<RequestOptions>,
    asynchronous_indicator: Option<u32>,
}

impl<'a> DiscoverVersionsBuilder<'a> {
    pub(super) fn new(client: &'a mut Client) -> Self {
        Self {
            client,
            protocol_versions: None,
            options: None,
            asynchronous_indicator: None,
        }
    }

    /// Sets the explicit ordered protocol versions to offer.
    ///
    /// An empty list is rejected by local validation. When this method is
    /// omitted, the existing request's KMIP 2.1 default is used.
    #[must_use]
    pub fn protocol_versions(mut self, value: Vec<ProtocolVersion>) -> Self {
        self.protocol_versions = Some(value);
        self
    }

    fluent_option_field_setters!("Sets exchange-local options without changing client policy."; options; asynchronous_indicator; const);

    /// Validates the offered versions without sending.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError::InvalidProtocolVersions`] for an empty or
    /// uncatalogued explicit offer, or a sanitized protocol construction error.
    pub fn validate(&self) -> Result<(), FluentBuildError> {
        let request = discover_versions_request(self.protocol_versions.clone())?;
        validate_fluent_request(&request)
    }

    /// Builds the existing typed Discover Versions request without sending it.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError::InvalidProtocolVersions`] for an empty or
    /// uncatalogued explicit offer, or a sanitized protocol construction error.
    pub fn build(self) -> Result<DiscoverVersionsRequest, FluentBuildError> {
        self.build_request()
    }

    /// Compatibility alias for [`Self::build`].
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError::InvalidProtocolVersions`] for an empty or
    /// uncatalogued explicit offer, or a sanitized protocol construction error.
    pub fn build_request(self) -> Result<DiscoverVersionsRequest, FluentBuildError> {
        let request = discover_versions_request(self.protocol_versions)?;
        validate_fluent_request(&request)?;
        Ok(request)
    }

    /// Converts Discover Versions to the existing typed client batch item.
    ///
    /// # Errors
    ///
    /// Returns a sanitized construction error when the request is invalid.
    pub fn to_batch_item(self) -> Result<ClientBatchItem, FluentBuildError> {
        self.build_request()
            .map(|request| ClientBatchItem::new(ClientRequest::DiscoverVersions(request)))
    }

    /// Executes one typed Discover Versions exchange and returns the unwrapped [`DiscoverVersionsResponse`].
    ///
    /// # Errors
    ///
    /// Returns a validation, protocol, transport, `ServerResult`, or `Pending` error.
    pub fn execute(self) -> Result<DiscoverVersionsResponse, ClientError> {
        let item = self.execute_outcome()?;
        unwrap_typed_outcome(
            item,
            |outcome| match outcome {
                ClientBatchOutcome::Completed(resp) => Some(resp),
                _ => None,
            },
            DiscoverVersionsResponse::result,
        )
    }

    /// Executes one typed Discover Versions exchange using configured [`CodecLimits`]
    /// and returns the raw [`ClientBatchItemResponse`] envelope.
    ///
    /// # Errors
    ///
    /// Returns a not-sent validation error or exchange/decoding failure.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let Self {
            client,
            protocol_versions,
            options,
            asynchronous_indicator,
        } = self;
        let request =
            discover_versions_request(protocol_versions).map_err(fluent_client_validation_error)?;
        validate_fluent_request(&request).map_err(fluent_client_validation_error)?;
        execute_fluent_outcome(
            client,
            ClientRequest::DiscoverVersions(request),
            asynchronous_indicator,
            options,
        )
    }

    /// Executes one typed Discover Versions exchange with explicit [`CodecLimits`] overrides.
    ///
    /// # Errors
    ///
    /// Returns a not-sent validation error before transport if the offer is
    /// invalid, or the existing client error for exchange and decoding failures.
    pub fn execute_outcome_with_limits(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let Self {
            client,
            protocol_versions,
            options,
            asynchronous_indicator,
        } = self;
        let request =
            discover_versions_request(protocol_versions).map_err(fluent_client_validation_error)?;
        validate_fluent_request(&request).map_err(fluent_client_validation_error)?;
        execute_fluent_request(
            client,
            ClientRequest::DiscoverVersions(request),
            asynchronous_indicator,
            options,
            limits,
        )
    }
}

fn discover_versions_request(
    protocol_versions: Option<Vec<ProtocolVersion>>,
) -> Result<DiscoverVersionsRequest, FluentBuildError> {
    match protocol_versions {
        Some(versions) => DiscoverVersionsRequest::with_versions(versions)
            .map_err(|_| FluentBuildError::InvalidProtocolVersions),
        None => Ok(DiscoverVersionsRequest::new()),
    }
}

/// A fluent builder for Get Usage Allocation requests.
/// A fluent builder for the existing typed Obtain Lease request.
pub struct ObtainLeaseBuilder<'a> {
    client: &'a mut Client,
    unique_identifier: Option<UniqueIdentifier>,
    options: Option<RequestOptions>,
    asynchronous_indicator: Option<u32>,
}

impl<'a> ObtainLeaseBuilder<'a> {
    pub(super) fn new(client: &'a mut Client) -> Self {
        Self {
            client,
            unique_identifier: None,
            options: None,
            asynchronous_indicator: None,
        }
    }

    /// Selects the object by its explicit identifier.
    ///
    /// Omit this field only when the request is part of an ordered batch whose
    /// preceding operation supplies the intended ID Placeholder.
    #[must_use]
    pub fn unique_identifier(mut self, value: impl Into<UniqueIdentifier>) -> Self {
        self.unique_identifier = Some(value.into());
        self
    }

    fluent_option_field_setters!("Sets exchange-local options without changing client policy."; options; asynchronous_indicator; const);

    /// Validates the typed Obtain Lease request without sending it.
    ///
    /// # Errors
    ///
    /// Returns a sanitized construction error if protocol validation fails.
    pub fn validate(&self) -> Result<(), FluentBuildError> {
        validated_obtain_lease_request(self.unique_identifier.clone()).map(|_| ())
    }

    /// Builds the existing typed Obtain Lease request without sending it.
    ///
    /// # Errors
    ///
    /// Returns a sanitized construction error if protocol validation fails.
    pub fn build(self) -> Result<ObtainLeaseRequest, FluentBuildError> {
        self.build_request()
    }

    /// Compatibility alias for [`Self::build`].
    ///
    /// # Errors
    ///
    /// Returns a sanitized construction error if protocol validation fails.
    pub fn build_request(self) -> Result<ObtainLeaseRequest, FluentBuildError> {
        validated_obtain_lease_request(self.unique_identifier)
    }

    /// Converts Obtain Lease to the existing typed client batch item.
    ///
    /// # Errors
    ///
    /// Returns a sanitized construction error if protocol validation fails.
    pub fn to_batch_item(self) -> Result<ClientBatchItem, FluentBuildError> {
        self.build_request()
            .map(|request| ClientBatchItem::new(ClientRequest::ObtainLease(request)))
    }

    /// Executes one typed Obtain Lease exchange and returns the unwrapped [`ObtainLeaseResponse`].
    ///
    /// # Errors
    ///
    /// Returns a validation, protocol, transport, `ServerResult`, or `Pending` error.
    pub fn execute(self) -> Result<ObtainLeaseResponse, ClientError> {
        let item = self.execute_outcome()?;
        unwrap_typed_outcome(
            item,
            |outcome| match outcome {
                ClientBatchOutcome::ObtainLease(resp) => Some(resp),
                _ => None,
            },
            ObtainLeaseResponse::result,
        )
    }

    /// Executes one typed Obtain Lease exchange using configured [`CodecLimits`]
    /// and returns the raw [`ClientBatchItemResponse`] envelope.
    ///
    /// # Errors
    ///
    /// Returns a not-sent validation error or exchange/decoding failure.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let Self {
            client,
            unique_identifier,
            options,
            asynchronous_indicator,
        } = self;
        let ver = effective_protocol_version(Some(client), options.as_ref());
        let unique_identifier =
            unique_identifier.map(|id| normalize_uid_for_version(id, ver));
        let request = validated_obtain_lease_request(unique_identifier)
            .map_err(fluent_client_validation_error)?;
        execute_fluent_outcome(
            client,
            ClientRequest::ObtainLease(request),
            asynchronous_indicator,
            options,
        )
    }

    /// Executes one typed Obtain Lease exchange with explicit [`CodecLimits`] overrides.
    ///
    /// # Errors
    ///
    /// Returns a not-sent validation error before transport if validation
    /// fails, or the existing client error for exchange and decoding failures.
    pub fn execute_outcome_with_limits(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let Self {
            client,
            unique_identifier,
            options,
            asynchronous_indicator,
        } = self;
        let request = validated_obtain_lease_request(unique_identifier)
            .map_err(fluent_client_validation_error)?;
        execute_fluent_request(
            client,
            ClientRequest::ObtainLease(request),
            asynchronous_indicator,
            options,
            limits,
        )
    }
}

fn obtain_lease_request(unique_identifier: Option<UniqueIdentifier>) -> ObtainLeaseRequest {
    match unique_identifier {
        Some(unique_identifier) => {
            ObtainLeaseRequest::new().with_unique_identifier(unique_identifier)
        }
        None => ObtainLeaseRequest::new(),
    }
}

fn validated_obtain_lease_request(
    unique_identifier: Option<UniqueIdentifier>,
) -> Result<ObtainLeaseRequest, FluentBuildError> {
    let request = obtain_lease_request(unique_identifier);
    validate_fluent_request(&request)?;
    Ok(request)
}
