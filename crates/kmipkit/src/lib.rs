//! The supported high-level Rust API for the `KMIPKit` client library.
#![forbid(unsafe_code)]

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
/// has no request-delivery state.
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
            | "client_batch_model"
            | "client_execution"
            | "crypto_response_ttlv"
            | "ffi_runtime_metadata"
            | "generic_ttlv_model"
            | "production_transport"
            | "ttlv_wire_codec"
            | "usage_allocation_guard"
            | "vendor_extension_registry"
            | "discover_versions_request_item"
            | "opaque_register_context"
            | "server_generated_creation"
    ))
}

pub use kmipkit_client::{
    Client, ClientBatch, ClientBatchItem, ClientBatchItemResponse, ClientBatchOutcome,
    ClientBatchResponse, ClientCauseCategory, ClientError, ClientErrorCategory,
    ClientMessageExtension, ClientOperation, ClientOperationOutcome, ClientRequest, PendingOutcome,
    RequestOptions, UsageAllocationContext, UsageAllocationContextError,
};
pub use kmipkit_protocol::{
    CancelRequest, DelegatedLoginError, DelegatedLoginRequest, DelegatedLoginResponse,
    GetConstraintsError, GetConstraintsRequest, GetConstraintsResponse, GetUsageAllocationError,
    GetUsageAllocationRequest, GetUsageAllocationResponse, KmipOperationResult, LoginError,
    LoginRequest, LoginResponse, LogoutError, LogoutRequest, LogoutResponse, PollRequest,
    ProcessRequest, ProtocolCauseCategory, ProtocolError, ProtocolErrorKind,
    QueryAsyncRequestsRequest, ResultMessage, ResultReason, ResultStatus, ResultValidationError,
    RngRetrieveError, RngRetrieveRequest, RngRetrieveResponse, RngSeedError, RngSeedRequest,
    RngSeedResponse, SecretBytes, SetConstraintsError, SetConstraintsRequest,
    SetConstraintsResponse, SetDefaultsError, SetDefaultsRequest, SetDefaultsResponse, Ticket,
    TicketType, TicketValidationError, TicketValidationErrorKind,
};
pub use kmipkit_transport::{RequestDeliveryState, TransportCauseCategory, TransportError};
