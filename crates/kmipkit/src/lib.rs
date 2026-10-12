//! The supported high-level Rust API for the `KMIPKit` client library.
#![forbid(unsafe_code)]

/// The synchronized KMIPKit package version compiled into this library.
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

/// Returns whether this build includes the named KMIPKit API capability.
///
/// This query reports the library's local API surface. It does not determine
/// whether a remote KMIP server supports the capability. Unknown and blocked
/// capability identifiers return `false`.
#[must_use]
pub fn capability_is_available(capability_id: &str) -> bool {
    matches!(
        capability_id,
        "ffi_runtime_metadata" | "generic_ttlv_model" | "ttlv_wire_codec"
    )
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
