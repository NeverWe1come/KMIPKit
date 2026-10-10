//! Typed KMIP 2.1 MAC Verify request and response models.

use std::fmt;

use kmipkit_ttlv::{Structure, Value};

use crate::cryptographic_operation as common;
use crate::{
    CryptographicOperationError, KmipOperationResult, OperationData, ProtocolError,
    ResponseBatchItemView, SecretBytes, UniqueIdentifier,
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
    pub fn validate_multipart_shape(&self) -> Result<(), ProtocolError> {
        common::validate_framing(
            self.mac_data.is_some(),
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
}

impl MacVerifyResponse {
    /// Converts a Pending MAC Verify response result.
    pub fn try_from_pending_response_item(
        item: ResponseBatchItemView<'_>,
    ) -> Result<Self, MacVerifyError> {
        Ok(Self {
            result: common::parse_result(item, OPERATION, "MAC Verify", Some(true))?,
        })
    }

    /// Converts a completed MAC Verify response item.
    pub fn try_from_response_item(item: ResponseBatchItemView<'_>) -> Result<Self, MacVerifyError> {
        Ok(Self {
            result: common::parse_result(item, OPERATION, "MAC Verify", Some(false))?,
        })
    }

    /// Returns the exact server-reported operation result.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }
}

/// A sanitized error converting a MAC Verify response item.
pub type MacVerifyError = CryptographicOperationError;

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
