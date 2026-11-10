//! Task-oriented, fluent builders over the existing typed KMIP requests.

use std::error::Error;
use std::fmt;
use std::ops::BitOr;

use kmipkit_protocol::attribute::typed;
use kmipkit_protocol::{
    Attribute, AttributeDefinition, AttributeSet, AttributeSetError, CreateRequest,
    CryptographicAlgorithm, DestroyRequest, GetRequest, ObjectType, UniqueIdentifier,
    ValidateKmipRequest,
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
    /// A catalogued attribute has an invalid TTLV type or structure.
    InvalidAttributeValue,
    /// A KMIP message extension tag was supplied as an Object Attribute.
    MessageExtensionTagNotObjectAttribute,
    /// A typed request failed its existing validation rules.
    InvalidRequest,
    /// A local TTLV value could not be copied into the fluent request.
    InvalidTtlvValue,
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
            Self::InvalidAttributeValue => "fluent Object Attribute value is invalid",
            Self::MessageExtensionTagNotObjectAttribute => {
                "KMIP message extension tags are not Object Attributes"
            }
            Self::InvalidRequest => "fluent request failed typed request validation",
            Self::InvalidTtlvValue => "fluent TTLV value could not be represented",
        })
    }
}

impl Error for FluentBuildError {}

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
        CreateSymmetricKeyBuilder::new(self.client)
    }

    /// Begins constructing a typed Get request for an explicit identifier.
    #[must_use]
    pub fn get(self, unique_identifier: impl Into<UniqueIdentifier>) -> GetBuilder<'a> {
        GetBuilder {
            client: self.client,
            request: GetRequest::new().with_unique_identifier(unique_identifier.into()),
            options: None,
            asynchronous_indicator: None,
        }
    }

    /// Begins constructing a typed Destroy request for an explicit identifier.
    #[must_use]
    pub fn destroy(self, unique_identifier: impl Into<UniqueIdentifier>) -> DestroyBuilder<'a> {
        DestroyBuilder {
            client: self.client,
            request: DestroyRequest::new(Some(unique_identifier.into())),
            options: None,
            asynchronous_indicator: None,
        }
    }
}

/// A Create Symmetric Key builder that returns the existing typed request.
pub struct CreateSymmetricKeyBuilder<'a> {
    client: &'a mut Client,
    algorithm: Option<CryptographicAlgorithm>,
    length: Option<i32>,
    usage_mask: Option<CryptographicUsageMask>,
    name: Option<String>,
    attributes: AttributeSet,
    attribute_error: Option<FluentBuildError>,
    options: Option<RequestOptions>,
    asynchronous_indicator: Option<u32>,
}

impl<'a> CreateSymmetricKeyBuilder<'a> {
    fn new(client: &'a mut Client) -> Self {
        Self {
            client,
            algorithm: None,
            length: None,
            usage_mask: None,
            name: None,
            attributes: AttributeSet::new(),
            attribute_error: None,
            options: None,
            asynchronous_indicator: None,
        }
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
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Adds a catalogued attribute constructed with its generated typed marker.
    #[must_use]
    pub fn attribute<D: AttributeDefinition>(mut self, attribute: Attribute<D>) -> Self {
        self.add_attribute_item(attribute.into_item());
        self
    }

    /// Adds an explicit TTLV Item as an expert Object Attribute.
    #[must_use]
    pub fn attribute_item(mut self, item: Item) -> Self {
        self.add_attribute_item(item);
        self
    }

    /// Builds the KMIP Vendor Attribute Structure from explicit components.
    #[must_use]
    pub fn vendor_attribute(
        mut self,
        vendor_identification: impl Into<String>,
        attribute_name: impl Into<String>,
        attribute_value: Item,
    ) -> Self {
        let value = match attribute_value.with_value(clone_value) {
            Ok(value) => value,
            Err(error) => {
                self.attribute_error.get_or_insert(error);
                return self;
            }
        };
        let structure = (|| {
            let mut fields = Structure::new();
            fields
                .try_push(item(
                    VENDOR_IDENTIFICATION_TAG,
                    Value::text_string(vendor_identification.into()),
                )?)
                .map_err(|_| FluentBuildError::InvalidTtlvValue)?;
            fields
                .try_push(item(
                    ATTRIBUTE_NAME_TAG,
                    Value::text_string(attribute_name.into()),
                )?)
                .map_err(|_| FluentBuildError::InvalidTtlvValue)?;
            fields
                .try_push(item(ATTRIBUTE_VALUE_TAG, value)?)
                .map_err(|_| FluentBuildError::InvalidTtlvValue)?;
            item(VENDOR_ATTRIBUTE_TAG, Value::structure(fields))
        })();
        match structure {
            Ok(item) => {
                self.add_attribute_item(item);
                self
            }
            Err(error) => {
                self.attribute_error.get_or_insert(error);
                self
            }
        }
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
    pub fn build(mut self) -> Result<CreateRequest, FluentBuildError> {
        self.build_request()
    }

    /// Converts the request to an existing typed batch item.
    pub fn to_batch_item(mut self) -> Result<ClientBatchItem, FluentBuildError> {
        self.build_request()
            .map(|request| ClientBatchItem::new(ClientRequest::Create(request)))
    }

    /// Executes one typed Create exchange and returns its existing response.
    pub fn execute(
        mut self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let request = self.build_request().map_err(|error| {
            ClientError::validation(
                ClientCauseCategory::InvalidInput,
                RequestDeliveryState::NotSent,
                error,
            )
        })?;
        let options = self.options.unwrap_or_default();
        self.client.execute_one_typed_request_with_async_indicator(
            ClientRequest::Create(request),
            self.asynchronous_indicator,
            limits,
            &options,
        )
    }

    fn build_request(&mut self) -> Result<CreateRequest, FluentBuildError> {
        if let Some(error) = self.attribute_error.take() {
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
        if let Some(name) = self.name.take() {
            attributes
                .try_push_typed(
                    Attribute::<typed::Name>::new(name_attribute(name)?)
                        .map_err(|_| FluentBuildError::InvalidAttributeValue)?,
                )
                .map_err(|_| FluentBuildError::InvalidAttributeValue)?;
        }
        for item in std::mem::replace(&mut self.attributes, AttributeSet::new()).into_items() {
            attributes
                .try_push(item)
                .map_err(|_| FluentBuildError::InvalidAttributeValue)?;
        }

        let request = CreateRequest::new(ObjectType::from_raw(SYMMETRIC_KEY_OBJECT_TYPE), attributes);
        if !request.validate().is_valid() {
            return Err(FluentBuildError::InvalidRequest);
        }
        Ok(request)
    }

    fn add_attribute_item(&mut self, item: Item) {
        if self.attribute_error.is_some() {
            return;
        }
        let raw_tag = item.tag().raw();
        if (MESSAGE_EXTENSION_TAG_START..=MESSAGE_EXTENSION_TAG_END).contains(&raw_tag) {
            self.attribute_error = Some(FluentBuildError::MessageExtensionTagNotObjectAttribute);
            return;
        }
        if self.attributes.try_push(item).is_err() {
            self.attribute_error = Some(FluentBuildError::InvalidAttributeValue);
        }
    }
}

/// A fluent builder for an existing typed Get request.
pub struct GetBuilder<'a> {
    client: &'a mut Client,
    request: GetRequest,
    options: Option<RequestOptions>,
    asynchronous_indicator: Option<u32>,
}

impl<'a> GetBuilder<'a> {
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

impl<'a> DestroyBuilder<'a> {
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
