//! Shared sanitized mechanics for OASIS KMIP 2.1 policy payloads.
use crate::{
    KmipOperationResult, ProtocolCauseCategory, ProtocolError, ProtocolErrorKind,
    ResponseBatchItemView, ResultMessage, ResultValidationError,
};
use kmipkit_ttlv::{Item, ModelError, RawTag, Structure, StructureView, Value};
use std::{error::Error, fmt};

/// A payload-free policy operation conversion failure.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PolicyControlsError {
    /// The response identifies another operation.
    UnexpectedOperation,
    /// Required common status is absent.
    MissingResultStatus,
    /// Common result metadata is inconsistent.
    InvalidOperationResult(ResultValidationError),
    /// A successful payload is absent, incomplete, duplicated or mistyped.
    MalformedSuccessPayload,
}
impl fmt::Display for PolicyControlsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedOperation => f.write_str("unexpected policy operation"),
            Self::MissingResultStatus => f.write_str("policy result status missing"),
            Self::InvalidOperationResult(e) => write!(f, "invalid policy result: {e}"),
            Self::MalformedSuccessPayload => f.write_str("malformed policy success payload"),
        }
    }
}
impl Error for PolicyControlsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidOperationResult(e) => Some(e),
            _ => None,
        }
    }
}
pub(crate) fn result(
    item: ResponseBatchItemView<'_>,
    operation: u32,
) -> Result<KmipOperationResult, PolicyControlsError> {
    if item.operation() != Some(operation) {
        return Err(PolicyControlsError::UnexpectedOperation);
    }
    let status = item
        .result_status()
        .ok_or(PolicyControlsError::MissingResultStatus)?;
    let message = item.with_result_message(|v| ResultMessage::new(v.to_owned()));
    KmipOperationResult::new(status, item.result_reason(), message)
        .map_err(PolicyControlsError::InvalidOperationResult)
}
pub(crate) fn push(payload: &mut Structure, tag: u32, value: Value) -> Result<(), ProtocolError> {
    let tag = RawTag::new(tag)
        .and_then(|raw| raw.try_checked())
        .map_err(model_error)?;
    payload
        .try_push(Item::new(tag, value).map_err(model_error)?)
        .map_err(model_error)
}
pub(crate) fn model_error(e: ModelError) -> ProtocolError {
    ProtocolError::new(
        ProtocolErrorKind::InvalidValue,
        ProtocolCauseCategory::InvalidValue,
        e,
    )
}
pub(crate) fn required<'a>(payload: &'a StructureView<'_>, tag: u32) -> Option<&'a Item> {
    let mut fields = payload.children().iter().filter(|f| f.tag().raw() == tag);
    let field = fields.next()?;
    if fields.next().is_some() {
        return None;
    }
    Some(field)
}
