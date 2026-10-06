//! Synchronous typed KMIP client execution foundation for `KMIPKit`.
//!
//! The current request enum supports an explicit Discover Versions operation
//! only. The client does not perform implicit version discovery, and this
//! feature provides no production [`Client`] constructor or live TLS/HTTPS
//! transport. Applications can prepare typed request batches, but cannot yet
//! construct a usable network client from this crate.
//!
//! The following example prepares a typed batch without constructing a client
//! or sending a request:
//!
//! ```
//! use kmipkit_client::{ClientBatch, ClientBatchItem, ClientRequest};
//!
//! let request = ClientRequest::discover_versions();
//! let batch = ClientBatch::new(ClientBatchItem::new(request));
//! assert_eq!(batch.items().len(), 1);
//! ```
//!
//! See the repository guide at `docs/user-guide/en/client-execution.md` for
//! resource limits, redaction, the low-level transport contract, and current
//! implementation boundaries.
#![forbid(unsafe_code)]

mod error;

mod execute;

#[cfg(test)]
#[path = "../tests/unit/execution_tests.rs"]
mod execution_tests;

#[cfg(test)]
#[path = "../tests/unit/option_tests.rs"]
mod option_tests;

#[cfg(test)]
#[path = "../tests/unit/response_boundary_tests.rs"]
mod response_boundary_tests;

#[cfg(test)]
#[path = "../tests/unit/request_time_stamp_tests.rs"]
mod request_time_stamp_tests;

#[cfg(test)]
#[path = "../tests/unit/codec_limits_identity_tests.rs"]
mod codec_limits_identity_tests;

#[cfg(test)]
#[path = "../tests/unit/execute_lifecycle_tests.rs"]
mod execute_lifecycle_tests;

#[cfg(test)]
#[path = "../tests/unit/execute_test_support.rs"]
mod execute_test_support;

#[cfg(test)]
#[path = "../tests/unit/asynchronous_execution_test_support.rs"]
mod asynchronous_execution_test_support;

#[cfg(test)]
#[path = "../tests/unit/poll_execution_tests.rs"]
mod poll_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/cancel_execution_tests.rs"]
mod cancel_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/process_execution_tests.rs"]
mod process_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/query_async_execution_tests.rs"]
mod query_async_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/execute_boundary_tests.rs"]
mod execute_boundary_tests;

#[cfg(test)]
#[path = "../tests/unit/transport_contract_tests.rs"]
mod transport_contract_tests;

pub use error::{ClientCauseCategory, ClientError, ClientErrorCategory};
pub use execute::{
    Client, ClientBatch, ClientBatchItem, ClientBatchItemResponse, ClientBatchOutcome,
    ClientBatchResponse, ClientMessageExtension, ClientRequest, PendingOutcome,
};
