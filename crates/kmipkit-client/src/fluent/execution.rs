use std::error::Error;
use std::fmt;
use std::ops::BitOr;

use kmipkit_protocol::named_values_generated::values::bitmask::cryptographic_usage_mask;
use kmipkit_protocol::named_values_generated::values::enumeration::alternative_name_type;
use kmipkit_protocol::{
    Attribute, AttributeSet, AttributeSetError, KmipOperationResult, LinkType, NewAttribute,
    ProtocolCauseCategory, ProtocolError, ProtocolErrorKind, ProtocolVersion, UniqueIdentifier,
    ValidateKmipRequest, attribute::typed,
};
use kmipkit_transport::{RequestDeliveryState, RequestOptions};
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::{Item, KmipTag, RawTag, Structure, Tag, Value, ValueView};

use crate::{
    Client, ClientBatchItemResponse, ClientBatchOutcome, ClientCauseCategory, ClientError,
    ClientRequest,
};

pub(super) const SYMMETRIC_KEY_OBJECT_TYPE: u32 = 2;
const MESSAGE_EXTENSION_TAG_START: u32 = 0x0054_0000;
const MESSAGE_EXTENSION_TAG_END: u32 = 0x0054_FFFF;
pub(super) const NAME_TYPE_TAG: u32 = KmipTag::NameType.as_u32();
pub(super) const NAME_VALUE_TAG: u32 = KmipTag::NameValue.as_u32();
pub(super) const NAME_TYPE_UNINTERPRETED_TEXT_STRING: u32 = 1;
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
    /// Import is missing its required Unique Identifier.
    MissingImportUniqueIdentifier,
    /// Import is missing its Object Type.
    MissingImportObjectType,
    /// Import is missing its payload object.
    MissingImportObject,
    /// Join Split Key is missing its Object Type.
    MissingJoinSplitKeyObjectType,
    /// Join Split Key requires at least one Split Key Reference.
    MissingJoinSplitKeyReferences,
    /// Delegated Login requires a Rights structure.
    MissingDelegatedLoginRights,
    /// PKCS#11 requires a Function structure.
    MissingPkcs11Function,
    /// Set Constraints requires a Constraints structure.
    MissingSetConstraints,
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
            Self::MissingImportUniqueIdentifier => "Import requires Unique Identifier",
            Self::MissingImportObjectType => "Import requires Object Type",
            Self::MissingImportObject => "Import requires an object",
            Self::MissingJoinSplitKeyObjectType => "Join Split Key requires Object Type",
            Self::MissingJoinSplitKeyReferences => {
                "Join Split Key requires at least one Split Key reference"
            }
            Self::MissingDelegatedLoginRights => "Delegated Login requires Rights",
            Self::MissingPkcs11Function => "PKCS#11 requires Function",
            Self::MissingSetConstraints => "Set Constraints requires Constraints",
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

    pub(super) fn try_clone(&self) -> Result<Self, FluentBuildError> {
        kmipkit_ttlv::try_clone_value(&self.0)
            .map(Self)
            .map_err(|_| FluentBuildError::InvalidTtlvValue)
    }

    pub(super) fn into_value(self) -> Value {
        self.0
    }
}

pub(super) struct FluentBuilderExecution<'a> {
    pub(super) client: &'a mut Client,
    pub(super) options: Option<RequestOptions>,
    pub(super) asynchronous_indicator: Option<u32>,
}

impl<'a> FluentBuilderExecution<'a> {
    pub(super) fn new(client: &'a mut Client) -> Self {
        Self {
            client,
            options: None,
            asynchronous_indicator: None,
        }
    }

    pub(super) fn with_options(mut self, options: RequestOptions) -> Self {
        self.options = Some(options);
        self
    }

    pub(super) fn with_protocol_version(mut self, version: ProtocolVersion) -> Self {
        let options = self.options.unwrap_or_default().with_protocol_version(version);
        self.options = Some(options);
        self
    }

    pub(super) fn protocol_version(&self) -> ProtocolVersion {
        effective_protocol_version(Some(self.client), self.options.as_ref())
    }

    pub(super) const fn with_asynchronous_indicator(mut self, value: u32) -> Self {
        self.asynchronous_indicator = Some(value);
        self
    }

    pub(super) fn run(
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

    pub(super) fn run_outcome(
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

    pub(super) fn run_typed<T>(
        self,
        request: ClientRequest,
        extract: impl FnOnce(ClientBatchOutcome) -> Option<T>,
        result_of: impl FnOnce(&T) -> &KmipOperationResult,
    ) -> Result<T, ClientError> {
        let item = self.run_outcome(request)?;
        unwrap_typed_outcome(item, extract, result_of)
    }
}

pub(super) fn effective_protocol_version(
    client: Option<&Client>,
    options: Option<&RequestOptions>,
) -> ProtocolVersion {
    options
        .and_then(RequestOptions::protocol_version)
        .or_else(|| client.map(Client::protocol_version))
        .unwrap_or(ProtocolVersion::V2_1)
}

pub(super) fn normalize_uid_for_version(
    identifier: UniqueIdentifier,
    version: ProtocolVersion,
) -> UniqueIdentifier {
    if version == ProtocolVersion::V3_0 {
        match identifier {
            UniqueIdentifier::TextString(value) => UniqueIdentifier::Identifier(value),
            other => other,
        }
    } else {
        identifier
    }
}

pub(super) fn name_attribute_item_for_version(
    name: &str,
    version: ProtocolVersion,
) -> Result<Item, FluentBuildError> {
    if version == ProtocolVersion::V3_0 {
        alternative_name_item(name, alternative_name_type::UNINTERPRETED_TEXT_STRING)
    } else {
        Attribute::<typed::Name>::new(name_attribute(name.to_owned())?)
            .map(Attribute::into_item)
            .map_err(|_| FluentBuildError::InvalidAttributeValue)
    }
}

pub(super) fn alternative_name_item(value: &str, name_type: u32) -> Result<Item, FluentBuildError> {
    let mut fields = Structure::new();
    fields
        .try_push(item(
            KmipTag::AlternativeNameValue.as_u32(),
            Value::text_string(value.to_owned()),
        )?)
        .map_err(|_| FluentBuildError::InvalidTtlvValue)?;
    fields
        .try_push(item(
            KmipTag::AlternativeNameType.as_u32(),
            Value::enumeration(name_type),
        )?)
        .map_err(|_| FluentBuildError::InvalidTtlvValue)?;
    item(KmipTag::AlternativeName.as_u32(), Value::structure(fields))
}

pub(super) fn link_item_for_version(
    link_type: LinkType,
    linked_object_identifier: UniqueIdentifier,
    version: ProtocolVersion,
) -> Result<Item, FluentBuildError> {
    if version == ProtocolVersion::V3_0 {
        let uid_value = match linked_object_identifier {
            UniqueIdentifier::TextString(value) | UniqueIdentifier::Identifier(value) => {
                Value::identifier(value)
            }
            UniqueIdentifier::Enumeration(_) | UniqueIdentifier::Integer(_) | _ => {
                return Err(FluentBuildError::InvalidAttributeValue);
            }
        };
        item(link_type.v3_tag(), uid_value)
    } else {
        let enum_value = link_type
            .try_v2_enumeration()
            .ok_or(FluentBuildError::InvalidAttributeValue)?;
        let uid_value = match linked_object_identifier {
            UniqueIdentifier::TextString(value) => Value::text_string(value),
            UniqueIdentifier::Identifier(value) => Value::identifier(value),
            UniqueIdentifier::Enumeration(value) => Value::enumeration(value),
            UniqueIdentifier::Integer(value) => Value::integer(value),
            _ => return Err(FluentBuildError::InvalidAttributeValue),
        };
        let mut fields = Structure::new();
        fields
            .try_push(item(
                KmipTag::LinkType.as_u32(),
                Value::enumeration(enum_value),
            )?)
            .map_err(|_| FluentBuildError::InvalidTtlvValue)?;
        fields
            .try_push(item(
                KmipTag::LinkedObjectIdentifier.as_u32(),
                uid_value,
            )?)
            .map_err(|_| FluentBuildError::InvalidTtlvValue)?;
        item(KmipTag::Link.as_u32(), Value::structure(fields))
    }
}

pub(super) fn validate_fluent_request<R: ValidateKmipRequest>(
    request: &R,
) -> Result<(), FluentBuildError> {
    if request.validate().is_valid() {
        Ok(())
    } else {
        Err(FluentBuildError::InvalidRequest)
    }
}

pub(super) fn fluent_client_validation_error(error: FluentBuildError) -> ClientError {
    ClientError::validation(
        ClientCauseCategory::InvalidInput,
        RequestDeliveryState::NotSent,
        error,
    )
}

pub(super) fn execute_fluent_request(
    client: &mut Client,
    request: ClientRequest,
    asynchronous_indicator: Option<u32>,
    options: Option<RequestOptions>,
    limits: &CodecLimits,
) -> Result<ClientBatchItemResponse, ClientError> {
    let options = options.unwrap_or_default();
    client.execute_one_typed_request_with_async_indicator(
        request,
        asynchronous_indicator,
        limits,
        &options,
    )
}

pub(super) fn execute_fluent_outcome(
    client: &mut Client,
    request: ClientRequest,
    asynchronous_indicator: Option<u32>,
    options: Option<RequestOptions>,
) -> Result<ClientBatchItemResponse, ClientError> {
    let options = options.unwrap_or_default();
    client.send_one_with_async_indicator(request, asynchronous_indicator, &options)
}

pub(super) fn unwrap_typed_outcome<T>(
    response: ClientBatchItemResponse,
    extract: impl FnOnce(ClientBatchOutcome) -> Option<T>,
    result_of: impl FnOnce(&T) -> &KmipOperationResult,
) -> Result<T, ClientError> {
    match response.into_outcome() {
        ClientBatchOutcome::Pending(pending) => Err(pending.into_client_error()),
        other => {
            let Some(typed) = extract(other) else {
                return Err(ClientError::protocol(
                    ProtocolError::new(
                        ProtocolErrorKind::MalformedMessage,
                        ProtocolCauseCategory::InvalidValue,
                        FluentBuildError::InvalidRequest,
                    ),
                    RequestDeliveryState::ResponseStarted,
                ));
            };
            let result = result_of(&typed);
            if result.status().raw() == 0 {
                Ok(typed)
            } else {
                Err(ClientError::server_result(result.clone()))
            }
        }
    }
}

pub(super) fn item(raw_tag: u32, value: Value) -> Result<Item, FluentBuildError> {
    let tag: Tag = RawTag::new(raw_tag)
        .and_then(|tag| tag.try_checked())
        .map_err(|_| FluentBuildError::InvalidTtlvValue)?;
    Item::new(tag, value).map_err(|_| FluentBuildError::InvalidTtlvValue)
}

pub(super) fn name_attribute(value: String) -> Result<Structure, FluentBuildError> {
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

pub(super) fn validate_object_attribute_tag(item: &Item) -> Result<(), FluentBuildError> {
    let raw_tag = item.tag().raw();
    let is_extension_tag =
        (MESSAGE_EXTENSION_TAG_START..=MESSAGE_EXTENSION_TAG_END).contains(&raw_tag);
    if !is_extension_tag && !crate::attribute_tags_generated::is_catalogued_attribute_tag(raw_tag) {
        return Err(FluentBuildError::InvalidAttributeValue);
    }
    Ok(())
}

pub(super) fn append_object_attribute(
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

pub(super) fn set_positioned_attribute(
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

pub(super) fn insert_positioned_attributes(
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

pub(super) fn make_vendor_attribute(
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

impl From<AttributeSetError> for FluentBuildError {
    fn from(_: AttributeSetError) -> Self {
        Self::InvalidAttributeValue
    }
}

pub(super) fn copy_new_attribute(attribute: &NewAttribute) -> Result<NewAttribute, FluentBuildError> {
    attribute
        .item()
        .try_clone()
        .map(NewAttribute::new)
        .map_err(|_| FluentBuildError::InvalidTtlvValue)
}

pub(super) fn copy_adjustment_value(
    value: &AttributeAdjustmentValue,
) -> Result<AttributeAdjustmentValue, FluentBuildError> {
    value.try_clone()
}
