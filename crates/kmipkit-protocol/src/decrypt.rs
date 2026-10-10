//! Typed KMIP 2.1 Decrypt request and response payloads.

use std::error::Error;
use std::fmt;

use kmipkit_ttlv::{Item, ModelError, RawTag, Structure, StructureView, Tag, Value, ValueView};

use crate::asynchronous::is_pending;
use crate::{
    KmipOperationResult, OperationData, ProtocolCauseCategory, ProtocolError, ProtocolErrorKind,
    ResponseBatchItemView, ResultMessage, ResultValidationError, SecretBytes, UniqueIdentifier,
};

const DECRYPT_OPERATION: u32 = 0x0000_0020;
const SUCCESS: u32 = 0;

const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const CRYPTOGRAPHIC_PARAMETERS: u32 = 0x0042_002B;
const DATA: u32 = 0x0042_00C2;
const IV_COUNTER_NONCE: u32 = 0x0042_003D;
const CORRELATION_VALUE: u32 = 0x0042_00D6;
const INIT_INDICATOR: u32 = 0x0042_00D7;
const FINAL_INDICATOR: u32 = 0x0042_00D8;
const AUTHENTICATED_ENCRYPTION_ADDITIONAL_DATA: u32 = 0x0042_00FE;
const AUTHENTICATED_ENCRYPTION_TAG: u32 = 0x0042_00FF;

const BLOCK_CIPHER_MODE: u32 = 0x0042_0011;
const CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;
const HASHING_ALGORITHM: u32 = 0x0042_0038;
const PADDING_METHOD: u32 = 0x0042_005F;
const KEY_ROLE_TYPE: u32 = 0x0042_0083;
const DIGITAL_SIGNATURE_ALGORITHM: u32 = 0x0042_00AE;
const RANDOM_IV: u32 = 0x0042_00C5;
const IV_LENGTH: u32 = 0x0042_00CD;
const TAG_LENGTH: u32 = 0x0042_00CE;
const FIXED_FIELD_LENGTH: u32 = 0x0042_00CF;
const COUNTER_LENGTH: u32 = 0x0042_00D0;
const INITIAL_COUNTER_VALUE: u32 = 0x0042_00D1;
const INVOCATION_FIELD_LENGTH: u32 = 0x0042_00D2;
const SALT_LENGTH: u32 = 0x0042_0100;
const MASK_GENERATOR: u32 = 0x0042_0101;
const MASK_GENERATOR_HASHING_ALGORITHM: u32 = 0x0042_0102;
const P_SOURCE: u32 = 0x0042_0103;
const TRAILER_FIELD: u32 = 0x0042_0104;

/// A typed KMIP 2.1 Decrypt request payload from §6.1.11, Table 196.
///
/// Optional members retain omission and caller-selected values. Sensitive
/// byte strings are moved into zeroizing TTLV values when the payload is built.
#[derive(Debug)]
pub struct DecryptRequest {
    unique_identifier: Option<UniqueIdentifier>,
    cryptographic_parameters: Option<Structure>,
    data: Option<OperationData>,
    iv_counter_nonce: Option<SecretBytes>,
    correlation_value: Option<SecretBytes>,
    init_indicator: Option<bool>,
    final_indicator: Option<bool>,
    authenticated_encryption_additional_data: Option<SecretBytes>,
    authenticated_encryption_tag: Option<SecretBytes>,
}

impl DecryptRequest {
    /// Creates a Decrypt request while preserving optional identifier and Data.
    #[must_use]
    pub const fn new(
        unique_identifier: Option<UniqueIdentifier>,
        data: Option<OperationData>,
    ) -> Self {
        Self {
            unique_identifier,
            cryptographic_parameters: None,
            data,
            iv_counter_nonce: None,
            correlation_value: None,
            init_indicator: None,
            final_indicator: None,
            authenticated_encryption_additional_data: None,
            authenticated_encryption_tag: None,
        }
    }

    /// Returns the optional identifier in its original wire form.
    #[must_use]
    pub const fn unique_identifier(&self) -> Option<&UniqueIdentifier> {
        self.unique_identifier.as_ref()
    }

    /// Returns the optional request Data in its original §7.9 encoding.
    #[must_use]
    pub const fn data(&self) -> Option<&OperationData> {
        self.data.as_ref()
    }

    /// Supplies the optional ordered Cryptographic Parameters Structure.
    #[must_use]
    pub fn with_cryptographic_parameters(mut self, parameters: Structure) -> Self {
        self.cryptographic_parameters = Some(parameters);
        self
    }

    /// Supplies the optional IV/Counter/Nonce Byte String.
    #[must_use]
    pub fn with_iv_counter_nonce(mut self, value: SecretBytes) -> Self {
        self.iv_counter_nonce = Some(value);
        self
    }

    /// Supplies the optional multipart Correlation Value Byte String.
    #[must_use]
    pub fn with_correlation_value(mut self, value: SecretBytes) -> Self {
        self.correlation_value = Some(value);
        self
    }

    /// Supplies the optional caller-controlled Init Indicator.
    #[must_use]
    pub const fn with_init_indicator(mut self, value: bool) -> Self {
        self.init_indicator = Some(value);
        self
    }

    /// Supplies the optional caller-controlled Final Indicator.
    #[must_use]
    pub const fn with_final_indicator(mut self, value: bool) -> Self {
        self.final_indicator = Some(value);
        self
    }

    /// Supplies optional Authenticated Encryption Additional Data (§7.3).
    #[must_use]
    pub fn with_authenticated_encryption_additional_data(mut self, value: SecretBytes) -> Self {
        self.authenticated_encryption_additional_data = Some(value);
        self
    }

    /// Supplies the optional Decrypt Authenticated Encryption Tag (§7.4).
    #[must_use]
    pub fn with_authenticated_encryption_tag(mut self, value: SecretBytes) -> Self {
        self.authenticated_encryption_tag = Some(value);
        self
    }

    /// Builds the ordered Table 196 payload, moving sensitive data into TTLV.
    ///
    /// The request is consumed so owned secret allocations and the generic
    /// Cryptographic Parameters Structure move into the returned payload.
    ///
    /// # Errors
    ///
    /// Returns a sanitized protocol error if a member is malformed or
    /// Cryptographic Parameters fail their local §4.16 validation.
    pub fn to_ttlv_payload(self) -> Result<Structure, ProtocolError> {
        let mut payload = Structure::new();
        if let Some(identifier) = self.unique_identifier {
            push(
                &mut payload,
                UNIQUE_IDENTIFIER,
                unique_identifier_value(identifier),
            )?;
        }
        if let Some(parameters) = self.cryptographic_parameters {
            validate_known_cryptographic_parameters_view(&parameters.view())?;
            let parameters = crate::cryptographic_parameters::validate_cryptographic_parameters(
                Some(parameters),
            )?
            .ok_or_else(invalid_request_payload)?;
            push(
                &mut payload,
                CRYPTOGRAPHIC_PARAMETERS,
                Value::structure(parameters),
            )?;
        }
        if let Some(data) = self.data {
            push(&mut payload, DATA, data.into_ttlv_value())?;
        }
        if let Some(value) = self.iv_counter_nonce {
            push(&mut payload, IV_COUNTER_NONCE, value.into_ttlv_value())?;
        }
        if let Some(value) = self.correlation_value {
            push(&mut payload, CORRELATION_VALUE, value.into_ttlv_value())?;
        }
        if let Some(value) = self.init_indicator {
            push(&mut payload, INIT_INDICATOR, Value::boolean(value))?;
        }
        if let Some(value) = self.final_indicator {
            push(&mut payload, FINAL_INDICATOR, Value::boolean(value))?;
        }
        if let Some(value) = self.authenticated_encryption_additional_data {
            push(
                &mut payload,
                AUTHENTICATED_ENCRYPTION_ADDITIONAL_DATA,
                value.into_ttlv_value(),
            )?;
        }
        if let Some(value) = self.authenticated_encryption_tag {
            push(
                &mut payload,
                AUTHENTICATED_ENCRYPTION_TAG,
                value.into_ttlv_value(),
            )?;
        }

        validate_request_payload(&payload)?;
        Ok(payload)
    }
}

/// A typed Decrypt result for one validated KMIP response batch item.
///
/// A successful response requires one Unique Identifier; the remaining Table
/// 197 fields are optional Byte Strings. A non-success result retains the
/// shared operation result and has no typed success fields. The originating
/// [`crate::ResponseMessage`] remains the source of the full generic payload,
/// including unknown fields.
#[derive(Debug)]
pub struct DecryptResponse {
    result: KmipOperationResult,
    unique_identifier: Option<UniqueIdentifier>,
    data: Option<SecretBytes>,
    correlation_value: Option<SecretBytes>,
}

impl DecryptResponse {
    /// Converts one already validated, completed Decrypt response batch item.
    ///
    /// Success payload fields follow Table 197. Unknown fields remain
    /// available through the source response message. The client must route
    /// `Operation Pending` through the shared `PendingOutcome` path before
    /// calling this completed-response converter.
    ///
    /// # Errors
    ///
    /// Returns [`DecryptError`] for a different operation, an `Operation
    /// Pending` result, invalid result metadata, or a malformed successful
    /// payload. Errors contain no server payload values.
    pub fn try_from_response_item(item: ResponseBatchItemView<'_>) -> Result<Self, DecryptError> {
        if item.operation() != Some(DECRYPT_OPERATION) {
            return Err(DecryptError::UnexpectedOperation);
        }

        let status = item
            .result_status()
            .ok_or(DecryptError::MissingResultStatus)?;
        let result_message = item.with_result_message(|text| ResultMessage::new(text.to_owned()));
        let result = KmipOperationResult::new(status, item.result_reason(), result_message)
            .map_err(DecryptError::InvalidOperationResult)?;

        if is_pending(status) {
            return Err(DecryptError::PendingOutcomeRequired);
        }

        if status.raw() != SUCCESS {
            return Ok(Self {
                result,
                unique_identifier: None,
                data: None,
                correlation_value: None,
            });
        }

        let parsed = item
            .with_response_payload(|payload| parse_success_payload(&payload))
            .ok_or(DecryptError::MissingSuccessPayload)??;
        Ok(Self {
            result,
            unique_identifier: Some(parsed.unique_identifier),
            data: parsed.data,
            correlation_value: parsed.correlation_value,
        })
    }

    /// Returns the exact KMIP operation result.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }

    /// Returns the Unique Identifier required by a successful result.
    #[must_use]
    pub const fn unique_identifier(&self) -> Option<&UniqueIdentifier> {
        self.unique_identifier.as_ref()
    }

    /// Returns optional response Data as zeroizing bytes.
    #[must_use]
    pub const fn data(&self) -> Option<&SecretBytes> {
        self.data.as_ref()
    }

    /// Returns the optional multipart Correlation Value as zeroizing bytes.
    #[must_use]
    pub const fn correlation_value(&self) -> Option<&SecretBytes> {
        self.correlation_value.as_ref()
    }
}

/// A payload-free error converting a Decrypt response item.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecryptError {
    /// The response item is not the client-to-server Decrypt operation.
    UnexpectedOperation,
    /// The validated response item did not expose a Result Status.
    MissingResultStatus,
    /// The response is pending and must be routed through the shared
    /// `PendingOutcome` path instead of completed-response conversion.
    PendingOutcomeRequired,
    /// The represented KMIP result violates the shared status/reason contract.
    InvalidOperationResult(ResultValidationError),
    /// A successful Decrypt result omitted its Response Payload.
    MissingSuccessPayload,
    /// A successful Decrypt Response Payload is malformed.
    MalformedSuccessPayload,
}

impl fmt::Display for DecryptError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedOperation => formatter.write_str("response item is not Decrypt"),
            Self::MissingResultStatus => formatter.write_str("Decrypt result status is missing"),
            Self::PendingOutcomeRequired => formatter.write_str(
                "Decrypt result is Pending; route it through the shared PendingOutcome path",
            ),
            Self::InvalidOperationResult(cause) => {
                write!(formatter, "Decrypt operation result is invalid: {cause}")
            }
            Self::MissingSuccessPayload => {
                formatter.write_str("successful Decrypt response payload is missing")
            }
            Self::MalformedSuccessPayload => {
                formatter.write_str("successful Decrypt response payload is malformed")
            }
        }
    }
}

impl Error for DecryptError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidOperationResult(cause) => Some(cause),
            _ => None,
        }
    }
}

struct ParsedSuccessPayload {
    unique_identifier: UniqueIdentifier,
    data: Option<SecretBytes>,
    correlation_value: Option<SecretBytes>,
}

fn parse_success_payload(
    payload: &StructureView<'_>,
) -> Result<ParsedSuccessPayload, DecryptError> {
    let mut unique_identifier = None;
    let mut data = None;
    let mut correlation_value = None;

    for field in payload.children() {
        match field.tag().raw() {
            UNIQUE_IDENTIFIER => {
                if unique_identifier.is_some() {
                    return Err(DecryptError::MalformedSuccessPayload);
                }
                unique_identifier = parse_unique_identifier(field);
                if unique_identifier.is_none() {
                    return Err(DecryptError::MalformedSuccessPayload);
                }
            }
            DATA => assign_byte_string(&mut data, field)?,
            CORRELATION_VALUE => assign_byte_string(&mut correlation_value, field)?,
            IV_COUNTER_NONCE
            | AUTHENTICATED_ENCRYPTION_TAG
            | CRYPTOGRAPHIC_PARAMETERS
            | INIT_INDICATOR
            | FINAL_INDICATOR
            | AUTHENTICATED_ENCRYPTION_ADDITIONAL_DATA => {
                return Err(DecryptError::MalformedSuccessPayload);
            }
            // Unknown or future fields remain in the original generic payload.
            _ => {}
        }
    }

    Ok(ParsedSuccessPayload {
        unique_identifier: unique_identifier.ok_or(DecryptError::MalformedSuccessPayload)?,
        data,
        correlation_value,
    })
}

fn parse_unique_identifier(field: &Item) -> Option<UniqueIdentifier> {
    field.with_value(|value| match value {
        ValueView::TextString(value) => Some(UniqueIdentifier::TextString(value.to_owned())),
        ValueView::Enumeration(value) => Some(UniqueIdentifier::Enumeration(*value)),
        ValueView::Integer(value) => Some(UniqueIdentifier::Integer(*value)),
        _ => None,
    })
}

fn assign_byte_string(field: &mut Option<SecretBytes>, item: &Item) -> Result<(), DecryptError> {
    if field.is_some() {
        return Err(DecryptError::MalformedSuccessPayload);
    }
    *field = item.with_value(|value| match value {
        ValueView::ByteString(bytes) => Some(SecretBytes::new(bytes.to_vec())),
        _ => None,
    });
    if field.is_none() {
        return Err(DecryptError::MalformedSuccessPayload);
    }
    Ok(())
}

/// Validates a Decrypt request payload before it is accepted for serialization.
///
/// This is crate-private so later client dispatch can share the same checked
/// boundary without exposing a raw request parser publicly.
pub(crate) fn validate_request_payload(payload: &Structure) -> Result<(), ProtocolError> {
    let mut seen = [false; 9];

    for field in payload.view().children() {
        let raw_tag = field.tag().raw();
        if let Some(index) = request_singleton_index(raw_tag) {
            if seen[index] {
                return Err(invalid_request_payload());
            }
            seen[index] = true;
        }

        let valid = field.with_value(|value| match raw_tag {
            UNIQUE_IDENTIFIER => matches!(
                value,
                ValueView::TextString(_) | ValueView::Enumeration(_) | ValueView::Integer(_)
            ),
            CRYPTOGRAPHIC_PARAMETERS => match value {
                ValueView::Structure(parameters) => {
                    validate_cryptographic_parameters_view(&parameters).is_ok()
                }
                _ => false,
            },
            DATA => matches!(
                value,
                ValueView::ByteString(_) | ValueView::Enumeration(_) | ValueView::Integer(_)
            ),
            IV_COUNTER_NONCE
            | CORRELATION_VALUE
            | AUTHENTICATED_ENCRYPTION_ADDITIONAL_DATA
            | AUTHENTICATED_ENCRYPTION_TAG => {
                matches!(value, ValueView::ByteString(_))
            }
            INIT_INDICATOR | FINAL_INDICATOR => matches!(value, ValueView::Boolean(_)),
            _ => true,
        });
        if !valid {
            return Err(invalid_request_payload());
        }
    }

    Ok(())
}

fn validate_cryptographic_parameters_view(
    parameters: &StructureView<'_>,
) -> Result<(), ProtocolError> {
    validate_known_cryptographic_parameters_view(parameters)?;
    crate::cryptographic_parameters::validate_cryptographic_parameters_view(parameters)
}

fn validate_known_cryptographic_parameters_view(
    parameters: &StructureView<'_>,
) -> Result<(), ProtocolError> {
    let mut seen = [false; 18];
    for field in parameters.children() {
        let (index, valid_type) = match field.tag().raw() {
            BLOCK_CIPHER_MODE => (
                Some(0),
                field.with_value(|value| matches!(value, ValueView::Enumeration(_))),
            ),
            CRYPTOGRAPHIC_ALGORITHM => (
                Some(1),
                field.with_value(|value| matches!(value, ValueView::Enumeration(_))),
            ),
            HASHING_ALGORITHM => (
                Some(2),
                field.with_value(|value| matches!(value, ValueView::Enumeration(_))),
            ),
            PADDING_METHOD => (
                Some(3),
                field.with_value(|value| matches!(value, ValueView::Enumeration(_))),
            ),
            KEY_ROLE_TYPE => (
                Some(4),
                field.with_value(|value| matches!(value, ValueView::Enumeration(_))),
            ),
            DIGITAL_SIGNATURE_ALGORITHM => (
                Some(5),
                field.with_value(|value| matches!(value, ValueView::Enumeration(_))),
            ),
            RANDOM_IV => (
                Some(6),
                field.with_value(|value| matches!(value, ValueView::Boolean(_))),
            ),
            IV_LENGTH => (
                Some(7),
                field.with_value(|value| matches!(value, ValueView::Integer(_))),
            ),
            TAG_LENGTH => (
                Some(8),
                field.with_value(|value| matches!(value, ValueView::Integer(_))),
            ),
            FIXED_FIELD_LENGTH => (
                Some(9),
                field.with_value(|value| matches!(value, ValueView::Integer(_))),
            ),
            COUNTER_LENGTH => (
                Some(10),
                field.with_value(|value| matches!(value, ValueView::Integer(_))),
            ),
            INITIAL_COUNTER_VALUE => (
                Some(11),
                field.with_value(|value| matches!(value, ValueView::Integer(_))),
            ),
            INVOCATION_FIELD_LENGTH => (
                Some(12),
                field.with_value(|value| matches!(value, ValueView::Integer(_))),
            ),
            SALT_LENGTH => (
                Some(13),
                field.with_value(|value| matches!(value, ValueView::Integer(_))),
            ),
            MASK_GENERATOR => (
                Some(14),
                field.with_value(|value| matches!(value, ValueView::Enumeration(_))),
            ),
            MASK_GENERATOR_HASHING_ALGORITHM => (
                Some(15),
                field.with_value(|value| matches!(value, ValueView::Enumeration(_))),
            ),
            P_SOURCE => (
                Some(16),
                field.with_value(|value| matches!(value, ValueView::ByteString(_))),
            ),
            TRAILER_FIELD => (
                Some(17),
                field.with_value(|value| matches!(value, ValueView::Integer(_))),
            ),
            _ => (None, true),
        };
        if !valid_type {
            return Err(invalid_request_payload());
        }
        if let Some(index) = index {
            if seen[index] {
                return Err(invalid_request_payload());
            }
            seen[index] = true;
        }
    }

    Ok(())
}

fn request_singleton_index(raw_tag: u32) -> Option<usize> {
    match raw_tag {
        UNIQUE_IDENTIFIER => Some(0),
        CRYPTOGRAPHIC_PARAMETERS => Some(1),
        DATA => Some(2),
        IV_COUNTER_NONCE => Some(3),
        CORRELATION_VALUE => Some(4),
        INIT_INDICATOR => Some(5),
        FINAL_INDICATOR => Some(6),
        AUTHENTICATED_ENCRYPTION_ADDITIONAL_DATA => Some(7),
        AUTHENTICATED_ENCRYPTION_TAG => Some(8),
        _ => None,
    }
}

fn unique_identifier_value(identifier: UniqueIdentifier) -> Value {
    match identifier {
        UniqueIdentifier::TextString(value) => Value::text_string(value),
        UniqueIdentifier::Enumeration(value) => Value::enumeration(value),
        UniqueIdentifier::Integer(value) => Value::integer(value),
    }
}

fn push(payload: &mut Structure, raw_tag: u32, value: Value) -> Result<(), ProtocolError> {
    payload.try_push(item(raw_tag, value)?).map_err(model_error)
}

fn item(raw_tag: u32, value: Value) -> Result<Item, ProtocolError> {
    Item::new(tag(raw_tag)?, value).map_err(model_error)
}

fn tag(raw_tag: u32) -> Result<Tag, ProtocolError> {
    RawTag::new(raw_tag)
        .and_then(|raw| raw.try_checked())
        .map_err(model_error)
}

fn invalid_request_payload() -> ProtocolError {
    ProtocolError::categorized(
        ProtocolErrorKind::InvalidValue,
        ProtocolCauseCategory::InvalidValue,
    )
}

fn model_error(error: ModelError) -> ProtocolError {
    ProtocolError::new(
        ProtocolErrorKind::InvalidValue,
        ProtocolCauseCategory::InvalidValue,
        error,
    )
}
