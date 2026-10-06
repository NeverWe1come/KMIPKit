//! Typed client-to-server Discover Versions payloads.
//!
//! The request advertises KMIP 2.1, the only protocol version supported by
//! `KMIPKit` 1.0. Response conversion borrows the validated KMIPKIT-0006 message
//! model, leaving its ordered generic TTLV tree available to the caller.

use std::error::Error;
use std::fmt;

use kmipkit_ttlv::{Item, ModelError, RawTag, Structure, StructureView, Tag, Value, ValueView};

use crate::{
    KmipOperationResult, ProtocolCauseCategory, ProtocolError, ProtocolErrorKind, ProtocolVersion,
    ResponseBatchItemView, ResultMessage, ResultReason, ResultStatus, ResultValidationError,
};

const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
const DISCOVER_VERSIONS_OPERATION: u32 = 0x0000_001E;
const SUCCESS: u32 = 0;

/// A client-to-server Discover Versions request for `KMIPKit` 1.0.
///
/// The request has no caller-supplied TTLV or version-list input. Its payload
/// always advertises exactly Protocol Version `(2, 1)` as the sole preference,
/// as assigned by ADR-0002 and OASIS KMIP Specification v2.1 §6.1.16, Table
/// 211.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DiscoverVersionsRequest {
    _private: (),
}

impl DiscoverVersionsRequest {
    /// Creates a request that offers KMIP 2.1 only.
    #[must_use]
    pub const fn new() -> Self {
        Self { _private: () }
    }

    /// Returns the fixed client version preference list.
    #[must_use]
    pub const fn protocol_versions(&self) -> [ProtocolVersion; 1] {
        [ProtocolVersion::from_raw(2, 1)]
    }

    /// Builds the ordered generic Structure for the Discover Versions request
    /// payload.
    ///
    /// This produces the typed request's payload fields; the common KMIPKIT-0006
    /// message model owns the enclosing Request Payload and Batch Item.
    ///
    /// # Errors
    ///
    /// Returns a sanitized protocol error if the pinned KMIP tags or generic
    /// Structure model reject this internally defined payload.
    pub fn to_ttlv_payload(&self) -> Result<Structure, ProtocolError> {
        let version = self.protocol_versions()[0];
        let mut version_fields = Structure::new();
        version_fields
            .try_push(item(
                PROTOCOL_VERSION_MAJOR,
                Value::integer(version.major()),
            )?)
            .map_err(model_error)?;
        version_fields
            .try_push(item(
                PROTOCOL_VERSION_MINOR,
                Value::integer(version.minor()),
            )?)
            .map_err(model_error)?;

        let mut payload = Structure::new();
        payload
            .try_push(item(PROTOCOL_VERSION, Value::structure(version_fields))?)
            .map_err(model_error)?;
        Ok(payload)
    }
}

/// A typed result for one client-to-server Discover Versions response item.
///
/// Successful responses expose the offered intersection as ordered
/// [`ProtocolVersion`] values. Non-success KMIP results retain the existing
/// [`KmipOperationResult`] and have no successful version list. The validated
/// source [`ResponseMessage`](crate::ResponseMessage) remains the owner of the
/// complete generic tree, including fields not modeled here.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoverVersionsResponse {
    result: KmipOperationResult,
    supported_versions: Option<Vec<ProtocolVersion>>,
}

impl DiscoverVersionsResponse {
    /// Converts one already validated KMIPKIT-0006 response batch item.
    ///
    /// The item is read through its borrowed view; this conversion does not
    /// consume, normalize, or duplicate its generic TTLV tree. Successful
    /// payloads may contain an empty version list. Every listed version must be
    /// among the `(2, 1)` value offered by [`DiscoverVersionsRequest`], and
    /// source order and repeated values are retained as allowed by §6.1.16,
    /// Table 212.
    ///
    /// # Errors
    ///
    /// Returns [`DiscoverVersionsError`] for an invalid represented result,
    /// malformed Protocol Version Structure, or a version the client did not
    /// offer.
    pub fn try_from_response_item(
        item: ResponseBatchItemView<'_>,
    ) -> Result<Self, DiscoverVersionsError> {
        if item.operation() != Some(DISCOVER_VERSIONS_OPERATION) {
            return Err(DiscoverVersionsError::UnexpectedOperation);
        }
        let status = item
            .result_status()
            .ok_or(DiscoverVersionsError::MissingResultStatus)?;
        let result_message = item.with_result_message(|text| ResultMessage::new(text.to_owned()));
        let result = operation_result(status, item.result_reason(), result_message)?;

        if status.raw() != SUCCESS {
            return Ok(Self {
                result,
                supported_versions: None,
            });
        }

        let versions = item
            .with_response_payload(|payload| parse_response_versions(&payload))
            .ok_or(DiscoverVersionsError::MissingSuccessPayload)??;
        Ok(Self {
            result,
            supported_versions: Some(versions),
        })
    }

    /// Returns the exact KMIP operation result, including its raw status and
    /// optional reason and message.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }

    /// Returns the supported-version list for a successful result.
    ///
    /// Non-success KMIP results return `None`; an empty successful server list
    /// returns `Some(&[])`.
    #[must_use]
    pub fn supported_versions(&self) -> Option<&[ProtocolVersion]> {
        self.supported_versions.as_deref()
    }
}

/// A payload-free error converting a validated Discover Versions response.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiscoverVersionsError {
    /// A response item did not identify the client-to-server Discover Versions operation.
    UnexpectedOperation,
    /// A validated response item did not expose a Result Status.
    MissingResultStatus,
    /// The response Result Status, Reason, or Message combination is invalid.
    ///
    /// The typed [`ResultValidationError`] is retained as the error source.
    InvalidOperationResult(ResultValidationError),
    /// A successful response omitted its required Response Payload Structure.
    MissingSuccessPayload,
    /// A Protocol Version field did not match §9.16, Table 421.
    MalformedProtocolVersion,
    /// The server returned a version not included in the client's offer.
    UnofferedProtocolVersion,
}

impl fmt::Display for DiscoverVersionsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::UnexpectedOperation => "response item is not Discover Versions",
            Self::MissingResultStatus => "Discover Versions result status is missing",
            Self::InvalidOperationResult(cause) => {
                return write!(
                    formatter,
                    "Discover Versions operation result is invalid: {cause}"
                );
            }
            Self::MissingSuccessPayload => "successful Discover Versions response has no payload",
            Self::MalformedProtocolVersion => {
                "Discover Versions response has a malformed Protocol Version"
            }
            Self::UnofferedProtocolVersion => {
                "Discover Versions response includes a version the client did not offer"
            }
        };
        formatter.write_str(message)
    }
}

impl Error for DiscoverVersionsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidOperationResult(cause) => Some(cause),
            _ => None,
        }
    }
}

fn operation_result(
    status: ResultStatus,
    reason: Option<ResultReason>,
    message: Option<ResultMessage>,
) -> Result<KmipOperationResult, DiscoverVersionsError> {
    KmipOperationResult::new(status, reason, message)
        .map_err(DiscoverVersionsError::InvalidOperationResult)
}

fn parse_response_versions(
    payload: &StructureView<'_>,
) -> Result<Vec<ProtocolVersion>, DiscoverVersionsError> {
    let offered = ProtocolVersion::from_raw(2, 1);
    let mut versions = Vec::new();

    for child in payload.children() {
        if child.tag().raw() != PROTOCOL_VERSION {
            // KMIPKIT-0006 continues to own the complete generic tree. Unknown
            // fields are left untouched there and are not a version entry.
            continue;
        }

        let version = child
            .with_value(|value| match value {
                ValueView::Structure(version) => Some(parse_protocol_version(&version)),
                _ => None,
            })
            .ok_or(DiscoverVersionsError::MalformedProtocolVersion)??;
        if version != offered {
            return Err(DiscoverVersionsError::UnofferedProtocolVersion);
        }
        versions.push(version);
    }

    Ok(versions)
}

fn parse_protocol_version(
    version: &StructureView<'_>,
) -> Result<ProtocolVersion, DiscoverVersionsError> {
    let mut major = None;
    let mut minor = None;

    for child in version.children() {
        match child.tag().raw() {
            PROTOCOL_VERSION_MAJOR => {
                if major.is_some() || minor.is_some() {
                    return Err(DiscoverVersionsError::MalformedProtocolVersion);
                }
                major = child.with_value(|value| match value {
                    ValueView::Integer(value) => Some(*value),
                    _ => None,
                });
                if major.is_none() {
                    return Err(DiscoverVersionsError::MalformedProtocolVersion);
                }
            }
            PROTOCOL_VERSION_MINOR => {
                if minor.is_some() {
                    return Err(DiscoverVersionsError::MalformedProtocolVersion);
                }
                minor = child.with_value(|value| match value {
                    ValueView::Integer(value) => Some(*value),
                    _ => None,
                });
                if minor.is_none() {
                    return Err(DiscoverVersionsError::MalformedProtocolVersion);
                }
            }
            _ => {}
        }
    }

    match (major, minor) {
        (Some(major), Some(minor)) => Ok(ProtocolVersion::from_raw(major, minor)),
        _ => Err(DiscoverVersionsError::MalformedProtocolVersion),
    }
}

fn item(raw_tag: u32, value: Value) -> Result<Item, ProtocolError> {
    Item::new(tag(raw_tag)?, value).map_err(model_error)
}

fn tag(raw_tag: u32) -> Result<Tag, ProtocolError> {
    RawTag::new(raw_tag)
        .and_then(|raw| raw.try_checked())
        .map_err(model_error)
}

fn model_error(error: ModelError) -> ProtocolError {
    ProtocolError::new(
        ProtocolErrorKind::InvalidValue,
        ProtocolCauseCategory::InvalidValue,
        error,
    )
}

#[cfg(test)]
mod result_error_tests {
    use std::error::Error;

    use crate::{ResultStatus, ResultValidationError};

    use super::{DiscoverVersionsError, operation_result};

    #[test]
    fn invalid_operation_result_error_retains_its_typed_validation_cause() {
        let invalid_results = [
            (
                ResultStatus::from_raw(1),
                None,
                ResultValidationError::FailureRequiresReason,
            ),
            (
                ResultStatus::from_raw(0),
                Some(crate::ResultReason::from_raw(1)),
                ResultValidationError::SuccessForbidsReason,
            ),
        ];

        for (status, reason, expected) in invalid_results {
            let error = operation_result(status, reason, None)
                .expect_err("invalid result combinations return a typed conversion error");
            assert_eq!(
                error,
                DiscoverVersionsError::InvalidOperationResult(expected)
            );
            let source = Error::source(&error)
                .expect("typed Discover Versions errors retain safe result-validation causes");
            assert_eq!(
                source.downcast_ref::<ResultValidationError>(),
                Some(&expected)
            );
        }
    }
}
