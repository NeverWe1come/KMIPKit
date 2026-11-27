//! Task-oriented, fluent builders over the existing typed KMIP requests.

use kmipkit_protocol::{
    CipherSpec, CryptographicParameters, DecryptRequest, DecryptResponse, EncryptRequest,
    EncryptResponse, HashAlgorithm, HashRequest, HashResponse, MacRequest, MacResponse, MacSpec,
    MacVerifyRequest, MacVerifyResponse, OperationData, ProtocolVersion, SecretBytes, SignRequest,
    SignResponse, SignatureSpec, SignatureVerifyRequest, SignatureVerifyResponse, UniqueIdentifier,
    ValidateKmipRequest,
};
use kmipkit_transport::RequestOptions;
use kmipkit_ttlv::Structure;

use crate::{
    Client, ClientBatchItem, ClientBatchItemResponse, ClientBatchOutcome, ClientError,
    ClientRequest,
};

use super::execution::{
    FluentBuildError, FluentBuilderExecution, fluent_client_validation_error,
    normalize_uid_for_version, validate_fluent_request,
};

// Holds a typed cryptographic request while consuming field transformations.
// Byte conversion and validation live here so each public facade follows the
// same ownership and execution rules without duplicating that logic.
struct DeferredUidCallback<R>(UniqueIdentifier, fn(R, UniqueIdentifier) -> R);

struct CryptoRequestState<'a, R> {
    execution: FluentBuilderExecution<'a>,
    request: R,
    deferred_uid: Option<DeferredUidCallback<R>>,
}

impl<'a, R: ValidateKmipRequest> CryptoRequestState<'a, R> {
    pub(super) fn new(client: &'a mut Client, request: R) -> Self {
        Self {
            execution: FluentBuilderExecution::new(client),
            request,
            deferred_uid: None,
        }
    }

    fn unique_identifier(
        mut self,
        value: UniqueIdentifier,
        setter: fn(R, UniqueIdentifier) -> R,
    ) -> Self {
        self.deferred_uid = Some(DeferredUidCallback(value, setter));
        self
    }

    fn field<V>(mut self, value: V, setter: fn(R, V) -> R) -> Self {
        self.request = setter(self.request, value);
        self
    }

    fn parameters(self, value: CryptographicParameters, setter: fn(R, Structure) -> R) -> Self {
        self.field(value.into_ttlv(), setter)
    }

    fn data(self, value: Vec<u8>, setter: fn(R, OperationData) -> R) -> Self {
        self.field(OperationData::ByteString(SecretBytes::new(value)), setter)
    }

    fn secret(self, value: Vec<u8>, setter: fn(R, SecretBytes) -> R) -> Self {
        self.field(SecretBytes::new(value), setter)
    }

    fn into_parts(mut self) -> Result<(R, FluentBuilderExecution<'a>), FluentBuildError> {
        let version = self.execution.protocol_version();
        if let Some(DeferredUidCallback(uid, setter)) = self.deferred_uid {
            self.request = setter(self.request, normalize_uid_for_version(uid, version));
        }
        validate_fluent_request(&self.request)?;
        Ok((self.request, self.execution))
    }
}

#[derive(Default)]
struct CryptoRequestFields {
    data: Option<OperationData>,
    cryptographic_parameters: Option<Structure>,
    iv_counter_nonce: Option<SecretBytes>,
    correlation_value: Option<SecretBytes>,
    init_indicator: Option<bool>,
    final_indicator: Option<bool>,
    authenticated_encryption_additional_data: Option<SecretBytes>,
}

trait CryptoFieldsOwner {
    fn crypto_fields_mut(&mut self) -> &mut CryptoRequestFields;
}

fn set_crypto_field<B, V>(mut builder: B, value: V, setter: fn(&mut CryptoRequestFields, V)) -> B
where
    B: CryptoFieldsOwner,
{
    setter(builder.crypto_fields_mut(), value);
    builder
}

impl CryptoRequestFields {
    fn set_data(&mut self, value: OperationData) {
        self.data = Some(value);
    }

    fn set_cryptographic_parameters(&mut self, value: Structure) {
        self.cryptographic_parameters = Some(value);
    }

    fn set_iv_counter_nonce(&mut self, value: Vec<u8>) {
        self.iv_counter_nonce = Some(SecretBytes::new(value));
    }

    fn set_correlation_value(&mut self, value: Vec<u8>) {
        self.correlation_value = Some(SecretBytes::new(value));
    }

    fn set_authenticated_encryption_additional_data(&mut self, value: Vec<u8>) {
        self.authenticated_encryption_additional_data = Some(SecretBytes::new(value));
    }
}

struct CipherRequestAdapter<R> {
    new: fn(Option<UniqueIdentifier>, Option<OperationData>) -> R,
    with_cryptographic_parameters: fn(R, Structure) -> R,
    with_iv_counter_nonce: fn(R, SecretBytes) -> R,
    with_correlation_value: fn(R, SecretBytes) -> R,
    with_init_indicator: fn(R, bool) -> R,
    with_final_indicator: fn(R, bool) -> R,
    with_authenticated_encryption_additional_data: fn(R, SecretBytes) -> R,
}

impl CryptoRequestFields {
    fn into_cipher_request<R>(
        self,
        unique_identifier: Option<UniqueIdentifier>,
        adapter: &CipherRequestAdapter<R>,
    ) -> R {
        let Self {
            data,
            cryptographic_parameters,
            iv_counter_nonce,
            correlation_value,
            init_indicator,
            final_indicator,
            authenticated_encryption_additional_data,
        } = self;
        let mut request = (adapter.new)(unique_identifier, data);
        if let Some(value) = cryptographic_parameters {
            request = (adapter.with_cryptographic_parameters)(request, value);
        }
        if let Some(value) = iv_counter_nonce {
            request = (adapter.with_iv_counter_nonce)(request, value);
        }
        if let Some(value) = correlation_value {
            request = (adapter.with_correlation_value)(request, value);
        }
        if let Some(value) = init_indicator {
            request = (adapter.with_init_indicator)(request, value);
        }
        if let Some(value) = final_indicator {
            request = (adapter.with_final_indicator)(request, value);
        }
        if let Some(value) = authenticated_encryption_additional_data {
            request = (adapter.with_authenticated_encryption_additional_data)(request, value);
        }
        request
    }
}

struct CipherBuilderCore {
    unique_identifier: Option<UniqueIdentifier>,
    fields: CryptoRequestFields,
}

impl CipherBuilderCore {
    pub(super) fn new() -> Self {
        Self {
            unique_identifier: None,
            fields: CryptoRequestFields::default(),
        }
    }

    fn set_data(&mut self, value: OperationData) {
        self.fields.set_data(value);
    }

    fn set_parameters(&mut self, value: Structure) {
        self.fields.set_cryptographic_parameters(value);
    }

    fn set_iv(&mut self, value: Vec<u8>) {
        self.fields.set_iv_counter_nonce(value);
    }

    fn set_correlation(&mut self, value: Vec<u8>) {
        self.fields.set_correlation_value(value);
    }

    const fn set_init_indicator(&mut self, value: bool) {
        self.fields.init_indicator = Some(value);
    }

    const fn set_final_indicator(&mut self, value: bool) {
        self.fields.final_indicator = Some(value);
    }

    fn set_aad(&mut self, value: Vec<u8>) {
        self.fields
            .set_authenticated_encryption_additional_data(value);
    }

    fn into_request<R>(self, adapter: &CipherRequestAdapter<R>, version: ProtocolVersion) -> R {
        self.fields.into_cipher_request(
            self.unique_identifier
                .map(|id| normalize_uid_for_version(id, version)),
            adapter,
        )
    }

    fn set_encrypt_identifier(&mut self, value: UniqueIdentifier) {
        self.unique_identifier = Some(value);
    }

    fn set_decrypt_identifier(&mut self, value: UniqueIdentifier) {
        self.unique_identifier = Some(value);
    }

    fn set_encrypt_data(&mut self, value: OperationData) {
        self.set_data(value);
    }

    fn set_decrypt_data(&mut self, value: OperationData) {
        self.set_data(value);
    }

    fn set_encrypt_parameters(&mut self, value: Structure) {
        self.set_parameters(value);
    }

    fn set_decrypt_parameters(&mut self, value: Structure) {
        self.set_parameters(value);
    }

    fn set_encrypt_iv(&mut self, value: Vec<u8>) {
        self.set_iv(value);
    }

    fn set_decrypt_iv(&mut self, value: Vec<u8>) {
        self.set_iv(value);
    }

    fn set_encrypt_correlation(&mut self, value: Vec<u8>) {
        self.set_correlation(value);
    }

    fn set_decrypt_correlation(&mut self, value: Vec<u8>) {
        self.set_correlation(value);
    }

    const fn set_encrypt_init_indicator(&mut self, value: bool) {
        self.set_init_indicator(value);
    }

    const fn set_decrypt_init_indicator(&mut self, value: bool) {
        self.set_init_indicator(value);
    }

    const fn set_encrypt_final_indicator(&mut self, value: bool) {
        self.set_final_indicator(value);
    }

    const fn set_decrypt_final_indicator(&mut self, value: bool) {
        self.set_final_indicator(value);
    }

    fn set_encrypt_aad(&mut self, value: Vec<u8>) {
        self.set_aad(value);
    }

    fn set_decrypt_aad(&mut self, value: Vec<u8>) {
        self.set_aad(value);
    }
}

/// A fluent builder for a typed KMIP Encrypt request.
pub struct EncryptBuilder<'a> {
    execution: FluentBuilderExecution<'a>,
    core: CipherBuilderCore,
}

impl<'a> EncryptBuilder<'a> {
    pub(super) fn new(client: &'a mut Client) -> Self {
        Self {
            execution: FluentBuilderExecution::new(client),
            core: CipherBuilderCore::new(),
        }
    }

    /// Sets the optional key identifier (`impl Into<UniqueIdentifier>`).
    #[must_use]
    pub fn unique_identifier(mut self, value: impl Into<UniqueIdentifier>) -> Self {
        self.core.set_encrypt_identifier(value.into());
        self
    }

    /// Moves plaintext bytes into the request's zeroizing secret owner.
    #[must_use]
    pub fn data(self, value: Vec<u8>) -> Self {
        self.data_value(OperationData::ByteString(SecretBytes::new(value)))
    }

    /// Sets an explicitly typed non-byte Data value for expert KMIP use.
    #[must_use]
    pub fn data_value(mut self, value: OperationData) -> Self {
        self.core.set_encrypt_data(value);
        self
    }

    /// Sets validated Cryptographic Parameters chosen by the caller.
    #[must_use]
    pub fn cryptographic_parameters(self, value: CryptographicParameters) -> Self {
        self.cryptographic_parameters_structure(value.into_ttlv())
    }

    /// Configures encryption parameters from a curated [`CipherSpec`] preset.
    #[must_use]
    pub fn with_cipher_spec(self, spec: CipherSpec) -> Self {
        if let Ok(params) = spec.to_cryptographic_parameters() {
            return self.cryptographic_parameters(params);
        }
        self
    }

    /// Sets an expert ordered Cryptographic Parameters Structure.
    #[must_use]
    pub fn cryptographic_parameters_structure(mut self, value: Structure) -> Self {
        self.core.set_encrypt_parameters(value);
        self
    }

    /// Sets the caller-selected IV, counter, or nonce bytes.
    #[must_use]
    pub fn iv_counter_nonce(mut self, value: Vec<u8>) -> Self {
        self.core.set_encrypt_iv(value);
        self
    }

    /// Sets the multipart correlation value.
    #[must_use]
    pub fn correlation_value(mut self, value: Vec<u8>) -> Self {
        self.core.set_encrypt_correlation(value);
        self
    }

    /// Sets the caller-controlled multipart Init Indicator.
    #[must_use]
    pub const fn init_indicator(mut self, value: bool) -> Self {
        self.core.set_encrypt_init_indicator(value);
        self
    }

    /// Sets the caller-controlled multipart Final Indicator.
    #[must_use]
    pub const fn final_indicator(mut self, value: bool) -> Self {
        self.core.set_encrypt_final_indicator(value);
        self
    }

    /// Sets caller-supplied authenticated-encryption additional data.
    #[must_use]
    pub fn authenticated_encryption_additional_data(mut self, value: Vec<u8>) -> Self {
        self.core.set_encrypt_aad(value);
        self
    }

    fluent_option_field_setters!(
        "Sets exchange-local options without changing client policy.";
        execution.options;
        execution.asynchronous_indicator;
        const
    );

    fn into_request_and_execution(
        self,
    ) -> Result<(EncryptRequest, FluentBuilderExecution<'a>), FluentBuildError> {
        let Self { execution, core } = self;
        let version = execution.protocol_version();
        let request = core.into_request(
            &CipherRequestAdapter {
                new: EncryptRequest::new,
                with_cryptographic_parameters: EncryptRequest::with_cryptographic_parameters,
                with_iv_counter_nonce: EncryptRequest::with_iv_counter_nonce,
                with_correlation_value: EncryptRequest::with_correlation_value,
                with_init_indicator: EncryptRequest::with_init_indicator,
                with_final_indicator: EncryptRequest::with_final_indicator,
                with_authenticated_encryption_additional_data:
                    EncryptRequest::with_authenticated_encryption_additional_data,
            },
            version,
        );
        validate_fluent_request(&request)?;
        Ok((request, execution))
    }

    /// Builds the existing typed Encrypt request without sending it.
    ///
    /// # Errors
    /// Returns an error if required fields are missing or the request is invalid.
    pub fn build(self) -> Result<EncryptRequest, FluentBuildError> {
        self.into_request_and_execution()
            .map(|(request, _)| request)
    }

    /// Converts the typed Encrypt request to a batch item.
    ///
    /// # Errors
    /// Returns an error if required fields are missing or the request is invalid.
    pub fn to_batch_item(self) -> Result<ClientBatchItem, FluentBuildError> {
        self.build()
            .map(|request| ClientBatchItem::new(ClientRequest::Encrypt(request)))
    }

    /// Executes one typed Encrypt exchange using configured [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits)
    /// and returns the unwrapped [`EncryptResponse`].
    ///
    /// # Errors
    /// Returns a validation, transport, protocol, `ServerResult`, or `Pending` error.
    pub fn execute(self) -> Result<EncryptResponse, ClientError> {
        let (request, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run_typed(
            ClientRequest::Encrypt(request),
            |outcome| match outcome {
                ClientBatchOutcome::Encrypt(resp) => Some(resp),
                _ => None,
            },
            EncryptResponse::result,
        )
    }

    /// Executes one typed Encrypt exchange using configured [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits)
    /// and returns the raw [`ClientBatchItemResponse`] envelope.
    ///
    /// # Errors
    /// Returns a validation, transport, or protocol error without retrying the exchange.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let (request, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run_outcome(ClientRequest::Encrypt(request))
    }

    /// Executes one typed Encrypt exchange with explicit [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits) overrides.
    ///
    /// # Errors
    /// Returns a validation, transport, or protocol error without retrying the exchange.
    pub fn execute_outcome_with_limits(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let (request, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run(ClientRequest::Encrypt(request), limits)
    }
}

/// A fluent builder for a typed KMIP Decrypt request.
pub struct DecryptBuilder<'a> {
    execution: FluentBuilderExecution<'a>,
    core: CipherBuilderCore,
    authenticated_encryption_tag: Option<SecretBytes>,
}

impl<'a> DecryptBuilder<'a> {
    pub(super) fn new(client: &'a mut Client) -> Self {
        Self {
            execution: FluentBuilderExecution::new(client),
            core: CipherBuilderCore::new(),
            authenticated_encryption_tag: None,
        }
    }

    /// Sets the optional key identifier (`impl Into<UniqueIdentifier>`).
    #[must_use]
    pub fn unique_identifier(mut self, value: impl Into<UniqueIdentifier>) -> Self {
        self.core.set_decrypt_identifier(value.into());
        self
    }

    /// Moves ciphertext bytes into the request's zeroizing secret owner.
    #[must_use]
    pub fn data(self, value: Vec<u8>) -> Self {
        self.data_value(OperationData::ByteString(SecretBytes::new(value)))
    }

    /// Sets an explicitly typed non-byte Data value for expert KMIP use.
    #[must_use]
    pub fn data_value(mut self, value: OperationData) -> Self {
        self.core.set_decrypt_data(value);
        self
    }

    /// Sets validated Cryptographic Parameters chosen by the caller.
    #[must_use]
    pub fn cryptographic_parameters(self, value: CryptographicParameters) -> Self {
        self.cryptographic_parameters_structure(value.into_ttlv())
    }

    /// Configures decryption parameters from a curated [`CipherSpec`] preset.
    #[must_use]
    pub fn with_cipher_spec(self, spec: CipherSpec) -> Self {
        if let Ok(params) = spec.to_cryptographic_parameters() {
            return self.cryptographic_parameters(params);
        }
        self
    }

    /// Sets an expert ordered Cryptographic Parameters Structure.
    #[must_use]
    pub fn cryptographic_parameters_structure(mut self, value: Structure) -> Self {
        self.core.set_decrypt_parameters(value);
        self
    }

    /// Sets the caller-selected IV, counter, or nonce bytes.
    #[must_use]
    pub fn iv_counter_nonce(mut self, value: Vec<u8>) -> Self {
        self.core.set_decrypt_iv(value);
        self
    }

    /// Sets the multipart correlation value.
    #[must_use]
    pub fn correlation_value(mut self, value: Vec<u8>) -> Self {
        self.core.set_decrypt_correlation(value);
        self
    }

    /// Sets the caller-controlled multipart Init Indicator.
    #[must_use]
    pub const fn init_indicator(mut self, value: bool) -> Self {
        self.core.set_decrypt_init_indicator(value);
        self
    }

    /// Sets the caller-controlled multipart Final Indicator.
    #[must_use]
    pub const fn final_indicator(mut self, value: bool) -> Self {
        self.core.set_decrypt_final_indicator(value);
        self
    }

    /// Sets caller-supplied authenticated-encryption additional data.
    #[must_use]
    pub fn authenticated_encryption_additional_data(mut self, value: Vec<u8>) -> Self {
        self.core.set_decrypt_aad(value);
        self
    }

    /// Sets caller-supplied authenticated-encryption tag bytes.
    #[must_use]
    pub fn authenticated_encryption_tag(mut self, value: Vec<u8>) -> Self {
        self.authenticated_encryption_tag = Some(SecretBytes::new(value));
        self
    }

    fluent_option_field_setters!(
        "Sets exchange-local options without changing client policy.";
        execution.options;
        execution.asynchronous_indicator;
        const
    );

    fn into_request_and_execution(
        self,
    ) -> Result<(DecryptRequest, FluentBuilderExecution<'a>), FluentBuildError> {
        let Self {
            execution,
            core,
            authenticated_encryption_tag,
        } = self;
        let version = execution.protocol_version();
        let mut request = core.into_request(
            &CipherRequestAdapter {
                new: DecryptRequest::new,
                with_cryptographic_parameters: DecryptRequest::with_cryptographic_parameters,
                with_iv_counter_nonce: DecryptRequest::with_iv_counter_nonce,
                with_correlation_value: DecryptRequest::with_correlation_value,
                with_init_indicator: DecryptRequest::with_init_indicator,
                with_final_indicator: DecryptRequest::with_final_indicator,
                with_authenticated_encryption_additional_data:
                    DecryptRequest::with_authenticated_encryption_additional_data,
            },
            version,
        );
        if let Some(value) = authenticated_encryption_tag {
            request = request.with_authenticated_encryption_tag(value);
        }
        validate_fluent_request(&request)?;
        Ok((request, execution))
    }

    /// Builds the existing typed Decrypt request without sending it.
    ///
    /// # Errors
    /// Returns an error if required fields are missing or the request is invalid.
    pub fn build(self) -> Result<DecryptRequest, FluentBuildError> {
        self.into_request_and_execution()
            .map(|(request, _)| request)
    }

    /// Converts the typed Decrypt request to a batch item.
    ///
    /// # Errors
    /// Returns an error if required fields are missing or the request is invalid.
    pub fn to_batch_item(self) -> Result<ClientBatchItem, FluentBuildError> {
        self.build()
            .map(|request| ClientBatchItem::new(ClientRequest::Decrypt(request)))
    }

    /// Executes one typed Decrypt exchange using configured [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits)
    /// and returns the unwrapped [`DecryptResponse`].
    ///
    /// # Errors
    /// Returns a validation, transport, protocol, `ServerResult`, or `Pending` error.
    pub fn execute(self) -> Result<DecryptResponse, ClientError> {
        let (request, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run_typed(
            ClientRequest::Decrypt(request),
            |outcome| match outcome {
                ClientBatchOutcome::Decrypt(resp) => Some(resp),
                _ => None,
            },
            DecryptResponse::result,
        )
    }

    /// Executes one typed Decrypt exchange using configured [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits)
    /// and returns the raw [`ClientBatchItemResponse`] envelope.
    ///
    /// # Errors
    /// Returns a validation, transport, or protocol error without retrying the exchange.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let (request, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run_outcome(ClientRequest::Decrypt(request))
    }

    /// Executes one typed Decrypt exchange with explicit [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits) overrides.
    ///
    /// # Errors
    /// Returns a validation, transport, or protocol error without retrying the exchange.
    pub fn execute_outcome_with_limits(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let (request, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run(ClientRequest::Decrypt(request), limits)
    }
}

/// A fluent builder for a typed KMIP Sign request.
pub struct SignBuilder<'a> {
    state: CryptoRequestState<'a, SignRequest>,
}

impl<'a> SignBuilder<'a> {
    pub(super) fn new(client: &'a mut Client) -> Self {
        Self {
            state: CryptoRequestState::new(client, SignRequest::new()),
        }
    }

    /// Sets the optional signing key identifier (`impl Into<UniqueIdentifier>`).
    #[must_use]
    pub fn unique_identifier(mut self, value: impl Into<UniqueIdentifier>) -> Self {
        self.state = self
            .state
            .unique_identifier(value.into(), SignRequest::with_unique_identifier);
        self
    }

    /// Sets caller-selected typed Cryptographic Parameters.
    #[must_use]
    pub fn cryptographic_parameters(mut self, value: CryptographicParameters) -> Self {
        self.state = self
            .state
            .parameters(value, SignRequest::with_cryptographic_parameters);
        self
    }

    /// Configures signature parameters from a curated [`SignatureSpec`] preset.
    #[must_use]
    pub fn with_signature_spec(self, spec: SignatureSpec) -> Self {
        if let Ok(params) = spec.to_cryptographic_parameters() {
            return self.cryptographic_parameters(params);
        }
        self
    }

    /// Sets an expert ordered Cryptographic Parameters Structure.
    #[must_use]
    pub fn cryptographic_parameters_structure(mut self, value: Structure) -> Self {
        self.state = self
            .state
            .field(value, SignRequest::with_cryptographic_parameters);
        self
    }

    /// Moves message bytes into the request's zeroizing secret owner.
    #[must_use]
    pub fn data(mut self, value: Vec<u8>) -> Self {
        self.state = self.state.data(value, SignRequest::with_data);
        self
    }

    /// Sets an explicitly typed non-byte Data value for expert KMIP use.
    #[must_use]
    pub fn data_value(mut self, value: OperationData) -> Self {
        self.state = self.state.field(value, SignRequest::with_data);
        self
    }

    /// Moves caller-supplied precomputed digest bytes into zeroizing storage.
    #[must_use]
    pub fn digested_data(mut self, value: Vec<u8>) -> Self {
        self.state = self.state.secret(value, SignRequest::with_digested_data);
        self
    }

    /// Sets the multipart correlation value.
    #[must_use]
    pub fn correlation_value(mut self, value: Vec<u8>) -> Self {
        self.state = self
            .state
            .secret(value, SignRequest::with_correlation_value);
        self
    }

    /// Sets the caller-controlled multipart Init Indicator.
    #[must_use]
    pub fn init_indicator(mut self, value: bool) -> Self {
        self.state = self.state.field(value, SignRequest::with_init_indicator);
        self
    }

    /// Sets the caller-controlled multipart Final Indicator.
    #[must_use]
    pub fn final_indicator(mut self, value: bool) -> Self {
        self.state = self.state.field(value, SignRequest::with_final_indicator);
        self
    }

    /// Sets exchange-local transport options.
    #[must_use]
    pub fn with_options(mut self, options: RequestOptions) -> Self {
        self.state.execution = self.state.execution.with_options(options);
        self
    }

    /// Sets the KMIP protocol version override for this request/exchange.
    #[must_use]
    pub fn with_protocol_version(mut self, version: ProtocolVersion) -> Self {
        self.state.execution = self.state.execution.with_protocol_version(version);
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
        self.state.execution = self.state.execution.with_asynchronous_indicator(value);
        self
    }

    fn into_request_and_execution(
        self,
    ) -> Result<(SignRequest, FluentBuilderExecution<'a>), FluentBuildError> {
        self.state.into_parts()
    }

    /// Builds the existing typed Sign request without sending it.
    ///
    /// # Errors
    /// Returns an error if required fields are missing or the request is invalid.
    pub fn build(self) -> Result<SignRequest, FluentBuildError> {
        self.into_request_and_execution()
            .map(|(request, _)| request)
    }

    /// Converts the typed Sign request to a batch item.
    ///
    /// # Errors
    /// Returns an error if required fields are missing or the request is invalid.
    pub fn to_batch_item(self) -> Result<ClientBatchItem, FluentBuildError> {
        self.build()
            .map(|request| ClientBatchItem::new(ClientRequest::Sign(request)))
    }

    /// Executes one typed Sign exchange using configured [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits)
    /// and returns the unwrapped [`SignResponse`].
    ///
    /// # Errors
    /// Returns a validation, transport, protocol, `ServerResult`, or `Pending` error.
    pub fn execute(self) -> Result<SignResponse, ClientError> {
        let (request, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run_typed(
            ClientRequest::Sign(request),
            |outcome| match outcome {
                ClientBatchOutcome::Sign(resp) => Some(resp),
                _ => None,
            },
            SignResponse::result,
        )
    }

    /// Executes one typed Sign exchange using configured [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits)
    /// and returns the raw [`ClientBatchItemResponse`] envelope.
    ///
    /// # Errors
    /// Returns a validation, transport, or protocol error without retrying the exchange.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let (request, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run_outcome(ClientRequest::Sign(request))
    }

    /// Executes one typed Sign exchange with explicit [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits) overrides.
    ///
    /// # Errors
    /// Returns a validation, transport, or protocol error without retrying the exchange.
    pub fn execute_outcome_with_limits(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let (request, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run(ClientRequest::Sign(request), limits)
    }
}

/// A fluent builder for a typed KMIP Signature Verify request.
pub struct SignatureVerifyBuilder<'a> {
    state: CryptoRequestState<'a, SignatureVerifyRequest>,
}

impl<'a> SignatureVerifyBuilder<'a> {
    pub(super) fn new(client: &'a mut Client) -> Self {
        Self {
            state: CryptoRequestState::new(client, SignatureVerifyRequest::new()),
        }
    }

    /// Sets the optional verification key identifier (`impl Into<UniqueIdentifier>`).
    #[must_use]
    pub fn unique_identifier(mut self, value: impl Into<UniqueIdentifier>) -> Self {
        self.state = self
            .state
            .unique_identifier(value.into(), SignatureVerifyRequest::with_unique_identifier);
        self
    }

    /// Sets caller-selected typed Cryptographic Parameters.
    #[must_use]
    pub fn cryptographic_parameters(mut self, value: CryptographicParameters) -> Self {
        self.state = self
            .state
            .parameters(value, SignatureVerifyRequest::with_cryptographic_parameters);
        self
    }

    /// Configures verification parameters from a curated [`SignatureSpec`] preset.
    #[must_use]
    pub fn with_signature_spec(self, spec: SignatureSpec) -> Self {
        if let Ok(params) = spec.to_cryptographic_parameters() {
            return self.cryptographic_parameters(params);
        }
        self
    }

    /// Sets an expert ordered Cryptographic Parameters Structure.
    #[must_use]
    pub fn cryptographic_parameters_structure(mut self, value: Structure) -> Self {
        self.state = self
            .state
            .field(value, SignatureVerifyRequest::with_cryptographic_parameters);
        self
    }

    /// Moves message bytes into the request's zeroizing secret owner.
    #[must_use]
    pub fn data(mut self, value: Vec<u8>) -> Self {
        self.state = self.state.data(value, SignatureVerifyRequest::with_data);
        self
    }

    /// Sets an explicitly typed non-byte Data value for expert KMIP use.
    #[must_use]
    pub fn data_value(mut self, value: OperationData) -> Self {
        self.state = self.state.field(value, SignatureVerifyRequest::with_data);
        self
    }

    /// Moves caller-supplied precomputed digest bytes into zeroizing storage.
    #[must_use]
    pub fn digested_data(mut self, value: Vec<u8>) -> Self {
        self.state = self
            .state
            .secret(value, SignatureVerifyRequest::with_digested_data);
        self
    }

    /// Moves signature bytes into the request's zeroizing secret owner.
    #[must_use]
    pub fn signature_data(mut self, value: Vec<u8>) -> Self {
        self.state = self
            .state
            .secret(value, SignatureVerifyRequest::with_signature_data);
        self
    }

    /// Sets the multipart correlation value.
    #[must_use]
    pub fn correlation_value(mut self, value: Vec<u8>) -> Self {
        self.state = self
            .state
            .secret(value, SignatureVerifyRequest::with_correlation_value);
        self
    }

    /// Sets the caller-controlled multipart Init Indicator.
    #[must_use]
    pub fn init_indicator(mut self, value: bool) -> Self {
        self.state = self
            .state
            .field(value, SignatureVerifyRequest::with_init_indicator);
        self
    }

    /// Sets the caller-controlled multipart Final Indicator.
    #[must_use]
    pub fn final_indicator(mut self, value: bool) -> Self {
        self.state = self
            .state
            .field(value, SignatureVerifyRequest::with_final_indicator);
        self
    }

    /// Sets exchange-local transport options.
    #[must_use]
    pub fn with_options(mut self, options: RequestOptions) -> Self {
        self.state.execution = self.state.execution.with_options(options);
        self
    }

    /// Sets the KMIP protocol version override for this request/exchange.
    #[must_use]
    pub fn with_protocol_version(mut self, version: ProtocolVersion) -> Self {
        self.state.execution = self.state.execution.with_protocol_version(version);
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
        self.state.execution = self.state.execution.with_asynchronous_indicator(value);
        self
    }

    fn into_request_and_execution(
        self,
    ) -> Result<(SignatureVerifyRequest, FluentBuilderExecution<'a>), FluentBuildError> {
        self.state.into_parts()
    }

    /// Builds the existing typed Signature Verify request without sending it.
    ///
    /// # Errors
    /// Returns an error if required fields are missing or the request is invalid.
    pub fn build(self) -> Result<SignatureVerifyRequest, FluentBuildError> {
        self.into_request_and_execution()
            .map(|(request, _)| request)
    }

    /// Converts the typed Signature Verify request to a batch item.
    ///
    /// # Errors
    /// Returns an error if required fields are missing or the request is invalid.
    pub fn to_batch_item(self) -> Result<ClientBatchItem, FluentBuildError> {
        self.build()
            .map(|request| ClientBatchItem::new(ClientRequest::SignatureVerify(request)))
    }

    /// Executes one typed Signature Verify exchange using configured [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits)
    /// and returns the unwrapped [`SignatureVerifyResponse`].
    ///
    /// # Errors
    /// Returns a validation, transport, protocol, `ServerResult`, or `Pending` error.
    pub fn execute(self) -> Result<SignatureVerifyResponse, ClientError> {
        let (request, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run_typed(
            ClientRequest::SignatureVerify(request),
            |outcome| match outcome {
                ClientBatchOutcome::SignatureVerify(resp) => Some(resp),
                _ => None,
            },
            SignatureVerifyResponse::result,
        )
    }

    /// Executes one typed Signature Verify exchange using configured [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits)
    /// and returns the raw [`ClientBatchItemResponse`] envelope.
    ///
    /// # Errors
    /// Returns a validation, transport, or protocol error without retrying the exchange.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let (request, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run_outcome(ClientRequest::SignatureVerify(request))
    }

    /// Executes one typed Signature Verify exchange with explicit [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits) overrides.
    ///
    /// # Errors
    /// Returns a validation, transport, or protocol error without retrying the exchange.
    pub fn execute_outcome_with_limits(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let (request, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run(ClientRequest::SignatureVerify(request), limits)
    }
}

/// A fluent builder for a typed KMIP MAC request.
pub struct MacBuilder<'a> {
    state: CryptoRequestState<'a, MacRequest>,
}

impl<'a> MacBuilder<'a> {
    pub(super) fn new(client: &'a mut Client) -> Self {
        Self {
            state: CryptoRequestState::new(client, MacRequest::new()),
        }
    }

    /// Sets the optional MAC key identifier (`impl Into<UniqueIdentifier>`).
    #[must_use]
    pub fn unique_identifier(mut self, value: impl Into<UniqueIdentifier>) -> Self {
        self.state = self
            .state
            .unique_identifier(value.into(), MacRequest::with_unique_identifier);
        self
    }

    /// Sets caller-selected typed Cryptographic Parameters.
    #[must_use]
    pub fn cryptographic_parameters(mut self, value: CryptographicParameters) -> Self {
        self.state = self
            .state
            .parameters(value, MacRequest::with_cryptographic_parameters);
        self
    }

    /// Configures MAC parameters from a curated [`MacSpec`] preset.
    #[must_use]
    pub fn with_mac_spec(self, spec: MacSpec) -> Self {
        if let Ok(params) = spec.to_cryptographic_parameters() {
            return self.cryptographic_parameters(params);
        }
        self
    }

    /// Sets an expert ordered Cryptographic Parameters Structure.
    #[must_use]
    pub fn cryptographic_parameters_structure(mut self, value: Structure) -> Self {
        self.state = self
            .state
            .field(value, MacRequest::with_cryptographic_parameters);
        self
    }

    /// Moves message bytes into the request's zeroizing secret owner.
    #[must_use]
    pub fn data(mut self, value: Vec<u8>) -> Self {
        self.state = self.state.data(value, MacRequest::with_data);
        self
    }

    /// Sets an explicitly typed non-byte Data value for expert KMIP use.
    #[must_use]
    pub fn data_value(mut self, value: OperationData) -> Self {
        self.state = self.state.field(value, MacRequest::with_data);
        self
    }

    /// Sets the multipart correlation value.
    #[must_use]
    pub fn correlation_value(mut self, value: Vec<u8>) -> Self {
        self.state = self.state.secret(value, MacRequest::with_correlation_value);
        self
    }

    /// Sets the caller-controlled multipart Init Indicator.
    #[must_use]
    pub fn init_indicator(mut self, value: bool) -> Self {
        self.state = self.state.field(value, MacRequest::with_init_indicator);
        self
    }

    /// Sets the caller-controlled multipart Final Indicator.
    #[must_use]
    pub fn final_indicator(mut self, value: bool) -> Self {
        self.state = self.state.field(value, MacRequest::with_final_indicator);
        self
    }

    /// Sets exchange-local transport options.
    #[must_use]
    pub fn with_options(mut self, options: RequestOptions) -> Self {
        self.state.execution = self.state.execution.with_options(options);
        self
    }

    /// Sets the KMIP protocol version override for this request/exchange.
    #[must_use]
    pub fn with_protocol_version(mut self, version: ProtocolVersion) -> Self {
        self.state.execution = self.state.execution.with_protocol_version(version);
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
        self.state.execution = self.state.execution.with_asynchronous_indicator(value);
        self
    }

    fn into_request_and_execution(
        self,
    ) -> Result<(MacRequest, FluentBuilderExecution<'a>), FluentBuildError> {
        self.state.into_parts()
    }

    /// Builds the existing typed MAC request without sending it.
    ///
    /// # Errors
    /// Returns an error if required fields are missing or the request is invalid.
    pub fn build(self) -> Result<MacRequest, FluentBuildError> {
        self.into_request_and_execution()
            .map(|(request, _)| request)
    }

    /// Converts the typed MAC request to a batch item.
    ///
    /// # Errors
    /// Returns an error if required fields are missing or the request is invalid.
    pub fn to_batch_item(self) -> Result<ClientBatchItem, FluentBuildError> {
        self.build()
            .map(|request| ClientBatchItem::new(ClientRequest::Mac(request)))
    }

    /// Executes one typed MAC exchange using configured [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits)
    /// and returns the unwrapped [`MacResponse`].
    ///
    /// # Errors
    /// Returns a validation, transport, protocol, `ServerResult`, or `Pending` error.
    pub fn execute(self) -> Result<MacResponse, ClientError> {
        let (request, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run_typed(
            ClientRequest::Mac(request),
            |outcome| match outcome {
                ClientBatchOutcome::Mac(resp) => Some(resp),
                _ => None,
            },
            MacResponse::result,
        )
    }

    /// Executes one typed MAC exchange using configured [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits)
    /// and returns the raw [`ClientBatchItemResponse`] envelope.
    ///
    /// # Errors
    /// Returns a validation, transport, or protocol error without retrying the exchange.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let (request, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run_outcome(ClientRequest::Mac(request))
    }

    /// Executes one typed MAC exchange with explicit [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits) overrides.
    ///
    /// # Errors
    /// Returns a validation, transport, or protocol error without retrying the exchange.
    pub fn execute_outcome_with_limits(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let (request, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run(ClientRequest::Mac(request), limits)
    }
}

/// A fluent builder for a typed KMIP MAC Verify request.
pub struct MacVerifyBuilder<'a> {
    state: CryptoRequestState<'a, MacVerifyRequest>,
}

impl<'a> MacVerifyBuilder<'a> {
    pub(super) fn new(client: &'a mut Client) -> Self {
        Self {
            state: CryptoRequestState::new(client, MacVerifyRequest::new()),
        }
    }

    /// Sets the optional MAC verification key identifier (`impl Into<UniqueIdentifier>`).
    #[must_use]
    pub fn unique_identifier(mut self, value: impl Into<UniqueIdentifier>) -> Self {
        self.state = self
            .state
            .unique_identifier(value.into(), MacVerifyRequest::with_unique_identifier);
        self
    }

    /// Sets caller-selected typed Cryptographic Parameters.
    #[must_use]
    pub fn cryptographic_parameters(mut self, value: CryptographicParameters) -> Self {
        self.state = self
            .state
            .parameters(value, MacVerifyRequest::with_cryptographic_parameters);
        self
    }

    /// Configures MAC verification parameters from a curated [`MacSpec`] preset.
    #[must_use]
    pub fn with_mac_spec(self, spec: MacSpec) -> Self {
        if let Ok(params) = spec.to_cryptographic_parameters() {
            return self.cryptographic_parameters(params);
        }
        self
    }

    /// Sets an expert ordered Cryptographic Parameters Structure.
    #[must_use]
    pub fn cryptographic_parameters_structure(mut self, value: Structure) -> Self {
        self.state = self
            .state
            .field(value, MacVerifyRequest::with_cryptographic_parameters);
        self
    }

    /// Moves message bytes into the request's zeroizing secret owner.
    #[must_use]
    pub fn data(mut self, value: Vec<u8>) -> Self {
        self.state = self.state.data(value, MacVerifyRequest::with_data);
        self
    }

    /// Sets an explicitly typed non-byte Data value for expert KMIP use.
    #[must_use]
    pub fn data_value(mut self, value: OperationData) -> Self {
        self.state = self.state.field(value, MacVerifyRequest::with_data);
        self
    }

    /// Moves expected MAC bytes into the request's zeroizing secret owner.
    #[must_use]
    pub fn mac_data(mut self, value: Vec<u8>) -> Self {
        self.state = self.state.secret(value, MacVerifyRequest::with_mac_data);
        self
    }

    /// Sets the multipart correlation value.
    #[must_use]
    pub fn correlation_value(mut self, value: Vec<u8>) -> Self {
        self.state = self
            .state
            .secret(value, MacVerifyRequest::with_correlation_value);
        self
    }

    /// Sets the caller-controlled multipart Init Indicator.
    #[must_use]
    pub fn init_indicator(mut self, value: bool) -> Self {
        self.state = self
            .state
            .field(value, MacVerifyRequest::with_init_indicator);
        self
    }

    /// Sets the caller-controlled multipart Final Indicator.
    #[must_use]
    pub fn final_indicator(mut self, value: bool) -> Self {
        self.state = self
            .state
            .field(value, MacVerifyRequest::with_final_indicator);
        self
    }

    /// Sets exchange-local transport options.
    #[must_use]
    pub fn with_options(mut self, options: RequestOptions) -> Self {
        self.state.execution = self.state.execution.with_options(options);
        self
    }

    /// Sets the KMIP protocol version override for this request/exchange.
    #[must_use]
    pub fn with_protocol_version(mut self, version: ProtocolVersion) -> Self {
        self.state.execution = self.state.execution.with_protocol_version(version);
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
        self.state.execution = self.state.execution.with_asynchronous_indicator(value);
        self
    }

    fn into_request_and_execution(
        self,
    ) -> Result<(MacVerifyRequest, FluentBuilderExecution<'a>), FluentBuildError> {
        self.state.into_parts()
    }

    /// Builds the existing typed MAC Verify request without sending it.
    ///
    /// # Errors
    /// Returns an error if required fields are missing or the request is invalid.
    pub fn build(self) -> Result<MacVerifyRequest, FluentBuildError> {
        self.into_request_and_execution()
            .map(|(request, _)| request)
    }

    /// Converts the typed MAC Verify request to a batch item.
    ///
    /// # Errors
    /// Returns an error if required fields are missing or the request is invalid.
    pub fn to_batch_item(self) -> Result<ClientBatchItem, FluentBuildError> {
        self.build()
            .map(|request| ClientBatchItem::new(ClientRequest::MacVerify(request)))
    }

    /// Executes one typed MAC Verify exchange using configured [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits)
    /// and returns the unwrapped [`MacVerifyResponse`].
    ///
    /// # Errors
    /// Returns a validation, transport, protocol, `ServerResult`, or `Pending` error.
    pub fn execute(self) -> Result<MacVerifyResponse, ClientError> {
        let (request, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run_typed(
            ClientRequest::MacVerify(request),
            |outcome| match outcome {
                ClientBatchOutcome::MacVerify(resp) => Some(resp),
                _ => None,
            },
            MacVerifyResponse::result,
        )
    }

    /// Executes one typed MAC Verify exchange using configured [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits)
    /// and returns the raw [`ClientBatchItemResponse`] envelope.
    ///
    /// # Errors
    /// Returns a validation, transport, or protocol error without retrying the exchange.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let (request, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run_outcome(ClientRequest::MacVerify(request))
    }

    /// Executes one typed MAC Verify exchange with explicit [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits) overrides.
    ///
    /// # Errors
    /// Returns a validation, transport, or protocol error without retrying the exchange.
    pub fn execute_outcome_with_limits(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let (request, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run(ClientRequest::MacVerify(request), limits)
    }
}

/// A fluent builder for a typed KMIP Hash request.
pub struct HashBuilder<'a> {
    execution: FluentBuilderExecution<'a>,
    fields: CryptoRequestFields,
}

impl CryptoFieldsOwner for HashBuilder<'_> {
    fn crypto_fields_mut(&mut self) -> &mut CryptoRequestFields {
        &mut self.fields
    }
}

impl<'a> HashBuilder<'a> {
    pub(super) fn new(client: &'a mut Client) -> Self {
        Self {
            execution: FluentBuilderExecution::new(client),
            fields: CryptoRequestFields::default(),
        }
    }

    /// Sets caller-selected typed Cryptographic Parameters.
    #[must_use]
    pub fn cryptographic_parameters(self, value: CryptographicParameters) -> Self {
        self.cryptographic_parameters_structure(value.into_ttlv())
    }

    /// Configures hash parameters from a curated [`HashAlgorithm`] preset.
    #[must_use]
    pub fn with_hash_algorithm(self, hash: HashAlgorithm) -> Self {
        if let Ok(params) = hash.to_cryptographic_parameters() {
            return self.cryptographic_parameters(params);
        }
        self
    }

    /// Sets an expert ordered Cryptographic Parameters Structure.
    #[must_use]
    pub fn cryptographic_parameters_structure(self, value: Structure) -> Self {
        set_crypto_field(
            self,
            value,
            CryptoRequestFields::set_cryptographic_parameters,
        )
    }

    /// Moves message bytes into the request's zeroizing secret owner.
    #[must_use]
    pub fn data(self, value: Vec<u8>) -> Self {
        self.data_value(OperationData::ByteString(SecretBytes::new(value)))
    }

    /// Sets an explicitly typed non-byte Data value for expert KMIP use.
    #[must_use]
    pub fn data_value(self, value: OperationData) -> Self {
        set_crypto_field(self, value, CryptoRequestFields::set_data)
    }

    /// Sets the multipart correlation value.
    #[must_use]
    pub fn correlation_value(self, value: Vec<u8>) -> Self {
        set_crypto_field(self, value, CryptoRequestFields::set_correlation_value)
    }

    /// Sets the caller-controlled multipart Init Indicator.
    #[must_use]
    pub const fn init_indicator(mut self, value: bool) -> Self {
        self.fields.init_indicator = Some(value);
        self
    }

    /// Sets the caller-controlled multipart Final Indicator.
    #[must_use]
    pub const fn final_indicator(mut self, value: bool) -> Self {
        self.fields.final_indicator = Some(value);
        self
    }

    fluent_option_field_setters!(
        "Sets exchange-local options without changing client policy.";
        execution.options;
        execution.asynchronous_indicator;
        const
    );

    fn into_request_and_execution(
        self,
    ) -> Result<(HashRequest, FluentBuilderExecution<'a>), FluentBuildError> {
        let Self { execution, fields } = self;
        let CryptoRequestFields {
            data,
            cryptographic_parameters,
            correlation_value,
            init_indicator,
            final_indicator,
            ..
        } = fields;
        let Some(parameters) = cryptographic_parameters else {
            return Err(FluentBuildError::MissingHashCryptographicParameters);
        };
        let mut request = HashRequest::new(parameters);
        if let Some(value) = data {
            request = request.with_data(value);
        }
        if let Some(value) = correlation_value {
            request = request.with_correlation_value(value);
        }
        if let Some(value) = init_indicator {
            request = request.with_init_indicator(value);
        }
        if let Some(value) = final_indicator {
            request = request.with_final_indicator(value);
        }
        validate_fluent_request(&request)?;
        Ok((request, execution))
    }

    /// Builds the existing typed Hash request without sending it.
    ///
    /// # Errors
    /// Returns an error if required fields are missing or the request is invalid.
    pub fn build(self) -> Result<HashRequest, FluentBuildError> {
        self.into_request_and_execution()
            .map(|(request, _)| request)
    }

    /// Converts the typed Hash request to a batch item.
    ///
    /// # Errors
    /// Returns an error if required fields are missing or the request is invalid.
    pub fn to_batch_item(self) -> Result<ClientBatchItem, FluentBuildError> {
        self.build()
            .map(|request| ClientBatchItem::new(ClientRequest::Hash(request)))
    }

    /// Executes one typed Hash exchange using configured [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits)
    /// and returns the unwrapped [`HashResponse`].
    ///
    /// # Errors
    /// Returns a validation, transport, protocol, `ServerResult`, or `Pending` error.
    pub fn execute(self) -> Result<HashResponse, ClientError> {
        let (request, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run_typed(
            ClientRequest::Hash(request),
            |outcome| match outcome {
                ClientBatchOutcome::Hash(resp) => Some(resp),
                _ => None,
            },
            HashResponse::result,
        )
    }

    /// Executes one typed Hash exchange using configured [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits)
    /// and returns the raw [`ClientBatchItemResponse`] envelope.
    ///
    /// # Errors
    /// Returns a validation, transport, or protocol error without retrying the exchange.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let (request, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run_outcome(ClientRequest::Hash(request))
    }

    /// Executes one typed Hash exchange with explicit [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits) overrides.
    ///
    /// # Errors
    /// Returns a validation, transport, or protocol error without retrying the exchange.
    pub fn execute_outcome_with_limits(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let (request, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run(ClientRequest::Hash(request), limits)
    }
}
