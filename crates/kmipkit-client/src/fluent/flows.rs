//! Phase 4 advanced fluent flows: batching, composite workflows, multipart streaming,
//! pending operation tracking, paginated Locate, and closure-based Server-to-Client handlers.

use kmipkit_protocol::{
    AttributeSet, Authentication, CipherSpec, CreateKeyPairResponse, CreateResponse,
    DecryptResponse, DiscoverVersionsResponse, EncryptResponse, HashAlgorithm, HashResponse,
    KmipOperationResult, LocateResponse, MacResponse, MacSpec, ProtocolVersion, RevocationReason,
    SecretBytes, SignResponse, SignatureSpec, UniqueIdentifier,
};
use kmipkit_transport::RequestOptions;

use crate::{
    Client, ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientBatchResponse, ClientError,
    ClientRequest,
};

use super::execution::{
    FluentBuildError, execute_fluent_outcome, fluent_client_validation_error, unwrap_typed_outcome,
};
use super::{
    ActivateBuilder, CancelBuilder, CreateKeyPairBuilder, CreateSymmetricKeyBuilder,
    CryptographicUsageMask, DecryptBuilder, DestroyBuilder, DiscoverVersionsBuilder,
    EncryptBuilder, HashBuilder, KeySpec, LocateBuilder, MacBuilder, PollBuilder, ProcessBuilder,
    RevokeBuilder, SignBuilder,
};

fn uid_to_string(uid: &UniqueIdentifier) -> Result<String, ClientError> {
    match uid {
        UniqueIdentifier::TextString(s) | UniqueIdentifier::Identifier(s) => Ok(s.clone()),
        _ => Err(fluent_client_validation_error(
            FluentBuildError::InvalidRequest,
        )),
    }
}

fn clone_secret(bytes: &SecretBytes) -> SecretBytes {
    bytes.with_bytes(|slice| SecretBytes::new(slice.to_vec()))
}

fn exec_typed<T>(
    client: &mut Client,
    request: ClientRequest,
    extract: impl FnOnce(ClientBatchOutcome) -> Option<T>,
    result_of: impl FnOnce(&T) -> &KmipOperationResult,
) -> Result<T, ClientError> {
    let outcome = execute_fluent_outcome(client, request, None, None)?;
    unwrap_typed_outcome(outcome, extract, result_of)
}

fn exec_kmi_result(
    client: &mut Client,
    request: ClientRequest,
    extract: impl FnOnce(ClientBatchOutcome) -> Option<KmipOperationResult>,
) -> Result<KmipOperationResult, ClientError> {
    let outcome = execute_fluent_outcome(client, request, None, None)?;
    let result = extract(outcome.into_outcome())
        .ok_or_else(|| fluent_client_validation_error(FluentBuildError::InvalidRequest))?;
    if result.status().raw() == 0 {
        Ok(result)
    } else {
        Err(ClientError::server_result(result))
    }
}

// ---------------------------------------------------------------------------
// 1. FluentBatchBuilder (client.batch())
// ---------------------------------------------------------------------------

/// Fluent builder for multi-operation KMIP request batches with ID Placeholder and ordering support.
pub struct FluentBatchBuilder<'a> {
    client: &'a mut Client,
    batch: ClientBatch,
    options: Option<RequestOptions>,
    build_error: Option<FluentBuildError>,
}

impl<'a> FluentBatchBuilder<'a> {
    pub(crate) fn new(client: &'a mut Client) -> Self {
        Self {
            client,
            batch: ClientBatch::empty(),
            options: None,
            build_error: None,
        }
    }

    /// Appends a prebuilt [`ClientBatchItem`] in wire order.
    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn add(mut self, item: ClientBatchItem) -> Self {
        self.batch.push(item);
        self
    }

    /// Appends the result of `builder.to_batch_item()`, preserving any local build error.
    #[must_use]
    pub fn try_add(mut self, item: Result<ClientBatchItem, FluentBuildError>) -> Self {
        match item {
            Ok(built) => self.batch.push(built),
            Err(err) => {
                self.build_error.get_or_insert(err);
            }
        }
        self
    }

    /// Sets `Batch Order Option` (`true` requires strict sequential execution, e.g. for ID Placeholder).
    #[must_use]
    pub fn batch_order(mut self, ordered: bool) -> Self {
        self.batch = self.batch.with_batch_order_option(ordered);
        self
    }

    /// Convenience alias for `.batch_order(true)`.
    #[must_use]
    pub fn ordered(self) -> Self {
        self.batch_order(true)
    }

    /// Sets per-batch `Authentication` in the Request Header.
    #[must_use]
    pub fn authentication(mut self, authentication: Authentication) -> Self {
        self.batch = self.batch.with_authentication(authentication);
        self
    }

    /// Sets `Batch Error Continuation Option` raw Enumeration value.
    #[must_use]
    pub fn error_continuation(mut self, value: u32) -> Self {
        self.batch = self.batch.with_batch_error_continuation_option(value);
        self
    }

    /// Sets the Request Header `Asynchronous Indicator`.
    #[must_use]
    pub fn asynchronous_indicator(mut self, value: u32) -> Self {
        self.batch = self.batch.with_asynchronous_indicator(value);
        self
    }

    /// Overrides the KMIP protocol version for this batch.
    #[must_use]
    pub fn with_protocol_version(mut self, version: ProtocolVersion) -> Self {
        self.batch = self.batch.with_protocol_version(version);
        self
    }

    /// Sets exchange-local transport options.
    #[must_use]
    pub fn with_options(mut self, options: RequestOptions) -> Self {
        self.options = Some(options);
        self
    }

    /// Builds the typed [`ClientBatch`] without executing it.
    ///
    /// # Errors
    /// Returns [`FluentBuildError`] if any queued `try_add` item failed validation.
    pub fn build(self) -> Result<ClientBatch, FluentBuildError> {
        if let Some(err) = self.build_error {
            return Err(err);
        }
        Ok(self.batch)
    }

    /// Executes the batch synchronously against the configured KMIP server.
    ///
    /// # Errors
    /// Returns [`ClientError`] on build failure, validation, transport, or protocol error.
    pub fn execute(self) -> Result<ClientBatchResponse, ClientError> {
        if let Some(err) = self.build_error {
            return Err(fluent_client_validation_error(err));
        }
        match self.options {
            Some(opts) => self.client.send_with_options(self.batch, &opts),
            None => self.client.send(self.batch),
        }
    }
}

// ---------------------------------------------------------------------------
// 2. WorkflowsClient (client.workflows())
// ---------------------------------------------------------------------------

/// High-level multi-step workflows combining common KMIP sequences safely.
pub struct WorkflowsClient<'a> {
    client: &'a mut Client,
}

impl<'a> WorkflowsClient<'a> {
    pub(crate) const fn new(client: &'a mut Client) -> Self {
        Self { client }
    }

    /// Creates a symmetric key (`Create`) and immediately activates it (`Activate`), returning its ID.
    ///
    /// # Errors
    /// Returns [`ClientError`] if creation or activation fails.
    pub fn create_active_symmetric_key(
        self,
        spec: KeySpec,
        usage_mask: CryptographicUsageMask,
        name: Option<&str>,
    ) -> Result<String, ClientError> {
        let mut builder = CreateSymmetricKeyBuilder::new(Some(self.client))
            .with_key_spec(spec)
            .usage_mask(usage_mask);
        if let Some(label) = name {
            builder = builder.name(label);
        }
        let req = builder
            .build_request()
            .map_err(fluent_client_validation_error)?;
        let created: CreateResponse = exec_typed(
            self.client,
            ClientRequest::Create(req),
            |outcome| match outcome {
                ClientBatchOutcome::CreateCompleted(resp) => Some(resp),
                _ => None,
            },
            CreateResponse::result,
        )?;
        let uid = created
            .unique_identifier()
            .ok_or_else(|| fluent_client_validation_error(FluentBuildError::InvalidRequest))?;
        let id = uid_to_string(uid)?;
        let act_req = ActivateBuilder::new(self.client)
            .unique_identifier(id.as_str())
            .build_request()
            .map_err(fluent_client_validation_error)?;
        exec_typed(
            self.client,
            ClientRequest::Activate(act_req),
            |outcome| match outcome {
                ClientBatchOutcome::Activate(resp) => Some(resp),
                _ => None,
            },
            kmipkit_protocol::ActivateResponse::result,
        )?;
        Ok(id)
    }

    /// Creates an asymmetric or PQC key pair (`CreateKeyPair`) and activates both keys, returning `(private_id, public_id)`.
    ///
    /// # Errors
    /// Returns [`ClientError`] if creation or either activation fails.
    pub fn create_active_key_pair(
        self,
        spec: KeySpec,
        private_usage_mask: CryptographicUsageMask,
        public_usage_mask: CryptographicUsageMask,
        private_name: Option<&str>,
        public_name: Option<&str>,
    ) -> Result<(String, String), ClientError> {
        let mut builder = CreateKeyPairBuilder::new(Some(self.client))
            .with_key_spec(spec)
            .private_usage_mask(private_usage_mask)
            .public_usage_mask(public_usage_mask);
        if let Some(p) = private_name {
            builder = builder.private_name(p);
        }
        if let Some(pub_n) = public_name {
            builder = builder.public_name(pub_n);
        }
        let req = builder
            .build_request()
            .map_err(fluent_client_validation_error)?;
        let created: CreateKeyPairResponse = exec_typed(
            self.client,
            ClientRequest::CreateKeyPair(req),
            |outcome| match outcome {
                ClientBatchOutcome::CreateKeyPairCompleted(resp) => Some(resp),
                _ => None,
            },
            CreateKeyPairResponse::result,
        )?;
        let priv_id =
            uid_to_string(created.private_key_unique_identifier().ok_or_else(|| {
                fluent_client_validation_error(FluentBuildError::InvalidRequest)
            })?)?;
        let pub_id =
            uid_to_string(created.public_key_unique_identifier().ok_or_else(|| {
                fluent_client_validation_error(FluentBuildError::InvalidRequest)
            })?)?;
        for target_id in [&priv_id, &pub_id] {
            let act_req = ActivateBuilder::new(self.client)
                .unique_identifier(target_id.as_str())
                .build_request()
                .map_err(fluent_client_validation_error)?;
            exec_typed(
                self.client,
                ClientRequest::Activate(act_req),
                |outcome| match outcome {
                    ClientBatchOutcome::Activate(resp) => Some(resp),
                    _ => None,
                },
                kmipkit_protocol::ActivateResponse::result,
            )?;
        }
        Ok((priv_id, pub_id))
    }

    /// Revokes an object (`Revoke`) and immediately destroys it (`Destroy`).
    ///
    /// # Errors
    /// Returns [`ClientError`] if revocation or destruction fails.
    pub fn revoke_and_destroy(
        self,
        unique_identifier: &str,
        reason: RevocationReason,
    ) -> Result<(), ClientError> {
        let rev_req = RevokeBuilder::new(self.client, reason)
            .unique_identifier(unique_identifier)
            .build_request()
            .map_err(fluent_client_validation_error)?;
        exec_typed(
            self.client,
            ClientRequest::Revoke(rev_req),
            |outcome| match outcome {
                ClientBatchOutcome::Revoke(resp) => Some(resp),
                _ => None,
            },
            kmipkit_protocol::RevokeResponse::result,
        )?;
        let dest_req = DestroyBuilder::new(self.client, Some(unique_identifier.into())).build();
        exec_typed(
            self.client,
            ClientRequest::Destroy(dest_req),
            |outcome| match outcome {
                ClientBatchOutcome::Destroy(resp) => Some(resp),
                _ => None,
            },
            kmipkit_protocol::DestroyResponse::result,
        )?;
        Ok(())
    }

    /// Queries server-supported KMIP versions (`DiscoverVersions`) and updates the client's active version
    /// to the highest mutually supported version (`3.0` -> `2.1` -> `2.0`).
    ///
    /// # Errors
    /// Returns [`ClientError`] if discovery fails or no common version is supported.
    pub fn discover_and_use_version(self) -> Result<ProtocolVersion, ClientError> {
        let req = DiscoverVersionsBuilder::new(self.client)
            .protocol_versions(vec![
                ProtocolVersion::V3_0,
                ProtocolVersion::V2_1,
                ProtocolVersion::V2_0,
            ])
            .build_request()
            .map_err(fluent_client_validation_error)?;
        let resp: DiscoverVersionsResponse = exec_typed(
            self.client,
            ClientRequest::DiscoverVersions(req),
            |outcome| match outcome {
                ClientBatchOutcome::Completed(resp) => Some(resp),
                _ => None,
            },
            DiscoverVersionsResponse::result,
        )?;
        let selected = resp
            .supported_versions()
            .and_then(|list| list.iter().copied().max())
            .ok_or_else(|| fluent_client_validation_error(FluentBuildError::InvalidRequest))?;
        self.client.set_protocol_version(selected);
        Ok(selected)
    }
}

// ---------------------------------------------------------------------------
// 3. Explicit Multipart Streaming (client.streams())
// ---------------------------------------------------------------------------

/// Entry point for explicit stateful multipart cryptographic streams (`Init` -> `Update*` -> `Final`).
pub struct StreamsClient<'a> {
    client: &'a mut Client,
}

impl<'a> StreamsClient<'a> {
    pub(crate) const fn new(client: &'a mut Client) -> Self {
        Self { client }
    }

    /// Starts an explicit multipart `Hash` stream.
    #[must_use]
    pub const fn hash(self, algorithm: HashAlgorithm) -> HashStream<'a> {
        HashStream {
            client: self.client,
            algorithm,
            correlation: None,
            started: false,
        }
    }

    /// Starts an explicit multipart `Encrypt` stream bound to `unique_identifier`.
    #[must_use]
    pub fn encrypt(self, unique_identifier: &str, spec: CipherSpec) -> EncryptStream<'a> {
        EncryptStream {
            client: self.client,
            unique_identifier: unique_identifier.to_owned(),
            spec,
            correlation: None,
            started: false,
        }
    }

    /// Starts an explicit multipart `Decrypt` stream bound to `unique_identifier`.
    #[must_use]
    pub fn decrypt(self, unique_identifier: &str, spec: CipherSpec) -> DecryptStream<'a> {
        DecryptStream {
            client: self.client,
            unique_identifier: unique_identifier.to_owned(),
            spec,
            correlation: None,
            started: false,
        }
    }

    /// Starts an explicit multipart `Sign` stream bound to `unique_identifier`.
    #[must_use]
    pub fn sign(self, unique_identifier: &str, spec: SignatureSpec) -> SignStream<'a> {
        SignStream {
            client: self.client,
            unique_identifier: unique_identifier.to_owned(),
            spec,
            correlation: None,
            started: false,
        }
    }

    /// Starts an explicit multipart `MAC` stream bound to `unique_identifier`.
    #[must_use]
    pub fn mac(self, unique_identifier: &str, spec: MacSpec) -> MacStream<'a> {
        MacStream {
            client: self.client,
            unique_identifier: unique_identifier.to_owned(),
            spec,
            correlation: None,
            started: false,
        }
    }
}

/// Stateful multipart `Hash` stream.
pub struct HashStream<'a> {
    client: &'a mut Client,
    algorithm: HashAlgorithm,
    correlation: Option<SecretBytes>,
    started: bool,
}

impl HashStream<'_> {
    /// Sends the next input chunk (setting `InitIndicator(true)` on the first chunk and forwarding `CorrelationValue`).
    ///
    /// # Errors
    /// Returns [`ClientError`] if the server rejects the chunk.
    #[allow(clippy::needless_pass_by_value)]
    pub fn update(&mut self, chunk: SecretBytes) -> Result<(), ClientError> {
        let raw = chunk.with_bytes(<[u8]>::to_vec);
        let mut b = HashBuilder::new(self.client)
            .with_hash_algorithm(self.algorithm)
            .data(raw);
        if !self.started {
            b = b.init_indicator(true);
            self.started = true;
        } else if let Some(corr) = &self.correlation {
            b = b.correlation_value(corr.with_bytes(<[u8]>::to_vec));
        }
        let req = b.build().map_err(fluent_client_validation_error)?;
        let resp: HashResponse = exec_typed(
            self.client,
            ClientRequest::Hash(req),
            |outcome| match outcome {
                ClientBatchOutcome::Hash(resp) => Some(resp),
                _ => None,
            },
            HashResponse::result,
        )?;
        if let Some(corr) = resp.correlation_value() {
            self.correlation = Some(clone_secret(corr));
        }
        Ok(())
    }

    /// Finalizes the multipart `Hash` stream (`FinalIndicator(true)`) and returns the digest.
    ///
    /// # Errors
    /// Returns [`ClientError`] if finalization fails or the response omits `Data`.
    pub fn finish(self) -> Result<SecretBytes, ClientError> {
        let mut b = HashBuilder::new(self.client)
            .with_hash_algorithm(self.algorithm)
            .final_indicator(true);
        if !self.started {
            b = b.init_indicator(true);
        } else if let Some(corr) = &self.correlation {
            b = b.correlation_value(corr.with_bytes(<[u8]>::to_vec));
        }
        let req = b.build().map_err(fluent_client_validation_error)?;
        let resp: HashResponse = exec_typed(
            self.client,
            ClientRequest::Hash(req),
            |outcome| match outcome {
                ClientBatchOutcome::Hash(resp) => Some(resp),
                _ => None,
            },
            HashResponse::result,
        )?;
        resp.data()
            .map(clone_secret)
            .ok_or_else(|| fluent_client_validation_error(FluentBuildError::InvalidRequest))
    }
}

/// Stateful multipart `Encrypt` stream.
pub struct EncryptStream<'a> {
    client: &'a mut Client,
    unique_identifier: String,
    spec: CipherSpec,
    correlation: Option<SecretBytes>,
    started: bool,
}

impl EncryptStream<'_> {
    /// Encrypts the next plaintext chunk and returns any ciphertext bytes emitted by the server.
    ///
    /// # Errors
    /// Returns [`ClientError`] on failure.
    #[allow(clippy::needless_pass_by_value)]
    pub fn update(&mut self, chunk: SecretBytes) -> Result<Option<SecretBytes>, ClientError> {
        let raw = chunk.with_bytes(<[u8]>::to_vec);
        let mut b = EncryptBuilder::new(self.client)
            .unique_identifier(self.unique_identifier.as_str())
            .with_cipher_spec(self.spec)
            .data(raw);
        if !self.started {
            b = b.init_indicator(true);
            self.started = true;
        } else if let Some(corr) = &self.correlation {
            b = b.correlation_value(corr.with_bytes(<[u8]>::to_vec));
        }
        let req = b.build().map_err(fluent_client_validation_error)?;
        let resp: EncryptResponse = exec_typed(
            self.client,
            ClientRequest::Encrypt(req),
            |outcome| match outcome {
                ClientBatchOutcome::Encrypt(resp) => Some(resp),
                _ => None,
            },
            EncryptResponse::result,
        )?;
        if let Some(corr) = resp.correlation_value() {
            self.correlation = Some(clone_secret(corr));
        }
        Ok(resp.data().map(clone_secret))
    }

    /// Finalizes the multipart `Encrypt` stream (`FinalIndicator(true)`).
    ///
    /// # Errors
    /// Returns [`ClientError`] on failure.
    pub fn finish(self) -> Result<EncryptResponse, ClientError> {
        let mut b = EncryptBuilder::new(self.client)
            .unique_identifier(self.unique_identifier.as_str())
            .with_cipher_spec(self.spec)
            .final_indicator(true);
        if !self.started {
            b = b.init_indicator(true);
        } else if let Some(corr) = &self.correlation {
            b = b.correlation_value(corr.with_bytes(<[u8]>::to_vec));
        }
        let req = b.build().map_err(fluent_client_validation_error)?;
        exec_typed(
            self.client,
            ClientRequest::Encrypt(req),
            |outcome| match outcome {
                ClientBatchOutcome::Encrypt(resp) => Some(resp),
                _ => None,
            },
            EncryptResponse::result,
        )
    }
}

/// Stateful multipart `Decrypt` stream.
pub struct DecryptStream<'a> {
    client: &'a mut Client,
    unique_identifier: String,
    spec: CipherSpec,
    correlation: Option<SecretBytes>,
    started: bool,
}

impl DecryptStream<'_> {
    /// Decrypts the next ciphertext chunk and returns any plaintext bytes emitted by the server.
    ///
    /// # Errors
    /// Returns [`ClientError`] on failure.
    #[allow(clippy::needless_pass_by_value)]
    pub fn update(&mut self, chunk: SecretBytes) -> Result<Option<SecretBytes>, ClientError> {
        let raw = chunk.with_bytes(<[u8]>::to_vec);
        let mut b = DecryptBuilder::new(self.client)
            .unique_identifier(self.unique_identifier.as_str())
            .with_cipher_spec(self.spec)
            .data(raw);
        if !self.started {
            b = b.init_indicator(true);
            self.started = true;
        } else if let Some(corr) = &self.correlation {
            b = b.correlation_value(corr.with_bytes(<[u8]>::to_vec));
        }
        let req = b.build().map_err(fluent_client_validation_error)?;
        let resp: DecryptResponse = exec_typed(
            self.client,
            ClientRequest::Decrypt(req),
            |outcome| match outcome {
                ClientBatchOutcome::Decrypt(resp) => Some(resp),
                _ => None,
            },
            DecryptResponse::result,
        )?;
        if let Some(corr) = resp.correlation_value() {
            self.correlation = Some(clone_secret(corr));
        }
        Ok(resp.data().map(clone_secret))
    }

    /// Finalizes the multipart `Decrypt` stream (`FinalIndicator(true)`).
    ///
    /// # Errors
    /// Returns [`ClientError`] on failure.
    pub fn finish(self) -> Result<DecryptResponse, ClientError> {
        let mut b = DecryptBuilder::new(self.client)
            .unique_identifier(self.unique_identifier.as_str())
            .with_cipher_spec(self.spec)
            .final_indicator(true);
        if !self.started {
            b = b.init_indicator(true);
        } else if let Some(corr) = &self.correlation {
            b = b.correlation_value(corr.with_bytes(<[u8]>::to_vec));
        }
        let req = b.build().map_err(fluent_client_validation_error)?;
        exec_typed(
            self.client,
            ClientRequest::Decrypt(req),
            |outcome| match outcome {
                ClientBatchOutcome::Decrypt(resp) => Some(resp),
                _ => None,
            },
            DecryptResponse::result,
        )
    }
}

/// Stateful multipart `Sign` stream.
pub struct SignStream<'a> {
    client: &'a mut Client,
    unique_identifier: String,
    spec: SignatureSpec,
    correlation: Option<SecretBytes>,
    started: bool,
}

impl SignStream<'_> {
    /// Feeds the next message chunk to the server signing state.
    ///
    /// # Errors
    /// Returns [`ClientError`] on failure.
    #[allow(clippy::needless_pass_by_value)]
    pub fn update(&mut self, chunk: SecretBytes) -> Result<(), ClientError> {
        let raw = chunk.with_bytes(<[u8]>::to_vec);
        let mut b = SignBuilder::new(self.client)
            .unique_identifier(self.unique_identifier.as_str())
            .with_signature_spec(self.spec)
            .data(raw);
        if !self.started {
            b = b.init_indicator(true);
            self.started = true;
        } else if let Some(corr) = &self.correlation {
            b = b.correlation_value(corr.with_bytes(<[u8]>::to_vec));
        }
        let req = b.build().map_err(fluent_client_validation_error)?;
        let resp: SignResponse = exec_typed(
            self.client,
            ClientRequest::Sign(req),
            |outcome| match outcome {
                ClientBatchOutcome::Sign(resp) => Some(resp),
                _ => None,
            },
            SignResponse::result,
        )?;
        if let Some(corr) = resp.correlation_value() {
            self.correlation = Some(clone_secret(corr));
        }
        Ok(())
    }

    /// Finalizes the multipart `Sign` stream (`FinalIndicator(true)`) and returns the signature bytes.
    ///
    /// # Errors
    /// Returns [`ClientError`] if finalization fails or the response omits signature data.
    pub fn finish(self) -> Result<SecretBytes, ClientError> {
        let mut b = SignBuilder::new(self.client)
            .unique_identifier(self.unique_identifier.as_str())
            .with_signature_spec(self.spec)
            .final_indicator(true);
        if !self.started {
            b = b.init_indicator(true);
        } else if let Some(corr) = &self.correlation {
            b = b.correlation_value(corr.with_bytes(<[u8]>::to_vec));
        }
        let req = b.build().map_err(fluent_client_validation_error)?;
        let resp: SignResponse = exec_typed(
            self.client,
            ClientRequest::Sign(req),
            |outcome| match outcome {
                ClientBatchOutcome::Sign(resp) => Some(resp),
                _ => None,
            },
            SignResponse::result,
        )?;
        resp.signature_data()
            .map(clone_secret)
            .ok_or_else(|| fluent_client_validation_error(FluentBuildError::InvalidRequest))
    }
}

/// Stateful multipart `MAC` stream.
pub struct MacStream<'a> {
    client: &'a mut Client,
    unique_identifier: String,
    spec: MacSpec,
    correlation: Option<SecretBytes>,
    started: bool,
}

impl MacStream<'_> {
    /// Feeds the next message chunk to the server MAC state.
    ///
    /// # Errors
    /// Returns [`ClientError`] on failure.
    #[allow(clippy::needless_pass_by_value)]
    pub fn update(&mut self, chunk: SecretBytes) -> Result<(), ClientError> {
        let raw = chunk.with_bytes(<[u8]>::to_vec);
        let mut b = MacBuilder::new(self.client)
            .unique_identifier(self.unique_identifier.as_str())
            .with_mac_spec(self.spec)
            .data(raw);
        if !self.started {
            b = b.init_indicator(true);
            self.started = true;
        } else if let Some(corr) = &self.correlation {
            b = b.correlation_value(corr.with_bytes(<[u8]>::to_vec));
        }
        let req = b.build().map_err(fluent_client_validation_error)?;
        let resp: MacResponse = exec_typed(
            self.client,
            ClientRequest::Mac(req),
            |outcome| match outcome {
                ClientBatchOutcome::Mac(resp) => Some(resp),
                _ => None,
            },
            MacResponse::result,
        )?;
        if let Some(corr) = resp.correlation_value() {
            self.correlation = Some(clone_secret(corr));
        }
        Ok(())
    }

    /// Finalizes the multipart `MAC` stream (`FinalIndicator(true)`) and returns the MAC tag.
    ///
    /// # Errors
    /// Returns [`ClientError`] if finalization fails or the response omits MAC data.
    pub fn finish(self) -> Result<SecretBytes, ClientError> {
        let mut b = MacBuilder::new(self.client)
            .unique_identifier(self.unique_identifier.as_str())
            .with_mac_spec(self.spec)
            .final_indicator(true);
        if !self.started {
            b = b.init_indicator(true);
        } else if let Some(corr) = &self.correlation {
            b = b.correlation_value(corr.with_bytes(<[u8]>::to_vec));
        }
        let req = b.build().map_err(fluent_client_validation_error)?;
        let resp: MacResponse = exec_typed(
            self.client,
            ClientRequest::Mac(req),
            |outcome| match outcome {
                ClientBatchOutcome::Mac(resp) => Some(resp),
                _ => None,
            },
            MacResponse::result,
        )?;
        resp.mac_data()
            .map(clone_secret)
            .ok_or_else(|| fluent_client_validation_error(FluentBuildError::InvalidRequest))
    }
}

// ---------------------------------------------------------------------------
// 4. PendingOperation handle
// ---------------------------------------------------------------------------

/// Handle for an asynchronous KMIP operation carrying its server-assigned `Asynchronous Correlation Value`.
pub struct PendingOperation {
    correlation_value: SecretBytes,
}

impl PendingOperation {
    /// Wraps a server-issued `Asynchronous Correlation Value`.
    #[must_use]
    pub const fn new(correlation_value: SecretBytes) -> Self {
        Self { correlation_value }
    }

    /// Borrows the underlying correlation value.
    #[must_use]
    pub const fn correlation_value(&self) -> &SecretBytes {
        &self.correlation_value
    }

    /// Sends a single synchronous `Poll` request for this operation.
    ///
    /// # Errors
    /// Returns [`ClientError`] on failure.
    pub fn poll(&self, client: &mut Client) -> Result<KmipOperationResult, ClientError> {
        let req = PollBuilder::new(client, clone_secret(&self.correlation_value)).build();
        exec_kmi_result(client, ClientRequest::Poll(req), |outcome| match outcome {
            ClientBatchOutcome::Poll(res) => Some(res),
            _ => None,
        })
    }

    /// Sends a single synchronous `Process` request for this operation.
    ///
    /// # Errors
    /// Returns [`ClientError`] on failure.
    pub fn process(&self, client: &mut Client) -> Result<KmipOperationResult, ClientError> {
        let req = ProcessBuilder::new(client, clone_secret(&self.correlation_value)).build();
        exec_kmi_result(
            client,
            ClientRequest::Process(req),
            |outcome| match outcome {
                ClientBatchOutcome::Process(res) => Some(res),
                _ => None,
            },
        )
    }

    /// Sends a single synchronous `Cancel` request for this operation.
    ///
    /// # Errors
    /// Returns [`ClientError`] on failure.
    pub fn cancel(&self, client: &mut Client) -> Result<KmipOperationResult, ClientError> {
        let req = CancelBuilder::new(client, clone_secret(&self.correlation_value)).build();
        exec_kmi_result(
            client,
            ClientRequest::Cancel(req),
            |outcome| match outcome {
                ClientBatchOutcome::Cancel(res) => Some(res),
                _ => None,
            },
        )
    }
}

// ---------------------------------------------------------------------------
// 5. Paginated Locate iterator
// ---------------------------------------------------------------------------

/// Page-by-page paginator for KMIP `Locate` using `Maximum Items` and `Offset Items`.
pub struct LocatePaginator<'a> {
    client: &'a mut Client,
    attributes: AttributeSet,
    page_size: i32,
    offset: i32,
    exhausted: bool,
}

impl<'a> LocatePaginator<'a> {
    pub(crate) fn new(client: &'a mut Client, attributes: AttributeSet, page_size: i32) -> Self {
        Self {
            client,
            attributes,
            page_size: page_size.max(1),
            offset: 0,
            exhausted: false,
        }
    }

    /// Fetches the next page of [`UniqueIdentifier`]s, or `Ok(None)` when all pages have been consumed.
    ///
    /// # Errors
    /// Returns [`ClientError`] if the `Locate` exchange fails.
    pub fn next_page(&mut self) -> Result<Option<Vec<UniqueIdentifier>>, ClientError> {
        if self.exhausted {
            return Ok(None);
        }
        let attrs = AttributeSet::try_new(
            self.attributes
                .as_items()
                .iter()
                .filter_map(|item| item.try_clone().ok()),
        )
        .map_err(|_| fluent_client_validation_error(FluentBuildError::InvalidAttributeValue))?;
        let req = LocateBuilder::new(self.client, attrs)
            .maximum_items(self.page_size)
            .offset_items(self.offset)
            .build_request()
            .map_err(fluent_client_validation_error)?;
        let resp: LocateResponse = exec_typed(
            self.client,
            ClientRequest::Locate(req),
            |outcome| match outcome {
                ClientBatchOutcome::LocateCompleted(resp) => Some(resp),
                _ => None,
            },
            LocateResponse::result,
        )?;
        let ids: Vec<UniqueIdentifier> = resp.unique_identifiers().to_vec();
        let count = i32::try_from(ids.len()).unwrap_or(self.page_size);
        self.offset = self.offset.saturating_add(count);
        if count < self.page_size {
            self.exhausted = true;
        }
        if ids.is_empty() {
            Ok(None)
        } else {
            Ok(Some(ids))
        }
    }
}

impl Client {
    /// Starts a fluent multi-operation request batch (`ClientBatch`).
    #[must_use]
    pub fn batch(&mut self) -> FluentBatchBuilder<'_> {
        FluentBatchBuilder::new(self)
    }

    /// Returns the composite high-level workflows namespace (`create_active_symmetric_key`, `revoke_and_destroy`, etc.).
    #[must_use]
    pub fn workflows(&mut self) -> WorkflowsClient<'_> {
        WorkflowsClient::new(self)
    }

    /// Returns the explicit stateful multipart streaming namespace (`hash`, `encrypt`, `decrypt`, `sign`, `mac`).
    #[must_use]
    pub fn streams(&mut self) -> StreamsClient<'_> {
        StreamsClient::new(self)
    }
}
