//! Typed KMIP 2.1 Re-certify request and response models.

use std::error::Error;
use std::fmt;

use kmipkit_ttlv::{Structure, StructureView, Value};

use crate::cryptographic_operation::{self as common, OwnedResponseTtlv};
use crate::{
    KmipOperationResult, ProtocolCauseCategory, ProtocolError, ProtocolErrorKind,
    ResponseBatchItemView, ResultMessage, ResultValidationError, SecretBytes, UniqueIdentifier,
};

const OPERATION: u32 = 0x0000_0007;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const CERTIFICATE_REQUEST_UNIQUE_IDENTIFIER: u32 = 0x0042_0139;
const CERTIFICATE_REQUEST_TYPE: u32 = 0x0042_0019;
const CERTIFICATE_REQUEST_VALUE: u32 = 0x0042_0140;
const OFFSET: u32 = 0x0042_0058;
const ATTRIBUTES: u32 = 0x0042_0125;
const PROTECTION_STORAGE_MASKS: u32 = 0x0042_015F;
const SUCCESS: u32 = 0;
const ID_PLACEHOLDER_ENUMERATION_VALUE: u32 = 1;

/// A KMIP 2.1 Re-certify request from §6.1.45, Table 300.
///
/// The client forwards request bytes and parameters to the server; it does not
/// create certificates, calculate certificate date links, or manage the
/// Replacement/Replaced links maintained by the server.
pub struct RecertifyRequest {
    unique_identifier: Option<UniqueIdentifier>,
    certificate_request_unique_identifier: Option<UniqueIdentifier>,
    certificate_request_type: Option<u32>,
    certificate_request_value: Option<SecretBytes>,
    offset: Option<u32>,
    attributes: Option<Structure>,
    protection_storage_masks: Option<Structure>,
}

impl RecertifyRequest {
    /// Creates a Re-certify request with all optional fields omitted.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            unique_identifier: None,
            certificate_request_unique_identifier: None,
            certificate_request_type: None,
            certificate_request_value: None,
            offset: None,
            attributes: None,
            protection_storage_masks: None,
        }
    }

    /// Selects the existing certificate to renew.
    #[must_use]
    pub fn with_unique_identifier(mut self, value: UniqueIdentifier) -> Self {
        self.unique_identifier = Some(value);
        self
    }

    /// Returns the optional explicit existing certificate identifier.
    #[must_use]
    pub const fn unique_identifier(&self) -> Option<&UniqueIdentifier> {
        self.unique_identifier.as_ref()
    }

    /// Selects an existing certificate request object by its Unique Identifier.
    #[must_use]
    pub fn with_certificate_request_unique_identifier(mut self, value: UniqueIdentifier) -> Self {
        self.certificate_request_unique_identifier = Some(value);
        self
    }

    /// Returns the optional certificate-request object identifier.
    #[must_use]
    pub const fn certificate_request_unique_identifier(&self) -> Option<&UniqueIdentifier> {
        self.certificate_request_unique_identifier.as_ref()
    }

    /// Supplies the raw Certificate Request Type Enumeration value.
    #[must_use]
    pub const fn with_certificate_request_type(mut self, value: u32) -> Self {
        self.certificate_request_type = Some(value);
        self
    }

    /// Supplies one opaque Certificate Request Value.
    #[must_use]
    pub fn with_certificate_request_value(mut self, value: SecretBytes) -> Self {
        self.certificate_request_value = Some(value);
        self
    }

    /// Supplies the server-side date difference for the replacement certificate.
    #[must_use]
    pub const fn with_offset(mut self, value: u32) -> Self {
        self.offset = Some(value);
        self
    }

    /// Supplies the ordered generic Attributes Structure.
    #[must_use]
    pub fn with_attributes(mut self, value: Structure) -> Self {
        self.attributes = Some(value);
        self
    }

    /// Supplies the ordered generic Protection Storage Masks Structure.
    #[must_use]
    pub fn with_protection_storage_masks(mut self, value: Structure) -> Self {
        self.protection_storage_masks = Some(value);
        self
    }

    /// Builds the ordered Request Payload from §6.1.45, Table 300.
    ///
    /// # Errors
    ///
    /// Returns a sanitized error if Certificate Request Value is present
    /// without its required Certificate Request Type, if Certificate Request
    /// Value is omitted without one explicit non-ID-Placeholder Unique
    /// Identifier, or if the payload cannot be represented by the generic
    /// TTLV model.
    pub fn into_ttlv_payload(self) -> Result<Structure, ProtocolError> {
        let has_request_value = self.certificate_request_value.is_some();
        if has_request_value && self.certificate_request_type.is_none() {
            return Err(invalid_request());
        }

        let has_explicit_identifier = self.unique_identifier.as_ref().is_some_and(|identifier| {
            !matches!(
                identifier,
                UniqueIdentifier::Enumeration(ID_PLACEHOLDER_ENUMERATION_VALUE)
            )
        });
        if !has_request_value && !has_explicit_identifier {
            return Err(invalid_request());
        }

        let mut payload = Structure::new();
        if let Some(identifier) = self.unique_identifier {
            common::push(
                &mut payload,
                UNIQUE_IDENTIFIER,
                common::unique_identifier_value(identifier),
            )?;
        }
        if let Some(identifier) = self.certificate_request_unique_identifier {
            common::push(
                &mut payload,
                CERTIFICATE_REQUEST_UNIQUE_IDENTIFIER,
                common::unique_identifier_value(identifier),
            )?;
        }
        if let Some(request_type) = self.certificate_request_type {
            common::push(
                &mut payload,
                CERTIFICATE_REQUEST_TYPE,
                Value::enumeration(request_type),
            )?;
        }
        if let Some(request_value) = self.certificate_request_value {
            common::push(
                &mut payload,
                CERTIFICATE_REQUEST_VALUE,
                common::secret_value(request_value),
            )?;
        }
        if let Some(offset) = self.offset {
            common::push(&mut payload, OFFSET, Value::interval(offset))?;
        }
        if let Some(attributes) = self.attributes {
            common::push(&mut payload, ATTRIBUTES, Value::structure(attributes))?;
        }
        if let Some(masks) = self.protection_storage_masks {
            common::push(
                &mut payload,
                PROTECTION_STORAGE_MASKS,
                Value::structure(masks),
            )?;
        }
        Ok(payload)
    }
}

impl Default for RecertifyRequest {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for RecertifyRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RecertifyRequest")
            .field("has_unique_identifier", &self.unique_identifier.is_some())
            .field(
                "has_certificate_request_unique_identifier",
                &self.certificate_request_unique_identifier.is_some(),
            )
            .field("certificate_request_type", &self.certificate_request_type)
            .field(
                "has_certificate_request_value",
                &self.certificate_request_value.is_some(),
            )
            .field("offset", &self.offset)
            .field("has_attributes", &self.attributes.is_some())
            .field(
                "has_protection_storage_masks",
                &self.protection_storage_masks.is_some(),
            )
            .finish()
    }
}

/// A typed Re-certify result from §6.1.45, Table 301.
///
/// The complete generic response item is retained for callback-scoped
/// inspection, including unknown or future fields.
#[derive(Debug)]
pub struct RecertifyResponse {
    result: KmipOperationResult,
    response_ttlv: OwnedResponseTtlv,
    unique_identifier: Option<UniqueIdentifier>,
}

impl RecertifyResponse {
    /// Converts one already validated Re-certify response batch item.
    ///
    /// A successful result requires exactly one well-formed Unique Identifier.
    /// Non-success results preserve their common status, reason, and message.
    ///
    /// # Errors
    ///
    /// Returns [`RecertifyError`] for a different operation, malformed result,
    /// or malformed successful payload. Error text never includes payload
    /// values.
    pub fn try_from_response_item(item: ResponseBatchItemView<'_>) -> Result<Self, RecertifyError> {
        if item.operation() != Some(OPERATION) {
            return Err(RecertifyError::UnexpectedOperation);
        }
        let status = item
            .result_status()
            .ok_or(RecertifyError::MissingResultStatus)?;
        let result_message = item.with_result_message(|text| ResultMessage::new(text.to_owned()));
        let result = KmipOperationResult::new(status, item.result_reason(), result_message)
            .map_err(RecertifyError::InvalidOperationResult)?;
        let response_ttlv = common::clone_response_item(item, "Re-certify")
            .map_err(|_| RecertifyError::MalformedResponseItem)?;

        let unique_identifier = if status.raw() == SUCCESS {
            Some(parse_success_identifier(item)?)
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

    /// Returns the replacement certificate Unique Identifier on success.
    #[must_use]
    pub const fn unique_identifier(&self) -> Option<&UniqueIdentifier> {
        self.unique_identifier.as_ref()
    }

    /// Lends the complete ordered generic response batch item.
    pub fn with_ttlv<R>(&self, callback: impl for<'a> FnOnce(StructureView<'a>) -> R) -> R {
        self.response_ttlv.with_ttlv(callback)
    }
}

/// A payload-free error converting a validated Re-certify response item.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecertifyError {
    /// The response item is not the client-to-server Re-certify operation.
    UnexpectedOperation,
    /// The validated response item did not expose a Result Status.
    MissingResultStatus,
    /// The represented KMIP result violates the shared result contract.
    InvalidOperationResult(ResultValidationError),
    /// The generic response item could not be retained.
    MalformedResponseItem,
    /// A successful response omitted or malformed its required identifier.
    MalformedSuccessPayload,
}

impl fmt::Display for RecertifyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedOperation => formatter.write_str("response item is not Re-certify"),
            Self::MissingResultStatus => formatter.write_str("Re-certify result status is missing"),
            Self::InvalidOperationResult(cause) => {
                write!(formatter, "Re-certify operation result is invalid: {cause}")
            }
            Self::MalformedResponseItem => {
                formatter.write_str("Re-certify response item is malformed")
            }
            Self::MalformedSuccessPayload => {
                formatter.write_str("successful Re-certify response payload is malformed")
            }
        }
    }
}

impl Error for RecertifyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidOperationResult(cause) => Some(cause),
            _ => None,
        }
    }
}

fn successful_identifier(payload: &StructureView<'_>) -> Result<UniqueIdentifier, RecertifyError> {
    super::lifecycle::successful_response_identifier(payload)
        .ok_or(RecertifyError::MalformedSuccessPayload)
}

fn parse_success_identifier(
    item: ResponseBatchItemView<'_>,
) -> Result<UniqueIdentifier, RecertifyError> {
    item.with_response_payload(|payload| successful_identifier(&payload))
        .ok_or(RecertifyError::MalformedSuccessPayload)?
}

fn invalid_request() -> ProtocolError {
    ProtocolError::categorized(
        ProtocolErrorKind::InvalidInput,
        ProtocolCauseCategory::InvalidValue,
    )
}
