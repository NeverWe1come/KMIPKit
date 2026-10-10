//! Typed KMIP 2.1 Hash request and response models.

use std::fmt;

use kmipkit_ttlv::{Structure, Value};

use crate::cryptographic_operation as common;
use crate::{
    CryptographicOperationError, KmipOperationResult, OperationData, ProtocolError,
    ResponseBatchItemView, SecretBytes,
};

const OPERATION: u32 = 0x0000_0027;

/// A Hash request from KMIP v2.1 §6.1.24, Table 235.
pub struct HashRequest {
    parameters: Structure,
    data: Option<OperationData>,
    correlation: Option<SecretBytes>,
    init: Option<bool>,
    final_part: Option<bool>,
}

impl HashRequest {
    /// Creates a request with explicit, ordered Cryptographic Parameters.
    #[must_use]
    pub const fn new(parameters: Structure) -> Self {
        Self {
            parameters,
            data: None,
            correlation: None,
            init: None,
            final_part: None,
        }
    }

    /// Supplies the optional single-part Data value (§7.9).
    #[must_use]
    pub fn with_data(mut self, data: OperationData) -> Self {
        self.data = Some(data);
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
            self.data.is_some(),
            self.correlation.is_some(),
            self.init,
            self.final_part,
        )
    }

    /// Builds a TTLV payload for Table 235.
    ///
    /// # Errors
    ///
    /// Returns a sanitized error for invalid Cryptographic Parameters or
    /// invalid single-part/multipart framing.
    pub fn to_ttlv_payload(self) -> Result<Structure, ProtocolError> {
        common::validate_parameters(&self.parameters)?;
        common::validate_framing(
            self.data.is_some(),
            self.correlation.is_some(),
            self.init,
            self.final_part,
        )?;
        let mut payload = Structure::new();
        common::push(
            &mut payload,
            common::CRYPTOGRAPHIC_PARAMETERS,
            Value::structure(self.parameters),
        )?;
        if let Some(data) = self.data {
            common::push(
                &mut payload,
                common::DATA,
                common::operation_data_value(data),
            )?;
        }
        append_framing(&mut payload, self.correlation, self.init, self.final_part)?;
        Ok(payload)
    }
}

impl fmt::Debug for HashRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HashRequest")
            .field("has_cryptographic_parameters", &true)
            .field("has_data", &self.data.is_some())
            .field("has_correlation_value", &self.correlation.is_some())
            .field("init_indicator", &self.init)
            .field("final_indicator", &self.final_part)
            .finish()
    }
}

/// A typed Hash response for one KMIP response batch item.
#[derive(Debug)]
pub struct HashResponse {
    result: KmipOperationResult,
}

impl HashResponse {
    /// Converts a Pending Hash response result.
    pub fn try_from_pending_response_item(
        item: ResponseBatchItemView<'_>,
    ) -> Result<Self, HashError> {
        Ok(Self {
            result: common::parse_result(item, OPERATION, "Hash", Some(true))?,
        })
    }

    /// Converts a completed Hash response item.
    pub fn try_from_response_item(item: ResponseBatchItemView<'_>) -> Result<Self, HashError> {
        Ok(Self {
            result: common::parse_result(item, OPERATION, "Hash", Some(false))?,
        })
    }

    /// Returns the exact server-reported operation result.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }
}

/// A sanitized error converting a Hash response item.
pub type HashError = CryptographicOperationError;

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
