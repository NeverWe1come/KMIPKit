//! The supported high-level Rust API for the `KMIPKit` client library.
//!
//! ## Per-exchange profile validation
//!
//! A profile selection is explicit for each call. The helper below accepts a
//! configured client and a caller-built typed batch:
//!
//! ```no_run
//! use kmipkit::{
//!     Client, ClientBatch, ClientBatchResponse, ClientError, ProfileValidationSelection,
//! };
//! use kmipkit_ttlv::codec::CodecLimits;
//!
//! fn execute_for_profile(
//!     client: &mut Client,
//!     batch: ClientBatch,
//!     limits: &CodecLimits,
//! ) -> Result<ClientBatchResponse, ClientError> {
//!     let selection =
//!         ProfileValidationSelection::new(["KMIPKIT-PROFILE-BASELINE-CLIENT"]);
//!     client.execute_with_profiles(batch, &selection, limits)
//! }
//! ```
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
        "ffi_runtime_metadata"
            | "generic_ttlv_model"
            | "ttlv_wire_codec"
            | "vendor_extension_registry"
            | "discover_versions_request_item"
            | "opaque_register_context"
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
    ProcessRequest, ProfileTransport, ProfileValidationError, ProfileValidationErrorCode,
    ProfileValidationResult, ProfileValidationSelection, ProtocolCauseCategory, ProtocolError,
    ProtocolErrorKind, QueryAsyncRequestsRequest, ResultMessage, ResultReason, ResultStatus,
    ResultValidationError, RngRetrieveError, RngRetrieveRequest, RngRetrieveResponse, RngSeedError,
    RngSeedRequest, RngSeedResponse, SecretBytes, SetConstraintsError, SetConstraintsRequest,
    SetConstraintsResponse, SetDefaultsError, SetDefaultsRequest, SetDefaultsResponse, Ticket,
    TicketType, TicketValidationError, TicketValidationErrorKind,
};
pub use kmipkit_transport::{RequestDeliveryState, TransportCauseCategory, TransportError};
