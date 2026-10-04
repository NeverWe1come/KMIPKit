//! KMIP 2.1 protocol models and validation for `KMIPKit`.
#![forbid(unsafe_code)]

mod error;
mod result;

pub use error::{ProtocolCauseCategory, ProtocolError, ProtocolErrorKind};
pub use result::{
    KmipOperationResult, ResultMessage, ResultReason, ResultStatus, ResultValidationError,
};
