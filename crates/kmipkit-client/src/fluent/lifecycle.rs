//! Task-oriented, fluent builders over the existing typed KMIP requests.


use kmipkit_protocol::{
    ActivateRequest, ActivateResponse, ArchiveRequest, ArchiveResponse, AttributeSet,
    DestroyRequest, DestroyResponse, GetRequest, GetResponse, KeyCompressionType, KeyFormatType,
    KeyWrapType, LocateRequest, LocateResponse, ObjectClassMask, ObjectGroupMember, ProtocolVersion,
    RecertifyRequest, RecertifyResponse, RecoverRequest, RecoverResponse, RekeyKeyPairRequest,
    RekeyKeyPairResponse, RekeyRequest, RekeyResponse, RevocationReason, RevokeRequest,
    RevokeResponse, SecretBytes, StorageStatusMask,
    UniqueIdentifier, ValidateKmipRequest, ValidationReport,
};
use kmipkit_transport::RequestOptions;
use kmipkit_ttlv::Structure;

use crate::{
    Client, ClientBatchItem, ClientBatchItemResponse, ClientBatchOutcome, ClientError,
    ClientRequest,
};

use super::execution::{
    FluentBuildError, effective_protocol_version, execute_fluent_outcome, execute_fluent_request, fluent_client_validation_error, normalize_uid_for_version, unwrap_typed_outcome, validate_fluent_request,
};

/// A fluent builder for an existing typed Get request.
pub struct GetBuilder<'a> {
    client: &'a mut Client,
    request: GetRequest,
    options: Option<RequestOptions>,
    asynchronous_indicator: Option<u32>,
}

impl<'a> GetBuilder<'a> {
    pub(super) fn new(client: &'a mut Client) -> Self {
        Self {
            client,
            request: GetRequest::new(),
            options: None,
            asynchronous_indicator: None,
        }
    }
}

impl GetBuilder<'_> {
    /// Sets exchange-local options.
    #[must_use]
    pub fn with_options(mut self, options: RequestOptions) -> Self {
        self.options = Some(options);
        self
    }

    /// Sets the KMIP protocol version override for this request/exchange.
    #[must_use]
    pub fn with_protocol_version(mut self, version: ProtocolVersion) -> Self {
        let options = self.options.unwrap_or_default().with_protocol_version(version);
        self.options = Some(options);
        if let Some(uid) = self.request.unique_identifier().cloned() {
            self.request = self
                .request
                .with_unique_identifier(normalize_uid_for_version(uid, version));
        }
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
        self.asynchronous_indicator = Some(value);
        self
    }

    /// Selects the object identifier. Leaving it unset uses the KMIP ID Placeholder.
    #[must_use]
    pub fn unique_identifier(mut self, value: impl Into<UniqueIdentifier>) -> Self {
        let version = effective_protocol_version(Some(self.client), self.options.as_ref());
        self.request = self
            .request
            .with_unique_identifier(normalize_uid_for_version(value.into(), version));
        self
    }

    /// Sets the requested Key Format Type.
    #[must_use]
    pub fn key_format_type(mut self, value: KeyFormatType) -> Self {
        self.request = self.request.with_key_format_type(value);
        self
    }

    /// Sets the original KMIP code, including extension values.
    #[must_use]
    pub fn key_format_type_code(self, value: u32) -> Self {
        self.key_format_type(KeyFormatType::from_raw(value))
    }

    /// Sets the requested Key Wrap Type.
    #[must_use]
    pub fn key_wrap_type(mut self, value: KeyWrapType) -> Self {
        self.request = self.request.with_key_wrap_type(value);
        self
    }

    /// Sets the original KMIP code, including extension values.
    #[must_use]
    pub fn key_wrap_type_code(self, value: u32) -> Self {
        self.key_wrap_type(KeyWrapType::from_raw(value))
    }

    /// Sets the requested Key Compression Type.
    #[must_use]
    pub fn key_compression_type(mut self, value: KeyCompressionType) -> Self {
        self.request = self.request.with_key_compression_type(value);
        self
    }

    /// Sets the original KMIP code, including extension values.
    #[must_use]
    pub fn key_compression_type_code(self, value: u32) -> Self {
        self.key_compression_type(KeyCompressionType::from_raw(value))
    }

    /// Sets the optional Key Wrapping Specification.
    #[must_use]
    pub fn key_wrapping_specification(mut self, value: Structure) -> Self {
        self.request = self.request.with_key_wrapping_specification(value);
        self
    }

    /// Checks the request payload locally using the same validator as execution.
    #[must_use]
    pub fn validate(&self) -> ValidationReport {
        self.request.validate()
    }

    /// Builds the existing typed Get request without sending it.
    #[must_use]
    pub fn build(self) -> GetRequest {
        self.request
    }

    /// Converts the request to an existing typed batch item.
    #[must_use]
    pub fn to_batch_item(self) -> ClientBatchItem {
        ClientBatchItem::new(ClientRequest::Get(self.request))
    }

    /// Executes one typed Get exchange using configured [`CodecLimits`]
    /// and returns the unwrapped [`GetResponse`].
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] for validation, transport, protocol, `ServerResult`, or `Pending`.
    pub fn execute(self) -> Result<GetResponse, ClientError> {
        let item = self.execute_outcome()?;
        unwrap_typed_outcome(
            item,
            |outcome| match outcome {
                ClientBatchOutcome::GetCompleted(resp) => Some(resp),
                _ => None,
            },
            GetResponse::result,
        )
    }

    /// Executes one typed Get exchange using configured [`CodecLimits`]
    /// and returns the raw [`ClientBatchItemResponse`] envelope.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] when request validation, transport, or decoding fails.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let options = self.options.unwrap_or_default();
        self.client.send_one_with_async_indicator(
            ClientRequest::Get(self.request),
            self.asynchronous_indicator,
            &options,
        )
    }

    /// Executes one typed Get exchange with explicit [`CodecLimits`] overrides.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] when request validation, transport, or response
    /// decoding fails.
    pub fn execute_outcome_with_limits(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let options = self.options.unwrap_or_default();
        self.client.execute_one_typed_request_with_async_indicator(
            ClientRequest::Get(self.request),
            self.asynchronous_indicator,
            limits,
            &options,
        )
    }
}

/// A fluent builder for an existing typed Destroy request.
pub struct DestroyBuilder<'a> {
    client: &'a mut Client,
    request: DestroyRequest,
    options: Option<RequestOptions>,
    asynchronous_indicator: Option<u32>,
}

impl<'a> DestroyBuilder<'a> {
    pub(super) fn new(client: &'a mut Client, unique_identifier: Option<UniqueIdentifier>) -> Self {
        Self {
            client,
            request: DestroyRequest::new(unique_identifier),
            options: None,
            asynchronous_indicator: None,
        }
    }
}

impl DestroyBuilder<'_> {
    /// Sets exchange-local options.
    #[must_use]
    pub fn with_options(mut self, options: RequestOptions) -> Self {
        self.options = Some(options);
        self
    }

    /// Sets the KMIP protocol version override for this request/exchange.
    #[must_use]
    pub fn with_protocol_version(mut self, version: ProtocolVersion) -> Self {
        let options = self.options.unwrap_or_default().with_protocol_version(version);
        self.options = Some(options);
        if let Some(uid) = self.request.unique_identifier().cloned() {
            self.request = DestroyRequest::new(Some(normalize_uid_for_version(uid, version)));
        }
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
        self.asynchronous_indicator = Some(value);
        self
    }

    /// Builds the existing typed Destroy request without sending it.
    #[must_use]
    pub fn build(self) -> DestroyRequest {
        self.request
    }

    /// Converts the request to an existing typed batch item.
    #[must_use]
    pub fn to_batch_item(self) -> ClientBatchItem {
        ClientBatchItem::new(ClientRequest::Destroy(self.request))
    }

    /// Executes one typed Destroy exchange using configured [`CodecLimits`]
    /// and returns the unwrapped [`DestroyResponse`].
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] for validation, transport, protocol, `ServerResult`, or `Pending`.
    pub fn execute(self) -> Result<DestroyResponse, ClientError> {
        let item = self.execute_outcome()?;
        unwrap_typed_outcome(
            item,
            |outcome| match outcome {
                ClientBatchOutcome::Destroy(resp) => Some(resp),
                _ => None,
            },
            DestroyResponse::result,
        )
    }

    /// Executes one typed Destroy exchange using configured [`CodecLimits`]
    /// and returns the raw [`ClientBatchItemResponse`] envelope.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] when request validation, transport, or decoding fails.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let options = self.options.unwrap_or_default();
        self.client.send_one_with_async_indicator(
            ClientRequest::Destroy(self.request),
            self.asynchronous_indicator,
            &options,
        )
    }

    /// Executes one typed Destroy exchange with explicit [`CodecLimits`] overrides.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] when request validation, transport, or response
    /// decoding fails.
    pub fn execute_outcome_with_limits(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let options = self.options.unwrap_or_default();
        self.client.execute_one_typed_request_with_async_indicator(
            ClientRequest::Destroy(self.request),
            self.asynchronous_indicator,
            limits,
            &options,
        )
    }
}

/// A fluent builder for an existing typed Activate request.
pub struct ActivateBuilder<'a> {
    client: &'a mut Client,
    request: ActivateRequest,
    options: Option<RequestOptions>,
    asynchronous_indicator: Option<u32>,
}

impl<'a> ActivateBuilder<'a> {
    pub(super) fn new(client: &'a mut Client) -> Self {
        Self {
            client,
            request: ActivateRequest::new(None),
            options: None,
            asynchronous_indicator: None,
        }
    }

    /// Sets the optional Unique Identifier (`impl Into<UniqueIdentifier>`).
    #[must_use]
    pub fn unique_identifier(mut self, value: impl Into<UniqueIdentifier>) -> Self {
        let version = effective_protocol_version(Some(self.client), self.options.as_ref());
        self.request = ActivateRequest::new(Some(normalize_uid_for_version(value.into(), version)));
        self
    }

    /// Sets exchange-local options without changing client policy.
    #[must_use]
    pub fn with_options(mut self, options: RequestOptions) -> Self {
        self.options = Some(options);
        self
    }

    /// Sets the KMIP protocol version override for this request/exchange.
    #[must_use]
    pub fn with_protocol_version(mut self, version: ProtocolVersion) -> Self {
        let options = self.options.unwrap_or_default().with_protocol_version(version);
        self.options = Some(options);
        if let Some(uid) = self.request.unique_identifier().cloned() {
            self.request = ActivateRequest::new(Some(normalize_uid_for_version(uid, version)));
        }
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
        self.asynchronous_indicator = Some(value);
        self
    }

    /// Validates the typed request without sending it.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError::InvalidRequest`] if the typed validator rejects the fields.
    pub fn validate(&self) -> Result<(), FluentBuildError> {
        validate_fluent_request(&self.request)
    }

    /// Builds the existing typed request without sending it.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError::InvalidRequest`] if the typed validator rejects the fields.
    pub fn build_request(self) -> Result<ActivateRequest, FluentBuildError> {
        validate_fluent_request(&self.request)?;
        Ok(self.request)
    }

    /// Converts this request to its typed batch item.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError::InvalidRequest`] if the typed validator rejects the fields.
    pub fn to_batch_item(self) -> Result<ClientBatchItem, FluentBuildError> {
        self.build_request()
            .map(|request| ClientBatchItem::new(ClientRequest::Activate(request)))
    }

    /// Executes one typed Activate exchange using configured [`CodecLimits`]
    /// and returns the unwrapped [`ActivateResponse`].
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] when validation, transport, protocol, `ServerResult`, or `Pending` occurs.
    pub fn execute(self) -> Result<ActivateResponse, ClientError> {
        let item = self.execute_outcome()?;
        unwrap_typed_outcome(
            item,
            |outcome| match outcome {
                ClientBatchOutcome::Activate(resp) => Some(resp),
                _ => None,
            },
            ActivateResponse::result,
        )
    }

    /// Executes one typed Activate exchange using configured [`CodecLimits`]
    /// and returns the raw [`ClientBatchItemResponse`] envelope.
    ///
    /// # Errors
    ///
    /// Returns a validation error before sending or the existing client error.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let Self {
            client,
            request,
            options,
            asynchronous_indicator,
        } = self;
        validate_fluent_request(&request).map_err(fluent_client_validation_error)?;
        execute_fluent_outcome(
            client,
            ClientRequest::Activate(request),
            asynchronous_indicator,
            options,
        )
    }

    /// Executes one typed Activate exchange with explicit [`CodecLimits`] overrides.
    ///
    /// # Errors
    ///
    /// Returns a validation error before sending when the request is invalid, or the existing
    /// client error for exchange and response failures.
    pub fn execute_outcome_with_limits(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let Self {
            client,
            request,
            options,
            asynchronous_indicator,
        } = self;
        validate_fluent_request(&request).map_err(fluent_client_validation_error)?;
        execute_fluent_request(
            client,
            ClientRequest::Activate(request),
            asynchronous_indicator,
            options,
            limits,
        )
    }
}

/// A fluent builder for an existing typed Archive request.
pub struct ArchiveBuilder<'a> {
    client: &'a mut Client,
    request: ArchiveRequest,
    options: Option<RequestOptions>,
    asynchronous_indicator: Option<u32>,
}

impl<'a> ArchiveBuilder<'a> {
    pub(super) fn new(client: &'a mut Client) -> Self {
        Self {
            client,
            request: ArchiveRequest::new(None),
            options: None,
            asynchronous_indicator: None,
        }
    }

    /// Sets the optional Unique Identifier (`impl Into<UniqueIdentifier>`).
    #[must_use]
    pub fn unique_identifier(mut self, value: impl Into<UniqueIdentifier>) -> Self {
        let version = effective_protocol_version(Some(self.client), self.options.as_ref());
        self.request = ArchiveRequest::new(Some(normalize_uid_for_version(value.into(), version)));
        self
    }

    /// Sets exchange-local options without changing client policy.
    #[must_use]
    pub fn with_options(mut self, options: RequestOptions) -> Self {
        self.options = Some(options);
        self
    }

    /// Sets the KMIP protocol version override for this request/exchange.
    #[must_use]
    pub fn with_protocol_version(mut self, version: ProtocolVersion) -> Self {
        let options = self.options.unwrap_or_default().with_protocol_version(version);
        self.options = Some(options);
        if let Some(uid) = self.request.unique_identifier().cloned() {
            self.request = ArchiveRequest::new(Some(normalize_uid_for_version(uid, version)));
        }
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
        self.asynchronous_indicator = Some(value);
        self
    }

    /// Validates the typed request without sending it.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError::InvalidRequest`] if the typed validator rejects the fields.
    pub fn validate(&self) -> Result<(), FluentBuildError> {
        validate_fluent_request(&self.request)
    }

    /// Builds the existing typed request without sending it.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError::InvalidRequest`] if the typed validator rejects the fields.
    pub fn build_request(self) -> Result<ArchiveRequest, FluentBuildError> {
        validate_fluent_request(&self.request)?;
        Ok(self.request)
    }

    /// Converts this request to its typed batch item.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError::InvalidRequest`] if the typed validator rejects the fields.
    pub fn to_batch_item(self) -> Result<ClientBatchItem, FluentBuildError> {
        self.build_request()
            .map(|request| ClientBatchItem::new(ClientRequest::Archive(request)))
    }

    /// Executes one typed Archive exchange using configured [`CodecLimits`]
    /// and returns the unwrapped [`ArchiveResponse`].
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] when validation, transport, protocol, `ServerResult`, or `Pending` occurs.
    pub fn execute(self) -> Result<ArchiveResponse, ClientError> {
        let item = self.execute_outcome()?;
        unwrap_typed_outcome(
            item,
            |outcome| match outcome {
                ClientBatchOutcome::Archive(resp) => Some(resp),
                _ => None,
            },
            ArchiveResponse::result,
        )
    }

    /// Executes one typed Archive exchange using configured [`CodecLimits`]
    /// and returns the raw [`ClientBatchItemResponse`] envelope.
    ///
    /// # Errors
    ///
    /// Returns a validation error before sending or the existing client error.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let Self {
            client,
            request,
            options,
            asynchronous_indicator,
        } = self;
        validate_fluent_request(&request).map_err(fluent_client_validation_error)?;
        execute_fluent_outcome(
            client,
            ClientRequest::Archive(request),
            asynchronous_indicator,
            options,
        )
    }

    /// Executes one typed Archive exchange with explicit [`CodecLimits`] overrides.
    ///
    /// # Errors
    ///
    /// Returns a validation error before sending when the request is invalid, or the existing
    /// client error for exchange and response failures.
    pub fn execute_outcome_with_limits(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let Self {
            client,
            request,
            options,
            asynchronous_indicator,
        } = self;
        validate_fluent_request(&request).map_err(fluent_client_validation_error)?;
        execute_fluent_request(
            client,
            ClientRequest::Archive(request),
            asynchronous_indicator,
            options,
            limits,
        )
    }
}

/// A fluent builder for an existing typed Recover request.
pub struct RecoverBuilder<'a> {
    client: &'a mut Client,
    request: RecoverRequest,
    options: Option<RequestOptions>,
    asynchronous_indicator: Option<u32>,
}

impl<'a> RecoverBuilder<'a> {
    pub(super) fn new(client: &'a mut Client) -> Self {
        Self {
            client,
            request: RecoverRequest::new(None),
            options: None,
            asynchronous_indicator: None,
        }
    }

    /// Sets the optional Unique Identifier (`impl Into<UniqueIdentifier>`).
    #[must_use]
    pub fn unique_identifier(mut self, value: impl Into<UniqueIdentifier>) -> Self {
        let version = effective_protocol_version(Some(self.client), self.options.as_ref());
        self.request = RecoverRequest::new(Some(normalize_uid_for_version(value.into(), version)));
        self
    }

    /// Sets exchange-local options without changing client policy.
    #[must_use]
    pub fn with_options(mut self, options: RequestOptions) -> Self {
        self.options = Some(options);
        self
    }

    /// Sets the KMIP protocol version override for this request/exchange.
    #[must_use]
    pub fn with_protocol_version(mut self, version: ProtocolVersion) -> Self {
        let options = self.options.unwrap_or_default().with_protocol_version(version);
        self.options = Some(options);
        if let Some(uid) = self.request.unique_identifier().cloned() {
            self.request = RecoverRequest::new(Some(normalize_uid_for_version(uid, version)));
        }
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
        self.asynchronous_indicator = Some(value);
        self
    }

    /// Validates the typed request without sending it.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError::InvalidRequest`] if the typed validator rejects the fields.
    pub fn validate(&self) -> Result<(), FluentBuildError> {
        validate_fluent_request(&self.request)
    }

    /// Builds the existing typed request without sending it.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError::InvalidRequest`] if the typed validator rejects the fields.
    pub fn build_request(self) -> Result<RecoverRequest, FluentBuildError> {
        validate_fluent_request(&self.request)?;
        Ok(self.request)
    }

    /// Converts this request to its typed batch item.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError::InvalidRequest`] if the typed validator rejects the fields.
    pub fn to_batch_item(self) -> Result<ClientBatchItem, FluentBuildError> {
        self.build_request()
            .map(|request| ClientBatchItem::new(ClientRequest::Recover(request)))
    }

    /// Executes one typed Recover exchange using configured [`CodecLimits`]
    /// and returns the unwrapped [`RecoverResponse`].
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] when validation, transport, protocol, `ServerResult`, or `Pending` occurs.
    pub fn execute(self) -> Result<RecoverResponse, ClientError> {
        let item = self.execute_outcome()?;
        unwrap_typed_outcome(
            item,
            |outcome| match outcome {
                ClientBatchOutcome::Recover(resp) => Some(resp),
                _ => None,
            },
            RecoverResponse::result,
        )
    }

    /// Executes one typed Recover exchange using configured [`CodecLimits`]
    /// and returns the raw [`ClientBatchItemResponse`] envelope.
    ///
    /// # Errors
    ///
    /// Returns a validation error before sending or the existing client error.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let Self {
            client,
            request,
            options,
            asynchronous_indicator,
        } = self;
        validate_fluent_request(&request).map_err(fluent_client_validation_error)?;
        execute_fluent_outcome(
            client,
            ClientRequest::Recover(request),
            asynchronous_indicator,
            options,
        )
    }

    /// Executes one typed Recover exchange with explicit [`CodecLimits`] overrides.
    ///
    /// # Errors
    ///
    /// Returns a validation error before sending when the request is invalid, or the existing
    /// client error for exchange and response failures.
    pub fn execute_outcome_with_limits(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let Self {
            client,
            request,
            options,
            asynchronous_indicator,
        } = self;
        validate_fluent_request(&request).map_err(fluent_client_validation_error)?;
        execute_fluent_request(
            client,
            ClientRequest::Recover(request),
            asynchronous_indicator,
            options,
            limits,
        )
    }
}

/// A fluent builder for an existing typed Revoke request.
pub struct RevokeBuilder<'a> {
    client: &'a mut Client,
    request: RevokeRequest,
    options: Option<RequestOptions>,
    asynchronous_indicator: Option<u32>,
}

impl<'a> RevokeBuilder<'a> {
    pub(super) fn new(client: &'a mut Client, reason: RevocationReason) -> Self {
        Self {
            client,
            request: RevokeRequest::new(reason),
            options: None,
            asynchronous_indicator: None,
        }
    }

    /// Sets the optional Unique Identifier (`impl Into<UniqueIdentifier>`).
    #[must_use]
    pub fn unique_identifier(mut self, value: impl Into<UniqueIdentifier>) -> Self {
        let version = effective_protocol_version(Some(self.client), self.options.as_ref());
        self.request = self
            .request
            .with_unique_identifier(normalize_uid_for_version(value.into(), version));
        self
    }

    /// Sets the optional audit Revocation Message.
    #[must_use]
    pub fn revocation_message(mut self, value: &str) -> Self {
        self.request = self.request.with_revocation_message(value.to_owned());
        self
    }

    /// Sets the optional compromise occurrence date in KMIP Date-Time seconds.
    #[must_use]
    pub fn compromise_occurrence_date(mut self, value: i64) -> Self {
        self.request = self.request.with_compromise_occurrence_date(value);
        self
    }

    /// Sets exchange-local options without changing client policy.
    #[must_use]
    pub fn with_options(mut self, options: RequestOptions) -> Self {
        self.options = Some(options);
        self
    }

    /// Sets the Request Header Asynchronous Indicator for this exchange.
    #[must_use]
    pub const fn with_asynchronous_indicator(mut self, value: u32) -> Self {
        self.asynchronous_indicator = Some(value);
        self
    }

    /// Validates the typed Revoke request without sending it.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError::InvalidRequest`] if the typed request
    /// validator rejects the supplied reason, identifier, or date combination.
    pub fn validate(&self) -> Result<(), FluentBuildError> {
        validate_fluent_request(&self.request)
    }

    /// Builds the existing typed Revoke request without sending it.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError::InvalidRequest`] if the typed request
    /// validator rejects the supplied reason, identifier, or date combination.
    pub fn build_request(self) -> Result<RevokeRequest, FluentBuildError> {
        validate_fluent_request(&self.request)?;
        Ok(self.request)
    }

    /// Converts this request to its typed batch item.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError::InvalidRequest`] if the typed request
    /// validator rejects the supplied reason, identifier, or date combination.
    pub fn to_batch_item(self) -> Result<ClientBatchItem, FluentBuildError> {
        self.build_request()
            .map(|request| ClientBatchItem::new(ClientRequest::Revoke(request)))
    }

    /// Executes exactly one typed Revoke exchange using configured [`CodecLimits`]
    /// and returns the unwrapped [`RevokeResponse`].
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] when validation, transport, protocol, `ServerResult`, or `Pending` occurs.
    pub fn execute(self) -> Result<RevokeResponse, ClientError> {
        let item = self.execute_outcome()?;
        unwrap_typed_outcome(
            item,
            |outcome| match outcome {
                ClientBatchOutcome::Revoke(resp) => Some(resp),
                _ => None,
            },
            RevokeResponse::result,
        )
    }

    /// Executes exactly one typed Revoke exchange with the supplied options using
    /// configured [`CodecLimits`] and returns the unwrapped [`RevokeResponse`].
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] when validation, transport, protocol, `ServerResult`, or `Pending` occurs.
    pub fn execute_with_options(
        mut self,
        options: RequestOptions,
    ) -> Result<RevokeResponse, ClientError> {
        self.options = Some(options);
        let item = self.execute_outcome()?;
        unwrap_typed_outcome(
            item,
            |outcome| match outcome {
                ClientBatchOutcome::Revoke(resp) => Some(resp),
                _ => None,
            },
            RevokeResponse::result,
        )
    }

    /// Executes exactly one typed Revoke exchange using configured [`CodecLimits`]
    /// and returns the raw [`ClientBatchItemResponse`] envelope.
    ///
    /// # Errors
    ///
    /// Returns a validation error before sending or the existing client error.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let Self {
            client,
            request,
            options,
            asynchronous_indicator,
        } = self;
        validate_fluent_request(&request).map_err(fluent_client_validation_error)?;
        execute_fluent_outcome(
            client,
            ClientRequest::Revoke(request),
            asynchronous_indicator,
            options,
        )
    }

    /// Executes exactly one typed Revoke exchange with explicit [`CodecLimits`] overrides.
    ///
    /// # Errors
    ///
    /// Returns a validation error before sending when the typed request is
    /// invalid, or the existing client error for exchange and response failures.
    pub fn execute_outcome_with_limits(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let Self {
            client,
            request,
            options,
            asynchronous_indicator,
        } = self;
        validate_fluent_request(&request).map_err(fluent_client_validation_error)?;
        execute_fluent_request(
            client,
            ClientRequest::Revoke(request),
            asynchronous_indicator,
            options,
            limits,
        )
    }

    /// Executes exactly one typed Revoke exchange with explicit [`CodecLimits`] and [`RequestOptions`].
    ///
    /// # Errors
    ///
    /// Returns a validation error before sending when the request is invalid,
    /// or the existing client error for exchange and response failures.
    pub fn execute_outcome_with_limits_and_options(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
        options: RequestOptions,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let Self {
            client,
            request,
            asynchronous_indicator,
            ..
        } = self;
        validate_fluent_request(&request).map_err(fluent_client_validation_error)?;
        execute_fluent_request(
            client,
            ClientRequest::Revoke(request),
            asynchronous_indicator,
            Some(options),
            limits,
        )
    }
}

/// A fluent builder for an existing typed Locate request.
pub struct LocateBuilder<'a> {
    client: &'a mut Client,
    request: LocateRequest,
    options: Option<RequestOptions>,
    asynchronous_indicator: Option<u32>,
}

impl<'a> LocateBuilder<'a> {
    pub(super) fn new(client: &'a mut Client, attributes: AttributeSet) -> Self {
        Self {
            client,
            request: LocateRequest::new(attributes),
            options: None,
            asynchronous_indicator: None,
        }
    }

    /// Sets the complete ordered Attributes filter set.
    #[must_use]
    pub fn attributes(mut self, attributes: AttributeSet) -> Self {
        self.request = self.request.with_attributes(attributes);
        self
    }

    /// Limits the number of located objects returned by the server.
    #[must_use]
    pub fn maximum_items(mut self, value: i32) -> Self {
        self.request = self.request.with_maximum_items(value);
        self
    }

    /// Selects the zero-based server-side result offset.
    #[must_use]
    pub fn offset_items(mut self, value: i32) -> Self {
        self.request = self.request.with_offset_items(value);
        self
    }

    /// Sets the raw-preserving Storage Status Mask filter.
    #[must_use]
    pub fn storage_status_mask(mut self, value: StorageStatusMask) -> Self {
        self.request = self.request.with_storage_status_mask(value);
        self
    }

    /// Sets the raw-preserving Object Group Member filter.
    #[must_use]
    pub fn object_group_member(mut self, value: ObjectGroupMember) -> Self {
        self.request = self.request.with_object_group_member(value);
        self
    }

    /// Sets the KMIP 3.0 Object Class Mask filter.
    #[must_use]
    pub fn object_class_mask(mut self, value: ObjectClassMask) -> Self {
        self.request = self.request.with_object_class_mask(value);
        self
    }

    /// Sets exchange-local options without changing client policy.
    #[must_use]
    pub fn with_options(mut self, options: RequestOptions) -> Self {
        self.options = Some(options);
        self
    }

    /// Sets the Request Header Asynchronous Indicator for this exchange.
    #[must_use]
    pub const fn with_asynchronous_indicator(mut self, value: u32) -> Self {
        self.asynchronous_indicator = Some(value);
        self
    }

    /// Validates the typed Locate request without sending it.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError::InvalidRequest`] if the typed Locate request
    /// validator rejects the supplied filters or attributes.
    pub fn validate(&self) -> Result<(), FluentBuildError> {
        validate_fluent_request(&self.request)
    }

    /// Builds the existing typed Locate request without sending it.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError::InvalidRequest`] if the typed Locate request
    /// validator rejects the supplied filters or attributes.
    pub fn build_request(self) -> Result<LocateRequest, FluentBuildError> {
        validate_fluent_request(&self.request)?;
        Ok(self.request)
    }

    /// Converts this request to its typed batch item.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError::InvalidRequest`] if the typed Locate request
    /// validator rejects the supplied filters or attributes.
    pub fn to_batch_item(self) -> Result<ClientBatchItem, FluentBuildError> {
        self.build_request()
            .map(|request| ClientBatchItem::new(ClientRequest::Locate(request)))
    }

    /// Executes exactly one typed Locate exchange using configured [`CodecLimits`]
    /// and returns the unwrapped [`LocateResponse`].
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] when validation, transport, protocol, `ServerResult`, or `Pending` occurs.
    pub fn execute(self) -> Result<LocateResponse, ClientError> {
        let item = self.execute_outcome()?;
        unwrap_typed_outcome(
            item,
            |outcome| match outcome {
                ClientBatchOutcome::LocateCompleted(resp) => Some(resp),
                _ => None,
            },
            LocateResponse::result,
        )
    }

    /// Executes exactly one typed Locate exchange with the supplied options using
    /// configured [`CodecLimits`] and returns the unwrapped [`LocateResponse`].
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] when validation, transport, protocol, `ServerResult`, or `Pending` occurs.
    pub fn execute_with_options(
        mut self,
        options: RequestOptions,
    ) -> Result<LocateResponse, ClientError> {
        self.options = Some(options);
        let item = self.execute_outcome()?;
        unwrap_typed_outcome(
            item,
            |outcome| match outcome {
                ClientBatchOutcome::LocateCompleted(resp) => Some(resp),
                _ => None,
            },
            LocateResponse::result,
        )
    }

    /// Executes exactly one typed Locate exchange using configured [`CodecLimits`]
    /// and returns the raw [`ClientBatchItemResponse`] envelope.
    ///
    /// # Errors
    ///
    /// Returns a validation error before sending or the existing client error.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let Self {
            client,
            request,
            options,
            asynchronous_indicator,
        } = self;
        validate_fluent_request(&request).map_err(fluent_client_validation_error)?;
        execute_fluent_outcome(
            client,
            ClientRequest::Locate(request),
            asynchronous_indicator,
            options,
        )
    }

    /// Executes exactly one typed Locate exchange with explicit [`CodecLimits`] overrides.
    ///
    /// # Errors
    ///
    /// Returns a validation error before sending when the typed request is
    /// invalid, or the existing client error for exchange and response failures.
    pub fn execute_outcome_with_limits(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let Self {
            client,
            request,
            options,
            asynchronous_indicator,
        } = self;
        validate_fluent_request(&request).map_err(fluent_client_validation_error)?;
        execute_fluent_request(
            client,
            ClientRequest::Locate(request),
            asynchronous_indicator,
            options,
            limits,
        )
    }

    /// Executes exactly one typed Locate exchange with explicit [`CodecLimits`] and [`RequestOptions`].
    ///
    /// # Errors
    ///
    /// Returns a validation error before sending when the request is invalid,
    /// or the existing client error for exchange and response failures.
    pub fn execute_outcome_with_limits_and_options(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
        options: RequestOptions,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let Self {
            client,
            request,
            asynchronous_indicator,
            ..
        } = self;
        validate_fluent_request(&request).map_err(fluent_client_validation_error)?;
        execute_fluent_request(
            client,
            ClientRequest::Locate(request),
            asynchronous_indicator,
            Some(options),
            limits,
        )
    }
}

trait ReplacementRequest: Sized + ValidateKmipRequest {
    fn new() -> Self;
    fn with_offset(self, value: u32) -> Self;
    fn with_attributes(self, value: Structure) -> Self;
    fn with_protection_storage_masks(self, value: Structure) -> Self;
    fn into_client_request(self) -> ClientRequest;
}

impl ReplacementRequest for RekeyRequest {
    fn new() -> Self {
        Self::new()
    }

    fn with_offset(self, value: u32) -> Self {
        self.with_offset(value)
    }

    fn with_attributes(self, value: Structure) -> Self {
        self.with_attributes(value)
    }

    fn with_protection_storage_masks(self, value: Structure) -> Self {
        self.with_protection_storage_masks(value)
    }

    fn into_client_request(self) -> ClientRequest {
        ClientRequest::Rekey(self)
    }
}

impl ReplacementRequest for RecertifyRequest {
    fn new() -> Self {
        Self::new()
    }

    fn with_offset(self, value: u32) -> Self {
        self.with_offset(value)
    }

    fn with_attributes(self, value: Structure) -> Self {
        self.with_attributes(value)
    }

    fn with_protection_storage_masks(self, value: Structure) -> Self {
        self.with_protection_storage_masks(value)
    }

    fn into_client_request(self) -> ClientRequest {
        ClientRequest::Recertify(self)
    }
}

struct ReplacementBuilderState<'a, R> {
    client: &'a mut Client,
    request: R,
    options: Option<RequestOptions>,
    asynchronous_indicator: Option<u32>,
}

impl<'a, R: ReplacementRequest> ReplacementBuilderState<'a, R> {
    pub(super) fn new(client: &'a mut Client) -> Self {
        Self {
            client,
            request: R::new(),
            options: None,
            asynchronous_indicator: None,
        }
    }

    fn offset(mut self, value: u32) -> Self {
        self.request = self.request.with_offset(value);
        self
    }

    fn attributes(mut self, value: Structure) -> Self {
        self.request = self.request.with_attributes(value);
        self
    }

    fn protection_storage_masks(mut self, value: Structure) -> Self {
        self.request = self.request.with_protection_storage_masks(value);
        self
    }

    fn with_options(mut self, options: RequestOptions) -> Self {
        self.options = Some(options);
        self
    }

    fn with_protocol_version(mut self, version: ProtocolVersion) -> Self {
        let options = self.options.unwrap_or_default().with_protocol_version(version);
        self.options = Some(options);
        self
    }

    fn with_asynchronous_indicator(mut self, value: u32) -> Self {
        self.asynchronous_indicator = Some(value);
        self
    }

    fn validate(&self) -> Result<(), FluentBuildError> {
        validate_fluent_request(&self.request)
    }

    fn build_request(self) -> Result<R, FluentBuildError> {
        validate_fluent_request(&self.request)?;
        Ok(self.request)
    }

    fn into_batch_item(self) -> Result<ClientBatchItem, FluentBuildError> {
        self.build_request()
            .map(ReplacementRequest::into_client_request)
            .map(ClientBatchItem::new)
    }

    fn run_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let Self {
            client,
            request,
            options,
            asynchronous_indicator,
        } = self;
        validate_fluent_request(&request).map_err(fluent_client_validation_error)?;
        execute_fluent_outcome(
            client,
            request.into_client_request(),
            asynchronous_indicator,
            options,
        )
    }

    fn run(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let Self {
            client,
            request,
            options,
            asynchronous_indicator,
        } = self;
        validate_fluent_request(&request).map_err(fluent_client_validation_error)?;
        execute_fluent_request(
            client,
            request.into_client_request(),
            asynchronous_indicator,
            options,
            limits,
        )
    }
}

/// A fluent builder for an existing typed Re-key request.
pub struct RekeyBuilder<'a> {
    state: ReplacementBuilderState<'a, RekeyRequest>,
}

impl<'a> RekeyBuilder<'a> {
    pub(super) fn new(client: &'a mut Client) -> Self {
        Self {
            state: ReplacementBuilderState::new(client),
        }
    }

    /// Sets the optional source key identifier (`impl Into<UniqueIdentifier>`).
    #[must_use]
    pub fn unique_identifier(mut self, value: impl Into<UniqueIdentifier>) -> Self {
        let version = effective_protocol_version(Some(self.state.client), self.state.options.as_ref());
        self.state.request = std::mem::take(&mut self.state.request)
            .with_unique_identifier(normalize_uid_for_version(value.into(), version));
        self
    }

    /// Sets the server-side date difference for the replacement object.
    #[must_use]
    pub fn offset(mut self, value: u32) -> Self {
        self.state.request = std::mem::take(&mut self.state.request).with_offset(value);
        self
    }

    /// Sets the ordered generic Attributes Structure.
    #[must_use]
    pub fn attributes(mut self, value: Structure) -> Self {
        self.state.request = std::mem::take(&mut self.state.request).with_attributes(value);
        self
    }

    /// Sets the optional Protection Storage Masks Structure.
    #[must_use]
    pub fn protection_storage_masks(mut self, value: Structure) -> Self {
        self.state.request =
            std::mem::take(&mut self.state.request).with_protection_storage_masks(value);
        self
    }

    /// Sets exchange-local options without changing client policy.
    #[must_use]
    pub fn with_options(mut self, options: RequestOptions) -> Self {
        self.state.options = Some(options);
        self
    }

    /// Sets the KMIP protocol version override for this request/exchange.
    #[must_use]
    pub fn with_protocol_version(self, version: ProtocolVersion) -> Self {
        Self {
            state: self.state.with_protocol_version(version),
        }
    }

    /// Sets the KMIP protocol version override (alias for [`Self::with_protocol_version`]).
    #[must_use]
    pub fn with_kmip_version(self, version: ProtocolVersion) -> Self {
        self.with_protocol_version(version)
    }

    /// Sets the Request Header Asynchronous Indicator for this exchange.
    #[must_use]
    pub fn with_asynchronous_indicator(mut self, value: u32) -> Self {
        self.state.asynchronous_indicator = Some(value);
        self
    }

    /// Validates the typed request without sending it.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError::InvalidRequest`] if typed validation fails.
    pub fn validate(&self) -> Result<(), FluentBuildError> {
        validate_fluent_request(&self.state.request)
    }

    /// Builds the existing typed request without sending it.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError::InvalidRequest`] if typed validation fails.
    pub fn build_request(self) -> Result<RekeyRequest, FluentBuildError> {
        self.state.build_request()
    }

    /// Converts this request to its typed batch item.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError::InvalidRequest`] if typed validation fails.
    pub fn to_batch_item(self) -> Result<ClientBatchItem, FluentBuildError> {
        self.state.into_batch_item()
    }

    /// Executes one Re-key exchange using configured [`CodecLimits`]
    /// and returns the unwrapped [`RekeyResponse`].
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] when validation, transport, protocol, `ServerResult`, or `Pending` occurs.
    pub fn execute(self) -> Result<RekeyResponse, ClientError> {
        let item = self.state.run_outcome()?;
        unwrap_typed_outcome(
            item,
            |outcome| match outcome {
                ClientBatchOutcome::Rekey(resp) => Some(resp),
                _ => None,
            },
            RekeyResponse::result,
        )
    }

    /// Executes one Re-key exchange using configured [`CodecLimits`]
    /// and returns the raw [`ClientBatchItemResponse`] envelope.
    ///
    /// # Errors
    ///
    /// Returns a validation error before sending or the existing client error.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        self.state.run_outcome()
    }

    /// Executes one Re-key exchange with explicit [`CodecLimits`] overrides.
    ///
    /// # Errors
    ///
    /// Returns a validation error before sending or the existing client error for
    /// exchange and response failures.
    pub fn execute_outcome_with_limits(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        self.state.run(limits)
    }
}

/// A fluent builder for an existing typed Re-key Key Pair request.
pub struct RekeyKeyPairBuilder<'a> {
    client: &'a mut Client,
    request: RekeyKeyPairRequest,
    options: Option<RequestOptions>,
    asynchronous_indicator: Option<u32>,
}

impl<'a> RekeyKeyPairBuilder<'a> {
    pub(super) fn new(client: &'a mut Client) -> Self {
        Self {
            client,
            request: RekeyKeyPairRequest::new(),
            options: None,
            asynchronous_indicator: None,
        }
    }

    /// Sets the optional source Private Key identifier in the typed request.
    #[must_use]
    pub fn private_key_unique_identifier(mut self, value: UniqueIdentifier) -> Self {
        self.request = std::mem::take(&mut self.request).with_private_key_unique_identifier(value);
        self
    }

    /// Sets the server-side date difference for the replacement key pair.
    #[must_use]
    pub fn offset(mut self, value: u32) -> Self {
        self.request = std::mem::take(&mut self.request).with_offset(value);
        self
    }

    /// Sets the attributes shared by both replacement keys.
    #[must_use]
    pub fn common_attributes(mut self, value: AttributeSet) -> Self {
        self.request = std::mem::take(&mut self.request).with_common_attributes(value);
        self
    }

    /// Sets attributes for the replacement private key.
    #[must_use]
    pub fn private_key_attributes(mut self, value: AttributeSet) -> Self {
        self.request = std::mem::take(&mut self.request).with_private_key_attributes(value);
        self
    }

    /// Sets attributes for the replacement public key.
    #[must_use]
    pub fn public_key_attributes(mut self, value: AttributeSet) -> Self {
        self.request = std::mem::take(&mut self.request).with_public_key_attributes(value);
        self
    }

    /// Sets masks shared by both replacement keys.
    #[must_use]
    pub fn common_protection_storage_masks(mut self, value: Structure) -> Self {
        self.request =
            std::mem::take(&mut self.request).with_common_protection_storage_masks(value);
        self
    }

    /// Sets masks for the replacement private key.
    #[must_use]
    pub fn private_protection_storage_masks(mut self, value: Structure) -> Self {
        self.request =
            std::mem::take(&mut self.request).with_private_protection_storage_masks(value);
        self
    }

    /// Sets masks for the replacement public key.
    #[must_use]
    pub fn public_protection_storage_masks(mut self, value: Structure) -> Self {
        self.request =
            std::mem::take(&mut self.request).with_public_protection_storage_masks(value);
        self
    }

    /// Sets exchange-local options without changing client policy.
    #[must_use]
    pub fn with_options(mut self, options: RequestOptions) -> Self {
        self.options = Some(options);
        self
    }

    /// Sets the KMIP protocol version override for this request/exchange.
    #[must_use]
    pub fn with_protocol_version(mut self, version: ProtocolVersion) -> Self {
        let options = self.options.unwrap_or_default().with_protocol_version(version);
        self.options = Some(options);
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
        self.asynchronous_indicator = Some(value);
        self
    }

    /// Validates the typed request without sending it.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError::InvalidRequest`] if the typed validator rejects the fields.
    pub fn validate(&self) -> Result<(), FluentBuildError> {
        validate_fluent_request(&self.request)
    }

    /// Builds the existing typed request without sending it.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError::InvalidRequest`] if the typed validator rejects the fields.
    pub fn build_request(self) -> Result<RekeyKeyPairRequest, FluentBuildError> {
        validate_fluent_request(&self.request)?;
        Ok(self.request)
    }

    /// Converts this request to its typed batch item.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError::InvalidRequest`] if the typed validator rejects the fields.
    pub fn to_batch_item(self) -> Result<ClientBatchItem, FluentBuildError> {
        self.build_request()
            .map(|request| ClientBatchItem::new(ClientRequest::RekeyKeyPair(request)))
    }

    /// Executes one Re-key Key Pair exchange using configured [`CodecLimits`]
    /// and returns the unwrapped [`RekeyKeyPairResponse`].
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] when validation, transport, protocol, `ServerResult`, or `Pending` occurs.
    pub fn execute(self) -> Result<RekeyKeyPairResponse, ClientError> {
        let item = self.execute_outcome()?;
        unwrap_typed_outcome(
            item,
            |outcome| match outcome {
                ClientBatchOutcome::RekeyKeyPair(resp) => Some(resp),
                _ => None,
            },
            RekeyKeyPairResponse::result,
        )
    }

    /// Executes one Re-key Key Pair exchange using configured [`CodecLimits`]
    /// and returns the raw [`ClientBatchItemResponse`] envelope.
    ///
    /// # Errors
    ///
    /// Returns a validation error before sending or the existing client error.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let Self {
            client,
            request,
            options,
            asynchronous_indicator,
        } = self;
        validate_fluent_request(&request).map_err(fluent_client_validation_error)?;
        execute_fluent_outcome(
            client,
            ClientRequest::RekeyKeyPair(request),
            asynchronous_indicator,
            options,
        )
    }

    /// Executes one Re-key Key Pair exchange with explicit [`CodecLimits`] overrides.
    ///
    /// # Errors
    ///
    /// Returns a validation error before sending when the request is invalid, or the existing
    /// client error for exchange and response failures.
    pub fn execute_outcome_with_limits(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let Self {
            client,
            request,
            options,
            asynchronous_indicator,
        } = self;
        validate_fluent_request(&request).map_err(fluent_client_validation_error)?;
        execute_fluent_request(
            client,
            ClientRequest::RekeyKeyPair(request),
            asynchronous_indicator,
            options,
            limits,
        )
    }
}

/// A fluent builder for an existing typed Re-certify request.
pub struct RecertifyBuilder<'a> {
    state: ReplacementBuilderState<'a, RecertifyRequest>,
}

impl<'a> RecertifyBuilder<'a> {
    pub(super) fn new(client: &'a mut Client) -> Self {
        Self {
            state: ReplacementBuilderState::new(client),
        }
    }

    /// Selects the existing certificate to renew (`impl Into<UniqueIdentifier>`).
    #[must_use]
    pub fn unique_identifier(mut self, value: impl Into<UniqueIdentifier>) -> Self {
        let version = effective_protocol_version(Some(self.state.client), self.state.options.as_ref());
        self.state.request = std::mem::take(&mut self.state.request)
            .with_unique_identifier(normalize_uid_for_version(value.into(), version));
        self
    }

    /// Selects a certificate request object (`impl Into<UniqueIdentifier>`).
    #[must_use]
    pub fn certificate_request_unique_identifier(mut self, value: impl Into<UniqueIdentifier>) -> Self {
        let version = effective_protocol_version(Some(self.state.client), self.state.options.as_ref());
        self.state.request = std::mem::take(&mut self.state.request)
            .with_certificate_request_unique_identifier(normalize_uid_for_version(value.into(), version));
        self
    }

    /// Sets the raw KMIP Certificate Request Type enumeration value.
    #[must_use]
    pub fn certificate_request_type(mut self, value: u32) -> Self {
        self.state.request =
            std::mem::take(&mut self.state.request).with_certificate_request_type(value);
        self
    }

    /// Sets the secret Certificate Request Value with zeroizing ownership.
    #[must_use]
    pub fn certificate_request_value(mut self, value: SecretBytes) -> Self {
        self.state.request =
            std::mem::take(&mut self.state.request).with_certificate_request_value(value);
        self
    }

    /// Sets the server-side date difference for the replacement object.
    #[must_use]
    pub fn offset(self, value: u32) -> Self {
        Self {
            state: self.state.offset(value),
        }
    }

    /// Sets the ordered generic Attributes Structure.
    #[must_use]
    pub fn attributes(self, value: Structure) -> Self {
        Self {
            state: self.state.attributes(value),
        }
    }

    /// Sets the optional Protection Storage Masks Structure.
    #[must_use]
    pub fn protection_storage_masks(self, value: Structure) -> Self {
        Self {
            state: self.state.protection_storage_masks(value),
        }
    }

    /// Sets exchange-local options without changing client policy.
    #[must_use]
    pub fn with_options(self, options: RequestOptions) -> Self {
        Self {
            state: self.state.with_options(options),
        }
    }

    /// Sets the KMIP protocol version override for this request/exchange.
    #[must_use]
    pub fn with_protocol_version(self, version: ProtocolVersion) -> Self {
        Self {
            state: self.state.with_protocol_version(version),
        }
    }

    /// Sets the KMIP protocol version override (alias for [`Self::with_protocol_version`]).
    #[must_use]
    pub fn with_kmip_version(self, version: ProtocolVersion) -> Self {
        self.with_protocol_version(version)
    }

    /// Sets the Request Header Asynchronous Indicator for this exchange.
    #[must_use]
    pub fn with_asynchronous_indicator(self, value: u32) -> Self {
        Self {
            state: self.state.with_asynchronous_indicator(value),
        }
    }

    /// Validates the typed request without sending it.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError::InvalidRequest`] if typed validation fails.
    pub fn validate(&self) -> Result<(), FluentBuildError> {
        self.state.validate()
    }

    /// Builds the existing typed request without sending it.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError::InvalidRequest`] if typed validation fails.
    pub fn build_request(self) -> Result<RecertifyRequest, FluentBuildError> {
        validate_fluent_request(&self.state.request)?;
        Ok(self.state.request)
    }

    /// Converts this request to its typed batch item.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError::InvalidRequest`] if typed validation fails.
    pub fn to_batch_item(self) -> Result<ClientBatchItem, FluentBuildError> {
        self.build_request()
            .map(|request| ClientBatchItem::new(ClientRequest::Recertify(request)))
    }

    /// Executes one Re-certify exchange using configured [`CodecLimits`]
    /// and returns the unwrapped [`RecertifyResponse`].
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] when validation, transport, protocol, `ServerResult`, or `Pending` occurs.
    pub fn execute(self) -> Result<RecertifyResponse, ClientError> {
        let item = self.state.run_outcome()?;
        unwrap_typed_outcome(
            item,
            |outcome| match outcome {
                ClientBatchOutcome::Recertify(resp) => Some(resp),
                _ => None,
            },
            RecertifyResponse::result,
        )
    }

    /// Executes one Re-certify exchange using configured [`CodecLimits`]
    /// and returns the raw [`ClientBatchItemResponse`] envelope.
    ///
    /// # Errors
    ///
    /// Returns a validation error before sending or the existing client error.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        self.state.run_outcome()
    }

    /// Executes one Re-certify exchange with explicit [`CodecLimits`] overrides.
    ///
    /// # Errors
    ///
    /// Returns a validation error before sending or the existing client error for
    /// exchange and response failures.
    pub fn execute_outcome_with_limits(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        self.state.run(limits)
    }
}
