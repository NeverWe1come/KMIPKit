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
