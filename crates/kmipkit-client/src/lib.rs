//! Synchronous typed KMIP client execution foundation for `KMIPKit`.
//!
//! The typed request enum supports explicit Discover Versions, Create,
//! Create Key Pair, and Create Split Key operations. The synchronous
//! [`Client`] is constructed from an immutable
//! client configuration and validated production transport configuration;
//! callers cannot inject an arbitrary transport or submit raw KMIP bytes.
//! Timeout overrides are available through the options-bearing typed methods.
//!
//! The following example prepares a typed batch without sending a request:
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
#![doc = include_str!("../../../docs/user-guide/en/production-transports.md")]
#![doc = include_str!("../../../docs/user-guide/es/transportes-produccion.md")]
#![forbid(unsafe_code)]

mod error;
pub mod extension_registry;

#[cfg(test)]
#[path = "../tests/fixtures/extensions/extension_fixtures.generated.rs"]
pub(crate) mod extension_fixtures;

#[cfg(test)]
#[path = "../tests/unit/extension_registry_test_support.rs"]
mod extension_registry_test_support;

#[cfg(test)]
#[path = "../tests/unit/extension_execution.rs"]
mod extension_execution_tests;

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
#[path = "../tests/unit/create_execution_tests.rs"]
mod create_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/create_key_pair_execution_tests.rs"]
mod create_key_pair_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/create_split_key_execution_tests.rs"]
mod create_split_key_execution_tests;

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
#[path = "../tests/unit/attestation_indicator_tests.rs"]
mod attestation_indicator_tests;

#[cfg(test)]
#[path = "../tests/unit/attribute_read_execution_tests.rs"]
mod attribute_read_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/transport_contract_tests.rs"]
mod transport_contract_tests;

pub use error::{ClientCauseCategory, ClientError, ClientErrorCategory};
pub use execute::{
    Client, ClientBatch, ClientBatchItem, ClientBatchItemResponse, ClientBatchOutcome,
    ClientBatchResponse, ClientMessageExtension, ClientOperation, ClientOperationOutcome,
    ClientRequest, PendingOutcome,
};
pub use kmipkit_transport::RequestOptions;
