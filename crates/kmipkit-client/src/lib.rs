//! Synchronous KMIP client orchestration for `KMIPKit`.
#![forbid(unsafe_code)]

mod error;

// T003 adds the private writer before the execute-owning feature wires its sole caller.
#[allow(dead_code)]
mod wire_encoder;

pub use error::{ClientCauseCategory, ClientError, ClientErrorCategory};
