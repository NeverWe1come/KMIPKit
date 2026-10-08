//! Closed typed request execution for the synchronous KMIP client.

use std::collections::HashSet;
use std::error::Error;
use std::fmt;

use kmipkit_protocol::extension::ExtensionIdentity;
use kmipkit_protocol::{
    AsynchronousOperationError, CancelRequest, CancelResponse, CancellationResult,
    DiscoverVersionsRequest, DiscoverVersionsResponse, MessageExtensionView, PollRequest,
    PollResponse, ProcessRequest, ProcessResponse, ProtocolCauseCategory, ProtocolError,
    ProtocolErrorKind, ProtocolVersion, QueryAsyncRequestsRequest, QueryAsyncRequestsResponse,
    RequestMessage, ResponseBatchItemView, ResponseMessage, ResultStatus,
};
use kmipkit_transport::{RequestDeliveryState, Transport};
use kmipkit_ttlv::codec::{CodecLimits, DecodeError, decode_with_limits};
use kmipkit_ttlv::{Item, ModelError, RawTag, Structure, StructureView, Tag, Value, ValueView};
#[cfg(test)]
use zeroize::Zeroize;
use zeroize::Zeroizing;

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
/// The initial feature supports only an explicitly requested Discover Versions
/// operation. Generic TTLV items and caller-provided wire bytes are not accepted.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClientRequest {
    /// An explicit client-to-server Discover Versions request.
    DiscoverVersions(DiscoverVersionsRequest),
}

impl ClientRequest {
    /// Creates the KMIP 2.1 Discover Versions request variant.
    #[must_use]
    pub const fn discover_versions() -> Self {
        Self::DiscoverVersions(DiscoverVersionsRequest::new())
    }

    const fn operation(self) -> u32 {
        match self {
            Self::DiscoverVersions(_) => DISCOVER_VERSIONS_OPERATION,
        }
    }

    fn payload(self) -> Result<Structure, ProtocolError> {
        match self {
            Self::DiscoverVersions(request) => request.to_ttlv_payload(),
        }
    }
}

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
    pub const fn request(&self) -> ClientRequest {
        self.request
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

/// A Pending Discover Versions result and its opaque correlation capability.
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
    response: DiscoverVersionsResponse,
    asynchronous_correlation_value: Zeroizing<Vec<u8>>,
}

impl PendingOutcome {
    /// Returns the typed Discover Versions result metadata.
    #[must_use]
    pub const fn response(&self) -> &DiscoverVersionsResponse {
        &self.response
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
            .field("response", &self.response)
            .field("asynchronous_correlation_value", &"[REDACTED]")
            .finish()
    }
}

/// Identifies a client-initiated asynchronous operation handled by one
/// `Client::execute_*` call.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClientOperation {
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

/// A completed operation response or an explicitly resumable Pending result.
#[non_exhaustive]
pub enum ClientBatchOutcome {
    /// The server returned a non-Pending KMIP result.
    Completed(DiscoverVersionsResponse),
    /// The server returned Pending with its required capability-like value.
    Pending(PendingOutcome),
}

impl ClientBatchOutcome {
    /// Lends the Pending Asynchronous Correlation Value when present.
    #[must_use]
    pub fn asynchronous_correlation_value(&self) -> Option<&[u8]> {
        match self {
            Self::Completed(_) => None,
            Self::Pending(pending) => Some(pending.asynchronous_correlation_value()),
        }
    }

    /// Returns the typed Discover Versions operation result.
    #[must_use]
    pub const fn response(&self) -> &DiscoverVersionsResponse {
        match self {
            Self::Completed(response) => response,
            Self::Pending(pending) => pending.response(),
        }
    }
}

impl fmt::Debug for ClientBatchOutcome {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Completed(response) => {
                formatter.debug_tuple("Completed").field(response).finish()
            }
            Self::Pending(pending) => formatter.debug_tuple("Pending").field(pending).finish(),
        }
    }
}

impl fmt::Display for ClientBatchOutcome {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Completed(response) => write!(formatter, "Completed({})", response.result()),
            Self::Pending(pending) => write!(formatter, "Pending({})", pending.response.result()),
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
/// The only admitted operation in this feature is an explicit Discover
/// Versions request. This feature intentionally defines no production
/// constructor or live network backend. A separately approved
/// transport-configuration feature supplies construction from validated
/// configuration without accepting arbitrary caller-implemented transports.
///
/// See `docs/user-guide/en/client-execution.md` in the repository for current
/// scope, limits, redaction, and transport boundaries.
pub struct Client {
    transport: Box<dyn Transport>,
    configuration: ClientConfiguration,
    #[cfg(test)]
    request_owner_observer: Option<private_wire_writer::ZeroizationObserver>,
    #[cfg(test)]
    pending_owner_observer: Option<private_wire_writer::ZeroizationObserver>,
    #[cfg(test)]
    limits_identity_observer: Option<LimitsIdentityObserver>,
}

impl Client {
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

        let request_message = build_request_message(&batch, &options)
            .map_err(|error| protocol_failure_at(error, RequestDeliveryState::NotSent))?;
        let (response_message, response_delivery_state) =
            self.exchange_operation(request_message, limits)?;

        validate_response(
            &batch,
            &options,
            response_message,
            self.configuration.extension_registry(),
            limits,
            #[cfg(test)]
            self.pending_owner_observer.as_ref(),
        )
        .map_err(|error| protocol_failure_at(error, response_delivery_state))
    }

    fn exchange_operation(
        &mut self,
        request_message: RequestMessage,
        limits: &CodecLimits,
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

        let transport_result = self
            .transport
            .exchange(encoded.as_bytes(), limits.max_message_bytes());
        drop(encoded);

        let response = transport_result.map_err(ClientError::transport)?;
        let response_delivery_state =
            RequestDeliveryState::PossiblySent.response_bytes_received(response.as_bytes().len());
        let response_message = decode_bounded_response(
            response.as_bytes(),
            limits,
            decode_response_message,
            #[cfg(test)]
            self.limits_identity_observer.as_ref(),
        )
        .map_err(|error| {
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
        drop(response);
        Ok((response_message, response_delivery_state))
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
        )
    }

    fn execute_async_request(
        &mut self,
        operation: u32,
        payload: Structure,
        kind: ClientOperation,
        asynchronous_indicator: Option<u32>,
        expected_cancel_correlation: Option<&[u8]>,
        limits: &CodecLimits,
    ) -> Result<ClientOperationOutcome, ClientError> {
        let request_message =
            build_async_request_message(operation, payload, asynchronous_indicator)
                .map_err(|error| protocol_failure_at(error, RequestDeliveryState::NotSent))?;
        let (response, delivery_state) = self.exchange_operation(request_message, limits)?;
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
            transport: Box::new(transport),
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
            transport: Box::new(transport),
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
            transport: Box::new(transport),
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
            transport: Box::new(transport),
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
            transport: Box::new(transport),
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
    batch: &ClientBatch,
    options: &ValidatedBatchOptions,
) -> Result<RequestMessage, ProtocolError> {
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

    let mut tree = Structure::new();
    push(&mut tree, REQUEST_HEADER, Value::structure(header))?;
    for item in &batch.items {
        let mut batch_item = Structure::new();
        push(
            &mut batch_item,
            OPERATION,
            Value::enumeration(item.request.operation()),
        )?;
        if let Some(id) = &item.unique_batch_item_id {
            push(
                &mut batch_item,
                UNIQUE_BATCH_ITEM_ID,
                Value::byte_string(id.clone()),
            )?;
        }
        push(
            &mut batch_item,
            REQUEST_PAYLOAD,
            Value::structure(item.request.payload()?),
        )?;
        for extension in &item.message_extensions {
            push(
                &mut batch_item,
                MESSAGE_EXTENSION,
                Value::structure(message_extension_structure(extension)?),
            )?;
        }
        push(&mut tree, BATCH_ITEM, Value::structure(batch_item))?;
    }

    RequestMessage::try_from_ttlv(tree).map_err(|error| {
        ProtocolError::new(
            ProtocolErrorKind::MalformedMessage,
            ProtocolCauseCategory::InvalidValue,
            error,
        )
    })
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
    request: &ClientBatch,
    options: &ValidatedBatchOptions,
    response: ResponseMessage,
    registry: &ClientExtensionRegistry,
    limits: &CodecLimits,
    #[cfg(test)] pending_owner_observer: Option<&ZeroizationObserver>,
) -> Result<ClientBatchResponse, ProtocolError> {
    if !protocol_version_is_supported(response.header().protocol_version()) {
        return Err(protocol_error(ProtocolErrorKind::UnsupportedValue));
    }

    let request_identities = request
        .items
        .iter()
        .map(BatchIdentity::from_request)
        .collect::<Vec<_>>();
    let response_items = response.batch_items().collect::<Vec<_>>();
    let response_identities = response_items
        .iter()
        .map(BatchIdentity::from_response)
        .collect::<Vec<_>>();
    let association = associate_batch_items(&request_identities, &response_identities)?;

    let outcome_states = response_items
        .iter()
        .map(|item| PendingState {
            pending: item.result_status() == Some(ResultStatus::from_raw(RESULT_STATUS_PENDING)),
            has_correlation_value: item.with_asynchronous_correlation_value(|_| ()).is_some(),
        })
        .collect::<Vec<_>>();
    validate_pending_states(options.asynchronous_indicator, &outcome_states)?;

    let mut ordered = Vec::with_capacity(request.items.len());
    for (request_index, response_index) in association.into_iter().enumerate() {
        let item = response_items[response_index];
        let extensions = preserve_response_extensions(item, registry, limits)?;

        let typed = DiscoverVersionsResponse::try_from_response_item(item).map_err(|error| {
            ProtocolError::new(
                ProtocolErrorKind::InvalidValue,
                ProtocolCauseCategory::InvalidValue,
                error,
            )
        })?;
        let outcome = if typed.result().status().raw() == RESULT_STATUS_PENDING {
            let correlation_value = item
                .with_asynchronous_correlation_value(|bytes| Zeroizing::new(bytes.to_vec()))
                .ok_or_else(|| protocol_error(ProtocolErrorKind::InvalidValue))?;
            ClientBatchOutcome::Pending(PendingOutcome {
                response: typed,
                asynchronous_correlation_value: correlation_value,
            })
        } else {
            ClientBatchOutcome::Completed(typed)
        };
        #[cfg(test)]
        if let (Some(observer), ClientBatchOutcome::Pending(pending)) =
            (pending_owner_observer, &outcome)
        {
            observer.expect_initialized_len(pending.asynchronous_correlation_value.len());
        }
        ordered.push(ClientBatchItemResponse {
            unique_batch_item_id: request.items[request_index].unique_batch_item_id.clone(),
            outcome,
            extensions,
            #[cfg(test)]
            pending_owner_observer: pending_owner_observer.cloned(),
        });
    }
    Ok(ClientBatchResponse { items: ordered })
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
    request: &ClientBatch,
    _limits: &CodecLimits,
) -> Result<RequestMessage, ProtocolError> {
    let options = validate_batch_options(request).map_err(|error| {
        ProtocolError::new(
            ProtocolErrorKind::InvalidValue,
            ProtocolCauseCategory::InvalidValue,
            error,
        )
    })?;
    build_request_message(request, &options)
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
mod provenance_order_tests {
    use syn::visit::{self, Visit};
    use syn::{Expr, ExprCall, ExprMethodCall, ImplItem, Item, ItemImpl, Stmt};

    #[test]
    fn registry_provenance_precedes_request_build_codec_and_adapter_handoff() {
        let source = include_str!("execute.rs");
        let syntax = syn::parse_file(source).expect("client execution source parses");
        let client_impl = syntax
            .items
            .iter()
            .find_map(|item| match item {
                Item::Impl(item_impl) if is_client_impl(item_impl) => Some(item_impl),
                _ => None,
            })
            .expect("Client implementation exists");
        let execute = method(client_impl, "execute");
        let provenance = call_position(
            execute.block.stmts.as_slice(),
            "validate_request_extension_ownership",
        );
        let request_build = call_position(execute.block.stmts.as_slice(), "build_request_message");
        let exchange = call_position(execute.block.stmts.as_slice(), "exchange_operation");
        assert!(provenance < request_build);
        assert!(request_build < exchange);

        let exchange_operation = method(client_impl, "exchange_operation");
        let encode = call_position(
            exchange_operation.block.stmts.as_slice(),
            "encode_for_execute",
        );
        let adapter = call_position(exchange_operation.block.stmts.as_slice(), "exchange");
        assert!(encode < adapter);
    }

    fn is_client_impl(item_impl: &ItemImpl) -> bool {
        matches!(item_impl.self_ty.as_ref(),
            syn::Type::Path(type_path) if type_path.path.is_ident("Client"))
    }

    fn method<'a>(item_impl: &'a ItemImpl, name: &str) -> &'a syn::ImplItemFn {
        item_impl
            .items
            .iter()
            .find_map(|item| match item {
                ImplItem::Fn(function) if function.sig.ident == name => Some(function),
                _ => None,
            })
            .expect("expected Client method exists")
    }

    fn call_position(statements: &[Stmt], name: &str) -> usize {
        statements
            .iter()
            .position(|statement| {
                let mut calls = CallNames::default();
                calls.visit_stmt(statement);
                calls.0.iter().any(|called| called == name)
            })
            .expect("expected call appears in method body")
    }

    #[derive(Default)]
    struct CallNames(Vec<String>);

    impl<'ast> Visit<'ast> for CallNames {
        fn visit_expr_call(&mut self, expression: &'ast ExprCall) {
            if let Expr::Path(path) = expression.func.as_ref()
                && let Some(segment) = path.path.segments.last()
            {
                self.0.push(segment.ident.to_string());
            }
            visit::visit_expr_call(self, expression);
        }

        fn visit_expr_method_call(&mut self, expression: &'ast ExprMethodCall) {
            self.0.push(expression.method.to_string());
            visit::visit_expr_method_call(self, expression);
        }
    }
}
