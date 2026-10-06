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
mod execution_tests;

#[cfg(test)]
mod option_tests;

#[cfg(test)]
mod response_boundary_tests;

#[cfg(test)]
mod request_time_stamp_tests;

#[cfg(test)]
mod codec_limits_identity_tests;

#[cfg(test)]
mod execute_lifecycle_tests;

#[cfg(test)]
mod execute_test_support;

#[cfg(test)]
mod execute_boundary_tests;

#[cfg(test)]
mod transport_contract_tests;

pub use error::{ClientCauseCategory, ClientError, ClientErrorCategory};
pub use execute::{
    Client, ClientBatch, ClientBatchItem, ClientBatchItemResponse, ClientBatchOutcome,
    ClientBatchResponse, ClientMessageExtension, ClientRequest, PendingOutcome,
};
