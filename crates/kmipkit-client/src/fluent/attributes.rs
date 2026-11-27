//! Task-oriented, fluent builders over the existing typed KMIP requests.


use kmipkit_protocol::{
    AddAttributeRequest, AddAttributeResponse, AdjustAttributeRequest, AdjustAttributeResponse,
    AdjustmentType, Attribute, AttributeDefinition, AttributeReference, CurrentAttribute,
    DeleteAttributeRequest, DeleteAttributeResponse, GetAttributeListRequest,
    GetAttributeListResponse, GetAttributesRequest, GetAttributesResponse, ModifyAttributeRequest,
    ModifyAttributeResponse, NewAttribute, ProtocolVersion, SetAttributeRequest,
    SetAttributeResponse,
};
use kmipkit_transport::RequestOptions;
use kmipkit_ttlv::Item;

use crate::{
    Client, ClientBatchItem, ClientBatchItemResponse, ClientBatchOutcome, ClientError,
    ClientRequest,
};

use super::execution::{
    AttributeAdjustmentValue, FluentBuildError, copy_adjustment_value, copy_new_attribute, execute_fluent_outcome, execute_fluent_request, fluent_client_validation_error, make_vendor_attribute, unwrap_typed_outcome, validate_fluent_request, validate_object_attribute_tag,
};

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
    pub(super) fn new(client: Option<&'a mut Client>) -> Self {
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

    fn set_protocol_version(&mut self, version: ProtocolVersion) {
        self.options = Some(self.options.unwrap_or_default().with_protocol_version(version));
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

fn execute_attribute_mutation_outcome<T>(
    client: Option<&mut Client>,
    request: Result<T, FluentBuildError>,
    make_client_request: impl FnOnce(T) -> ClientRequest,
    asynchronous_indicator: Option<u32>,
    options: Option<RequestOptions>,
) -> Result<ClientBatchItemResponse, ClientError> {
    let request = request.map_err(fluent_client_validation_error)?;
    let Some(client) = client else {
        return Err(fluent_client_validation_error(
            FluentBuildError::MissingClientForExecution,
        ));
    };
    execute_fluent_outcome(
        client,
        make_client_request(request),
        asynchronous_indicator,
        options,
    )
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
    pub(super) fn new(client: Option<&'a mut Client>) -> Self {
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

    /// Sets the KMIP protocol version override for this request/exchange.
    #[must_use]
    pub fn with_protocol_version(mut self, version: ProtocolVersion) -> Self {
        let options = self.options.unwrap_or_default().with_protocol_version(version);
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

    /// Executes one typed Delete Attribute exchange using configured [`CodecLimits`]
    /// and returns the unwrapped [`DeleteAttributeResponse`].
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] when validation, transport, protocol, `ServerResult`, or `Pending` occurs.
    pub fn execute(self) -> Result<DeleteAttributeResponse, ClientError> {
        let item = self.execute_outcome()?;
        unwrap_typed_outcome(
            item,
            |outcome| match outcome {
                ClientBatchOutcome::DeleteAttribute(resp) => Some(resp),
                _ => None,
            },
            DeleteAttributeResponse::result,
        )
    }

    /// Executes one typed Delete Attribute exchange using configured [`CodecLimits`]
    /// and returns the raw [`ClientBatchItemResponse`] envelope.
    ///
    /// # Errors
    ///
    /// Returns a local `NotSent` validation error, or the existing protocol or
    /// transport error with its delivery state.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let request = self
            .construct_request()
            .map_err(fluent_client_validation_error)?;
        let Some(client) = self.client else {
            return Err(fluent_client_validation_error(
                FluentBuildError::MissingClientForExecution,
            ));
        };
        execute_fluent_outcome(
            client,
            ClientRequest::DeleteAttribute(request),
            self.asynchronous_indicator,
            self.options,
        )
    }

    /// Executes one typed Delete Attribute exchange with explicit [`CodecLimits`] overrides.
    ///
    /// # Errors
    ///
    /// Returns a local `NotSent` validation error, or the existing protocol or
    /// transport error with its delivery state.
    pub fn execute_outcome_with_limits(
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

    /// Executes one typed Add Attribute exchange using configured [`CodecLimits`]
    /// and returns the unwrapped [`AddAttributeResponse`].
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] when validation, transport, protocol, `ServerResult`, or `Pending` occurs.
    pub fn execute(self) -> Result<AddAttributeResponse, ClientError> {
        let item = self.execute_outcome()?;
        unwrap_typed_outcome(
            item,
            |outcome| match outcome {
                ClientBatchOutcome::AddAttribute(resp) => Some(resp),
                _ => None,
            },
            AddAttributeResponse::result,
        )
    }

    /// Executes one typed Add Attribute exchange using configured [`CodecLimits`]
    /// and returns the raw [`ClientBatchItemResponse`] envelope.
    ///
    /// # Errors
    ///
    /// Returns a local `NotSent` validation error, or the existing protocol or
    /// transport error with its delivery state.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let request = self.construct_request();
        execute_attribute_mutation_outcome(
            self.state.client,
            request,
            ClientRequest::AddAttribute,
            self.state.asynchronous_indicator,
            self.state.options,
        )
    }

    /// Executes one typed Add Attribute exchange with explicit [`CodecLimits`] overrides.
    ///
    /// # Errors
    ///
    /// Returns a local `NotSent` validation error, or the existing protocol or
    /// transport error with its delivery state.
    pub fn execute_outcome_with_limits(
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
    pub(super) fn new(client: Option<&'a mut Client>) -> Self {
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

    /// Sets the KMIP protocol version override for this request/exchange.
    #[must_use]
    pub fn with_protocol_version(mut self, version: ProtocolVersion) -> Self {
        self.state.set_protocol_version(version);
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
        self.state.set_asynchronous_indicator(value);
        self
    }
}

impl<'a> SetAttributeBuilder<'a> {
    pub(super) fn new(client: Option<&'a mut Client>) -> Self {
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

    /// Sets the KMIP protocol version override for this request/exchange.
    #[must_use]
    pub fn with_protocol_version(mut self, version: ProtocolVersion) -> Self {
        self.state.set_protocol_version(version);
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

    /// Executes one typed Set Attribute exchange using configured [`CodecLimits`]
    /// and returns the unwrapped [`SetAttributeResponse`].
    ///
    /// # Errors
    /// Returns [`ClientError`] when validation, transport, protocol, `ServerResult`, or `Pending` occurs.
    pub fn execute(self) -> Result<SetAttributeResponse, ClientError> {
        let item = self.execute_outcome()?;
        unwrap_typed_outcome(
            item,
            |outcome| match outcome {
                ClientBatchOutcome::SetAttribute(resp) => Some(resp),
                _ => None,
            },
            SetAttributeResponse::result,
        )
    }

    /// Executes one typed Set Attribute exchange using configured [`CodecLimits`]
    /// and returns the raw [`ClientBatchItemResponse`] envelope.
    ///
    /// # Errors
    /// Returns a payload-free local validation error or the existing protocol/transport error.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let request = self.construct_request();
        execute_attribute_mutation_outcome(
            self.state.client,
            request,
            ClientRequest::SetAttribute,
            self.state.asynchronous_indicator,
            self.state.options,
        )
    }

    /// Executes one typed Set Attribute exchange with explicit [`CodecLimits`] overrides.
    ///
    /// # Errors
    /// Returns a payload-free local validation error with `NotSent`, or the
    /// existing protocol or transport error with its delivery state.
    pub fn execute_outcome_with_limits(
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
    pub(super) fn new(client: &'a mut Client) -> Self {
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

    /// Sets the KMIP protocol version override for this request/exchange.
    #[must_use]
    pub fn with_protocol_version(mut self, version: ProtocolVersion) -> Self {
        let options = self.options.unwrap_or_default().with_protocol_version(version);
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

    /// Executes one typed Get Attributes exchange using configured [`CodecLimits`]
    /// and returns the unwrapped [`GetAttributesResponse`].
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] when validation, transport, protocol, `ServerResult`, or `Pending` occurs.
    pub fn execute(self) -> Result<GetAttributesResponse, ClientError> {
        let item = self.execute_outcome()?;
        unwrap_typed_outcome(
            item,
            |outcome| match outcome {
                ClientBatchOutcome::GetAttributes(resp) => Some(resp),
                _ => None,
            },
            GetAttributesResponse::result,
        )
    }

    /// Executes one typed Get Attributes exchange using configured [`CodecLimits`]
    /// and returns the raw [`ClientBatchItemResponse`] envelope.
    ///
    /// # Errors
    ///
    /// Returns a local `NotSent` validation error or the existing protocol/transport error.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let request = self
            .construct_request()
            .map_err(fluent_client_validation_error)?;
        execute_fluent_outcome(
            self.client,
            ClientRequest::GetAttributes(request),
            self.asynchronous_indicator,
            self.options,
        )
    }

    /// Executes one typed Get Attributes exchange with explicit [`CodecLimits`] overrides.
    ///
    /// # Errors
    ///
    /// Returns a payload-free local validation error with `NotSent`, or the
    /// existing protocol or transport error with its delivery state.
    pub fn execute_outcome_with_limits(
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
    pub(super) fn new(client: &'a mut Client) -> Self {
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

    /// Sets the KMIP protocol version override for this request/exchange.
    #[must_use]
    pub fn with_protocol_version(mut self, version: ProtocolVersion) -> Self {
        let options = self.options.unwrap_or_default().with_protocol_version(version);
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

    /// Executes one typed Get Attribute List exchange using configured [`CodecLimits`]
    /// and returns the unwrapped [`GetAttributeListResponse`].
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] when validation, transport, protocol, `ServerResult`, or `Pending` occurs.
    pub fn execute(self) -> Result<GetAttributeListResponse, ClientError> {
        let item = self.execute_outcome()?;
        unwrap_typed_outcome(
            item,
            |outcome| match outcome {
                ClientBatchOutcome::GetAttributeList(resp) => Some(resp),
                _ => None,
            },
            GetAttributeListResponse::result,
        )
    }

    /// Executes one typed Get Attribute List exchange using configured [`CodecLimits`]
    /// and returns the raw [`ClientBatchItemResponse`] envelope.
    ///
    /// # Errors
    ///
    /// Returns a local `NotSent` validation error or the existing protocol/transport error.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let request = self
            .construct_request()
            .map_err(fluent_client_validation_error)?;
        execute_fluent_outcome(
            self.client,
            ClientRequest::GetAttributeList(request),
            self.asynchronous_indicator,
            self.options,
        )
    }

    /// Executes one typed Get Attribute List exchange with explicit [`CodecLimits`] overrides.
    ///
    /// # Errors
    ///
    /// Returns a payload-free local validation error with `NotSent`, or the
    /// existing protocol or transport error with its delivery state.
    pub fn execute_outcome_with_limits(
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
    pub(super) fn new(client: &'a mut Client) -> Self {
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

    /// Executes one typed Modify Attribute exchange using configured [`CodecLimits`]
    /// and returns the unwrapped [`ModifyAttributeResponse`].
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] when validation, transport, protocol, `ServerResult`, or `Pending` occurs.
    pub fn execute(self) -> Result<ModifyAttributeResponse, ClientError> {
        let item = self.execute_outcome()?;
        unwrap_typed_outcome(
            item,
            |outcome| match outcome {
                ClientBatchOutcome::ModifyAttribute(resp) => Some(resp),
                _ => None,
            },
            ModifyAttributeResponse::result,
        )
    }

    /// Executes one typed Modify Attribute exchange using configured [`CodecLimits`]
    /// and returns the raw [`ClientBatchItemResponse`] envelope.
    ///
    /// # Errors
    ///
    /// Returns a not-sent validation error or exchange/decoding failure.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
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
        execute_fluent_outcome(
            client,
            ClientRequest::ModifyAttribute(request),
            asynchronous_indicator,
            options,
        )
    }

    /// Executes one typed Modify Attribute exchange with explicit [`CodecLimits`] overrides.
    ///
    /// # Errors
    ///
    /// Returns a not-sent validation error before transport when the request
    /// is invalid, or the existing client error for exchange and decoding failures.
    pub fn execute_outcome_with_limits(
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
    pub(super) fn new(client: &'a mut Client) -> Self {
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

    /// Executes one typed Adjust Attribute exchange using configured [`CodecLimits`]
    /// and returns the unwrapped [`AdjustAttributeResponse`].
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] when validation, transport, protocol, `ServerResult`, or `Pending` occurs.
    pub fn execute(self) -> Result<AdjustAttributeResponse, ClientError> {
        let item = self.execute_outcome()?;
        unwrap_typed_outcome(
            item,
            |outcome| match outcome {
                ClientBatchOutcome::AdjustAttribute(resp) => Some(resp),
                _ => None,
            },
            AdjustAttributeResponse::result,
        )
    }

    /// Executes one typed Adjust Attribute exchange using configured [`CodecLimits`]
    /// and returns the raw [`ClientBatchItemResponse`] envelope.
    ///
    /// # Errors
    ///
    /// Returns a not-sent validation error or exchange/decoding failure.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
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
        execute_fluent_outcome(
            client,
            ClientRequest::AdjustAttribute(request),
            asynchronous_indicator,
            options,
        )
    }

    /// Executes one typed Adjust Attribute exchange with explicit [`CodecLimits`] overrides.
    ///
    /// # Errors
    ///
    /// Returns a not-sent validation error before transport when the request
    /// is invalid, or the existing client error for exchange and decoding failures.
    pub fn execute_outcome_with_limits(
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
