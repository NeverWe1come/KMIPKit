//! Fluent builders for KMIP 3.0 object, group, credential, deactivation, and KEM operations.

use kmipkit_protocol::{
    AttributeSet, CreateCredentialObject, CreateCredentialRequest, CreateCredentialResponse,
    CreateGroupRequest, CreateGroupResponse, CreateUserRequest, CreateUserResponse,
    CryptographicParameters, DeactivateRequest, DeactivateResponse, DecapsulateRequest,
    DecapsulateResponse, EncapsulateRequest, EncapsulateResponse, KemSpec, ObliterateRequest,
    ObliterateResponse, ProtocolVersion, UniqueIdentifier,
};
use kmipkit_transport::RequestOptions;
use kmipkit_ttlv::Structure;
use kmipkit_ttlv::codec::CodecLimits;

use crate::{
    Client, ClientBatchItem, ClientBatchItemResponse, ClientBatchOutcome, ClientError,
    ClientRequest, Kmip30OperationResponse,
};

use super::execution::{
    FluentBuildError, effective_protocol_version, execute_fluent_outcome, execute_fluent_request,
    normalize_uid_for_version, unwrap_typed_outcome,
};

struct Kmip30BuilderExecution<'a> {
    client: &'a mut Client,
    options: Option<RequestOptions>,
    asynchronous_indicator: Option<u32>,
}

impl<'a> Kmip30BuilderExecution<'a> {
    pub(super) fn new(client: &'a mut Client) -> Self {
        Self {
            client,
            options: None,
            asynchronous_indicator: None,
        }
    }

    fn protocol_version(&self) -> ProtocolVersion {
        effective_protocol_version(Some(self.client), self.options.as_ref())
    }

    fn execute_outcome_with_limits(
        self,
        request: ClientRequest,
        limits: &CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        execute_fluent_request(
            self.client,
            request,
            self.asynchronous_indicator,
            self.options,
            limits,
        )
    }

    fn execute_outcome(
        self,
        request: ClientRequest,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        execute_fluent_outcome(
            self.client,
            request,
            self.asynchronous_indicator,
            self.options,
        )
    }
}

trait Kmip30BuilderParts<'a> {
    fn into_parts(self) -> (Kmip30BuilderExecution<'a>, ClientRequest);
    fn execution(&mut self) -> &mut Kmip30BuilderExecution<'a>;
}

macro_rules! impl_kmip30_builder_methods {
    ($builder:ident, $resp:ty, $variant:ident) => {
        impl<'a> $builder<'a> {
            /// Sets one-exchange options, including a caller-selected protocol version.
            #[must_use]
            pub fn with_options(mut self, options: RequestOptions) -> Self {
                self.execution().options = Some(options);
                self
            }

            /// Sets the KMIP protocol version override for this request/exchange.
            #[must_use]
            pub fn with_protocol_version(mut self, version: ProtocolVersion) -> Self {
                let exec = self.execution();
                let options = exec
                    .options
                    .unwrap_or_default()
                    .with_protocol_version(version);
                exec.options = Some(options);
                self
            }

            /// Sets the KMIP protocol version override (alias for [`Self::with_protocol_version`]).
            #[must_use]
            pub fn with_kmip_version(self, version: ProtocolVersion) -> Self {
                self.with_protocol_version(version)
            }

            /// Sets the Request Header Asynchronous Indicator for this exchange.
            #[must_use]
            pub fn with_asynchronous_indicator(mut self, value: u32) -> Self {
                self.execution().asynchronous_indicator = Some(value);
                self
            }

            /// Validates the semantic inputs held by the builder without sending.
            ///
            /// # Errors
            ///
            /// This operation currently has no local failure case because the
            /// builder accepts only required, already typed request fields.
            pub fn validate(&self) -> Result<(), FluentBuildError> {
                Ok(())
            }

            /// Builds the existing typed request variant.
            #[must_use]
            pub fn build(self) -> ClientRequest {
                self.into_parts().1
            }

            /// Converts the request to one typed batch item for explicit batching.
            #[must_use]
            pub fn to_batch_item(self) -> ClientBatchItem {
                let (_, request) = self.into_parts();
                ClientBatchItem::new(request)
            }

            /// Executes exactly one typed exchange using the client's configured [`CodecLimits`]
            /// and returns the unwrapped response, erroring on server failure or `Pending`.
            ///
            /// # Errors
            /// Returns the local validation, version, transport, protocol, `ServerResult`, or `Pending` error.
            pub fn execute(self) -> Result<$resp, ClientError> {
                let item = self.execute_outcome()?;
                unwrap_typed_outcome(
                    item,
                    |outcome| match outcome {
                        ClientBatchOutcome::Kmip30(Kmip30OperationResponse::$variant(resp)) => {
                            Some(resp)
                        }
                        _ => None,
                    },
                    |resp| resp.result(),
                )
            }

            /// Executes exactly one typed exchange using the client's configured [`CodecLimits`]
            /// and returns the raw [`ClientBatchItemResponse`] envelope (`Pending` and failures included).
            ///
            /// # Errors
            /// Returns the local validation, version, transport, or response-decoding error.
            pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
                let (execution, request) = self.into_parts();
                execution.execute_outcome(request)
            }

            /// Executes exactly one typed exchange with explicit [`CodecLimits`] overrides
            /// and returns the raw [`ClientBatchItemResponse`] envelope.
            ///
            /// # Errors
            /// Returns the local validation, version, transport, or response-decoding error.
            pub fn execute_outcome_with_limits(
                self,
                limits: &CodecLimits,
            ) -> Result<ClientBatchItemResponse, ClientError> {
                let (execution, request) = self.into_parts();
                execution.execute_outcome_with_limits(request, limits)
            }
        }
    };
}

/// A KMIP 3.0 Create Credential fluent builder.
pub struct CreateCredentialBuilder<'a> {
    execution: Kmip30BuilderExecution<'a>,
    attributes: AttributeSet,
    object: CreateCredentialObject,
}

impl<'a> CreateCredentialBuilder<'a> {
    pub(super) fn new(
        client: &'a mut Client,
        attributes: AttributeSet,
        object: CreateCredentialObject,
    ) -> Self {
        Self {
            execution: Kmip30BuilderExecution::new(client),
            attributes,
            object,
        }
    }
}
impl<'a> Kmip30BuilderParts<'a> for CreateCredentialBuilder<'a> {
    fn into_parts(self) -> (Kmip30BuilderExecution<'a>, ClientRequest) {
        (
            self.execution,
            ClientRequest::create_credential(CreateCredentialRequest::new(
                self.attributes,
                self.object,
            )),
        )
    }
    fn execution(&mut self) -> &mut Kmip30BuilderExecution<'a> {
        &mut self.execution
    }
}
impl_kmip30_builder_methods!(
    CreateCredentialBuilder,
    CreateCredentialResponse,
    CreateCredential
);

/// A KMIP 3.0 Create Group fluent builder.
pub struct CreateGroupBuilder<'a> {
    execution: Kmip30BuilderExecution<'a>,
    attributes: AttributeSet,
}
impl<'a> CreateGroupBuilder<'a> {
    pub(super) fn new(client: &'a mut Client, attributes: AttributeSet) -> Self {
        Self {
            execution: Kmip30BuilderExecution::new(client),
            attributes,
        }
    }
}
impl<'a> Kmip30BuilderParts<'a> for CreateGroupBuilder<'a> {
    fn into_parts(self) -> (Kmip30BuilderExecution<'a>, ClientRequest) {
        (
            self.execution,
            ClientRequest::create_group(CreateGroupRequest::new(self.attributes)),
        )
    }
    fn execution(&mut self) -> &mut Kmip30BuilderExecution<'a> {
        &mut self.execution
    }
}
impl_kmip30_builder_methods!(CreateGroupBuilder, CreateGroupResponse, CreateGroup);

/// A KMIP 3.0 Create User fluent builder.
pub struct CreateUserBuilder<'a> {
    execution: Kmip30BuilderExecution<'a>,
    attributes: AttributeSet,
}
impl<'a> CreateUserBuilder<'a> {
    pub(super) fn new(client: &'a mut Client, attributes: AttributeSet) -> Self {
        Self {
            execution: Kmip30BuilderExecution::new(client),
            attributes,
        }
    }
}
impl<'a> Kmip30BuilderParts<'a> for CreateUserBuilder<'a> {
    fn into_parts(self) -> (Kmip30BuilderExecution<'a>, ClientRequest) {
        (
            self.execution,
            ClientRequest::create_user(CreateUserRequest::new(self.attributes)),
        )
    }
    fn execution(&mut self) -> &mut Kmip30BuilderExecution<'a> {
        &mut self.execution
    }
}
impl_kmip30_builder_methods!(CreateUserBuilder, CreateUserResponse, CreateUser);

/// A KMIP 3.0 Deactivate fluent builder.
pub struct DeactivateBuilder<'a> {
    execution: Kmip30BuilderExecution<'a>,
    unique_identifier: UniqueIdentifier,
    reason: Option<Structure>,
    date: Option<i64>,
}
impl<'a> DeactivateBuilder<'a> {
    pub(super) fn new(client: &'a mut Client, unique_identifier: UniqueIdentifier) -> Self {
        Self {
            execution: Kmip30BuilderExecution::new(client),
            unique_identifier,
            reason: None,
            date: None,
        }
    }
    /// Sets the optional typed Deactivation Reason Structure.
    #[must_use]
    pub fn deactivation_reason(mut self, reason: Structure) -> Self {
        self.reason = Some(reason);
        self
    }
    /// Sets the optional Deactivation Date as seconds since the Unix epoch.
    #[must_use]
    pub const fn deactivation_date(mut self, unix_seconds: i64) -> Self {
        self.date = Some(unix_seconds);
        self
    }
}
impl<'a> Kmip30BuilderParts<'a> for DeactivateBuilder<'a> {
    fn into_parts(self) -> (Kmip30BuilderExecution<'a>, ClientRequest) {
        let uid =
            normalize_uid_for_version(self.unique_identifier, self.execution.protocol_version());
        let mut request = DeactivateRequest::new(uid);
        if let Some(reason) = self.reason {
            request = request.with_deactivation_reason(reason);
        }
        if let Some(date) = self.date {
            request = request.with_deactivation_date(date);
        }
        (self.execution, ClientRequest::deactivate(request))
    }
    fn execution(&mut self) -> &mut Kmip30BuilderExecution<'a> {
        &mut self.execution
    }
}
impl_kmip30_builder_methods!(DeactivateBuilder, DeactivateResponse, Deactivate);

/// A KMIP 3.0 Decapsulate fluent builder.
pub struct DecapsulateBuilder<'a> {
    execution: Kmip30BuilderExecution<'a>,
    unique_identifier: UniqueIdentifier,
    ciphertext: Vec<u8>,
    parameters: Option<CryptographicParameters>,
    attributes: Option<AttributeSet>,
}
impl<'a> DecapsulateBuilder<'a> {
    pub(super) fn new(
        client: &'a mut Client,
        unique_identifier: UniqueIdentifier,
        ciphertext: Vec<u8>,
    ) -> Self {
        Self {
            execution: Kmip30BuilderExecution::new(client),
            unique_identifier,
            ciphertext,
            parameters: None,
            attributes: None,
        }
    }
    /// Sets caller-selected typed Cryptographic Parameters.
    #[must_use]
    pub fn cryptographic_parameters(mut self, value: CryptographicParameters) -> Self {
        self.parameters = Some(value);
        self
    }
    /// Configures decapsulation parameters from a curated [`KemSpec`] preset.
    #[must_use]
    pub fn with_kem_spec(self, spec: KemSpec) -> Self {
        if let Ok(params) = spec.to_cryptographic_parameters() {
            return self.cryptographic_parameters(params);
        }
        self
    }
    /// Sets the optional prevalidated ordered Attribute Set.
    #[must_use]
    pub fn attributes(mut self, value: AttributeSet) -> Self {
        self.attributes = Some(value);
        self
    }
}
impl<'a> Kmip30BuilderParts<'a> for DecapsulateBuilder<'a> {
    fn into_parts(self) -> (Kmip30BuilderExecution<'a>, ClientRequest) {
        let uid =
            normalize_uid_for_version(self.unique_identifier, self.execution.protocol_version());
        let mut request = DecapsulateRequest::new(uid, self.ciphertext);
        if let Some(value) = self.parameters {
            request = request.with_typed_cryptographic_parameters(value);
        }
        if let Some(value) = self.attributes {
            request = request.with_attributes(value);
        }
        (self.execution, ClientRequest::decapsulate(request))
    }
    fn execution(&mut self) -> &mut Kmip30BuilderExecution<'a> {
        &mut self.execution
    }
}
impl_kmip30_builder_methods!(DecapsulateBuilder, DecapsulateResponse, Decapsulate);

/// A KMIP 3.0 Encapsulate fluent builder.
pub struct EncapsulateBuilder<'a> {
    execution: Kmip30BuilderExecution<'a>,
    unique_identifier: UniqueIdentifier,
    parameters: Option<CryptographicParameters>,
    attributes: Option<AttributeSet>,
}
impl<'a> EncapsulateBuilder<'a> {
    pub(super) fn new(client: &'a mut Client, unique_identifier: UniqueIdentifier) -> Self {
        Self {
            execution: Kmip30BuilderExecution::new(client),
            unique_identifier,
            parameters: None,
            attributes: None,
        }
    }
    /// Sets caller-selected typed Cryptographic Parameters.
    #[must_use]
    pub fn cryptographic_parameters(mut self, value: CryptographicParameters) -> Self {
        self.parameters = Some(value);
        self
    }
    /// Configures encapsulation parameters from a curated [`KemSpec`] preset.
    #[must_use]
    pub fn with_kem_spec(self, spec: KemSpec) -> Self {
        if let Ok(params) = spec.to_cryptographic_parameters() {
            return self.cryptographic_parameters(params);
        }
        self
    }
    /// Sets the optional prevalidated ordered Attribute Set.
    #[must_use]
    pub fn attributes(mut self, value: AttributeSet) -> Self {
        self.attributes = Some(value);
        self
    }
}
impl<'a> Kmip30BuilderParts<'a> for EncapsulateBuilder<'a> {
    fn into_parts(self) -> (Kmip30BuilderExecution<'a>, ClientRequest) {
        let uid =
            normalize_uid_for_version(self.unique_identifier, self.execution.protocol_version());
        let mut request = EncapsulateRequest::new(uid);
        if let Some(value) = self.parameters {
            request = request.with_typed_cryptographic_parameters(value);
        }
        if let Some(value) = self.attributes {
            request = request.with_attributes(value);
        }
        (self.execution, ClientRequest::encapsulate(request))
    }
    fn execution(&mut self) -> &mut Kmip30BuilderExecution<'a> {
        &mut self.execution
    }
}
impl_kmip30_builder_methods!(EncapsulateBuilder, EncapsulateResponse, Encapsulate);

/// A KMIP 3.0 Obliterate fluent builder.
pub struct ObliterateBuilder<'a> {
    execution: Kmip30BuilderExecution<'a>,
    unique_identifier: UniqueIdentifier,
}
impl<'a> ObliterateBuilder<'a> {
    pub(super) fn new(client: &'a mut Client, unique_identifier: UniqueIdentifier) -> Self {
        Self {
            execution: Kmip30BuilderExecution::new(client),
            unique_identifier,
        }
    }
}
impl<'a> Kmip30BuilderParts<'a> for ObliterateBuilder<'a> {
    fn into_parts(self) -> (Kmip30BuilderExecution<'a>, ClientRequest) {
        let uid =
            normalize_uid_for_version(self.unique_identifier, self.execution.protocol_version());
        (
            self.execution,
            ClientRequest::obliterate(ObliterateRequest::new(uid)),
        )
    }
    fn execution(&mut self) -> &mut Kmip30BuilderExecution<'a> {
        &mut self.execution
    }
}
impl_kmip30_builder_methods!(ObliterateBuilder, ObliterateResponse, Obliterate);
