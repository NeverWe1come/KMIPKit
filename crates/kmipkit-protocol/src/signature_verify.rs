//! Typed KMIP 2.1 Signature Verify request and response models.

use std::fmt;

use kmipkit_ttlv::{Structure, Value};

use crate::cryptographic_operation as common;
use crate::{
    CryptographicOperationError, KmipOperationResult, OperationData, ProtocolError,
    ResponseBatchItemView, SecretBytes, UniqueIdentifier, ValidityIndicator,
    VerificationResponseContext,
};

const OPERATION: u32 = 0x0000_0022;

/// A Signature Verify request from KMIP v2.1 §6.1.56, Table 337.
pub struct SignatureVerifyRequest {
    identifier: Option<UniqueIdentifier>,
    parameters: Option<Structure>,
    data: Option<OperationData>,
    digested_data: Option<SecretBytes>,
    signature_data: Option<SecretBytes>,
    correlation: Option<SecretBytes>,
    init: Option<bool>,
    final_part: Option<bool>,
}

impl SignatureVerifyRequest {
    /// Creates a Signature Verify request with all optional fields omitted.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            identifier: None,
            parameters: None,
            data: None,
            digested_data: None,
            signature_data: None,
            correlation: None,
            init: None,
            final_part: None,
        }
    }

    /// Supplies the optional Unique Identifier.
    #[must_use]
    pub fn with_unique_identifier(mut self, value: UniqueIdentifier) -> Self {
        self.identifier = Some(value);
        self
    }

    /// Supplies optional ordered Cryptographic Parameters.
    #[must_use]
    pub fn with_cryptographic_parameters(mut self, value: Structure) -> Self {
        self.parameters = Some(value);
        self
    }

    /// Supplies optional original Data (§7.9).
    #[must_use]
    pub fn with_data(mut self, value: OperationData) -> Self {
        self.data = Some(value);
        self
    }

    /// Supplies optional precomputed Digested Data.
    #[must_use]
    pub fn with_digested_data(mut self, value: SecretBytes) -> Self {
        self.digested_data = Some(value);
        self
    }

    /// Supplies single-part Signature Data (§7.38).
    #[must_use]
    pub fn with_signature_data(mut self, value: SecretBytes) -> Self {
        self.signature_data = Some(value);
        self
    }

    /// Supplies a multipart Correlation Value (§7.8).
    #[must_use]
    pub fn with_correlation_value(mut self, value: SecretBytes) -> Self {
        self.correlation = Some(value);
        self
    }

    /// Supplies the caller-controlled Init Indicator (§7.17).
    #[must_use]
    pub const fn with_init_indicator(mut self, value: bool) -> Self {
        self.init = Some(value);
        self
    }

    /// Supplies the caller-controlled Final Indicator (§7.14).
    #[must_use]
    pub const fn with_final_indicator(mut self, value: bool) -> Self {
        self.final_part = Some(value);
        self
    }

    /// Validates the locally knowable single-part or multipart shape.
    pub fn validate_multipart_shape(&self) -> Result<(), ProtocolError> {
        common::validate_framing(
            self.signature_data.is_some(),
            self.correlation.is_some(),
            self.init,
            self.final_part,
        )
    }

    /// Returns the request-derived response context for Validity Indicator
    /// shape validation.
    #[must_use]
    pub fn verification_response_context(&self) -> VerificationResponseContext {
        common::verification_response_context(
            self.correlation.is_some(),
            self.init,
            self.final_part,
        )
    }

    /// Builds a TTLV payload for Table 337.
    ///
    /// # Errors
    ///
    /// Returns a sanitized error for invalid Cryptographic Parameters or
    /// invalid single-part/multipart framing.
    pub fn to_ttlv_payload(self) -> Result<Structure, ProtocolError> {
        common::validate_framing(
            self.signature_data.is_some(),
            self.correlation.is_some(),
            self.init,
            self.final_part,
        )?;
        let mut payload = Structure::new();
        if let Some(identifier) = self.identifier {
            common::push(
                &mut payload,
                common::UNIQUE_IDENTIFIER,
                common::unique_identifier_value(identifier),
            )?;
        }
        if let Some(parameters) = self.parameters {
            common::validate_parameters(&parameters)?;
            common::push(
                &mut payload,
                common::CRYPTOGRAPHIC_PARAMETERS,
                Value::structure(parameters),
            )?;
        }
        if let Some(data) = self.data {
            common::push(
                &mut payload,
                common::DATA,
                common::operation_data_value(data),
            )?;
        }
        if let Some(data) = self.digested_data {
            common::push(
                &mut payload,
                common::DIGESTED_DATA,
                common::secret_value(data),
            )?;
        }
        if let Some(data) = self.signature_data {
            common::push(
                &mut payload,
                common::SIGNATURE_DATA,
                common::secret_value(data),
            )?;
        }
        append_framing(&mut payload, self.correlation, self.init, self.final_part)?;
        Ok(payload)
    }
}

impl Default for SignatureVerifyRequest {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for SignatureVerifyRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SignatureVerifyRequest")
            .field("has_unique_identifier", &self.identifier.is_some())
            .field("has_cryptographic_parameters", &self.parameters.is_some())
            .field("has_data", &self.data.is_some())
            .field("has_digested_data", &self.digested_data.is_some())
            .field("has_signature_data", &self.signature_data.is_some())
            .field("has_correlation_value", &self.correlation.is_some())
            .field("init_indicator", &self.init)
            .field("final_indicator", &self.final_part)
            .finish()
    }
}

/// A typed Signature Verify response for one KMIP response batch item.
#[derive(Debug)]
pub struct SignatureVerifyResponse {
    result: KmipOperationResult,
    unique_identifier: Option<UniqueIdentifier>,
    validity_indicator: Option<ValidityIndicator>,
    recovered_data: Option<SecretBytes>,
    correlation_value: Option<SecretBytes>,
}

impl SignatureVerifyResponse {
    /// Converts a Pending Signature Verify response result.
    pub fn try_from_pending_response_item(
        item: ResponseBatchItemView<'_>,
    ) -> Result<Self, SignatureVerifyError> {
        Ok(Self {
            result: common::parse_result(item, OPERATION, "Signature Verify", Some(true))?,
            unique_identifier: None,
            validity_indicator: None,
            recovered_data: None,
            correlation_value: None,
        })
    }

    /// Converts a completed, single-part Signature Verify response item.
    ///
    /// Use [`Self::try_from_response_item_with_context`] when the response
    /// belongs to a multipart request.
    pub fn try_from_response_item(
        item: ResponseBatchItemView<'_>,
    ) -> Result<Self, SignatureVerifyError> {
        Self::try_from_response_item_with_context(item, VerificationResponseContext::SinglePart)
    }

    /// Converts a completed Signature Verify response using its original
    /// request's single-part or multipart context.
    pub fn try_from_response_item_with_context(
        item: ResponseBatchItemView<'_>,
        context: VerificationResponseContext,
    ) -> Result<Self, SignatureVerifyError> {
        let result = common::parse_result(item, OPERATION, "Signature Verify", Some(false))?;
        if result.status().raw() != common::SUCCESS {
            return Ok(Self {
                result,
                unique_identifier: None,
                validity_indicator: None,
                recovered_data: None,
                correlation_value: None,
            });
        }

        let parsed = item
            .with_response_payload(|payload| parse_success_payload(&payload, context))
            .ok_or_else(|| {
                CryptographicOperationError::new(
                    "Signature Verify",
                    crate::CryptographicOperationErrorKind::MissingSuccessPayload,
                )
            })??;
        Ok(Self {
            result,
            unique_identifier: Some(parsed.unique_identifier),
            validity_indicator: parsed.validity_indicator,
            recovered_data: parsed.recovered_data,
            correlation_value: parsed.correlation_value,
        })
    }

    /// Returns the exact server-reported operation result.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }

    /// Returns the required Unique Identifier from a successful response.
    #[must_use]
    pub const fn unique_identifier(&self) -> Option<&UniqueIdentifier> {
        self.unique_identifier.as_ref()
    }

    /// Returns the open server-reported Validity Indicator when present.
    #[must_use]
    pub const fn validity_indicator(&self) -> Option<ValidityIndicator> {
        self.validity_indicator
    }

    /// Returns optional recovered Data as zeroizing bytes.
    #[must_use]
    pub const fn recovered_data(&self) -> Option<&SecretBytes> {
        self.recovered_data.as_ref()
    }

    /// Returns the optional multipart Correlation Value returned by the
    /// server.
    #[must_use]
    pub const fn correlation_value(&self) -> Option<&SecretBytes> {
        self.correlation_value.as_ref()
    }
}

/// A sanitized error converting a Signature Verify response item.
pub type SignatureVerifyError = CryptographicOperationError;

struct ParsedSignatureVerifyPayload {
    unique_identifier: UniqueIdentifier,
    validity_indicator: Option<ValidityIndicator>,
    recovered_data: Option<SecretBytes>,
    correlation_value: Option<SecretBytes>,
}

fn parse_success_payload(
    payload: &kmipkit_ttlv::StructureView<'_>,
    context: VerificationResponseContext,
) -> Result<ParsedSignatureVerifyPayload, SignatureVerifyError> {
    let mut unique_identifier = None;
    let mut validity_indicator = None;
    let mut recovered_data = None;
    let mut correlation_value = None;
    for field in payload.children() {
        match field.tag().raw() {
            common::UNIQUE_IDENTIFIER => {
                common::parse_required_identifier(
                    &mut unique_identifier,
                    field,
                    "Signature Verify",
                )?;
            }
            common::VALIDITY_INDICATOR => {
                common::parse_validity_indicator(
                    &mut validity_indicator,
                    field,
                    "Signature Verify",
                )?;
            }
            common::DATA => {
                common::parse_optional_secret(&mut recovered_data, field, "Signature Verify")?;
            }
            common::CORRELATION_VALUE => {
                common::parse_optional_secret(&mut correlation_value, field, "Signature Verify")?;
            }
            _ => {}
        }
    }

    common::validate_validity_indicator_shape(validity_indicator, context, "Signature Verify")?;

    Ok(ParsedSignatureVerifyPayload {
        unique_identifier: unique_identifier
            .ok_or_else(|| common::response_shape_error("Signature Verify"))?,
        validity_indicator,
        recovered_data,
        correlation_value,
    })
}

fn append_framing(
    payload: &mut Structure,
    correlation: Option<SecretBytes>,
    init: Option<bool>,
    final_part: Option<bool>,
) -> Result<(), ProtocolError> {
    if let Some(correlation) = correlation {
        common::push(
            payload,
            common::CORRELATION_VALUE,
            common::secret_value(correlation),
        )?;
    }
    if let Some(init) = init {
        common::push(payload, common::INIT_INDICATOR, Value::boolean(init))?;
    }
    if let Some(final_part) = final_part {
        common::push(payload, common::FINAL_INDICATOR, Value::boolean(final_part))?;
    }
    Ok(())
}
