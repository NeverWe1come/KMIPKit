//! Synchronous KMIP client orchestration for `KMIPKit`.
#![forbid(unsafe_code)]

mod error;

#[cfg(test)]
mod wire_encoder;

pub use error::{ClientCauseCategory, ClientError, ClientErrorCategory};
