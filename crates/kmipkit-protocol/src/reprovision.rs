//! Typed KMIP 2.1 Re-Provision request and response models.

use std::error::Error;
use std::fmt;

use kmipkit_ttlv::{Item, ModelError, RawTag, Structure, StructureView, Tag, Value};

use crate::cryptographic_operation::{self as common, OwnedResponseTtlv};
use crate::{
    KmipOperationResult, ProtocolCauseCategory, ProtocolError, ProtocolErrorKind,
    ResponseBatchItemView, ResultMessage, ResultValidationError, UniqueIdentifier,
};

const OPERATION: u32 = 0x0000_0035;
const CERTIFICATE: u32 = 0x0042_0013;
const CERTIFICATE_REQUEST: u32 = 0x0042_0018;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const SUCCESS: u32 = 0;

/// A KMIP 2.1 Re-Provision request from §6.1.48, Table 313.
///
/// Supply either or both optional certificate objects, or omit both when the
/// server creates the new credential. KMIP v2.1 does not prohibit including
/// both Table 313 fields in one request. This client forwards supplied KMIP
/// object structures without parsing them or changing local credential state.
pub struct ReProvisionRequest {
    certificate_request: Option<Structure>,
    certificate: Option<Structure>,
}

impl ReProvisionRequest {
    /// Creates a parameterless request for server-created credentials.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            certificate_request: None,
            certificate: None,
        }
    }

    /// Supplies the Certificate Request object to be signed by the server.
    #[must_use]
    pub fn with_certificate_request(mut self, value: Structure) -> Self {
        self.certificate_request = Some(value);
        self
    }

    /// Supplies a Certificate object to associate with the client.
    #[must_use]
    pub fn with_certificate(mut self, value: Structure) -> Self {
        self.certificate = Some(value);
        self
    }

    /// Builds the ordered Request Payload from §6.1.48, Table 313.
    ///
    /// # Errors
    ///
    /// Returns a sanitized error if a generic TTLV structure cannot be
    /// represented.
    pub fn into_ttlv_payload(self) -> Result<Structure, ProtocolError> {
        let mut payload = Structure::new();
        if let Some(certificate_request) = self.certificate_request {
            push_structure(&mut payload, CERTIFICATE_REQUEST, certificate_request)?;
        }
        if let Some(certificate) = self.certificate {
            push_structure(&mut payload, CERTIFICATE, certificate)?;
        }
        Ok(payload)
    }
}

impl Default for ReProvisionRequest {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for ReProvisionRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ReProvisionRequest")
            .field(
                "has_certificate_request",
                &self.certificate_request.is_some(),
            )
            .field("has_certificate", &self.certificate.is_some())
            .finish()
    }
}

/// A typed Re-Provision result from §6.1.48, Table 314.
///
/// The full generic response item remains available for callback-scoped
/// inspection, including unknown fields. The Table 314 Unique Identifier is
/// optional and is not synthesized when absent.
#[derive(Debug)]
pub struct ReProvisionResponse {
    result: KmipOperationResult,
    response_ttlv: OwnedResponseTtlv,
    unique_identifier: Option<UniqueIdentifier>,
}

impl ReProvisionResponse {
    /// Converts one already validated Re-Provision response batch item.
    ///
    /// # Errors
    ///
    /// Returns [`ReProvisionError`] for another operation, an invalid common
    /// result, an unretainable response, or a malformed present identifier.
    pub fn try_from_response_item(
        item: ResponseBatchItemView<'_>,
    ) -> Result<Self, ReProvisionError> {
        if item.operation() != Some(OPERATION) {
            return Err(ReProvisionError::UnexpectedOperation);
        }
        let status = item
            .result_status()
            .ok_or(ReProvisionError::MissingResultStatus)?;
        let result_message = item.with_result_message(|text| ResultMessage::new(text.to_owned()));
        let result = KmipOperationResult::new(status, item.result_reason(), result_message)
            .map_err(ReProvisionError::InvalidOperationResult)?;
        let response_ttlv = common::clone_response_item(item, "Re-Provision")
            .map_err(|_| ReProvisionError::MalformedResponseItem)?;

        let unique_identifier = if status.raw() == SUCCESS {
            item.with_response_payload(|payload| parse_success_identifier(&payload))
                .transpose()?
                .flatten()
        } else {
            None
        };

        Ok(Self {
            result,
            response_ttlv,
            unique_identifier,
        })
    }

    /// Returns the exact KMIP operation result.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }

    /// Returns the optional certificate or private-key identifier.
    #[must_use]
    pub const fn unique_identifier(&self) -> Option<&UniqueIdentifier> {
        self.unique_identifier.as_ref()
    }

    /// Lends the complete ordered generic response batch item.
    pub fn with_ttlv<R>(&self, callback: impl for<'a> FnOnce(StructureView<'a>) -> R) -> R {
        self.response_ttlv.with_ttlv(callback)
    }
}

/// A payload-free error converting a validated Re-Provision response item.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReProvisionError {
    /// The response item is not the client-to-server Re-Provision operation.
    UnexpectedOperation,
    /// The validated response item did not expose a Result Status.
    MissingResultStatus,
    /// The represented KMIP result violates the shared result contract.
    InvalidOperationResult(ResultValidationError),
    /// The generic response item could not be retained.
    MalformedResponseItem,
    /// The success payload contains a malformed or duplicate identifier.
    MalformedSuccessPayload,
}

impl fmt::Display for ReProvisionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedOperation => formatter.write_str("response item is not Re-Provision"),
            Self::MissingResultStatus => {
                formatter.write_str("Re-Provision result status is missing")
            }
            Self::InvalidOperationResult(cause) => {
                write!(
                    formatter,
                    "Re-Provision operation result is invalid: {cause}"
                )
            }
            Self::MalformedResponseItem => {
                formatter.write_str("Re-Provision response item is malformed")
            }
            Self::MalformedSuccessPayload => {
                formatter.write_str("successful Re-Provision payload is malformed")
            }
        }
    }
}

impl Error for ReProvisionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidOperationResult(cause) => Some(cause),
            _ => None,
        }
    }
}

fn parse_success_identifier(
    payload: &StructureView<'_>,
) -> Result<Option<UniqueIdentifier>, ReProvisionError> {
    let mut identifier = None;
    for field in payload.children() {
        if field.tag().raw() == UNIQUE_IDENTIFIER {
            if identifier.is_some() {
                return Err(ReProvisionError::MalformedSuccessPayload);
            }
            identifier = crate::lifecycle::parse_unique_identifier(field);
            if identifier.is_none() {
                return Err(ReProvisionError::MalformedSuccessPayload);
            }
        }
    }
    Ok(identifier)
}

fn push_structure(
    payload: &mut Structure,
    raw_tag: u32,
    structure: Structure,
) -> Result<(), ProtocolError> {
    let item = Item::new(tag(raw_tag)?, Value::structure(structure)).map_err(model_error)?;
    payload.try_push(item).map_err(model_error)
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
