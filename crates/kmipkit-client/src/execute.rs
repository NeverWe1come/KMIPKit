//! Closed typed request execution for the synchronous KMIP client.

use std::collections::HashSet;
use std::error::Error;
use std::fmt;

use kmipkit_protocol::{
    DiscoverVersionsRequest, DiscoverVersionsResponse, ProtocolCauseCategory, ProtocolError,
    ProtocolErrorKind, ProtocolVersion, RequestMessage, ResponseBatchItemView, ResponseMessage,
    ResultStatus,
};
use kmipkit_transport::{RequestDeliveryState, Transport};
use kmipkit_ttlv::codec::{CodecLimits, DecodeError, decode_with_limits};
use kmipkit_ttlv::{Item, ModelError, RawTag, Structure, StructureView, Tag, Value, ValueView};
#[cfg(test)]
use zeroize::Zeroize;
use zeroize::Zeroizing;

use crate::{ClientCauseCategory, ClientError};

#[path = "wire_encoder.rs"]
mod private_wire_writer;

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
const BATCH_ORDER_OPTION: u32 = 0x0042_0010;
const ASYNCHRONOUS_INDICATOR: u32 = 0x0042_0007;
const OPERATION: u32 = 0x0042_005C;
const REQUEST_PAYLOAD: u32 = 0x0042_0079;
const CLIENT_CORRELATION_VALUE: u32 = 0x0042_0105;
const TIME_STAMP: u32 = 0x0042_0092;
const UNIQUE_BATCH_ITEM_ID: u32 = 0x0042_0093;
const DISCOVER_VERSIONS_OPERATION: u32 = 0x0000_001E;
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
}

impl ClientBatchItem {
    /// Creates a batch item from an admitted typed request.
    #[must_use]
    pub const fn new(request: ClientRequest) -> Self {
        Self {
            request,
            unique_batch_item_id: None,
        }
    }

    /// Sets this item's Unique Batch Item ID.
    #[must_use]
    pub fn with_unique_batch_item_id(mut self, id: Vec<u8>) -> Self {
        self.unique_batch_item_id = Some(id);
        self
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

/// An opaque preserved non-critical Message Extension.
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

    /// Returns the preserved non-critical Message Extensions.
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
    #[must_use]
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

        let request_message = build_request_message(&batch, &options)
            .map_err(|error| protocol_failure_at(error, RequestDeliveryState::NotSent))?;
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
            protocol_failure_at(protocol, RequestDeliveryState::ResponseStarted)
        })?;
        drop(response);

        validate_response(
            &batch,
            &options,
            response_message,
            #[cfg(test)]
            self.pending_owner_observer.as_ref(),
        )
        .map_err(|error| protocol_failure_at(error, RequestDeliveryState::ResponseStarted))
    }

    #[cfg(test)]
    pub(super) fn for_test<T: Transport + 'static>(transport: T) -> Self {
        Self {
            transport: Box::new(transport),
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
            request_owner_observer: Some(request_observer),
            pending_owner_observer: Some(pending_observer),
            limits_identity_observer: None,
        }
    }
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
        let mut extensions = Vec::new();
        for index in 0..item.message_extension_count() {
            let extension = item
                .message_extension(index)
                .ok_or_else(|| protocol_error(ProtocolErrorKind::MalformedMessage))?;
            let critical = extension
                .criticality_indicator()
                .ok_or_else(|| protocol_error(ProtocolErrorKind::MalformedMessage))?;
            validate_unknown_extension(critical).map_err(ProtocolError::from)?;
            let structure = extension
                .with_ttlv(|view| copy_structure(&view))
                .ok_or_else(|| protocol_error(ProtocolErrorKind::MalformedMessage))??;
            extensions.push(ClientMessageExtension { structure });
        }

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
