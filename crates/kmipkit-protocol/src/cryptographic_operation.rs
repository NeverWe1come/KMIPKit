//! Shared raw values and payload helpers for KMIP cryptographic operations.

use std::error::Error;
use std::fmt;

use kmipkit_ttlv::{Item, ModelError, RawTag, Structure, StructureView, Tag, Value, ValueView};

use crate::{
    KmipOperationResult, OperationData, ProtocolCauseCategory, ProtocolError, ProtocolErrorKind,
    ResponseBatchItemView, ResultValidationError, SecretBytes, UniqueIdentifier,
};

pub(crate) const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
pub(crate) const CRYPTOGRAPHIC_PARAMETERS: u32 = 0x0042_002B;
pub(crate) const HASHING_ALGORITHM: u32 = 0x0042_0038;
pub(crate) const DATA: u32 = 0x0042_00C2;
pub(crate) const DIGESTED_DATA: u32 = 0x0042_0107;
pub(crate) const CORRELATION_VALUE: u32 = 0x0042_00D6;
pub(crate) const INIT_INDICATOR: u32 = 0x0042_00D7;
pub(crate) const FINAL_INDICATOR: u32 = 0x0042_00D8;
pub(crate) const MAC_DATA: u32 = 0x0042_00C4;
pub(crate) const SIGNATURE_DATA: u32 = 0x0042_00C7;
pub(crate) const VALIDITY_INDICATOR: u32 = 0x0042_0128;
pub(crate) const SUCCESS: u32 = 0;
pub(crate) const PENDING: u32 = 2;

macro_rules! raw_enumeration {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        pub struct $name(u32);

        impl $name {
            /// Creates a value from its raw KMIP Enumeration code.
            #[must_use]
            pub const fn from_raw(raw: u32) -> Self {
                Self(raw)
            }

            /// Returns the exact raw KMIP Enumeration code.
            #[must_use]
            pub const fn raw(self) -> u32 {
                self.0
            }
        }
    };
}

raw_enumeration!(
    HashingAlgorithm,
    "An open KMIP Hashing Algorithm value from §11.21."
);
raw_enumeration!(
    CryptographicAlgorithm,
    "An open KMIP Cryptographic Algorithm value from §11.12."
);
raw_enumeration!(
    DigitalSignatureAlgorithm,
    "An open KMIP Digital Signature Algorithm value from §11.16."
);
raw_enumeration!(
    ValidityIndicator,
    "An open KMIP Validity Indicator value from §11.61."
);

/// The request-derived framing context used to validate cryptographic
/// operation response fields.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CryptographicOperationResponseContext {
    /// A single-part operation whose response fields are required by its table.
    SinglePart,
    /// A multipart response before the request marks the final part.
    MultipartNonFinal,
    /// A response to a multipart request marked final. Hash, MAC, and Sign
    /// outputs remain absent; Verify Validity Indicator presence remains
    /// tolerant while KMIPKIT-DISC-048 is unresolved.
    MultipartFinal,
}

/// Backwards-compatible name for the request-derived operation response context.
pub type VerificationResponseContext = CryptographicOperationResponseContext;

/// Classifies request framing for operation response validation.
pub(crate) fn response_context_from_framing(
    has_correlation: bool,
    init: Option<bool>,
    final_part: Option<bool>,
) -> CryptographicOperationResponseContext {
    if has_correlation {
        if final_part == Some(true) {
            CryptographicOperationResponseContext::MultipartFinal
        } else {
            CryptographicOperationResponseContext::MultipartNonFinal
        }
    } else if init == Some(true) && final_part != Some(true) {
        CryptographicOperationResponseContext::MultipartNonFinal
    } else {
        CryptographicOperationResponseContext::SinglePart
    }
}

/// A sanitized error while converting a typed cryptographic operation model.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CryptographicOperationErrorKind {
    /// The response has a different operation code.
    UnexpectedOperation,
    /// The response omitted its Result Status.
    MissingResultStatus,
    /// The response result violated common KMIP result rules.
    InvalidOperationResult(ResultValidationError),
    /// A Pending result was passed to the completed-response conversion.
    PendingOutcomeRequired,
    /// A non-Pending result was passed to the Pending-response conversion.
    NotPendingOutcome,
    /// A successful response omitted its Response Payload.
    MissingSuccessPayload,
    /// A successful request or response payload has an invalid shape.
    MalformedPayload,
}

/// A payload-free operation error; it never retains untrusted TTLV values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CryptographicOperationError {
    operation: &'static str,
    kind: CryptographicOperationErrorKind,
}

impl CryptographicOperationError {
    pub(crate) const fn new(
        operation: &'static str,
        kind: CryptographicOperationErrorKind,
    ) -> Self {
        Self { operation, kind }
    }

    /// Returns the safe structural error category.
    #[must_use]
    pub const fn kind(self) -> CryptographicOperationErrorKind {
        self.kind
    }
}

impl fmt::Display for CryptographicOperationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} response is invalid: {:?}",
            self.operation, self.kind
        )
    }
}

impl Error for CryptographicOperationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.kind {
            CryptographicOperationErrorKind::InvalidOperationResult(error) => Some(error),
            _ => None,
        }
    }
}

pub(crate) fn request_error() -> ProtocolError {
    ProtocolError::categorized(
        ProtocolErrorKind::InvalidValue,
        ProtocolCauseCategory::InvalidValue,
    )
}

pub(crate) fn push(
    structure: &mut Structure,
    raw_tag: u32,
    value: Value,
) -> Result<(), ProtocolError> {
    let item = Item::new(tag(raw_tag)?, value).map_err(model_error)?;
    structure.try_push(item).map_err(model_error)
}

pub(crate) fn tag(raw_tag: u32) -> Result<Tag, ProtocolError> {
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

pub(crate) fn unique_identifier_value(identifier: UniqueIdentifier) -> Value {
    match identifier {
        UniqueIdentifier::TextString(value) => Value::text_string(value),
        UniqueIdentifier::Enumeration(value) => Value::enumeration(value),
        UniqueIdentifier::Integer(value) => Value::integer(value),
    }
}

pub(crate) fn validate_parameters(parameters: &Structure) -> Result<(), ProtocolError> {
    crate::cryptographic_parameters::validate_known_cryptographic_parameters_view(
        &parameters.view(),
    )?;
    crate::cryptographic_parameters::validate_cryptographic_parameters_view(&parameters.view())
}

pub(crate) fn operation_data_value(data: OperationData) -> Value {
    data.into_ttlv_value()
}

pub(crate) fn secret_value(bytes: SecretBytes) -> Value {
    bytes.into_ttlv_value()
}

/// Validates framing without deriving operation state or supplying later parts.
pub(crate) fn validate_framing(
    has_input: bool,
    has_correlation: bool,
    init: Option<bool>,
    final_part: Option<bool>,
) -> Result<(), ProtocolError> {
    let valid = match (init, final_part, has_correlation) {
        (None, None, false) => has_input,
        (Some(true), _, false) => final_part == Some(true) || !has_input,
        (Some(false), _, true) | (_, Some(true | false), true) | (None, None, true) => !has_input,
        _ => false,
    };
    if valid { Ok(()) } else { Err(request_error()) }
}

pub(crate) fn encode_common_fields(
    payload: &mut Structure,
    identifier: Option<UniqueIdentifier>,
    parameters: Option<Structure>,
    data: Option<OperationData>,
    correlation: Option<SecretBytes>,
    init: Option<bool>,
    final_part: Option<bool>,
) -> Result<(), ProtocolError> {
    if let Some(identifier) = identifier {
        push(
            payload,
            UNIQUE_IDENTIFIER,
            unique_identifier_value(identifier),
        )?;
    }
    if let Some(parameters) = parameters {
        validate_parameters(&parameters)?;
        push(
            payload,
            CRYPTOGRAPHIC_PARAMETERS,
            Value::structure(parameters),
        )?;
    }
    if let Some(data) = data {
        push(payload, DATA, operation_data_value(data))?;
    }
    if let Some(correlation) = correlation {
        push(payload, CORRELATION_VALUE, secret_value(correlation))?;
    }
    if let Some(init) = init {
        push(payload, INIT_INDICATOR, Value::boolean(init))?;
    }
    if let Some(final_part) = final_part {
        push(payload, FINAL_INDICATOR, Value::boolean(final_part))?;
    }
    Ok(())
}

pub(crate) fn parse_result(
    item: ResponseBatchItemView<'_>,
    expected_operation: u32,
    operation: &'static str,
    pending_expected: Option<bool>,
) -> Result<KmipOperationResult, CryptographicOperationError> {
    use CryptographicOperationErrorKind as Kind;

    if item.operation() != Some(expected_operation) {
        return Err(CryptographicOperationError::new(
            operation,
            Kind::UnexpectedOperation,
        ));
    }
    let result = crate::result::parse_operation_result(item).map_err(|error| match error {
        crate::result::OperationResultParseError::MissingResultStatus => {
            CryptographicOperationError::new(operation, Kind::MissingResultStatus)
        }
        crate::result::OperationResultParseError::Invalid(cause) => {
            CryptographicOperationError::new(operation, Kind::InvalidOperationResult(cause))
        }
    })?;
    match (pending_expected, result.status().raw() == PENDING) {
        (Some(true), false) => Err(CryptographicOperationError::new(
            operation,
            Kind::NotPendingOutcome,
        )),
        (Some(false), true) => Err(CryptographicOperationError::new(
            operation,
            Kind::PendingOutcomeRequired,
        )),
        _ => Ok(result),
    }
}

/// Copies the complete source batch item so typed responses retain generic
/// fields after the owning response message is dropped.
pub(crate) fn clone_response_item(
    item: ResponseBatchItemView<'_>,
    operation: &'static str,
) -> Result<OwnedResponseTtlv, CryptographicOperationError> {
    match item.with_ttlv(|ttlv| ttlv.try_clone()) {
        Some(Ok(ttlv)) => Ok(OwnedResponseTtlv { ttlv }),
        Some(Err(_)) | None => Err(response_shape_error(operation)),
    }
}

/// An owned, zeroizing copy of one generic response batch item.
#[derive(Debug)]
pub(crate) struct OwnedResponseTtlv {
    ttlv: Structure,
}

impl OwnedResponseTtlv {
    /// Lends the complete generic response item for callback-scoped access.
    pub(crate) fn with_ttlv<R>(&self, callback: impl for<'a> FnOnce(StructureView<'a>) -> R) -> R {
        callback(self.ttlv.view())
    }
}

pub(crate) fn parse_identifier(field: &Item) -> Option<UniqueIdentifier> {
    field.with_value(|value| match value {
        ValueView::TextString(value) => Some(UniqueIdentifier::TextString(value.to_owned())),
        ValueView::Enumeration(value) => Some(UniqueIdentifier::Enumeration(*value)),
        ValueView::Integer(value) => Some(UniqueIdentifier::Integer(*value)),
        _ => None,
    })
}

pub(crate) fn parse_secret(field: &Item) -> Option<SecretBytes> {
    field.with_value(|value| match value {
        ValueView::ByteString(bytes) => Some(SecretBytes::new(bytes.to_vec())),
        _ => None,
    })
}

/// Parses one required Unique Identifier field while rejecting duplicates and
/// values outside the Unique Identifier alternatives in Table 187.
pub(crate) fn parse_required_identifier(
    slot: &mut Option<UniqueIdentifier>,
    field: &Item,
    operation: &'static str,
) -> Result<(), CryptographicOperationError> {
    if slot.is_some() {
        return Err(response_shape_error(operation));
    }
    *slot = Some(parse_identifier(field).ok_or_else(|| response_shape_error(operation))?);
    Ok(())
}

/// Parses one optional sensitive Byte String field without retaining raw
/// malformed payload values in the returned error.
pub(crate) fn parse_optional_secret(
    slot: &mut Option<SecretBytes>,
    field: &Item,
    operation: &'static str,
) -> Result<(), CryptographicOperationError> {
    if slot.is_some() {
        return Err(response_shape_error(operation));
    }
    *slot = Some(parse_secret(field).ok_or_else(|| response_shape_error(operation))?);
    Ok(())
}

/// Parses one open Validity Indicator Enumeration while rejecting duplicates.
pub(crate) fn parse_validity_indicator(
    slot: &mut Option<ValidityIndicator>,
    field: &Item,
    operation: &'static str,
) -> Result<(), CryptographicOperationError> {
    if slot.is_some() {
        return Err(response_shape_error(operation));
    }
    *slot = Some(
        field
            .with_value(|value| match value {
                ValueView::Enumeration(raw) => Some(ValidityIndicator::from_raw(*raw)),
                _ => None,
            })
            .ok_or_else(|| response_shape_error(operation))?,
    );
    Ok(())
}

/// Validates the indicator presence rule for the request-derived multipart
/// context while preserving KMIPKIT-DISC-048's final-part ambiguity.
pub(crate) fn validate_validity_indicator_shape(
    indicator: Option<ValidityIndicator>,
    context: CryptographicOperationResponseContext,
    operation: &'static str,
) -> Result<(), CryptographicOperationError> {
    let valid = match context {
        CryptographicOperationResponseContext::SinglePart => indicator.is_some(),
        CryptographicOperationResponseContext::MultipartNonFinal => indicator.is_none(),
        CryptographicOperationResponseContext::MultipartFinal => true,
    };
    if valid {
        Ok(())
    } else {
        Err(response_shape_error(operation))
    }
}

/// Validates whether an operation output is present for the request framing.
pub(crate) fn validate_operation_output_shape(
    has_output: bool,
    context: CryptographicOperationResponseContext,
    operation: &'static str,
) -> Result<(), CryptographicOperationError> {
    let is_single_part = matches!(context, CryptographicOperationResponseContext::SinglePart);
    if has_output == is_single_part {
        Ok(())
    } else {
        Err(response_shape_error(operation))
    }
}

/// Returns the sanitized error used for malformed successful crypto payloads.
pub(crate) fn response_shape_error(operation: &'static str) -> CryptographicOperationError {
    CryptographicOperationError::new(operation, CryptographicOperationErrorKind::MalformedPayload)
}

/// Parsed fields shared by successful MAC and Sign operation responses.
pub(crate) struct ParsedOperationOutputPayload {
    pub(crate) unique_identifier: UniqueIdentifier,
    pub(crate) output_data: Option<SecretBytes>,
    pub(crate) correlation_value: Option<SecretBytes>,
}

/// Parses the common successful-response fields for MAC and Sign.
pub(crate) fn parse_operation_output_payload(
    payload: &StructureView<'_>,
    output_tag: u32,
    operation: &'static str,
) -> Result<ParsedOperationOutputPayload, CryptographicOperationError> {
    let mut unique_identifier = None;
    let mut output_data = None;
    let mut correlation_value = None;

    for field in payload.children() {
        match field.tag().raw() {
            UNIQUE_IDENTIFIER => {
                parse_required_identifier(&mut unique_identifier, field, operation)?;
            }
            tag if tag == output_tag => {
                parse_optional_secret(&mut output_data, field, operation)?;
            }
            CORRELATION_VALUE => {
                parse_optional_secret(&mut correlation_value, field, operation)?;
            }
            _ => {}
        }
    }

    Ok(ParsedOperationOutputPayload {
        unique_identifier: unique_identifier.ok_or_else(|| response_shape_error(operation))?,
        output_data,
        correlation_value,
    })
}
