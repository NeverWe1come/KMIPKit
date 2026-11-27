//! Task-oriented, fluent builders over the existing typed KMIP requests.


use kmipkit_protocol::{
    CheckRequest, CheckResponse, DiscoverVersionsRequest, DiscoverVersionsResponse,
    GetConstraintsRequest, GetConstraintsResponse, GetUsageAllocationRequest,
    GetUsageAllocationResponse, InteropRequest, InteropResponse, LogRequest, LogResponse,
    ObtainLeaseRequest, ObtainLeaseResponse, PingRequest, PingResponse, Pkcs11Request,
    Pkcs11Response, ProtocolVersion, QueryFunction, QueryRequest, QueryResponse,
    ReProvisionRequest, ReProvisionResponse, RngRetrieveRequest, RngRetrieveResponse,
    RngSeedRequest, RngSeedResponse, SecretBytes, SecretText, SetConstraintsRequest,
    SetConstraintsResponse, SetDefaultsRequest, SetDefaultsResponse, UniqueIdentifier,
};
use kmipkit_transport::RequestOptions;
use kmipkit_ttlv::{Item, Structure};

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
    fluent_option_field_setters!(
        "Sets exchange-local options without changing client policy.";
        execution.options;
        execution.asynchronous_indicator;
        const
    );

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

    /// Executes at most one exchange using the client's configured [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits)
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

    /// Executes at most one exchange using the client's configured [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits)
    /// and returns the raw [`ClientBatchItemResponse`] envelope.
    ///
    /// # Errors
    /// Returns the existing sanitized validation, protocol, or transport error.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        self.execution
            .run_outcome(ClientRequest::Ping(PingRequest::new()))
    }

    /// Executes at most one exchange with explicit [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits) overrides.
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

    fluent_option_field_setters!(
        "Sets exchange-local options without changing client policy.";
        execution.options;
        execution.asynchronous_indicator;
        const
    );

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

    /// Executes at most one exchange using configured [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits)
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

    /// Executes at most one exchange using configured [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits)
    /// and returns the raw [`ClientBatchItemResponse`] envelope.
    ///
    /// # Errors
    /// Returns the existing sanitized protocol or transport error.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        self.execution.run_outcome(ClientRequest::get_constraints())
    }

    /// Executes at most one exchange with explicit [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits) overrides.
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

    fluent_option_field_setters!(
        "Sets exchange-local options without changing client policy.";
        execution.options;
        execution.asynchronous_indicator;
        const
    );

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

    /// Executes at most one exchange with explicit [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits) overrides.
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

    fluent_option_field_setters!(
        "Sets exchange-local options without changing client policy.";
        execution.options;
        execution.asynchronous_indicator;
        const
    );

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

    /// Executes at most one exchange with explicit [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits) overrides.
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

    fluent_option_field_setters!(
        "Sets exchange-local options without changing client policy.";
        execution.options;
        execution.asynchronous_indicator;
        const
    );

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

    /// Executes at most one exchange with explicit [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits) overrides.
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

    /// Executes one Check exchange using the client's configured [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits)
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

    /// Executes one Check exchange with explicit [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits) overrides.
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

    /// Executes one typed Query exchange using configured [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits)
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

    /// Executes one typed Query exchange with explicit [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits) overrides.
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

    /// Executes one typed Discover Versions exchange using configured [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits)
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

    /// Executes one typed Discover Versions exchange with explicit [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits) overrides.
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

    /// Executes one typed Obtain Lease exchange using configured [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits)
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

    /// Executes one typed Obtain Lease exchange with explicit [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits) overrides.
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

fn clone_items(items: &[Item]) -> Result<Vec<Item>, FluentBuildError> {
    items
        .iter()
        .map(|item| {
            item.try_clone()
                .map_err(|_| FluentBuildError::InvalidTtlvValue)
        })
        .collect()
}

/// A fluent builder for KMIP Log requests (§6.1.29, Table 250).
pub struct LogBuilder<'a> {
    execution: FluentBuilderExecution<'a>,
    log_message: SecretText,
    extensions: Vec<Item>,
}

impl<'a> LogBuilder<'a> {
    pub(super) fn new(client: &'a mut Client, log_message: &str) -> Self {
        Self {
            execution: FluentBuilderExecution::new(client),
            log_message: SecretText::new(log_message.to_owned()),
            extensions: Vec::new(),
        }
    }

    /// Appends an unknown operation-payload extension Item in caller-selected order.
    #[must_use]
    pub fn extension(mut self, extension: Item) -> Self {
        self.extensions.push(extension);
        self
    }

    fluent_option_field_setters!(
        "Sets exchange-local options without changing client policy.";
        execution.options;
        execution.asynchronous_indicator;
        const
    );

    /// Validates extension tags without touching the secret message.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError::InvalidRequest`] if an extension overrides a defined field.
    pub fn validate(&self) -> Result<(), FluentBuildError> {
        let mut request = LogRequest::new(SecretText::new(String::new()));
        for ext in clone_items(&self.extensions)? {
            request = request.with_extension(ext);
        }
        validate_fluent_request(&request)
    }

    fn into_request_and_execution(
        self,
    ) -> Result<(LogRequest, FluentBuilderExecution<'a>), FluentBuildError> {
        let mut request = LogRequest::new(self.log_message);
        for ext in self.extensions {
            request = request.with_extension(ext);
        }
        validate_fluent_request(&request)?;
        Ok((request, self.execution))
    }

    /// Builds the typed [`LogRequest`] without sending it.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError::InvalidRequest`] if an extension overrides a defined field.
    pub fn build(self) -> Result<LogRequest, FluentBuildError> {
        self.into_request_and_execution().map(|(req, _)| req)
    }

    /// Converts Log to a typed [`ClientBatchItem`].
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError`] if typed request validation fails.
    pub fn to_batch_item(self) -> Result<ClientBatchItem, FluentBuildError> {
        self.build()
            .map(|req| ClientBatchItem::new(ClientRequest::Log(req)))
    }

    /// Executes one Log exchange and returns the unwrapped [`LogResponse`].
    ///
    /// # Errors
    ///
    /// Returns a validation, protocol, transport, `ServerResult`, or `Pending` error.
    pub fn execute(self) -> Result<LogResponse, ClientError> {
        let (req, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run_typed(
            ClientRequest::Log(req),
            |outcome| match outcome {
                ClientBatchOutcome::Log(resp) => Some(resp),
                _ => None,
            },
            LogResponse::result,
        )
    }

    /// Executes one Log exchange and returns the raw [`ClientBatchItemResponse`].
    ///
    /// # Errors
    ///
    /// Returns validation, exchange, or decoding failures.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let (req, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run_outcome(ClientRequest::Log(req))
    }

    /// Executes one Log exchange with explicit [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits).
    ///
    /// # Errors
    ///
    /// Returns validation, exchange, or decoding failures.
    pub fn execute_outcome_with_limits(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let (req, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run(ClientRequest::Log(req), limits)
    }
}

/// A fluent builder for KMIP Interop requests (§6.1.26, Table 241).
pub struct InteropBuilder<'a> {
    execution: FluentBuilderExecution<'a>,
    interop_function: u32,
    interop_identifier: SecretText,
    extensions: Vec<Item>,
}

impl<'a> InteropBuilder<'a> {
    pub(super) fn new(
        client: &'a mut Client,
        interop_function: u32,
        interop_identifier: &str,
    ) -> Self {
        Self {
            execution: FluentBuilderExecution::new(client),
            interop_function,
            interop_identifier: SecretText::new(interop_identifier.to_owned()),
            extensions: Vec::new(),
        }
    }

    /// Appends an unknown operation-payload extension Item in caller-selected order.
    #[must_use]
    pub fn extension(mut self, extension: Item) -> Self {
        self.extensions.push(extension);
        self
    }

    fluent_option_field_setters!(
        "Sets exchange-local options without changing client policy.";
        execution.options;
        execution.asynchronous_indicator;
        const
    );

    /// Validates extension tags without touching the secret identifier.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError::InvalidRequest`] if an extension overrides a defined field.
    pub fn validate(&self) -> Result<(), FluentBuildError> {
        let mut request =
            InteropRequest::new(self.interop_function, SecretText::new(String::new()));
        for ext in clone_items(&self.extensions)? {
            request = request.with_extension(ext);
        }
        validate_fluent_request(&request)
    }

    fn into_request_and_execution(
        self,
    ) -> Result<(InteropRequest, FluentBuilderExecution<'a>), FluentBuildError> {
        let mut request = InteropRequest::new(self.interop_function, self.interop_identifier);
        for ext in self.extensions {
            request = request.with_extension(ext);
        }
        validate_fluent_request(&request)?;
        Ok((request, self.execution))
    }

    /// Builds the typed [`InteropRequest`] without sending it.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError::InvalidRequest`] if typed validation fails.
    pub fn build(self) -> Result<InteropRequest, FluentBuildError> {
        self.into_request_and_execution().map(|(req, _)| req)
    }

    /// Converts Interop to a typed [`ClientBatchItem`].
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError`] if typed validation fails.
    pub fn to_batch_item(self) -> Result<ClientBatchItem, FluentBuildError> {
        self.build()
            .map(|req| ClientBatchItem::new(ClientRequest::Interop(req)))
    }

    /// Executes one Interop exchange and returns the unwrapped [`InteropResponse`].
    ///
    /// # Errors
    ///
    /// Returns a validation, protocol, transport, `ServerResult`, or `Pending` error.
    pub fn execute(self) -> Result<InteropResponse, ClientError> {
        let (req, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run_typed(
            ClientRequest::Interop(req),
            |outcome| match outcome {
                ClientBatchOutcome::Interop(resp) => Some(resp),
                _ => None,
            },
            InteropResponse::result,
        )
    }

    /// Executes one Interop exchange and returns the raw [`ClientBatchItemResponse`].
    ///
    /// # Errors
    ///
    /// Returns validation, exchange, or decoding failures.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let (req, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run_outcome(ClientRequest::Interop(req))
    }

    /// Executes one Interop exchange with explicit [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits).
    ///
    /// # Errors
    ///
    /// Returns validation, exchange, or decoding failures.
    pub fn execute_outcome_with_limits(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let (req, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run(ClientRequest::Interop(req), limits)
    }
}

/// A fluent builder for KMIP PKCS#11 forwarding requests (§6.1.37, Table 273).
pub struct Pkcs11Builder<'a> {
    execution: FluentBuilderExecution<'a>,
    function: Option<Structure>,
    interface: Option<Structure>,
    correlation: Option<SecretBytes>,
    input: Option<Structure>,
    extensions: Vec<Item>,
}

impl<'a> Pkcs11Builder<'a> {
    pub(super) fn new(client: &'a mut Client) -> Self {
        Self {
            execution: FluentBuilderExecution::new(client),
            function: None,
            interface: None,
            correlation: None,
            input: None,
            extensions: Vec::new(),
        }
    }

    /// Sets the required PKCS#11 Function Structure.
    #[must_use]
    pub fn function(mut self, function: Structure) -> Self {
        self.function = Some(function);
        self
    }

    /// Sets the optional PKCS#11 Interface Structure.
    #[must_use]
    pub fn interface(mut self, interface: Structure) -> Self {
        self.interface = Some(interface);
        self
    }

    /// Sets the optional previous server Correlation Value bytes.
    #[must_use]
    pub fn correlation_value(mut self, correlation: SecretBytes) -> Self {
        self.correlation = Some(correlation);
        self
    }

    /// Sets the optional PKCS#11 Input Parameters Structure.
    #[must_use]
    pub fn input_parameters(mut self, input: Structure) -> Self {
        self.input = Some(input);
        self
    }

    /// Appends an extension Item to the PKCS#11 payload.
    #[must_use]
    pub fn extension(mut self, extension: Item) -> Self {
        self.extensions.push(extension);
        self
    }

    fluent_option_field_setters!(
        "Sets exchange-local options without changing client policy.";
        execution.options;
        execution.asynchronous_indicator;
        const
    );

    /// Validates required builder fields without consuming `self`.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError::MissingPkcs11Function`] if Function is unset.
    pub const fn validate(&self) -> Result<(), FluentBuildError> {
        if self.function.is_none() {
            return Err(FluentBuildError::MissingPkcs11Function);
        }
        Ok(())
    }

    fn into_request_and_execution(
        self,
    ) -> Result<(Pkcs11Request, FluentBuilderExecution<'a>), FluentBuildError> {
        let function = self
            .function
            .ok_or(FluentBuildError::MissingPkcs11Function)?;
        let mut request = Pkcs11Request::new(function);
        if let Some(interface) = self.interface {
            request = request.with_interface(interface);
        }
        if let Some(correlation) = self.correlation {
            request = request.with_correlation_value(correlation);
        }
        if let Some(input) = self.input {
            request = request.with_input_parameters(input);
        }
        for ext in self.extensions {
            request = request.with_extension(ext);
        }
        Ok((request, self.execution))
    }

    /// Builds the typed [`Pkcs11Request`] without sending it.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError::MissingPkcs11Function`] if Function is unset.
    pub fn build(self) -> Result<Pkcs11Request, FluentBuildError> {
        self.into_request_and_execution().map(|(req, _)| req)
    }

    /// Converts PKCS#11 to a typed [`ClientBatchItem`].
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError`] if required fields are missing.
    pub fn to_batch_item(self) -> Result<ClientBatchItem, FluentBuildError> {
        self.build()
            .map(|req| ClientBatchItem::new(ClientRequest::Pkcs11(req)))
    }

    /// Executes one PKCS#11 exchange and returns the unwrapped [`Pkcs11Response`].
    ///
    /// # Errors
    ///
    /// Returns a validation, protocol, transport, `ServerResult`, or `Pending` error.
    pub fn execute(self) -> Result<Pkcs11Response, ClientError> {
        let (req, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run_typed(
            ClientRequest::Pkcs11(req),
            |outcome| match outcome {
                ClientBatchOutcome::Pkcs11(resp) => Some(resp),
                _ => None,
            },
            Pkcs11Response::result,
        )
    }

    /// Executes one PKCS#11 exchange and returns the raw [`ClientBatchItemResponse`].
    ///
    /// # Errors
    ///
    /// Returns validation, exchange, or decoding failures.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let (req, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run_outcome(ClientRequest::Pkcs11(req))
    }

    /// Executes one PKCS#11 exchange with explicit [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits).
    ///
    /// # Errors
    ///
    /// Returns validation, exchange, or decoding failures.
    pub fn execute_outcome_with_limits(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let (req, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run(ClientRequest::Pkcs11(req), limits)
    }
}

/// A fluent builder for KMIP Re-Provision requests (§6.1.48, Table 313).
pub struct ReProvisionBuilder<'a> {
    execution: FluentBuilderExecution<'a>,
    certificate_request: Option<Structure>,
    certificate: Option<Structure>,
}

impl<'a> ReProvisionBuilder<'a> {
    pub(super) fn new(client: &'a mut Client) -> Self {
        Self {
            execution: FluentBuilderExecution::new(client),
            certificate_request: None,
            certificate: None,
        }
    }

    /// Sets the optional Certificate Request Structure to be signed by the server.
    #[must_use]
    pub fn certificate_request(mut self, value: Structure) -> Self {
        self.certificate_request = Some(value);
        self
    }

    /// Sets the optional Certificate Structure to associate with the client.
    #[must_use]
    pub fn certificate(mut self, value: Structure) -> Self {
        self.certificate = Some(value);
        self
    }

    fluent_option_field_setters!(
        "Sets exchange-local options without changing client policy.";
        execution.options;
        execution.asynchronous_indicator;
        const
    );

    /// Validates the typed Re-Provision request without sending it.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError::InvalidRequest`] if nested structure validation fails.
    pub fn validate(&self) -> Result<(), FluentBuildError> {
        let mut request = ReProvisionRequest::new();
        if let Some(req) = &self.certificate_request {
            let copy = req
                .view()
                .try_clone()
                .map_err(|_| FluentBuildError::InvalidTtlvValue)?;
            request = request.with_certificate_request(copy);
        }
        if let Some(cert) = &self.certificate {
            let copy = cert
                .view()
                .try_clone()
                .map_err(|_| FluentBuildError::InvalidTtlvValue)?;
            request = request.with_certificate(copy);
        }
        validate_fluent_request(&request)
    }

    fn into_request_and_execution(
        self,
    ) -> Result<(ReProvisionRequest, FluentBuilderExecution<'a>), FluentBuildError> {
        let mut request = ReProvisionRequest::new();
        if let Some(cert_req) = self.certificate_request {
            request = request.with_certificate_request(cert_req);
        }
        if let Some(cert) = self.certificate {
            request = request.with_certificate(cert);
        }
        validate_fluent_request(&request)?;
        Ok((request, self.execution))
    }

    /// Builds the typed [`ReProvisionRequest`] without sending it.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError`] if nested structure validation fails.
    pub fn build(self) -> Result<ReProvisionRequest, FluentBuildError> {
        self.into_request_and_execution().map(|(req, _)| req)
    }

    /// Converts Re-Provision to a typed [`ClientBatchItem`].
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError`] if typed request validation fails.
    pub fn to_batch_item(self) -> Result<ClientBatchItem, FluentBuildError> {
        self.build()
            .map(|req| ClientBatchItem::new(ClientRequest::ReProvision(req)))
    }

    /// Executes one Re-Provision exchange and returns [`ReProvisionResponse`].
    ///
    /// # Errors
    ///
    /// Returns a validation, protocol, transport, `ServerResult`, or `Pending` error.
    pub fn execute(self) -> Result<ReProvisionResponse, ClientError> {
        let (req, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run_typed(
            ClientRequest::ReProvision(req),
            |outcome| match outcome {
                ClientBatchOutcome::ReProvision(resp) => Some(resp),
                _ => None,
            },
            ReProvisionResponse::result,
        )
    }

    /// Executes one Re-Provision exchange and returns [`ClientBatchItemResponse`].
    ///
    /// # Errors
    ///
    /// Returns validation, exchange, or decoding failures.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let (req, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run_outcome(ClientRequest::ReProvision(req))
    }

    /// Executes one Re-Provision exchange with explicit [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits).
    ///
    /// # Errors
    ///
    /// Returns validation, exchange, or decoding failures.
    pub fn execute_outcome_with_limits(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let (req, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run(ClientRequest::ReProvision(req), limits)
    }
}

/// A fluent builder for KMIP Set Defaults requests (§6.1.53, Table 328).
pub struct SetDefaultsBuilder<'a> {
    execution: FluentBuilderExecution<'a>,
    request: SetDefaultsRequest,
}

impl<'a> SetDefaultsBuilder<'a> {
    pub(super) fn new(client: &'a mut Client) -> Self {
        Self {
            execution: FluentBuilderExecution::new(client),
            request: SetDefaultsRequest::new(),
        }
    }

    /// Sets the optional Defaults Information Structure (omitted clears defaults).
    #[must_use]
    pub fn defaults_information(mut self, information: Structure) -> Self {
        self.request = self.request.with_defaults_information(information);
        self
    }

    fluent_option_field_setters!(
        "Sets exchange-local options without changing client policy.";
        execution.options;
        execution.asynchronous_indicator;
        const
    );

    /// Validates the typed Set Defaults request locally (always valid).
    ///
    /// # Errors
    ///
    /// This method is infallible and always returns `Ok(())`.
    pub const fn validate(&self) -> Result<(), FluentBuildError> {
        Ok(())
    }

    /// Builds the typed [`SetDefaultsRequest`] without sending it.
    #[must_use]
    pub fn build(self) -> SetDefaultsRequest {
        self.request
    }

    /// Converts Set Defaults to a typed [`ClientBatchItem`].
    #[must_use]
    pub fn to_batch_item(self) -> ClientBatchItem {
        ClientBatchItem::new(ClientRequest::SetDefaults(self.request))
    }

    /// Executes one Set Defaults exchange and returns [`SetDefaultsResponse`].
    ///
    /// # Errors
    ///
    /// Returns a validation, protocol, transport, `ServerResult`, or `Pending` error.
    pub fn execute(self) -> Result<SetDefaultsResponse, ClientError> {
        self.execution.run_typed(
            ClientRequest::SetDefaults(self.request),
            |outcome| match outcome {
                ClientBatchOutcome::SetDefaults(resp) => Some(resp),
                _ => None,
            },
            SetDefaultsResponse::result,
        )
    }

    /// Executes one Set Defaults exchange and returns [`ClientBatchItemResponse`].
    ///
    /// # Errors
    ///
    /// Returns exchange or decoding failures.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        self.execution
            .run_outcome(ClientRequest::SetDefaults(self.request))
    }

    /// Executes one Set Defaults exchange with explicit [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits).
    ///
    /// # Errors
    ///
    /// Returns exchange or decoding failures.
    pub fn execute_outcome_with_limits(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        self.execution
            .run(ClientRequest::SetDefaults(self.request), limits)
    }
}

/// A fluent builder for KMIP Set Constraints requests (§6.1.52, Table 325).
pub struct SetConstraintsBuilder<'a> {
    execution: FluentBuilderExecution<'a>,
    constraints: Option<Structure>,
}

impl<'a> SetConstraintsBuilder<'a> {
    pub(super) fn new(client: &'a mut Client) -> Self {
        Self {
            execution: FluentBuilderExecution::new(client),
            constraints: None,
        }
    }

    /// Sets the required ordered Constraints Structure.
    #[must_use]
    pub fn constraints(mut self, constraints: Structure) -> Self {
        self.constraints = Some(constraints);
        self
    }

    fluent_option_field_setters!(
        "Sets exchange-local options without changing client policy.";
        execution.options;
        execution.asynchronous_indicator;
        const
    );

    /// Validates required builder fields without consuming `self`.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError::MissingSetConstraints`] if Constraints is unset.
    pub const fn validate(&self) -> Result<(), FluentBuildError> {
        if self.constraints.is_none() {
            return Err(FluentBuildError::MissingSetConstraints);
        }
        Ok(())
    }

    fn into_request_and_execution(
        self,
    ) -> Result<(SetConstraintsRequest, FluentBuilderExecution<'a>), FluentBuildError> {
        let constraints = self
            .constraints
            .ok_or(FluentBuildError::MissingSetConstraints)?;
        Ok((SetConstraintsRequest::new(constraints), self.execution))
    }

    /// Builds the typed [`SetConstraintsRequest`] without sending it.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError::MissingSetConstraints`] if Constraints is unset.
    pub fn build(self) -> Result<SetConstraintsRequest, FluentBuildError> {
        self.into_request_and_execution().map(|(req, _)| req)
    }

    /// Converts Set Constraints to a typed [`ClientBatchItem`].
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError`] if required fields are missing.
    pub fn to_batch_item(self) -> Result<ClientBatchItem, FluentBuildError> {
        self.build()
            .map(|req| ClientBatchItem::new(ClientRequest::SetConstraints(req)))
    }

    /// Executes one Set Constraints exchange and returns [`SetConstraintsResponse`].
    ///
    /// # Errors
    ///
    /// Returns a validation, protocol, transport, `ServerResult`, or `Pending` error.
    pub fn execute(self) -> Result<SetConstraintsResponse, ClientError> {
        let (req, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run_typed(
            ClientRequest::SetConstraints(req),
            |outcome| match outcome {
                ClientBatchOutcome::SetConstraints(resp) => Some(resp),
                _ => None,
            },
            SetConstraintsResponse::result,
        )
    }

    /// Executes one Set Constraints exchange and returns [`ClientBatchItemResponse`].
    ///
    /// # Errors
    ///
    /// Returns validation, exchange, or decoding failures.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let (req, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run_outcome(ClientRequest::SetConstraints(req))
    }

    /// Executes one Set Constraints exchange with explicit [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits).
    ///
    /// # Errors
    ///
    /// Returns validation, exchange, or decoding failures.
    pub fn execute_outcome_with_limits(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let (req, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run(ClientRequest::SetConstraints(req), limits)
    }
}
