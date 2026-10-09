//! Closed typed request execution for the synchronous KMIP client.

use std::collections::HashSet;
use std::error::Error;
use std::fmt;

use kmipkit_protocol::attribute::{
    ClientAttributeMutation, client_attribute_mutation_is_prohibited,
    client_vendor_attribute_mutation_is_prohibited,
};
use kmipkit_protocol::extension::ExtensionIdentity;
use kmipkit_protocol::{
    AddAttributeRequest, AddAttributeResponse, AdjustAttributeRequest, AdjustAttributeResponse,
    AsynchronousOperationError, AttributeReference, CancelRequest, CancelResponse,
    CancellationResult, CreateKeyPairRequest, CreateKeyPairResponse, CreateRequest, CreateResponse,
    CreateSplitKeyRequest, CreateSplitKeyResponse, DeleteAttributeRequest, DeleteAttributeResponse,
    DiscoverVersionsRequest, DiscoverVersionsResponse, GetAttributeListRequest,
    GetAttributeListResponse, GetAttributesRequest, GetAttributesResponse, KmipOperationResult,
    MessageExtensionView, ModifyAttributeRequest, ModifyAttributeResponse, NewAttribute,
    PollRequest, PollResponse, ProcessRequest, ProcessResponse, ProtocolCauseCategory,
    ProtocolError, ProtocolErrorKind, ProtocolVersion, QueryAsyncRequestsRequest,
    QueryAsyncRequestsResponse, RequestMessage, ResponseBatchItemView, ResponseMessage,
    ResultStatus, SetAttributeRequest, SetAttributeResponse,
};
#[cfg(test)]
use kmipkit_transport::Transport;
use kmipkit_transport::{
    HttpsTransport, RawTlsTransport, RequestDeliveryState, RequestOptions, TransportConfig,
    TransportConfigError,
};
use kmipkit_ttlv::codec::{CodecLimits, DecodeError, decode_with_limits};
use kmipkit_ttlv::{Item, ModelError, RawTag, Structure, StructureView, Tag, Value, ValueView};
#[cfg(test)]
use zeroize::Zeroize;
use zeroize::Zeroizing;

#[cfg(test)]
use crate::execute_test_support::TestTransport;
use crate::extension_registry::{
    self, ClientConfiguration, ClientExtensionRegistry, ClientRequestMessageExtension,
};
use crate::{ClientCauseCategory, ClientError};

#[path = "wire_encoder.rs"]
mod private_wire_writer;

#[cfg(test)]
#[path = "../tests/unit/extension_outbound.rs"]
mod extension_outbound_tests;

#[cfg(test)]
pub(super) use private_wire_writer::ZeroizationObserver;

const MESSAGE: u32 = 0x0042_0078;
const REQUEST_HEADER: u32 = 0x0042_0077;
const MAXIMUM_RESPONSE_SIZE: u32 = 0x0042_0050;
const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
const BATCH_COUNT: u32 = 0x0042_000D;
const BATCH_ERROR_CONTINUATION_OPTION: u32 = 0x0042_000E;
const BATCH_ITEM: u32 = 0x0042_000F;
const MESSAGE_EXTENSION: u32 = 0x0042_0051;
const BATCH_ORDER_OPTION: u32 = 0x0042_0010;
const ASYNCHRONOUS_INDICATOR: u32 = 0x0042_0007;
const ASYNCHRONOUS_CORRELATION_VALUE: u32 = 0x0042_0006;
const ATTESTATION_CAPABLE_INDICATOR: u32 = 0x0042_00D3;
const OPERATION: u32 = 0x0042_005C;
const REQUEST_PAYLOAD: u32 = 0x0042_0079;
const CLIENT_CORRELATION_VALUE: u32 = 0x0042_0105;
const TIME_STAMP: u32 = 0x0042_0092;
const UNIQUE_BATCH_ITEM_ID: u32 = 0x0042_0093;
const DISCOVER_VERSIONS_OPERATION: u32 = 0x0000_001E;
const CREATE_OPERATION: u32 = 0x0000_0001;
const CREATE_KEY_PAIR_OPERATION: u32 = 0x0000_0002;
const CREATE_SPLIT_KEY_OPERATION: u32 = 0x0000_0003;
const ADD_ATTRIBUTE_OPERATION: u32 = 0x0000_000D;
const MODIFY_ATTRIBUTE_OPERATION: u32 = 0x0000_000E;
const DELETE_ATTRIBUTE_OPERATION: u32 = 0x0000_000F;
const ADJUST_ATTRIBUTE_OPERATION: u32 = 0x0000_0030;
const SET_ATTRIBUTE_OPERATION: u32 = 0x0000_0031;
const CANCEL_OPERATION: u32 = 0x0000_0019;
const POLL_OPERATION: u32 = 0x0000_001A;
const QUERY_ASYNCHRONOUS_REQUESTS_OPERATION: u32 = 0x0000_0039;
const PROCESS_OPERATION: u32 = 0x0000_003A;
const ASYNCHRONOUS_MANDATORY: u32 = 1;
const ASYNCHRONOUS_OPTIONAL: u32 = 2;
const ASYNCHRONOUS_PROHIBITED: u32 = 3;
const BATCH_ERROR_CONTINUATION_STOP: u32 = 2;
const ENUMERATION_EXTENSION_MIN: u32 = 0x8000_0000;
const ENUMERATION_EXTENSION_MAX: u32 = 0x8fff_ffff;
const RESULT_STATUS_PENDING: u32 = 2;

/// One request variant admitted by the typed client execution path.
///
/// Only explicitly requested typed operations are accepted. Generic TTLV items
/// and caller-provided wire bytes are not accepted.
#[non_exhaustive]
pub enum ClientRequest {
    /// An explicit client-to-server Discover Versions request.
    DiscoverVersions(DiscoverVersionsRequest),
    /// An explicit client-to-server Create request.
    Create(CreateRequest),
    /// An explicit client-to-server Create Key Pair request.
    CreateKeyPair(CreateKeyPairRequest),
    /// An explicit client-to-server Create Split Key request.
    CreateSplitKey(CreateSplitKeyRequest),
    /// An explicit client-to-server Add Attribute request.
    AddAttribute(AddAttributeRequest),
    /// An explicit client-to-server Adjust Attribute request.
    AdjustAttribute(AdjustAttributeRequest),
    /// An explicit client-to-server Delete Attribute request.
    DeleteAttribute(DeleteAttributeRequest),
    /// An explicit client-to-server Modify Attribute request.
    ModifyAttribute(ModifyAttributeRequest),
    /// An explicit client-to-server Set Attribute request.
    SetAttribute(SetAttributeRequest),
    /// An explicit client-to-server Get Attributes request.
    GetAttributes(GetAttributesRequest),
    /// An explicit client-to-server Get Attribute List request.
    GetAttributeList(GetAttributeListRequest),
}

impl ClientRequest {
    /// Creates the KMIP 2.1 Discover Versions request variant.
    #[must_use]
    pub const fn discover_versions() -> Self {
        Self::DiscoverVersions(DiscoverVersionsRequest::new())
    }

    /// Creates a typed Get Attributes request variant.
    #[must_use]
    pub fn get_attributes(request: GetAttributesRequest) -> Self {
        Self::GetAttributes(request)
    }

    /// Creates a typed Get Attribute List request variant.
    #[must_use]
    pub fn get_attribute_list(request: GetAttributeListRequest) -> Self {
        Self::GetAttributeList(request)
    }

    /// Creates a typed Add Attribute request variant.
    #[must_use]
    pub fn add_attribute(request: AddAttributeRequest) -> Self {
        Self::AddAttribute(request)
    }

    /// Creates a typed Adjust Attribute request variant.
    #[must_use]
    pub fn adjust_attribute(request: AdjustAttributeRequest) -> Self {
        Self::AdjustAttribute(request)
    }

    /// Creates a typed Delete Attribute request variant.
    #[must_use]
    pub fn delete_attribute(request: DeleteAttributeRequest) -> Self {
        Self::DeleteAttribute(request)
    }

    /// Creates a typed Modify Attribute request variant.
    #[must_use]
    pub fn modify_attribute(request: ModifyAttributeRequest) -> Self {
        Self::ModifyAttribute(request)
    }

    /// Creates a typed Set Attribute request variant.
    #[must_use]
    pub fn set_attribute(request: SetAttributeRequest) -> Self {
        Self::SetAttribute(request)
    }

    const fn operation(&self) -> u32 {
        match self {
            Self::DiscoverVersions(_) => DISCOVER_VERSIONS_OPERATION,
            Self::Create(_) => CREATE_OPERATION,
            Self::CreateKeyPair(_) => CREATE_KEY_PAIR_OPERATION,
            Self::CreateSplitKey(_) => CREATE_SPLIT_KEY_OPERATION,
            Self::AddAttribute(_) => ADD_ATTRIBUTE_OPERATION,
            Self::AdjustAttribute(_) => ADJUST_ATTRIBUTE_OPERATION,
            Self::DeleteAttribute(_) => DELETE_ATTRIBUTE_OPERATION,
            Self::ModifyAttribute(_) => MODIFY_ATTRIBUTE_OPERATION,
            Self::SetAttribute(_) => SET_ATTRIBUTE_OPERATION,
            Self::GetAttributes(_) => GET_ATTRIBUTES_OPERATION,
            Self::GetAttributeList(_) => GET_ATTRIBUTE_LIST_OPERATION,
        }
    }

    fn payload(self) -> Result<Structure, ProtocolError> {
        match self {
            Self::DiscoverVersions(request) => request.to_ttlv_payload(),
            Self::Create(request) => request.into_ttlv_payload(),
            Self::CreateKeyPair(request) => request.into_ttlv_payload(),
            Self::CreateSplitKey(request) => request.into_ttlv_payload(),
            Self::AddAttribute(request) => request.to_ttlv_payload(),
            Self::AdjustAttribute(request) => request.to_ttlv_payload(),
            Self::DeleteAttribute(request) => request.to_ttlv_payload(),
            Self::ModifyAttribute(request) => request.to_ttlv_payload(),
            Self::SetAttribute(request) => request.to_ttlv_payload(),
            Self::GetAttributes(request) => request.to_ttlv_payload(),
            Self::GetAttributeList(request) => request.to_ttlv_payload(),
        }
    }
}

impl fmt::Debug for ClientRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DiscoverVersions(_) => formatter.write_str("DiscoverVersions"),
            Self::Create(_) => formatter.write_str("Create([REDACTED])"),
            Self::CreateKeyPair(_) => formatter.write_str("CreateKeyPair([REDACTED])"),
            Self::CreateSplitKey(_) => formatter.write_str("CreateSplitKey([REDACTED])"),
            Self::AddAttribute(_) => formatter.write_str("AddAttribute([REDACTED])"),
            Self::AdjustAttribute(_) => formatter.write_str("AdjustAttribute([REDACTED])"),
            Self::DeleteAttribute(_) => formatter.write_str("DeleteAttribute([REDACTED])"),
            Self::ModifyAttribute(_) => formatter.write_str("ModifyAttribute([REDACTED])"),
            Self::SetAttribute(_) => formatter.write_str("SetAttribute([REDACTED])"),
            Self::GetAttributes(request) => formatter
                .debug_tuple("GetAttributes")
                .field(request)
                .finish(),
            Self::GetAttributeList(request) => formatter
                .debug_tuple("GetAttributeList")
                .field(request)
                .finish(),
        }
    }
}

const GET_ATTRIBUTES_OPERATION: u32 = 0x0000_000B;
const GET_ATTRIBUTE_LIST_OPERATION: u32 = 0x0000_000C;

/// One closed typed request and its optional Unique Batch Item ID.
pub struct ClientBatchItem {
    request: ClientRequest,
    unique_batch_item_id: Option<Vec<u8>>,
    message_extensions: Vec<ClientRequestMessageExtension>,
}

impl ClientBatchItem {
    /// Creates a batch item that issues the KMIP 2.1 Discover Versions operation.
    #[must_use]
    pub const fn discover_versions() -> Self {
        Self::new(ClientRequest::discover_versions())
    }

    /// Creates a batch item from an admitted typed request.
    #[must_use]
    pub const fn new(request: ClientRequest) -> Self {
        Self {
            request,
            unique_batch_item_id: None,
            message_extensions: Vec::new(),
        }
    }

    /// Sets this item's Unique Batch Item ID.
    #[must_use]
    pub fn with_unique_batch_item_id(mut self, id: Vec<u8>) -> Self {
        self.unique_batch_item_id = Some(id);
        self
    }

    /// Appends one schema-validated Message Extension with explicit criticality.
    ///
    /// Repeated calls preserve the order in which extensions are attached.
    #[must_use]
    pub fn with_extension(mut self, extension: ClientRequestMessageExtension) -> Self {
        self.message_extensions.push(extension);
        self
    }

    /// Returns the number of attached Message Extensions.
    #[must_use]
    pub fn extension_count(&self) -> usize {
        self.message_extensions.len()
    }

    /// Returns a copy of the registered identity at `index`, if present.
    #[must_use]
    pub fn extension_identity_at(&self, index: usize) -> Option<ExtensionIdentity> {
        self.message_extensions
            .get(index)
            .map(ClientRequestMessageExtension::identity)
    }

    /// Returns the explicit Criticality Indicator at `index`, if present.
    #[must_use]
    pub fn extension_criticality_indicator_at(&self, index: usize) -> Option<bool> {
        self.message_extensions
            .get(index)
            .map(ClientRequestMessageExtension::criticality_indicator)
    }

    /// Returns the typed operation request.
    #[must_use]
    pub const fn request(&self) -> &ClientRequest {
        &self.request
    }

    /// Lends the optional Unique Batch Item ID.
    #[must_use]
    pub fn unique_batch_item_id(&self) -> Option<&[u8]> {
        self.unique_batch_item_id.as_deref()
    }
}

impl fmt::Debug for ClientBatchItem {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ClientBatchItem")
            .field("request", &self.request)
            .field("unique_batch_item_id", &self.unique_batch_item_id.is_some())
            .field("message_extension_count", &self.message_extensions.len())
            .finish()
    }
}

/// A typed ordered request batch with common client-selected message options.
pub struct ClientBatch {
    items: Vec<ClientBatchItem>,
    request_time_stamp: Option<i64>,
    batch_order_option: Option<bool>,
    batch_error_continuation_values: Vec<u32>,
    asynchronous_indicator: Option<u32>,
    client_correlation_value: Option<String>,
}

impl ClientBatch {
    /// Creates a batch with one typed request item.
    #[must_use]
    pub fn new(item: ClientBatchItem) -> Self {
        Self {
            items: vec![item],
            request_time_stamp: None,
            batch_order_option: None,
            batch_error_continuation_values: Vec::new(),
            asynchronous_indicator: None,
            client_correlation_value: None,
        }
    }

    /// Creates a batch from an ordered iterator of typed request items.
    #[must_use]
    pub fn from_items(items: impl IntoIterator<Item = ClientBatchItem>) -> Self {
        Self {
            items: items.into_iter().collect(),
            request_time_stamp: None,
            batch_order_option: None,
            batch_error_continuation_values: Vec::new(),
            asynchronous_indicator: None,
            client_correlation_value: None,
        }
    }

    /// Appends a typed request item in execution order.
    pub fn push(&mut self, item: ClientBatchItem) {
        self.items.push(item);
    }

    /// Sets the optional caller-supplied Request Header Time Stamp.
    ///
    /// The exact Date-Time value is emitted when present. No clock or
    /// countdown-derived value is generated by this API.
    #[must_use]
    pub const fn with_request_time_stamp(mut self, time_stamp: i64) -> Self {
        self.request_time_stamp = Some(time_stamp);
        self
    }

    /// Sets the optional Batch Order Option.
    #[must_use]
    pub const fn with_batch_order_option(mut self, value: bool) -> Self {
        self.batch_order_option = Some(value);
        self
    }

    /// Adds a Batch Error Continuation Option value.
    ///
    /// Calling this method more than once is rejected by [`Client::execute`]
    /// as a repeated singleton header field.
    #[must_use]
    pub fn with_batch_error_continuation_option(mut self, value: u32) -> Self {
        self.batch_error_continuation_values.push(value);
        self
    }

    /// Sets the optional Asynchronous Indicator raw Enumeration value.
    #[must_use]
    pub const fn with_asynchronous_indicator(mut self, value: u32) -> Self {
        self.asynchronous_indicator = Some(value);
        self
    }

    /// Sets the optional Client Correlation Value.
    ///
    /// It remains independent of every Unique Batch Item ID and is never used
    /// to associate response items.
    #[must_use]
    pub fn with_client_correlation_value(mut self, value: String) -> Self {
        self.client_correlation_value = Some(value);
        self
    }

    /// Returns the typed batch items in caller order.
    #[must_use]
    pub fn items(&self) -> &[ClientBatchItem] {
        &self.items
    }

    /// Returns the optional Request Header Time Stamp exactly as supplied.
    #[must_use]
    pub const fn request_time_stamp(&self) -> Option<i64> {
        self.request_time_stamp
    }
}

impl fmt::Debug for ClientBatch {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ClientBatch")
            .field("item_count", &self.items.len())
            .field("request_time_stamp", &self.request_time_stamp.is_some())
            .field("batch_order_option", &self.batch_order_option)
            .field(
                "batch_error_continuation_option_count",
                &self.batch_error_continuation_values.len(),
            )
            .field("asynchronous_indicator", &self.asynchronous_indicator)
            .field(
                "client_correlation_value",
                &self.client_correlation_value.is_some(),
            )
            .finish()
    }
}

impl fmt::Display for ClientBatch {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "ClientBatch ({} items)", self.items.len())
    }
}

/// An opaque preserved Message Extension accepted by the client's recognition
/// and criticality policy.
pub struct ClientMessageExtension {
    structure: Structure,
}

impl ClientMessageExtension {
    /// Lends the preserved extension Structure for explicit inspection.
    pub fn with_ttlv<R>(&self, callback: impl for<'a> FnOnce(StructureView<'a>) -> R) -> R {
        callback(self.structure.view())
    }
}

impl fmt::Debug for ClientMessageExtension {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ClientMessageExtension([REDACTED])")
    }
}

enum PendingResponse {
    DiscoverVersions(DiscoverVersionsResponse),
    Create(CreateResponse),
    CreateKeyPair(CreateKeyPairResponse),
    CreateSplitKey(CreateSplitKeyResponse),
    AddAttribute(AddAttributeResponse),
    AdjustAttribute(AdjustAttributeResponse),
    DeleteAttribute(DeleteAttributeResponse),
    ModifyAttribute(ModifyAttributeResponse),
    SetAttribute(SetAttributeResponse),
    GetAttributes(GetAttributesResponse),
    GetAttributeList(GetAttributeListResponse),
}

impl PendingResponse {
    const fn view(&self) -> ClientResponseView<'_> {
        let response = match self {
            Self::DiscoverVersions(response) => ClientResponseRef::DiscoverVersions(response),
            Self::Create(response) => ClientResponseRef::Create(response),
            Self::CreateKeyPair(response) => ClientResponseRef::CreateKeyPair(response),
            Self::CreateSplitKey(response) => ClientResponseRef::CreateSplitKey(response),
            Self::AddAttribute(response) => ClientResponseRef::AddAttribute(response),
            Self::AdjustAttribute(response) => ClientResponseRef::AdjustAttribute(response),
            Self::DeleteAttribute(response) => ClientResponseRef::DeleteAttribute(response),
            Self::ModifyAttribute(response) => ClientResponseRef::ModifyAttribute(response),
            Self::SetAttribute(response) => ClientResponseRef::SetAttribute(response),
            Self::GetAttributes(response) => ClientResponseRef::GetAttributes(response),
            Self::GetAttributeList(response) => ClientResponseRef::GetAttributeList(response),
        };
        ClientResponseView { response }
    }
}

fn read_response_error<E: Error + 'static>(error: E) -> ProtocolError {
    ProtocolError::new(
        ProtocolErrorKind::InvalidValue,
        ProtocolCauseCategory::InvalidValue,
        error,
    )
}

fn read_operation_outcome<T, E>(
    operation: ClientOperation,
    item: ResponseBatchItemView<'_>,
    convert: impl FnOnce(ResponseBatchItemView<'_>) -> Result<T, E>,
    result_of: impl FnOnce(&T) -> &KmipOperationResult,
    into_pending_response: impl FnOnce(T) -> PendingResponse,
    into_completed_outcome: impl FnOnce(T) -> ClientBatchOutcome,
) -> Result<ClientBatchOutcome, ProtocolError>
where
    E: Error + 'static,
{
    let response = convert(item).map_err(read_response_error)?;
    let result = result_of(&response).clone();
    if result.status().raw() == RESULT_STATUS_PENDING {
        pending_outcome(operation, result, into_pending_response(response), item)
            .map(ClientBatchOutcome::Pending)
    } else {
        Ok(into_completed_outcome(response))
    }
}

impl fmt::Debug for PendingResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("PendingResponse")
            .field(&self.view())
            .finish()
    }
}

/// A Pending result for one typed operation and its opaque correlation capability.
///
/// The correlation bytes are available only through the borrowed accessor and
/// remain in zeroizing-owned storage for this value's lifetime.
///
/// The owner is not directly accessible from downstream crates:
///
/// ```compile_fail
/// use kmipkit_client::PendingOutcome;
///
/// fn copy_owner(pending: &PendingOutcome) -> &[u8] {
///     pending.asynchronous_correlation_value
/// }
/// ```
pub struct PendingOutcome {
    operation: ClientOperation,
    result: KmipOperationResult,
    response: PendingResponse,
    asynchronous_correlation_value: Zeroizing<Vec<u8>>,
}

impl PendingOutcome {
    /// Returns the operation that reported Pending.
    #[must_use]
    pub const fn operation(&self) -> ClientOperation {
        self.operation
    }

    /// Returns the exact KMIP result reported by the server.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }

    /// Returns a borrowed view of the original operation's typed response.
    #[must_use]
    pub const fn response(&self) -> ClientResponseView<'_> {
        self.response.view()
    }

    /// Lends the exact opaque Asynchronous Correlation Value.
    #[must_use]
    pub fn asynchronous_correlation_value(&self) -> &[u8] {
        self.asynchronous_correlation_value.as_slice()
    }
}

impl fmt::Debug for PendingOutcome {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PendingOutcome")
            .field("operation", &self.operation)
            .field("result", &self.result)
            .field("asynchronous_correlation_value", &"[REDACTED]")
            // The typed response may contain caller-sensitive attribute data.
            .finish_non_exhaustive()
    }
}

/// Identifies a client-initiated asynchronous operation handled by one
/// `Client::execute_*` call.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClientOperation {
    /// Discover Versions.
    DiscoverVersions,
    /// Create.
    Create,
    /// Create Key Pair.
    CreateKeyPair,
    /// Create Split Key.
    CreateSplitKey,
    /// Add Attribute.
    AddAttribute,
    /// Adjust Attribute.
    AdjustAttribute,
    /// Delete Attribute.
    DeleteAttribute,
    /// Modify Attribute.
    ModifyAttribute,
    /// Set Attribute.
    SetAttribute,
    /// Get Attributes.
    GetAttributes,
    /// Get Attribute List.
    GetAttributeList,
    /// Poll one previously Pending operation.
    Poll,
    /// Cancel one previously Pending operation.
    Cancel,
    /// Request processing-mode change for one Pending operation.
    Process,
    /// Query outstanding asynchronous requests with a generic response.
    QueryAsyncRequests,
}

/// A completed or Pending asynchronous operation response owned by the client.
///
/// Correlation values stay inside the zeroizing generic response tree and are
/// never copied into an ordinary client-owned buffer. Accepted Message
/// Extensions are available as opaque generic values through [`Self::extensions`].
pub struct ClientOperationOutcome {
    operation: ClientOperation,
    result: kmipkit_protocol::KmipOperationResult,
    response: ResponseMessage,
    cancellation_result: Option<CancellationResult>,
    extensions: Vec<ClientMessageExtension>,
}

impl ClientOperationOutcome {
    /// Returns the operation that produced this outcome.
    #[must_use]
    pub const fn operation(&self) -> ClientOperation {
        self.operation
    }

    /// Returns the exact operation result, including unknown status values.
    #[must_use]
    pub const fn result(&self) -> &kmipkit_protocol::KmipOperationResult {
        &self.result
    }

    /// Returns the accepted Message Extensions preserved from this response.
    ///
    /// Recognized critical extensions and non-critical extensions are
    /// preserved. An unrecognized critical extension causes execution to fail
    /// before an outcome is returned.
    #[must_use]
    pub fn extensions(&self) -> &[ClientMessageExtension] {
        &self.extensions
    }

    /// Returns whether the server reported Operation Pending.
    #[must_use]
    pub const fn is_pending(&self) -> bool {
        self.result.status().raw() == RESULT_STATUS_PENDING
    }

    /// Lends a top-level Asynchronous Correlation Value to a callback.
    pub fn with_asynchronous_correlation_value<R>(
        &self,
        callback: impl for<'a> FnOnce(&'a [u8]) -> R,
    ) -> Option<R> {
        self.response
            .batch_items()
            .next()?
            .with_asynchronous_correlation_value(callback)
    }

    /// Lends the Cancel response's echoed Asynchronous Correlation Value to a
    /// callback when the successful response contains it.
    pub fn with_cancel_echo<R>(&self, callback: impl for<'a> FnOnce(&'a [u8]) -> R) -> Option<R> {
        if self.operation != ClientOperation::Cancel || self.cancellation_result.is_none() {
            return None;
        }
        self.response
            .batch_items()
            .next()?
            .with_response_payload(|payload| {
                payload
                    .children()
                    .iter()
                    .find(|field| field.tag().raw() == ASYNCHRONOUS_CORRELATION_VALUE)
                    .and_then(|field| {
                        field.with_value(|value| match value {
                            ValueView::ByteString(bytes) => Some(callback(bytes)),
                            _ => None,
                        })
                    })
            })
            .flatten()
    }

    /// Returns the known or preserved raw Cancellation Result on Cancel success.
    #[must_use]
    pub const fn cancellation_result(&self) -> Option<CancellationResult> {
        self.cancellation_result
    }

    /// Lends the generic Response Payload to a callback.
    pub fn with_response_payload<R>(
        &self,
        callback: impl for<'a> FnOnce(StructureView<'a>) -> R,
    ) -> Option<R> {
        self.response
            .batch_items()
            .next()?
            .with_response_payload(callback)
    }
}

impl fmt::Debug for ClientOperationOutcome {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ClientOperationOutcome")
            .field("operation", &self.operation)
            .field("result", &self.result)
            .field("cancellation_result", &self.cancellation_result)
            .field(
                "has_response_payload",
                &self.with_response_payload(|_| ()).is_some(),
            )
            .field("correlation", &"[REDACTED]")
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Copy)]
enum ClientResponseRef<'a> {
    DiscoverVersions(&'a DiscoverVersionsResponse),
    Create(&'a CreateResponse),
    CreateKeyPair(&'a CreateKeyPairResponse),
    CreateSplitKey(&'a CreateSplitKeyResponse),
    AddAttribute(&'a AddAttributeResponse),
    AdjustAttribute(&'a AdjustAttributeResponse),
    DeleteAttribute(&'a DeleteAttributeResponse),
    ModifyAttribute(&'a ModifyAttributeResponse),
    SetAttribute(&'a SetAttributeResponse),
    GetAttributes(&'a GetAttributesResponse),
    GetAttributeList(&'a GetAttributeListResponse),
}

/// A borrowed view of one typed response in a [`ClientBatchOutcome`].
///
/// The operation result is available uniformly. Operation-specific response
/// data remains accessible through the corresponding typed accessor.
#[derive(Clone, Copy)]
pub struct ClientResponseView<'a> {
    response: ClientResponseRef<'a>,
}

impl ClientResponseView<'_> {
    /// Returns the complete KMIP status, reason, and optional Result Message.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        match self.response {
            ClientResponseRef::DiscoverVersions(response) => response.result(),
            ClientResponseRef::Create(response) => response.result(),
            ClientResponseRef::CreateKeyPair(response) => response.result(),
            ClientResponseRef::CreateSplitKey(response) => response.result(),
            ClientResponseRef::AddAttribute(response) => response.result(),
            ClientResponseRef::AdjustAttribute(response) => response.result(),
            ClientResponseRef::DeleteAttribute(response) => response.result(),
            ClientResponseRef::ModifyAttribute(response) => response.result(),
            ClientResponseRef::SetAttribute(response) => response.result(),
            ClientResponseRef::GetAttributes(response) => response.result(),
            ClientResponseRef::GetAttributeList(response) => response.result(),
        }
    }

    /// Returns the server-advertised versions for Discover Versions.
    ///
    /// Other operations return `None`.
    #[must_use]
    pub fn supported_versions(&self) -> Option<&[ProtocolVersion]> {
        match self.response {
            ClientResponseRef::DiscoverVersions(response) => response.supported_versions(),
            ClientResponseRef::Create(_)
            | ClientResponseRef::CreateKeyPair(_)
            | ClientResponseRef::CreateSplitKey(_)
            | ClientResponseRef::AddAttribute(_)
            | ClientResponseRef::AdjustAttribute(_)
            | ClientResponseRef::DeleteAttribute(_)
            | ClientResponseRef::ModifyAttribute(_)
            | ClientResponseRef::SetAttribute(_)
            | ClientResponseRef::GetAttributes(_)
            | ClientResponseRef::GetAttributeList(_) => None,
        }
    }

    /// Returns the Discover Versions response when this view represents it.
    #[must_use]
    pub const fn discover_versions(&self) -> Option<&DiscoverVersionsResponse> {
        match self.response {
            ClientResponseRef::DiscoverVersions(response) => Some(response),
            ClientResponseRef::Create(_)
            | ClientResponseRef::CreateKeyPair(_)
            | ClientResponseRef::CreateSplitKey(_)
            | ClientResponseRef::AddAttribute(_)
            | ClientResponseRef::AdjustAttribute(_)
            | ClientResponseRef::DeleteAttribute(_)
            | ClientResponseRef::ModifyAttribute(_)
            | ClientResponseRef::SetAttribute(_)
            | ClientResponseRef::GetAttributes(_)
            | ClientResponseRef::GetAttributeList(_) => None,
        }
    }

    /// Returns the Create response when this view represents it.
    #[must_use]
    pub const fn create(&self) -> Option<&CreateResponse> {
        match self.response {
            ClientResponseRef::Create(response) => Some(response),
            _ => None,
        }
    }

    /// Returns the Create Key Pair response when this view represents it.
    #[must_use]
    pub const fn create_key_pair(&self) -> Option<&CreateKeyPairResponse> {
        match self.response {
            ClientResponseRef::CreateKeyPair(response) => Some(response),
            _ => None,
        }
    }

    /// Returns the Create Split Key response when this view represents it.
    #[must_use]
    pub const fn create_split_key(&self) -> Option<&CreateSplitKeyResponse> {
        match self.response {
            ClientResponseRef::CreateSplitKey(response) => Some(response),
            _ => None,
        }
    }

    /// Returns the Add Attribute response when this view represents it.
    #[must_use]
    pub const fn add_attribute(&self) -> Option<&AddAttributeResponse> {
        match self.response {
            ClientResponseRef::AddAttribute(response) => Some(response),
            _ => None,
        }
    }

    /// Returns the Adjust Attribute response when this view represents it.
    #[must_use]
    pub const fn adjust_attribute(&self) -> Option<&AdjustAttributeResponse> {
        match self.response {
            ClientResponseRef::AdjustAttribute(response) => Some(response),
            _ => None,
        }
    }

    /// Returns the Delete Attribute response when this view represents it.
    #[must_use]
    pub const fn delete_attribute(&self) -> Option<&DeleteAttributeResponse> {
        match self.response {
            ClientResponseRef::DeleteAttribute(response) => Some(response),
            _ => None,
        }
    }

    /// Returns the Modify Attribute response when this view represents it.
    #[must_use]
    pub const fn modify_attribute(&self) -> Option<&ModifyAttributeResponse> {
        match self.response {
            ClientResponseRef::ModifyAttribute(response) => Some(response),
            _ => None,
        }
    }

    /// Returns the Set Attribute response when this view represents it.
    #[must_use]
    pub const fn set_attribute(&self) -> Option<&SetAttributeResponse> {
        match self.response {
            ClientResponseRef::SetAttribute(response) => Some(response),
            _ => None,
        }
    }

    /// Returns the Get Attributes response when this view represents it.
    #[must_use]
    pub const fn get_attributes(&self) -> Option<&GetAttributesResponse> {
        match self.response {
            ClientResponseRef::GetAttributes(response) => Some(response),
            _ => None,
        }
    }

    /// Returns the Get Attribute List response when this view represents it.
    #[must_use]
    pub const fn get_attribute_list(&self) -> Option<&GetAttributeListResponse> {
        match self.response {
            ClientResponseRef::GetAttributeList(response) => Some(response),
            _ => None,
        }
    }
}

impl fmt::Debug for ClientResponseView<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.response {
            ClientResponseRef::DiscoverVersions(response) => formatter
                .debug_tuple("DiscoverVersions")
                .field(response)
                .finish(),
            ClientResponseRef::Create(response) => {
                formatter.debug_tuple("Create").field(response).finish()
            }
            ClientResponseRef::CreateKeyPair(response) => formatter
                .debug_tuple("CreateKeyPair")
                .field(response)
                .finish(),
            ClientResponseRef::CreateSplitKey(response) => formatter
                .debug_tuple("CreateSplitKey")
                .field(response)
                .finish(),
            ClientResponseRef::AddAttribute(response) => formatter
                .debug_tuple("AddAttribute")
                .field(response)
                .finish(),
            ClientResponseRef::AdjustAttribute(response) => formatter
                .debug_tuple("AdjustAttribute")
                .field(response)
                .finish(),
            ClientResponseRef::DeleteAttribute(response) => formatter
                .debug_tuple("DeleteAttribute")
                .field(response)
                .finish(),
            ClientResponseRef::ModifyAttribute(response) => formatter
                .debug_tuple("ModifyAttribute")
                .field(response)
                .finish(),
            ClientResponseRef::SetAttribute(response) => formatter
                .debug_tuple("SetAttribute")
                .field(response)
                .finish(),
            ClientResponseRef::GetAttributes(response) => formatter
                .debug_tuple("GetAttributes")
                .field(response)
                .finish(),
            ClientResponseRef::GetAttributeList(response) => formatter
                .debug_tuple("GetAttributeList")
                .field(response)
                .finish(),
        }
    }
}

/// A typed operation response or an explicitly resumable Pending result.
#[non_exhaustive]
pub enum ClientBatchOutcome {
    /// The server returned a non-Pending KMIP result.
    Completed(DiscoverVersionsResponse),
    /// The server returned a non-Pending Create result.
    CreateCompleted(CreateResponse),
    /// The server returned a non-Pending Create Key Pair result.
    CreateKeyPairCompleted(CreateKeyPairResponse),
    /// The server returned a non-Pending Create Split Key result.
    CreateSplitKeyCompleted(CreateSplitKeyResponse),
    /// The server returned an Add Attribute result.
    AddAttribute(AddAttributeResponse),
    /// The server returned an Adjust Attribute result.
    AdjustAttribute(AdjustAttributeResponse),
    /// The server returned a Delete Attribute result.
    DeleteAttribute(DeleteAttributeResponse),
    /// The server returned a Modify Attribute result.
    ModifyAttribute(ModifyAttributeResponse),
    /// The server returned a Set Attribute result.
    SetAttribute(SetAttributeResponse),
    /// The server returned Pending with its required capability-like value.
    Pending(PendingOutcome),
    /// The server returned a Get Attributes result.
    GetAttributes(GetAttributesResponse),
    /// The server returned a Get Attribute List result.
    GetAttributeList(GetAttributeListResponse),
}

impl ClientBatchOutcome {
    /// Lends the Pending Asynchronous Correlation Value when present.
    #[must_use]
    pub fn asynchronous_correlation_value(&self) -> Option<&[u8]> {
        match self {
            Self::Completed(_)
            | Self::CreateCompleted(_)
            | Self::CreateKeyPairCompleted(_)
            | Self::CreateSplitKeyCompleted(_)
            | Self::AddAttribute(_)
            | Self::AdjustAttribute(_)
            | Self::DeleteAttribute(_)
            | Self::ModifyAttribute(_)
            | Self::SetAttribute(_)
            | Self::GetAttributes(_)
            | Self::GetAttributeList(_) => None,
            Self::Pending(pending) => Some(pending.asynchronous_correlation_value()),
        }
    }

    /// Returns the exact KMIP result for every completed or Pending operation.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        match self {
            Self::Completed(response) => response.result(),
            Self::CreateCompleted(response) => response.result(),
            Self::CreateKeyPairCompleted(response) => response.result(),
            Self::CreateSplitKeyCompleted(response) => response.result(),
            Self::AddAttribute(response) => response.result(),
            Self::AdjustAttribute(response) => response.result(),
            Self::DeleteAttribute(response) => response.result(),
            Self::ModifyAttribute(response) => response.result(),
            Self::SetAttribute(response) => response.result(),
            Self::GetAttributes(response) => response.result(),
            Self::GetAttributeList(response) => response.result(),
            Self::Pending(pending) => pending.result(),
        }
    }

    /// Returns the operation that produced this outcome.
    #[must_use]
    pub const fn operation(&self) -> ClientOperation {
        match self {
            Self::Completed(_) => ClientOperation::DiscoverVersions,
            Self::CreateCompleted(_) => ClientOperation::Create,
            Self::CreateKeyPairCompleted(_) => ClientOperation::CreateKeyPair,
            Self::CreateSplitKeyCompleted(_) => ClientOperation::CreateSplitKey,
            Self::AddAttribute(_) => ClientOperation::AddAttribute,
            Self::AdjustAttribute(_) => ClientOperation::AdjustAttribute,
            Self::DeleteAttribute(_) => ClientOperation::DeleteAttribute,
            Self::ModifyAttribute(_) => ClientOperation::ModifyAttribute,
            Self::SetAttribute(_) => ClientOperation::SetAttribute,
            Self::GetAttributes(_) => ClientOperation::GetAttributes,
            Self::GetAttributeList(_) => ClientOperation::GetAttributeList,
            Self::Pending(pending) => pending.operation(),
        }
    }

    /// Returns the typed Discover Versions response, when this is one.
    #[must_use]
    pub const fn discover_versions_response(&self) -> Option<&DiscoverVersionsResponse> {
        match self {
            Self::Completed(response) => Some(response),
            _ => None,
        }
    }

    /// Returns the typed Create response, when this is one.
    #[must_use]
    pub const fn create_response(&self) -> Option<&CreateResponse> {
        match self {
            Self::CreateCompleted(response) => Some(response),
            _ => None,
        }
    }

    /// Returns the typed Create Key Pair response, when this is one.
    #[must_use]
    pub const fn create_key_pair_response(&self) -> Option<&CreateKeyPairResponse> {
        match self {
            Self::CreateKeyPairCompleted(response) => Some(response),
            _ => None,
        }
    }

    /// Returns the typed Create Split Key response, when this is one.
    #[must_use]
    pub const fn create_split_key_response(&self) -> Option<&CreateSplitKeyResponse> {
        match self {
            Self::CreateSplitKeyCompleted(response) => Some(response),
            _ => None,
        }
    }

    /// Returns a borrowed view of the typed operation response.
    ///
    /// Existing Discover Versions result access remains available through
    /// [`ClientResponseView::result`] and [`ClientResponseView::supported_versions`].
    #[must_use]
    pub const fn response(&self) -> ClientResponseView<'_> {
        match self {
            Self::Completed(response) => ClientResponseView {
                response: ClientResponseRef::DiscoverVersions(response),
            },
            Self::CreateCompleted(response) => ClientResponseView {
                response: ClientResponseRef::Create(response),
            },
            Self::CreateKeyPairCompleted(response) => ClientResponseView {
                response: ClientResponseRef::CreateKeyPair(response),
            },
            Self::CreateSplitKeyCompleted(response) => ClientResponseView {
                response: ClientResponseRef::CreateSplitKey(response),
            },
            Self::AddAttribute(response) => ClientResponseView {
                response: ClientResponseRef::AddAttribute(response),
            },
            Self::AdjustAttribute(response) => ClientResponseView {
                response: ClientResponseRef::AdjustAttribute(response),
            },
            Self::DeleteAttribute(response) => ClientResponseView {
                response: ClientResponseRef::DeleteAttribute(response),
            },
            Self::ModifyAttribute(response) => ClientResponseView {
                response: ClientResponseRef::ModifyAttribute(response),
            },
            Self::SetAttribute(response) => ClientResponseView {
                response: ClientResponseRef::SetAttribute(response),
            },
            Self::Pending(pending) => pending.response(),
            Self::GetAttributes(response) => ClientResponseView {
                response: ClientResponseRef::GetAttributes(response),
            },
            Self::GetAttributeList(response) => ClientResponseView {
                response: ClientResponseRef::GetAttributeList(response),
            },
        }
    }
}

impl fmt::Debug for ClientBatchOutcome {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Completed(response) => {
                formatter.debug_tuple("Completed").field(response).finish()
            }
            Self::CreateCompleted(response) => formatter
                .debug_tuple("CreateCompleted")
                .field(response)
                .finish(),
            Self::CreateKeyPairCompleted(response) => formatter
                .debug_tuple("CreateKeyPairCompleted")
                .field(response)
                .finish(),
            Self::CreateSplitKeyCompleted(response) => formatter
                .debug_tuple("CreateSplitKeyCompleted")
                .field(response)
                .finish(),
            Self::AddAttribute(response) => formatter
                .debug_tuple("AddAttribute")
                .field(response)
                .finish(),
            Self::AdjustAttribute(response) => formatter
                .debug_tuple("AdjustAttribute")
                .field(response)
                .finish(),
            Self::DeleteAttribute(response) => formatter
                .debug_tuple("DeleteAttribute")
                .field(response)
                .finish(),
            Self::ModifyAttribute(response) => formatter
                .debug_tuple("ModifyAttribute")
                .field(response)
                .finish(),
            Self::SetAttribute(response) => formatter
                .debug_tuple("SetAttribute")
                .field(response)
                .finish(),
            Self::Pending(pending) => formatter.debug_tuple("Pending").field(pending).finish(),
            Self::GetAttributes(response) => formatter
                .debug_tuple("GetAttributes")
                .field(response)
                .finish(),
            Self::GetAttributeList(response) => formatter
                .debug_tuple("GetAttributeList")
                .field(response)
                .finish(),
        }
    }
}

impl fmt::Display for ClientBatchOutcome {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Completed(response) => write!(formatter, "Completed({})", response.result()),
            Self::CreateCompleted(response) => {
                write!(formatter, "CreateCompleted({})", response.result())
            }
            Self::CreateKeyPairCompleted(response) => {
                write!(formatter, "CreateKeyPairCompleted({})", response.result())
            }
            Self::CreateSplitKeyCompleted(response) => {
                write!(formatter, "CreateSplitKeyCompleted({})", response.result())
            }
            Self::AddAttribute(response) => {
                write!(formatter, "AddAttribute({})", response.result())
            }
            Self::AdjustAttribute(response) => {
                write!(formatter, "AdjustAttribute({})", response.result())
            }
            Self::DeleteAttribute(response) => {
                write!(formatter, "DeleteAttribute({})", response.result())
            }
            Self::ModifyAttribute(response) => {
                write!(formatter, "ModifyAttribute({})", response.result())
            }
            Self::SetAttribute(response) => {
                write!(formatter, "SetAttribute({})", response.result())
            }
            Self::Pending(pending) => write!(formatter, "Pending({})", pending.response().result()),
            Self::GetAttributes(response) => {
                write!(formatter, "GetAttributes({})", response.result())
            }
            Self::GetAttributeList(response) => {
                write!(formatter, "GetAttributeList({})", response.result())
            }
        }
    }
}

/// A validated result associated with its originating typed request item.
pub struct ClientBatchItemResponse {
    unique_batch_item_id: Option<Vec<u8>>,
    outcome: ClientBatchOutcome,
    extensions: Vec<ClientMessageExtension>,
    #[cfg(test)]
    pending_owner_observer: Option<ZeroizationObserver>,
}

#[cfg(test)]
impl Drop for ClientBatchItemResponse {
    fn drop(&mut self) {
        let Some(observer) = &self.pending_owner_observer else {
            return;
        };
        if let ClientBatchOutcome::Pending(pending) = &mut self.outcome {
            let asynchronous_correlation_value = &mut pending.asynchronous_correlation_value;
            asynchronous_correlation_value.as_mut_slice().zeroize();
            observer.observe_before_deallocation(asynchronous_correlation_value.as_slice());
        }
    }
}

impl ClientBatchItemResponse {
    /// Lends the associated Unique Batch Item ID, if the request supplied one.
    #[must_use]
    pub fn unique_batch_item_id(&self) -> Option<&[u8]> {
        self.unique_batch_item_id.as_deref()
    }

    /// Returns the typed item outcome.
    #[must_use]
    pub const fn outcome(&self) -> &ClientBatchOutcome {
        &self.outcome
    }

    /// Returns the accepted Message Extensions preserved from this response.
    ///
    /// Recognized critical extensions and non-critical extensions are
    /// preserved. An unrecognized critical extension causes execution to fail
    /// before a response is returned.
    #[must_use]
    pub fn extensions(&self) -> &[ClientMessageExtension] {
        &self.extensions
    }
}

impl fmt::Debug for ClientBatchItemResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ClientBatchItemResponse")
            .field(
                "has_unique_batch_item_id",
                &self.unique_batch_item_id.is_some(),
            )
            .field("outcome", &self.outcome)
            .field("extension_count", &self.extensions.len())
            .finish_non_exhaustive()
    }
}

/// Results for one explicitly executed typed request batch.
pub struct ClientBatchResponse {
    items: Vec<ClientBatchItemResponse>,
}

impl ClientBatchResponse {
    /// Returns the number of associated results.
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns whether the response contains no batch results.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns a result by its request-order index.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&ClientBatchItemResponse> {
        self.items.get(index)
    }

    /// Iterates over the results in request order, independent of response order.
    #[must_use = "use the iterator or explicitly ignore its results"]
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &ClientBatchItemResponse> + '_ {
        self.items.iter()
    }
}

impl fmt::Debug for ClientBatchResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ClientBatchResponse")
            .field("item_count", &self.items.len())
            .finish()
    }
}

/// Synchronous typed KMIP client execution foundation.
///
/// Admitted operations are explicit variants of the typed [`ClientRequest`]
/// set, including discovery, creation, attribute, and asynchronous operations.
/// Production construction accepts validated transport
/// configuration and retains the immutable client extension configuration;
/// callers cannot inject an arbitrary transport implementation.
///
/// See `docs/user-guide/en/client-execution.md` in the repository for current
/// scope, limits, redaction, and transport boundaries.
pub struct Client {
    transport: ClientTransport,
    configuration: ClientConfiguration,
    #[cfg(test)]
    request_owner_observer: Option<private_wire_writer::ZeroizationObserver>,
    #[cfg(test)]
    pending_owner_observer: Option<private_wire_writer::ZeroizationObserver>,
    #[cfg(test)]
    limits_identity_observer: Option<LimitsIdentityObserver>,
}

enum ClientTransport {
    RawTls(RawTlsTransport),
    Https(HttpsTransport),
    #[cfg(test)]
    Test(TestTransport),
}

#[cfg(test)]
#[path = "../tests/unit/production_transport_selection_tests.rs"]
mod production_transport_selection_tests;

impl ClientTransport {
    fn from_configuration(configuration: TransportConfig) -> Result<Self, ClientError> {
        if configuration.target_uri().is_some() {
            HttpsTransport::new(configuration)
                .map(Self::Https)
                .map_err(transport_configuration_error)
        } else {
            RawTlsTransport::new(configuration)
                .map(Self::RawTls)
                .map_err(transport_configuration_error)
        }
    }

    fn exchange(
        &mut self,
        request: &[u8],
        max_response_bytes: usize,
        options: &RequestOptions,
    ) -> Result<kmipkit_transport::TransportResponse, kmipkit_transport::TransportError> {
        match self {
            Self::RawTls(transport) => {
                transport.exchange_with_options(request, max_response_bytes, options)
            }
            Self::Https(transport) => {
                transport.exchange_with_options(request, max_response_bytes, options)
            }
            #[cfg(test)]
            Self::Test(transport) => {
                transport.exchange_with_options(request, max_response_bytes, options)
            }
        }
    }
}

fn transport_configuration_error(error: TransportConfigError) -> ClientError {
    ClientError::validation(
        ClientCauseCategory::InvalidInput,
        RequestDeliveryState::NotSent,
        error,
    )
}

impl Client {
    /// Creates a typed client from its immutable extension configuration and
    /// one validated production transport configuration.
    ///
    /// The endpoint selects raw TLS or HTTPS. Construction completes TLS
    /// policy setup without resolving the endpoint or opening a socket. The
    /// client does not accept caller-implemented transports or raw request
    /// bytes.
    ///
    /// # Errors
    ///
    /// Returns sanitized `InvalidInput`/`NotSent` if the selected adapter
    /// cannot be created from the supplied validated configuration.
    pub fn new(
        configuration: ClientConfiguration,
        transport_configuration: TransportConfig,
    ) -> Result<Self, ClientError> {
        let transport = ClientTransport::from_configuration(transport_configuration)?;
        Ok(Self {
            transport,
            configuration,
            #[cfg(test)]
            request_owner_observer: None,
            #[cfg(test)]
            pending_owner_observer: None,
            #[cfg(test)]
            limits_identity_observer: None,
        })
    }

    /// Executes one explicitly supplied typed request batch and performs one exchange.
    ///
    /// The same borrowed `limits` value reaches the private request writer and
    /// response decoder. The response byte cap is exactly
    /// [`CodecLimits::max_message_bytes`]. No Poll, Cancel, retry, failover,
    /// implicit Discover Versions call, or background wait is performed.
    ///
    /// Every request advertises `Attestation Capable Indicator = True` because
    /// the public protocol API can construct an Attestation Credential. This
    /// reports construction capability only: it does not submit
    /// Authentication or Credential data, generate or verify evidence, or
    /// guarantee server acceptance. There is no per-request override.
    ///
    /// # Errors
    ///
    /// Returns a sanitized validation, protocol, or transport error with the
    /// strongest available request-delivery evidence.
    // The approved API takes ownership of the caller's typed batch. It keeps
    // the execution lifetime local and avoids retaining request values after
    // the synchronous exchange completes.
    #[allow(clippy::needless_pass_by_value)]
    pub fn execute(
        &mut self,
        batch: ClientBatch,
        limits: &CodecLimits,
    ) -> Result<ClientBatchResponse, ClientError> {
        self.execute_with_options(batch, limits, &RequestOptions::default())
    }

    /// Executes one typed request batch with per-exchange timeout overrides.
    ///
    /// Unspecified timeout phases inherit the validated transport
    /// configuration's timeout policy. The same borrowed `limits` value
    /// bounds request encoding and response decoding, and exactly
    /// `limits.max_message_bytes()` is passed as the response byte cap.
    ///
    /// # Errors
    ///
    /// Returns a sanitized validation, protocol, or transport error with the
    /// strongest available request-delivery evidence.
    #[allow(clippy::needless_pass_by_value)]
    pub fn execute_with_options(
        &mut self,
        batch: ClientBatch,
        limits: &CodecLimits,
        request_options: &RequestOptions,
    ) -> Result<ClientBatchResponse, ClientError> {
        let options = validate_batch(&batch).map_err(|error| {
            ClientError::validation(
                ClientCauseCategory::InvalidInput,
                RequestDeliveryState::NotSent,
                error,
            )
        })?;
        validate_request_extension_ownership(&batch, &self.configuration).map_err(|error| {
            ClientError::validation(
                ClientCauseCategory::InvalidInput,
                RequestDeliveryState::NotSent,
                error,
            )
        })?;
        validate_attribute_mutation_policy(&batch)?;

        let request_identities = batch
            .items
            .iter()
            .map(BatchIdentity::from_request)
            .collect::<Vec<_>>();
        let maximum_response_size = maximum_response_size_for_batch(&batch, limits);
        let request_message = build_request_message(batch, &options, maximum_response_size)
            .map_err(|error| protocol_failure_at(error, RequestDeliveryState::NotSent))?;
        let (response_message, response_delivery_state) =
            self.exchange_operation(request_message, limits, request_options)?;

        validate_response(
            &request_identities,
            &options,
            response_message,
            self.configuration.extension_registry(),
            limits,
            #[cfg(test)]
            self.pending_owner_observer.as_ref(),
        )
        .map_err(|error| protocol_failure_at(error, response_delivery_state))
    }

    /// Executes one typed Create request through the shared batch writer.
    ///
    /// Use [`Self::execute`] with a one-item batch and an explicit
    /// Asynchronous Indicator when the caller wants to accept Operation
    /// Pending. This convenience method leaves batch options at their defaults.
    ///
    /// # Errors
    ///
    /// Returns a sanitized validation, protocol, or transport error with the
    /// strongest available request-delivery evidence.
    pub fn create(
        &mut self,
        request: CreateRequest,
        limits: &CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        self.create_with_options(request, limits, &RequestOptions::default())
    }

    /// Executes one typed Create request with transport timeout overrides.
    ///
    /// The request uses the same writer, response bounds, and one-exchange
    /// lifecycle as [`Self::execute_with_options`].
    ///
    /// # Errors
    ///
    /// Returns a sanitized validation, protocol, or transport error with the
    /// strongest available request-delivery evidence.
    pub fn create_with_options(
        &mut self,
        request: CreateRequest,
        limits: &CodecLimits,
        request_options: &RequestOptions,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let mut response = self.execute_with_options(
            ClientBatch::new(ClientBatchItem::new(ClientRequest::Create(request))),
            limits,
            request_options,
        )?;
        response.items.pop().ok_or_else(|| {
            protocol_failure_at(
                protocol_error(ProtocolErrorKind::MalformedMessage),
                RequestDeliveryState::ResponseStarted,
            )
        })
    }

    /// Executes one typed Create Key Pair request through the shared batch writer.
    ///
    /// Use [`Self::execute`] with a one-item batch and an explicit
    /// Asynchronous Indicator when the caller wants to accept Operation
    /// Pending. This convenience method leaves batch options at their defaults.
    ///
    /// # Errors
    ///
    /// Returns a sanitized validation, protocol, or transport error with the
    /// strongest available request-delivery evidence.
    pub fn create_key_pair(
        &mut self,
        request: CreateKeyPairRequest,
        limits: &CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        self.create_key_pair_with_options(request, limits, &RequestOptions::default())
    }

    /// Executes one typed Create Key Pair request with transport timeout overrides.
    ///
    /// The request uses the same writer, response bounds, and one-exchange
    /// lifecycle as [`Self::execute_with_options`].
    ///
    /// # Errors
    ///
    /// Returns a sanitized validation, protocol, or transport error with the
    /// strongest available request-delivery evidence.
    pub fn create_key_pair_with_options(
        &mut self,
        request: CreateKeyPairRequest,
        limits: &CodecLimits,
        request_options: &RequestOptions,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let mut response = self.execute_with_options(
            ClientBatch::new(ClientBatchItem::new(ClientRequest::CreateKeyPair(request))),
            limits,
            request_options,
        )?;
        response.items.pop().ok_or_else(|| {
            protocol_failure_at(
                protocol_error(ProtocolErrorKind::MalformedMessage),
                RequestDeliveryState::ResponseStarted,
            )
        })
    }

    /// Executes one typed Create Split Key request through the shared batch writer.
    ///
    /// The request advertises the local response byte limit as KMIP
    /// `Maximum Response Size`, clamped to the field's signed 32-bit range.
    /// Use [`Self::execute`] with a one-item batch and an explicit
    /// Asynchronous Indicator when the caller wants to accept Operation
    /// Pending. This convenience method leaves batch options at their defaults.
    ///
    /// # Errors
    ///
    /// Returns a sanitized validation, protocol, or transport error with the
    /// strongest available request-delivery evidence.
    pub fn create_split_key(
        &mut self,
        request: CreateSplitKeyRequest,
        limits: &CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        self.create_split_key_with_options(request, limits, &RequestOptions::default())
    }

    /// Executes one typed Create Split Key request with transport timeout overrides.
    ///
    /// The request uses the shared writer, response bounds, and one-exchange
    /// lifecycle. Its `Maximum Response Size` is derived from `limits`.
    ///
    /// # Errors
    ///
    /// Returns a sanitized validation, protocol, or transport error with the
    /// strongest available request-delivery evidence.
    pub fn create_split_key_with_options(
        &mut self,
        request: CreateSplitKeyRequest,
        limits: &CodecLimits,
        request_options: &RequestOptions,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let mut response = self.execute_with_options(
            ClientBatch::new(ClientBatchItem::new(ClientRequest::CreateSplitKey(request))),
            limits,
            request_options,
        )?;
        response.items.pop().ok_or_else(|| {
            protocol_failure_at(
                protocol_error(ProtocolErrorKind::MalformedMessage),
                RequestDeliveryState::ResponseStarted,
            )
        })
    }

    fn exchange_operation(
        &mut self,
        request_message: RequestMessage,
        limits: &CodecLimits,
        request_options: &RequestOptions,
    ) -> Result<(ResponseMessage, RequestDeliveryState), ClientError> {
        let request_item = root_message_item(request_message.into_ttlv())
            .map_err(|error| protocol_failure_at(error, RequestDeliveryState::NotSent))?;
        let permit = OperationEncodingPermit::mint();

        #[cfg(test)]
        let cleanup_observer = self.request_owner_observer.clone();
        #[cfg(test)]
        let encoding_observer = private_wire_writer::EncodingObserver::default();
        let encoded = private_wire_writer::encode_for_execute(
            &request_item,
            limits,
            permit,
            #[cfg(test)]
            self.limits_identity_observer.as_ref(),
            #[cfg(test)]
            &encoding_observer,
            #[cfg(test)]
            cleanup_observer,
        )
        .map_err(|error| {
            let protocol = ProtocolError::new(
                ProtocolErrorKind::InvalidValue,
                ProtocolCauseCategory::InvalidEncoding,
                error,
            );
            protocol_failure_at(protocol, RequestDeliveryState::NotSent)
        })?;

        let transport_result = self.transport.exchange(
            encoded.as_bytes(),
            limits.max_message_bytes(),
            request_options,
        );
        drop(encoded);

        let response = transport_result.map_err(ClientError::transport)?;
        decode_transport_response(
            response,
            limits,
            #[cfg(test)]
            self.limits_identity_observer.as_ref(),
        )
    }

    /// Executes one explicit Poll request and returns the original operation's
    /// state without polling again when it remains Pending.
    ///
    /// The request uses the exact correlation bytes in `request`. Poll itself
    /// is not marked asynchronous; its Pending result describes the original
    /// operation under OASIS KMIP v2.1 §6.1.38.
    ///
    /// The caller decides whether another Poll is appropriate:
    ///
    /// ```
    /// use kmipkit_client::{Client, ClientError};
    /// use kmipkit_protocol::PollRequest;
    /// use kmipkit_ttlv::codec::CodecLimits;
    ///
    /// fn poll_once(
    ///     client: &mut Client,
    ///     correlation: &[u8],
    /// ) -> Result<bool, ClientError> {
    ///     let outcome = client.execute_poll(
    ///         PollRequest::new(correlation),
    ///         &CodecLimits::defaults(),
    ///     )?;
    ///     if outcome.is_pending() {
    ///         let _ = outcome.with_asynchronous_correlation_value(|bytes| bytes.len());
    ///     }
    ///     Ok(outcome.is_pending())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a sanitized validation, protocol, or transport error with the
    /// request delivery state when available.
    pub fn execute_poll(
        &mut self,
        request: PollRequest,
        limits: &CodecLimits,
    ) -> Result<ClientOperationOutcome, ClientError> {
        self.execute_poll_with_options(request, limits, &RequestOptions::default())
    }

    /// Executes one Poll request with per-exchange timeout overrides.
    ///
    /// # Errors
    ///
    /// Returns a sanitized validation, protocol, or transport error with the
    /// request delivery state when available.
    pub fn execute_poll_with_options(
        &mut self,
        request: PollRequest,
        limits: &CodecLimits,
        request_options: &RequestOptions,
    ) -> Result<ClientOperationOutcome, ClientError> {
        let payload = request
            .to_ttlv_payload()
            .map_err(|error| protocol_failure_at(error, RequestDeliveryState::NotSent))?;
        drop(request);
        self.execute_async_request(
            POLL_OPERATION,
            payload,
            ClientOperation::Poll,
            None,
            None,
            limits,
            request_options,
        )
    }

    /// Executes one explicit Cancel request and verifies the successful response
    /// echoes the exact correlation bytes supplied by the caller.
    ///
    /// Cancel is synchronous under OASIS KMIP v2.1 §6.1.5; a Pending response
    /// is rejected even when the original operation was asynchronous.
    ///
    /// # Errors
    ///
    /// Returns a sanitized validation, protocol, or transport error with the
    /// request delivery state when available.
    pub fn execute_cancel(
        &mut self,
        request: CancelRequest,
        limits: &CodecLimits,
    ) -> Result<ClientOperationOutcome, ClientError> {
        self.execute_cancel_with_options(request, limits, &RequestOptions::default())
    }

    /// Executes one Cancel request with per-exchange timeout overrides.
    ///
    /// # Errors
    ///
    /// Returns a sanitized validation, protocol, or transport error with the
    /// request delivery state when available.
    pub fn execute_cancel_with_options(
        &mut self,
        request: CancelRequest,
        limits: &CodecLimits,
        request_options: &RequestOptions,
    ) -> Result<ClientOperationOutcome, ClientError> {
        let correlation = request.asynchronous_correlation_value();
        let payload = request
            .to_ttlv_payload()
            .map_err(|error| protocol_failure_at(error, RequestDeliveryState::NotSent))?;
        let outcome = self.execute_async_request(
            CANCEL_OPERATION,
            payload,
            ClientOperation::Cancel,
            None,
            Some(correlation),
            limits,
            request_options,
        )?;
        drop(request);
        Ok(outcome)
    }

    /// Executes one explicit Process request using the caller-selected
    /// Asynchronous Indicator for the Process operation itself.
    ///
    /// A Pending Process outcome is returned directly; `KMIPKit` does not wait,
    /// Poll, or claim that a later Poll will complete.
    ///
    /// # Errors
    ///
    /// Returns a sanitized validation, protocol, or transport error with the
    /// request delivery state when available.
    pub fn execute_process(
        &mut self,
        request: ProcessRequest,
        asynchronous_indicator: Option<u32>,
        limits: &CodecLimits,
    ) -> Result<ClientOperationOutcome, ClientError> {
        self.execute_process_with_options(
            request,
            asynchronous_indicator,
            limits,
            &RequestOptions::default(),
        )
    }

    /// Executes one Process request with per-exchange timeout overrides.
    ///
    /// # Errors
    ///
    /// Returns a sanitized validation, protocol, or transport error with the
    /// request delivery state when available.
    pub fn execute_process_with_options(
        &mut self,
        request: ProcessRequest,
        asynchronous_indicator: Option<u32>,
        limits: &CodecLimits,
        request_options: &RequestOptions,
    ) -> Result<ClientOperationOutcome, ClientError> {
        validate_follow_up_indicator(asynchronous_indicator)?;
        let payload = request
            .to_ttlv_payload()
            .map_err(|error| protocol_failure_at(error, RequestDeliveryState::NotSent))?;
        drop(request);
        self.execute_async_request(
            PROCESS_OPERATION,
            payload,
            ClientOperation::Process,
            asynchronous_indicator,
            None,
            limits,
            request_options,
        )
    }

    /// Executes one Query Asynchronous Requests operation and exposes its
    /// response payload generically while `KMIPKIT-DISC-039` remains unresolved.
    ///
    /// # Errors
    ///
    /// Returns a sanitized validation, protocol, or transport error with the
    /// request delivery state when available.
    pub fn execute_query_async_requests(
        &mut self,
        request: QueryAsyncRequestsRequest,
        asynchronous_indicator: Option<u32>,
        limits: &CodecLimits,
    ) -> Result<ClientOperationOutcome, ClientError> {
        self.execute_query_async_requests_with_options(
            request,
            asynchronous_indicator,
            limits,
            &RequestOptions::default(),
        )
    }

    /// Executes one Query Asynchronous Requests operation with per-exchange
    /// timeout overrides.
    ///
    /// # Errors
    ///
    /// Returns a sanitized validation, protocol, or transport error with the
    /// request delivery state when available.
    pub fn execute_query_async_requests_with_options(
        &mut self,
        request: QueryAsyncRequestsRequest,
        asynchronous_indicator: Option<u32>,
        limits: &CodecLimits,
        request_options: &RequestOptions,
    ) -> Result<ClientOperationOutcome, ClientError> {
        validate_follow_up_indicator(asynchronous_indicator)?;
        let payload = request
            .to_ttlv_payload()
            .map_err(|error| protocol_failure_at(error, RequestDeliveryState::NotSent))?;
        drop(request);
        self.execute_async_request(
            QUERY_ASYNCHRONOUS_REQUESTS_OPERATION,
            payload,
            ClientOperation::QueryAsyncRequests,
            asynchronous_indicator,
            None,
            limits,
            request_options,
        )
    }

    #[allow(clippy::too_many_arguments)] // Keeps each validated async-operation field explicit.
    fn execute_async_request(
        &mut self,
        operation: u32,
        payload: Structure,
        kind: ClientOperation,
        asynchronous_indicator: Option<u32>,
        expected_cancel_correlation: Option<&[u8]>,
        limits: &CodecLimits,
        request_options: &RequestOptions,
    ) -> Result<ClientOperationOutcome, ClientError> {
        let request_message =
            build_async_request_message(operation, payload, asynchronous_indicator)
                .map_err(|error| protocol_failure_at(error, RequestDeliveryState::NotSent))?;
        let (response, delivery_state) =
            self.exchange_operation(request_message, limits, request_options)?;
        validate_async_response(
            response,
            operation,
            kind,
            asynchronous_indicator,
            expected_cancel_correlation,
            self.configuration.extension_registry(),
            limits,
        )
        .map_err(|error| protocol_failure_at(error, delivery_state))
    }

    #[cfg(test)]
    pub(super) fn for_test<T: Transport + 'static>(transport: T) -> Self {
        Self::for_test_with_configuration(transport, empty_test_configuration())
    }

    #[cfg(test)]
    pub(super) fn for_test_with_configuration<T: Transport + 'static>(
        transport: T,
        configuration: ClientConfiguration,
    ) -> Self {
        Self {
            transport: ClientTransport::Test(TestTransport::new(transport)),
            configuration,
            request_owner_observer: None,
            pending_owner_observer: None,
            limits_identity_observer: None,
        }
    }

    #[cfg(test)]
    pub(super) fn for_test_with_request_observer<T: Transport + 'static>(
        transport: T,
        observer: private_wire_writer::ZeroizationObserver,
    ) -> Self {
        Self {
            transport: ClientTransport::Test(TestTransport::new(transport)),
            configuration: empty_test_configuration(),
            request_owner_observer: Some(observer),
            pending_owner_observer: None,
            limits_identity_observer: None,
        }
    }

    #[cfg(test)]
    pub(super) fn for_test_with_configuration_and_request_observer<T: Transport + 'static>(
        transport: T,
        configuration: ClientConfiguration,
        observer: private_wire_writer::ZeroizationObserver,
    ) -> Self {
        Self {
            transport: ClientTransport::Test(TestTransport::new(transport)),
            configuration,
            request_owner_observer: Some(observer),
            pending_owner_observer: None,
            limits_identity_observer: None,
        }
    }

    #[cfg(test)]
    pub(super) fn for_test_with_limits_observer<T: Transport + 'static>(
        transport: T,
        observer: LimitsIdentityObserver,
    ) -> Self {
        Self {
            transport: ClientTransport::Test(TestTransport::new(transport)),
            configuration: empty_test_configuration(),
            request_owner_observer: None,
            pending_owner_observer: None,
            limits_identity_observer: Some(observer),
        }
    }

    #[cfg(test)]
    pub(super) fn for_test_with_lifecycle_observers<T: Transport + 'static>(
        transport: T,
        request_observer: ZeroizationObserver,
        pending_observer: ZeroizationObserver,
    ) -> Self {
        Self {
            transport: ClientTransport::Test(TestTransport::new(transport)),
            configuration: empty_test_configuration(),
            request_owner_observer: Some(request_observer),
            pending_owner_observer: Some(pending_observer),
            limits_identity_observer: None,
        }
    }
}

#[cfg(test)]
fn empty_test_configuration() -> ClientConfiguration {
    let registry = crate::extension_registry::client_extension_registry(
        Vec::new(),
        kmipkit_protocol::extension::defaults(),
    )
    .expect("an empty extension registry is valid");
    ClientConfiguration::new(registry)
}

struct OperationEncodingPermit {
    _private: (),
}

impl OperationEncodingPermit {
    fn mint() -> Self {
        Self { _private: () }
    }
}

#[cfg(test)]
#[derive(Clone)]
pub(super) struct LimitsIdentityObserver {
    expected: *const CodecLimits,
    encode_same: std::rc::Rc<std::cell::Cell<bool>>,
    decode_same: std::rc::Rc<std::cell::Cell<bool>>,
    decode_calls: std::rc::Rc<std::cell::Cell<usize>>,
}

#[cfg(test)]
impl LimitsIdentityObserver {
    pub(super) fn new(expected: &CodecLimits) -> Self {
        Self {
            expected,
            encode_same: std::rc::Rc::new(std::cell::Cell::new(false)),
            decode_same: std::rc::Rc::new(std::cell::Cell::new(false)),
            decode_calls: std::rc::Rc::new(std::cell::Cell::new(0)),
        }
    }

    fn record_encode(&self, actual: &CodecLimits) {
        self.encode_same.set(std::ptr::eq(self.expected, actual));
    }

    fn record_decode(&self, actual: &CodecLimits) {
        self.decode_calls.set(self.decode_calls.get() + 1);
        self.decode_same.set(std::ptr::eq(self.expected, actual));
    }

    pub(super) fn both_same(&self) -> bool {
        self.encode_same.get() && self.decode_same.get()
    }

    pub(super) fn decode_calls(&self) -> usize {
        self.decode_calls.get()
    }
}

#[cfg(test)]
pub(super) fn encode_message_for_test(
    tree: Structure,
    limits: &CodecLimits,
) -> Result<Vec<u8>, ProtocolError> {
    let item = root_message_item(tree)?;
    let encoded = private_wire_writer::encode_item_for_test(&item, limits).map_err(|error| {
        ProtocolError::new(
            ProtocolErrorKind::InvalidValue,
            ProtocolCauseCategory::InvalidEncoding,
            error,
        )
    })?;
    Ok(encoded.as_bytes().to_vec())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum BatchValidationError {
    EmptyBatch,
    MissingBatchItemId,
    DuplicateBatchItemId,
    InvalidAsynchronousIndicator,
    RepeatedBatchErrorContinuation,
    SingleItemBatchErrorContinuation,
    InvalidBatchErrorContinuation,
    ExtensionRegistryMismatch,
}

impl fmt::Display for BatchValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::EmptyBatch => "client batch must contain an item",
            Self::MissingBatchItemId => "multi-item request requires every batch item ID",
            Self::DuplicateBatchItemId => "request batch item IDs must be unique",
            Self::InvalidAsynchronousIndicator => "asynchronous indicator is unassigned",
            Self::RepeatedBatchErrorContinuation => "batch error continuation is repeated",
            Self::SingleItemBatchErrorContinuation => {
                "batch error continuation is invalid for a single item"
            }
            Self::InvalidBatchErrorContinuation => "batch error continuation is unassigned",
            Self::ExtensionRegistryMismatch => {
                "request extension was validated for a different client registry"
            }
        };
        formatter.write_str(message)
    }
}

impl Error for BatchValidationError {}

pub(super) fn validate_batch(
    batch: &ClientBatch,
) -> Result<ValidatedBatchOptions, BatchValidationError> {
    if batch.items.is_empty() {
        return Err(BatchValidationError::EmptyBatch);
    }
    let options = validate_batch_options(batch)?;

    let mut identifiers = HashSet::new();
    for item in &batch.items {
        match (&item.unique_batch_item_id, batch.items.len()) {
            (None, count) if count > 1 => return Err(BatchValidationError::MissingBatchItemId),
            (Some(id), _) if !identifiers.insert(id.as_slice()) => {
                return Err(BatchValidationError::DuplicateBatchItemId);
            }
            _ => {}
        }
    }
    Ok(options)
}

fn validate_request_extension_ownership(
    batch: &ClientBatch,
    configuration: &ClientConfiguration,
) -> Result<(), BatchValidationError> {
    if batch.items.iter().any(|item| {
        item.message_extensions
            .iter()
            .any(|extension| !extension.is_owned_by(configuration))
    }) {
        return Err(BatchValidationError::ExtensionRegistryMismatch);
    }
    Ok(())
}

#[derive(Debug)]
struct SourceBackedMutationProhibited;

impl fmt::Display for SourceBackedMutationProhibited {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("attribute mutation is prohibited by source-backed KMIP policy")
    }
}

impl Error for SourceBackedMutationProhibited {}

fn validate_attribute_mutation_policy(batch: &ClientBatch) -> Result<(), ClientError> {
    if batch
        .items
        .iter()
        .any(|item| request_mutation_is_prohibited(&item.request))
    {
        Err(ClientError::validation(
            ClientCauseCategory::InvalidInput,
            RequestDeliveryState::NotSent,
            SourceBackedMutationProhibited,
        ))
    } else {
        Ok(())
    }
}

fn request_mutation_is_prohibited(request: &ClientRequest) -> bool {
    match request {
        ClientRequest::AddAttribute(request) => {
            new_attribute_is_prohibited(request.new_attribute(), ClientAttributeMutation::Add)
        }
        ClientRequest::AdjustAttribute(request) => attribute_reference_is_prohibited(
            request.attribute_reference(),
            ClientAttributeMutation::Adjust,
        ),
        ClientRequest::DeleteAttribute(request) => {
            request.current_attribute().is_some_and(|attribute| {
                attribute_item_is_prohibited(attribute.item(), ClientAttributeMutation::Delete)
            }) || request.attribute_reference().is_some_and(|reference| {
                attribute_reference_is_prohibited(reference, ClientAttributeMutation::Delete)
            })
        }
        ClientRequest::ModifyAttribute(request) => {
            request.current_attribute().is_some_and(|attribute| {
                attribute_item_is_prohibited(attribute.item(), ClientAttributeMutation::Modify)
            }) || new_attribute_is_prohibited(
                request.new_attribute(),
                ClientAttributeMutation::Modify,
            )
        }
        ClientRequest::SetAttribute(request) => {
            new_attribute_is_prohibited(request.new_attribute(), ClientAttributeMutation::Set)
        }
        ClientRequest::DiscoverVersions(_)
        | ClientRequest::Create(_)
        | ClientRequest::CreateKeyPair(_)
        | ClientRequest::CreateSplitKey(_)
        | ClientRequest::GetAttributes(_)
        | ClientRequest::GetAttributeList(_) => false,
    }
}

fn new_attribute_is_prohibited(
    attribute: &NewAttribute,
    mutation: ClientAttributeMutation,
) -> bool {
    attribute_item_is_prohibited(attribute.item(), mutation)
}

fn attribute_item_is_prohibited(item: &Item, mutation: ClientAttributeMutation) -> bool {
    client_attribute_mutation_is_prohibited(item.tag().raw(), mutation)
        || vendor_attribute_item_is_prohibited(item, mutation)
}

fn attribute_reference_is_prohibited(
    reference: &AttributeReference,
    mutation: ClientAttributeMutation,
) -> bool {
    if let Some(tag) = reference.tag_value() {
        client_attribute_mutation_is_prohibited(tag, mutation)
    } else if let Some((vendor_identification, _attribute_name)) = reference.name_parts() {
        client_vendor_attribute_mutation_is_prohibited(vendor_identification, mutation)
    } else {
        false
    }
}

fn vendor_attribute_item_is_prohibited(item: &Item, mutation: ClientAttributeMutation) -> bool {
    const VENDOR_ATTRIBUTE_TAG: u32 = 0x0042_0008;
    const VENDOR_IDENTIFICATION_TAG: u32 = 0x0042_009D;

    if item.tag().raw() != VENDOR_ATTRIBUTE_TAG {
        return false;
    }
    item.with_value(|value| {
        let ValueView::Structure(structure) = value else {
            return false;
        };
        structure
            .children()
            .iter()
            .filter(|field| field.tag().raw() == VENDOR_IDENTIFICATION_TAG)
            .any(|identifier| {
                identifier.with_value(|value| match value {
                    ValueView::TextString(value) => {
                        client_vendor_attribute_mutation_is_prohibited(value, mutation)
                    }
                    _ => false,
                })
            })
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ValidatedBatchOptions {
    asynchronous_indicator: Option<u32>,
    batch_error_continuation: BatchErrorContinuation,
}

fn validate_batch_options(
    batch: &ClientBatch,
) -> Result<ValidatedBatchOptions, BatchValidationError> {
    let asynchronous_indicator = validate_asynchronous_indicator(batch.asynchronous_indicator)
        .map_err(|()| BatchValidationError::InvalidAsynchronousIndicator)?;
    let batch_error_continuation = validate_batch_error_continuation(
        batch.items.len(),
        &batch.batch_error_continuation_values,
    )?;
    Ok(ValidatedBatchOptions {
        asynchronous_indicator,
        batch_error_continuation,
    })
}

pub(super) fn validate_asynchronous_indicator(raw: Option<u32>) -> Result<Option<u32>, ()> {
    match raw {
        None
        | Some(
            ASYNCHRONOUS_MANDATORY..=ASYNCHRONOUS_PROHIBITED
            | ENUMERATION_EXTENSION_MIN..=ENUMERATION_EXTENSION_MAX,
        ) => Ok(raw),
        Some(_) => Err(()),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct BatchErrorContinuation {
    pub(super) encoded: Option<u32>,
    pub(super) effective: u32,
}

pub(super) fn validate_batch_error_continuation(
    batch_count: usize,
    values: &[u32],
) -> Result<BatchErrorContinuation, BatchValidationError> {
    if values.len() > 1 {
        return Err(BatchValidationError::RepeatedBatchErrorContinuation);
    }
    if !values.is_empty() && batch_count == 1 {
        return Err(BatchValidationError::SingleItemBatchErrorContinuation);
    }
    let encoded = values.first().copied();
    if let Some(value) = encoded
        && !matches!(value, 1..=3)
    {
        return Err(BatchValidationError::InvalidBatchErrorContinuation);
    }
    Ok(BatchErrorContinuation {
        encoded,
        effective: encoded.unwrap_or(BATCH_ERROR_CONTINUATION_STOP),
    })
}

fn build_request_message(
    batch: ClientBatch,
    options: &ValidatedBatchOptions,
    maximum_response_size: Option<i32>,
) -> Result<RequestMessage, ProtocolError> {
    let header = build_request_header(&batch, options, maximum_response_size)?;
    let mut tree = Structure::new();
    push(&mut tree, REQUEST_HEADER, Value::structure(header))?;
    for item in batch.items {
        push(
            &mut tree,
            BATCH_ITEM,
            Value::structure(build_request_batch_item(item)?),
        )?;
    }

    RequestMessage::try_from_ttlv(tree).map_err(|error| {
        ProtocolError::new(
            ProtocolErrorKind::MalformedMessage,
            ProtocolCauseCategory::InvalidValue,
            error,
        )
    })
}

fn build_request_header(
    batch: &ClientBatch,
    options: &ValidatedBatchOptions,
    maximum_response_size: Option<i32>,
) -> Result<Structure, ProtocolError> {
    let mut version_fields = Structure::new();
    push(
        &mut version_fields,
        PROTOCOL_VERSION_MAJOR,
        Value::integer(2),
    )?;
    push(
        &mut version_fields,
        PROTOCOL_VERSION_MINOR,
        Value::integer(1),
    )?;

    let mut header = Structure::new();
    push(
        &mut header,
        PROTOCOL_VERSION,
        Value::structure(version_fields),
    )?;
    if let Some(value) = maximum_response_size {
        push(&mut header, MAXIMUM_RESPONSE_SIZE, Value::integer(value))?;
    }
    if let Some(value) = &batch.client_correlation_value {
        push(
            &mut header,
            CLIENT_CORRELATION_VALUE,
            Value::text_string(value.clone()),
        )?;
    }
    if let Some(indicator) = batch.asynchronous_indicator {
        push(
            &mut header,
            ASYNCHRONOUS_INDICATOR,
            Value::enumeration(indicator),
        )?;
    }
    push(
        &mut header,
        ATTESTATION_CAPABLE_INDICATOR,
        Value::boolean(true),
    )?;
    if let Some(value) = options.batch_error_continuation.encoded {
        push(
            &mut header,
            BATCH_ERROR_CONTINUATION_OPTION,
            Value::enumeration(value),
        )?;
    }
    if let Some(value) = batch.batch_order_option {
        push(&mut header, BATCH_ORDER_OPTION, Value::boolean(value))?;
    }
    if let Some(value) = batch.request_time_stamp {
        push(&mut header, TIME_STAMP, Value::date_time(value))?;
    }
    let count = i32::try_from(batch.items.len()).map_err(|error| {
        ProtocolError::new(
            ProtocolErrorKind::InvalidValue,
            ProtocolCauseCategory::InvalidValue,
            error,
        )
    })?;
    push(&mut header, BATCH_COUNT, Value::integer(count))?;
    Ok(header)
}

fn maximum_response_size_for_batch(batch: &ClientBatch, limits: &CodecLimits) -> Option<i32> {
    batch
        .items
        .iter()
        .any(|item| item.request.operation() == CREATE_SPLIT_KEY_OPERATION)
        .then(|| i32::try_from(limits.max_message_bytes()).unwrap_or(i32::MAX))
}

fn build_request_batch_item(item: ClientBatchItem) -> Result<Structure, ProtocolError> {
    let ClientBatchItem {
        request,
        unique_batch_item_id,
        message_extensions,
    } = item;
    let mut batch_item = Structure::new();
    push(
        &mut batch_item,
        OPERATION,
        Value::enumeration(request.operation()),
    )?;
    if let Some(id) = unique_batch_item_id {
        push(
            &mut batch_item,
            UNIQUE_BATCH_ITEM_ID,
            Value::byte_string(id),
        )?;
    }
    push(
        &mut batch_item,
        REQUEST_PAYLOAD,
        Value::structure(request.payload()?),
    )?;
    for extension in &message_extensions {
        push(
            &mut batch_item,
            MESSAGE_EXTENSION,
            Value::structure(message_extension_structure(extension)?),
        )?;
    }
    Ok(batch_item)
}

fn message_extension_structure(
    extension: &ClientRequestMessageExtension,
) -> Result<Structure, ProtocolError> {
    let validated = extension.value().value();
    let identity = kmipkit_protocol::extension::validated_extension_value_identity(validated);
    let payload = copy_structure(&kmipkit_protocol::extension::generic_value(validated).view())?;
    structure([
        (
            0x0042_009D,
            Value::text_string(identity.vendor_identifier().to_owned()),
        ),
        (
            0x0042_0026,
            Value::boolean(extension.criticality_indicator()),
        ),
        (0x0042_009C, Value::structure(payload)),
    ])
}

fn build_async_request_message(
    operation: u32,
    payload: Structure,
    asynchronous_indicator: Option<u32>,
) -> Result<RequestMessage, ProtocolError> {
    let version_fields = structure([
        (PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        (PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ])?;
    let mut header = Structure::new();
    push(
        &mut header,
        PROTOCOL_VERSION,
        Value::structure(version_fields),
    )?;
    if let Some(indicator) = asynchronous_indicator {
        push(
            &mut header,
            ASYNCHRONOUS_INDICATOR,
            Value::enumeration(indicator),
        )?;
    }
    push(
        &mut header,
        ATTESTATION_CAPABLE_INDICATOR,
        Value::boolean(true),
    )?;
    push(&mut header, BATCH_COUNT, Value::integer(1))?;

    let mut batch_item = Structure::new();
    push(&mut batch_item, OPERATION, Value::enumeration(operation))?;
    push(&mut batch_item, REQUEST_PAYLOAD, Value::structure(payload))?;

    let mut tree = Structure::new();
    push(&mut tree, REQUEST_HEADER, Value::structure(header))?;
    push(&mut tree, BATCH_ITEM, Value::structure(batch_item))?;
    RequestMessage::try_from_ttlv(tree).map_err(|error| {
        ProtocolError::new(
            ProtocolErrorKind::MalformedMessage,
            ProtocolCauseCategory::InvalidValue,
            error,
        )
    })
}

fn structure(fields: impl IntoIterator<Item = (u32, Value)>) -> Result<Structure, ProtocolError> {
    let mut structure = Structure::new();
    for (raw_tag, value) in fields {
        push(&mut structure, raw_tag, value)?;
    }
    Ok(structure)
}

fn root_message_item(tree: Structure) -> Result<Item, ProtocolError> {
    Item::new(checked_tag(MESSAGE)?, Value::structure(tree)).map_err(model_protocol_error)
}

fn push(structure: &mut Structure, raw_tag: u32, value: Value) -> Result<(), ProtocolError> {
    let item = Item::new(checked_tag(raw_tag)?, value).map_err(model_protocol_error)?;
    structure.try_push(item).map_err(model_protocol_error)
}

fn checked_tag(raw_tag: u32) -> Result<Tag, ProtocolError> {
    RawTag::new(raw_tag)
        .and_then(|tag| tag.try_checked())
        .map_err(model_protocol_error)
}

fn model_protocol_error(error: ModelError) -> ProtocolError {
    ProtocolError::new(
        ProtocolErrorKind::InvalidValue,
        ProtocolCauseCategory::InvalidValue,
        error,
    )
}

pub(super) fn protocol_version_is_supported(version: ProtocolVersion) -> bool {
    version.major() == 2 && version.minor() == 1
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct BatchIdentity {
    pub(super) operation: u32,
    pub(super) unique_batch_item_id: Option<Vec<u8>>,
}

impl BatchIdentity {
    fn from_request(item: &ClientBatchItem) -> Self {
        Self {
            operation: item.request.operation(),
            unique_batch_item_id: item.unique_batch_item_id.clone(),
        }
    }

    fn from_response(item: &ResponseBatchItemView<'_>) -> Self {
        Self {
            operation: item.operation().unwrap_or_default(),
            unique_batch_item_id: item.with_unique_batch_item_id(<[u8]>::to_vec),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ResponseAssociationError {
    ItemCountMismatch,
    ItemOperationMismatch,
    MissingOrUnexpectedId,
    DuplicateId,
    UnknownId,
}

pub(super) fn associate_batch_items(
    requests: &[BatchIdentity],
    responses: &[BatchIdentity],
) -> Result<Vec<usize>, ResponseAssociationError> {
    if requests.len() != responses.len() {
        return Err(ResponseAssociationError::ItemCountMismatch);
    }
    if requests.len() > 1
        && requests
            .iter()
            .any(|item| item.unique_batch_item_id.is_none())
    {
        return Err(ResponseAssociationError::MissingOrUnexpectedId);
    }
    let mut response_ids = HashSet::new();
    for response in responses {
        if let Some(id) = &response.unique_batch_item_id
            && !response_ids.insert(id.as_slice())
        {
            return Err(ResponseAssociationError::DuplicateId);
        }
    }

    let mut associated = Vec::with_capacity(requests.len());
    let mut consumed = vec![false; responses.len()];
    for request in requests {
        let index = match &request.unique_batch_item_id {
            Some(id) => responses
                .iter()
                .position(|response| response.unique_batch_item_id.as_deref() == Some(id)),
            None => {
                (responses.len() == 1 && responses[0].unique_batch_item_id.is_none()).then_some(0)
            }
        }
        .ok_or(ResponseAssociationError::UnknownId)?;
        if consumed[index] {
            return Err(ResponseAssociationError::DuplicateId);
        }
        if responses[index].operation != request.operation {
            return Err(ResponseAssociationError::ItemOperationMismatch);
        }
        consumed[index] = true;
        associated.push(index);
    }
    if consumed.iter().any(|matched| !matched) {
        return Err(ResponseAssociationError::UnknownId);
    }
    Ok(associated)
}

#[derive(Clone, Copy)]
pub(super) struct PendingState {
    pub(super) pending: bool,
    pub(super) has_correlation_value: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum OutcomeValidationError {
    PendingNotPermitted,
    PendingCorrelationMissing,
    UnknownCriticalExtension,
}

impl From<ResponseAssociationError> for ProtocolError {
    fn from(_error: ResponseAssociationError) -> Self {
        protocol_error(ProtocolErrorKind::InvalidValue)
    }
}

impl From<OutcomeValidationError> for ProtocolError {
    fn from(error: OutcomeValidationError) -> Self {
        let kind = match error {
            OutcomeValidationError::PendingNotPermitted
            | OutcomeValidationError::PendingCorrelationMissing => ProtocolErrorKind::InvalidValue,
            OutcomeValidationError::UnknownCriticalExtension => ProtocolErrorKind::UnsupportedValue,
        };
        protocol_error(kind)
    }
}

pub(super) fn validate_pending_states(
    asynchronous_indicator: Option<u32>,
    outcomes: &[PendingState],
) -> Result<(), OutcomeValidationError> {
    let permits_pending = matches!(
        asynchronous_indicator,
        Some(ASYNCHRONOUS_MANDATORY | ASYNCHRONOUS_OPTIONAL)
    );
    for outcome in outcomes {
        if outcome.pending && !permits_pending {
            return Err(OutcomeValidationError::PendingNotPermitted);
        }
        if outcome.pending && !outcome.has_correlation_value {
            return Err(OutcomeValidationError::PendingCorrelationMissing);
        }
    }
    Ok(())
}

pub(super) fn validate_unknown_extension(critical: bool) -> Result<(), OutcomeValidationError> {
    if critical {
        Err(OutcomeValidationError::UnknownCriticalExtension)
    } else {
        Ok(())
    }
}

// Consuming the validated response makes this the single lifetime boundary
// for the owned, decoded response tree on both success and error paths.
#[allow(clippy::needless_pass_by_value)]
fn validate_response(
    request_identities: &[BatchIdentity],
    options: &ValidatedBatchOptions,
    response: ResponseMessage,
    registry: &ClientExtensionRegistry,
    limits: &CodecLimits,
    #[cfg(test)] pending_owner_observer: Option<&ZeroizationObserver>,
) -> Result<ClientBatchResponse, ProtocolError> {
    if !protocol_version_is_supported(response.header().protocol_version()) {
        return Err(protocol_error(ProtocolErrorKind::UnsupportedValue));
    }

    let response_items = response.batch_items().collect::<Vec<_>>();
    let response_identities = response_items
        .iter()
        .map(BatchIdentity::from_response)
        .collect::<Vec<_>>();
    let association = associate_batch_items(request_identities, &response_identities)?;

    let outcome_states = response_items
        .iter()
        .map(|item| PendingState {
            pending: item.result_status() == Some(ResultStatus::from_raw(RESULT_STATUS_PENDING)),
            has_correlation_value: item.with_asynchronous_correlation_value(|_| ()).is_some(),
        })
        .collect::<Vec<_>>();
    validate_pending_states(options.asynchronous_indicator, &outcome_states)?;

    let mut ordered = Vec::with_capacity(request_identities.len());
    for (request_index, response_index) in association.into_iter().enumerate() {
        let item = response_items[response_index];
        let extensions = preserve_response_extensions(item, registry, limits)?;
        let outcome = response_outcome(request_identities[request_index].operation, item)?;
        #[cfg(test)]
        if let (Some(observer), ClientBatchOutcome::Pending(pending)) =
            (pending_owner_observer, &outcome)
        {
            observer.expect_initialized_len(pending.asynchronous_correlation_value.len());
        }
        ordered.push(ClientBatchItemResponse {
            unique_batch_item_id: request_identities[request_index]
                .unique_batch_item_id
                .clone(),
            outcome,
            extensions,
            #[cfg(test)]
            pending_owner_observer: pending_owner_observer.cloned(),
        });
    }
    Ok(ClientBatchResponse { items: ordered })
}

fn response_outcome(
    operation: u32,
    item: ResponseBatchItemView<'_>,
) -> Result<ClientBatchOutcome, ProtocolError> {
    match operation {
        DISCOVER_VERSIONS_OPERATION => {
            let response = DiscoverVersionsResponse::try_from_response_item(item)
                .map_err(invalid_typed_response)?;
            operation_outcome(
                ClientOperation::DiscoverVersions,
                response,
                item,
                DiscoverVersionsResponse::result,
                PendingResponse::DiscoverVersions,
                ClientBatchOutcome::Completed,
            )
        }
        CREATE_OPERATION | CREATE_KEY_PAIR_OPERATION | CREATE_SPLIT_KEY_OPERATION => {
            creation_response_outcome(operation, item)
        }
        ADD_ATTRIBUTE_OPERATION
        | ADJUST_ATTRIBUTE_OPERATION
        | DELETE_ATTRIBUTE_OPERATION
        | MODIFY_ATTRIBUTE_OPERATION
        | SET_ATTRIBUTE_OPERATION => attribute_mutation_response_outcome(operation, item),
        GET_ATTRIBUTES_OPERATION => read_operation_outcome(
            ClientOperation::GetAttributes,
            item,
            GetAttributesResponse::try_from_response_item,
            GetAttributesResponse::result,
            PendingResponse::GetAttributes,
            ClientBatchOutcome::GetAttributes,
        ),
        GET_ATTRIBUTE_LIST_OPERATION => read_operation_outcome(
            ClientOperation::GetAttributeList,
            item,
            GetAttributeListResponse::try_from_response_item,
            GetAttributeListResponse::result,
            PendingResponse::GetAttributeList,
            ClientBatchOutcome::GetAttributeList,
        ),
        _ => Err(protocol_error(ProtocolErrorKind::UnsupportedValue)),
    }
}

fn creation_response_outcome(
    operation: u32,
    item: ResponseBatchItemView<'_>,
) -> Result<ClientBatchOutcome, ProtocolError> {
    match operation {
        CREATE_OPERATION => {
            let response =
                CreateResponse::try_from_response_item(item).map_err(invalid_typed_response)?;
            operation_outcome(
                ClientOperation::Create,
                response,
                item,
                CreateResponse::result,
                PendingResponse::Create,
                ClientBatchOutcome::CreateCompleted,
            )
        }
        CREATE_KEY_PAIR_OPERATION => {
            let response = CreateKeyPairResponse::try_from_response_item(item)
                .map_err(invalid_typed_response)?;
            operation_outcome(
                ClientOperation::CreateKeyPair,
                response,
                item,
                CreateKeyPairResponse::result,
                PendingResponse::CreateKeyPair,
                ClientBatchOutcome::CreateKeyPairCompleted,
            )
        }
        CREATE_SPLIT_KEY_OPERATION => {
            let response = CreateSplitKeyResponse::try_from_response_item(item)
                .map_err(invalid_typed_response)?;
            operation_outcome(
                ClientOperation::CreateSplitKey,
                response,
                item,
                CreateSplitKeyResponse::result,
                PendingResponse::CreateSplitKey,
                ClientBatchOutcome::CreateSplitKeyCompleted,
            )
        }
        _ => Err(protocol_error(ProtocolErrorKind::UnsupportedValue)),
    }
}

fn attribute_mutation_response_outcome(
    operation: u32,
    item: ResponseBatchItemView<'_>,
) -> Result<ClientBatchOutcome, ProtocolError> {
    match operation {
        ADD_ATTRIBUTE_OPERATION => {
            let response = AddAttributeResponse::try_from_response_item(item)
                .map_err(invalid_typed_response)?;
            operation_outcome(
                ClientOperation::AddAttribute,
                response,
                item,
                AddAttributeResponse::result,
                PendingResponse::AddAttribute,
                ClientBatchOutcome::AddAttribute,
            )
        }
        ADJUST_ATTRIBUTE_OPERATION => {
            let response = AdjustAttributeResponse::try_from_response_item(item)
                .map_err(invalid_typed_response)?;
            operation_outcome(
                ClientOperation::AdjustAttribute,
                response,
                item,
                AdjustAttributeResponse::result,
                PendingResponse::AdjustAttribute,
                ClientBatchOutcome::AdjustAttribute,
            )
        }
        DELETE_ATTRIBUTE_OPERATION => {
            let response = DeleteAttributeResponse::try_from_response_item(item)
                .map_err(invalid_typed_response)?;
            operation_outcome(
                ClientOperation::DeleteAttribute,
                response,
                item,
                DeleteAttributeResponse::result,
                PendingResponse::DeleteAttribute,
                ClientBatchOutcome::DeleteAttribute,
            )
        }
        MODIFY_ATTRIBUTE_OPERATION => {
            let response = ModifyAttributeResponse::try_from_response_item(item)
                .map_err(invalid_typed_response)?;
            operation_outcome(
                ClientOperation::ModifyAttribute,
                response,
                item,
                ModifyAttributeResponse::result,
                PendingResponse::ModifyAttribute,
                ClientBatchOutcome::ModifyAttribute,
            )
        }
        SET_ATTRIBUTE_OPERATION => {
            let response = SetAttributeResponse::try_from_response_item(item)
                .map_err(invalid_typed_response)?;
            operation_outcome(
                ClientOperation::SetAttribute,
                response,
                item,
                SetAttributeResponse::result,
                PendingResponse::SetAttribute,
                ClientBatchOutcome::SetAttribute,
            )
        }
        _ => Err(protocol_error(ProtocolErrorKind::UnsupportedValue)),
    }
}

fn invalid_typed_response<E>(error: E) -> ProtocolError
where
    E: Error + 'static,
{
    ProtocolError::new(
        ProtocolErrorKind::InvalidValue,
        ProtocolCauseCategory::InvalidValue,
        error,
    )
}

fn pending_outcome(
    operation: ClientOperation,
    result: KmipOperationResult,
    response: PendingResponse,
    item: ResponseBatchItemView<'_>,
) -> Result<PendingOutcome, ProtocolError> {
    let asynchronous_correlation_value = item
        .with_asynchronous_correlation_value(|bytes| Zeroizing::new(bytes.to_vec()))
        .ok_or_else(|| protocol_error(ProtocolErrorKind::InvalidValue))?;
    Ok(PendingOutcome {
        operation,
        result,
        response,
        asynchronous_correlation_value,
    })
}

fn operation_outcome<T>(
    operation: ClientOperation,
    response: T,
    item: ResponseBatchItemView<'_>,
    result: impl FnOnce(&T) -> &KmipOperationResult,
    into_pending: impl FnOnce(T) -> PendingResponse,
    into_completed: impl FnOnce(T) -> ClientBatchOutcome,
) -> Result<ClientBatchOutcome, ProtocolError> {
    let result = result(&response).clone();
    if result.status().raw() == RESULT_STATUS_PENDING {
        pending_outcome(operation, result, into_pending(response), item)
            .map(ClientBatchOutcome::Pending)
    } else {
        Ok(into_completed(response))
    }
}

fn validate_follow_up_indicator(raw: Option<u32>) -> Result<(), ClientError> {
    validate_asynchronous_indicator(raw)
        .map(|_| ())
        .map_err(|()| {
            ClientError::validation(
                ClientCauseCategory::InvalidInput,
                RequestDeliveryState::NotSent,
                BatchValidationError::InvalidAsynchronousIndicator,
            )
        })
}

#[allow(clippy::needless_pass_by_value)]
fn validate_async_response(
    response: ResponseMessage,
    operation: u32,
    kind: ClientOperation,
    asynchronous_indicator: Option<u32>,
    expected_cancel_correlation: Option<&[u8]>,
    registry: &ClientExtensionRegistry,
    limits: &CodecLimits,
) -> Result<ClientOperationOutcome, ProtocolError> {
    if !protocol_version_is_supported(response.header().protocol_version()) {
        return Err(protocol_error(ProtocolErrorKind::UnsupportedValue));
    }

    let response_items = response.batch_items().collect::<Vec<_>>();
    let requests = [BatchIdentity {
        operation,
        unique_batch_item_id: None,
    }];
    let response_identities = response_items
        .iter()
        .map(BatchIdentity::from_response)
        .collect::<Vec<_>>();
    let association = associate_batch_items(&requests, &response_identities)?;
    let response_index = association
        .first()
        .copied()
        .ok_or_else(|| protocol_error(ProtocolErrorKind::MalformedMessage))?;
    let item = response_items
        .get(response_index)
        .copied()
        .ok_or_else(|| protocol_error(ProtocolErrorKind::MalformedMessage))?;

    let extensions = preserve_response_extensions(item, registry, limits)?;

    let (result, cancellation_result) = match kind {
        ClientOperation::DiscoverVersions
        | ClientOperation::Create
        | ClientOperation::CreateKeyPair
        | ClientOperation::CreateSplitKey
        | ClientOperation::AddAttribute
        | ClientOperation::AdjustAttribute
        | ClientOperation::DeleteAttribute
        | ClientOperation::ModifyAttribute
        | ClientOperation::SetAttribute
        | ClientOperation::GetAttributes
        | ClientOperation::GetAttributeList => {
            return Err(protocol_error(ProtocolErrorKind::UnsupportedValue));
        }
        ClientOperation::Poll => {
            let typed =
                PollResponse::try_from_response_item(item).map_err(asynchronous_operation_error)?;
            (typed.result().clone(), None)
        }
        ClientOperation::Cancel => {
            let typed = CancelResponse::try_from_response_item(item)
                .map_err(asynchronous_operation_error)?;
            if typed.result().status().raw() == 0 {
                let expected = expected_cancel_correlation
                    .ok_or_else(|| protocol_error(ProtocolErrorKind::InvalidValue))?;
                let echoes_request = typed
                    .with_asynchronous_correlation_value(|echo| echo == expected)
                    .unwrap_or(false);
                if !echoes_request {
                    return Err(protocol_error(ProtocolErrorKind::InvalidValue));
                }
            }
            (typed.result().clone(), typed.cancellation_result())
        }
        ClientOperation::Process => {
            let typed = ProcessResponse::try_from_response_item(item)
                .map_err(asynchronous_operation_error)?;
            (typed.result().clone(), None)
        }
        ClientOperation::QueryAsyncRequests => {
            let typed = QueryAsyncRequestsResponse::try_from_response_item(item)
                .map_err(asynchronous_operation_error)?;
            (typed.result().clone(), None)
        }
    };

    // Poll Pending reports the original operation's state and is explicitly
    // permitted by §6.1.38 without making Poll itself asynchronous. Cancel's
    // typed model rejects Pending. Process and Query use the caller's indicator.
    if matches!(
        kind,
        ClientOperation::Process | ClientOperation::QueryAsyncRequests
    ) {
        validate_pending_states(
            asynchronous_indicator,
            &[PendingState {
                pending: result.status().raw() == RESULT_STATUS_PENDING,
                has_correlation_value: item.with_asynchronous_correlation_value(|_| ()).is_some(),
            }],
        )
        .map_err(ProtocolError::from)?;
    }

    Ok(ClientOperationOutcome {
        operation: kind,
        result,
        response,
        cancellation_result,
        extensions,
    })
}

fn preserve_response_extensions(
    item: ResponseBatchItemView<'_>,
    registry: &ClientExtensionRegistry,
    limits: &CodecLimits,
) -> Result<Vec<ClientMessageExtension>, ProtocolError> {
    let mut extensions = Vec::new();
    for index in 0..item.message_extension_count() {
        let extension = item
            .message_extension(index)
            .ok_or_else(|| protocol_error(ProtocolErrorKind::MalformedMessage))?;
        let critical = extension
            .criticality_indicator()
            .ok_or_else(|| protocol_error(ProtocolErrorKind::MalformedMessage))?;
        let recognized = inspect_response_extension(extension, registry, limits)?;
        validate_unknown_extension(critical && !recognized).map_err(ProtocolError::from)?;
        let structure = extension
            .with_ttlv(|view| copy_structure(&view))
            .ok_or_else(|| protocol_error(ProtocolErrorKind::MalformedMessage))??;
        extensions.push(ClientMessageExtension { structure });
    }
    Ok(extensions)
}

fn inspect_response_extension(
    extension: MessageExtensionView<'_>,
    registry: &ClientExtensionRegistry,
    limits: &CodecLimits,
) -> Result<bool, ProtocolError> {
    let vendor = extension
        .with_vendor_identification(str::to_owned)
        .ok_or_else(|| protocol_error(ProtocolErrorKind::MalformedMessage))?;
    let payload = extension
        .with_vendor_extension(|view| copy_structure(&view))
        .ok_or_else(|| protocol_error(ProtocolErrorKind::MalformedMessage))??;
    let recognition = extension_registry::inspect(registry, &vendor, payload, limits).map_err(
        |error| match error {
            ClientError::Protocol { error, .. } => error,
            _ => protocol_error(ProtocolErrorKind::ResourceLimit),
        },
    )?;
    Ok(extension_registry::is_recognized(&recognition))
}

fn asynchronous_operation_error(error: AsynchronousOperationError) -> ProtocolError {
    ProtocolError::new(
        ProtocolErrorKind::InvalidValue,
        ProtocolCauseCategory::InvalidValue,
        error,
    )
}

fn protocol_error(kind: ProtocolErrorKind) -> ProtocolError {
    ProtocolError::new(
        kind,
        ProtocolCauseCategory::InvalidValue,
        ResponseValidationFailure,
    )
}

#[derive(Debug)]
struct ResponseValidationFailure;

impl fmt::Display for ResponseValidationFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("response validation failed")
    }
}

impl Error for ResponseValidationFailure {}

#[derive(Debug)]
struct ResponseLimitExceeded;

impl fmt::Display for ResponseLimitExceeded {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("response exceeds the configured byte limit")
    }
}

impl Error for ResponseLimitExceeded {}

#[derive(Debug)]
pub(super) enum BoundedResponseError<E> {
    TooLarge,
    Decode(E),
}

pub(super) fn decode_bounded_response<T, E>(
    bytes: &[u8],
    limits: &CodecLimits,
    decoder: impl FnOnce(&[u8], &CodecLimits) -> Result<T, E>,
    #[cfg(test)] observer: Option<&LimitsIdentityObserver>,
) -> Result<T, BoundedResponseError<E>> {
    if bytes.len() > limits.max_message_bytes() {
        return Err(BoundedResponseError::TooLarge);
    }
    #[cfg(test)]
    if let Some(observer) = observer {
        observer.record_decode(limits);
    }
    decoder(bytes, limits).map_err(BoundedResponseError::Decode)
}

fn decode_transport_response(
    response: kmipkit_transport::TransportResponse,
    limits: &CodecLimits,
    #[cfg(test)] observer: Option<&LimitsIdentityObserver>,
) -> Result<(ResponseMessage, RequestDeliveryState), ClientError> {
    let response_delivery_state =
        RequestDeliveryState::PossiblySent.response_bytes_received(response.as_bytes().len());
    let decode_result = decode_bounded_response(
        response.as_bytes(),
        limits,
        decode_response_message,
        #[cfg(test)]
        observer,
    );
    drop(response);

    let response_message = decode_result.map_err(|error| {
        let protocol = match error {
            BoundedResponseError::TooLarge => ProtocolError::new(
                ProtocolErrorKind::MalformedMessage,
                ProtocolCauseCategory::InvalidEncoding,
                ResponseLimitExceeded,
            ),
            BoundedResponseError::Decode(error) => error,
        };
        protocol_failure_at(protocol, response_delivery_state)
    })?;
    Ok((response_message, response_delivery_state))
}

fn protocol_failure_at(error: ProtocolError, delivery_state: RequestDeliveryState) -> ClientError {
    ClientError::protocol(error, delivery_state)
}

fn decode_response_message(
    bytes: &[u8],
    limits: &CodecLimits,
) -> Result<ResponseMessage, ProtocolError> {
    let decoded = decode_with_limits(bytes, limits).map_err(|error: DecodeError| {
        ProtocolError::new(
            ProtocolErrorKind::MalformedMessage,
            ProtocolCauseCategory::InvalidEncoding,
            error,
        )
    })?;
    let tree = decoded
        .with_value(|value| match value {
            ValueView::Structure(structure) => Some(copy_structure(&structure)),
            _ => None,
        })
        .ok_or_else(|| protocol_error(ProtocolErrorKind::MalformedMessage))??;
    ResponseMessage::try_from_ttlv(tree).map_err(|error| {
        ProtocolError::new(
            ProtocolErrorKind::MalformedMessage,
            ProtocolCauseCategory::InvalidValue,
            error,
        )
    })
}

fn copy_structure(view: &StructureView<'_>) -> Result<Structure, ProtocolError> {
    let mut structure = Structure::new();
    for child in view.children() {
        let value = child.with_value(copy_value)?;
        let item = Item::new(child.tag(), value).map_err(model_protocol_error)?;
        structure.try_push(item).map_err(model_protocol_error)?;
    }
    Ok(structure)
}

fn copy_value(view: ValueView<'_>) -> Result<Value, ProtocolError> {
    match view {
        ValueView::Structure(value) => Ok(Value::structure(copy_structure(&value)?)),
        ValueView::Integer(value) => Ok(Value::integer(*value)),
        ValueView::LongInteger(value) => Ok(Value::long_integer(*value)),
        ValueView::BigInteger(value) => Ok(Value::big_integer(value.to_vec())),
        ValueView::Enumeration(value) => Ok(Value::enumeration(*value)),
        ValueView::Boolean(value) => Ok(Value::boolean(*value)),
        ValueView::TextString(value) => Ok(Value::text_string((*value).to_owned())),
        ValueView::ByteString(value) => Ok(Value::byte_string(value.to_vec())),
        ValueView::DateTime(value) => Ok(Value::date_time(*value)),
        ValueView::Interval(value) => Ok(Value::interval(*value)),
        ValueView::DateTimeExtended(value) => Ok(Value::date_time_extended(*value)),
        _ => Err(protocol_error(ProtocolErrorKind::UnsupportedValue)),
    }
}

#[cfg(test)]
pub(super) fn decode_request_message_for_test(
    request: ClientBatch,
    limits: &CodecLimits,
) -> Result<RequestMessage, ProtocolError> {
    let options = validate_batch_options(&request).map_err(|error| {
        ProtocolError::new(
            ProtocolErrorKind::InvalidValue,
            ProtocolCauseCategory::InvalidValue,
            error,
        )
    })?;
    let maximum_response_size = maximum_response_size_for_batch(&request, limits);
    build_request_message(request, &options, maximum_response_size)
}

#[cfg(test)]
pub(super) use decode_request_message_for_test as request_message_for_test;

#[cfg(test)]
pub(super) use protocol_version_is_supported as version_is_supported;

#[cfg(test)]
pub(super) use validate_asynchronous_indicator as validate_async_indicator_for_test;

#[cfg(test)]
pub(super) use validate_batch_error_continuation as validate_batch_error_continuation_for_test;

#[cfg(test)]
pub(super) use validate_pending_states as validate_pending_states_for_test;

#[cfg(test)]
pub(super) use validate_unknown_extension as validate_unknown_extension_for_test;

#[cfg(test)]
#[path = "../tests/unit/execute_private_error_tests.rs"]
mod private_error_tests;

#[cfg(test)]
#[path = "../tests/unit/execute_provenance_order_tests.rs"]
mod provenance_order_tests;
