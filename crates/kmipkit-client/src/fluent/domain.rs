//! Task-oriented domain namespaces and object-scoped handles (`KeyRef`) for `Client`.

use kmipkit_protocol::{
    AttributeSet, CreateCredentialObject, EndpointRole, RevocationReason, SecretBytes, Ticket,
};

use crate::Client;

use super::{
    ActivateBuilder, AddAttributeBuilder, AdjustAttributeBuilder, ArchiveBuilder, CancelBuilder,
    CertifyBuilder, CheckBuilder, CreateBuilder, CreateCredentialBuilder, CreateGroupBuilder,
    CreateKeyPairBuilder, CreateSplitKeyBuilder, CreateSymmetricKeyBuilder, CreateUserBuilder,
    DeactivateBuilder, DecapsulateBuilder, DecryptBuilder, DelegatedLoginBuilder,
    DeleteAttributeBuilder, DeriveKeyBuilder, DestroyBuilder, DiscoverVersionsBuilder,
    EncapsulateBuilder, EncryptBuilder, ExportBuilder, GetAttributeListBuilder,
    GetAttributesBuilder, GetBuilder, GetConstraintsBuilder, GetUsageAllocationBuilder,
    HashBuilder, ImportBuilder, InteropBuilder, JoinSplitKeyBuilder, LocateBuilder, LogBuilder,
    LoginBuilder, LogoutBuilder, MacBuilder, MacVerifyBuilder, ModifyAttributeBuilder,
    ObliterateBuilder, ObtainLeaseBuilder, PingBuilder, Pkcs11Builder, PollBuilder, ProcessBuilder,
    QueryAsyncRequestsBuilder, QueryBuilder, ReProvisionBuilder, RecertifyBuilder, RecoverBuilder,
    RegisterBuilder, RekeyBuilder, RekeyKeyPairBuilder, RequestBuilder, RevokeBuilder,
    RngRetrieveBuilder, RngSeedBuilder, SetAttributeBuilder, SetConstraintsBuilder,
    SetDefaultsBuilder, SetEndpointRoleBuilder, SignBuilder, SignatureVerifyBuilder,
    ValidateBuilder,
};

/// Domain namespace for key generation, import/export, derivation, split keys, and lookup.
pub struct KeysClient<'a> {
    client: &'a mut Client,
}

impl<'a> KeysClient<'a> {
    pub(crate) const fn new(client: &'a mut Client) -> Self {
        Self { client }
    }

    /// Returns the underlying unified `RequestBuilder`.
    #[must_use]
    pub fn raw(self) -> RequestBuilder<'a> {
        RequestBuilder::new(self.client)
    }

    /// Begins creating a symmetric key (use `.with_key_spec(KeySpec::Aes256)`).
    #[must_use]
    pub fn create_symmetric(self) -> CreateSymmetricKeyBuilder<'a> {
        CreateSymmetricKeyBuilder::new(Some(self.client))
    }

    /// Begins creating an asymmetric or PQC key pair (use `.with_key_spec(KeySpec::...)`).
    #[must_use]
    pub fn create_key_pair(self) -> CreateKeyPairBuilder<'a> {
        CreateKeyPairBuilder::new(Some(self.client))
    }

    /// Begins creating a generic KMIP object.
    #[must_use]
    pub fn create(self) -> CreateBuilder<'a> {
        CreateBuilder::new(self.client)
    }

    /// Begins creating a split key.
    #[must_use]
    pub fn create_split_key(self) -> CreateSplitKeyBuilder<'a> {
        CreateSplitKeyBuilder::new(self.client)
    }

    /// Begins joining split key parts.
    #[must_use]
    pub fn join_split_key(self) -> JoinSplitKeyBuilder<'a> {
        JoinSplitKeyBuilder::new(self.client)
    }

    /// Begins deriving a key from an existing key/secret (use `.with_kdf_spec(...)`).
    #[must_use]
    pub fn derive(self) -> DeriveKeyBuilder<'a> {
        DeriveKeyBuilder::new(self.client)
    }

    /// Begins registering an existing `ManagedObject` (from PEM/DER/raw bytes).
    #[must_use]
    pub fn register(self) -> RegisterBuilder<'a> {
        RegisterBuilder::new(self.client)
    }

    /// Begins importing an existing managed object with attributes.
    #[must_use]
    pub fn import_object(self) -> ImportBuilder<'a> {
        ImportBuilder::new(self.client)
    }

    /// Begins exporting a managed object.
    #[must_use]
    pub fn export(self) -> ExportBuilder<'a> {
        ExportBuilder::new(self.client)
    }

    /// Begins re-keying an existing symmetric key.
    #[must_use]
    pub fn rekey(self) -> RekeyBuilder<'a> {
        RekeyBuilder::new(self.client)
    }

    /// Begins re-keying an existing asymmetric key pair.
    #[must_use]
    pub fn rekey_key_pair(self) -> RekeyKeyPairBuilder<'a> {
        RekeyKeyPairBuilder::new(self.client)
    }

    /// Begins locating objects matching `attributes`.
    #[must_use]
    pub fn locate(self, attributes: AttributeSet) -> LocateBuilder<'a> {
        LocateBuilder::new(self.client, attributes)
    }

    /// Starts a paginated `Locate` iterator fetching pages of `page_size` objects.
    #[must_use]
    pub fn locate_iter(
        self,
        attributes: AttributeSet,
        page_size: i32,
    ) -> super::flows::LocatePaginator<'a> {
        super::flows::LocatePaginator::new(self.client, attributes, page_size)
    }

    /// Begins retrieving a managed object by ID.
    #[must_use]
    pub fn get(self, unique_identifier: &str) -> GetBuilder<'a> {
        GetBuilder::new(self.client).unique_identifier(unique_identifier)
    }

    /// Begins activating a key or managed object.
    #[must_use]
    pub fn activate(self, unique_identifier: &str) -> ActivateBuilder<'a> {
        ActivateBuilder::new(self.client).unique_identifier(unique_identifier)
    }

    /// Begins revoking a key or managed object.
    #[must_use]
    pub fn revoke(self, unique_identifier: &str, reason: RevocationReason) -> RevokeBuilder<'a> {
        RevokeBuilder::new(self.client, reason).unique_identifier(unique_identifier)
    }

    /// Begins destroying a key or managed object.
    #[must_use]
    pub fn destroy(self, unique_identifier: &str) -> DestroyBuilder<'a> {
        DestroyBuilder::new(self.client, Some(unique_identifier.into()))
    }
}

/// Domain namespace for cryptographic, hashing, RNG, and KMIP 3.0 KEM operations.
pub struct CryptoClient<'a> {
    client: &'a mut Client,
}

impl<'a> CryptoClient<'a> {
    pub(crate) const fn new(client: &'a mut Client) -> Self {
        Self { client }
    }

    /// Begins an encryption operation (use `.with_cipher_spec(CipherSpec::AesGcm)`).
    #[must_use]
    pub fn encrypt(self) -> EncryptBuilder<'a> {
        EncryptBuilder::new(self.client)
    }

    /// Begins a decryption operation.
    #[must_use]
    pub fn decrypt(self) -> DecryptBuilder<'a> {
        DecryptBuilder::new(self.client)
    }

    /// Begins a digital signature operation (use `.with_signature_spec(SignatureSpec::EcdsaSha256)`).
    #[must_use]
    pub fn sign(self) -> SignBuilder<'a> {
        SignBuilder::new(self.client)
    }

    /// Begins verifying a digital signature.
    #[must_use]
    pub fn signature_verify(self) -> SignatureVerifyBuilder<'a> {
        SignatureVerifyBuilder::new(self.client)
    }

    /// Begins a MAC generation operation (use `.with_mac_spec(MacSpec::HmacSha256)`).
    #[must_use]
    pub fn mac(self) -> MacBuilder<'a> {
        MacBuilder::new(self.client)
    }

    /// Begins verifying a MAC tag.
    #[must_use]
    pub fn mac_verify(self) -> MacVerifyBuilder<'a> {
        MacVerifyBuilder::new(self.client)
    }

    /// Begins computing a cryptographic digest (use `.with_hash_algorithm(HashAlgorithm::Sha256)`).
    #[must_use]
    pub fn hash(self) -> HashBuilder<'a> {
        HashBuilder::new(self.client)
    }

    /// Begins retrieving random bytes from the KMS RNG.
    #[must_use]
    pub fn rng_retrieve(self, data_length: i32) -> RngRetrieveBuilder<'a> {
        RngRetrieveBuilder::new(self.client).data_length(data_length)
    }

    /// Begins seeding the KMS RNG.
    #[must_use]
    pub fn rng_seed(self, seed_data: SecretBytes) -> RngSeedBuilder<'a> {
        RngSeedBuilder::new(self.client).data(seed_data)
    }

    /// Begins a KMIP 3.0 KEM encapsulation using a public KEM key ID.
    #[must_use]
    pub fn encapsulate(self, public_key_id: &str) -> EncapsulateBuilder<'a> {
        EncapsulateBuilder::new(self.client, public_key_id.into())
    }

    /// Begins a KMIP 3.0 KEM decapsulation using a private KEM key ID and ciphertext bytes.
    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    pub fn decapsulate(
        self,
        private_key_id: &str,
        ciphertext: SecretBytes,
    ) -> DecapsulateBuilder<'a> {
        let bytes = ciphertext.with_bytes(<[u8]>::to_vec);
        DecapsulateBuilder::new(self.client, private_key_id.into(), bytes)
    }
}

/// Domain namespace for X.509 certificate issuance, re-certification, and validation.
pub struct CertificatesClient<'a> {
    client: &'a mut Client,
}

impl<'a> CertificatesClient<'a> {
    pub(crate) const fn new(client: &'a mut Client) -> Self {
        Self { client }
    }

    /// Begins certifying a certificate request / public key (`Certify`).
    #[must_use]
    pub fn certify(self) -> CertifyBuilder<'a> {
        CertifyBuilder::new(self.client)
    }

    /// Begins re-certifying an existing certificate (`Recertify`).
    #[must_use]
    pub fn recertify(self) -> RecertifyBuilder<'a> {
        RecertifyBuilder::new(self.client)
    }

    /// Begins validating one or more certificates (`Validate`).
    #[must_use]
    pub fn validate(self) -> ValidateBuilder<'a> {
        ValidateBuilder::new(self.client)
    }

    /// Begins retrieving a stored certificate.
    #[must_use]
    pub fn get(self, certificate_id: &str) -> GetBuilder<'a> {
        GetBuilder::new(self.client).unique_identifier(certificate_id)
    }

    /// Begins locating certificates matching attributes.
    #[must_use]
    pub fn locate(self, attributes: AttributeSet) -> LocateBuilder<'a> {
        LocateBuilder::new(self.client, attributes)
    }

    /// Begins revoking a certificate.
    #[must_use]
    pub fn revoke(self, certificate_id: &str, reason: RevocationReason) -> RevokeBuilder<'a> {
        RevokeBuilder::new(self.client, reason).unique_identifier(certificate_id)
    }

    /// Begins destroying a certificate.
    #[must_use]
    pub fn destroy(self, certificate_id: &str) -> DestroyBuilder<'a> {
        DestroyBuilder::new(self.client, Some(certificate_id.into()))
    }
}

/// Domain namespace for secret data, leases, archival, and usage allocation.
pub struct SecretsClient<'a> {
    client: &'a mut Client,
}

impl<'a> SecretsClient<'a> {
    pub(crate) const fn new(client: &'a mut Client) -> Self {
        Self { client }
    }

    /// Begins registering secret data or opaque objects.
    #[must_use]
    pub fn register(self) -> RegisterBuilder<'a> {
        RegisterBuilder::new(self.client)
    }

    /// Begins retrieving a secret object.
    #[must_use]
    pub fn get(self, secret_id: &str) -> GetBuilder<'a> {
        GetBuilder::new(self.client).unique_identifier(secret_id)
    }

    /// Begins archiving an object (`Archive`).
    #[must_use]
    pub fn archive(self, object_id: &str) -> ArchiveBuilder<'a> {
        ArchiveBuilder::new(self.client).unique_identifier(object_id)
    }

    /// Begins recovering an archived object (`Recover`).
    #[must_use]
    pub fn recover(self, object_id: &str) -> RecoverBuilder<'a> {
        RecoverBuilder::new(self.client).unique_identifier(object_id)
    }

    /// Begins obtaining a lease on an object (`ObtainLease`).
    #[must_use]
    pub fn obtain_lease(self, object_id: &str) -> ObtainLeaseBuilder<'a> {
        ObtainLeaseBuilder::new(self.client).unique_identifier(object_id)
    }

    /// Begins getting usage allocation (`GetUsageAllocation`).
    #[must_use]
    pub fn get_usage_allocation(self, object_id: &str) -> GetUsageAllocationBuilder<'a> {
        GetUsageAllocationBuilder::new(self.client).unique_identifier(object_id)
    }

    /// Begins destroying a secret object.
    #[must_use]
    pub fn destroy(self, secret_id: &str) -> DestroyBuilder<'a> {
        DestroyBuilder::new(self.client, Some(secret_id.into()))
    }
}

/// Domain namespace for reading, adding, modifying, setting, and deleting object attributes.
pub struct AttributesClient<'a> {
    client: &'a mut Client,
}

impl<'a> AttributesClient<'a> {
    pub(crate) const fn new(client: &'a mut Client) -> Self {
        Self { client }
    }

    /// Begins reading specific attributes of an object (`GetAttributes`).
    #[must_use]
    pub fn get(self, unique_identifier: &str) -> GetAttributesBuilder<'a> {
        GetAttributesBuilder::new(self.client).unique_identifier(unique_identifier)
    }

    /// Begins listing attribute names defined on an object (`GetAttributeList`).
    #[must_use]
    pub fn get_list(self, unique_identifier: &str) -> GetAttributeListBuilder<'a> {
        GetAttributeListBuilder::new(self.client).unique_identifier(unique_identifier)
    }

    /// Begins adding a new attribute to an object (`AddAttribute`).
    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn add(self, unique_identifier: &str) -> AddAttributeBuilder<'a> {
        AddAttributeBuilder::new(Some(self.client)).unique_identifier(unique_identifier)
    }

    /// Begins setting/replacing an attribute on an object (`SetAttribute`).
    #[must_use]
    pub fn set(self, unique_identifier: &str) -> SetAttributeBuilder<'a> {
        SetAttributeBuilder::new(Some(self.client)).unique_identifier(unique_identifier)
    }

    /// Begins modifying an existing attribute value (`ModifyAttribute`).
    #[must_use]
    pub fn modify(self, unique_identifier: &str) -> ModifyAttributeBuilder<'a> {
        ModifyAttributeBuilder::new(self.client).unique_identifier(unique_identifier)
    }

    /// Begins adjusting an existing numeric/date attribute (`AdjustAttribute`).
    #[must_use]
    pub fn adjust(self, unique_identifier: &str) -> AdjustAttributeBuilder<'a> {
        AdjustAttributeBuilder::new(self.client).unique_identifier(unique_identifier)
    }

    /// Begins deleting an attribute from an object (`DeleteAttribute`).
    #[must_use]
    pub fn delete(self, unique_identifier: &str) -> DeleteAttributeBuilder<'a> {
        DeleteAttributeBuilder::new(Some(self.client)).unique_identifier(unique_identifier)
    }

    /// Begins retrieving constraints (`GetConstraints`).
    #[must_use]
    pub fn get_constraints(self) -> GetConstraintsBuilder<'a> {
        GetConstraintsBuilder::new(self.client)
    }

    /// Begins updating server constraints (`SetConstraints`).
    #[must_use]
    pub fn set_constraints(self) -> SetConstraintsBuilder<'a> {
        SetConstraintsBuilder::new(self.client)
    }

    /// Begins updating server default attribute policy (`SetDefaults`).
    #[must_use]
    pub fn set_defaults(self) -> SetDefaultsBuilder<'a> {
        SetDefaultsBuilder::new(self.client)
    }
}

/// Domain namespace for authentication sessions, endpoint roles, and KMIP 3.0 users/groups/credentials.
pub struct SessionsClient<'a> {
    client: &'a mut Client,
}

impl<'a> SessionsClient<'a> {
    pub(crate) const fn new(client: &'a mut Client) -> Self {
        Self { client }
    }

    /// Begins creating a session ticket (`Login`).
    #[must_use]
    pub fn login(self) -> LoginBuilder<'a> {
        LoginBuilder::new(self.client)
    }

    /// Begins terminating an authenticated ticket (`Logout`).
    #[must_use]
    pub fn logout(self, ticket: Ticket) -> LogoutBuilder<'a> {
        LogoutBuilder::new(self.client, ticket)
    }

    /// Begins performing a delegated login (`DelegatedLogin`).
    #[must_use]
    pub fn delegated_login(self) -> DelegatedLoginBuilder<'a> {
        DelegatedLoginBuilder::new(self.client)
    }

    /// Begins inverting or configuring endpoint role (`SetEndpointRole`).
    #[must_use]
    pub fn set_endpoint_role(self, role: EndpointRole) -> SetEndpointRoleBuilder<'a> {
        SetEndpointRoleBuilder::new(self.client, role)
    }

    /// Begins creating a KMIP 3.0 user object (`CreateUser`).
    #[must_use]
    pub fn create_user(self, attributes: AttributeSet) -> CreateUserBuilder<'a> {
        CreateUserBuilder::new(self.client, attributes)
    }

    /// Begins creating a KMIP 3.0 group object (`CreateGroup`).
    #[must_use]
    pub fn create_group(self, attributes: AttributeSet) -> CreateGroupBuilder<'a> {
        CreateGroupBuilder::new(self.client, attributes)
    }

    /// Begins creating a KMIP 3.0 system credential object (`CreateCredential`).
    #[must_use]
    pub fn create_credential(
        self,
        attributes: AttributeSet,
        object: CreateCredentialObject,
    ) -> CreateCredentialBuilder<'a> {
        CreateCredentialBuilder::new(self.client, attributes, object)
    }
}

/// Domain namespace for server capabilities, health checks, auditing, and KMIP 3.0 decommissioning.
pub struct AdminClient<'a> {
    client: &'a mut Client,
}

impl<'a> AdminClient<'a> {
    pub(crate) const fn new(client: &'a mut Client) -> Self {
        Self { client }
    }

    /// Begins sending a `Ping` request.
    #[must_use]
    pub fn ping(self) -> PingBuilder<'a> {
        PingBuilder::new(self.client)
    }

    /// Begins querying server capabilities (`Query`).
    #[must_use]
    pub fn query(self) -> QueryBuilder<'a> {
        QueryBuilder::new(self.client)
    }

    /// Begins discovering supported KMIP protocol versions (`DiscoverVersions`).
    #[must_use]
    pub fn discover_versions(self) -> DiscoverVersionsBuilder<'a> {
        DiscoverVersionsBuilder::new(self.client)
    }

    /// Begins checking object usage/cryptographic permissions (`Check`).
    #[must_use]
    pub fn check(self) -> CheckBuilder<'a> {
        CheckBuilder::new(self.client)
    }

    /// Begins writing a server audit log entry (`Log`).
    #[must_use]
    pub fn log(self, log_message: &str) -> LogBuilder<'a> {
        LogBuilder::new(self.client, log_message)
    }

    /// Begins forwarding a PKCS#11 operation (`PKCS#11`).
    #[must_use]
    pub fn pkcs11(self) -> Pkcs11Builder<'a> {
        Pkcs11Builder::new(self.client)
    }

    /// Begins an interoperability check request (`Interop`).
    #[must_use]
    pub fn interop(self, interop_function: u32, interop_identifier: &str) -> InteropBuilder<'a> {
        InteropBuilder::new(self.client, interop_function, interop_identifier)
    }

    /// Begins re-provisioning an object (`ReProvision`).
    #[must_use]
    pub fn reprovision(self) -> ReProvisionBuilder<'a> {
        ReProvisionBuilder::new(self.client)
    }

    /// Begins deactivating a KMIP 3.0 object (`Deactivate`).
    #[must_use]
    pub fn deactivate(self, unique_identifier: &str) -> DeactivateBuilder<'a> {
        DeactivateBuilder::new(self.client, unique_identifier.into())
    }

    /// Begins obliterating a KMIP 3.0 object (`Obliterate`).
    #[must_use]
    pub fn obliterate(self, unique_identifier: &str) -> ObliterateBuilder<'a> {
        ObliterateBuilder::new(self.client, unique_identifier.into())
    }
}

/// Domain namespace for KMIP asynchronous job polling, processing, and cancellation.
pub struct AsyncOpsClient<'a> {
    client: &'a mut Client,
}

impl<'a> AsyncOpsClient<'a> {
    pub(crate) const fn new(client: &'a mut Client) -> Self {
        Self { client }
    }

    /// Begins polling an asynchronous request (`Poll`).
    #[must_use]
    pub fn poll(self, correlation_value: SecretBytes) -> PollBuilder<'a> {
        PollBuilder::new(self.client, correlation_value)
    }

    /// Begins cancelling an asynchronous request (`Cancel`).
    #[must_use]
    pub fn cancel(self, correlation_value: SecretBytes) -> CancelBuilder<'a> {
        CancelBuilder::new(self.client, correlation_value)
    }

    /// Begins triggering asynchronous queue processing (`Process`).
    #[must_use]
    pub fn process(self, correlation_value: SecretBytes) -> ProcessBuilder<'a> {
        ProcessBuilder::new(self.client, correlation_value)
    }

    /// Begins querying pending asynchronous request metadata (`QueryAsynchronousRequests`).
    #[must_use]
    pub fn query(self) -> QueryAsyncRequestsBuilder<'a> {
        QueryAsyncRequestsBuilder::new(self.client)
    }
}

/// Fluent object-scoped handle returned by `client.key(id)`, pre-binding `UniqueIdentifier` on all operations.
pub struct KeyRef<'a> {
    client: &'a mut Client,
    unique_identifier: String,
}

impl<'a> KeyRef<'a> {
    pub(crate) fn new(client: &'a mut Client, unique_identifier: &str) -> Self {
        Self {
            client,
            unique_identifier: unique_identifier.to_owned(),
        }
    }

    /// Returns the bound Unique Identifier text.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.unique_identifier
    }

    /// Retrieves the object (`Get`).
    #[must_use]
    pub fn get(self) -> GetBuilder<'a> {
        GetBuilder::new(self.client).unique_identifier(self.unique_identifier.as_str())
    }

    /// Activates the object (`Activate`).
    #[must_use]
    pub fn activate(self) -> ActivateBuilder<'a> {
        ActivateBuilder::new(self.client).unique_identifier(self.unique_identifier.as_str())
    }

    /// Deactivates a KMIP 3.0 object (`Deactivate`).
    #[must_use]
    pub fn deactivate(self) -> DeactivateBuilder<'a> {
        DeactivateBuilder::new(self.client, self.unique_identifier.as_str().into())
    }

    /// Obliterates a KMIP 3.0 object (`Obliterate`).
    #[must_use]
    pub fn obliterate(self) -> ObliterateBuilder<'a> {
        ObliterateBuilder::new(self.client, self.unique_identifier.as_str().into())
    }

    /// Revokes the object (`Revoke`).
    #[must_use]
    pub fn revoke(self, reason: RevocationReason) -> RevokeBuilder<'a> {
        RevokeBuilder::new(self.client, reason).unique_identifier(self.unique_identifier.as_str())
    }

    /// Destroys the object (`Destroy`).
    #[must_use]
    pub fn destroy(self) -> DestroyBuilder<'a> {
        DestroyBuilder::new(self.client, Some(self.unique_identifier.as_str().into()))
    }

    /// Archives the object (`Archive`).
    #[must_use]
    pub fn archive(self) -> ArchiveBuilder<'a> {
        ArchiveBuilder::new(self.client).unique_identifier(self.unique_identifier.as_str())
    }

    /// Recovers the object (`Recover`).
    #[must_use]
    pub fn recover(self) -> RecoverBuilder<'a> {
        RecoverBuilder::new(self.client).unique_identifier(self.unique_identifier.as_str())
    }

    /// Re-keys the symmetric key (`Rekey`).
    #[must_use]
    pub fn rekey(self) -> RekeyBuilder<'a> {
        RekeyBuilder::new(self.client).unique_identifier(self.unique_identifier.as_str())
    }

    /// Re-keys the asymmetric key pair using this handle as the private key ID (`RekeyKeyPair`).
    #[must_use]
    pub fn rekey_key_pair(self) -> RekeyKeyPairBuilder<'a> {
        RekeyKeyPairBuilder::new(self.client)
            .private_key_unique_identifier(self.unique_identifier.as_str().into())
    }

    /// Exports the object (`Export`).
    #[must_use]
    pub fn export(self) -> ExportBuilder<'a> {
        ExportBuilder::new(self.client).unique_identifier(self.unique_identifier.as_str())
    }

    /// Obtains a lease on the object (`ObtainLease`).
    #[must_use]
    pub fn obtain_lease(self) -> ObtainLeaseBuilder<'a> {
        ObtainLeaseBuilder::new(self.client).unique_identifier(self.unique_identifier.as_str())
    }

    /// Encrypts data using this key (`Encrypt`).
    #[must_use]
    pub fn encrypt(self) -> EncryptBuilder<'a> {
        EncryptBuilder::new(self.client).unique_identifier(self.unique_identifier.as_str())
    }

    /// Decrypts data using this key (`Decrypt`).
    #[must_use]
    pub fn decrypt(self) -> DecryptBuilder<'a> {
        DecryptBuilder::new(self.client).unique_identifier(self.unique_identifier.as_str())
    }

    /// Signs data using this key (`Sign`).
    #[must_use]
    pub fn sign(self) -> SignBuilder<'a> {
        SignBuilder::new(self.client).unique_identifier(self.unique_identifier.as_str())
    }

    /// Verifies a signature using this key (`SignatureVerify`).
    #[must_use]
    pub fn verify(self) -> SignatureVerifyBuilder<'a> {
        SignatureVerifyBuilder::new(self.client).unique_identifier(self.unique_identifier.as_str())
    }

    /// Alias for [`Self::verify`].
    #[must_use]
    pub fn signature_verify(self) -> SignatureVerifyBuilder<'a> {
        self.verify()
    }

    /// Generates a MAC using this key (`MAC`).
    #[must_use]
    pub fn mac(self) -> MacBuilder<'a> {
        MacBuilder::new(self.client).unique_identifier(self.unique_identifier.as_str())
    }

    /// Verifies a MAC tag using this key (`MACVerify`).
    #[must_use]
    pub fn mac_verify(self) -> MacVerifyBuilder<'a> {
        MacVerifyBuilder::new(self.client).unique_identifier(self.unique_identifier.as_str())
    }

    /// Encapsulates a shared secret using this KMIP 3.0 KEM public key (`Encapsulate`).
    #[must_use]
    pub fn encapsulate(self) -> EncapsulateBuilder<'a> {
        EncapsulateBuilder::new(self.client, self.unique_identifier.as_str().into())
    }

    /// Decapsulates ciphertext using this KMIP 3.0 KEM private key (`Decapsulate`).
    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    pub fn decapsulate(self, ciphertext: SecretBytes) -> DecapsulateBuilder<'a> {
        let bytes = ciphertext.with_bytes(<[u8]>::to_vec);
        DecapsulateBuilder::new(self.client, self.unique_identifier.as_str().into(), bytes)
    }

    /// Reads object attributes (`GetAttributes`).
    #[must_use]
    pub fn attributes(self) -> GetAttributesBuilder<'a> {
        GetAttributesBuilder::new(self.client).unique_identifier(self.unique_identifier.as_str())
    }

    /// Lists attribute names (`GetAttributeList`).
    #[must_use]
    pub fn attribute_list(self) -> GetAttributeListBuilder<'a> {
        GetAttributeListBuilder::new(self.client).unique_identifier(self.unique_identifier.as_str())
    }

    /// Adds a new attribute (`AddAttribute`).
    #[must_use]
    pub fn add_attribute(self) -> AddAttributeBuilder<'a> {
        AddAttributeBuilder::new(Some(self.client))
            .unique_identifier(self.unique_identifier.as_str())
    }

    /// Sets an attribute (`SetAttribute`).
    #[must_use]
    pub fn set_attribute(self) -> SetAttributeBuilder<'a> {
        SetAttributeBuilder::new(Some(self.client))
            .unique_identifier(self.unique_identifier.as_str())
    }

    /// Modifies an existing attribute (`ModifyAttribute`).
    #[must_use]
    pub fn modify_attribute(self) -> ModifyAttributeBuilder<'a> {
        ModifyAttributeBuilder::new(self.client).unique_identifier(self.unique_identifier.as_str())
    }

    /// Adjusts a numeric/date attribute (`AdjustAttribute`).
    #[must_use]
    pub fn adjust_attribute(self) -> AdjustAttributeBuilder<'a> {
        AdjustAttributeBuilder::new(self.client).unique_identifier(self.unique_identifier.as_str())
    }

    /// Deletes an attribute (`DeleteAttribute`).
    #[must_use]
    pub fn delete_attribute(self) -> DeleteAttributeBuilder<'a> {
        DeleteAttributeBuilder::new(Some(self.client))
            .unique_identifier(self.unique_identifier.as_str())
    }
}

impl Client {
    /// Returns the domain namespace for key management operations.
    #[must_use]
    pub fn keys(&mut self) -> KeysClient<'_> {
        KeysClient::new(self)
    }

    /// Returns the domain namespace for cryptographic, hash, RNG, and KEM operations.
    #[must_use]
    pub fn crypto(&mut self) -> CryptoClient<'_> {
        CryptoClient::new(self)
    }

    /// Returns the domain namespace for X.509 certificate lifecycle and validation operations.
    #[must_use]
    pub fn certificates(&mut self) -> CertificatesClient<'_> {
        CertificatesClient::new(self)
    }

    /// Returns the domain namespace for secret data, archival, and lease operations.
    #[must_use]
    pub fn secrets(&mut self) -> SecretsClient<'_> {
        SecretsClient::new(self)
    }

    /// Returns the domain namespace for object attribute and server policy operations.
    #[must_use]
    pub fn attributes(&mut self) -> AttributesClient<'_> {
        AttributesClient::new(self)
    }

    /// Returns the domain namespace for authentication sessions and KMIP 3.0 identity objects.
    #[must_use]
    pub fn sessions(&mut self) -> SessionsClient<'_> {
        SessionsClient::new(self)
    }

    /// Returns the domain namespace for diagnostics, auditing, and administrative operations.
    #[must_use]
    pub fn admin(&mut self) -> AdminClient<'_> {
        AdminClient::new(self)
    }

    /// Returns the domain namespace for asynchronous operation tracking (`Poll`/`Cancel`/`Process`).
    #[must_use]
    pub fn async_ops(&mut self) -> AsyncOpsClient<'_> {
        AsyncOpsClient::new(self)
    }

    /// Returns an object-scoped handle binding `unique_identifier` across all object/crypto operations.
    #[must_use]
    pub fn key(&mut self, unique_identifier: &str) -> KeyRef<'_> {
        KeyRef::new(self, unique_identifier)
    }
}
