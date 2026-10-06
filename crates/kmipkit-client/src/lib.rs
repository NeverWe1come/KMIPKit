//! Synchronous KMIP client orchestration for `KMIPKit`.
#![forbid(unsafe_code)]

mod error;

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

// T003 adds the private writer before the execute-owning feature wires its sole caller.
#[allow(dead_code)]
mod wire_encoder;

pub use error::{ClientCauseCategory, ClientError, ClientErrorCategory};
