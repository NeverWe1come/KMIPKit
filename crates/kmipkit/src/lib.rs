//! The supported high-level Rust API for the `KMIPKit` client library.
#![forbid(unsafe_code)]

pub use kmipkit_client::{ClientCauseCategory, ClientError, ClientErrorCategory};
pub use kmipkit_protocol::{
    KmipOperationResult, ProtocolCauseCategory, ProtocolError, ProtocolErrorKind, ResultMessage,
    ResultReason, ResultStatus, ResultValidationError,
};
pub use kmipkit_transport::{RequestDeliveryState, TransportCauseCategory, TransportError};
