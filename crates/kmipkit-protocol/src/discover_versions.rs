//! Typed client-to-server Discover Versions payloads.
//!
//! The request advertises client-supported protocol versions. By default, it
//! advertises KMIP 2.1 preserving 1.0 zero-breakage behavior. Response conversion
//! borrows the validated KMIPKIT-0006 message model, leaving its ordered generic
//! TTLV tree available to the caller.

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

static DEFAULT_OFFER: [ProtocolVersion; 1] = [ProtocolVersion::from_raw(2, 1)];

/// A client-to-server Discover Versions request.
///
/// By default, its payload advertises exactly Protocol Version `(2, 1)` as the sole
/// preference, maintaining full backwards compatibility. Callers may configure multiple
/// preferences via [`DiscoverVersionsRequest::with_versions`].
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DiscoverVersionsRequest {
    versions: Vec<ProtocolVersion>,
}

impl DiscoverVersionsRequest {
    /// Creates a request that offers KMIP 2.1 only.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            versions: Vec::new(),
        }
    }

    /// Creates a request that offers the specified ordered protocol versions.
    ///
    /// # Errors
    ///
    /// Returns [`DiscoverVersionsError::InvalidOfferedVersions`] if `versions` is empty
    /// or contains uncatalogued protocol versions.
    pub fn with_versions(versions: Vec<ProtocolVersion>) -> Result<Self, DiscoverVersionsError> {
        if versions.is_empty() {
            return Err(DiscoverVersionsError::InvalidOfferedVersions);
        }
        for v in &versions {
            if !v.is_catalogued() {
                return Err(DiscoverVersionsError::InvalidOfferedVersions);
            }
        }
        Ok(Self { versions })
    }

    /// Returns the client version preference list.
    #[must_use]
    pub fn protocol_versions(&self) -> &[ProtocolVersion] {
        if self.versions.is_empty() {
            &DEFAULT_OFFER
        } else {
            &self.versions
        }
    }

    /// Builds the ordered generic Structure for the Discover Versions request
    /// payload.
    ///
    /// # Errors
    ///
    /// Returns a sanitized protocol error if the pinned KMIP tags or generic
    /// Structure model reject this internally defined payload.
    pub fn to_ttlv_payload(&self) -> Result<Structure, ProtocolError> {
        let mut payload = Structure::new();
        for &version in self.protocol_versions() {
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

            payload
                .try_push(item(PROTOCOL_VERSION, Value::structure(version_fields))?)
                .map_err(model_error)?;
        }
        Ok(payload)
    }
}

/// A typed result for one client-to-server Discover Versions response item.
///
/// Successful responses expose the offered intersection as ordered
/// [`ProtocolVersion`] values. Non-success KMIP results retain the existing
/// [`KmipOperationResult`] and have no successful version list.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoverVersionsResponse {
    result: KmipOperationResult,
    supported_versions: Option<Vec<ProtocolVersion>>,
}

impl DiscoverVersionsResponse {
    /// Converts one already validated KMIPKIT-0006 response batch item against the default offer (KMIP 2.1).
    ///
    /// # Errors
    ///
    /// Returns [`DiscoverVersionsError`] for an invalid represented result,
    /// malformed Protocol Version Structure, or a version the client did not
    /// offer.
    pub fn try_from_response_item(
        item: ResponseBatchItemView<'_>,
    ) -> Result<Self, DiscoverVersionsError> {
        Self::try_from_response_item_for_offer(item, &DEFAULT_OFFER)
    }

    /// Converts one response batch item validating returned versions against the caller-provided offered list.
    ///
    /// # Errors
    ///
    /// Returns [`DiscoverVersionsError`] for an invalid represented result,
    /// malformed Protocol Version Structure, or a version not in `offered`.
    pub fn try_from_response_item_for_offer(
        item: ResponseBatchItemView<'_>,
        offered: &[ProtocolVersion],
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
            .with_response_payload(|payload| parse_response_versions(&payload, offered))
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

/// A payload-free error converting a validated Discover Versions response or building a request.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiscoverVersionsError {
    /// A response item did not identify the client-to-server Discover Versions operation.
    UnexpectedOperation,
    /// A validated response item did not expose a Result Status.
    MissingResultStatus,
    /// The response Result Status, Reason, or Message combination is invalid.
    InvalidOperationResult(ResultValidationError),
    /// A successful response omitted its required Response Payload Structure.
    MissingSuccessPayload,
    /// A Protocol Version field did not match §9.16, Table 421.
    MalformedProtocolVersion,
    /// The server returned a version not included in the client's offer.
    UnofferedProtocolVersion,
    /// The offered versions list is empty or contains uncatalogued protocol versions.
    InvalidOfferedVersions,
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
            Self::InvalidOfferedVersions => {
                "Discover Versions request offered versions list is invalid"
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
    offered: &[ProtocolVersion],
) -> Result<Vec<ProtocolVersion>, DiscoverVersionsError> {
    let mut versions = Vec::new();

    for child in payload.children() {
        if child.tag().raw() != PROTOCOL_VERSION {
            continue;
        }

        let version = child
            .with_value(|value| match value {
                ValueView::Structure(version) => Some(parse_protocol_version(&version)),
                _ => None,
            })
            .ok_or(DiscoverVersionsError::MalformedProtocolVersion)??;
        if !offered.contains(&version) {
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
                    ValueView::Integer(major) => Some(*major),
                    _ => None,
                });
                if major.is_none() {
                    return Err(DiscoverVersionsError::MalformedProtocolVersion);
                }
            }
            PROTOCOL_VERSION_MINOR => {
                if minor.is_some() || major.is_none() {
                    return Err(DiscoverVersionsError::MalformedProtocolVersion);
                }
                minor = child.with_value(|value| match value {
                    ValueView::Integer(minor) => Some(*minor),
                    _ => None,
                });
                if minor.is_none() {
                    return Err(DiscoverVersionsError::MalformedProtocolVersion);
                }
            }
            _ => return Err(DiscoverVersionsError::MalformedProtocolVersion),
        }
    }

    match (major, minor) {
        (Some(major), Some(minor)) => Ok(ProtocolVersion::from_raw(major, minor)),
        _ => Err(DiscoverVersionsError::MalformedProtocolVersion),
    }
}

fn tag(raw: u32) -> Result<Tag, ProtocolError> {
    RawTag::new(raw)
        .and_then(|t| t.try_checked())
        .map_err(model_error)
}

fn item(raw: u32, value: Value) -> Result<Item, ProtocolError> {
    Item::new(tag(raw)?, value).map_err(model_error)
}

fn model_error(error: ModelError) -> ProtocolError {
    ProtocolError::new(
        ProtocolErrorKind::InvalidValue,
        ProtocolCauseCategory::InvalidValue,
        error,
    )
}
