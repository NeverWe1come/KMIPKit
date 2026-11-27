//! Task-oriented, fluent builders over the existing typed KMIP requests.

macro_rules! fluent_option_field_setters {
    ($option_doc:literal; $($options_path:ident).+; $($indicator_path:ident).+; const) => {
        #[doc = $option_doc]
        #[must_use]
        pub fn with_options(mut self, options: kmipkit_transport::RequestOptions) -> Self {
            self.$($options_path).+ = Some(options);
            self
        }

        /// Sets the KMIP protocol version override for this request/exchange.
        #[must_use]
        pub fn with_protocol_version(mut self, version: kmipkit_protocol::ProtocolVersion) -> Self {
            let options = self.$($options_path).+.unwrap_or_default().with_protocol_version(version);
            self.$($options_path).+ = Some(options);
            self
        }

        /// Sets the KMIP protocol version override for this request/exchange (alias for [`Self::with_protocol_version`]).
        #[must_use]
        pub fn with_kmip_version(self, version: kmipkit_protocol::ProtocolVersion) -> Self {
            self.with_protocol_version(version)
        }

        /// Sets the Request Header Asynchronous Indicator for this exchange.
        #[must_use]
        pub const fn with_asynchronous_indicator(mut self, value: u32) -> Self {
            self.$($indicator_path).+ = Some(value);
            self
        }
    };
    ($option_doc:literal; $($options_path:ident).+; $($indicator_path:ident).+; non_const) => {
        #[doc = $option_doc]
        #[must_use]
        pub fn with_options(mut self, options: kmipkit_transport::RequestOptions) -> Self {
            self.$($options_path).+ = Some(options);
            self
        }

        /// Sets the KMIP protocol version override for this request/exchange.
        #[must_use]
        pub fn with_protocol_version(mut self, version: kmipkit_protocol::ProtocolVersion) -> Self {
            let options = self.$($options_path).+.unwrap_or_default().with_protocol_version(version);
            self.$($options_path).+ = Some(options);
            self
        }

        /// Sets the KMIP protocol version override for this request/exchange (alias for [`Self::with_protocol_version`]).
        #[must_use]
        pub fn with_kmip_version(self, version: kmipkit_protocol::ProtocolVersion) -> Self {
            self.with_protocol_version(version)
        }

        /// Sets the Request Header Asynchronous Indicator for this exchange.
        #[must_use]
        pub fn with_asynchronous_indicator(mut self, value: u32) -> Self {
            self.$($indicator_path).+ = Some(value);
            self
        }
    };
}

mod admin;
mod async_ops;
mod attributes;
mod crypto;
mod domain;
mod execution;
mod flows;
mod keys;
mod kmip30;
mod lifecycle;
mod sessions;

use crate::Client;
use kmipkit_protocol::{
    AttributeSet, CreateCredentialObject, EndpointRole, RevocationReason, SecretBytes, Ticket,
    UniqueIdentifier,
};

pub use admin::{
    CheckBuilder, DiscoverVersionsBuilder, GetConstraintsBuilder, GetUsageAllocationBuilder,
    InteropBuilder, LogBuilder, ObtainLeaseBuilder, PingBuilder, Pkcs11Builder, QueryBuilder,
    ReProvisionBuilder, RngRetrieveBuilder, RngSeedBuilder, SetConstraintsBuilder,
    SetDefaultsBuilder,
};
pub use async_ops::{CancelBuilder, PollBuilder, ProcessBuilder, QueryAsyncRequestsBuilder};
pub use attributes::{
    AddAttributeBuilder, AdjustAttributeBuilder, DeleteAttributeBuilder, GetAttributeListBuilder,
    GetAttributesBuilder, ModifyAttributeBuilder, SetAttributeBuilder,
};
pub use crypto::{
    DecryptBuilder, EncryptBuilder, HashBuilder, MacBuilder, MacVerifyBuilder, SignBuilder,
    SignatureVerifyBuilder,
};
pub use domain::{
    AdminClient, AsyncOpsClient, AttributesClient, CertificatesClient, CryptoClient, KeyRef,
    KeysClient, SecretsClient, SessionsClient,
};
pub use execution::{
    AttributeAdjustmentValue, CryptographicUsageMask, FluentAttributePosition, FluentBuildError,
};
pub use flows::{
    DecryptStream, EncryptStream, FluentBatchBuilder, HashStream, LocatePaginator, MacStream,
    PendingOperation, SignStream, StreamsClient, WorkflowsClient,
};
pub use keys::{
    CreateBuilder, CreateKeyPairBuilder, CreateSplitKeyBuilder, CreateSymmetricKeyBuilder,
    DeriveKeyBuilder, ExportBuilder, ImportBuilder, JoinSplitKeyBuilder, RegisterBuilder,
};
pub use kmip30::{
    CreateCredentialBuilder, CreateGroupBuilder, CreateUserBuilder, DeactivateBuilder,
    DecapsulateBuilder, EncapsulateBuilder, ObliterateBuilder,
};
pub use kmipkit_protocol::{
    AttributesBuilder, CipherSpec, HashAlgorithm, KdfSpec, KemSpec, KeySpec, MacSpec, PemBlock,
    PresetError, SignatureSpec, decode_pem, decode_pem_der,
};
pub use lifecycle::{
    ActivateBuilder, ArchiveBuilder, CertifyBuilder, DestroyBuilder, GetBuilder, LocateBuilder,
    RecertifyBuilder, RecoverBuilder, RekeyBuilder, RekeyKeyPairBuilder, RevokeBuilder,
    ValidateBuilder,
};
pub use sessions::{DelegatedLoginBuilder, LoginBuilder, LogoutBuilder, SetEndpointRoleBuilder};

/// A scoped fluent namespace borrowed from a mutable client.
pub struct RequestBuilder<'a> {
    client: &'a mut Client,
}

impl<'a> RequestBuilder<'a> {
    /// Begins constructing a typed Discover Versions request with the KMIP 2.1 default offer.
    #[must_use]
    pub fn discover_versions(self) -> DiscoverVersionsBuilder<'a> {
        DiscoverVersionsBuilder::new(self.client)
    }

    /// Begins constructing a typed Query request.
    #[must_use]
    pub fn query(self) -> QueryBuilder<'a> {
        QueryBuilder::new(self.client)
    }

    /// Begins constructing a typed Check request.
    #[must_use]
    pub fn check(self) -> CheckBuilder<'a> {
        CheckBuilder::new(self.client)
    }

    /// Begins constructing an Obtain Lease request.
    #[must_use]
    pub fn obtain_lease(self) -> ObtainLeaseBuilder<'a> {
        ObtainLeaseBuilder::new(self.client)
    }

    /// Begins constructing a typed Modify Attribute request.
    #[must_use]
    pub fn modify_attribute(self) -> ModifyAttributeBuilder<'a> {
        ModifyAttributeBuilder::new(self.client)
    }

    /// Begins constructing a typed Adjust Attribute request.
    #[must_use]
    pub fn adjust_attribute(self) -> AdjustAttributeBuilder<'a> {
        AdjustAttributeBuilder::new(self.client)
    }

    /// Begins a KMIP 3.0 Create Credential builder using a typed credential
    /// alternative. Its Credential Type is derived from the selected object.
    /// The caller must explicitly select protocol version 3.0 on the client or
    /// in exchange options before execution.
    #[must_use]
    pub fn create_credential(
        self,
        attributes: AttributeSet,
        object: CreateCredentialObject,
    ) -> CreateCredentialBuilder<'a> {
        CreateCredentialBuilder::new(self.client, attributes, object)
    }

    /// Begins a KMIP 3.0 Create Group builder. Protocol version 3.0 remains an
    /// explicit caller choice.
    #[must_use]
    pub fn create_group(self, attributes: AttributeSet) -> CreateGroupBuilder<'a> {
        CreateGroupBuilder::new(self.client, attributes)
    }

    /// Begins a KMIP 3.0 Create User builder. Protocol version 3.0 remains an
    /// explicit caller choice.
    #[must_use]
    pub fn create_user(self, attributes: AttributeSet) -> CreateUserBuilder<'a> {
        CreateUserBuilder::new(self.client, attributes)
    }

    /// Begins a KMIP 3.0 Deactivate builder for a caller-supplied identifier.
    #[must_use]
    pub fn deactivate(
        self,
        unique_identifier: impl Into<UniqueIdentifier>,
    ) -> DeactivateBuilder<'a> {
        DeactivateBuilder::new(self.client, unique_identifier.into())
    }

    /// Begins a KMIP 3.0 Decapsulate builder for a caller-supplied identifier
    /// and encapsulated ciphertext.
    #[must_use]
    pub fn decapsulate(
        self,
        unique_identifier: impl Into<UniqueIdentifier>,
        ciphertext: Vec<u8>,
    ) -> DecapsulateBuilder<'a> {
        DecapsulateBuilder::new(self.client, unique_identifier.into(), ciphertext)
    }

    /// Begins a KMIP 3.0 Encapsulate builder for a caller-supplied identifier.
    #[must_use]
    pub fn encapsulate(
        self,
        unique_identifier: impl Into<UniqueIdentifier>,
    ) -> EncapsulateBuilder<'a> {
        EncapsulateBuilder::new(self.client, unique_identifier.into())
    }

    /// Begins a KMIP 3.0 Obliterate builder for a caller-supplied identifier.
    #[must_use]
    pub fn obliterate(
        self,
        unique_identifier: impl Into<UniqueIdentifier>,
    ) -> ObliterateBuilder<'a> {
        ObliterateBuilder::new(self.client, unique_identifier.into())
    }
    pub(crate) const fn new(client: &'a mut Client) -> Self {
        Self { client }
    }

    /// Begins constructing a Create Symmetric Key request.
    #[must_use]
    pub fn create_symmetric_key(self) -> CreateSymmetricKeyBuilder<'a> {
        CreateSymmetricKeyBuilder::new(Some(self.client))
    }

    /// Begins constructing a typed Create Key Pair request.
    #[must_use]
    pub fn create_key_pair(self) -> CreateKeyPairBuilder<'a> {
        CreateKeyPairBuilder::new(Some(self.client))
    }

    /// Begins constructing a typed Create Split Key request.
    #[must_use]
    pub fn create_split_key(self) -> CreateSplitKeyBuilder<'a> {
        CreateSplitKeyBuilder::new(self.client)
    }

    /// Begins a generic Create request with Object Type unset.
    #[must_use]
    pub fn create(self) -> CreateBuilder<'a> {
        CreateBuilder::new(self.client)
    }

    /// Begins a typed Derive Key request with its source, method, parameters,
    /// and output attributes unset.
    #[must_use]
    pub fn derive_key(self) -> DeriveKeyBuilder<'a> {
        DeriveKeyBuilder::new(self.client)
    }

    /// Begins constructing a typed Register request.
    #[must_use]
    pub fn register(self) -> RegisterBuilder<'a> {
        RegisterBuilder::new(self.client)
    }

    /// Begins constructing a typed Get request for an explicit identifier.
    #[must_use]
    pub fn get(self, unique_identifier: impl Into<UniqueIdentifier>) -> GetBuilder<'a> {
        self.get_request().unique_identifier(unique_identifier)
    }

    /// Begins a Get request with all selectors unset, including the identifier.
    #[must_use]
    pub fn get_request(self) -> GetBuilder<'a> {
        GetBuilder::new(self.client)
    }

    /// Begins a typed Set Attribute request with its optional identifier
    /// omitted.
    #[must_use]
    pub fn set_attribute(self) -> SetAttributeBuilder<'a> {
        SetAttributeBuilder::new(Some(self.client))
    }

    /// Begins constructing an Add Attribute request with no Unique Identifier.
    #[must_use]
    pub fn add_attribute(self) -> AddAttributeBuilder<'a> {
        AddAttributeBuilder::new(Some(self.client))
    }

    /// Begins a typed Delete Attribute request with all optional fields omitted.
    #[must_use]
    pub fn delete_attribute(self) -> DeleteAttributeBuilder<'a> {
        DeleteAttributeBuilder::new(Some(self.client))
    }

    /// Begins a typed Get Attributes request with no Attribute References.
    ///
    /// With no references, the server returns all attributes. A text Unique
    /// Identifier can be added with [`GetAttributesBuilder::unique_identifier`].
    #[must_use]
    pub fn get_attributes(self) -> GetAttributesBuilder<'a> {
        GetAttributesBuilder::new(self.client)
    }

    /// Begins a typed Get Attribute List request with its Unique Identifier
    /// omitted.
    ///
    /// The server resolves the KMIP ID Placeholder when this builder is used
    /// in a batch that contains a preceding operation.
    #[must_use]
    pub fn get_attribute_list(self) -> GetAttributeListBuilder<'a> {
        GetAttributeListBuilder::new(self.client)
    }

    /// Begins a typed empty Get Constraints request.
    #[must_use]
    pub fn get_constraints(self) -> GetConstraintsBuilder<'a> {
        GetConstraintsBuilder::new(self.client)
    }

    /// Begins a typed Get Usage Allocation request with Usage Limits Count unset.
    ///
    /// The Unique Identifier is optional when the surrounding batch context
    /// establishes the KMIP ID Placeholder.
    #[must_use]
    pub fn get_usage_allocation(self) -> GetUsageAllocationBuilder<'a> {
        GetUsageAllocationBuilder::new(self.client)
    }

    /// Begins a typed RNG Retrieve request with Data Length unset.
    #[must_use]
    pub fn rng_retrieve(self) -> RngRetrieveBuilder<'a> {
        RngRetrieveBuilder::new(self.client)
    }

    /// Begins a typed RNG Seed request with secret Data unset.
    #[must_use]
    pub fn rng_seed(self) -> RngSeedBuilder<'a> {
        RngSeedBuilder::new(self.client)
    }

    /// Begins constructing a typed Destroy request for an explicit identifier.
    #[must_use]
    pub fn destroy(self, unique_identifier: impl Into<UniqueIdentifier>) -> DestroyBuilder<'a> {
        DestroyBuilder::new(self.client, Some(unique_identifier.into()))
    }

    /// Begins an Activate request with its optional identifier omitted.
    #[must_use]
    pub fn activate(self) -> ActivateBuilder<'a> {
        ActivateBuilder::new(self.client)
    }

    /// Begins an Archive request with its optional identifier omitted.
    #[must_use]
    pub fn archive(self) -> ArchiveBuilder<'a> {
        ArchiveBuilder::new(self.client)
    }

    /// Begins a Recover request with its optional identifier omitted.
    #[must_use]
    pub fn recover(self) -> RecoverBuilder<'a> {
        RecoverBuilder::new(self.client)
    }

    /// Begins constructing a Revoke request with an explicit reason.
    #[must_use]
    pub fn revoke(self, reason: RevocationReason) -> RevokeBuilder<'a> {
        RevokeBuilder::new(self.client, reason)
    }

    /// Begins a Locate request with the caller's required Attributes criteria.
    #[must_use]
    pub fn locate(self, attributes: AttributeSet) -> LocateBuilder<'a> {
        LocateBuilder::new(self.client, attributes)
    }

    /// Begins a Re-key request with its optional identifier omitted.
    #[must_use]
    pub fn rekey(self) -> RekeyBuilder<'a> {
        RekeyBuilder::new(self.client)
    }

    /// Begins constructing a typed Re-key Key Pair request for a private key.
    #[must_use]
    pub fn rekey_key_pair(self) -> RekeyKeyPairBuilder<'a> {
        RekeyKeyPairBuilder::new(self.client)
    }

    /// Begins constructing a typed Re-certify request.
    #[must_use]
    pub fn recertify(self) -> RecertifyBuilder<'a> {
        RecertifyBuilder::new(self.client)
    }

    /// Begins constructing an Encrypt request with caller-supplied data.
    #[must_use]
    pub fn encrypt(self) -> EncryptBuilder<'a> {
        EncryptBuilder::new(self.client)
    }

    /// Begins constructing a Decrypt request with caller-supplied data.
    #[must_use]
    pub fn decrypt(self) -> DecryptBuilder<'a> {
        DecryptBuilder::new(self.client)
    }

    /// Begins constructing a Sign request.
    #[must_use]
    pub fn sign(self) -> SignBuilder<'a> {
        SignBuilder::new(self.client)
    }

    /// Begins constructing a Signature Verify request.
    #[must_use]
    pub fn signature_verify(self) -> SignatureVerifyBuilder<'a> {
        SignatureVerifyBuilder::new(self.client)
    }

    /// Begins constructing a MAC request.
    #[must_use]
    pub fn mac(self) -> MacBuilder<'a> {
        MacBuilder::new(self.client)
    }

    /// Begins constructing a MAC Verify request.
    #[must_use]
    pub fn mac_verify(self) -> MacVerifyBuilder<'a> {
        MacVerifyBuilder::new(self.client)
    }

    /// Begins constructing a Hash request with explicit Cryptographic Parameters.
    #[must_use]
    pub fn hash(self) -> HashBuilder<'a> {
        HashBuilder::new(self.client)
    }

    /// Begins an empty typed Ping request (OASIS KMIP v2.1 §6.1.36).
    #[must_use]
    pub fn ping(self) -> PingBuilder<'a> {
        PingBuilder::new(self.client)
    }

    /// Begins constructing a typed Certify request (§6.1.6).
    #[must_use]
    pub fn certify(self) -> CertifyBuilder<'a> {
        CertifyBuilder::new(self.client)
    }

    /// Begins constructing a typed Validate request (§6.1.57).
    #[must_use]
    pub fn validate(self) -> ValidateBuilder<'a> {
        ValidateBuilder::new(self.client)
    }

    /// Begins constructing a typed Cancel request for Pending correlation bytes (§6.1.5).
    #[must_use]
    pub fn cancel(self, correlation_value: SecretBytes) -> CancelBuilder<'a> {
        CancelBuilder::new(self.client, correlation_value)
    }

    /// Begins constructing a typed Poll request for Pending correlation bytes (§6.1.38).
    #[must_use]
    pub fn poll(self, correlation_value: SecretBytes) -> PollBuilder<'a> {
        PollBuilder::new(self.client, correlation_value)
    }

    /// Begins constructing a typed Process request for Pending correlation bytes (§6.1.39).
    #[must_use]
    pub fn process(self, correlation_value: SecretBytes) -> ProcessBuilder<'a> {
        ProcessBuilder::new(self.client, correlation_value)
    }

    /// Begins constructing a typed Query Asynchronous Requests request (§6.1.41).
    #[must_use]
    pub fn query_async_requests(self) -> QueryAsyncRequestsBuilder<'a> {
        QueryAsyncRequestsBuilder::new(self.client)
    }

    /// Begins constructing a typed Join Split Key request (§6.1.27).
    #[must_use]
    pub fn join_split_key(self) -> JoinSplitKeyBuilder<'a> {
        JoinSplitKeyBuilder::new(self.client)
    }

    /// Begins constructing a typed Import request (§6.1.25).
    #[must_use]
    pub fn import_object(self) -> ImportBuilder<'a> {
        ImportBuilder::new(self.client)
    }

    /// Begins constructing a typed Export request (§6.1.18).
    #[must_use]
    pub fn export(self) -> ExportBuilder<'a> {
        ExportBuilder::new(self.client)
    }

    /// Begins constructing a typed Log request (§6.1.29).
    #[must_use]
    pub fn log(self, log_message: &str) -> LogBuilder<'a> {
        LogBuilder::new(self.client, log_message)
    }

    /// Begins constructing a typed Login request (§6.1.30).
    #[must_use]
    pub fn login(self) -> LoginBuilder<'a> {
        LoginBuilder::new(self.client)
    }

    /// Begins constructing a typed Logout request (§6.1.31).
    #[must_use]
    pub fn logout(self, ticket: Ticket) -> LogoutBuilder<'a> {
        LogoutBuilder::new(self.client, ticket)
    }

    /// Begins constructing a typed Delegated Login request (§6.1.12).
    #[must_use]
    pub fn delegated_login(self) -> DelegatedLoginBuilder<'a> {
        DelegatedLoginBuilder::new(self.client)
    }

    /// Begins constructing a typed Set Endpoint Role request (§6.1.54).
    #[must_use]
    pub fn set_endpoint_role(self, role: EndpointRole) -> SetEndpointRoleBuilder<'a> {
        SetEndpointRoleBuilder::new(self.client, role)
    }

    /// Begins constructing a typed PKCS#11 forwarding request (§6.1.37).
    #[must_use]
    pub fn pkcs11(self) -> Pkcs11Builder<'a> {
        Pkcs11Builder::new(self.client)
    }

    /// Begins constructing a typed Interop request (§6.1.26).
    #[must_use]
    pub fn interop(self, interop_function: u32, interop_identifier: &str) -> InteropBuilder<'a> {
        InteropBuilder::new(self.client, interop_function, interop_identifier)
    }

    /// Begins constructing a typed Re-Provision request (§6.1.48).
    #[must_use]
    pub fn reprovision(self) -> ReProvisionBuilder<'a> {
        ReProvisionBuilder::new(self.client)
    }

    /// Begins constructing a typed Set Defaults request (§6.1.53).
    #[must_use]
    pub fn set_defaults(self) -> SetDefaultsBuilder<'a> {
        SetDefaultsBuilder::new(self.client)
    }

    /// Begins constructing a typed Set Constraints request (§6.1.52).
    #[must_use]
    pub fn set_constraints(self) -> SetConstraintsBuilder<'a> {
        SetConstraintsBuilder::new(self.client)
    }
}

impl Client {
    /// Returns the scoped fluent request namespace for common typed operations.
    #[must_use]
    pub fn request(&mut self) -> RequestBuilder<'_> {
        RequestBuilder::new(self)
    }
}
