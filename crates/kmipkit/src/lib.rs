//! The supported high-level Rust API for the `KMIPKit` client library.
//!
//! ## Per-exchange profile validation
//!
//! A profile selection and any caller-known object state are explicit for
//! each call. State is attached to the exact batch item and object ID:
//!
//! ```no_run
//! use kmipkit::{
//!     Client, ClientBatch, ClientBatchResponse, ClientError, ProfileObjectState,
//!     ProfileValidationContext, ProfileValidationSelection,
//! };
//! use kmipkit::ttlv::codec::CodecLimits;
//!
//! fn execute_for_profile(
//!     client: &mut Client,
//!     batch: ClientBatch,
//!     batch_item_index: usize,
//!     unique_identifier: &str,
//!     limits: &CodecLimits,
//! ) -> Result<ClientBatchResponse, ClientError> {
//!     let selection =
//!         ProfileValidationSelection::new(["KMIPKIT-PROFILE-BASELINE-CLIENT"]);
//!     let context = ProfileValidationContext::new().with_object_state(
//!         batch_item_index,
//!         ProfileObjectState::new(unique_identifier)
//!             .with_original_creation_date_present(Some(true)),
//!     );
//!     client.execute_with_profiles_and_context(batch, &selection, &context, limits)
//! }
//! ```
#![forbid(unsafe_code)]

/// Complete typed protocol models and operation helpers.
pub use kmipkit_protocol as protocol;
/// Generic TTLV values and the bounded decoder.
pub use kmipkit_ttlv as ttlv;

/// Configuration for client-owned vendor extension definitions.
pub mod client_config {
    pub use kmipkit_client::extension_registry::{
        ClientConfiguration, ClientExtensionRegistry, client_extension_registry,
    };
}

/// Selected configuration types for the supported TLS 1.3 mutual-TLS transports.
///
/// This namespace configures the high-level client. It intentionally omits the
/// raw transport exchange trait, response byte containers, and concrete
/// transport adapters.
pub mod transport_config {
    pub use kmipkit_transport::{
        CertificateInput, ClientIdentity, Endpoint, PrivateKeyInput, RevocationListInput,
        TimeoutLimit, TimeoutPolicy, TransportConfig, TransportConfigBuilder, TransportConfigError,
        TrustSource,
    };
}

/// The synchronized `KMIPKit` package version compiled into this library.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The stable C ABI revision reported by this release line.
pub const ABI_VERSION: u32 = 1;

/// Synchronized product and C ABI version metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeVersion {
    /// The package's synchronized semantic version.
    pub product_version: &'static str,
    /// The stable C ABI revision.
    pub abi_version: u32,
}

/// Returns the synchronized product version and stable C ABI revision.
#[must_use]
pub const fn runtime_version() -> RuntimeVersion {
    RuntimeVersion {
        product_version: VERSION,
        abi_version: ABI_VERSION,
    }
}

/// Returns the local facade levels provided by this library build.
///
/// The ordered levels describe the `KMIPKit` API surface: high-level builders,
/// typed protocol requests and responses, and generic structurally valid TTLV.
/// This metadata query does not send a request and is distinct from the KMIP
/// Discover Versions operation. It reports the API taxonomy only; it does not
/// assert that every KMIP operation is mapped or complete.
#[must_use]
pub const fn api_levels() -> &'static [&'static str] {
    &["high-level", "typed-protocol", "generic-ttlv"]
}

/// A stable error returned by local runtime metadata queries.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeMetadataError {
    /// The UTF-8 capability identifier exceeded the fixed 128-byte input limit.
    ResourceLimit,
}

impl std::fmt::Display for RuntimeMetadataError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ResourceLimit => {
                formatter.write_str("capability identifier exceeds the 128-byte limit")
            }
        }
    }
}

impl std::error::Error for RuntimeMetadataError {}

/// Returns whether this build includes the named `KMIPKit` API capability.
///
/// This query reports the library's local API surface. It does not determine
/// whether a remote KMIP server supports the capability. Unknown and blocked
/// capability identifiers return `Ok(false)`. Identifiers longer than 128 UTF-8
/// bytes return [`RuntimeMetadataError::ResourceLimit`], matching the fixed-width
/// binding boundary. This local query does not send a KMIP request and therefore
/// has no request-delivery state. The opt-in `kmip30_draft_routes` capability
/// reports route availability and does not assert KMIP 3.0 conformance.
///
/// # Errors
///
/// Returns [`RuntimeMetadataError::ResourceLimit`] when the identifier exceeds
/// 128 UTF-8 bytes.
pub fn capability_is_available(capability_id: &str) -> Result<bool, RuntimeMetadataError> {
    if capability_id.len() > 128 {
        return Err(RuntimeMetadataError::ResourceLimit);
    }

    Ok(matches!(
        capability_id,
        "api_level_discovery"
            | "client_asynchronous_operations"
            | "client_batch_model"
            | "client_execution"
            | "client_kmip30_typed_operations"
            | "client_operation_identity"
            | "wire_encoding_selection"
            | "client_profile_readiness"
            | "client_profile_validation"
            | "credential_secret_parity"
            | "credential_ttlv_parity"
            | "credential_type_parity"
            | "credential_validation_error_parity"
            | "crypto_response_ttlv"
            | "ffi_runtime_metadata"
            | "fluent_add_attribute_parity"
            | "fluent_create_key_pair_parity"
            | "fluent_delete_attribute_parity"
            | "fluent_dx_api"
            | "fluent_locate_revoke_parity"
            | "fluent_named_tag_values"
            | "fluent_set_attribute_parity"
            | "generic_ttlv_model"
            | "hashed_password_timestamp_sequence"
            | "kmip30_draft_routes"
            | "kmip_protocol_version_selection"
            | "named_protocol_values"
            | "nonce_ttlv_parity"
            | "production_transport"
            | "request_message_authentication"
            | "request_validation"
            | "ttlv_wire_codec"
            | "usage_allocation_guard"
            | "vendor_extension_registry"
            | "discover_versions_request_item"
            | "opaque_register_context"
            | "server_generated_creation"
            | "typed_message_model"
            | "public_binding_surface"
            | "response_ttlv_navigation"
            | "typed_async_response_readers"
            | "typed_cryptographic_parameters"
            | "typed_managed_objects_and_key_material"
            | "typed_object_attributes"
            | "server_to_client_operations"
            | "endpoint_role_inversion"
    ))
}

pub use kmipkit_client::extension_registry::{ClientConfiguration, ClientExtensionRegistry};
pub use kmipkit_client::fluent;
pub use kmipkit_client::fluent::{
    AdminClient, AsyncOpsClient, AttributesClient, CertificatesClient, CryptographicUsageMask,
    CryptoClient, DecryptStream, EncryptStream, FluentBatchBuilder, FluentBuildError, HashStream,
    KeyRef, KeysClient, LocatePaginator, MacStream, PendingOperation, RequestBuilder,
    SecretsClient, SessionsClient, SignStream, StreamsClient, WorkflowsClient,
};
pub use kmipkit_protocol::{
    AttributesBuilder, CipherSpec, HashAlgorithm, KdfSpec, KemSpec, KeySpec, MacSpec, PemBlock,
    PresetError, SignatureSpec, decode_pem, decode_pem_der,
};
pub use kmipkit_client::{
    Client, ClientBatch, ClientBatchItem, ClientBatchItemResponse, ClientBatchOutcome,
    ClientBatchResponse, ClientBuilder, ClientCauseCategory, ClientError, ClientErrorCategory,
    ClientListener, ClientListenerConnection, ClientMessageExtension, ClientOperation,
    ClientOperationOutcome, ClientRequest, ClientResponseView, IncomingServerBatch, PendingOutcome,
    RedactedCorrelation, RequestOptions, ResponseEncodingError, RoleReversedSession,
    ServerRequestDispatcher, ServerRequestHandler, ServerRequestItemPayload,
    ServerRequestItemView, UsageAllocationContext, UsageAllocationContextError, WireEncoding,
};
pub use kmipkit_ttlv::codec::CodecLimits;
pub use kmipkit_protocol::{
    CancelRequest, CodecContext, DelegatedLoginError, DelegatedLoginRequest,
    DelegatedLoginResponse, EndpointRole, GetConstraintsError, GetConstraintsRequest,
    GetConstraintsResponse, GetUsageAllocationError, GetUsageAllocationRequest,
    GetUsageAllocationResponse, KmipOperationResult, LoginError, LoginRequest, LoginResponse,
    LogoutError, LogoutRequest, LogoutResponse, PollRequest, ProcessRequest, ProfileObjectState,
    ProfileTransport, ProfileValidationContext, ProfileValidationError, ProfileValidationErrorCode,
    ProfileValidationResult, ProfileValidationSelection, ProtocolCauseCategory, ProtocolError,
    ProtocolErrorKind, ProtocolVersion, QueryAsyncRequestsRequest, ResultMessage, ResultReason,
    ResultStatus, ResultValidationError, RngRetrieveError, RngRetrieveRequest, RngRetrieveResponse,
    RngSeedError, RngSeedRequest, RngSeedResponse, SecretBytes, SetConstraintsError,
    SetConstraintsRequest, SetConstraintsResponse, SetDefaultsError, SetDefaultsRequest,
    SetDefaultsResponse, SetEndpointRoleError, SetEndpointRoleRequest, SetEndpointRoleResponse,
    Ticket, TicketType, TicketValidationError, TicketValidationErrorKind, VersionGate,
    VersionGateError, VersionSpan, VersionedCodec,
};
pub use kmipkit_transport::{RequestDeliveryState, TransportCauseCategory, TransportError};

type HandlerCallback<Req, Resp> =
    Option<Box<dyn FnMut(&Req) -> Result<Resp, kmipkit_protocol::KmipOperationResult> + Send>>;

#[derive(Default)]
struct ServerHandlers {
    on_discover_versions: HandlerCallback<
        kmipkit_protocol::ServerDiscoverVersionsRequest,
        kmipkit_protocol::ServerDiscoverVersionsResponse,
    >,
    on_notify: HandlerCallback<
        kmipkit_protocol::ServerNotifyRequest,
        kmipkit_protocol::ServerNotifyResponse,
    >,
    on_put:
        HandlerCallback<kmipkit_protocol::ServerPutRequest, kmipkit_protocol::ServerPutResponse>,
    on_query: HandlerCallback<
        kmipkit_protocol::ServerQueryRequest,
        kmipkit_protocol::ServerQueryResponse,
    >,
    on_set_endpoint_role: HandlerCallback<
        kmipkit_protocol::SetEndpointRoleRequest,
        kmipkit_protocol::SetEndpointRoleResponse,
    >,
}

/// Closure-based builder implementing [`ServerRequestHandler`] for Server-to-Client listeners.
#[derive(Default)]
pub struct ServerHandlerBuilder {
    handlers: ServerHandlers,
}

impl ServerHandlerBuilder {
    /// Creates an empty builder where unconfigured operations return `Operation Not Supported`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a handler for incoming `DiscoverVersions` requests (§6.2.1).
    #[must_use]
    pub fn on_discover_versions(
        mut self,
        handler: impl FnMut(
            &kmipkit_protocol::ServerDiscoverVersionsRequest,
        ) -> Result<
            kmipkit_protocol::ServerDiscoverVersionsResponse,
            kmipkit_protocol::KmipOperationResult,
        > + Send
        + 'static,
    ) -> Self {
        self.handlers.on_discover_versions = Some(Box::new(handler));
        self
    }

    /// Registers a handler for incoming `Notify` requests (§6.2.2).
    #[must_use]
    pub fn on_notify(
        mut self,
        handler: impl FnMut(
            &kmipkit_protocol::ServerNotifyRequest,
        ) -> Result<
            kmipkit_protocol::ServerNotifyResponse,
            kmipkit_protocol::KmipOperationResult,
        > + Send
        + 'static,
    ) -> Self {
        self.handlers.on_notify = Some(Box::new(handler));
        self
    }

    /// Registers a handler for incoming `Put` requests (§6.2.3).
    #[must_use]
    pub fn on_put(
        mut self,
        handler: impl FnMut(
            &kmipkit_protocol::ServerPutRequest,
        ) -> Result<
            kmipkit_protocol::ServerPutResponse,
            kmipkit_protocol::KmipOperationResult,
        > + Send
        + 'static,
    ) -> Self {
        self.handlers.on_put = Some(Box::new(handler));
        self
    }

    /// Registers a handler for incoming `Query` requests (§6.2.4).
    #[must_use]
    pub fn on_query(
        mut self,
        handler: impl FnMut(
            &kmipkit_protocol::ServerQueryRequest,
        ) -> Result<
            kmipkit_protocol::ServerQueryResponse,
            kmipkit_protocol::KmipOperationResult,
        > + Send
        + 'static,
    ) -> Self {
        self.handlers.on_query = Some(Box::new(handler));
        self
    }

    /// Registers a handler for incoming `SetEndpointRole` requests (§6.2.5).
    #[must_use]
    pub fn on_set_endpoint_role(
        mut self,
        handler: impl FnMut(
            &kmipkit_protocol::SetEndpointRoleRequest,
        ) -> Result<
            kmipkit_protocol::SetEndpointRoleResponse,
            kmipkit_protocol::KmipOperationResult,
        > + Send
        + 'static,
    ) -> Self {
        self.handlers.on_set_endpoint_role = Some(Box::new(handler));
        self
    }

    /// Finalizes the [`ClosureServerHandler`] ready for [`ServerRequestDispatcher`].
    #[must_use]
    pub fn build(self) -> ClosureServerHandler {
        ClosureServerHandler {
            handlers: self.handlers,
        }
    }
}

/// Built [`ServerRequestHandler`] backed by user-provided closures.
pub struct ClosureServerHandler {
    handlers: ServerHandlers,
}

fn unsupported_s2c_result() -> kmipkit_protocol::KmipOperationResult {
    kmipkit_protocol::KmipOperationResult::new(
        kmipkit_protocol::ResultStatus::from_raw(1),
        Some(kmipkit_protocol::ResultReason::from_raw(5)),
        Some(kmipkit_protocol::ResultMessage::new(
            "handler not configured".to_owned(),
        )),
    )
    .unwrap_or_else(|_| kmipkit_protocol::KmipOperationResult::success())
}

impl ServerRequestHandler for ClosureServerHandler {
    fn handle_discover_versions(
        &mut self,
        request: &kmipkit_protocol::ServerDiscoverVersionsRequest,
    ) -> Result<
        kmipkit_protocol::ServerDiscoverVersionsResponse,
        kmipkit_protocol::KmipOperationResult,
    > {
        self.handlers
            .on_discover_versions
            .as_mut()
            .map_or_else(|| Err(unsupported_s2c_result()), |cb| cb(request))
    }

    fn handle_notify(
        &mut self,
        request: &kmipkit_protocol::ServerNotifyRequest,
    ) -> Result<kmipkit_protocol::ServerNotifyResponse, kmipkit_protocol::KmipOperationResult> {
        self.handlers
            .on_notify
            .as_mut()
            .map_or_else(|| Err(unsupported_s2c_result()), |cb| cb(request))
    }

    fn handle_put(
        &mut self,
        request: &kmipkit_protocol::ServerPutRequest,
    ) -> Result<kmipkit_protocol::ServerPutResponse, kmipkit_protocol::KmipOperationResult> {
        self.handlers
            .on_put
            .as_mut()
            .map_or_else(|| Err(unsupported_s2c_result()), |cb| cb(request))
    }

    fn handle_query(
        &mut self,
        request: &kmipkit_protocol::ServerQueryRequest,
    ) -> Result<kmipkit_protocol::ServerQueryResponse, kmipkit_protocol::KmipOperationResult> {
        self.handlers
            .on_query
            .as_mut()
            .map_or_else(|| Err(unsupported_s2c_result()), |cb| cb(request))
    }

    fn handle_set_endpoint_role(
        &mut self,
        request: &kmipkit_protocol::SetEndpointRoleRequest,
    ) -> Result<kmipkit_protocol::SetEndpointRoleResponse, kmipkit_protocol::KmipOperationResult> {
        self.handlers
            .on_set_endpoint_role
            .as_mut()
            .map_or_else(|| Err(unsupported_s2c_result()), |cb| cb(request))
    }
}

#[cfg(test)]
mod server_handler_builder_tests {
    use super::*;

    #[test]
    fn server_handler_builder_dispatches_configured_and_unconfigured_ops() {
        let mut handler = ServerHandlerBuilder::new()
            .on_notify(|_req| Ok(kmipkit_protocol::ServerNotifyResponse::new()))
            .build();
        let notify_req =
            kmipkit_protocol::ServerNotifyRequest::new("id-123".to_owned(), None, Vec::new());
        assert!(handler.handle_notify(&notify_req).is_ok());

        let query_req = kmipkit_protocol::ServerQueryRequest::new(vec![]);
        let err = handler
            .handle_query(&query_req)
            .expect_err("unconfigured handler returns OperationNotSupported");
        assert_eq!(err.status(), kmipkit_protocol::ResultStatus::from_raw(1));
    }
}
