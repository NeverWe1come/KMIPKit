//! KMIP 2.1 protocol models and validation for `KMIPKit`.
#![forbid(unsafe_code)]

mod asynchronous;
mod cancel;
mod discover_versions;
mod error;
mod message;
mod poll;
mod process;
mod query_async_requests;
mod result;

pub use asynchronous::AsynchronousOperationError;
pub use cancel::{CancelRequest, CancelResponse, CancellationResult};
pub use discover_versions::{
    DiscoverVersionsError, DiscoverVersionsRequest, DiscoverVersionsResponse,
};
pub use error::{ProtocolCauseCategory, ProtocolError, ProtocolErrorKind};
pub use message::{
    MessageExtensionView, MessageValidationError, MessageValidationErrorKind, ProtocolVersion,
    RequestBatchItemView, RequestHeaderView, RequestMessage, ResponseBatchItemView,
    ResponseHeaderView, ResponseMessage,
};
pub use poll::{PollRequest, PollResponse};
pub use process::{ProcessRequest, ProcessResponse};
pub use query_async_requests::{QueryAsyncRequestsRequest, QueryAsyncRequestsResponse};
pub use result::{
    KmipOperationResult, ResultMessage, ResultReason, ResultStatus, ResultValidationError,
};

#[cfg(test)]
#[path = "../tests/unit/discover_versions_tests.rs"]
mod discover_versions_tests;

#[cfg(test)]
#[path = "../tests/support/async_operation_fixtures.rs"]
mod async_operation_fixtures;

#[cfg(test)]
#[path = "../tests/unit/poll_tests.rs"]
mod poll_tests;

#[cfg(test)]
#[path = "../tests/unit/cancel_tests.rs"]
mod cancel_tests;

#[cfg(test)]
#[path = "../tests/unit/process_tests.rs"]
mod process_tests;

#[cfg(test)]
#[path = "../tests/unit/query_async_requests_tests.rs"]
mod query_async_requests_tests;

#[cfg(test)]
#[path = "../tests/unit/query_async_response_tests.rs"]
mod query_async_response_tests;
