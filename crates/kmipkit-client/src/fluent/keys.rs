//! Task-oriented, fluent builders over the existing typed KMIP requests.

use kmipkit_protocol::attribute::typed;
use kmipkit_protocol::{
    Attribute, AttributeDefinition, AttributeSet, CreateKeyPairRequest, CreateRequest,
    CreateSplitKeyRequest, CryptographicAlgorithm, DerivationMethod, DerivationParameters,
    DeriveKeyRequest, ExportRequest, ExportResponse, ImportRequest, ImportResponse,
    JoinSplitKeyRequest, JoinSplitKeyResponse, KdfSpec, KeyCompressionType, KeyFormatType, KeySpec,
    KeyWrapType, ManagedObject, ObjectType, OpaqueObjectContext, ProtocolVersion, RegisterRequest,
    SecretBytes, SplitKeyMethod, SplitKeyReference, UniqueIdentifier, ValidateKmipRequest,
};
use kmipkit_transport::{RequestDeliveryState, RequestOptions};
use kmipkit_ttlv::{Item, RawTag, Structure, Tag, Value};

use crate::{
    Client, ClientBatchItem, ClientBatchItemResponse, ClientBatchOutcome, ClientCauseCategory,
    ClientError, ClientRequest,
};

use super::execution::{
    CryptographicUsageMask, FluentAttributePosition, FluentBuildError, FluentBuilderExecution,
    SYMMETRIC_KEY_OBJECT_TYPE, alternative_name_item, append_object_attribute,
    effective_protocol_version, execute_fluent_outcome, fluent_client_validation_error,
    insert_positioned_attributes, link_item_for_version, make_vendor_attribute,
    name_attribute_item_for_version, normalize_uid_for_version, set_positioned_attribute,
    unwrap_typed_outcome, validate_fluent_request, validate_object_attribute_tag,
};

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
    pub(super) fn new(client: &'a mut Client) -> Self {
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

    fluent_option_field_setters!(
        "Sets exchange-local options without changing client policy.";
        execution.options;
        execution.asynchronous_indicator;
        const
    );

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
        let masks = match &self.protection_storage_masks {
            Some(masks) => Some(
                masks
                    .view()
                    .try_clone()
                    .map_err(|_| FluentBuildError::InvalidTtlvValue)?,
            ),
            None => None,
        };
        create_typed_request(object_type, attributes, masks)
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

    /// Executes at most one exchange using configured [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits)
    /// and returns the unwrapped [`CreateResponse`](kmipkit_protocol::CreateResponse).
    ///
    /// # Errors
    /// Returns the validation, protocol, transport, `ServerResult`, or `Pending` error.
    pub fn execute(self) -> Result<kmipkit_protocol::CreateResponse, ClientError> {
        let (request, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run_typed(
            ClientRequest::Create(request),
            |outcome| match outcome {
                ClientBatchOutcome::CreateCompleted(resp) => Some(resp),
                _ => None,
            },
            kmipkit_protocol::CreateResponse::result,
        )
    }

    /// Executes at most one exchange using configured [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits)
    /// and returns the raw [`ClientBatchItemResponse`] envelope.
    ///
    /// # Errors
    /// Returns the existing sanitized validation, protocol, or transport error.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let (request, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run_outcome(ClientRequest::Create(request))
    }

    /// Executes at most one exchange with explicit [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits) overrides.
    ///
    /// # Errors
    /// Returns the existing sanitized validation, protocol, or transport error
    /// with its request-delivery evidence.
    pub fn execute_outcome_with_limits(
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
    pub(super) fn new(client: &'a mut Client) -> Self {
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
    /// Appends one existing key or derivation source (`impl Into<UniqueIdentifier>`).
    /// Placeholder. The existing typed request validator enforces that rule.
    #[must_use]
    pub fn unique_identifier(mut self, identifier: impl Into<UniqueIdentifier>) -> Self {
        self.unique_identifiers.push(identifier.into());
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

    /// Applies algorithm, length, and key derivation parameters from a curated `KdfSpec`.
    #[must_use]
    pub fn with_kdf_spec(
        mut self,
        spec: KdfSpec,
        salt: Option<SecretBytes>,
        derivation_data: Option<SecretBytes>,
        iteration_count: Option<i32>,
    ) -> Self {
        self.derivation_method = Some(spec.derivation_method());
        let salt_bytes = salt.map(|s| s.with_bytes(<[u8]>::to_vec));
        let data_bytes = derivation_data.map(|d| d.with_bytes(<[u8]>::to_vec));
        if let Ok(params) = spec.to_derivation_parameters(salt_bytes, data_bytes, iteration_count) {
            self.derivation_parameters = Some(params);
        }
        self
    }

    /// Replaces the caller-selected output Attributes.
    #[must_use]
    pub fn attributes(mut self, attributes: AttributeSet) -> Self {
        self.attributes = Some(attributes);
        self
    }

    fluent_option_field_setters!(
        "Sets exchange-local options without changing client policy.";
        execution.options;
        execution.asynchronous_indicator;
        const
    );

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
        let version = self.execution.protocol_version();
        let unique_identifiers = self
            .unique_identifiers
            .into_iter()
            .map(|id| normalize_uid_for_version(id, version))
            .collect();
        let request = DeriveKeyRequest::new(
            self.object_type
                .ok_or(FluentBuildError::MissingDeriveKeyObjectType)?,
            unique_identifiers,
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

    /// Executes at most one exchange using configured [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits)
    /// and returns the unwrapped [`DeriveKeyResponse`](kmipkit_protocol::DeriveKeyResponse).
    ///
    /// # Errors
    /// Returns the validation, protocol, transport, `ServerResult`, or `Pending` error.
    pub fn execute(self) -> Result<kmipkit_protocol::DeriveKeyResponse, ClientError> {
        let (request, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run_typed(
            ClientRequest::DeriveKey(request),
            |outcome| match outcome {
                ClientBatchOutcome::DeriveKey(resp) => Some(resp),
                _ => None,
            },
            kmipkit_protocol::DeriveKeyResponse::result,
        )
    }

    /// Executes at most one exchange using configured [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits)
    /// and returns the raw [`ClientBatchItemResponse`] envelope.
    ///
    /// # Errors
    /// Returns the existing sanitized validation, protocol, or transport error.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let (request, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run_outcome(ClientRequest::DeriveKey(request))
    }

    /// Executes at most one exchange with explicit [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits) overrides.
    ///
    /// # Errors
    /// Returns the existing sanitized validation, protocol, or transport error
    /// with its request-delivery evidence.
    pub fn execute_outcome_with_limits(
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

/// A Register builder that validates and executes the existing typed request.
pub struct RegisterBuilder<'a> {
    client: &'a mut Client,
    object: Option<(ObjectType, Item)>,
    names: Vec<String>,
    alternative_names: Vec<(String, u32)>,
    links: Vec<(kmipkit_protocol::LinkType, UniqueIdentifier)>,
    attributes: AttributeSet,
    protection_storage_masks: Option<Structure>,
    opaque_object_context: Option<OpaqueObjectContext>,
    positioned_attributes: Vec<(usize, Item)>,
    attribute_error: Option<FluentBuildError>,
    options: Option<RequestOptions>,
    asynchronous_indicator: Option<u32>,
}

impl<'a> RegisterBuilder<'a> {
    pub(super) fn new(client: &'a mut Client) -> Self {
        Self {
            client,
            object: None,
            names: Vec::new(),
            alternative_names: Vec::new(),
            links: Vec::new(),
            attributes: AttributeSet::new(),
            protection_storage_masks: None,
            opaque_object_context: None,
            positioned_attributes: Vec::new(),
            attribute_error: None,
            options: None,
            asynchronous_indicator: None,
        }
    }

    /// Adds a version-aware KMIP object name (`Name` in 2.x, `AlternativeName` in 3.0).
    #[must_use]
    pub fn name(mut self, name: &str) -> Self {
        self.names.push(name.to_owned());
        self
    }

    /// Adds an explicit KMIP `AlternativeName` attribute (Uninterpreted Text String).
    #[must_use]
    pub fn alternative_name(self, value: &str) -> Self {
        self.alternative_name_typed(value, 1)
    }

    /// Adds an explicit KMIP `AlternativeName` attribute with an explicit name type code.
    #[must_use]
    pub fn alternative_name_typed(mut self, value: &str, name_type: u32) -> Self {
        self.alternative_names.push((value.to_owned(), name_type));
        self
    }

    /// Adds a version-aware KMIP `Link` attribute (`Link` in 2.x, typed link tag in 3.0).
    #[must_use]
    pub fn link(
        mut self,
        link_type: kmipkit_protocol::LinkType,
        linked_object_identifier: impl Into<UniqueIdentifier>,
    ) -> Self {
        self.links
            .push((link_type, linked_object_identifier.into()));
        self
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

    /// Sets the KMIP protocol version override for this request/exchange.
    #[must_use]
    pub fn with_protocol_version(mut self, version: ProtocolVersion) -> Self {
        let options = self
            .options
            .unwrap_or_default()
            .with_protocol_version(version);
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
        let version = effective_protocol_version(Some(self.client), self.options.as_ref());
        let mut attributes = AttributeSet::new();
        for name in &self.names {
            attributes
                .try_push(name_attribute_item_for_version(name, version)?)
                .map_err(|_| FluentBuildError::InvalidAttributeValue)?;
        }
        for (value, name_type) in &self.alternative_names {
            attributes
                .try_push(alternative_name_item(value, *name_type)?)
                .map_err(|_| FluentBuildError::InvalidAttributeValue)?;
        }
        for (link_type, target) in &self.links {
            attributes
                .try_push(link_item_for_version(*link_type, target.clone(), version)?)
                .map_err(|_| FluentBuildError::InvalidAttributeValue)?;
        }
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

    /// Executes one typed Register exchange using configured [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits)
    /// and returns the unwrapped [`RegisterResponse`](kmipkit_protocol::RegisterResponse).
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] for validation, transport, protocol, `ServerResult`, or `Pending`.
    pub fn execute(self) -> Result<kmipkit_protocol::RegisterResponse, ClientError> {
        let item = self.execute_outcome()?;
        unwrap_typed_outcome(
            item,
            |outcome| match outcome {
                ClientBatchOutcome::Register(resp) => Some(resp),
                _ => None,
            },
            kmipkit_protocol::RegisterResponse::result,
        )
    }

    /// Executes one typed Register exchange using configured [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits)
    /// and returns the raw [`ClientBatchItemResponse`] envelope.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] when local validation or exchange fails.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let request = self
            .build_request()
            .map_err(fluent_client_validation_error)?;
        execute_fluent_outcome(
            self.client,
            ClientRequest::Register(request),
            self.asynchronous_indicator,
            self.options,
        )
    }

    /// Executes one typed Register exchange with explicit [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits) overrides.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] when local validation fails or the existing
    /// client exchange fails.
    pub fn execute_outcome_with_limits(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let request = self
            .build_request()
            .map_err(fluent_client_validation_error)?;
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
    pub(super) fn new(client: Option<&'a mut Client>) -> Self {
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

    /// Selects symmetric algorithm and bit length from a curated [`KeySpec`]; usage policy remains explicit.
    #[must_use]
    pub fn with_key_spec(mut self, spec: KeySpec) -> Self {
        if !spec.is_symmetric() {
            self.attribute_error
                .get_or_insert(FluentBuildError::InvalidAttributeValue);
            return self;
        }
        self.algorithm(spec.algorithm()).length(spec.length())
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

    /// Sets the KMIP protocol version override for this request/exchange.
    #[must_use]
    pub fn with_protocol_version(mut self, version: ProtocolVersion) -> Self {
        let options = self
            .options
            .unwrap_or_default()
            .with_protocol_version(version);
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

    /// Builds a request for an explicit target KMIP [`ProtocolVersion`] without consuming inputs.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError`] when required cryptographic properties or
    /// previously supplied attributes are invalid.
    pub fn build_request_for_version(
        &self,
        version: ProtocolVersion,
    ) -> Result<CreateRequest, FluentBuildError> {
        self.build_request_inner_for_version(version)
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

    /// Executes one typed Create exchange using configured [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits)
    /// and returns the unwrapped [`CreateResponse`](kmipkit_protocol::CreateResponse).
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] when validation, transport, protocol, `ServerResult`, or `Pending` occurs.
    pub fn execute(self) -> Result<kmipkit_protocol::CreateResponse, ClientError> {
        let item = self.execute_outcome()?;
        unwrap_typed_outcome(
            item,
            |outcome| match outcome {
                ClientBatchOutcome::CreateCompleted(resp) => Some(resp),
                _ => None,
            },
            kmipkit_protocol::CreateResponse::result,
        )
    }

    /// Executes one typed Create exchange using configured [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits)
    /// and returns the raw [`ClientBatchItemResponse`] envelope.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] when local request validation fails or the
    /// configured client exchange fails.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let request = self
            .build_request_inner()
            .map_err(fluent_client_validation_error)?;
        let Some(client) = self.client else {
            return Err(fluent_client_validation_error(
                FluentBuildError::MissingClientForExecution,
            ));
        };
        execute_fluent_outcome(
            client,
            ClientRequest::Create(request),
            self.asynchronous_indicator,
            self.options,
        )
    }

    /// Executes one typed Create exchange with explicit [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits) overrides.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] when local request validation fails or the
    /// configured client exchange fails.
    pub fn execute_outcome_with_limits(
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
        let version = effective_protocol_version(self.client.as_deref(), self.options.as_ref());
        self.build_request_inner_for_version(version)
    }

    fn build_request_inner_for_version(
        &self,
        version: ProtocolVersion,
    ) -> Result<CreateRequest, FluentBuildError> {
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
                .try_push(name_attribute_item_for_version(name, version)?)
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
    pub(super) fn new(client: &'a mut Client) -> Self {
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

    /// Sets the optional input key identifier (`impl Into<UniqueIdentifier>`).
    #[must_use]
    pub fn unique_identifier(mut self, identifier: impl Into<UniqueIdentifier>) -> Self {
        self.unique_identifier = Some(identifier.into());
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

    /// Sets the KMIP protocol version override for this request/exchange.
    #[must_use]
    pub fn with_protocol_version(mut self, version: ProtocolVersion) -> Self {
        let options = self
            .options
            .unwrap_or_default()
            .with_protocol_version(version);
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
            let version = effective_protocol_version(Some(self.client), self.options.as_ref());
            request =
                request.with_unique_identifier(normalize_uid_for_version(identifier, version));
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

    /// Executes one typed Create Split Key exchange using configured [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits)
    /// and returns the unwrapped [`CreateSplitKeyResponse`](kmipkit_protocol::CreateSplitKeyResponse).
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] when validation, transport, protocol, `ServerResult`, or `Pending` occurs.
    pub fn execute(self) -> Result<kmipkit_protocol::CreateSplitKeyResponse, ClientError> {
        let item = self.execute_outcome()?;
        unwrap_typed_outcome(
            item,
            |outcome| match outcome {
                ClientBatchOutcome::CreateSplitKeyCompleted(resp) => Some(resp),
                _ => None,
            },
            kmipkit_protocol::CreateSplitKeyResponse::result,
        )
    }

    /// Executes one typed Create Split Key exchange using configured [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits)
    /// and returns the raw [`ClientBatchItemResponse`] envelope.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] for local validation or client exchange failure.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let request = self
            .build_request()
            .map_err(fluent_client_validation_error)?;
        execute_fluent_outcome(
            self.client,
            ClientRequest::CreateSplitKey(request),
            self.asynchronous_indicator,
            self.options,
        )
    }

    /// Executes one typed Create Split Key exchange with explicit [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits) overrides.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] for local validation or client exchange failure.
    pub fn execute_outcome_with_limits(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let request = self
            .build_request()
            .map_err(fluent_client_validation_error)?;
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
    seed: Option<SecretBytes>,
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
    pub(super) fn new(client: Option<&'a mut Client>) -> Self {
        Self {
            client,
            seed: None,
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

    /// Supplies the optional KMIP 3.0 key-generation Seed.
    #[must_use]
    pub fn seed(mut self, seed: SecretBytes) -> Self {
        self.seed = Some(seed);
        self
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

    /// Selects asymmetric or KMIP 3.0 post-quantum algorithm, length, and curve from a [`KeySpec`].
    #[must_use]
    pub fn with_key_spec(mut self, spec: KeySpec) -> Self {
        if !spec.is_asymmetric() {
            self.attribute_error
                .get_or_insert(FluentBuildError::InvalidAttributeValue);
            return self;
        }
        let mut next = self.algorithm(spec.algorithm()).length(spec.length());
        if let Some(curve) = spec.recommended_curve_raw() {
            next = next.recommended_curve(curve);
        }
        next
    }

    /// Adds a `CryptographicDomainParameters` attribute setting `RecommendedCurve`.
    #[must_use]
    pub fn recommended_curve(mut self, curve_raw: u32) -> Self {
        const RECOMMENDED_CURVE_TAG: u32 = 0x0042_0075;
        let built = RawTag::new(RECOMMENDED_CURVE_TAG)
            .and_then(|raw| raw.try_checked())
            .ok()
            .and_then(|tag: Tag| Item::new(tag, Value::enumeration(curve_raw)).ok())
            .and_then(|item| {
                let mut s = Structure::new();
                s.try_push(item).ok()?;
                Attribute::<typed::CryptographicDomainParameters>::new(s).ok()
            });
        if let Some(attr) = built {
            self.common_attribute(attr)
        } else {
            self.attribute_error
                .get_or_insert(FluentBuildError::InvalidAttributeValue);
            self
        }
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

    /// Sets the KMIP protocol version override for this request/exchange.
    #[must_use]
    pub fn with_protocol_version(mut self, version: ProtocolVersion) -> Self {
        let options = self
            .options
            .unwrap_or_default()
            .with_protocol_version(version);
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
        let version = effective_protocol_version(self.client.as_deref(), self.options.as_ref());
        self.build_request_for_version(version)
    }

    /// Builds a typed request for an explicit target KMIP [`ProtocolVersion`].
    ///
    /// # Errors
    ///
    /// Returns a payload-free error for missing fields or invalid attributes.
    pub fn build_request_for_version(
        &self,
        version: ProtocolVersion,
    ) -> Result<CreateKeyPairRequest, FluentBuildError> {
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
            version,
        )?;
        let public = key_pair_attributes(
            self.public_usage_mask,
            self.public_name.as_deref(),
            &self.public_attributes,
            version,
        )?;
        let mut request = CreateKeyPairRequest::new().with_common_attributes(common);
        if !private.is_empty() {
            request = request.with_private_key_attributes(private);
        }
        if !public.is_empty() {
            request = request.with_public_key_attributes(public);
        }
        if let Some(seed) = &self.seed {
            request = request.with_seed(seed.with_bytes(|bytes| SecretBytes::new(bytes.to_vec())));
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

    /// Executes one typed Create Key Pair exchange using configured [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits)
    /// and returns the unwrapped [`CreateKeyPairResponse`](kmipkit_protocol::CreateKeyPairResponse).
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] when validation, transport, protocol, `ServerResult`, or `Pending` occurs.
    pub fn execute(self) -> Result<kmipkit_protocol::CreateKeyPairResponse, ClientError> {
        let item = self.execute_outcome()?;
        unwrap_typed_outcome(
            item,
            |outcome| match outcome {
                ClientBatchOutcome::CreateKeyPairCompleted(resp) => Some(resp),
                _ => None,
            },
            kmipkit_protocol::CreateKeyPairResponse::result,
        )
    }

    /// Executes one typed Create Key Pair exchange using configured [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits)
    /// and returns the raw [`ClientBatchItemResponse`] envelope.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] for local validation or client exchange failure.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let request = self
            .build_request()
            .map_err(fluent_client_validation_error)?;
        let Some(client) = self.client else {
            return Err(fluent_client_validation_error(
                FluentBuildError::MissingClientForExecution,
            ));
        };
        execute_fluent_outcome(
            client,
            ClientRequest::CreateKeyPair(request),
            self.asynchronous_indicator,
            self.options,
        )
    }

    /// Executes one typed Create Key Pair exchange with explicit [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits) overrides.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] for local validation or client exchange failure.
    pub fn execute_outcome_with_limits(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let request = self
            .build_request()
            .map_err(fluent_client_validation_error)?;
        let Some(client) = self.client else {
            return Err(fluent_client_validation_error(
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
    version: ProtocolVersion,
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
            .try_push(name_attribute_item_for_version(name, version)?)
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
                .map_err(|_| FluentBuildError::InvalidTtlvValue)?, // COVERAGE-EXCLUDE: source Item respects clone depth limit.
        )?; // COVERAGE-EXCLUDE: cloned AttributeSet Item was already validated.
    }
    Ok(())
}

/// A fluent builder for KMIP Export requests (§6.1.18, Table 217).
pub struct ExportBuilder<'a> {
    execution: FluentBuilderExecution<'a>,
    unique_identifier: Option<UniqueIdentifier>,
    key_format_type: Option<KeyFormatType>,
    key_wrap_type: Option<KeyWrapType>,
    key_compression_type: Option<KeyCompressionType>,
    key_wrapping_specification: Option<Structure>,
}

impl<'a> ExportBuilder<'a> {
    pub(super) fn new(client: &'a mut Client) -> Self {
        Self {
            execution: FluentBuilderExecution::new(client),
            unique_identifier: None,
            key_format_type: None,
            key_wrap_type: None,
            key_compression_type: None,
            key_wrapping_specification: None,
        }
    }

    /// Selects the object identifier to export (omitted uses the KMIP ID Placeholder).
    #[must_use]
    pub fn unique_identifier(mut self, value: impl Into<UniqueIdentifier>) -> Self {
        let version = self.execution.protocol_version();
        self.unique_identifier = Some(normalize_uid_for_version(value.into(), version));
        self
    }

    /// Sets the requested Key Format Type.
    #[must_use]
    pub const fn key_format_type(mut self, value: KeyFormatType) -> Self {
        self.key_format_type = Some(value);
        self
    }

    /// Sets the raw KMIP Key Format Type code.
    #[must_use]
    pub const fn key_format_type_code(self, value: u32) -> Self {
        self.key_format_type(KeyFormatType::from_raw(value))
    }

    /// Sets the requested Key Wrap Type.
    #[must_use]
    pub const fn key_wrap_type(mut self, value: KeyWrapType) -> Self {
        self.key_wrap_type = Some(value);
        self
    }

    /// Sets the requested Key Compression Type.
    #[must_use]
    pub const fn key_compression_type(mut self, value: KeyCompressionType) -> Self {
        self.key_compression_type = Some(value);
        self
    }

    /// Sets the optional Key Wrapping Specification Structure.
    #[must_use]
    pub fn key_wrapping_specification(mut self, value: Structure) -> Self {
        self.key_wrapping_specification = Some(value);
        self
    }

    /// Sets exchange-local options without changing client policy.
    #[must_use]
    pub fn with_options(mut self, options: RequestOptions) -> Self {
        self.execution = self.execution.with_options(options);
        self
    }

    /// Sets the KMIP protocol version override for this request/exchange.
    #[must_use]
    pub fn with_protocol_version(mut self, version: ProtocolVersion) -> Self {
        self.execution = self.execution.with_protocol_version(version);
        if let Some(uid) = self.unique_identifier.take() {
            self.unique_identifier = Some(normalize_uid_for_version(uid, version));
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
        self.execution = self.execution.with_asynchronous_indicator(value);
        self
    }

    /// Validates the typed Export request without sending it.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError::InvalidRequest`] if typed request validation fails.
    pub fn validate(&self) -> Result<(), FluentBuildError> {
        let mut request = ExportRequest::new();
        if let Some(uid) = &self.unique_identifier {
            request = request.with_unique_identifier(uid.clone());
        }
        if let Some(value) = self.key_format_type {
            request = request.with_key_format_type(value);
        }
        if let Some(value) = self.key_wrap_type {
            request = request.with_key_wrap_type(value);
        }
        if let Some(value) = self.key_compression_type {
            request = request.with_key_compression_type(value);
        }
        validate_fluent_request(&request)
    }

    fn into_request_and_execution(
        self,
    ) -> Result<(ExportRequest, FluentBuilderExecution<'a>), FluentBuildError> {
        let mut request = ExportRequest::new();
        if let Some(uid) = self.unique_identifier {
            request = request.with_unique_identifier(uid);
        }
        if let Some(value) = self.key_format_type {
            request = request.with_key_format_type(value);
        }
        if let Some(value) = self.key_wrap_type {
            request = request.with_key_wrap_type(value);
        }
        if let Some(value) = self.key_compression_type {
            request = request.with_key_compression_type(value);
        }
        if let Some(spec) = self.key_wrapping_specification {
            request = request.with_key_wrapping_specification(spec);
        }
        validate_fluent_request(&request)?;
        Ok((request, self.execution))
    }

    /// Builds the typed [`ExportRequest`] without sending it.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError`] if typed request validation fails.
    pub fn build(self) -> Result<ExportRequest, FluentBuildError> {
        self.into_request_and_execution().map(|(req, _)| req)
    }

    /// Converts Export to a typed [`ClientBatchItem`].
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError`] if typed request validation fails.
    pub fn to_batch_item(self) -> Result<ClientBatchItem, FluentBuildError> {
        self.build()
            .map(|req| ClientBatchItem::new(ClientRequest::Export(req)))
    }

    /// Executes one Export exchange and returns the unwrapped [`ExportResponse`].
    ///
    /// # Errors
    ///
    /// Returns a validation, protocol, transport, `ServerResult`, or `Pending` error.
    pub fn execute(self) -> Result<ExportResponse, ClientError> {
        let (req, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run_typed(
            ClientRequest::Export(req),
            |outcome| match outcome {
                ClientBatchOutcome::Export(resp) => Some(resp),
                _ => None,
            },
            ExportResponse::result,
        )
    }

    /// Executes one Export exchange and returns the raw [`ClientBatchItemResponse`].
    ///
    /// # Errors
    ///
    /// Returns validation, exchange, or decoding failures.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let (req, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run_outcome(ClientRequest::Export(req))
    }

    /// Executes one Export exchange with explicit [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits).
    ///
    /// # Errors
    ///
    /// Returns validation, exchange, or decoding failures.
    pub fn execute_outcome_with_limits(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let (req, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run(ClientRequest::Export(req), limits)
    }
}

/// A fluent builder for KMIP Import requests (§6.1.25, Table 238).
pub struct ImportBuilder<'a> {
    execution: FluentBuilderExecution<'a>,
    unique_identifier: Option<UniqueIdentifier>,
    object_type: Option<ObjectType>,
    replace_existing: Option<bool>,
    key_wrap_type: Option<KeyWrapType>,
    attributes: AttributeSet,
    object: Option<Item>,
    managed_object: Option<ManagedObject>,
}

impl<'a> ImportBuilder<'a> {
    pub(super) fn new(client: &'a mut Client) -> Self {
        Self {
            execution: FluentBuilderExecution::new(client),
            unique_identifier: None,
            object_type: None,
            replace_existing: None,
            key_wrap_type: None,
            attributes: AttributeSet::new(),
            object: None,
            managed_object: None,
        }
    }

    /// Sets the required Unique Identifier for the imported object.
    #[must_use]
    pub fn unique_identifier(mut self, value: impl Into<UniqueIdentifier>) -> Self {
        let version = self.execution.protocol_version();
        self.unique_identifier = Some(normalize_uid_for_version(value.into(), version));
        self
    }

    /// Sets the explicit KMIP Object Type.
    #[must_use]
    pub const fn object_type(mut self, value: ObjectType) -> Self {
        self.object_type = Some(value);
        self
    }

    /// Sets whether an existing object with the same identifier is replaced.
    #[must_use]
    pub const fn replace_existing(mut self, value: bool) -> Self {
        self.replace_existing = Some(value);
        self
    }

    /// Sets the optional Key Wrap Type for wrapped objects.
    #[must_use]
    pub const fn key_wrap_type(mut self, value: KeyWrapType) -> Self {
        self.key_wrap_type = Some(value);
        self
    }

    /// Sets the ordered Attributes to store with the imported object.
    #[must_use]
    pub fn attributes(mut self, attributes: AttributeSet) -> Self {
        self.attributes = attributes;
        self
    }

    /// Sets the raw Any Object TTLV item to import.
    #[must_use]
    pub fn object(mut self, object: Item) -> Self {
        self.object = Some(object);
        self.managed_object = None;
        self
    }

    /// Sets a typed [`ManagedObject`] to import (automatically setting Object Type if unset).
    #[must_use]
    pub fn managed_object(mut self, object: ManagedObject) -> Self {
        if self.object_type.is_none() {
            self.object_type = Some(object.object_type());
        }
        self.managed_object = Some(object);
        self.object = None;
        self
    }

    /// Sets exchange-local options without changing client policy.
    #[must_use]
    pub fn with_options(mut self, options: RequestOptions) -> Self {
        self.execution = self.execution.with_options(options);
        self
    }

    /// Sets the KMIP protocol version override for this request/exchange.
    #[must_use]
    pub fn with_protocol_version(mut self, version: ProtocolVersion) -> Self {
        self.execution = self.execution.with_protocol_version(version);
        if let Some(uid) = self.unique_identifier.take() {
            self.unique_identifier = Some(normalize_uid_for_version(uid, version));
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
        self.execution = self.execution.with_asynchronous_indicator(value);
        self
    }

    /// Validates required builder inputs without consuming `self`.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError`] if identifier, object type, or object is missing.
    pub const fn validate(&self) -> Result<(), FluentBuildError> {
        if self.unique_identifier.is_none() {
            return Err(FluentBuildError::MissingImportUniqueIdentifier);
        }
        if self.object.is_none() && self.managed_object.is_none() {
            return Err(FluentBuildError::MissingImportObject);
        }
        if self.object.is_some() && self.object_type.is_none() {
            return Err(FluentBuildError::MissingImportObjectType);
        }
        Ok(())
    }

    fn into_request_and_execution(
        self,
    ) -> Result<(ImportRequest, FluentBuilderExecution<'a>), FluentBuildError> {
        let uid = self
            .unique_identifier
            .ok_or(FluentBuildError::MissingImportUniqueIdentifier)?;
        let mut request = if let Some(managed) = self.managed_object {
            ImportRequest::from_managed_object(uid, managed, self.attributes)
        } else {
            let object = self.object.ok_or(FluentBuildError::MissingImportObject)?;
            let object_type = self
                .object_type
                .ok_or(FluentBuildError::MissingImportObjectType)?;
            ImportRequest::new(uid, object_type, self.attributes, object)
        };
        if let Some(replace_existing) = self.replace_existing {
            request = request.with_replace_existing(replace_existing);
        }
        if let Some(key_wrap_type) = self.key_wrap_type {
            request = request.with_key_wrap_type(key_wrap_type);
        }
        Ok((request, self.execution))
    }

    /// Builds the typed [`ImportRequest`] without sending it.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError`] if required fields are missing.
    pub fn build(self) -> Result<ImportRequest, FluentBuildError> {
        self.into_request_and_execution().map(|(req, _)| req)
    }

    /// Converts Import to a typed [`ClientBatchItem`].
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError`] if required fields are missing.
    pub fn to_batch_item(self) -> Result<ClientBatchItem, FluentBuildError> {
        self.build()
            .map(|req| ClientBatchItem::new(ClientRequest::Import(req)))
    }

    /// Executes one Import exchange and returns the unwrapped [`ImportResponse`].
    ///
    /// # Errors
    ///
    /// Returns a validation, protocol, transport, `ServerResult`, or `Pending` error.
    pub fn execute(self) -> Result<ImportResponse, ClientError> {
        let (req, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run_typed(
            ClientRequest::Import(req),
            |outcome| match outcome {
                ClientBatchOutcome::Import(resp) => Some(resp),
                _ => None,
            },
            ImportResponse::result,
        )
    }

    /// Executes one Import exchange and returns the raw [`ClientBatchItemResponse`].
    ///
    /// # Errors
    ///
    /// Returns validation, exchange, or decoding failures.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let (req, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run_outcome(ClientRequest::Import(req))
    }

    /// Executes one Import exchange with explicit [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits).
    ///
    /// # Errors
    ///
    /// Returns validation, exchange, or decoding failures.
    pub fn execute_outcome_with_limits(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let (req, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run(ClientRequest::Import(req), limits)
    }
}

/// A fluent builder for KMIP Join Split Key requests (§6.1.27, Table 244).
pub struct JoinSplitKeyBuilder<'a> {
    execution: FluentBuilderExecution<'a>,
    object_type: Option<ObjectType>,
    references: Vec<SplitKeyReference>,
    secret_data_type: Option<u32>,
    attributes: Option<AttributeSet>,
    protection_storage_masks: Option<Structure>,
}

impl<'a> JoinSplitKeyBuilder<'a> {
    pub(super) fn new(client: &'a mut Client) -> Self {
        Self {
            execution: FluentBuilderExecution::new(client),
            object_type: None,
            references: Vec::new(),
            secret_data_type: None,
            attributes: None,
            protection_storage_masks: None,
        }
    }

    /// Sets the target KMIP Object Type of the joined key.
    #[must_use]
    pub const fn object_type(mut self, value: ObjectType) -> Self {
        self.object_type = Some(value);
        self
    }

    /// Appends one [`SplitKeyReference`] preserving threshold metadata.
    #[must_use]
    pub fn reference(mut self, reference: SplitKeyReference) -> Self {
        self.references.push(reference);
        self
    }

    /// Appends multiple [`SplitKeyReference`] values in order.
    #[must_use]
    pub fn references(mut self, references: Vec<SplitKeyReference>) -> Self {
        self.references.extend(references);
        self
    }

    /// Appends a split-key Unique Identifier reference with threshold metadata.
    #[must_use]
    pub fn split_key_identifier(
        mut self,
        unique_identifier: impl Into<UniqueIdentifier>,
        threshold: Option<u32>,
    ) -> Self {
        let version = self.execution.protocol_version();
        let uid = normalize_uid_for_version(unique_identifier.into(), version);
        let reference = match threshold {
            Some(t) => SplitKeyReference::new(uid).with_split_key_threshold(t),
            None => SplitKeyReference::new(uid),
        };
        self.references.push(reference);
        self
    }

    /// Sets the optional Secret Data Type enumeration code.
    #[must_use]
    pub const fn secret_data_type(mut self, value: u32) -> Self {
        self.secret_data_type = Some(value);
        self
    }

    /// Sets optional output object Attributes.
    #[must_use]
    pub fn attributes(mut self, attributes: AttributeSet) -> Self {
        self.attributes = Some(attributes);
        self
    }

    /// Sets optional Protection Storage Masks Structure.
    #[must_use]
    pub fn protection_storage_masks(mut self, masks: Structure) -> Self {
        self.protection_storage_masks = Some(masks);
        self
    }

    fluent_option_field_setters!(
        "Sets exchange-local options without changing client policy.";
        execution.options;
        execution.asynchronous_indicator;
        const
    );

    /// Validates required builder inputs without consuming `self`.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError`] if object type or references are missing.
    pub fn validate(&self) -> Result<(), FluentBuildError> {
        if self.object_type.is_none() {
            return Err(FluentBuildError::MissingJoinSplitKeyObjectType);
        }
        if self.references.is_empty() {
            return Err(FluentBuildError::MissingJoinSplitKeyReferences);
        }
        Ok(())
    }

    fn into_request_and_execution(
        self,
    ) -> Result<(JoinSplitKeyRequest, FluentBuilderExecution<'a>), FluentBuildError> {
        let object_type = self
            .object_type
            .ok_or(FluentBuildError::MissingJoinSplitKeyObjectType)?;
        if self.references.is_empty() {
            return Err(FluentBuildError::MissingJoinSplitKeyReferences);
        }
        let mut request = JoinSplitKeyRequest::new(object_type, self.references);
        if let Some(secret_data_type) = self.secret_data_type {
            request = request.with_secret_data_type(secret_data_type);
        }
        if let Some(attributes) = self.attributes {
            request = request.with_attributes(attributes);
        }
        if let Some(masks) = self.protection_storage_masks {
            request = request.with_protection_storage_masks(masks);
        }
        Ok((request, self.execution))
    }

    /// Builds the typed [`JoinSplitKeyRequest`] without sending it.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError`] if required fields are missing.
    pub fn build(self) -> Result<JoinSplitKeyRequest, FluentBuildError> {
        self.into_request_and_execution().map(|(req, _)| req)
    }

    /// Converts Join Split Key to a typed [`ClientBatchItem`].
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError`] if required fields are missing.
    pub fn to_batch_item(self) -> Result<ClientBatchItem, FluentBuildError> {
        self.build()
            .map(|req| ClientBatchItem::new(ClientRequest::JoinSplitKey(req)))
    }

    /// Executes one Join Split Key exchange and returns [`JoinSplitKeyResponse`].
    ///
    /// # Errors
    ///
    /// Returns a validation, protocol, transport, `ServerResult`, or `Pending` error.
    pub fn execute(self) -> Result<JoinSplitKeyResponse, ClientError> {
        let (req, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run_typed(
            ClientRequest::JoinSplitKey(req),
            |outcome| match outcome {
                ClientBatchOutcome::JoinSplitKeyCompleted(resp) => Some(resp),
                _ => None,
            },
            JoinSplitKeyResponse::result,
        )
    }

    /// Executes one Join Split Key exchange and returns [`ClientBatchItemResponse`].
    ///
    /// # Errors
    ///
    /// Returns validation, exchange, or decoding failures.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let (req, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run_outcome(ClientRequest::JoinSplitKey(req))
    }

    /// Executes one Join Split Key exchange with explicit [`CodecLimits`](kmipkit_ttlv::codec::CodecLimits).
    ///
    /// # Errors
    ///
    /// Returns validation, exchange, or decoding failures.
    pub fn execute_outcome_with_limits(
        self,
        limits: &kmipkit_ttlv::codec::CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let (req, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run(ClientRequest::JoinSplitKey(req), limits)
    }
}
