//! Typed KMIP 2.1 Certify request and response models.

use std::error::Error;
use std::fmt;

use kmipkit_ttlv::{Structure, StructureView, Value};

use crate::cryptographic_operation::{self as common, OwnedResponseTtlv};
use crate::{
    KmipOperationResult, ProtocolCauseCategory, ProtocolError, ProtocolErrorKind,
    ResponseBatchItemView, ResultMessage, ResultValidationError, SecretBytes, UniqueIdentifier,
};

const OPERATION: u32 = 0x0000_0006;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const CERTIFICATE_REQUEST_TYPE: u32 = 0x0042_0019;
const ATTRIBUTES: u32 = 0x0042_0125;
const CERTIFICATE_TYPE: u32 = 0x0042_001D;
const CERTIFICATE_REQUEST_VALUE: u32 = 0x0042_0140;
const PROTECTION_STORAGE_MASKS: u32 = 0x0042_015F;
const SUCCESS: u32 = 0;

/// A KMIP 2.1 Certify request from §6.1.6, Table 179.
///
/// The request value is one opaque certificate request. `KMIPKit` neither
/// parses nor creates certificates or certificate requests locally.
pub struct CertifyRequest {
    unique_identifier: Option<UniqueIdentifier>,
    certificate_request_type: Option<u32>,
    certificate_request_value: Option<SecretBytes>,
    attributes: Option<Structure>,
    protection_storage_masks: Option<Structure>,
}

impl CertifyRequest {
    /// Creates a Certify request with all optional fields omitted.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            unique_identifier: None,
            certificate_request_type: None,
            certificate_request_value: None,
            attributes: None,
            protection_storage_masks: None,
        }
    }

    /// Supplies the public key or certificate-request Unique Identifier.
    #[must_use]
    pub fn with_unique_identifier(mut self, value: UniqueIdentifier) -> Self {
        self.unique_identifier = Some(value);
        self
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

    /// Builds the ordered Request Payload from §6.1.6, Table 179.
    ///
    /// # Errors
    ///
    /// Returns a sanitized error when local conditional requirements are not
    /// met or a generic Structure cannot be represented.
    pub fn into_ttlv_payload(self) -> Result<Structure, ProtocolError> {
        let has_request_value = self.certificate_request_value.is_some();
        if has_request_value && self.certificate_request_type.is_none() {
            return Err(invalid_request());
        }
        if !has_request_value && self.unique_identifier.is_none() {
            return Err(invalid_request());
        }
        if !has_request_value
            && self.certificate_request_type.is_none()
            && !self
                .attributes
                .as_ref()
                .is_some_and(contains_certificate_type)
        {
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

impl Default for CertifyRequest {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for CertifyRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CertifyRequest")
            .field("has_unique_identifier", &self.unique_identifier.is_some())
            .field("certificate_request_type", &self.certificate_request_type)
            .field(
                "has_certificate_request_value",
                &self.certificate_request_value.is_some(),
            )
            .field("has_attributes", &self.attributes.is_some())
            .field(
                "has_protection_storage_masks",
                &self.protection_storage_masks.is_some(),
            )
            .finish()
    }
}

/// A typed Certify result from §6.1.6, Table 180.
///
/// The complete generic response item is retained for callback-scoped
/// inspection, including unknown or future fields.
#[derive(Debug)]
pub struct CertifyResponse {
    result: KmipOperationResult,
    response_ttlv: Option<OwnedResponseTtlv>,
    unique_identifier: Option<UniqueIdentifier>,
}

impl CertifyResponse {
    /// Converts one already validated Certify response batch item.
    ///
    /// A successful result requires exactly one well-formed Unique Identifier.
    /// Non-success results preserve their common status, reason, and message.
    ///
    /// # Errors
    ///
    /// Returns [`CertifyError`] for a different operation, malformed result,
    /// or malformed successful payload. Error text never includes payload
    /// values.
    pub fn try_from_response_item(item: ResponseBatchItemView<'_>) -> Result<Self, CertifyError> {
        if item.operation() != Some(OPERATION) {
            return Err(CertifyError::UnexpectedOperation);
        }
        let status = item
            .result_status()
            .ok_or(CertifyError::MissingResultStatus)?;
        let result_message = item.with_result_message(|text| ResultMessage::new(text.to_owned()));
        let result = KmipOperationResult::new(status, item.result_reason(), result_message)
            .map_err(CertifyError::InvalidOperationResult)?;
        let (response_ttlv, unique_identifier) = if status.raw() == SUCCESS {
            let response_ttlv = common::clone_response_item(item, "Certify")
                .map_err(|_| CertifyError::MalformedResponseItem)?;
            let unique_identifier = item
                .with_response_payload(|payload| successful_identifier(&payload))
                .ok_or(CertifyError::MalformedSuccessPayload)??;
            (Some(response_ttlv), Some(unique_identifier))
        } else {
            (None, None)
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

    /// Returns the generated certificate Unique Identifier on success.
    #[must_use]
    pub const fn unique_identifier(&self) -> Option<&UniqueIdentifier> {
        self.unique_identifier.as_ref()
    }

    /// Lends the complete ordered generic response batch item on success.
    ///
    /// Error response contents are intentionally unavailable to prevent
    /// response payloads from escaping through ordinary error handling.
    pub fn with_ttlv<R>(&self, callback: impl for<'a> FnOnce(StructureView<'a>) -> R) -> Option<R> {
        self.response_ttlv
            .as_ref()
            .map(|response_ttlv| response_ttlv.with_ttlv(callback))
    }
}

/// A payload-free error converting a validated Certify response item.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CertifyError {
    /// The response item is not the client-to-server Certify operation.
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

impl fmt::Display for CertifyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedOperation => formatter.write_str("response item is not Certify"),
            Self::MissingResultStatus => formatter.write_str("Certify result status is missing"),
            Self::InvalidOperationResult(cause) => {
                write!(formatter, "Certify operation result is invalid: {cause}")
            }
            Self::MalformedResponseItem => {
                formatter.write_str("Certify response item is malformed")
            }
            Self::MalformedSuccessPayload => {
                formatter.write_str("successful Certify response payload is malformed")
            }
        }
    }
}

impl Error for CertifyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidOperationResult(cause) => Some(cause),
            _ => None,
        }
    }
}

fn contains_certificate_type(attributes: &Structure) -> bool {
    attributes.view().children().iter().any(|field| {
        field.tag().raw() == CERTIFICATE_TYPE
            && field.with_value(|value| matches!(value, kmipkit_ttlv::ValueView::Enumeration(_)))
    })
}

fn successful_identifier(payload: &StructureView<'_>) -> Result<UniqueIdentifier, CertifyError> {
    super::lifecycle::successful_response_identifier(payload)
        .ok_or(CertifyError::MalformedSuccessPayload)
}

fn invalid_request() -> ProtocolError {
    ProtocolError::categorized(
        ProtocolErrorKind::InvalidInput,
        ProtocolCauseCategory::InvalidValue,
    )
}
