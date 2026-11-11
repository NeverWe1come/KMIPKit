//! Task-oriented, fluent builders over the existing typed KMIP requests.

use std::error::Error;
use std::fmt;
use std::ops::BitOr;

use kmipkit_protocol::attribute::typed;
use kmipkit_protocol::{
    Attribute, AttributeDefinition, AttributeSet, AttributeSetError, CreateKeyPairRequest,
    CreateRequest, CreateSplitKeyRequest, CryptographicAlgorithm, DestroyRequest, GetRequest,
    ManagedObject, ObjectType, OpaqueObjectContext, RegisterRequest, SplitKeyMethod,
    UniqueIdentifier, ValidateKmipRequest,
};
use kmipkit_transport::{RequestDeliveryState, RequestOptions};
use kmipkit_ttlv::{Item, RawTag, Structure, Tag, Value, ValueView};

use crate::{
    Client, ClientBatchItem, ClientBatchItemResponse, ClientCauseCategory, ClientError,
    ClientRequest,
};

const SYMMETRIC_KEY_OBJECT_TYPE: u32 = 2;
const MESSAGE_EXTENSION_TAG_START: u32 = 0x0054_0000;
const MESSAGE_EXTENSION_TAG_END: u32 = 0x0054_FFFF;
const NAME_TYPE_TAG: u32 = 0x0042_0097;
const NAME_VALUE_TAG: u32 = 0x0042_0098;
const NAME_TYPE_UNINTERPRETED_TEXT_STRING: u32 = 1;
const VENDOR_ATTRIBUTE_TAG: u32 = 0x0042_0008;
const VENDOR_IDENTIFICATION_TAG: u32 = 0x0042_009D;
const ATTRIBUTE_NAME_TAG: u32 = 0x0042_000A;
const ATTRIBUTE_VALUE_TAG: u32 = 0x0042_000B;

/// KMIP Cryptographic Usage Mask bits encoded as a signed TTLV Integer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CryptographicUsageMask(i32);

impl CryptographicUsageMask {
    /// Permit signing operations.
    pub const SIGN: Self = Self(0x0000_0001);
    /// Permit signature verification operations.
    pub const VERIFY: Self = Self(0x0000_0002);
    /// Permit encryption operations.
    pub const ENCRYPT: Self = Self(0x0000_0004);
    /// Permit decryption operations.
    pub const DECRYPT: Self = Self(0x0000_0008);
    /// Permit MAC generation operations.
    pub const MAC_GENERATE: Self = Self(0x0000_0080);
    /// Permit MAC verification operations.
    pub const MAC_VERIFY: Self = Self(0x0000_0100);
    /// Permit key derivation operations.
    pub const DERIVE_KEY: Self = Self(0x0000_0200);
    /// Permit key agreement operations.
    pub const KEY_AGREEMENT: Self = Self(0x0000_0800);

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
    /// Create Split Key is missing Split Key Method.
    MissingCreateSplitKeyMethod,
    /// Create Split Key is missing Split Key Parts.
    MissingCreateSplitKeyParts,
    /// Create Split Key is missing Split Key Threshold.
    MissingCreateSplitKeyThreshold,
    /// Register is missing its object.
    MissingRegisterObject,
    /// Register's object or context is invalid.
    InvalidRegisterObject,
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
            Self::MissingCreateSplitKeyMethod => "Create Split Key requires Split Key Method",
            Self::MissingCreateSplitKeyParts => "Create Split Key requires Split Key Parts",
            Self::MissingCreateSplitKeyThreshold => "Create Split Key requires Split Key Threshold",
            Self::MissingRegisterObject => "Register requires an object",
            Self::InvalidRegisterObject => "Register object or context is invalid",
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
pub struct RequestBuilder<'a> {
    client: &'a mut Client,
}

impl<'a> RequestBuilder<'a> {
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
        CreateKeyPairBuilder::new(self.client)
    }

    /// Begins constructing a typed Create Split Key request.
    #[must_use]
    pub fn create_split_key(self) -> CreateSplitKeyBuilder<'a> {
        CreateSplitKeyBuilder::new(self.client)
    }

    /// Begins constructing a typed Register request.
    #[must_use]
    pub fn register(self) -> RegisterBuilder<'a> {
        RegisterBuilder::new(self.client)
    }

    /// Begins constructing a typed Get request for an explicit identifier.
    #[must_use]
    pub fn get(self, unique_identifier: UniqueIdentifier) -> GetBuilder<'a> {
        GetBuilder {
            client: self.client,
            request: GetRequest::new().with_unique_identifier(unique_identifier),
            options: None,
            asynchronous_indicator: None,
        }
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
                append_object_attribute(&mut self.attributes, &mut self.attribute_error, item);
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
    client: &'a mut Client,
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
    fn new(client: &'a mut Client) -> Self {
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

    /// Selects an explicit public key Cryptographic Usage Mask.
    #[must_use]
    pub const fn public_usage_mask(mut self, usage_mask: CryptographicUsageMask) -> Self {
        self.public_usage_mask = Some(usage_mask);
        self
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
                self.attribute_error
                    .get_or_insert(FluentBuildError::InvalidAttributeValue);
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
                self.attribute_error
                    .get_or_insert(FluentBuildError::InvalidAttributeValue);
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
                self.attribute_error
                    .get_or_insert(FluentBuildError::InvalidAttributeValue);
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
                    .map_err(|_| FluentBuildError::InvalidAttributeValue)?,
            )
            .map_err(|_| FluentBuildError::InvalidAttributeValue)?;
        common
            .try_push_typed(
                Attribute::<typed::CryptographicLength>::new(length)
                    .map_err(|_| FluentBuildError::InvalidAttributeValue)?,
            )
            .map_err(|_| FluentBuildError::InvalidAttributeValue)?;
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
        self.client.execute_one_typed_request_with_async_indicator(
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
                    .map_err(|_| FluentBuildError::InvalidAttributeValue)?,
            )
            .map_err(|_| FluentBuildError::InvalidAttributeValue)?;
    }
    if let Some(name) = name {
        attributes
            .try_push_typed(
                Attribute::<typed::Name>::new(name_attribute(name.to_owned())?)
                    .map_err(|_| FluentBuildError::InvalidAttributeValue)?,
            )
            .map_err(|_| FluentBuildError::InvalidAttributeValue)?;
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
                .map_err(|_| FluentBuildError::InvalidTtlvValue)?,
        )?;
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
