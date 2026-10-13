//! The supported high-level Rust API for the `KMIPKit` client library.
#![forbid(unsafe_code)]

pub use kmipkit_client::{
    Client, ClientBatch, ClientBatchItem, ClientBatchItemResponse, ClientBatchOutcome,
    ClientBatchResponse, ClientCauseCategory, ClientError, ClientErrorCategory,
    ClientMessageExtension, ClientOperation, ClientOperationOutcome, ClientRequest, PendingOutcome,
    RequestOptions,
};
pub use kmipkit_protocol::{
    CancelRequest, KmipOperationResult, PollRequest, ProcessRequest, ProtocolCauseCategory,
    ProtocolError, ProtocolErrorKind, QueryAsyncRequestsRequest, ResultMessage, ResultReason,
    ResultStatus, ResultValidationError,
};
pub use kmipkit_transport::{RequestDeliveryState, TransportCauseCategory, TransportError};

pub use kmipkit_protocol::{GetConstraintsRequest, GetConstraintsResponse};

pub use kmipkit_protocol::{GetUsageAllocationRequest, GetUsageAllocationResponse};

pub use kmipkit_client::{GuardedProtectionRequest, UsageAllocationContext};

pub use kmipkit_protocol::{SetConstraintsRequest, SetConstraintsResponse};

pub use kmipkit_protocol::{SetDefaultsRequest, SetDefaultsResponse};

pub use kmipkit_protocol::{SetEndpointRoleRequest, SetEndpointRoleResponse};
