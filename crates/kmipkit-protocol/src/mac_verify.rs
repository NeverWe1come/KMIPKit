//! Typed KMIP 2.1 MAC Verify request and response models.

use std::fmt;

use kmipkit_ttlv::{Structure, StructureView, Value};

use crate::cryptographic_operation as common;
use crate::{
    CryptographicOperationError, KmipOperationResult, OperationData, ProtocolError,
    ResponseBatchItemView, SecretBytes, UniqueIdentifier, ValidityIndicator,
    VerificationResponseContext,
};

const OPERATION: u32 = 0x0000_0024;

/// A MAC Verify request from KMIP v2.1 §6.1.33, Table 262.
pub struct MacVerifyRequest {
    identifier: Option<UniqueIdentifier>,
    parameters: Option<Structure>,
    data: Option<OperationData>,
    mac_data: Option<SecretBytes>,
    correlation: Option<SecretBytes>,
    init: Option<bool>,
    final_part: Option<bool>,
}

impl MacVerifyRequest {
    /// Creates a MAC Verify request with all optional fields omitted.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            identifier: None,
            parameters: None,
            data: None,
            mac_data: None,
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

    /// Supplies optional original Data required by some verification algorithms.
    #[must_use]
    pub fn with_data(mut self, value: OperationData) -> Self {
        self.data = Some(value);
        self
    }

    /// Supplies single-part MAC Data (§7.20).
    #[must_use]
    pub fn with_mac_data(mut self, value: SecretBytes) -> Self {
        self.mac_data = Some(value);
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
    ///
    /// # Errors
    /// Returns an error if required single-part MAC Data is absent or
    /// multipart framing conflicts with the supplied MAC Data.
    pub fn validate_multipart_shape(&self) -> Result<(), ProtocolError> {
        common::validate_framing(
            self.mac_data.is_some(),
            self.correlation.is_some(),
            self.init,
            self.final_part,
        )
    }

    /// Returns the request-derived response context for Validity Indicator
    /// shape validation.
    #[must_use]
    pub fn verification_response_context(&self) -> VerificationResponseContext {
        common::response_context_from_framing(
            self.correlation.is_some(),
            self.init,
            self.final_part,
        )
    }

    /// Builds a TTLV payload for Table 262.
    ///
    /// # Errors
    ///
    /// Returns a sanitized error for invalid Cryptographic Parameters or
    /// invalid single-part/multipart framing.
    pub fn to_ttlv_payload(self) -> Result<Structure, ProtocolError> {
        common::validate_framing(
            self.mac_data.is_some(),
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
        if let Some(mac_data) = self.mac_data {
            common::push(
                &mut payload,
                common::MAC_DATA,
                common::secret_value(mac_data),
            )?;
        }
        append_framing(&mut payload, self.correlation, self.init, self.final_part)?;
        Ok(payload)
    }
}

impl Default for MacVerifyRequest {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for MacVerifyRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MacVerifyRequest")
            .field("has_unique_identifier", &self.identifier.is_some())
            .field("has_cryptographic_parameters", &self.parameters.is_some())
            .field("has_data", &self.data.is_some())
            .field("has_mac_data", &self.mac_data.is_some())
            .field("has_correlation_value", &self.correlation.is_some())
            .field("init_indicator", &self.init)
            .field("final_indicator", &self.final_part)
            .finish()
    }
}

/// A typed MAC Verify response for one KMIP response batch item.
#[derive(Debug)]
pub struct MacVerifyResponse {
    result: KmipOperationResult,
    response_ttlv: Structure,
    unique_identifier: Option<UniqueIdentifier>,
    validity_indicator: Option<ValidityIndicator>,
    correlation_value: Option<SecretBytes>,
}

impl MacVerifyResponse {
    /// Converts a Pending MAC Verify response result.
    ///
    /// # Errors
    /// Returns an error if the item is not a Pending MAC Verify result or its
    /// common result fields are malformed.
    pub fn try_from_pending_response_item(
        item: ResponseBatchItemView<'_>,
    ) -> Result<Self, MacVerifyError> {
        let result = common::parse_result(item, OPERATION, "MAC Verify", Some(true))?;
        let response_ttlv = common::clone_response_item(item, "MAC Verify")?;
        Ok(Self {
            result,
            response_ttlv,
            unique_identifier: None,
            validity_indicator: None,
            correlation_value: None,
        })
    }

    /// Converts a completed, single-part MAC Verify response item.
    ///
    /// Use [`Self::try_from_response_item_with_context`] when the response
    /// belongs to a multipart request.
    ///
    /// # Errors
    /// Returns an error if the item is not a completed MAC Verify result or
    /// its single-part response payload is malformed.
    pub fn try_from_response_item(item: ResponseBatchItemView<'_>) -> Result<Self, MacVerifyError> {
        Self::try_from_response_item_with_context(item, VerificationResponseContext::SinglePart)
    }

    /// Converts a completed MAC Verify response using its original request's
    /// single-part or multipart context.
    ///
    /// # Errors
    /// Returns an error if the item is not a completed MAC Verify result or
    /// its response payload violates the supplied request context.
    pub fn try_from_response_item_with_context(
        item: ResponseBatchItemView<'_>,
        context: VerificationResponseContext,
    ) -> Result<Self, MacVerifyError> {
        let result = common::parse_result(item, OPERATION, "MAC Verify", Some(false))?;
        let response_ttlv = common::clone_response_item(item, "MAC Verify")?;
        if result.status().raw() != common::SUCCESS {
            return Ok(Self {
                result,
                response_ttlv,
                unique_identifier: None,
                validity_indicator: None,
                correlation_value: None,
            });
        }

        let parsed = item
            .with_response_payload(|payload| parse_success_payload(&payload, context))
            .ok_or_else(|| {
                CryptographicOperationError::new(
                    "MAC Verify",
                    crate::CryptographicOperationErrorKind::MissingSuccessPayload,
                )
            })??;
        Ok(Self {
            result,
            response_ttlv,
            unique_identifier: Some(parsed.unique_identifier),
            validity_indicator: parsed.validity_indicator,
            correlation_value: parsed.correlation_value,
        })
    }

    /// Returns the exact server-reported operation result.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }

    /// Lends the complete ordered generic TTLV batch item, including unknown
    /// fields and values that the typed accessors do not interpret.
    pub fn with_ttlv<R>(&self, callback: impl for<'a> FnOnce(StructureView<'a>) -> R) -> R {
        callback(self.response_ttlv.view())
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

    /// Returns the optional multipart Correlation Value returned by the
    /// server.
    #[must_use]
    pub const fn correlation_value(&self) -> Option<&SecretBytes> {
        self.correlation_value.as_ref()
    }
}

/// A sanitized error converting a MAC Verify response item.
pub type MacVerifyError = CryptographicOperationError;

struct ParsedMacVerifyPayload {
    unique_identifier: UniqueIdentifier,
    validity_indicator: Option<ValidityIndicator>,
    correlation_value: Option<SecretBytes>,
}

fn parse_success_payload(
    payload: &kmipkit_ttlv::StructureView<'_>,
    context: VerificationResponseContext,
) -> Result<ParsedMacVerifyPayload, MacVerifyError> {
    let mut unique_identifier = None;
    let mut validity_indicator = None;
    let mut correlation_value = None;
    for field in payload.children() {
        match field.tag().raw() {
            common::UNIQUE_IDENTIFIER => {
                common::parse_required_identifier(&mut unique_identifier, field, "MAC Verify")?;
            }
            common::VALIDITY_INDICATOR => {
                common::parse_validity_indicator(&mut validity_indicator, field, "MAC Verify")?;
            }
            common::CORRELATION_VALUE => {
                common::parse_optional_secret(&mut correlation_value, field, "MAC Verify")?;
            }
            _ => {}
        }
    }

    common::validate_validity_indicator_shape(validity_indicator, context, "MAC Verify")?;

    Ok(ParsedMacVerifyPayload {
        unique_identifier: unique_identifier
            .ok_or_else(|| common::response_shape_error("MAC Verify"))?,
        validity_indicator,
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
