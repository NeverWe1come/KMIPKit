//! The supported high-level Rust API for the `KMIPKit` client library.
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
    ProcessRequest, ProtocolCauseCategory, ProtocolError, ProtocolErrorKind,
    QueryAsyncRequestsRequest, ResultMessage, ResultReason, ResultStatus, ResultValidationError,
    SetConstraintsError, SetConstraintsRequest, SetConstraintsResponse, Ticket, TicketType,
    TicketValidationError, TicketValidationErrorKind,
};
pub use kmipkit_transport::{RequestDeliveryState, TransportCauseCategory, TransportError};
