//! Task-oriented, fluent builders over the existing typed KMIP requests.

use std::error::Error;
use std::fmt;
use std::ops::BitOr;

use kmipkit_protocol::attribute::typed;
use kmipkit_protocol::named_values_generated::values::bitmask::cryptographic_usage_mask;
use kmipkit_protocol::{
    ActivateRequest, AddAttributeRequest, AdjustAttributeRequest, AdjustmentType, ArchiveRequest,
    Attribute, AttributeDefinition, AttributeReference, AttributeSet, AttributeSetError,
    CheckRequest, CreateCredentialObject, CreateCredentialRequest, CreateGroupRequest,
    CreateKeyPairRequest, CreateRequest, CreateSplitKeyRequest, CreateUserRequest,
    CryptographicAlgorithm, CryptographicParameters, CurrentAttribute, DeactivateRequest,
    DecapsulateRequest, DecryptRequest, DeleteAttributeRequest, DerivationMethod,
    DerivationParameters, DeriveKeyRequest, DestroyRequest, DiscoverVersionsRequest,
    EncapsulateRequest, EncryptRequest, GetAttributeListRequest, GetAttributesRequest,
    GetConstraintsRequest, GetRequest, GetUsageAllocationRequest, HashRequest, KeyCompressionType,
    KeyFormatType, KeyWrapType, LocateRequest, MacRequest, MacVerifyRequest, ManagedObject,
    ModifyAttributeRequest, NewAttribute, ObjectGroupMember, ObjectType, ObliterateRequest,
    ObtainLeaseRequest, OpaqueObjectContext, OperationData, PingRequest, ProtocolVersion,
    QueryFunction, QueryRequest, RecertifyRequest, RecoverRequest, RegisterRequest,
    RekeyKeyPairRequest, RekeyRequest, RevocationReason, RevokeRequest, RngRetrieveRequest,
    RngSeedRequest, SecretBytes, SetAttributeRequest, SignRequest, SignatureVerifyRequest,
    SplitKeyMethod, StorageStatusMask, UniqueIdentifier, ValidateKmipRequest, ValidationReport,
};
use kmipkit_transport::{RequestDeliveryState, RequestOptions};
use kmipkit_ttlv::{Item, KmipTag, RawTag, Structure, Tag, Value, ValueView};

use crate::{
    Client, ClientBatchItem, ClientBatchItemResponse, ClientCauseCategory, ClientError,
    ClientRequest,
};

macro_rules! fluent_option_field_setters {
    ($option_doc:literal; $($options_path:ident).+; $($indicator_path:ident).+; const) => {
        #[doc = $option_doc]
        #[must_use]
        pub fn with_options(mut self, options: RequestOptions) -> Self {
            self.$($options_path).+ = Some(options);
            self
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
        pub fn with_options(mut self, options: RequestOptions) -> Self {
            self.$($options_path).+ = Some(options);
            self
        }

        /// Sets the Request Header Asynchronous Indicator for this exchange.
        #[must_use]
        pub fn with_asynchronous_indicator(mut self, value: u32) -> Self {
            self.$($indicator_path).+ = Some(value);
            self
        }
    };
}

const SYMMETRIC_KEY_OBJECT_TYPE: u32 = 2;
const MESSAGE_EXTENSION_TAG_START: u32 = 0x0054_0000;
const MESSAGE_EXTENSION_TAG_END: u32 = 0x0054_FFFF;
const NAME_TYPE_TAG: u32 = KmipTag::NameType.as_u32();
const NAME_VALUE_TAG: u32 = KmipTag::NameValue.as_u32();
const NAME_TYPE_UNINTERPRETED_TEXT_STRING: u32 = 1;
const VENDOR_ATTRIBUTE_TAG: u32 = KmipTag::Attribute.as_u32();
const VENDOR_IDENTIFICATION_TAG: u32 = KmipTag::VendorIdentification.as_u32();
const ATTRIBUTE_NAME_TAG: u32 = KmipTag::AttributeName.as_u32();
const ATTRIBUTE_VALUE_TAG: u32 = KmipTag::AttributeValue.as_u32();

/// KMIP Cryptographic Usage Mask bits encoded as a signed TTLV Integer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CryptographicUsageMask(i32);

impl CryptographicUsageMask {
    /// KMIP Sign usage flag.
    pub const SIGN: Self = Self(cryptographic_usage_mask::SIGN.cast_signed());
    /// KMIP Verify usage flag.
    pub const VERIFY: Self = Self(cryptographic_usage_mask::VERIFY.cast_signed());
    /// KMIP Encrypt usage flag.
    pub const ENCRYPT: Self = Self(cryptographic_usage_mask::ENCRYPT.cast_signed());
    /// KMIP Decrypt usage flag.
    pub const DECRYPT: Self = Self(cryptographic_usage_mask::DECRYPT.cast_signed());
    /// KMIP Wrap Key usage flag.
    pub const WRAP_KEY: Self = Self(cryptographic_usage_mask::WRAP_KEY.cast_signed());
    /// KMIP Unwrap Key usage flag.
    pub const UNWRAP_KEY: Self = Self(cryptographic_usage_mask::UNWRAP_KEY.cast_signed());
    /// KMIP Mac Generate usage flag.
    pub const MAC_GENERATE: Self = Self(cryptographic_usage_mask::MAC_GENERATE.cast_signed());
    /// KMIP Mac Verify usage flag.
    pub const MAC_VERIFY: Self = Self(cryptographic_usage_mask::MAC_VERIFY.cast_signed());
    /// KMIP Derive Key usage flag.
    pub const DERIVE_KEY: Self = Self(cryptographic_usage_mask::DERIVE_KEY.cast_signed());
    /// KMIP Key Agreement usage flag.
    pub const KEY_AGREEMENT: Self = Self(cryptographic_usage_mask::KEY_AGREEMENT.cast_signed());
    /// KMIP Certificate Sign usage flag.
    pub const CERTIFICATE_SIGN: Self =
        Self(cryptographic_usage_mask::CERTIFICATE_SIGN.cast_signed());
    /// KMIP Crl Sign usage flag.
    pub const CRL_SIGN: Self = Self(cryptographic_usage_mask::CRL_SIGN.cast_signed());
    /// KMIP Authenticate usage flag.
    pub const AUTHENTICATE: Self = Self(cryptographic_usage_mask::AUTHENTICATE.cast_signed());
    /// KMIP Unrestricted usage flag.
    pub const UNRESTRICTED: Self = Self(cryptographic_usage_mask::UNRESTRICTED.cast_signed());
    /// KMIP Fpe Encrypt usage flag.
    pub const FPE_ENCRYPT: Self = Self(cryptographic_usage_mask::FPE_ENCRYPT.cast_signed());
    /// KMIP Fpe Decrypt usage flag.
    pub const FPE_DECRYPT: Self = Self(cryptographic_usage_mask::FPE_DECRYPT.cast_signed());

    /// Preserves an exact signed TTLV Integer bit pattern.
    #[must_use]
    pub const fn from_raw(raw: i32) -> Self {
        Self(raw)
    }

    /// Returns the exact signed TTLV Integer bit pattern.
    #[must_use]
    pub const fn raw(self) -> i32 {
        self.0
    }
}

impl BitOr for CryptographicUsageMask {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

/// A payload-free reason a fluent request could not be built locally.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FluentBuildError {
    /// Add Attribute is missing its required New Attribute.
    MissingAddAttributeValue,
    /// Modify Attribute is missing its required New Attribute.
    MissingModifyAttributeValue,
    /// Set Attribute is missing its required New Attribute.
    MissingSetAttributeValue,
    /// Adjust Attribute is missing its required Attribute Reference.
    MissingAdjustAttributeReference,
    /// Adjust Attribute is missing its required Adjustment Type.
    MissingAdjustAttributeType,
    /// Discover Versions was given an empty or uncatalogued protocol offer.
    InvalidProtocolVersions,
    /// Get Usage Allocation is missing its required Usage Limits Count.
    MissingGetUsageAllocationCount,

    /// Create Symmetric Key is missing Cryptographic Algorithm.
    MissingCryptographicAlgorithm,
    /// Create Symmetric Key is missing Cryptographic Length.
    MissingCryptographicLength,
    /// Create Symmetric Key is missing Cryptographic Usage Mask.
    MissingCryptographicUsageMask,
    /// Create Key Pair is missing Cryptographic Algorithm.
    MissingCreateKeyPairAlgorithm,
    /// Create Key Pair is missing Cryptographic Length.
    MissingCreateKeyPairLength,
    /// Create Split Key is missing Object Type.
    MissingCreateSplitKeyObjectType,
    /// Create is missing Object Type.
    MissingCreateObjectType,
    /// Derive Key is missing Object Type.
    MissingDeriveKeyObjectType,
    /// Derive Key requires at least one source Unique Identifier.
    MissingDeriveKeyUniqueIdentifier,
    /// Derive Key is missing the caller-selected Derivation Method.
    MissingDeriveKeyMethod,
    /// Derive Key is missing the caller-selected Derivation Parameters.
    MissingDeriveKeyParameters,
    /// Derive Key is missing the required output Attributes Structure.
    MissingDeriveKeyAttributes,
    /// Create Split Key is missing Split Key Method.
    MissingCreateSplitKeyMethod,
    /// Create Split Key is missing Split Key Parts.
    MissingCreateSplitKeyParts,
    /// Create Split Key is missing Split Key Threshold.
    MissingCreateSplitKeyThreshold,
    /// Hash is missing its caller-selected Cryptographic Parameters.
    MissingHashCryptographicParameters,
    /// Get Usage Allocation is missing Usage Limits Count.
    MissingUsageLimitsCount,
    /// RNG Retrieve is missing the caller-selected Data Length.
    MissingRngRetrieveDataLength,
    /// RNG Seed is missing its caller-supplied secret Data.
    MissingRngSeedData,
    /// Register is missing its object.
    MissingRegisterObject,
    /// Register's object or context is invalid.
    InvalidRegisterObject,
    /// Set Attribute is missing its required New Attribute.
    MissingSetAttributeAttribute,
    /// Add Attribute is missing its required New Attribute.
    MissingAddAttributeAttribute,
    /// A catalogued attribute has an invalid TTLV type or structure.
    InvalidAttributeValue,
    /// Legacy compatibility error; explicit extension Items are retained as
    /// raw TTLV Object Attribute entries by current fluent builders.
    MessageExtensionTagNotObjectAttribute,
    /// More than one expert attribute requested the same final position.
    DuplicateAttributePosition,
    /// An expert attribute position is outside the final Object Attributes list.
    AttributePositionOutOfBounds,
    /// A typed request failed its existing validation rules.
    InvalidRequest,
    /// A local TTLV value could not be copied into the fluent request.
    InvalidTtlvValue,
    /// A detached builder cannot execute until it is attached to a client.
    MissingClientForExecution,
}

impl fmt::Display for FluentBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::MissingAddAttributeValue => "Add Attribute requires a New Attribute",
            Self::MissingModifyAttributeValue => "Modify Attribute requires a New Attribute",
            Self::MissingSetAttributeValue => "Set Attribute requires a New Attribute",
            Self::MissingAdjustAttributeReference => {
                "Adjust Attribute requires an Attribute Reference"
            }
            Self::MissingAdjustAttributeType => "Adjust Attribute requires an Adjustment Type",
            Self::InvalidProtocolVersions => {
                "fluent Discover Versions offers are empty or unsupported"
            }
            Self::MissingGetUsageAllocationCount => {
                "Get Usage Allocation requires a Usage Limits Count"
            }

            Self::MissingCryptographicAlgorithm => {
                "Create Symmetric Key requires Cryptographic Algorithm"
            }
            Self::MissingCryptographicLength => {
                "Create Symmetric Key requires Cryptographic Length"
            }
            Self::MissingCryptographicUsageMask => {
                "Create Symmetric Key requires Cryptographic Usage Mask"
            }
            Self::MissingCreateKeyPairAlgorithm => {
                "Create Key Pair requires Cryptographic Algorithm"
            }
            Self::MissingCreateKeyPairLength => "Create Key Pair requires Cryptographic Length",
            Self::MissingCreateSplitKeyObjectType => "Create Split Key requires Object Type",
            Self::MissingCreateObjectType => "Create requires Object Type",
            Self::MissingDeriveKeyObjectType => "Derive Key requires Object Type",
            Self::MissingDeriveKeyUniqueIdentifier => {
                "Derive Key requires at least one source Unique Identifier"
            }
            Self::MissingDeriveKeyMethod => "Derive Key requires a Derivation Method",
            Self::MissingDeriveKeyParameters => "Derive Key requires Derivation Parameters",
            Self::MissingDeriveKeyAttributes => "Derive Key requires output Attributes",
            Self::MissingCreateSplitKeyMethod => "Create Split Key requires Split Key Method",
            Self::MissingCreateSplitKeyParts => "Create Split Key requires Split Key Parts",
            Self::MissingCreateSplitKeyThreshold => "Create Split Key requires Split Key Threshold",
            Self::MissingHashCryptographicParameters => {
                "Hash requires caller-selected Cryptographic Parameters"
            }
            Self::MissingUsageLimitsCount => "Get Usage Allocation requires Usage Limits Count",
            Self::MissingRngRetrieveDataLength => "RNG Retrieve requires Data Length",
            Self::MissingRngSeedData => "RNG Seed requires secret Data",
            Self::MissingRegisterObject => "Register requires an object",
            Self::InvalidRegisterObject => "Register object or context is invalid",
            Self::MissingSetAttributeAttribute => "Set Attribute requires one New Attribute",
            Self::MissingAddAttributeAttribute => "Add Attribute requires one New Attribute",
            Self::InvalidAttributeValue => "fluent Object Attribute value is invalid",
            Self::MessageExtensionTagNotObjectAttribute => {
                "KMIP message extension tags are not Object Attributes"
            }
            Self::DuplicateAttributePosition => "fluent Object Attribute positions must be unique",
            Self::AttributePositionOutOfBounds => {
                "fluent Object Attribute position is outside the final attribute list"
            }
            Self::InvalidRequest => "fluent request failed typed request validation",
            Self::InvalidTtlvValue => "fluent TTLV value could not be represented",
            Self::MissingClientForExecution => {
                "detached fluent builder has no client for execution"
            }
        })
    }
}

impl Error for FluentBuildError {}

/// Explicit parent path and final zero-based position for an expert attribute.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FluentAttributePosition {
    /// Insert a direct item into Request Payload / Object Attributes.
    ObjectAttributes {
        /// Index in the final Object Attributes sequence after semantic fields
        /// and append-only expert attributes have been assembled.
        index: usize,
    },
}

/// A scoped fluent namespace borrowed from a mutable client.
/// value or adjustment type for the caller.
#[non_exhaustive]
#[derive(Debug)]
pub struct AttributeAdjustmentValue(Value);

impl AttributeAdjustmentValue {
    /// Creates a signed 32-bit Integer adjustment value.
    #[must_use]
    pub fn integer(value: i32) -> Self {
        Self(Value::integer(value))
    }

    /// Creates a signed 64-bit Long Integer adjustment value.
    #[must_use]
    pub fn long_integer(value: i64) -> Self {
        Self(Value::long_integer(value))
    }

    /// Creates an Enumeration adjustment value from its exact raw value.
    #[must_use]
    pub fn enumeration(value: u32) -> Self {
        Self(Value::enumeration(value))
    }

    /// Creates a Boolean adjustment value.
    #[must_use]
    pub fn boolean(value: bool) -> Self {
        Self(Value::boolean(value))
    }

    /// Creates a Text String adjustment value without normalization.
    #[must_use]
    pub fn text_string(value: String) -> Self {
        Self(Value::text_string(value))
    }

    /// Creates a signed 64-bit Date Time adjustment value.
    #[must_use]
    pub fn date_time(value: i64) -> Self {
        Self(Value::date_time(value))
    }

    /// Creates an unsigned 32-bit Interval adjustment value.
    #[must_use]
    pub fn interval(value: u32) -> Self {
        Self(Value::interval(value))
    }

    /// Creates a signed 64-bit Date Time Extended adjustment value.
    #[must_use]
    pub fn date_time_extended(value: i64) -> Self {
        Self(Value::date_time_extended(value))
    }

    fn try_clone(&self) -> Result<Self, FluentBuildError> {
        kmipkit_ttlv::try_clone_value(&self.0)
            .map(Self)
            .map_err(|_| FluentBuildError::InvalidTtlvValue)
    }

    fn into_value(self) -> Value {
        self.0
    }
}

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
    pub fn deactivate(self, unique_identifier: UniqueIdentifier) -> DeactivateBuilder<'a> {
        DeactivateBuilder::new(self.client, unique_identifier)
    }

    /// Begins a KMIP 3.0 Decapsulate builder for a caller-supplied identifier
    /// and encapsulated ciphertext.
    #[must_use]
    pub fn decapsulate(
        self,
        unique_identifier: UniqueIdentifier,
        ciphertext: Vec<u8>,
    ) -> DecapsulateBuilder<'a> {
        DecapsulateBuilder::new(self.client, unique_identifier, ciphertext)
    }

    /// Begins a KMIP 3.0 Encapsulate builder for a caller-supplied identifier.
    #[must_use]
    pub fn encapsulate(self, unique_identifier: UniqueIdentifier) -> EncapsulateBuilder<'a> {
        EncapsulateBuilder::new(self.client, unique_identifier)
    }

    /// Begins a KMIP 3.0 Obliterate builder for a caller-supplied identifier.
    #[must_use]
    pub fn obliterate(self, unique_identifier: UniqueIdentifier) -> ObliterateBuilder<'a> {
        ObliterateBuilder::new(self.client, unique_identifier)
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
    pub fn get(self, unique_identifier: UniqueIdentifier) -> GetBuilder<'a> {
        self.get_request().unique_identifier(unique_identifier)
    }

    /// Begins a Get request with all selectors unset, including the identifier.
    #[must_use]
    pub fn get_request(self) -> GetBuilder<'a> {
        GetBuilder {
            client: self.client,
            request: GetRequest::new(),
            options: None,
            asynchronous_indicator: None,
        }
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
    pub fn destroy(self, unique_identifier: UniqueIdentifier) -> DestroyBuilder<'a> {
        DestroyBuilder {
            client: self.client,
            request: DestroyRequest::new(Some(unique_identifier)),
            options: None,
            asynchronous_indicator: None,
        }
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
        PingBuilder {
            execution: FluentBuilderExecution::new(self.client),
        }
    }
}

/// A fluent generic Create builder for OASIS KMIP v2.1 §6.1.8, Tables 186–188.
/// It preserves the existing typed `CreateRequest` and does not choose object
/// attributes or storage policy for the caller.
pub struct CreateBuilder<'a> {
    execution: FluentBuilderExecution<'a>,
    object_type: Option<ObjectType>,
    attributes: AttributeSet,
    protection_storage_masks: Option<Structure>,
}

impl<'a> CreateBuilder<'a> {
    fn new(client: &'a mut Client) -> Self {
        Self {
            execution: FluentBuilderExecution::new(client),
            object_type: None,
            attributes: AttributeSet::new(),
            protection_storage_masks: None,
        }
    }

    /// Selects the caller's explicit KMIP Object Type.
    #[must_use]
    pub const fn object_type(mut self, object_type: ObjectType) -> Self {
        self.object_type = Some(object_type);
        self
    }

    /// Replaces the direct ordered Attributes to include in the request.
    ///
    /// An empty set is permitted by §5.1, Table 157; the required outer
    /// Attributes Structure is still emitted.
    #[must_use]
    pub fn attributes(mut self, attributes: AttributeSet) -> Self {
        self.attributes = attributes;
        self
    }

    /// Sets the optional generic Protection Storage Masks Structure.
    #[must_use]
    pub fn protection_storage_masks(mut self, masks: Structure) -> Self {
        self.protection_storage_masks = Some(masks);
        self
    }

    /// Sets exchange-local options without changing client policy.
    #[must_use]
    pub fn with_options(mut self, options: RequestOptions) -> Self {
        self.execution = self.execution.with_options(options);
        self
    }

    /// Sets the Request Header Asynchronous Indicator for this exchange.
    #[must_use]
    pub const fn with_asynchronous_indicator(mut self, value: u32) -> Self {
        self.execution = self.execution.with_asynchronous_indicator(value);
        self
    }

    fn into_request_and_execution(
        self,
    ) -> Result<(CreateRequest, FluentBuilderExecution<'a>), FluentBuildError> {
        let request = create_typed_request(
            self.object_type
                .ok_or(FluentBuildError::MissingCreateObjectType)?,
            self.attributes,
            self.protection_storage_masks,
        )?;
        Ok((request, self.execution))
    }

    /// Validates the typed Create request without sending it.
    ///
    /// # Errors
    /// Returns [`FluentBuildError::MissingCreateObjectType`] when Object Type
    /// is absent, or [`FluentBuildError::InvalidRequest`] when typed request
    /// validation rejects the payload.
    /// Builds the existing typed Create request without consuming the builder.
    ///
    /// # Errors
    /// Returns [`FluentBuildError::MissingCreateObjectType`] when Object Type
    /// is absent, or [`FluentBuildError::InvalidRequest`] when typed validation
    /// fails.
    pub fn build_request(&self) -> Result<CreateRequest, FluentBuildError> {
        let object_type = self
            .object_type
            .ok_or(FluentBuildError::MissingCreateObjectType)?;
        let mut attributes = AttributeSet::new();
        for attribute in self.attributes.as_items() {
            attributes
                .try_push(
                    attribute
                        .try_clone()
                        .map_err(|_| FluentBuildError::InvalidTtlvValue)?,
                )
                .map_err(|_| FluentBuildError::InvalidAttributeValue)?;
        }
        let mut request = CreateRequest::new(object_type, attributes);
        if let Some(masks) = &self.protection_storage_masks {
            request = request.with_protection_storage_masks(
                masks
                    .view()
                    .try_clone()
                    .map_err(|_| FluentBuildError::InvalidTtlvValue)?,
            );
        }
        validate_fluent_request(&request)?;
        Ok(request)
    }

    /// Validates the typed Create request without sending it.
    ///
    /// # Errors
    /// Returns [`FluentBuildError::MissingCreateObjectType`] when Object Type
    /// is absent, or [`FluentBuildError::InvalidRequest`] when typed request
    /// validation rejects the payload.
    pub fn validate(&self) -> Result<(), FluentBuildError> {
        self.build_request().map(|_| ())
    }

    /// Builds the existing typed Create request without sending it.
    ///
    /// # Errors
    /// Returns [`FluentBuildError::MissingCreateObjectType`] when Object Type
    /// is absent, or [`FluentBuildError::InvalidRequest`] when typed validation
    /// fails.
    pub fn build(self) -> Result<CreateRequest, FluentBuildError> {
        self.into_request_and_execution()
            .map(|(request, _execution)| request)
    }

    /// Converts this request to the existing typed batch item without sending it.
    ///
    /// Exchange options and the asynchronous indicator belong to the batch
    /// header and are not retained in an individual batch item.
    ///
    /// # Errors
    /// Returns a local builder error if a required field is absent or typed
    /// request validation fails.
    pub fn to_batch_item(self) -> Result<ClientBatchItem, FluentBuildError> {
        self.into_request_and_execution()
            .map(|(request, _execution)| ClientBatchItem::new(ClientRequest::Create(request)))
    }

    /// Executes at most one exchange through the existing typed client path.
    ///
    /// Pending preserves the originating Create operation and correlation
    /// value without a follow-up or retry.
    ///
    /// # Errors
    /// Returns the existing sanitized validation, protocol, or transport error
    /// with its request-delivery evidence.
    pub fn execute(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let (request, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run(ClientRequest::Create(request), limits)
    }
}

/// A fluent Derive Key builder for OASIS KMIP v2.1 §6.1.14, Tables 205–207.
/// It preserves repeated source identifiers, caller-selected derivation
/// parameters, and the existing typed result and delivery-state path.
pub struct DeriveKeyBuilder<'a> {
    execution: FluentBuilderExecution<'a>,
    object_type: Option<ObjectType>,
    unique_identifiers: Vec<UniqueIdentifier>,
    derivation_method: Option<DerivationMethod>,
    derivation_parameters: Option<DerivationParameters>,
    attributes: Option<AttributeSet>,
}

impl<'a> DeriveKeyBuilder<'a> {
    fn new(client: &'a mut Client) -> Self {
        Self {
            execution: FluentBuilderExecution::new(client),
            object_type: None,
            unique_identifiers: Vec::new(),
            derivation_method: None,
            derivation_parameters: None,
            attributes: None,
        }
    }

    /// Selects the caller's explicit output Object Type.
    #[must_use]
    pub const fn object_type(mut self, object_type: ObjectType) -> Self {
        self.object_type = Some(object_type);
        self
    }

    /// Appends one explicit source Unique Identifier in request order.
    ///
    /// §6.1.14 permits repeated identifiers but prohibits substituting the ID
    /// Placeholder. The existing typed request validator enforces that rule.
    #[must_use]
    pub fn unique_identifier(mut self, identifier: UniqueIdentifier) -> Self {
        self.unique_identifiers.push(identifier);
        self
    }

    /// Selects the caller's explicit Derivation Method.
    #[must_use]
    pub const fn derivation_method(mut self, method: DerivationMethod) -> Self {
        self.derivation_method = Some(method);
        self
    }

    /// Supplies the caller's typed §7.12 Derivation Parameters.
    #[must_use]
    pub fn derivation_parameters(mut self, parameters: DerivationParameters) -> Self {
        self.derivation_parameters = Some(parameters);
        self
    }

    /// Replaces the caller-selected output Attributes.
    #[must_use]
    pub fn attributes(mut self, attributes: AttributeSet) -> Self {
        self.attributes = Some(attributes);
        self
    }

    /// Sets exchange-local options without changing client policy.
    #[must_use]
    pub fn with_options(mut self, options: RequestOptions) -> Self {
        self.execution = self.execution.with_options(options);
        self
    }

    /// Sets the Request Header Asynchronous Indicator for this exchange.
    #[must_use]
    pub const fn with_asynchronous_indicator(mut self, value: u32) -> Self {
        self.execution = self.execution.with_asynchronous_indicator(value);
        self
    }

    fn validate_required_fields(&self) -> Result<(), FluentBuildError> {
        if self.object_type.is_none() {
            return Err(FluentBuildError::MissingDeriveKeyObjectType);
        }
        if self.unique_identifiers.is_empty() {
            return Err(FluentBuildError::MissingDeriveKeyUniqueIdentifier);
        }
        if self.derivation_method.is_none() {
            return Err(FluentBuildError::MissingDeriveKeyMethod);
        }
        if self.derivation_parameters.is_none() {
            return Err(FluentBuildError::MissingDeriveKeyParameters);
        }
        if self.attributes.is_none() {
            return Err(FluentBuildError::MissingDeriveKeyAttributes);
        }
        Ok(())
    }

    fn into_request_and_execution(
        self,
    ) -> Result<(DeriveKeyRequest, FluentBuilderExecution<'a>), FluentBuildError> {
        self.validate_required_fields()?;
        let request = DeriveKeyRequest::new(
            self.object_type
                .ok_or(FluentBuildError::MissingDeriveKeyObjectType)?,
            self.unique_identifiers,
            self.derivation_method
                .ok_or(FluentBuildError::MissingDeriveKeyMethod)?,
            self.derivation_parameters
                .ok_or(FluentBuildError::MissingDeriveKeyParameters)?,
            self.attributes
                .ok_or(FluentBuildError::MissingDeriveKeyAttributes)?,
        );
        if !request.validate().is_valid() {
            return Err(FluentBuildError::InvalidRequest);
        }
        Ok((request, self.execution))
    }

    /// Checks that every required Table 205 field has been supplied.
    ///
    /// The typed request validation also runs in [`Self::build`],
    /// [`Self::to_batch_item`], and [`Self::execute`].
    ///
    /// # Errors
    /// Returns the specific missing-field error for the first absent required
    /// field.
    pub fn validate(&self) -> Result<(), FluentBuildError> {
        self.validate_required_fields()
    }

    /// Builds the existing typed Derive Key request without sending it.
    ///
    /// # Errors
    /// Returns a payload-free error when a required field is absent or the
    /// typed request validation rejects the selected values.
    pub fn build(self) -> Result<DeriveKeyRequest, FluentBuildError> {
        self.into_request_and_execution()
            .map(|(request, _execution)| request)
    }

    /// Converts this request to the existing typed batch item without sending it.
    ///
    /// Exchange options and the asynchronous indicator belong to the batch
    /// header and are not retained in an individual batch item.
    ///
    /// # Errors
    /// Returns a payload-free error when a required field is absent or the
    /// typed request validation rejects the selected values.
    pub fn to_batch_item(self) -> Result<ClientBatchItem, FluentBuildError> {
        self.into_request_and_execution()
            .map(|(request, _execution)| ClientBatchItem::new(ClientRequest::DeriveKey(request)))
    }

    /// Executes at most one exchange through the existing typed client path.
    ///
    /// Pending preserves the originating Derive Key operation and correlation
    /// value without a follow-up or retry.
    ///
    /// # Errors
    /// Returns the existing sanitized validation, protocol, or transport error
    /// with its request-delivery evidence.
    pub fn execute(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let (request, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run(ClientRequest::DeriveKey(request), limits)
    }
}

fn create_typed_request(
    object_type: ObjectType,
    attributes: AttributeSet,
    protection_storage_masks: Option<Structure>,
) -> Result<CreateRequest, FluentBuildError> {
    let mut request = CreateRequest::new(object_type, attributes);
    if let Some(masks) = protection_storage_masks {
        request = request.with_protection_storage_masks(masks);
    }
    validate_fluent_request(&request)?;
    Ok(request)
}

/// A fluent builder for the empty Ping payload in OASIS KMIP v2.1 §6.1.36,
/// Table 271. Execution preserves the existing typed result and Pending outcome.
pub struct PingBuilder<'a> {
    execution: FluentBuilderExecution<'a>,
}

impl PingBuilder<'_> {
    /// Sets exchange-local options without changing client policy.
    #[must_use]
    pub fn with_options(mut self, options: RequestOptions) -> Self {
        self.execution = self.execution.with_options(options);
        self
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

    /// Executes at most one exchange through the existing typed client path.
    ///
    /// Pending preserves its correlation value without a follow-up or retry.
    ///
    /// # Errors
    /// Returns the existing sanitized validation, protocol, or transport error
    /// with its request-delivery evidence. KMIP 2.0 is rejected before sending.
    pub fn execute(
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
    fn new(client: &'a mut Client) -> Self {
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

    /// Executes at most one exchange through the existing typed client path.
    ///
    /// Pending remains explicit; this builder does not poll or retry.
    ///
    /// # Errors
    /// Returns the existing sanitized protocol or transport error with its
    /// request-delivery evidence.
    pub fn execute(
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
    fn new(client: &'a mut Client) -> Self {
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
    pub fn unique_identifier(mut self, value: UniqueIdentifier) -> Self {
        self.unique_identifier = Some(value);
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
    /// Returns the existing sanitized validation, protocol, or transport error
    /// with its request-delivery evidence.
    pub fn execute(
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
    fn new(client: &'a mut Client) -> Self {
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
    /// Returns the existing sanitized validation, protocol, or transport error
    /// with its request-delivery evidence.
    pub fn execute(
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
    fn new(client: &'a mut Client) -> Self {
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
    /// Returns the existing sanitized validation, protocol, or transport error
    /// with its request-delivery evidence.
    pub fn execute(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let (request, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run(ClientRequest::rng_seed(request), limits)
    }
}

struct FluentBuilderExecution<'a> {
    client: &'a mut Client,
    options: Option<RequestOptions>,
    asynchronous_indicator: Option<u32>,
}

impl<'a> FluentBuilderExecution<'a> {
    fn new(client: &'a mut Client) -> Self {
        Self {
            client,
            options: None,
            asynchronous_indicator: None,
        }
    }

    fn with_options(mut self, options: RequestOptions) -> Self {
        self.options = Some(options);
        self
    }

    const fn with_asynchronous_indicator(mut self, value: u32) -> Self {
        self.asynchronous_indicator = Some(value);
        self
    }

    fn run(
        self,
        request: ClientRequest,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        execute_fluent_request(
            self.client,
            request,
            self.asynchronous_indicator,
            self.options,
            limits,
        )
    }
}

// Holds a typed cryptographic request while consuming field transformations.
// Byte conversion and validation live here so each public facade follows the
// same ownership and execution rules without duplicating that logic.
struct CryptoRequestState<'a, R> {
    execution: FluentBuilderExecution<'a>,
    request: R,
}

impl<'a, R: ValidateKmipRequest> CryptoRequestState<'a, R> {
    fn new(client: &'a mut Client, request: R) -> Self {
        Self {
            execution: FluentBuilderExecution::new(client),
            request,
        }
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

    fn into_parts(self) -> Result<(R, FluentBuilderExecution<'a>), FluentBuildError> {
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
    fn new() -> Self {
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

    fn into_request<R>(self, adapter: &CipherRequestAdapter<R>) -> R {
        self.fields
            .into_cipher_request(self.unique_identifier, adapter)
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
    fn new(client: &'a mut Client) -> Self {
        Self {
            execution: FluentBuilderExecution::new(client),
            core: CipherBuilderCore::new(),
        }
    }

    /// Sets the optional key identifier without converting its representation.
    #[must_use]
    pub fn unique_identifier(mut self, value: UniqueIdentifier) -> Self {
        self.core.set_encrypt_identifier(value);
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

    /// Sets exchange-local transport options.
    #[must_use]
    pub fn with_options(mut self, options: RequestOptions) -> Self {
        self.execution = self.execution.with_options(options);
        self
    }

    /// Sets the Request Header Asynchronous Indicator for this exchange.
    #[must_use]
    pub const fn with_asynchronous_indicator(mut self, value: u32) -> Self {
        self.execution = self.execution.with_asynchronous_indicator(value);
        self
    }

    fn into_request_and_execution(
        self,
    ) -> Result<(EncryptRequest, FluentBuilderExecution<'a>), FluentBuildError> {
        let Self { execution, core } = self;
        let request = core.into_request(&CipherRequestAdapter {
            new: EncryptRequest::new,
            with_cryptographic_parameters: EncryptRequest::with_cryptographic_parameters,
            with_iv_counter_nonce: EncryptRequest::with_iv_counter_nonce,
            with_correlation_value: EncryptRequest::with_correlation_value,
            with_init_indicator: EncryptRequest::with_init_indicator,
            with_final_indicator: EncryptRequest::with_final_indicator,
            with_authenticated_encryption_additional_data:
                EncryptRequest::with_authenticated_encryption_additional_data,
        });
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

    /// Executes one typed Encrypt exchange without retrying it.
    ///
    /// # Errors
    /// Returns a validation, transport, or protocol error without retrying the exchange.
    pub fn execute(
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
    fn new(client: &'a mut Client) -> Self {
        Self {
            execution: FluentBuilderExecution::new(client),
            core: CipherBuilderCore::new(),
            authenticated_encryption_tag: None,
        }
    }

    /// Sets the optional key identifier without converting its representation.
    #[must_use]
    pub fn unique_identifier(mut self, value: UniqueIdentifier) -> Self {
        self.core.set_decrypt_identifier(value);
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

    /// Sets exchange-local transport options.
    #[must_use]
    pub fn with_options(mut self, options: RequestOptions) -> Self {
        self.execution.options = Some(options);
        self
    }

    /// Sets the Request Header Asynchronous Indicator for this exchange.
    #[must_use]
    pub const fn with_asynchronous_indicator(mut self, value: u32) -> Self {
        self.execution.asynchronous_indicator = Some(value);
        self
    }

    fn into_request_and_execution(
        self,
    ) -> Result<(DecryptRequest, FluentBuilderExecution<'a>), FluentBuildError> {
        let Self {
            execution,
            core,
            authenticated_encryption_tag,
        } = self;
        let mut request = core.into_request(&CipherRequestAdapter {
            new: DecryptRequest::new,
            with_cryptographic_parameters: DecryptRequest::with_cryptographic_parameters,
            with_iv_counter_nonce: DecryptRequest::with_iv_counter_nonce,
            with_correlation_value: DecryptRequest::with_correlation_value,
            with_init_indicator: DecryptRequest::with_init_indicator,
            with_final_indicator: DecryptRequest::with_final_indicator,
            with_authenticated_encryption_additional_data:
                DecryptRequest::with_authenticated_encryption_additional_data,
        });
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

    /// Executes one typed Decrypt exchange without retrying it.
    ///
    /// # Errors
    /// Returns a validation, transport, or protocol error without retrying the exchange.
    pub fn execute(
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
    fn new(client: &'a mut Client) -> Self {
        Self {
            state: CryptoRequestState::new(client, SignRequest::new()),
        }
    }

    /// Sets the optional signing key identifier.
    #[must_use]
    pub fn unique_identifier(mut self, value: UniqueIdentifier) -> Self {
        self.state = self.state.field(value, SignRequest::with_unique_identifier);
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
        self.state.execution.options = Some(options);
        self
    }

    /// Sets the Request Header Asynchronous Indicator for this exchange.
    #[must_use]
    pub const fn with_asynchronous_indicator(mut self, value: u32) -> Self {
        self.state.execution.asynchronous_indicator = Some(value);
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

    /// Executes one typed Sign exchange without retrying it.
    ///
    /// # Errors
    /// Returns a validation, transport, or protocol error without retrying the exchange.
    pub fn execute(
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
    fn new(client: &'a mut Client) -> Self {
        Self {
            state: CryptoRequestState::new(client, SignatureVerifyRequest::new()),
        }
    }

    /// Sets the optional verification key identifier.
    #[must_use]
    pub fn unique_identifier(mut self, value: UniqueIdentifier) -> Self {
        self.state = self
            .state
            .field(value, SignatureVerifyRequest::with_unique_identifier);
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
        self.state.execution.options = Some(options);
        self
    }

    /// Sets the Request Header Asynchronous Indicator for this exchange.
    #[must_use]
    pub const fn with_asynchronous_indicator(mut self, value: u32) -> Self {
        self.state.execution.asynchronous_indicator = Some(value);
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

    /// Executes one typed Signature Verify exchange without retrying it.
    ///
    /// # Errors
    /// Returns a validation, transport, or protocol error without retrying the exchange.
    pub fn execute(
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
    fn new(client: &'a mut Client) -> Self {
        Self {
            state: CryptoRequestState::new(client, MacRequest::new()),
        }
    }

    /// Sets the optional MAC key identifier.
    #[must_use]
    pub fn unique_identifier(mut self, value: UniqueIdentifier) -> Self {
        self.state = self.state.field(value, MacRequest::with_unique_identifier);
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
        self.state.execution.options = Some(options);
        self
    }

    /// Sets the Request Header Asynchronous Indicator for this exchange.
    #[must_use]
    pub const fn with_asynchronous_indicator(mut self, value: u32) -> Self {
        self.state.execution.asynchronous_indicator = Some(value);
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

    /// Executes one typed MAC exchange without retrying it.
    ///
    /// # Errors
    /// Returns a validation, transport, or protocol error without retrying the exchange.
    pub fn execute(
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
    fn new(client: &'a mut Client) -> Self {
        Self {
            state: CryptoRequestState::new(client, MacVerifyRequest::new()),
        }
    }

    /// Sets the optional MAC verification key identifier.
    #[must_use]
    pub fn unique_identifier(mut self, value: UniqueIdentifier) -> Self {
        self.state = self
            .state
            .field(value, MacVerifyRequest::with_unique_identifier);
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
        self.state.execution.options = Some(options);
        self
    }

    /// Sets the Request Header Asynchronous Indicator for this exchange.
    #[must_use]
    pub const fn with_asynchronous_indicator(mut self, value: u32) -> Self {
        self.state.execution.asynchronous_indicator = Some(value);
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

    /// Executes one typed MAC Verify exchange without retrying it.
    ///
    /// # Errors
    /// Returns a validation, transport, or protocol error without retrying the exchange.
    pub fn execute(
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
    fn new(client: &'a mut Client) -> Self {
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

    /// Sets exchange-local transport options.
    #[must_use]
    pub fn with_options(mut self, options: RequestOptions) -> Self {
        self.execution.options = Some(options);
        self
    }

    /// Sets the Request Header Asynchronous Indicator for this exchange.
    #[must_use]
    pub const fn with_asynchronous_indicator(mut self, value: u32) -> Self {
        self.execution.asynchronous_indicator = Some(value);
        self
    }

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

    /// Executes one typed Hash exchange without retrying it.
    ///
    /// # Errors
    /// Returns a validation, transport, or protocol error without retrying the exchange.
    pub fn execute(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let (request, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run(ClientRequest::Hash(request), limits)
    }
}

/// A Register builder that validates and executes the existing typed request.
pub struct RegisterBuilder<'a> {
    client: &'a mut Client,
    object: Option<(ObjectType, Item)>,
    attributes: AttributeSet,
    protection_storage_masks: Option<Structure>,
    opaque_object_context: Option<OpaqueObjectContext>,
    positioned_attributes: Vec<(usize, Item)>,
    attribute_error: Option<FluentBuildError>,
    options: Option<RequestOptions>,
    asynchronous_indicator: Option<u32>,
}

impl<'a> RegisterBuilder<'a> {
    fn new(client: &'a mut Client) -> Self {
        Self {
            client,
            object: None,
            attributes: AttributeSet::new(),
            protection_storage_masks: None,
            opaque_object_context: None,
            positioned_attributes: Vec::new(),
            attribute_error: None,
            options: None,
            asynchronous_indicator: None,
        }
    }

    /// Sets a typed object and derives its KMIP Object Type.
    #[must_use]
    pub fn object(mut self, object: ManagedObject) -> Self {
        let object_type = object.object_type();
        self.object = Some((object_type, object.into_item()));
        self
    }

    /// Sets an explicit Object Type and structurally typed object Item.
    ///
    /// This is the expert path for future, vendor, or otherwise generic object
    /// types. Final validation still checks known Object Type and object-tag
    /// relationships through the existing Register request model.
    #[must_use]
    pub fn object_item(mut self, object_type: ObjectType, object: Item) -> Self {
        self.object = Some((object_type, object));
        self
    }

    /// Appends a catalogued attribute constructed with its typed marker.
    #[must_use]
    pub fn attribute<D: AttributeDefinition>(self, attribute: Attribute<D>) -> Self {
        self.attribute_item(attribute.into_item())
    }

    /// Appends an explicit TTLV Item as an expert Object Attribute.
    ///
    /// An allocated extension tag remains the exact supplied Item. It is not
    /// converted into a KMIP Vendor Attribute; use [`Self::vendor_attribute`]
    /// to build that standard structure.
    #[must_use]
    pub fn attribute_item(mut self, item: Item) -> Self {
        self.append_object_attribute_item(item);
        self
    }

    /// Appends a prebuilt Attribute Set in its existing item order.
    #[must_use]
    pub fn attributes(mut self, attributes: AttributeSet) -> Self {
        for item in attributes.into_items() {
            self.append_object_attribute_item(item);
        }
        self
    }

    /// Inserts an explicit expert attribute at its final Object Attributes index.
    #[must_use]
    pub fn attribute_at(mut self, position: FluentAttributePosition, item: Item) -> Self {
        set_positioned_attribute(
            &mut self.positioned_attributes,
            &mut self.attribute_error,
            position,
            item,
        );
        self
    }

    /// Builds a KMIP Vendor Attribute structure from explicit components.
    #[must_use]
    pub fn vendor_attribute(
        mut self,
        vendor_identification: &str,
        attribute_name: &str,
        attribute_value: &Item,
    ) -> Self {
        if self.attribute_error.is_some() {
            return self;
        }
        match make_vendor_attribute(vendor_identification, attribute_name, attribute_value) {
            Ok(item) => {
                self.append_object_attribute_item(item);
            }
            Err(error) => self.attribute_error = Some(error),
        }
        self
    }

    /// Sets optional Protection Storage Masks as an explicit KMIP Structure.
    #[must_use]
    pub fn protection_storage_masks(mut self, masks: Structure) -> Self {
        self.protection_storage_masks = Some(masks);
        self
    }

    /// Selects local Register attribute handling for an Opaque Object.
    #[must_use]
    pub const fn opaque_object_context(mut self, context: OpaqueObjectContext) -> Self {
        self.opaque_object_context = Some(context);
        self
    }

    /// Sets exchange-local options without changing client or batch policy.
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

    /// Validates all accumulated inputs without sending a request.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError`] when the object is missing, its local
    /// Register validation fails, or an attribute cannot be represented.
    pub fn validate(&self) -> Result<(), FluentBuildError> {
        self.build_request().map(|_| ())
    }

    /// Builds the existing typed Register request without sending it.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError`] when the object is missing, its local
    /// Register validation fails, or an attribute cannot be represented.
    pub fn build(self) -> Result<RegisterRequest, FluentBuildError> {
        self.build_request()
    }

    /// Builds a typed Register request without consuming the builder inputs.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError`] when the object is missing, its local
    /// Register validation fails, or an attribute cannot be represented.
    pub fn build_request(&self) -> Result<RegisterRequest, FluentBuildError> {
        if let Some(error) = self.attribute_error {
            return Err(error);
        }
        let (object_type, object) = self
            .object
            .as_ref()
            .ok_or(FluentBuildError::MissingRegisterObject)?;
        let mut attributes = AttributeSet::new();
        for attribute in self.attributes.as_items() {
            attributes
                .try_push(
                    attribute
                        .try_clone()
                        .map_err(|_| FluentBuildError::InvalidTtlvValue)?,
                )
                .map_err(|_| FluentBuildError::InvalidAttributeValue)?;
        }
        attributes = insert_positioned_attributes(attributes, &self.positioned_attributes)?;

        let object = object
            .try_clone()
            .map_err(|_| FluentBuildError::InvalidTtlvValue)?;
        let mut request = RegisterRequest::new(*object_type, attributes, object);
        if let Some(masks) = &self.protection_storage_masks {
            let masks = masks
                .view()
                .try_clone()
                .map_err(|_| FluentBuildError::InvalidTtlvValue)?;
            request = request.with_protection_storage_masks(masks);
        }
        if let Some(context) = self.opaque_object_context {
            request = request.with_opaque_object_context(context);
        }
        if !request.validate().is_valid() {
            return Err(FluentBuildError::InvalidRegisterObject);
        }
        Ok(request)
    }

    /// Converts the validated request to one existing typed batch item.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError`] when the object is missing, its local
    /// Register validation fails, or an attribute cannot be represented.
    pub fn to_batch_item(self) -> Result<ClientBatchItem, FluentBuildError> {
        self.build_request()
            .map(|request| ClientBatchItem::new(ClientRequest::Register(request)))
    }

    /// Executes one typed Register exchange through the existing client.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] when local validation fails or the existing
    /// client exchange fails.
    pub fn execute(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let request = self.build_request().map_err(|error| {
            ClientError::validation(
                ClientCauseCategory::InvalidInput,
                RequestDeliveryState::NotSent,
                error,
            )
        })?;
        self.client.execute_one_typed_request_with_async_indicator(
            ClientRequest::Register(request),
            self.asynchronous_indicator,
            limits,
            &self.options.unwrap_or_default(),
        )
    }

    fn append_object_attribute_item(&mut self, item: Item) {
        append_object_attribute(&mut self.attributes, &mut self.attribute_error, item);
    }
}

/// A Create Symmetric Key builder that returns the existing typed request.
pub struct CreateSymmetricKeyBuilder<'a> {
    client: Option<&'a mut Client>,
    algorithm: Option<CryptographicAlgorithm>,
    length: Option<i32>,
    usage_mask: Option<CryptographicUsageMask>,
    name: Option<String>,
    attributes: AttributeSet,
    positioned_attributes: Vec<(usize, Item)>,
    attribute_error: Option<FluentBuildError>,
    options: Option<RequestOptions>,
    asynchronous_indicator: Option<u32>,
}

impl<'a> CreateSymmetricKeyBuilder<'a> {
    fn new(client: Option<&'a mut Client>) -> Self {
        Self {
            client,
            algorithm: None,
            length: None,
            usage_mask: None,
            name: None,
            attributes: AttributeSet::new(),
            positioned_attributes: Vec::new(),
            attribute_error: None,
            options: None,
            asynchronous_indicator: None,
        }
    }

    /// Creates a reusable request builder without binding it to a client.
    ///
    /// This form can prepare typed requests and batch items for a later
    /// exchange. Use [`Self::build_request`] or [`Self::to_batch_item`] to
    /// materialize the request.
    #[must_use]
    pub fn detached() -> Self {
        Self::new(None)
    }

    /// Selects an explicit KMIP Cryptographic Algorithm value.
    #[must_use]
    pub const fn algorithm(mut self, algorithm: CryptographicAlgorithm) -> Self {
        self.algorithm = Some(algorithm);
        self
    }

    /// Selects an explicit KMIP Cryptographic Length in bits.
    #[must_use]
    pub const fn length(mut self, length: i32) -> Self {
        self.length = Some(length);
        self
    }

    /// Selects an explicit set of Cryptographic Usage Mask bits.
    #[must_use]
    pub const fn usage_mask(mut self, usage_mask: CryptographicUsageMask) -> Self {
        self.usage_mask = Some(usage_mask);
        self
    }

    /// Sets the raw KMIP Cryptographic Usage Mask bits.
    #[must_use]
    pub const fn usage_mask_bits(self, bits: i32) -> Self {
        self.usage_mask(CryptographicUsageMask::from_raw(bits))
    }

    /// Selects AES with a 128-bit key; usage policy remains explicit.
    #[must_use]
    pub const fn aes_128(self) -> Self {
        self.algorithm(CryptographicAlgorithm::from_raw(
            kmipkit_protocol::named_values_generated::values::enumeration::cryptographic_algorithm::AES,
        ))
        .length(128)
    }

    /// Selects AES with a 192-bit key; usage policy remains explicit.
    #[must_use]
    pub const fn aes_192(self) -> Self {
        self.algorithm(CryptographicAlgorithm::from_raw(
            kmipkit_protocol::named_values_generated::values::enumeration::cryptographic_algorithm::AES,
        ))
        .length(192)
    }

    /// Selects AES with a 256-bit key; usage policy remains explicit.
    #[must_use]
    pub const fn aes_256(self) -> Self {
        self.algorithm(CryptographicAlgorithm::from_raw(
            kmipkit_protocol::named_values_generated::values::enumeration::cryptographic_algorithm::AES,
        ))
        .length(256)
    }

    /// Selects HMAC-SHA256 with a 256-bit key; usage policy remains explicit.
    #[must_use]
    pub const fn hmac_sha256_256(self) -> Self {
        self.algorithm(CryptographicAlgorithm::from_raw(
            kmipkit_protocol::named_values_generated::values::enumeration::cryptographic_algorithm::HMAC_SHA256,
        ))
        .length(256)
    }

    /// Adds the common KMIP Name attribute using Uninterpreted Text String.
    #[must_use]
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_owned());
        self
    }

    /// Adds a catalogued attribute constructed with its generated typed marker.
    #[must_use]
    pub fn attribute<D: AttributeDefinition>(self, attribute: Attribute<D>) -> Self {
        self.attribute_item(attribute.into_item())
    }

    /// Adds an explicit TTLV Item as an expert Object Attribute.
    ///
    /// An allocated extension tag remains the exact supplied Item. It is not
    /// converted into a KMIP Vendor Attribute; use [`Self::vendor_attribute`]
    /// to build that standard structure.
    #[must_use]
    pub fn attribute_item(mut self, item: Item) -> Self {
        append_object_attribute(&mut self.attributes, &mut self.attribute_error, item);
        self
    }

    /// Inserts an explicit expert Item at a path and position in the final
    /// Request Payload / Object Attributes sequence.
    ///
    /// The Item's tag and TTLV value type are explicit in `item`. Catalogued
    /// tags and Vendor Attributes are validated by the same `AttributeSet` and
    /// typed request checks used for semantic attributes.
    #[must_use]
    pub fn attribute_at(mut self, position: FluentAttributePosition, item: Item) -> Self {
        set_positioned_attribute(
            &mut self.positioned_attributes,
            &mut self.attribute_error,
            position,
            item,
        );
        self
    }

    /// Builds the KMIP Vendor Attribute Structure from explicit components.
    #[must_use]
    pub fn vendor_attribute(
        mut self,
        vendor_identification: &str,
        attribute_name: &str,
        attribute_value: &Item,
    ) -> Self {
        if self.attribute_error.is_some() {
            return self;
        }
        match make_vendor_attribute(vendor_identification, attribute_name, attribute_value) {
            Ok(item) => self.add_attribute_item(item),
            Err(error) => {
                self.attribute_error.get_or_insert(error);
            }
        }
        self
    }

    /// Sets exchange-local options without changing client or batch policy.
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

    /// Builds the existing typed Create request without sending it.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError`] when required cryptographic properties or
    /// previously supplied attributes are invalid.
    pub fn build(self) -> Result<CreateRequest, FluentBuildError> {
        self.build_request_inner()
    }

    /// Builds a request without consuming the builder's accumulated inputs.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError`] when required cryptographic properties or
    /// previously supplied attributes are invalid.
    pub fn build_request(&self) -> Result<CreateRequest, FluentBuildError> {
        self.build_request_inner()
    }

    /// Converts the request to an existing typed batch item.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError`] when request construction fails.
    pub fn to_batch_item(self) -> Result<ClientBatchItem, FluentBuildError> {
        self.build_request_inner()
            .map(|request| ClientBatchItem::new(ClientRequest::Create(request)))
    }

    /// Executes one typed Create exchange and returns its existing response.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] when local request validation fails or the
    /// configured client exchange fails.
    pub fn execute(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let request = self.build_request_inner().map_err(|error| {
            ClientError::validation(
                ClientCauseCategory::InvalidInput,
                RequestDeliveryState::NotSent,
                error,
            )
        })?;
        let Some(client) = self.client else {
            return Err(ClientError::validation(
                ClientCauseCategory::InvalidInput,
                RequestDeliveryState::NotSent,
                FluentBuildError::MissingClientForExecution,
            ));
        };
        let options = self.options.unwrap_or_default();
        client.execute_one_typed_request_with_async_indicator(
            ClientRequest::Create(request),
            self.asynchronous_indicator,
            limits,
            &options,
        )
    }

    fn build_request_inner(&self) -> Result<CreateRequest, FluentBuildError> {
        if let Some(error) = self.attribute_error {
            return Err(error);
        }
        let algorithm = self
            .algorithm
            .ok_or(FluentBuildError::MissingCryptographicAlgorithm)?;
        let length = self
            .length
            .ok_or(FluentBuildError::MissingCryptographicLength)?;
        let usage_mask = self
            .usage_mask
            .ok_or(FluentBuildError::MissingCryptographicUsageMask)?;

        let mut attributes = AttributeSet::new();
        attributes
            .try_push_typed(
                Attribute::<typed::CryptographicAlgorithm>::new(algorithm.raw())
                    .map_err(|_| FluentBuildError::InvalidAttributeValue)?,
            )
            .map_err(|_| FluentBuildError::InvalidAttributeValue)?;
        attributes
            .try_push_typed(
                Attribute::<typed::CryptographicLength>::new(length)
                    .map_err(|_| FluentBuildError::InvalidAttributeValue)?,
            )
            .map_err(|_| FluentBuildError::InvalidAttributeValue)?;
        attributes
            .try_push_typed(
                Attribute::<typed::CryptographicUsageMask>::new(usage_mask.raw())
                    .map_err(|_| FluentBuildError::InvalidAttributeValue)?,
            )
            .map_err(|_| FluentBuildError::InvalidAttributeValue)?;
        if let Some(name) = &self.name {
            attributes
                .try_push_typed(
                    Attribute::<typed::Name>::new(name_attribute(name.clone())?)
                        .map_err(|_| FluentBuildError::InvalidAttributeValue)?,
                )
                .map_err(|_| FluentBuildError::InvalidAttributeValue)?;
        }
        for item in self.attributes.as_items() {
            attributes
                .try_push(
                    item.try_clone()
                        .map_err(|_| FluentBuildError::InvalidTtlvValue)?,
                )
                .map_err(|_| FluentBuildError::InvalidAttributeValue)?;
        }

        let attributes = insert_positioned_attributes(attributes, &self.positioned_attributes)?;

        let request =
            CreateRequest::new(ObjectType::from_raw(SYMMETRIC_KEY_OBJECT_TYPE), attributes);
        if !request.validate().is_valid() {
            return Err(FluentBuildError::InvalidRequest);
        }
        Ok(request)
    }

    fn add_attribute_item(&mut self, item: Item) {
        if self.attribute_error.is_some() {
            return;
        }
        if !self.accept_object_attribute_tag(&item) {
            return;
        }
        if self.attributes.try_push(item).is_err() {
            self.attribute_error = Some(FluentBuildError::InvalidAttributeValue);
        }
    }

    fn accept_object_attribute_tag(&mut self, item: &Item) -> bool {
        if let Err(error) = validate_object_attribute_tag(item) {
            self.attribute_error = Some(error);
            return false;
        }
        true
    }
}

/// A Create Split Key builder that preserves the caller's explicit split inputs.
pub struct CreateSplitKeyBuilder<'a> {
    client: &'a mut Client,
    object_type: Option<ObjectType>,
    method: Option<SplitKeyMethod>,
    parts: Option<i32>,
    threshold: Option<i32>,
    unique_identifier: Option<UniqueIdentifier>,
    prime_field_size: Option<Vec<u8>>,
    attributes: AttributeSet,
    protection_storage_masks: Option<Structure>,
    positioned_attributes: Vec<(usize, Item)>,
    attribute_error: Option<FluentBuildError>,
    options: Option<RequestOptions>,
    asynchronous_indicator: Option<u32>,
}

impl<'a> CreateSplitKeyBuilder<'a> {
    fn new(client: &'a mut Client) -> Self {
        Self {
            client,
            object_type: None,
            method: None,
            parts: None,
            threshold: None,
            unique_identifier: None,
            prime_field_size: None,
            attributes: AttributeSet::new(),
            protection_storage_masks: None,
            positioned_attributes: Vec::new(),
            attribute_error: None,
            options: None,
            asynchronous_indicator: None,
        }
    }

    /// Selects the explicit KMIP Object Type of the split input.
    #[must_use]
    pub const fn object_type(mut self, object_type: ObjectType) -> Self {
        self.object_type = Some(object_type);
        self
    }

    /// Selects the explicit KMIP Split Key Method.
    #[must_use]
    pub const fn method(mut self, method: SplitKeyMethod) -> Self {
        self.method = Some(method);
        self
    }

    /// Selects the explicit number of output key parts.
    #[must_use]
    pub const fn parts(mut self, parts: i32) -> Self {
        self.parts = Some(parts);
        self
    }

    /// Selects the explicit reconstruction threshold.
    #[must_use]
    pub const fn threshold(mut self, threshold: i32) -> Self {
        self.threshold = Some(threshold);
        self
    }

    /// Sets the optional input key identifier.
    #[must_use]
    pub fn unique_identifier(mut self, identifier: UniqueIdentifier) -> Self {
        self.unique_identifier = Some(identifier);
        self
    }

    /// Sets the exact caller-supplied Prime Field Size bytes.
    #[must_use]
    pub fn prime_field_size(mut self, value: Vec<u8>) -> Self {
        self.prime_field_size = Some(value);
        self
    }

    /// Sets optional Protection Storage Masks as an explicit KMIP Structure.
    #[must_use]
    pub fn protection_storage_masks(mut self, masks: Structure) -> Self {
        self.protection_storage_masks = Some(masks);
        self
    }

    /// Appends one typed catalogued Object Attribute.
    #[must_use]
    pub fn attribute<D: AttributeDefinition>(self, attribute: Attribute<D>) -> Self {
        self.attribute_item(attribute.into_item())
    }

    /// Appends an explicit TTLV Item as an expert Object Attribute.
    ///
    /// An allocated extension tag remains the exact supplied Item. It is not
    /// converted into a KMIP Vendor Attribute; use [`Self::vendor_attribute`]
    /// to build that standard structure.
    #[must_use]
    pub fn attribute_item(mut self, item: Item) -> Self {
        append_object_attribute(&mut self.attributes, &mut self.attribute_error, item);
        self
    }

    /// Appends a prebuilt Attribute Set in its existing item order.
    #[must_use]
    pub fn attributes(mut self, attributes: AttributeSet) -> Self {
        for item in attributes.into_items() {
            append_object_attribute(&mut self.attributes, &mut self.attribute_error, item);
        }
        self
    }

    /// Builds a valid Vendor Attribute Structure from explicit components.
    #[must_use]
    pub fn vendor_attribute(
        mut self,
        vendor_identification: &str,
        attribute_name: &str,
        attribute_value: &Item,
    ) -> Self {
        if self.attribute_error.is_some() {
            return self;
        }
        match make_vendor_attribute(vendor_identification, attribute_name, attribute_value) {
            Ok(item) => {
                append_object_attribute(&mut self.attributes, &mut self.attribute_error, item);
            }
            Err(error) => {
                self.attribute_error = Some(error);
            }
        }
        self
    }

    /// Inserts an expert Item at a zero-based position in the final Object Attributes list.
    #[must_use]
    pub fn attribute_at(mut self, position: FluentAttributePosition, item: Item) -> Self {
        set_positioned_attribute(
            &mut self.positioned_attributes,
            &mut self.attribute_error,
            position,
            item,
        );
        self
    }

    /// Sets exchange-local options without changing client or batch policy.
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

    /// Validates locally using the existing typed request rules.
    ///
    /// # Errors
    ///
    /// Returns a payload-free error for missing split parameters, invalid
    /// attributes, or a typed request validation failure.
    pub fn validate(&self) -> Result<(), FluentBuildError> {
        self.build_request().map(|_| ())
    }

    /// Builds the typed Create Split Key request without sending it.
    ///
    /// # Errors
    ///
    /// Returns a payload-free error for missing split parameters, invalid
    /// attributes, or a typed request validation failure.
    pub fn build(self) -> Result<CreateSplitKeyRequest, FluentBuildError> {
        self.build_request()
    }

    /// Builds a typed request without consuming the builder's accumulated inputs.
    ///
    /// # Errors
    ///
    /// Returns a payload-free error for missing split parameters, invalid
    /// attributes, or a typed request validation failure.
    pub fn build_request(&self) -> Result<CreateSplitKeyRequest, FluentBuildError> {
        if let Some(error) = self.attribute_error {
            return Err(error);
        }
        let object_type = self
            .object_type
            .ok_or(FluentBuildError::MissingCreateSplitKeyObjectType)?;
        let method = self
            .method
            .ok_or(FluentBuildError::MissingCreateSplitKeyMethod)?;
        let parts = self
            .parts
            .ok_or(FluentBuildError::MissingCreateSplitKeyParts)?;
        let threshold = self
            .threshold
            .ok_or(FluentBuildError::MissingCreateSplitKeyThreshold)?;

        let mut attributes = AttributeSet::new();
        for attribute in self.attributes.as_items() {
            attributes
                .try_push(
                    attribute
                        .try_clone()
                        .map_err(|_| FluentBuildError::InvalidTtlvValue)?,
                )
                .map_err(|_| FluentBuildError::InvalidAttributeValue)?;
        }
        attributes = insert_positioned_attributes(attributes, &self.positioned_attributes)?;

        let mut request =
            CreateSplitKeyRequest::new(object_type, parts, threshold, method, attributes);
        if let Some(identifier) = self.unique_identifier.clone() {
            request = request.with_unique_identifier(identifier);
        }
        if let Some(prime_field_size) = &self.prime_field_size {
            request = request.with_prime_field_size(prime_field_size.clone());
        }
        if let Some(masks) = &self.protection_storage_masks {
            let copied = masks
                .view()
                .try_clone()
                .map_err(|_| FluentBuildError::InvalidTtlvValue)?;
            request = request.with_protection_storage_masks(copied);
        }
        if !request.validate().is_valid() {
            return Err(FluentBuildError::InvalidRequest);
        }
        Ok(request)
    }

    /// Converts the typed request to an existing batch item.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError`] if the request cannot be built.
    pub fn to_batch_item(self) -> Result<ClientBatchItem, FluentBuildError> {
        self.build_request()
            .map(|request| ClientBatchItem::new(ClientRequest::CreateSplitKey(request)))
    }

    /// Executes one typed Create Split Key exchange.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] for local validation or client exchange failure.
    pub fn execute(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let request = self.build_request().map_err(|error| {
            ClientError::validation(
                ClientCauseCategory::InvalidInput,
                RequestDeliveryState::NotSent,
                error,
            )
        })?;
        self.client.execute_one_typed_request_with_async_indicator(
            ClientRequest::CreateSplitKey(request),
            self.asynchronous_indicator,
            limits,
            &self.options.unwrap_or_default(),
        )
    }
}

/// A Create Key Pair builder with independent common and key-specific attributes.
pub struct CreateKeyPairBuilder<'a> {
    client: Option<&'a mut Client>,
    algorithm: Option<CryptographicAlgorithm>,
    length: Option<i32>,
    private_usage_mask: Option<CryptographicUsageMask>,
    public_usage_mask: Option<CryptographicUsageMask>,
    private_name: Option<String>,
    public_name: Option<String>,
    common_attributes: AttributeSet,
    private_attributes: AttributeSet,
    public_attributes: AttributeSet,
    attribute_error: Option<FluentBuildError>,
    options: Option<RequestOptions>,
    asynchronous_indicator: Option<u32>,
}

impl<'a> CreateKeyPairBuilder<'a> {
    fn new(client: Option<&'a mut Client>) -> Self {
        Self {
            client,
            algorithm: None,
            length: None,
            private_usage_mask: None,
            public_usage_mask: None,
            private_name: None,
            public_name: None,
            common_attributes: AttributeSet::new(),
            private_attributes: AttributeSet::new(),
            public_attributes: AttributeSet::new(),
            attribute_error: None,
            options: None,
            asynchronous_indicator: None,
        }
    }

    /// Creates a reusable builder without binding it to a client.
    ///
    /// A detached builder can construct typed requests and batch items for a
    /// later exchange. Calling [`Self::execute`] on it returns a local
    /// `NotSent` error because no client is attached.
    #[must_use]
    pub fn detached() -> Self {
        Self::new(None)
    }

    /// Selects an explicit common KMIP Cryptographic Algorithm.
    #[must_use]
    pub const fn algorithm(mut self, algorithm: CryptographicAlgorithm) -> Self {
        self.algorithm = Some(algorithm);
        self
    }

    /// Selects an explicit common KMIP Cryptographic Length in bits.
    #[must_use]
    pub const fn length(mut self, length: i32) -> Self {
        self.length = Some(length);
        self
    }

    /// Selects an explicit private key Cryptographic Usage Mask.
    #[must_use]
    pub const fn private_usage_mask(mut self, usage_mask: CryptographicUsageMask) -> Self {
        self.private_usage_mask = Some(usage_mask);
        self
    }

    /// Sets the raw KMIP Cryptographic Usage Mask bits.
    #[must_use]
    pub const fn private_usage_mask_bits(self, bits: i32) -> Self {
        self.private_usage_mask(CryptographicUsageMask::from_raw(bits))
    }

    /// Selects an explicit public key Cryptographic Usage Mask.
    #[must_use]
    pub const fn public_usage_mask(mut self, usage_mask: CryptographicUsageMask) -> Self {
        self.public_usage_mask = Some(usage_mask);
        self
    }

    /// Sets the raw KMIP Cryptographic Usage Mask bits.
    #[must_use]
    pub const fn public_usage_mask_bits(self, bits: i32) -> Self {
        self.public_usage_mask(CryptographicUsageMask::from_raw(bits))
    }

    /// Adds a private key Name using Uninterpreted Text String.
    #[must_use]
    pub fn private_name(mut self, name: &str) -> Self {
        self.private_name = Some(name.to_owned());
        self
    }

    /// Adds a public key Name using Uninterpreted Text String.
    #[must_use]
    pub fn public_name(mut self, name: &str) -> Self {
        self.public_name = Some(name.to_owned());
        self
    }

    /// Appends a typed Common Attribute after the explicit algorithm and length.
    #[must_use]
    pub fn common_attribute<D: AttributeDefinition>(mut self, attribute: Attribute<D>) -> Self {
        if self.common_attributes.try_push_typed(attribute).is_err() {
            self.attribute_error
                .get_or_insert(FluentBuildError::InvalidAttributeValue);
        }
        self
    }

    /// Appends a typed Private Key Attribute after its usage mask and name.
    #[must_use]
    pub fn private_attribute<D: AttributeDefinition>(mut self, attribute: Attribute<D>) -> Self {
        if self.private_attributes.try_push_typed(attribute).is_err() {
            self.attribute_error
                .get_or_insert(FluentBuildError::InvalidAttributeValue);
        }
        self
    }

    /// Appends a typed Public Key Attribute after its usage mask and name.
    #[must_use]
    pub fn public_attribute<D: AttributeDefinition>(mut self, attribute: Attribute<D>) -> Self {
        if self.public_attributes.try_push_typed(attribute).is_err() {
            self.attribute_error
                .get_or_insert(FluentBuildError::InvalidAttributeValue);
        }
        self
    }

    /// Appends prebuilt Common Attributes after the semantic algorithm and length.
    ///
    /// Preserves item order and appends after earlier typed attributes or sets.
    /// Final request validation rejects duplicate or conflicting values.
    #[must_use]
    pub fn common_attributes(mut self, attributes: AttributeSet) -> Self {
        for item in attributes.into_items() {
            if !self.accept_object_attribute_tag(&item) {
                continue;
            }
            if self.common_attributes.try_push(item).is_err() {
                self.attribute_error // COVERAGE-EXCLUDE: source AttributeSet items already passed this validator.
                    .get_or_insert(FluentBuildError::InvalidAttributeValue); // COVERAGE-EXCLUDE: repeat validation cannot fail.
            }
        }
        self
    }

    /// Appends prebuilt Private Key Attributes after the semantic usage mask and name.
    ///
    /// Preserves item order and appends after earlier typed attributes or sets.
    /// Final request validation rejects duplicate or conflicting values.
    #[must_use]
    pub fn private_key_attributes(mut self, attributes: AttributeSet) -> Self {
        for item in attributes.into_items() {
            if !self.accept_object_attribute_tag(&item) {
                continue;
            }
            if self.private_attributes.try_push(item).is_err() {
                self.attribute_error // COVERAGE-EXCLUDE: source AttributeSet items already passed this validator.
                    .get_or_insert(FluentBuildError::InvalidAttributeValue); // COVERAGE-EXCLUDE: repeat validation cannot fail.
            }
        }
        self
    }

    /// Appends prebuilt Public Key Attributes after the semantic usage mask and name.
    ///
    /// Preserves item order and appends after earlier typed attributes or sets.
    /// Final request validation rejects duplicate or conflicting values.
    #[must_use]
    pub fn public_key_attributes(mut self, attributes: AttributeSet) -> Self {
        for item in attributes.into_items() {
            if !self.accept_object_attribute_tag(&item) {
                continue;
            }
            if self.public_attributes.try_push(item).is_err() {
                self.attribute_error // COVERAGE-EXCLUDE: source AttributeSet items already passed this validator.
                    .get_or_insert(FluentBuildError::InvalidAttributeValue); // COVERAGE-EXCLUDE: repeat validation cannot fail.
            }
        }
        self
    }

    /// Sets exchange-local options without changing client or batch policy.
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

    /// Validates the accumulated inputs locally without consuming the builder.
    ///
    /// Uses the same construction and typed request validation as
    /// [`Self::build_request`] without invoking the transport.
    ///
    /// # Errors
    ///
    /// Returns the same payload-free error as [`Self::build_request`].
    pub fn validate(&self) -> Result<(), FluentBuildError> {
        self.build_request().map(|_| ())
    }

    /// Builds the existing typed Create Key Pair request without sending it.
    ///
    /// # Errors
    ///
    /// Returns a payload-free error for missing fields or invalid attributes.
    pub fn build(self) -> Result<CreateKeyPairRequest, FluentBuildError> {
        self.build_request()
    }

    /// Builds a typed request without consuming the accumulated inputs.
    ///
    /// # Errors
    ///
    /// Returns a payload-free error for missing fields or invalid attributes.
    pub fn build_request(&self) -> Result<CreateKeyPairRequest, FluentBuildError> {
        if let Some(error) = self.attribute_error {
            return Err(error);
        }
        let algorithm = self
            .algorithm
            .ok_or(FluentBuildError::MissingCreateKeyPairAlgorithm)?;
        let length = self
            .length
            .ok_or(FluentBuildError::MissingCreateKeyPairLength)?;
        let mut common = AttributeSet::new();
        common
            .try_push_typed(
                Attribute::<typed::CryptographicAlgorithm>::new(algorithm.raw())
                    .map_err(|_| FluentBuildError::InvalidAttributeValue)?, // COVERAGE-EXCLUDE: static tag and Enumeration are valid.
            )
            .map_err(|_| FluentBuildError::InvalidAttributeValue)?; // COVERAGE-EXCLUDE: typed Item matches its catalog marker.
        common
            .try_push_typed(
                Attribute::<typed::CryptographicLength>::new(length)
                    .map_err(|_| FluentBuildError::InvalidAttributeValue)?, // COVERAGE-EXCLUDE: static tag and Integer are valid.
            )
            .map_err(|_| FluentBuildError::InvalidAttributeValue)?; // COVERAGE-EXCLUDE: typed Item matches its catalog marker.
        append_key_pair_attributes(&mut common, &self.common_attributes)?;
        let private = key_pair_attributes(
            self.private_usage_mask,
            self.private_name.as_deref(),
            &self.private_attributes,
        )?;
        let public = key_pair_attributes(
            self.public_usage_mask,
            self.public_name.as_deref(),
            &self.public_attributes,
        )?;
        let mut request = CreateKeyPairRequest::new().with_common_attributes(common);
        if !private.is_empty() {
            request = request.with_private_key_attributes(private);
        }
        if !public.is_empty() {
            request = request.with_public_key_attributes(public);
        }
        if !request.validate().is_valid() {
            return Err(FluentBuildError::InvalidRequest);
        }
        Ok(request)
    }

    /// Converts the request to an existing typed batch item.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError`] when request construction fails.
    pub fn to_batch_item(self) -> Result<ClientBatchItem, FluentBuildError> {
        self.build_request()
            .map(|request| ClientBatchItem::new(ClientRequest::CreateKeyPair(request)))
    }

    /// Executes one typed Create Key Pair exchange using the existing response.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] for local validation or client exchange failure.
    pub fn execute(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let request = self.build_request().map_err(|error| {
            ClientError::validation(
                ClientCauseCategory::InvalidInput,
                RequestDeliveryState::NotSent,
                error,
            )
        })?;
        let Some(client) = self.client else {
            return Err(ClientError::validation(
                ClientCauseCategory::InvalidInput,
                RequestDeliveryState::NotSent,
                FluentBuildError::MissingClientForExecution,
            ));
        };
        client.execute_one_typed_request_with_async_indicator(
            ClientRequest::CreateKeyPair(request),
            self.asynchronous_indicator,
            limits,
            &self.options.unwrap_or_default(),
        )
    }

    fn accept_object_attribute_tag(&mut self, item: &Item) -> bool {
        if let Err(error) = validate_object_attribute_tag(item) {
            self.attribute_error.get_or_insert(error);
            return false;
        }
        true
    }
}

fn key_pair_attributes(
    usage_mask: Option<CryptographicUsageMask>,
    name: Option<&str>,
    additional: &AttributeSet,
) -> Result<AttributeSet, FluentBuildError> {
    let mut attributes = AttributeSet::new();
    if let Some(usage_mask) = usage_mask {
        attributes
            .try_push_typed(
                Attribute::<typed::CryptographicUsageMask>::new(usage_mask.raw())
                    .map_err(|_| FluentBuildError::InvalidAttributeValue)?, // COVERAGE-EXCLUDE: static tag and Integer are valid.
            )
            .map_err(|_| FluentBuildError::InvalidAttributeValue)?; // COVERAGE-EXCLUDE: typed Item matches its catalog marker.
    }
    if let Some(name) = name {
        attributes
            .try_push_typed(
                Attribute::<typed::Name>::new(name_attribute(name.to_owned())?)
                    .map_err(|_| FluentBuildError::InvalidAttributeValue)?, // COVERAGE-EXCLUDE: name_attribute builds valid Name TTLV.
            )
            .map_err(|_| FluentBuildError::InvalidAttributeValue)?; // COVERAGE-EXCLUDE: typed Item matches its catalog marker.
    }
    append_key_pair_attributes(&mut attributes, additional)?;
    Ok(attributes)
}

fn append_key_pair_attributes(
    attributes: &mut AttributeSet,
    additional: &AttributeSet,
) -> Result<(), FluentBuildError> {
    for item in additional.as_items() {
        attributes.try_push(
            item.try_clone()
                .map_err(|_| FluentBuildError::InvalidTtlvValue)?, // COVERAGE-EXCLUDE: source Item respects clone depth limit.
        )?; // COVERAGE-EXCLUDE: cloned AttributeSet Item was already validated.
    }
    Ok(())
}

/// A fluent builder for an existing typed Get request.
pub struct GetBuilder<'a> {
    client: &'a mut Client,
    request: GetRequest,
    options: Option<RequestOptions>,
    asynchronous_indicator: Option<u32>,
}

impl GetBuilder<'_> {
    /// Sets exchange-local options.
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

    /// Selects the object identifier. Leaving it unset uses the KMIP ID Placeholder.
    #[must_use]
    pub fn unique_identifier(mut self, value: UniqueIdentifier) -> Self {
        self.request = self.request.with_unique_identifier(value);
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

    /// Executes one typed Get exchange and returns its existing response.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] when request validation, transport, or response
    /// decoding fails.
    pub fn execute(
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

#[derive(Default)]
struct NewAttributeBuilderState {
    item: Option<Item>,
    error: Option<FluentBuildError>,
}

impl NewAttributeBuilderState {
    fn set_item(&mut self, item: Item) {
        if self.item.is_some() || self.error.is_some() {
            self.error
                .get_or_insert(FluentBuildError::InvalidAttributeValue);
            return;
        }
        if let Err(error) = validate_object_attribute_tag(&item) {
            self.error = Some(error);
            return;
        }
        self.item = Some(item);
    }

    fn set_vendor_attribute(
        &mut self,
        vendor_identification: &str,
        attribute_name: &str,
        attribute_value: &Item,
    ) {
        match make_vendor_attribute(vendor_identification, attribute_name, attribute_value) {
            Ok(item) => self.set_item(item),
            Err(error) => {
                self.error.get_or_insert(error);
            }
        }
    }

    fn build_item(&self, missing_error: FluentBuildError) -> Result<Item, FluentBuildError> {
        if let Some(error) = self.error {
            return Err(error);
        }
        self.item
            .as_ref()
            .ok_or(missing_error)?
            .try_clone()
            .map_err(|_| FluentBuildError::InvalidTtlvValue)
    }
}

struct AttributeMutationBuilderState<'a> {
    client: Option<&'a mut Client>,
    unique_identifier: Option<String>,
    attribute: NewAttributeBuilderState,
    options: Option<RequestOptions>,
    asynchronous_indicator: Option<u32>,
}

impl<'a> AttributeMutationBuilderState<'a> {
    fn new(client: Option<&'a mut Client>) -> Self {
        Self {
            client,
            unique_identifier: None,
            attribute: NewAttributeBuilderState::default(),
            options: None,
            asynchronous_indicator: None,
        }
    }

    fn set_unique_identifier(&mut self, value: &str) {
        self.unique_identifier = Some(value.to_owned());
    }

    fn set_catalogued_attribute<D: AttributeDefinition>(&mut self, attribute: Attribute<D>) {
        self.attribute.set_item(attribute.into_item());
    }

    fn set_attribute_item(&mut self, item: Item) {
        self.attribute.set_item(item);
    }

    fn set_vendor_attribute(
        &mut self,
        vendor_identification: &str,
        attribute_name: &str,
        attribute_value: &Item,
    ) {
        self.attribute
            .set_vendor_attribute(vendor_identification, attribute_name, attribute_value);
    }

    fn set_options(&mut self, options: RequestOptions) {
        self.options = Some(options);
    }

    const fn set_asynchronous_indicator(&mut self, value: u32) {
        self.asynchronous_indicator = Some(value);
    }
}

fn build_attribute_mutation_request<T>(
    state: &AttributeMutationBuilderState<'_>,
    missing_attribute_error: FluentBuildError,
    build: impl FnOnce(Option<String>, NewAttribute) -> T,
    validate: impl FnOnce(&T) -> Result<(), FluentBuildError>,
) -> Result<T, FluentBuildError> {
    let item = state.attribute.build_item(missing_attribute_error)?;
    let request = build(state.unique_identifier.clone(), NewAttribute::new(item));
    validate(&request)?;
    Ok(request)
}

fn execute_attribute_mutation<T>(
    client: Option<&mut Client>,
    request: Result<T, FluentBuildError>,
    make_client_request: impl FnOnce(T) -> ClientRequest,
    asynchronous_indicator: Option<u32>,
    options: Option<RequestOptions>,
    limits: &kmipkit_ttlv::codec::CodecLimits,
) -> Result<ClientBatchItemResponse, ClientError> {
    let request = request.map_err(fluent_client_validation_error)?;
    let Some(client) = client else {
        return Err(fluent_client_validation_error(
            FluentBuildError::MissingClientForExecution,
        ));
    };
    execute_fluent_request(
        client,
        make_client_request(request),
        asynchronous_indicator,
        options,
        limits,
    )
}

/// A fluent builder for the existing typed Add Attribute request.
///
/// The required New Attribute is one direct typed Item. Catalogued attributes
/// can be supplied with [`Self::attribute`], unknown attributes require an
/// explicit [`Self::attribute_item`], and Vendor Attribute uses its dedicated
/// structure constructor.
pub struct AddAttributeBuilder<'a> {
    state: AttributeMutationBuilderState<'a>,
}

/// A fluent builder for the existing typed Delete Attribute request.
///
/// KMIP v2.1 §6.1.13 Table 202 permits the Unique Identifier, Current
/// Attribute, and Attribute Reference fields to be omitted independently.
/// Catalogued current values use [`Self::current_attribute`]; unknown values
/// require an explicitly typed TTLV Item.
pub struct DeleteAttributeBuilder<'a> {
    client: Option<&'a mut Client>,
    unique_identifier: Option<String>,
    current_attribute: Option<Item>,
    current_attribute_error: Option<FluentBuildError>,
    attribute_reference: Option<AttributeReference>,
    options: Option<RequestOptions>,
    asynchronous_indicator: Option<u32>,
}

impl<'a> DeleteAttributeBuilder<'a> {
    fn new(client: Option<&'a mut Client>) -> Self {
        Self {
            client,
            unique_identifier: None,
            current_attribute: None,
            current_attribute_error: None,
            attribute_reference: None,
            options: None,
            asynchronous_indicator: None,
        }
    }

    /// Creates a detached builder for request construction and batch use.
    #[must_use]
    pub fn detached() -> Self {
        Self::new(None)
    }

    /// Sets the optional exact text Unique Identifier.
    #[must_use]
    pub fn unique_identifier(mut self, value: &str) -> Self {
        self.unique_identifier = Some(value.to_owned());
        self
    }

    /// Selects a catalogued Current Attribute value using its typed value.
    #[must_use]
    pub fn current_attribute<D: AttributeDefinition>(mut self, attribute: Attribute<D>) -> Self {
        self.set_current_attribute(attribute.into_item());
        self
    }

    /// Selects a Current Attribute using an explicitly typed generic TTLV Item.
    #[must_use]
    pub fn current_attribute_item(mut self, item: Item) -> Self {
        self.set_current_attribute(item);
        self
    }

    /// Constructs a KMIP Vendor Attribute Current Attribute selector.
    #[must_use]
    pub fn current_vendor_attribute(
        mut self,
        vendor_identification: &str,
        attribute_name: &str,
        attribute_value: &Item,
    ) -> Self {
        match make_vendor_attribute(vendor_identification, attribute_name, attribute_value) {
            Ok(item) => self.set_current_attribute(item),
            Err(error) => {
                self.current_attribute_error.get_or_insert(error);
            }
        }
        self
    }

    /// Sets the optional exact KMIP Attribute Reference.
    #[must_use]
    pub fn attribute_reference(mut self, value: AttributeReference) -> Self {
        self.attribute_reference = Some(value);
        self
    }

    /// Sets exchange-local transport options.
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

    fn set_current_attribute(&mut self, item: Item) {
        if self.current_attribute.is_some() || self.current_attribute_error.is_some() {
            self.current_attribute_error
                .get_or_insert(FluentBuildError::InvalidAttributeValue);
        } else {
            self.current_attribute = Some(item);
        }
    }

    fn construct_request(&self) -> Result<DeleteAttributeRequest, FluentBuildError> {
        if let Some(error) = self.current_attribute_error {
            return Err(error);
        }
        let current_attribute = self
            .current_attribute
            .as_ref()
            .map(Item::try_clone)
            .transpose()
            .map_err(|_| FluentBuildError::InvalidTtlvValue)?
            .map(CurrentAttribute::new);
        let request = DeleteAttributeRequest::new(
            self.unique_identifier.clone(),
            current_attribute,
            self.attribute_reference.clone(),
        );
        validate_fluent_request(&request)?;
        Ok(request)
    }

    /// Validates the typed request without sending it.
    ///
    /// # Errors
    ///
    /// Returns an error when a selector is malformed or duplicated, or the
    /// existing typed request validator rejects its fields.
    pub fn validate(&self) -> Result<(), FluentBuildError> {
        self.construct_request().map(|_| ())
    }

    /// Builds the existing typed Delete Attribute request without sending it.
    ///
    /// # Errors
    ///
    /// Returns an error when local selector or typed request validation fails.
    pub fn build_request(&self) -> Result<DeleteAttributeRequest, FluentBuildError> {
        self.construct_request()
    }

    /// Builds the existing typed Delete Attribute request without sending it.
    ///
    /// # Errors
    ///
    /// Returns an error when local selector or typed request validation fails.
    pub fn build(self) -> Result<DeleteAttributeRequest, FluentBuildError> {
        self.construct_request()
    }

    /// Converts the request to the existing typed batch item.
    ///
    /// # Errors
    ///
    /// Returns an error when local selector or typed request validation fails.
    pub fn to_batch_item(self) -> Result<ClientBatchItem, FluentBuildError> {
        Ok(ClientBatchItem::new(ClientRequest::DeleteAttribute(
            self.construct_request()?,
        )))
    }

    /// Executes one typed Delete Attribute exchange through the existing client.
    ///
    /// # Errors
    ///
    /// Returns a local `NotSent` validation error, or the existing protocol or
    /// transport error with its delivery state.
    pub fn execute(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let request = self
            .construct_request()
            .map_err(fluent_client_validation_error)?;
        let Some(client) = self.client else {
            return Err(fluent_client_validation_error(
                FluentBuildError::MissingClientForExecution,
            ));
        };
        execute_fluent_request(
            client,
            ClientRequest::DeleteAttribute(request),
            self.asynchronous_indicator,
            self.options,
            limits,
        )
    }
}

impl AddAttributeBuilder<'_> {
    fn construct_request(&self) -> Result<AddAttributeRequest, FluentBuildError> {
        build_attribute_mutation_request(
            &self.state,
            FluentBuildError::MissingAddAttributeAttribute,
            AddAttributeRequest::new,
            validate_fluent_request,
        )
    }

    /// Validates the complete typed request without transport or consumption.
    ///
    /// # Errors
    ///
    /// Returns an error when the required New Attribute is absent or the
    /// existing typed request validation rejects the supplied attribute.
    pub fn validate(&self) -> Result<(), FluentBuildError> {
        self.construct_request().map(|_| ())
    }

    /// Builds the existing typed Add Attribute request without sending it.
    ///
    /// # Errors
    ///
    /// Returns an error when local attribute or typed request validation fails.
    pub fn build_request(&self) -> Result<AddAttributeRequest, FluentBuildError> {
        self.construct_request()
    }

    /// Builds the existing typed Add Attribute request without sending it.
    ///
    /// # Errors
    ///
    /// Returns an error when local attribute or typed request validation fails.
    pub fn build(self) -> Result<AddAttributeRequest, FluentBuildError> {
        self.construct_request()
    }

    /// Converts the request to the existing typed batch item.
    ///
    /// # Errors
    ///
    /// Returns an error when local attribute or typed request validation fails.
    pub fn to_batch_item(self) -> Result<ClientBatchItem, FluentBuildError> {
        Ok(ClientBatchItem::new(ClientRequest::AddAttribute(
            self.construct_request()?,
        )))
    }

    /// Executes one typed Add Attribute exchange through the existing client.
    ///
    /// # Errors
    ///
    /// Returns a local `NotSent` validation error, or the existing protocol or
    /// transport error with its delivery state.
    pub fn execute(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let request = self.construct_request();
        execute_attribute_mutation(
            self.state.client,
            request,
            ClientRequest::AddAttribute,
            self.state.asynchronous_indicator,
            self.state.options,
            limits,
        )
    }
}

/// A fluent builder for the existing typed Set Attribute request.
pub struct SetAttributeBuilder<'a> {
    state: AttributeMutationBuilderState<'a>,
}

trait AttributeMutationBuilderMethods<'client>: Sized {
    fn state_mut(&mut self) -> &mut AttributeMutationBuilderState<'client>;

    fn update_unique_identifier(mut self, value: &str) -> Self {
        self.state_mut().set_unique_identifier(value);
        self
    }

    fn update_catalogued_attribute<D: AttributeDefinition>(
        mut self,
        attribute: Attribute<D>,
    ) -> Self {
        self.state_mut().set_catalogued_attribute(attribute);
        self
    }

    fn update_attribute_item(mut self, item: Item) -> Self {
        self.state_mut().set_attribute_item(item);
        self
    }

    fn update_vendor_attribute(
        mut self,
        vendor_identification: &str,
        attribute_name: &str,
        attribute_value: &Item,
    ) -> Self {
        self.state_mut().set_vendor_attribute(
            vendor_identification,
            attribute_name,
            attribute_value,
        );
        self
    }

    fn update_options(mut self, options: RequestOptions) -> Self {
        self.state_mut().set_options(options);
        self
    }
}

impl<'client> AttributeMutationBuilderMethods<'client> for AddAttributeBuilder<'client> {
    fn state_mut(&mut self) -> &mut AttributeMutationBuilderState<'client> {
        &mut self.state
    }
}

impl<'client> AttributeMutationBuilderMethods<'client> for SetAttributeBuilder<'client> {
    fn state_mut(&mut self) -> &mut AttributeMutationBuilderState<'client> {
        &mut self.state
    }
}

impl<'a> AddAttributeBuilder<'a> {
    fn new(client: Option<&'a mut Client>) -> Self {
        Self {
            state: AttributeMutationBuilderState::new(client),
        }
    }

    /// Creates a detached builder for typed request and batch use.
    ///
    /// Executing it returns a local `NotSent` error because no client is attached.
    #[must_use]
    pub fn detached() -> Self {
        Self::new(None)
    }

    /// Sets an optional exact text Unique Identifier.
    #[must_use]
    pub fn unique_identifier(self, value: &str) -> Self {
        <AddAttributeBuilder<'a> as AttributeMutationBuilderMethods<'a>>::update_unique_identifier(
            self, value,
        )
    }

    /// Sets one catalogued New Attribute with its typed value.
    #[must_use]
    pub fn attribute<D: AttributeDefinition>(self, attribute: Attribute<D>) -> Self {
        <AddAttributeBuilder<'a> as AttributeMutationBuilderMethods<'a>>::update_catalogued_attribute(
            self, attribute,
        )
    }

    /// Sets one explicitly typed TTLV Item as the New Attribute.
    #[must_use]
    pub fn attribute_item(self, item: Item) -> Self {
        <AddAttributeBuilder<'a> as AttributeMutationBuilderMethods<'a>>::update_attribute_item(
            self, item,
        )
    }

    /// Builds a KMIP Vendor Attribute structure as the New Attribute.
    #[must_use]
    pub fn vendor_attribute(
        self,
        vendor_identification: &str,
        attribute_name: &str,
        attribute_value: &Item,
    ) -> Self {
        <AddAttributeBuilder<'a> as AttributeMutationBuilderMethods<'a>>::update_vendor_attribute(
            self,
            vendor_identification,
            attribute_name,
            attribute_value,
        )
    }

    /// Sets exchange-local transport options.
    #[must_use]
    pub fn with_options(self, options: RequestOptions) -> Self {
        <AddAttributeBuilder<'a> as AttributeMutationBuilderMethods<'a>>::update_options(
            self, options,
        )
    }

    /// Sets the Request Header Asynchronous Indicator for this exchange.
    #[must_use]
    pub const fn with_asynchronous_indicator(mut self, value: u32) -> Self {
        self.state.set_asynchronous_indicator(value);
        self
    }
}

impl<'a> SetAttributeBuilder<'a> {
    fn new(client: Option<&'a mut Client>) -> Self {
        Self {
            state: AttributeMutationBuilderState::new(client),
        }
    }

    /// Creates a detached builder for typed request and batch use.
    ///
    /// Executing it returns a local `NotSent` error because no client is attached.
    #[must_use]
    pub fn detached() -> Self {
        Self::new(None)
    }

    /// Sets an optional exact text Unique Identifier.
    #[must_use]
    pub fn unique_identifier(self, value: &str) -> Self {
        <SetAttributeBuilder<'a> as AttributeMutationBuilderMethods<'a>>::update_unique_identifier(
            self, value,
        )
    }

    /// Sets one catalogued New Attribute with its typed value.
    #[must_use]
    pub fn attribute<D: AttributeDefinition>(self, attribute: Attribute<D>) -> Self {
        <SetAttributeBuilder<'a> as AttributeMutationBuilderMethods<'a>>::update_catalogued_attribute(
            self, attribute,
        )
    }

    /// Sets one explicitly typed TTLV Item as the New Attribute.
    #[must_use]
    pub fn attribute_item(self, item: Item) -> Self {
        <SetAttributeBuilder<'a> as AttributeMutationBuilderMethods<'a>>::update_attribute_item(
            self, item,
        )
    }

    /// Builds a KMIP Vendor Attribute structure as the New Attribute.
    #[must_use]
    pub fn vendor_attribute(
        self,
        vendor_identification: &str,
        attribute_name: &str,
        attribute_value: &Item,
    ) -> Self {
        <SetAttributeBuilder<'a> as AttributeMutationBuilderMethods<'a>>::update_vendor_attribute(
            self,
            vendor_identification,
            attribute_name,
            attribute_value,
        )
    }

    /// Sets exchange-local transport options.
    #[must_use]
    pub fn with_options(self, options: RequestOptions) -> Self {
        <SetAttributeBuilder<'a> as AttributeMutationBuilderMethods<'a>>::update_options(
            self, options,
        )
    }

    /// Sets the Request Header Asynchronous Indicator for this exchange.
    #[must_use]
    pub const fn with_asynchronous_indicator(mut self, value: u32) -> Self {
        self.state.set_asynchronous_indicator(value);
        self
    }
}

impl SetAttributeBuilder<'_> {
    fn construct_request(&self) -> Result<SetAttributeRequest, FluentBuildError> {
        build_attribute_mutation_request(
            &self.state,
            FluentBuildError::MissingSetAttributeAttribute,
            SetAttributeRequest::new,
            validate_fluent_request,
        )
    }

    /// Validates the complete typed request without transport or consumption.
    ///
    /// # Errors
    /// Returns an error if the required attribute or typed request is invalid.
    pub fn validate(&self) -> Result<(), FluentBuildError> {
        self.construct_request().map(|_| ())
    }

    /// Builds the existing typed Set Attribute request.
    ///
    /// # Errors
    /// Returns an error if the required attribute or typed request is invalid.
    pub fn build_request(&self) -> Result<SetAttributeRequest, FluentBuildError> {
        self.construct_request()
    }

    /// Builds the existing typed Set Attribute request without sending it.
    ///
    /// # Errors
    /// Returns an error if the required attribute or typed request is invalid.
    pub fn build(self) -> Result<SetAttributeRequest, FluentBuildError> {
        self.construct_request()
    }

    /// Converts the request to the existing typed batch item.
    ///
    /// # Errors
    /// Returns an error if the required attribute or typed request is invalid.
    pub fn to_batch_item(self) -> Result<ClientBatchItem, FluentBuildError> {
        Ok(ClientBatchItem::new(ClientRequest::SetAttribute(
            self.construct_request()?,
        )))
    }

    /// Executes one typed Set Attribute exchange.
    ///
    /// # Errors
    /// Returns a payload-free local validation error with `NotSent`, or the
    /// existing protocol or transport error with its delivery state.
    pub fn execute(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let request = self.construct_request();
        execute_attribute_mutation(
            self.state.client,
            request,
            ClientRequest::SetAttribute,
            self.state.asynchronous_indicator,
            self.state.options,
            limits,
        )
    }
}

/// A fluent builder for the existing typed Get Attributes request.
pub struct GetAttributesBuilder<'a> {
    client: &'a mut Client,
    unique_identifier: Option<String>,
    attribute_references: Vec<AttributeReference>,
    options: Option<RequestOptions>,
    asynchronous_indicator: Option<u32>,
}

impl<'a> GetAttributesBuilder<'a> {
    fn new(client: &'a mut Client) -> Self {
        Self {
            client,
            unique_identifier: None,
            attribute_references: Vec::new(),
            options: None,
            asynchronous_indicator: None,
        }
    }

    /// Selects a text Unique Identifier. Omit this method to preserve the
    /// protocol's optional identifier or batch placeholder behavior.
    #[must_use]
    pub fn unique_identifier(mut self, value: &str) -> Self {
        self.unique_identifier = Some(value.to_owned());
        self
    }

    /// Appends an Attribute Reference, preserving caller order.
    #[must_use]
    pub fn attribute_reference(mut self, value: AttributeReference) -> Self {
        self.attribute_references.push(value);
        self
    }

    /// Sets exchange-local transport options.
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

    fn construct_request(&self) -> Result<GetAttributesRequest, FluentBuildError> {
        let request = GetAttributesRequest::try_new(
            self.unique_identifier.clone(),
            self.attribute_references.iter().cloned(),
        )
        .map_err(|_| FluentBuildError::InvalidRequest)?;
        validate_fluent_request(&request)?;
        Ok(request)
    }

    /// Validates the complete typed request without transport or consumption.
    ///
    /// # Errors
    /// Returns an error if the typed request is invalid.
    pub fn validate(&self) -> Result<(), FluentBuildError> {
        self.construct_request().map(|_| ())
    }

    /// Builds the existing typed Get Attributes request.
    ///
    /// # Errors
    /// Returns an error if the typed request is invalid.
    pub fn build_request(&self) -> Result<GetAttributesRequest, FluentBuildError> {
        self.construct_request()
    }

    /// Builds the existing typed Get Attributes request without sending it.
    ///
    /// # Errors
    /// Returns an error if the typed request is invalid.
    pub fn build(self) -> Result<GetAttributesRequest, FluentBuildError> {
        self.construct_request()
    }

    /// Converts the request to the existing typed batch item.
    ///
    /// # Errors
    /// Returns an error if the typed request is invalid.
    pub fn to_batch_item(self) -> Result<ClientBatchItem, FluentBuildError> {
        Ok(ClientBatchItem::new(ClientRequest::GetAttributes(
            self.construct_request()?,
        )))
    }

    /// Executes one typed Get Attributes exchange and returns its operation-
    /// specific response view.
    ///
    /// # Errors
    ///
    /// Returns a payload-free local validation error with `NotSent`, or the
    /// existing protocol or transport error with its delivery state.
    pub fn execute(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let request = self
            .construct_request()
            .map_err(fluent_client_validation_error)?;
        execute_fluent_request(
            self.client,
            ClientRequest::GetAttributes(request),
            self.asynchronous_indicator,
            self.options,
            limits,
        )
    }
}

/// A fluent builder for the existing typed Get Attribute List request.
pub struct GetAttributeListBuilder<'a> {
    client: &'a mut Client,
    unique_identifier: Option<String>,
    options: Option<RequestOptions>,
    asynchronous_indicator: Option<u32>,
}

impl<'a> GetAttributeListBuilder<'a> {
    fn new(client: &'a mut Client) -> Self {
        Self {
            client,
            unique_identifier: None,
            options: None,
            asynchronous_indicator: None,
        }
    }

    /// Selects a text Unique Identifier. Omit this method to preserve the
    /// protocol's optional identifier or batch placeholder behavior.
    #[must_use]
    pub fn unique_identifier(mut self, value: &str) -> Self {
        self.unique_identifier = Some(value.to_owned());
        self
    }

    /// Sets exchange-local transport options.
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

    fn construct_request(&self) -> Result<GetAttributeListRequest, FluentBuildError> {
        let request = GetAttributeListRequest::new(self.unique_identifier.clone());
        validate_fluent_request(&request)?;
        Ok(request)
    }

    /// Validates the complete typed request without transport or consumption.
    ///
    /// # Errors
    /// Returns an error if the typed request is invalid.
    pub fn validate(&self) -> Result<(), FluentBuildError> {
        self.construct_request().map(|_| ())
    }

    /// Builds the existing typed Get Attribute List request.
    ///
    /// # Errors
    /// Returns an error if the typed request is invalid.
    pub fn build_request(&self) -> Result<GetAttributeListRequest, FluentBuildError> {
        self.construct_request()
    }

    /// Builds the existing typed Get Attribute List request without sending it.
    ///
    /// # Errors
    /// Returns an error if the typed request is invalid.
    pub fn build(self) -> Result<GetAttributeListRequest, FluentBuildError> {
        self.construct_request()
    }

    /// Converts the request to the existing typed batch item.
    ///
    /// # Errors
    /// Returns an error if the typed request is invalid.
    pub fn to_batch_item(self) -> Result<ClientBatchItem, FluentBuildError> {
        Ok(ClientBatchItem::new(ClientRequest::GetAttributeList(
            self.construct_request()?,
        )))
    }

    /// Executes one typed Get Attribute List exchange and returns its
    /// operation-specific response view.
    ///
    /// # Errors
    ///
    /// Returns a payload-free local validation error with `NotSent`, or the
    /// existing protocol or transport error with its delivery state.
    pub fn execute(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let request = self
            .construct_request()
            .map_err(fluent_client_validation_error)?;
        execute_fluent_request(
            self.client,
            ClientRequest::GetAttributeList(request),
            self.asynchronous_indicator,
            self.options,
            limits,
        )
    }
}

impl DestroyBuilder<'_> {
    /// Sets exchange-local options.
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

    /// Executes one typed Destroy exchange and returns its existing response.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] when request validation, transport, or response
    /// decoding fails.
    pub fn execute(
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

fn validate_fluent_request<R: ValidateKmipRequest>(request: &R) -> Result<(), FluentBuildError> {
    if request.validate().is_valid() {
        Ok(())
    } else {
        Err(FluentBuildError::InvalidRequest)
    }
}

fn fluent_client_validation_error(error: FluentBuildError) -> ClientError {
    ClientError::validation(
        ClientCauseCategory::InvalidInput,
        RequestDeliveryState::NotSent,
        error,
    )
}

fn execute_fluent_request(
    client: &mut Client,
    request: ClientRequest,
    asynchronous_indicator: Option<u32>,
    options: Option<RequestOptions>,
    limits: &kmipkit_ttlv::codec::CodecLimits,
) -> Result<ClientBatchItemResponse, ClientError> {
    let options = options.unwrap_or_default();
    client.execute_one_typed_request_with_async_indicator(
        request,
        asynchronous_indicator,
        limits,
        &options,
    )
}

/// A fluent builder for an existing typed Activate request.
pub struct ActivateBuilder<'a> {
    client: &'a mut Client,
    request: ActivateRequest,
    options: Option<RequestOptions>,
    asynchronous_indicator: Option<u32>,
}

impl<'a> ActivateBuilder<'a> {
    fn new(client: &'a mut Client) -> Self {
        Self {
            client,
            request: ActivateRequest::new(None),
            options: None,
            asynchronous_indicator: None,
        }
    }

    /// Sets the optional Unique Identifier in the typed request.
    #[must_use]
    pub fn unique_identifier(mut self, value: UniqueIdentifier) -> Self {
        self.request = ActivateRequest::new(Some(value));
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

    /// Executes exactly one typed request exchange.
    ///
    /// # Errors
    ///
    /// Returns a validation error before sending when the request is invalid, or the existing
    /// client error for exchange and response failures.
    pub fn execute(
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
    fn new(client: &'a mut Client) -> Self {
        Self {
            client,
            request: ArchiveRequest::new(None),
            options: None,
            asynchronous_indicator: None,
        }
    }

    /// Sets the optional Unique Identifier in the typed request.
    #[must_use]
    pub fn unique_identifier(mut self, value: UniqueIdentifier) -> Self {
        self.request = ArchiveRequest::new(Some(value));
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

    /// Executes exactly one typed request exchange.
    ///
    /// # Errors
    ///
    /// Returns a validation error before sending when the request is invalid, or the existing
    /// client error for exchange and response failures.
    pub fn execute(
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
    fn new(client: &'a mut Client) -> Self {
        Self {
            client,
            request: RecoverRequest::new(None),
            options: None,
            asynchronous_indicator: None,
        }
    }

    /// Sets the optional Unique Identifier in the typed request.
    #[must_use]
    pub fn unique_identifier(mut self, value: UniqueIdentifier) -> Self {
        self.request = RecoverRequest::new(Some(value));
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

    /// Executes exactly one typed request exchange.
    ///
    /// # Errors
    ///
    /// Returns a validation error before sending when the request is invalid, or the existing
    /// client error for exchange and response failures.
    pub fn execute(
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
    fn new(client: &'a mut Client, reason: RevocationReason) -> Self {
        Self {
            client,
            request: RevokeRequest::new(reason),
            options: None,
            asynchronous_indicator: None,
        }
    }

    /// Sets the optional Unique Identifier in the typed request.
    #[must_use]
    pub fn unique_identifier(mut self, value: UniqueIdentifier) -> Self {
        self.request = self.request.with_unique_identifier(value);
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

    /// Executes exactly one typed Revoke exchange.
    ///
    /// # Errors
    ///
    /// Returns a validation error before sending when the typed request is
    /// invalid, or the existing client error for exchange and response failures.
    pub fn execute(
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

    /// Executes exactly one typed Revoke exchange with the supplied options.
    ///
    /// # Errors
    ///
    /// Returns a validation error before sending when the request is invalid,
    /// or the existing client error for exchange and response failures.
    pub fn execute_with_options(
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
    fn new(client: &'a mut Client, attributes: AttributeSet) -> Self {
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
        let maximum_items = self.request.maximum_items();
        let offset_items = self.request.offset_items();
        let storage_status_mask = self.request.storage_status_mask();
        let object_group_member = self.request.object_group_member();

        let mut request = LocateRequest::new(attributes);
        if let Some(value) = maximum_items {
            request = request.with_maximum_items(value);
        }
        if let Some(value) = offset_items {
            request = request.with_offset_items(value);
        }
        if let Some(value) = storage_status_mask {
            request = request.with_storage_status_mask(value);
        }
        if let Some(value) = object_group_member {
            request = request.with_object_group_member(value);
        }
        self.request = request;
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

    /// Executes exactly one typed Locate exchange.
    ///
    /// # Errors
    ///
    /// Returns a validation error before sending when the typed request is
    /// invalid, or the existing client error for exchange and response failures.
    pub fn execute(
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

    /// Executes exactly one typed Locate exchange with the supplied options.
    ///
    /// # Errors
    ///
    /// Returns a validation error before sending when the request is invalid,
    /// or the existing client error for exchange and response failures.
    pub fn execute_with_options(
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
    fn new(client: &'a mut Client) -> Self {
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
    fn new(client: &'a mut Client) -> Self {
        Self {
            state: ReplacementBuilderState::new(client),
        }
    }

    /// Sets the optional source key identifier in the typed request.
    #[must_use]
    pub fn unique_identifier(mut self, value: UniqueIdentifier) -> Self {
        self.state.request = std::mem::take(&mut self.state.request).with_unique_identifier(value);
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

    /// Executes exactly one typed request exchange.
    ///
    /// # Errors
    ///
    /// Returns a validation error before sending or the existing client error for
    /// exchange and response failures.
    pub fn execute(
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
    fn new(client: &'a mut Client) -> Self {
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

    /// Executes exactly one typed request exchange.
    ///
    /// # Errors
    ///
    /// Returns a validation error before sending when the request is invalid, or the existing
    /// client error for exchange and response failures.
    pub fn execute(
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
    fn new(client: &'a mut Client) -> Self {
        Self {
            state: ReplacementBuilderState::new(client),
        }
    }

    /// Selects the existing certificate to renew.
    #[must_use]
    pub fn unique_identifier(mut self, value: UniqueIdentifier) -> Self {
        self.state.request = std::mem::take(&mut self.state.request).with_unique_identifier(value);
        self
    }

    /// Selects a certificate request object by its Unique Identifier.
    #[must_use]
    pub fn certificate_request_unique_identifier(mut self, value: UniqueIdentifier) -> Self {
        self.state.request = std::mem::take(&mut self.state.request)
            .with_certificate_request_unique_identifier(value);
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

    /// Executes exactly one typed request exchange.
    ///
    /// # Errors
    ///
    /// Returns a validation error before sending or the existing client error for
    /// exchange and response failures.
    pub fn execute(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        self.state.run(limits)
    }
}

fn item(raw_tag: u32, value: Value) -> Result<Item, FluentBuildError> {
    let tag: Tag = RawTag::new(raw_tag)
        .and_then(|tag| tag.try_checked())
        .map_err(|_| FluentBuildError::InvalidTtlvValue)?;
    Item::new(tag, value).map_err(|_| FluentBuildError::InvalidTtlvValue)
}

fn name_attribute(value: String) -> Result<Structure, FluentBuildError> {
    let mut fields = Structure::new();
    fields
        .try_push(item(
            NAME_TYPE_TAG,
            Value::enumeration(NAME_TYPE_UNINTERPRETED_TEXT_STRING),
        )?)
        .map_err(|_| FluentBuildError::InvalidTtlvValue)?;
    fields
        .try_push(item(NAME_VALUE_TAG, Value::text_string(value))?)
        .map_err(|_| FluentBuildError::InvalidTtlvValue)?;
    Ok(fields)
}

fn clone_value(value: ValueView<'_>) -> Result<Value, FluentBuildError> {
    match value {
        ValueView::Structure(structure) => {
            let mut fields = Structure::new();
            for child in structure.children() {
                fields
                    .try_push(
                        child
                            .try_clone()
                            .map_err(|_| FluentBuildError::InvalidTtlvValue)?,
                    )
                    .map_err(|_| FluentBuildError::InvalidTtlvValue)?;
            }
            Ok(Value::structure(fields))
        }
        ValueView::Integer(value) => Ok(Value::integer(*value)),
        ValueView::LongInteger(value) => Ok(Value::long_integer(*value)),
        ValueView::BigInteger(value) => Ok(Value::big_integer(value.to_vec())),
        ValueView::Enumeration(value) => Ok(Value::enumeration(*value)),
        ValueView::Boolean(value) => Ok(Value::boolean(*value)),
        ValueView::TextString(value) => Ok(Value::text_string(value.to_owned())),
        ValueView::ByteString(value) => Ok(Value::byte_string(value.to_vec())),
        ValueView::DateTime(value) => Ok(Value::date_time(*value)),
        ValueView::Interval(value) => Ok(Value::interval(*value)),
        ValueView::DateTimeExtended(value) => Ok(Value::date_time_extended(*value)),
        _ => Err(FluentBuildError::InvalidTtlvValue),
    }
}

fn validate_object_attribute_tag(item: &Item) -> Result<(), FluentBuildError> {
    let raw_tag = item.tag().raw();
    let is_extension_tag =
        (MESSAGE_EXTENSION_TAG_START..=MESSAGE_EXTENSION_TAG_END).contains(&raw_tag);
    if !is_extension_tag && !crate::attribute_tags_generated::is_catalogued_attribute_tag(raw_tag) {
        return Err(FluentBuildError::InvalidAttributeValue);
    }
    Ok(())
}

fn append_object_attribute(
    attributes: &mut AttributeSet,
    attribute_error: &mut Option<FluentBuildError>,
    item: Item,
) {
    if attribute_error.is_some() {
        return;
    }
    if let Err(error) = validate_object_attribute_tag(&item) {
        *attribute_error = Some(error);
        return;
    }
    if attributes.try_push(item).is_err() {
        *attribute_error = Some(FluentBuildError::InvalidAttributeValue);
    }
}

fn set_positioned_attribute(
    positioned_attributes: &mut Vec<(usize, Item)>,
    attribute_error: &mut Option<FluentBuildError>,
    position: FluentAttributePosition,
    item: Item,
) {
    if attribute_error.is_some() {
        return;
    }
    let FluentAttributePosition::ObjectAttributes { index } = position;
    if let Err(error) = validate_object_attribute_tag(&item) {
        *attribute_error = Some(error);
        return;
    }
    if positioned_attributes
        .iter()
        .any(|(existing, _)| *existing == index)
    {
        *attribute_error = Some(FluentBuildError::DuplicateAttributePosition);
        return;
    }
    positioned_attributes.push((index, item));
}

fn insert_positioned_attributes(
    attributes: AttributeSet,
    positioned_attributes: &[(usize, Item)],
) -> Result<AttributeSet, FluentBuildError> {
    if positioned_attributes.is_empty() {
        return Ok(attributes);
    }
    let final_len = attributes
        .len()
        .checked_add(positioned_attributes.len())
        .ok_or(FluentBuildError::AttributePositionOutOfBounds)?;
    let mut positioned = positioned_attributes.iter().collect::<Vec<_>>();
    positioned.sort_by_key(|(index, _)| *index);
    if positioned.iter().any(|(index, _)| *index >= final_len) {
        return Err(FluentBuildError::AttributePositionOutOfBounds);
    }
    let mut ordered = attributes
        .as_items()
        .iter()
        .map(Item::try_clone)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| FluentBuildError::InvalidTtlvValue)?;
    for (index, item) in positioned {
        let copy = item
            .try_clone()
            .map_err(|_| FluentBuildError::InvalidTtlvValue)?;
        if *index > ordered.len() {
            return Err(FluentBuildError::AttributePositionOutOfBounds);
        }
        ordered.insert(*index, copy);
    }
    AttributeSet::try_new(ordered).map_err(|_| FluentBuildError::InvalidAttributeValue)
}

fn make_vendor_attribute(
    vendor_identification: &str,
    attribute_name: &str,
    attribute_value: &Item,
) -> Result<Item, FluentBuildError> {
    let value = attribute_value.with_value(clone_value)?;
    let mut fields = Structure::new();
    fields
        .try_push(item(
            VENDOR_IDENTIFICATION_TAG,
            Value::text_string(vendor_identification.to_owned()),
        )?)
        .map_err(|_| FluentBuildError::InvalidTtlvValue)?;
    fields
        .try_push(item(
            ATTRIBUTE_NAME_TAG,
            Value::text_string(attribute_name.to_owned()),
        )?)
        .map_err(|_| FluentBuildError::InvalidTtlvValue)?;
    fields
        .try_push(item(ATTRIBUTE_VALUE_TAG, value)?)
        .map_err(|_| FluentBuildError::InvalidTtlvValue)?;
    item(VENDOR_ATTRIBUTE_TAG, Value::structure(fields))
}

impl Client {
    /// Returns the scoped fluent request namespace for common typed operations.
    #[must_use]
    pub fn request(&mut self) -> RequestBuilder<'_> {
        RequestBuilder::new(self)
    }
}

impl From<AttributeSetError> for FluentBuildError {
    fn from(_: AttributeSetError) -> Self {
        Self::InvalidAttributeValue
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
    fn new(client: &'a mut Client) -> Self {
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
    pub fn unique_identifier(mut self, value: UniqueIdentifier) -> Self {
        self.unique_identifier = Some(value);
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

    /// Executes one Check exchange and returns the existing operation result.
    ///
    /// # Errors
    ///
    /// Returns a not-sent validation error before transport if request
    /// validation fails, or the existing client error for exchange and decoding failures.
    pub fn execute(
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
    fn new(client: &'a mut Client) -> Self {
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

    /// Executes one typed exchange and returns the existing operation result.
    ///
    /// # Errors
    ///
    /// Returns a not-sent validation error before transport if the request is
    /// invalid, or the existing client error for exchange and decoding failures.
    pub fn execute(
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
    fn new(client: &'a mut Client) -> Self {
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

    /// Executes one typed exchange and returns the existing operation result.
    ///
    /// # Errors
    ///
    /// Returns a not-sent validation error before transport if the offer is
    /// invalid, or the existing client error for exchange and decoding failures.
    pub fn execute(
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
    fn new(client: &'a mut Client) -> Self {
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
    pub fn unique_identifier(mut self, value: UniqueIdentifier) -> Self {
        self.unique_identifier = Some(value);
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

    /// Executes one typed exchange and returns the existing operation result.
    ///
    /// # Errors
    ///
    /// Returns a not-sent validation error before transport if validation
    /// fails, or the existing client error for exchange and decoding failures.
    pub fn execute(
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

/// A fluent builder for the empty typed Get Constraints request.
fn copy_new_attribute(attribute: &NewAttribute) -> Result<NewAttribute, FluentBuildError> {
    attribute
        .item()
        .try_clone()
        .map(NewAttribute::new)
        .map_err(|_| FluentBuildError::InvalidTtlvValue)
}

fn copy_adjustment_value(
    value: &AttributeAdjustmentValue,
) -> Result<AttributeAdjustmentValue, FluentBuildError> {
    value.try_clone()
}

/// A fluent builder for the existing typed Modify Attribute request.
pub struct ModifyAttributeBuilder<'a> {
    client: &'a mut Client,
    unique_identifier: Option<String>,
    current_attribute: Option<CurrentAttribute>,
    new_attribute: Option<NewAttribute>,
    options: Option<RequestOptions>,
    asynchronous_indicator: Option<u32>,
}

impl<'a> ModifyAttributeBuilder<'a> {
    fn new(client: &'a mut Client) -> Self {
        Self {
            client,
            unique_identifier: None,
            current_attribute: None,
            new_attribute: None,
            options: None,
            asynchronous_indicator: None,
        }
    }

    /// Sets the optional KMIP Unique Identifier as caller-supplied text.
    #[must_use]
    pub fn unique_identifier(mut self, value: &str) -> Self {
        self.unique_identifier = Some(value.to_owned());
        self
    }

    /// Sets the optional typed Current Attribute selector.
    #[must_use]
    pub fn current_attribute(mut self, value: CurrentAttribute) -> Self {
        self.current_attribute = Some(value);
        self
    }

    /// Sets the required typed New Attribute.
    #[must_use]
    pub fn new_attribute(mut self, value: NewAttribute) -> Self {
        self.new_attribute = Some(value);
        self
    }

    fluent_option_field_setters!("Sets exchange-local options without changing client policy."; options; asynchronous_indicator; const);

    /// Validates completeness and protocol constraints without sending.
    ///
    /// # Errors
    ///
    /// Returns a sanitized construction error for an absent New Attribute,
    /// an invalid typed request, or a value that cannot be copied safely.
    pub fn validate(&self) -> Result<(), FluentBuildError> {
        let new_attribute = self
            .new_attribute
            .as_ref()
            .ok_or(FluentBuildError::MissingModifyAttributeValue)?;
        let current_attribute = self
            .current_attribute
            .as_ref()
            .map(|attribute| {
                attribute
                    .item()
                    .try_clone()
                    .map(CurrentAttribute::new)
                    .map_err(|_| FluentBuildError::InvalidTtlvValue)
            })
            .transpose()?;
        let request = ModifyAttributeRequest::new(
            self.unique_identifier.clone(),
            current_attribute,
            copy_new_attribute(new_attribute)?,
        );
        validate_fluent_request(&request)
    }

    /// Builds the existing typed KMIP request without sending it.
    ///
    /// # Errors
    ///
    /// Returns a sanitized construction error for an absent New Attribute
    /// or a request rejected by the existing protocol validator.
    pub fn build(self) -> Result<ModifyAttributeRequest, FluentBuildError> {
        self.build_request()
    }

    /// Compatibility alias for [`Self::build`].
    ///
    /// # Errors
    ///
    /// Returns a sanitized construction error for an absent New Attribute
    /// or a request rejected by the existing protocol validator.
    pub fn build_request(self) -> Result<ModifyAttributeRequest, FluentBuildError> {
        let new_attribute = self
            .new_attribute
            .ok_or(FluentBuildError::MissingModifyAttributeValue)?;
        let request = ModifyAttributeRequest::new(
            self.unique_identifier,
            self.current_attribute,
            new_attribute,
        );
        validate_fluent_request(&request)?;
        Ok(request)
    }

    /// Converts this request to the existing typed client batch item.
    ///
    /// # Errors
    ///
    /// Returns a sanitized construction error when the request is invalid.
    pub fn to_batch_item(self) -> Result<ClientBatchItem, FluentBuildError> {
        self.build_request()
            .map(|request| ClientBatchItem::new(ClientRequest::ModifyAttribute(request)))
    }

    /// Executes one typed exchange and returns the existing operation result.
    ///
    /// # Errors
    ///
    /// Returns a not-sent validation error before transport when the request
    /// is invalid, or the existing client error for exchange and decoding failures.
    pub fn execute(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let Self {
            client,
            unique_identifier,
            current_attribute,
            new_attribute,
            options,
            asynchronous_indicator,
        } = self;
        let new_attribute = new_attribute
            .ok_or(FluentBuildError::MissingModifyAttributeValue)
            .map_err(fluent_client_validation_error)?;
        let request =
            ModifyAttributeRequest::new(unique_identifier, current_attribute, new_attribute);
        validate_fluent_request(&request).map_err(fluent_client_validation_error)?;
        execute_fluent_request(
            client,
            ClientRequest::ModifyAttribute(request),
            asynchronous_indicator,
            options,
            limits,
        )
    }
}

/// A fluent builder for the existing typed Adjust Attribute request.
pub struct AdjustAttributeBuilder<'a> {
    client: &'a mut Client,
    unique_identifier: Option<String>,
    attribute_reference: Option<AttributeReference>,
    adjustment_type: Option<AdjustmentType>,
    adjustment_value: Option<AttributeAdjustmentValue>,
    options: Option<RequestOptions>,
    asynchronous_indicator: Option<u32>,
}

impl<'a> AdjustAttributeBuilder<'a> {
    fn new(client: &'a mut Client) -> Self {
        Self {
            client,
            unique_identifier: None,
            attribute_reference: None,
            adjustment_type: None,
            adjustment_value: None,
            options: None,
            asynchronous_indicator: None,
        }
    }

    /// Sets the optional KMIP Unique Identifier as caller-supplied text.
    #[must_use]
    pub fn unique_identifier(mut self, value: &str) -> Self {
        self.unique_identifier = Some(value.to_owned());
        self
    }

    /// Sets the required explicit Attribute Reference.
    #[must_use]
    pub fn attribute_reference(mut self, value: AttributeReference) -> Self {
        self.attribute_reference = Some(value);
        self
    }

    /// Sets the required Adjustment Type without choosing a default.
    #[must_use]
    pub const fn adjustment_type(mut self, value: AdjustmentType) -> Self {
        self.adjustment_type = Some(value);
        self
    }

    /// Sets the optional caller-supplied TTLV Adjustment Value.
    #[must_use]
    pub fn adjustment_value(mut self, value: AttributeAdjustmentValue) -> Self {
        self.adjustment_value = Some(value);
        self
    }

    fluent_option_field_setters!("Sets exchange-local options without changing client policy."; options; asynchronous_indicator; const);

    /// Validates completeness and protocol constraints without sending.
    ///
    /// # Errors
    ///
    /// Returns a sanitized construction error for a missing required field, an
    /// invalid typed request, or a value that cannot be copied safely.
    pub fn validate(&self) -> Result<(), FluentBuildError> {
        let attribute_reference = self
            .attribute_reference
            .as_ref()
            .ok_or(FluentBuildError::MissingAdjustAttributeReference)?;
        let adjustment_type = self
            .adjustment_type
            .ok_or(FluentBuildError::MissingAdjustAttributeType)?;
        let adjustment_value = self
            .adjustment_value
            .as_ref()
            .map(copy_adjustment_value)
            .transpose()?;
        let request = AdjustAttributeRequest::new(
            self.unique_identifier.clone(),
            attribute_reference.clone(),
            adjustment_type,
            adjustment_value.map(AttributeAdjustmentValue::into_value),
        );
        validate_fluent_request(&request)
    }

    /// Builds the existing typed KMIP request without sending it.
    ///
    /// # Errors
    ///
    /// Returns a sanitized construction error for a missing required field or
    /// a request rejected by the existing protocol validator.
    pub fn build(self) -> Result<AdjustAttributeRequest, FluentBuildError> {
        self.build_request()
    }

    /// Compatibility alias for [`Self::build`].
    ///
    /// # Errors
    ///
    /// Returns a sanitized construction error for a missing required field or
    /// a request rejected by the existing protocol validator.
    pub fn build_request(self) -> Result<AdjustAttributeRequest, FluentBuildError> {
        let attribute_reference = self
            .attribute_reference
            .ok_or(FluentBuildError::MissingAdjustAttributeReference)?;
        let adjustment_type = self
            .adjustment_type
            .ok_or(FluentBuildError::MissingAdjustAttributeType)?;
        let request = AdjustAttributeRequest::new(
            self.unique_identifier,
            attribute_reference,
            adjustment_type,
            self.adjustment_value
                .map(AttributeAdjustmentValue::into_value),
        );
        validate_fluent_request(&request)?;
        Ok(request)
    }

    /// Converts this request to the existing typed client batch item.
    ///
    /// # Errors
    ///
    /// Returns a sanitized construction error when the request is invalid.
    pub fn to_batch_item(self) -> Result<ClientBatchItem, FluentBuildError> {
        self.build_request()
            .map(|request| ClientBatchItem::new(ClientRequest::AdjustAttribute(request)))
    }

    /// Executes one typed exchange and returns the existing operation result.
    ///
    /// # Errors
    ///
    /// Returns a not-sent validation error before transport when the request
    /// is invalid, or the existing client error for exchange and decoding failures.
    pub fn execute(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let Self {
            client,
            unique_identifier,
            attribute_reference,
            adjustment_type,
            adjustment_value,
            options,
            asynchronous_indicator,
        } = self;
        let attribute_reference = attribute_reference
            .ok_or(FluentBuildError::MissingAdjustAttributeReference)
            .map_err(fluent_client_validation_error)?;
        let adjustment_type = adjustment_type
            .ok_or(FluentBuildError::MissingAdjustAttributeType)
            .map_err(fluent_client_validation_error)?;
        let request = AdjustAttributeRequest::new(
            unique_identifier,
            attribute_reference,
            adjustment_type,
            adjustment_value.map(AttributeAdjustmentValue::into_value),
        );
        validate_fluent_request(&request).map_err(fluent_client_validation_error)?;
        execute_fluent_request(
            client,
            ClientRequest::AdjustAttribute(request),
            asynchronous_indicator,
            options,
            limits,
        )
    }
}

/// A fluent builder for an existing typed Activate request.
struct Kmip30BuilderExecution<'a> {
    client: &'a mut Client,
    options: Option<RequestOptions>,
    asynchronous_indicator: Option<u32>,
}

impl<'a> Kmip30BuilderExecution<'a> {
    fn new(client: &'a mut Client) -> Self {
        Self {
            client,
            options: None,
            asynchronous_indicator: None,
        }
    }

    fn execute(
        self,
        request: ClientRequest,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        execute_fluent_request(
            self.client,
            request,
            self.asynchronous_indicator,
            self.options,
            limits,
        )
    }
}

trait Kmip30BuilderParts<'a> {
    fn into_parts(self) -> (Kmip30BuilderExecution<'a>, ClientRequest);
    fn execution(&mut self) -> &mut Kmip30BuilderExecution<'a>;
}

macro_rules! impl_kmip30_builder_methods {
    ($builder:ident) => {
        impl<'a> $builder<'a> {
            /// Sets one-exchange options, including a caller-selected protocol version.
            #[must_use]
            pub fn with_options(mut self, options: RequestOptions) -> Self {
                self.execution().options = Some(options);
                self
            }

            /// Sets the Request Header Asynchronous Indicator for this exchange.
            #[must_use]
            pub fn with_asynchronous_indicator(mut self, value: u32) -> Self {
                self.execution().asynchronous_indicator = Some(value);
                self
            }

            /// Validates the semantic inputs held by the builder without sending.
            /// The selected protocol version and typed request are checked again before transport.
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

            /// Executes exactly one typed exchange.
            ///
            /// # Errors
            /// Returns the existing local validation, version, transport, or response error.
            pub fn execute(
                self,
                limits: &kmipkit_ttlv::codec::CodecLimits,
            ) -> Result<ClientBatchItemResponse, ClientError> {
                let (execution, request) = self.into_parts();
                execution.execute(request, limits)
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
    fn new(
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
impl_kmip30_builder_methods!(CreateCredentialBuilder);

/// A KMIP 3.0 Create Group fluent builder.
pub struct CreateGroupBuilder<'a> {
    execution: Kmip30BuilderExecution<'a>,
    attributes: AttributeSet,
}
impl<'a> CreateGroupBuilder<'a> {
    fn new(client: &'a mut Client, attributes: AttributeSet) -> Self {
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
impl_kmip30_builder_methods!(CreateGroupBuilder);

/// A KMIP 3.0 Create User fluent builder.
pub struct CreateUserBuilder<'a> {
    execution: Kmip30BuilderExecution<'a>,
    attributes: AttributeSet,
}
impl<'a> CreateUserBuilder<'a> {
    fn new(client: &'a mut Client, attributes: AttributeSet) -> Self {
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
impl_kmip30_builder_methods!(CreateUserBuilder);

/// A KMIP 3.0 Deactivate fluent builder.
pub struct DeactivateBuilder<'a> {
    execution: Kmip30BuilderExecution<'a>,
    unique_identifier: UniqueIdentifier,
    reason: Option<Structure>,
    date: Option<i64>,
}
impl<'a> DeactivateBuilder<'a> {
    fn new(client: &'a mut Client, unique_identifier: UniqueIdentifier) -> Self {
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
        let mut request = DeactivateRequest::new(self.unique_identifier);
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
impl_kmip30_builder_methods!(DeactivateBuilder);

/// A KMIP 3.0 Decapsulate fluent builder.
pub struct DecapsulateBuilder<'a> {
    execution: Kmip30BuilderExecution<'a>,
    unique_identifier: UniqueIdentifier,
    ciphertext: Vec<u8>,
    parameters: Option<CryptographicParameters>,
    attributes: Option<AttributeSet>,
}
impl<'a> DecapsulateBuilder<'a> {
    fn new(
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
    /// Sets the optional prevalidated ordered Attribute Set.
    #[must_use]
    pub fn attributes(mut self, value: AttributeSet) -> Self {
        self.attributes = Some(value);
        self
    }
}
impl<'a> Kmip30BuilderParts<'a> for DecapsulateBuilder<'a> {
    fn into_parts(self) -> (Kmip30BuilderExecution<'a>, ClientRequest) {
        let mut request = DecapsulateRequest::new(self.unique_identifier, self.ciphertext);
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
impl_kmip30_builder_methods!(DecapsulateBuilder);

/// A KMIP 3.0 Encapsulate fluent builder.
pub struct EncapsulateBuilder<'a> {
    execution: Kmip30BuilderExecution<'a>,
    unique_identifier: UniqueIdentifier,
    parameters: Option<CryptographicParameters>,
    attributes: Option<AttributeSet>,
}
impl<'a> EncapsulateBuilder<'a> {
    fn new(client: &'a mut Client, unique_identifier: UniqueIdentifier) -> Self {
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
    /// Sets the optional prevalidated ordered Attribute Set.
    #[must_use]
    pub fn attributes(mut self, value: AttributeSet) -> Self {
        self.attributes = Some(value);
        self
    }
}
impl<'a> Kmip30BuilderParts<'a> for EncapsulateBuilder<'a> {
    fn into_parts(self) -> (Kmip30BuilderExecution<'a>, ClientRequest) {
        let mut request = EncapsulateRequest::new(self.unique_identifier);
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
impl_kmip30_builder_methods!(EncapsulateBuilder);

/// A KMIP 3.0 Obliterate fluent builder.
pub struct ObliterateBuilder<'a> {
    execution: Kmip30BuilderExecution<'a>,
    unique_identifier: UniqueIdentifier,
}
impl<'a> ObliterateBuilder<'a> {
    fn new(client: &'a mut Client, unique_identifier: UniqueIdentifier) -> Self {
        Self {
            execution: Kmip30BuilderExecution::new(client),
            unique_identifier,
        }
    }
}
impl<'a> Kmip30BuilderParts<'a> for ObliterateBuilder<'a> {
    fn into_parts(self) -> (Kmip30BuilderExecution<'a>, ClientRequest) {
        (
            self.execution,
            ClientRequest::obliterate(ObliterateRequest::new(self.unique_identifier)),
        )
    }
    fn execution(&mut self) -> &mut Kmip30BuilderExecution<'a> {
        &mut self.execution
    }
}
impl_kmip30_builder_methods!(ObliterateBuilder);
