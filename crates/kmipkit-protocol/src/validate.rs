//! Typed KMIP 2.1 Validate request and response payloads.
//!
//! Fields follow OASIS KMIP Specification v2.1 §6.1.57, Tables 340–342.
//! Section 2.1, Table 2 defines the supplied Certificate structures.

use std::{error::Error, fmt};

use kmipkit_ttlv::{Item, ItemType, Structure, StructureView, Value, ValueView};

use crate::cryptographic_operation;

use crate::{
    KmipOperationResult, ProtocolError, ResponseBatchItemView, ResultMessage,
    ResultValidationError, UniqueIdentifier, ValidityIndicator,
};

const OPERATION: u32 = 0x0000_0017;

const SUCCESS: u32 = 0;
const CERTIFICATE: u32 = 0x0042_0013;
const CERTIFICATE_TYPE: u32 = 0x0042_001d;
const CERTIFICATE_VALUE: u32 = 0x0042_001e;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const VALIDITY_DATE: u32 = 0x0042_009a;
const VALIDITY_INDICATOR: u32 = 0x0042_009b;

/// One certificate chain with ordered lists from section 6.1.57, Table 340.
///
/// The request must contain at least one Certificate or Unique Identifier
/// across the two lists. Either list may be empty, and either may repeat. There
/// is no second-chain field. Fields use Table 340 order as required by section
/// 10.1.2. The server controls chain validation, trust anchors, and policy.
pub struct ValidateRequest {
    certificates: Vec<Structure>,
    identifiers: Vec<UniqueIdentifier>,
    extensions: Vec<Item>,
    validity_date: Option<i64>,
}

impl ValidateRequest {
    /// Creates a request builder with no chain members or evaluation date.
    ///
    /// Add at least one Certificate or Unique Identifier before encoding the
    /// request; an optional Validity Date does not count as a chain member.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            certificates: Vec::new(),
            identifiers: Vec::new(),
            extensions: Vec::new(),
            validity_date: None,
        }
    }

    /// Appends one Certificate Structure, preserving its relative list order.
    ///
    /// Table 2 field types are checked when the payload is built; DER contents
    /// are forwarded without local cryptographic or trust validation.
    #[must_use]
    pub fn with_certificate(mut self, certificate: Structure) -> Self {
        self.certificates.push(certificate);
        self
    }

    /// Appends one Managed Certificate identifier in its relative list order.
    #[must_use]
    pub fn with_unique_identifier(mut self, identifier: UniqueIdentifier) -> Self {
        self.identifiers.push(identifier);
        self
    }

    /// Supplies the exact evaluation date; no date is synthesized when omitted.
    #[must_use]
    pub const fn with_validity_date(mut self, date: i64) -> Self {
        self.validity_date = Some(date);
        self
    }

    /// Returns the total entries in the two lists composing one chain.
    #[must_use]
    pub fn chain_len(&self) -> usize {
        self.certificates.len() + self.identifiers.len()
    }

    /// Returns the supplied date or its preserved absence.
    #[must_use]
    pub const fn validity_date(&self) -> Option<i64> {
        self.validity_date
    }

    /// Appends an unknown operation-payload extension in caller-selected order.
    ///
    /// Table-defined fields must use the typed builders; using an extension
    /// to override them fails locally while all other checked tags remain open.
    #[must_use]
    pub fn with_extension(mut self, extension: Item) -> Self {
        self.extensions.push(extension);
        self
    }

    /// Moves Certificates, Identifiers and the date into Table 340 field order.
    ///
    /// # Errors
    ///
    /// Returns a sanitized error for malformed Table 2 Certificate fields or
    /// an unrepresentable TTLV structure, or when both chain-member lists are
    /// empty. It does not evaluate certificates.
    pub fn into_ttlv_payload(self) -> Result<Structure, ProtocolError> {
        if self.certificates.is_empty() && self.identifiers.is_empty() {
            return Err(cryptographic_operation::request_error());
        }

        let mut payload = Structure::new();
        for certificate in self.certificates {
            validate_certificate_shape(&certificate)?;
            cryptographic_operation::push(
                &mut payload,
                CERTIFICATE,
                Value::structure(certificate),
            )?;
        }
        for identifier in self.identifiers {
            cryptographic_operation::push(
                &mut payload,
                UNIQUE_IDENTIFIER,
                cryptographic_operation::unique_identifier_value(identifier),
            )?;
        }
        if let Some(date) = self.validity_date {
            cryptographic_operation::push(&mut payload, VALIDITY_DATE, Value::date_time(date))?;
        }
        crate::diagnostics::append_extensions(
            &mut payload,
            self.extensions,
            &[CERTIFICATE, UNIQUE_IDENTIFIER, VALIDITY_DATE],
        )?;
        Ok(payload)
    }
}

fn validate_certificate_shape(certificate: &Structure) -> Result<(), ProtocolError> {
    let mut certificate_type_seen = false;
    let mut certificate_value_seen = false;
    for field in certificate.view().children() {
        match field.tag().raw() {
            CERTIFICATE_TYPE
                if !certificate_type_seen && field.item_type() == ItemType::Enumeration =>
            {
                certificate_type_seen = true;
            }
            CERTIFICATE_VALUE
                if certificate_type_seen
                    && !certificate_value_seen
                    && field.item_type() == ItemType::ByteString =>
            {
                certificate_value_seen = true;
            }
            CERTIFICATE_TYPE | CERTIFICATE_VALUE => {
                return Err(cryptographic_operation::request_error());
            }
            _ => {}
        }
    }
    if !certificate_type_seen || !certificate_value_seen {
        return Err(cryptographic_operation::request_error());
    }
    Ok(())
}

impl Default for ValidateRequest {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for ValidateRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ValidateRequest")
            .field("certificates", &"[REDACTED]")
            .field("identifiers", &"[REDACTED]")
            .field("validity_date", &self.validity_date)
            .field("extensions", &"[REDACTED]")
            .finish()
    }
}

/// A typed Validate result from §6.1.57, Tables 341–342.
pub struct ValidateResponse {
    result: KmipOperationResult,
    validity_indicator: Option<ValidityIndicator>,
    response_ttlv: cryptographic_operation::OwnedResponseTtlv,
}

impl ValidateResponse {
    /// Converts one validated Validate response item.
    ///
    /// Success requires exactly one open Validity Indicator from Table 341.
    /// Failure and Pending preserve the common result metadata.
    ///
    /// # Errors
    ///
    /// Returns [`ValidateError`] for another operation, inconsistent result
    /// metadata, or a missing/malformed success payload.
    pub fn try_from_response_item(item: ResponseBatchItemView<'_>) -> Result<Self, ValidateError> {
        if item.operation() != Some(OPERATION) {
            return Err(ValidateError::UnexpectedOperation);
        }

        let result = validate_operation_result(item)?;

        let validity_indicator = if result.status().raw() == SUCCESS {
            let indicator = item
                .with_response_payload(|payload| {
                    let mut indicator = None;
                    for field in payload.children() {
                        if field.tag().raw() == VALIDITY_INDICATOR {
                            if indicator.is_some() {
                                return Err(ValidateError::MalformedResponsePayload);
                            }
                            indicator = field.with_value(|value| match value {
                                ValueView::Enumeration(raw) => {
                                    Some(ValidityIndicator::from_raw(*raw))
                                }
                                _ => None,
                            });
                            if indicator.is_none() {
                                return Err(ValidateError::MalformedResponsePayload);
                            }
                        }
                    }
                    indicator.ok_or(ValidateError::MalformedResponsePayload)
                })
                .ok_or(ValidateError::MissingResponsePayload)??;
            Some(indicator)
        } else {
            None
        };

        let response_ttlv = cryptographic_operation::clone_response_item(item, "Validate")
            .map_err(|_| ValidateError::MalformedResponseItem)?;

        Ok(Self {
            result,
            validity_indicator,
            response_ttlv,
        })
    }
    /// Returns the complete KMIP result, including status and optional reason.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }
    /// Returns the server's required open indicator on Success.
    #[must_use]
    pub const fn validity_indicator(&self) -> Option<ValidityIndicator> {
        self.validity_indicator
    }

    /// Lends the complete generic response, preserving unknown fields and order.
    pub fn with_ttlv<R>(&self, callback: impl for<'a> FnOnce(StructureView<'a>) -> R) -> R {
        self.response_ttlv.with_ttlv(callback)
    }
}

fn validate_operation_result(
    item: ResponseBatchItemView<'_>,
) -> Result<KmipOperationResult, ValidateError> {
    let status = item
        .result_status()
        .ok_or(ValidateError::MissingResultStatus)?;

    let result_message = item.with_result_message(|text| ResultMessage::new(text.to_owned()));

    KmipOperationResult::new(status, item.result_reason(), result_message)
        .map_err(ValidateError::InvalidOperationResult)
}
/// A payload-free error converting an Validate response batch item.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidateError {
    /// The response item identifies an operation other than Validate.
    UnexpectedOperation,
    /// The validated response item is missing Result Status.
    MissingResultStatus,
    /// The common result metadata is inconsistent.
    InvalidOperationResult(ResultValidationError),
    /// A successful Validate response omitted its Response Payload.
    MissingResponsePayload,
    /// A successful Validate response contains missing, duplicate, or incorrectly typed indicator fields.
    MalformedResponsePayload,
    /// The generic response item cannot be retained.
    MalformedResponseItem,
}

impl fmt::Display for ValidateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedOperation => formatter.write_str("response item is not Validate"),

            Self::MissingResultStatus => formatter.write_str("Validate result status is missing"),

            Self::InvalidOperationResult(error) => {
                write!(formatter, "Validate operation result is invalid: {error}")
            }

            Self::MissingResponsePayload => {
                formatter.write_str("successful Validate response payload is missing")
            }

            Self::MalformedResponseItem => {
                formatter.write_str("Validate response item is malformed")
            }

            Self::MalformedResponsePayload => {
                formatter.write_str("successful Validate response payload is malformed")
            }
        }
    }
}

impl Error for ValidateError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidOperationResult(error) => Some(error),

            Self::UnexpectedOperation
            | Self::MissingResultStatus
            | Self::MissingResponsePayload
            | Self::MalformedResponsePayload
            | Self::MalformedResponseItem => None,
        }
    }
}

impl fmt::Debug for ValidateResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ValidateResponse")
            .field("result", &self.result)
            .field("validity_indicator", &self.validity_indicator)
            .field("response_ttlv", &"[REDACTED]")
            .finish()
    }
}
