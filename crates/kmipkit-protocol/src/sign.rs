//! Typed KMIP 2.1 Sign request and response models.

use std::fmt;

use kmipkit_ttlv::{Structure, Value};

use crate::cryptographic_operation as common;
use crate::{
    CryptographicOperationError, CryptographicOperationErrorKind, KmipOperationResult,
    OperationData, ProtocolError, ResponseBatchItemView, SecretBytes, UniqueIdentifier,
};

const OPERATION: u32 = 0x0000_0021;

/// A Sign request from KMIP v2.1 §6.1.55, Table 334.
pub struct SignRequest {
    identifier: Option<UniqueIdentifier>,
    parameters: Option<Structure>,
    data: Option<OperationData>,
    digested_data: Option<SecretBytes>,
    correlation: Option<SecretBytes>,
    init: Option<bool>,
    final_part: Option<bool>,
}

impl SignRequest {
    /// Creates a Sign request with all optional fields omitted.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            identifier: None,
            parameters: None,
            data: None,
            digested_data: None,
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

    /// Supplies request Data (§7.9).
    #[must_use]
    pub fn with_data(mut self, value: OperationData) -> Self {
        self.data = Some(value);
        self
    }

    /// Supplies precomputed Digested Data (§6.1.55, Table 334).
    #[must_use]
    pub fn with_digested_data(mut self, value: SecretBytes) -> Self {
        self.digested_data = Some(value);
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
    /// Returns an error if single-part Data and Digested Data are both absent,
    /// or multipart framing conflicts with supplied input.
    pub fn validate_multipart_shape(&self) -> Result<(), ProtocolError> {
        common::validate_framing(
            self.data.is_some() || self.digested_data.is_some(),
            self.correlation.is_some(),
            self.init,
            self.final_part,
        )
    }

    /// Builds a TTLV payload for Table 334.
    ///
    /// # Errors
    ///
    /// Returns a sanitized error for invalid Cryptographic Parameters or
    /// invalid single-part/multipart framing.
    pub fn to_ttlv_payload(self) -> Result<Structure, ProtocolError> {
        common::validate_framing(
            self.data.is_some() || self.digested_data.is_some(),
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
        append_framing(&mut payload, self.correlation, self.init, self.final_part)?;
        Ok(payload)
    }
}

impl Default for SignRequest {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for SignRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SignRequest")
            .field("has_unique_identifier", &self.identifier.is_some())
            .field("has_cryptographic_parameters", &self.parameters.is_some())
            .field("has_data", &self.data.is_some())
            .field("has_digested_data", &self.digested_data.is_some())
            .field("has_correlation_value", &self.correlation.is_some())
            .field("init_indicator", &self.init)
            .field("final_indicator", &self.final_part)
            .finish()
    }
}

/// A typed Sign response for one KMIP response batch item.
#[derive(Debug)]
pub struct SignResponse {
    result: KmipOperationResult,
    unique_identifier: Option<UniqueIdentifier>,
    signature_data: Option<SecretBytes>,
    correlation_value: Option<SecretBytes>,
}

impl SignResponse {
    /// Converts a Pending Sign response result.
    ///
    /// # Errors
    /// Returns an error if the item is not a Pending Sign result or its common
    /// result fields are malformed.
    pub fn try_from_pending_response_item(
        item: ResponseBatchItemView<'_>,
    ) -> Result<Self, SignError> {
        Ok(Self {
            result: common::parse_result(item, OPERATION, "Sign", Some(true))?,
            unique_identifier: None,
            signature_data: None,
            correlation_value: None,
        })
    }

    /// Converts a completed Sign response item.
    ///
    /// # Errors
    /// Returns an error if the item is not a completed Sign result or its
    /// successful response payload is malformed.
    pub fn try_from_response_item(item: ResponseBatchItemView<'_>) -> Result<Self, SignError> {
        let result = common::parse_result(item, OPERATION, "Sign", Some(false))?;
        if result.status().raw() != common::SUCCESS {
            return Ok(Self {
                result,
                unique_identifier: None,
                signature_data: None,
                correlation_value: None,
            });
        }

        let parsed = item
            .with_response_payload(|payload| {
                common::parse_operation_output_payload(&payload, common::SIGNATURE_DATA, "Sign")
            })
            .ok_or_else(|| {
                CryptographicOperationError::new(
                    "Sign",
                    CryptographicOperationErrorKind::MissingSuccessPayload,
                )
            })??;
        Ok(Self {
            result,
            unique_identifier: Some(parsed.unique_identifier),
            signature_data: parsed.output_data,
            correlation_value: parsed.correlation_value,
        })
    }

    /// Returns the exact server-reported operation result.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }

    /// Returns the Unique Identifier reported by a successful response.
    #[must_use]
    pub const fn unique_identifier(&self) -> Option<&UniqueIdentifier> {
        self.unique_identifier.as_ref()
    }

    /// Returns Signature Data returned for a completed single-part operation.
    #[must_use]
    pub const fn signature_data(&self) -> Option<&SecretBytes> {
        self.signature_data.as_ref()
    }

    /// Returns the server-provided multipart Correlation Value, when present.
    #[must_use]
    pub const fn correlation_value(&self) -> Option<&SecretBytes> {
        self.correlation_value.as_ref()
    }
}

/// A sanitized error converting a Sign response item.
pub type SignError = CryptographicOperationError;

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
