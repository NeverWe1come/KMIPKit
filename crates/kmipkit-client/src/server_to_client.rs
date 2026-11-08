//! Server-to-Client operations and listener dispatcher for `KMIPKit`.
//!
//! Implements OASIS KMIP 2.1 §§6.1.54, 6.2.1–6.2.5 and KMIP Profiles v2.1 §5.3.1.

use std::fmt;
use std::io;
use std::net::SocketAddr;
use std::time::Duration;

use kmipkit_protocol::server_to_client::{
    ServerDiscoverVersionsRequest, ServerDiscoverVersionsResponse, ServerNotifyRequest,
    ServerNotifyResponse, ServerPutRequest, ServerPutResponse, ServerQueryRequest,
    ServerQueryResponse, ServerResponseBuilder,
};
use kmipkit_protocol::{
    KmipOperationResult, ProtocolCauseCategory, ProtocolError, ProtocolErrorKind, ProtocolVersion,
    RequestBatchItemView, RequestMessage, ResponseMessage, ResultReason, ResultStatus,
    SetEndpointRoleRequest, SetEndpointRoleResponse,
};
use kmipkit_transport::listener::{
    HttpsListener, HttpsServerConnection, ListenerTransportConfig, RawTlsListener,
    RawTlsRoleReversedStream, RawTlsServerConnection,
};
use kmipkit_transport::{
    RequestDeliveryState, TransportCauseCategory, TransportError, WireEncoding,
};
use kmipkit_ttlv::codec::{CodecLimits, decode_with_limits};
use kmipkit_ttlv::{Item, StructureView};

use crate::error::ClientError;

const DISCOVER_VERSIONS_OPERATION: u32 = 0x0000_001E;
const NOTIFY_OPERATION: u32 = 0x0000_001B;
const PUT_OPERATION: u32 = 0x0000_001C;
const QUERY_OPERATION: u32 = 0x0000_0018;
const SET_ENDPOINT_ROLE_OPERATION: u32 = 0x0000_0032;
const RESPONSE_MESSAGE_TAG: u32 = 0x0042_007B;

fn make_protocol_error<E: fmt::Display>(source: E) -> ProtocolError {
    ProtocolError::new(
        ProtocolErrorKind::MalformedMessage,
        ProtocolCauseCategory::InvalidValue,
        std::io::Error::other(source.to_string()),
    )
}

/// Parsed Server-to-Client request item.
pub enum ServerRequestItemPayload {
    /// Discover Versions (§6.2.1).
    DiscoverVersions(ServerDiscoverVersionsRequest),
    /// Notify (§6.2.2).
    Notify(ServerNotifyRequest),
    /// Put (§6.2.3).
    Put(ServerPutRequest),
    /// Query (§6.2.4).
    Query(ServerQueryRequest),
    /// Set Endpoint Role (§6.2.5 / §6.1.54).
    SetEndpointRole(SetEndpointRoleRequest),
    /// Unrecognized or raw vendor operation.
    Unknown {
        /// The operation enumeration.
        operation: u32,
    },
}

impl fmt::Debug for ServerRequestItemPayload {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DiscoverVersions(req) => formatter
                .debug_tuple("DiscoverVersions")
                .field(req)
                .finish(),
            Self::Notify(req) => formatter.debug_tuple("Notify").field(req).finish(),
            Self::Put(req) => formatter.debug_tuple("Put").field(req).finish(),
            Self::Query(req) => formatter.debug_tuple("Query").field(req).finish(),
            Self::SetEndpointRole(req) => {
                formatter.debug_tuple("SetEndpointRole").field(req).finish()
            }
            Self::Unknown { operation } => formatter
                .debug_struct("Unknown")
                .field("operation", operation)
                .finish(),
        }
    }
}

/// A borrowed view over one incoming request batch item.
pub struct ServerRequestItemView<'a> {
    operation: u32,
    unique_batch_item_id: Option<Vec<u8>>,
    payload: &'a ServerRequestItemPayload,
}

impl<'a> ServerRequestItemView<'a> {
    /// Returns the KMIP operation enumeration.
    #[must_use]
    pub const fn operation(&self) -> u32 {
        self.operation
    }

    /// Returns the unique batch item ID if provided by the server.
    #[must_use]
    pub fn unique_batch_item_id(&self) -> Option<&[u8]> {
        self.unique_batch_item_id.as_deref()
    }

    /// Returns the parsed Discover Versions request if this item represents it.
    #[must_use]
    pub fn as_discover_versions(&self) -> Option<&'a ServerDiscoverVersionsRequest> {
        match self.payload {
            ServerRequestItemPayload::DiscoverVersions(req) => Some(req),
            _ => None,
        }
    }

    /// Returns the parsed Notify request if this item represents it.
    #[must_use]
    pub fn as_notify(&self) -> Option<&'a ServerNotifyRequest> {
        match self.payload {
            ServerRequestItemPayload::Notify(req) => Some(req),
            _ => None,
        }
    }

    /// Returns the parsed Put request if this item represents it.
    #[must_use]
    pub fn as_put(&self) -> Option<&'a ServerPutRequest> {
        match self.payload {
            ServerRequestItemPayload::Put(req) => Some(req),
            _ => None,
        }
    }

    /// Returns the parsed Query request if this item represents it.
    #[must_use]
    pub fn as_query(&self) -> Option<&'a ServerQueryRequest> {
        match self.payload {
            ServerRequestItemPayload::Query(req) => Some(req),
            _ => None,
        }
    }

    /// Returns the parsed Set Endpoint Role request if this item represents it.
    #[must_use]
    pub fn as_set_endpoint_role(&self) -> Option<&'a SetEndpointRoleRequest> {
        match self.payload {
            ServerRequestItemPayload::SetEndpointRole(req) => Some(req),
            _ => None,
        }
    }
}

/// A decoded incoming request batch from the server.
pub struct IncomingServerBatch {
    message: RequestMessage,
    items: Vec<ServerRequestItemPayload>,
}

impl IncomingServerBatch {
    /// Decodes an incoming request batch from encoded bytes using the given wire encoding.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] if decoding or validation fails.
    #[allow(clippy::too_many_lines)]
    pub(crate) fn from_encoded_bytes(
        bytes: &[u8],
        encoding: WireEncoding,
        limits: &CodecLimits,
    ) -> Result<Self, ClientError> {
        let item = match encoding {
            WireEncoding::Ttlv => decode_with_limits(bytes, limits).map_err(|err| {
                ClientError::protocol(
                    ProtocolError::new(
                        ProtocolErrorKind::MalformedMessage,
                        ProtocolCauseCategory::InvalidEncoding,
                        std::io::Error::other(err.to_string()),
                    ),
                    RequestDeliveryState::PossiblySent,
                )
            })?,
            WireEncoding::Json => crate::json_encoding::decode(bytes, limits).map_err(|err| {
                ClientError::protocol(
                    ProtocolError::new(
                        ProtocolErrorKind::MalformedMessage,
                        ProtocolCauseCategory::InvalidEncoding,
                        err,
                    ),
                    RequestDeliveryState::PossiblySent,
                )
            })?,
            WireEncoding::Xml => crate::xml_codec::decode(bytes, limits).map_err(|err| {
                ClientError::protocol(
                    ProtocolError::new(
                        ProtocolErrorKind::MalformedMessage,
                        ProtocolCauseCategory::InvalidEncoding,
                        err,
                    ),
                    RequestDeliveryState::PossiblySent,
                )
            })?,
            _ => {
                return Err(ClientError::protocol(
                    ProtocolError::new(
                        ProtocolErrorKind::UnsupportedValue,
                        ProtocolCauseCategory::InvalidEncoding,
                        std::io::Error::other("unsupported wire encoding"),
                    ),
                    RequestDeliveryState::PossiblySent,
                ));
            }
        };

        let structure = item
            .with_value(|value| match value {
                kmipkit_ttlv::ValueView::Structure(s) => s.try_clone(),
                _ => Err(kmipkit_ttlv::ModelError::StructureDepthExceeded),
            })
            .map_err(|err| {
                ClientError::protocol(make_protocol_error(err), RequestDeliveryState::PossiblySent)
            })?;

        let message = RequestMessage::try_from_ttlv(structure).map_err(|err| {
            ClientError::protocol(make_protocol_error(err), RequestDeliveryState::PossiblySent)
        })?;

        let mut items = Vec::new();
        for item_view in message.batch_items() {
            let op = item_view.operation();
            let parsed = match op {
                Some(DISCOVER_VERSIONS_OPERATION) => {
                    let req = ServerDiscoverVersionsRequest::try_from_request_item(item_view)
                        .map_err(|err| {
                            ClientError::protocol(
                                make_protocol_error(err),
                                RequestDeliveryState::PossiblySent,
                            )
                        })?;
                    ServerRequestItemPayload::DiscoverVersions(req)
                }
                Some(NOTIFY_OPERATION) => {
                    let req =
                        ServerNotifyRequest::try_from_request_item(item_view).map_err(|err| {
                            ClientError::protocol(
                                make_protocol_error(err),
                                RequestDeliveryState::PossiblySent,
                            )
                        })?;
                    ServerRequestItemPayload::Notify(req)
                }
                Some(PUT_OPERATION) => {
                    let req =
                        ServerPutRequest::try_from_request_item(item_view).map_err(|err| {
                            ClientError::protocol(
                                make_protocol_error(err),
                                RequestDeliveryState::PossiblySent,
                            )
                        })?;
                    ServerRequestItemPayload::Put(req)
                }
                Some(QUERY_OPERATION) => {
                    let req =
                        ServerQueryRequest::try_from_request_item(item_view).map_err(|err| {
                            ClientError::protocol(
                                make_protocol_error(err),
                                RequestDeliveryState::PossiblySent,
                            )
                        })?;
                    ServerRequestItemPayload::Query(req)
                }
                Some(SET_ENDPOINT_ROLE_OPERATION) => {
                    let req = SetEndpointRoleRequest::try_from_request_item(item_view).map_err(
                        |err| {
                            ClientError::protocol(
                                make_protocol_error(err),
                                RequestDeliveryState::PossiblySent,
                            )
                        },
                    )?;
                    ServerRequestItemPayload::SetEndpointRole(req)
                }
                Some(raw_op) => ServerRequestItemPayload::Unknown { operation: raw_op },
                None => {
                    return Err(ClientError::protocol(
                        make_protocol_error("missing operation in batch item"),
                        RequestDeliveryState::PossiblySent,
                    ));
                }
            };
            items.push(parsed);
        }

        Ok(Self { message, items })
    }

    /// Decodes an incoming request batch from TTLV bytes.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] if decoding or validation fails.
    pub(crate) fn from_ttlv_bytes(bytes: &[u8], limits: &CodecLimits) -> Result<Self, ClientError> {
        Self::from_encoded_bytes(bytes, WireEncoding::Ttlv, limits)
    }

    /// Returns the KMIP protocol version from the request header.
    #[must_use]
    pub fn protocol_version(&self) -> ProtocolVersion {
        self.message.header().protocol_version()
    }

    /// Returns the number of items in this batch.
    #[must_use]
    pub fn batch_count(&self) -> u32 {
        u32::try_from(self.items.len()).unwrap_or(u32::MAX)
    }

    /// Returns an iterator over the request item views.
    #[must_use]
    pub fn items(&self) -> Vec<ServerRequestItemView<'_>> {
        let mut views = Vec::new();
        let message_items: Vec<RequestBatchItemView<'_>> = self.message.batch_items().collect();
        for (i, payload) in self.items.iter().enumerate() {
            let item_view = message_items[i];
            let unique_batch_item_id = item_view.with_unique_batch_item_id(<[u8]>::to_vec);
            views.push(ServerRequestItemView {
                operation: item_view.operation().unwrap_or(0),
                unique_batch_item_id,
                payload,
            });
        }
        views
    }
}

/// Handler trait implemented by applications to process Server-to-Client operations.
pub trait ServerRequestHandler {
    /// Handles incoming Discover Versions request (§6.2.1).
    ///
    /// # Errors
    ///
    /// Returns [`KmipOperationResult`] with a failure status and reason if handling fails.
    fn handle_discover_versions(
        &mut self,
        request: &ServerDiscoverVersionsRequest,
    ) -> Result<ServerDiscoverVersionsResponse, KmipOperationResult>;

    /// Handles incoming Notify request (§6.2.2).
    ///
    /// # Errors
    ///
    /// Returns [`KmipOperationResult`] with a failure status and reason if handling fails.
    fn handle_notify(
        &mut self,
        request: &ServerNotifyRequest,
    ) -> Result<ServerNotifyResponse, KmipOperationResult>;

    /// Handles incoming Put request (§6.2.3).
    ///
    /// # Errors
    ///
    /// Returns [`KmipOperationResult`] with a failure status and reason if handling fails.
    fn handle_put(
        &mut self,
        request: &ServerPutRequest,
    ) -> Result<ServerPutResponse, KmipOperationResult>;

    /// Handles incoming Query request (§6.2.4).
    ///
    /// # Errors
    ///
    /// Returns [`KmipOperationResult`] with a failure status and reason if handling fails.
    fn handle_query(
        &mut self,
        request: &ServerQueryRequest,
    ) -> Result<ServerQueryResponse, KmipOperationResult>;

    /// Handles incoming Set Endpoint Role request (§6.2.5).
    ///
    /// # Errors
    ///
    /// Returns [`KmipOperationResult`] with a failure status and reason if handling fails.
    fn handle_set_endpoint_role(
        &mut self,
        request: &SetEndpointRoleRequest,
    ) -> Result<SetEndpointRoleResponse, KmipOperationResult>;
}

/// Dispatches an incoming server batch to a [`ServerRequestHandler`] and builds a [`ResponseMessage`].
#[derive(Default)]
pub struct ServerRequestDispatcher;

impl ServerRequestDispatcher {
    /// Creates a new dispatcher.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Dispatches all items in `batch` to `handler` and compiles the canonical response message.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] if response construction fails.
    #[allow(clippy::too_many_lines)]
    pub fn dispatch(
        &mut self,
        batch: &IncomingServerBatch,
        handler: &mut dyn ServerRequestHandler,
    ) -> Result<ResponseMessage, ClientError> {
        let mut builder = ServerResponseBuilder::with_version(batch.protocol_version());

        for item_view in batch.items() {
            let unique_id = item_view.unique_batch_item_id().map(<[u8]>::to_vec);

            match item_view.payload {
                ServerRequestItemPayload::DiscoverVersions(req) => {
                    match handler.handle_discover_versions(req) {
                        Ok(resp) => {
                            let payload = resp.to_ttlv_payload().map_err(map_protocol_err)?;
                            builder
                                .add_success_item(DISCOVER_VERSIONS_OPERATION, unique_id, payload)
                                .map_err(map_protocol_err)?;
                        }
                        Err(result) => {
                            builder
                                .add_failure_item(
                                    DISCOVER_VERSIONS_OPERATION,
                                    unique_id,
                                    result.status(),
                                    result.reason(),
                                    result.message().map(|m| m.as_str().to_owned()),
                                )
                                .map_err(map_protocol_err)?;
                        }
                    }
                }
                ServerRequestItemPayload::Notify(req) => match handler.handle_notify(req) {
                    Ok(resp) => {
                        let payload = resp.to_ttlv_payload().map_err(map_protocol_err)?;
                        builder
                            .add_success_item(NOTIFY_OPERATION, unique_id, payload)
                            .map_err(map_protocol_err)?;
                    }
                    Err(result) => {
                        builder
                            .add_failure_item(
                                NOTIFY_OPERATION,
                                unique_id,
                                result.status(),
                                result.reason(),
                                result.message().map(|m| m.as_str().to_owned()),
                            )
                            .map_err(map_protocol_err)?;
                    }
                },
                ServerRequestItemPayload::Put(req) => match handler.handle_put(req) {
                    Ok(resp) => {
                        let payload = resp.to_ttlv_payload().map_err(map_protocol_err)?;
                        builder
                            .add_success_item(PUT_OPERATION, unique_id, payload)
                            .map_err(map_protocol_err)?;
                    }
                    Err(result) => {
                        builder
                            .add_failure_item(
                                PUT_OPERATION,
                                unique_id,
                                result.status(),
                                result.reason(),
                                result.message().map(|m| m.as_str().to_owned()),
                            )
                            .map_err(map_protocol_err)?;
                    }
                },
                ServerRequestItemPayload::Query(req) => match handler.handle_query(req) {
                    Ok(resp) => {
                        let payload = resp.to_ttlv_payload().map_err(map_protocol_err)?;
                        builder
                            .add_success_item(QUERY_OPERATION, unique_id, payload)
                            .map_err(map_protocol_err)?;
                    }
                    Err(result) => {
                        builder
                            .add_failure_item(
                                QUERY_OPERATION,
                                unique_id,
                                result.status(),
                                result.reason(),
                                result.message().map(|m| m.as_str().to_owned()),
                            )
                            .map_err(map_protocol_err)?;
                    }
                },
                ServerRequestItemPayload::SetEndpointRole(req) => {
                    match handler.handle_set_endpoint_role(req) {
                        Ok(resp) => {
                            let payload = resp.to_ttlv_payload().map_err(map_protocol_err)?;
                            builder
                                .add_success_item(SET_ENDPOINT_ROLE_OPERATION, unique_id, payload)
                                .map_err(map_protocol_err)?;
                        }
                        Err(result) => {
                            builder
                                .add_failure_item(
                                    SET_ENDPOINT_ROLE_OPERATION,
                                    unique_id,
                                    result.status(),
                                    result.reason(),
                                    result.message().map(|m| m.as_str().to_owned()),
                                )
                                .map_err(map_protocol_err)?;
                        }
                    }
                }
                ServerRequestItemPayload::Unknown { operation } => {
                    builder
                        .add_failure_item(
                            *operation,
                            unique_id,
                            ResultStatus::from_raw(1),
                            Some(ResultReason::from_raw(0x05)),
                            Some("operation not supported".to_owned()),
                        )
                        .map_err(map_protocol_err)?;
                }
            }
        }

        builder.build().map_err(map_protocol_err)
    }
}

fn map_protocol_err(err: ProtocolError) -> ClientError {
    ClientError::protocol(err, RequestDeliveryState::PossiblySent)
}

/// An established session operating under role reversal.
pub struct RoleReversedSession {
    stream: RawTlsRoleReversedStream,
}

impl RoleReversedSession {
    /// Creates a session wrapping a raw-TLS role-reversed stream.
    #[must_use]
    pub fn new(stream: RawTlsRoleReversedStream) -> Self {
        Self { stream }
    }

    /// Reads and decodes an incoming request batch from the server.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] if receiving or parsing fails.
    pub fn recv_batch(
        &mut self,
        limits: &CodecLimits,
        timeout: Option<Duration>,
    ) -> Result<IncomingServerBatch, ClientError> {
        let max_bytes = limits.max_message_bytes();
        let resp = self
            .stream
            .recv_server_request(max_bytes, timeout)
            .map_err(ClientError::transport)?;
        IncomingServerBatch::from_ttlv_bytes(resp.as_bytes(), limits)
    }

    /// Sends a compiled response message to the server.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] if serialization or sending fails.
    pub fn send_response(
        &mut self,
        response: &ResponseMessage,
        timeout: Option<Duration>,
    ) -> Result<(), ClientError> {
        let mut bytes = Vec::new();
        response.with_ttlv(|view| {
            encode_structure_view(RESPONSE_MESSAGE_TAG, &view, &mut bytes);
        });
        self.stream
            .send_client_response(&bytes, timeout)
            .map_err(ClientError::transport)
    }

    /// Closes the session.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] if closing fails.
    pub fn close(&mut self) -> Result<(), ClientError> {
        self.stream.close().map_err(ClientError::transport)
    }
}

enum ListenerInner {
    RawTls(RawTlsListener),
    Https(HttpsListener),
}

/// A listener for incoming server-initiated KMIP connections.
pub struct ClientListener {
    inner: ListenerInner,
}

impl ClientListener {
    /// Binds a raw-TLS listener.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] if binding fails.
    pub fn bind_raw_tls(config: ListenerTransportConfig) -> Result<Self, ClientError> {
        let listener = RawTlsListener::bind(config).map_err(ClientError::transport)?;
        Ok(Self {
            inner: ListenerInner::RawTls(listener),
        })
    }

    /// Binds an HTTPS listener.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] if binding fails.
    pub fn bind_https(config: ListenerTransportConfig) -> Result<Self, ClientError> {
        let listener = HttpsListener::bind(config).map_err(ClientError::transport)?;
        Ok(Self {
            inner: ListenerInner::Https(listener),
        })
    }

    /// Returns the local bound socket address.
    ///
    /// # Errors
    ///
    /// Returns an IO error if querying fails.
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        match &self.inner {
            ListenerInner::RawTls(l) => l.local_addr(),
            ListenerInner::Https(l) => l.local_addr(),
        }
    }

    /// Returns the bound TCP port.
    ///
    /// # Errors
    ///
    /// Returns an IO error if querying fails.
    pub fn port(&self) -> io::Result<u16> {
        self.local_addr().map(|addr| addr.port())
    }

    /// Accepts an incoming server connection.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] if accepting fails.
    pub fn accept(
        &mut self,
        timeout: Option<Duration>,
    ) -> Result<ClientListenerConnection, ClientError> {
        match &mut self.inner {
            ListenerInner::RawTls(l) => {
                let conn = l.accept(timeout).map_err(ClientError::transport)?;
                Ok(ClientListenerConnection {
                    inner: ConnectionInner::RawTls(conn),
                })
            }
            ListenerInner::Https(l) => {
                let conn = l.accept(timeout).map_err(ClientError::transport)?;
                Ok(ClientListenerConnection {
                    inner: ConnectionInner::Https(conn, WireEncoding::Ttlv),
                })
            }
        }
    }
}

enum ConnectionInner {
    RawTls(RawTlsServerConnection),
    Https(HttpsServerConnection, WireEncoding),
}

/// An established connection accepted by [`ClientListener`].
pub struct ClientListenerConnection {
    inner: ConnectionInner,
}

impl ClientListenerConnection {
    /// Returns the peer address.
    #[must_use]
    pub fn peer_addr(&self) -> SocketAddr {
        match &self.inner {
            ConnectionInner::RawTls(conn) => conn.peer_addr(),
            ConnectionInner::Https(conn, _) => conn.peer_addr(),
        }
    }

    /// Reads and decodes an incoming request batch from the server.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] if reading or decoding fails.
    pub fn recv_batch(
        &mut self,
        limits: &CodecLimits,
        timeout: Option<Duration>,
    ) -> Result<IncomingServerBatch, ClientError> {
        let max_bytes = limits.max_message_bytes();
        match &mut self.inner {
            ConnectionInner::RawTls(conn) => {
                let resp = conn
                    .recv_request(max_bytes, timeout)
                    .map_err(ClientError::transport)?;
                IncomingServerBatch::from_encoded_bytes(resp.as_bytes(), WireEncoding::Ttlv, limits)
            }
            ConnectionInner::Https(conn, encoding) => {
                let (resp, enc) = conn
                    .recv_request(max_bytes, timeout)
                    .map_err(ClientError::transport)?;
                *encoding = enc;
                IncomingServerBatch::from_encoded_bytes(resp.as_bytes(), enc, limits)
            }
        }
    }

    /// Sends a response message to the server over the connection.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] if serializing or writing fails.
    pub fn send_response(
        &mut self,
        response: &ResponseMessage,
        timeout: Option<Duration>,
    ) -> Result<(), ClientError> {
        let mut bytes = Vec::new();
        response.with_ttlv(|view| {
            encode_structure_view(RESPONSE_MESSAGE_TAG, &view, &mut bytes);
        });

        match &mut self.inner {
            ConnectionInner::RawTls(conn) => conn
                .send_response(&bytes, timeout)
                .map_err(ClientError::transport),
            ConnectionInner::Https(conn, encoding) => conn
                .send_response(&bytes, *encoding, timeout)
                .map_err(ClientError::transport),
        }
    }

    /// Converts a raw-TLS connection into a role-reversed session.
    ///
    /// # Errors
    ///
    /// Returns an error if this connection is HTTPS (which does not support raw role reversal).
    pub fn into_role_reversed(self) -> Result<RoleReversedSession, ClientError> {
        match self.inner {
            ConnectionInner::RawTls(conn) => {
                let stream = conn.into_role_reversed();
                Ok(RoleReversedSession::new(stream))
            }
            ConnectionInner::Https(..) => Err(ClientError::transport(TransportError::new(
                RequestDeliveryState::NotSent,
                TransportCauseCategory::Other,
                io::Error::other(
                    "HTTPS connections cannot be converted to RawTlsRoleReversedStream",
                ),
            ))),
        }
    }

    /// Closes the connection.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] if closing fails.
    pub fn close(&mut self) -> Result<(), ClientError> {
        match &mut self.inner {
            ConnectionInner::RawTls(conn) => conn.close().map_err(ClientError::transport),
            ConnectionInner::Https(conn, _) => conn.close().map_err(ClientError::transport),
        }
    }
}

pub(crate) fn encode_structure_view(tag: u32, view: &StructureView<'_>, out: &mut Vec<u8>) {
    let raw_tag = tag.to_be_bytes();
    let tag_bytes = [raw_tag[1], raw_tag[2], raw_tag[3]];
    let mut children_bytes = Vec::new();
    for child in view.children() {
        encode_ttlv_item(child, &mut children_bytes);
    }
    write_ttlv_header(out, tag_bytes, 0x01, children_bytes.len());
    out.extend_from_slice(&children_bytes);
}

fn encode_ttlv_item(item: &Item, out: &mut Vec<u8>) {
    let raw_tag = item.tag().raw().to_be_bytes();
    let tag_bytes = [raw_tag[1], raw_tag[2], raw_tag[3]];

    item.with_value(|view| match view {
        kmipkit_ttlv::ValueView::Structure(structure) => {
            let mut children_bytes = Vec::new();
            for child in structure.children() {
                encode_ttlv_item(child, &mut children_bytes);
            }
            write_ttlv_header(out, tag_bytes, 0x01, children_bytes.len());
            out.extend_from_slice(&children_bytes);
        }
        kmipkit_ttlv::ValueView::Integer(val) => {
            write_ttlv_header(out, tag_bytes, 0x02, 4);
            out.extend_from_slice(&val.to_be_bytes());
            out.extend_from_slice(&[0u8; 4]);
        }
        kmipkit_ttlv::ValueView::LongInteger(val) => {
            write_ttlv_header(out, tag_bytes, 0x03, 8);
            out.extend_from_slice(&val.to_be_bytes());
        }
        kmipkit_ttlv::ValueView::BigInteger(bytes) => {
            write_ttlv_header(out, tag_bytes, 0x04, bytes.len());
            out.extend_from_slice(bytes);
            write_ttlv_padding(out, bytes.len());
        }
        kmipkit_ttlv::ValueView::Enumeration(val) => {
            write_ttlv_header(out, tag_bytes, 0x05, 4);
            out.extend_from_slice(&val.to_be_bytes());
            out.extend_from_slice(&[0u8; 4]);
        }
        kmipkit_ttlv::ValueView::Boolean(val) => {
            write_ttlv_header(out, tag_bytes, 0x06, 8);
            let byte_val = u8::from(*val);
            out.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, byte_val]);
        }
        kmipkit_ttlv::ValueView::TextString(s) => {
            let bytes = s.as_bytes();
            write_ttlv_header(out, tag_bytes, 0x07, bytes.len());
            out.extend_from_slice(bytes);
            write_ttlv_padding(out, bytes.len());
        }
        kmipkit_ttlv::ValueView::ByteString(bytes) => {
            write_ttlv_header(out, tag_bytes, 0x08, bytes.len());
            out.extend_from_slice(bytes);
            write_ttlv_padding(out, bytes.len());
        }
        kmipkit_ttlv::ValueView::DateTime(dt) => {
            write_ttlv_header(out, tag_bytes, 0x09, 8);
            out.extend_from_slice(&dt.to_be_bytes());
        }
        kmipkit_ttlv::ValueView::Interval(val) => {
            write_ttlv_header(out, tag_bytes, 0x0A, 4);
            out.extend_from_slice(&val.to_be_bytes());
            out.extend_from_slice(&[0u8; 4]);
        }
        _ => {}
    });
}

fn write_ttlv_header(out: &mut Vec<u8>, tag_bytes: [u8; 3], item_type: u8, len: usize) {
    out.extend_from_slice(&tag_bytes);
    out.push(item_type);
    out.extend_from_slice(&u32::try_from(len).unwrap_or(0).to_be_bytes());
}

fn write_ttlv_padding(out: &mut Vec<u8>, len: usize) {
    let rem = len % 8;
    if rem > 0 {
        let pad = 8 - rem;
        out.extend(std::iter::repeat_n(0u8, pad));
    }
}
