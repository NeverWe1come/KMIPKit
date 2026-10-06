//! Synchronous KMIP client orchestration for `KMIPKit`.
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
