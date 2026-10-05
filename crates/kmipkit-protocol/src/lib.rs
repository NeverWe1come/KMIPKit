//! KMIP 2.1 protocol models and validation for `KMIPKit`.
#![forbid(unsafe_code)]

mod error;
mod message;
mod result;

pub use error::{ProtocolCauseCategory, ProtocolError, ProtocolErrorKind};
pub use message::{
    MessageExtensionView, MessageValidationError, MessageValidationErrorKind, ProtocolVersion,
    RequestBatchItemView, RequestHeaderView, RequestMessage, ResponseBatchItemView,
    ResponseHeaderView, ResponseMessage,
};
pub use result::{
    KmipOperationResult, ResultMessage, ResultReason, ResultStatus, ResultValidationError,
};
