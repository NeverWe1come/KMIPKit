//! Typed KMIP 2.1 MAC request and response models.

use std::fmt;

use kmipkit_ttlv::Structure;

use crate::cryptographic_operation as common;
use crate::{
    CryptographicOperationError, KmipOperationResult, OperationData, ProtocolError,
    ResponseBatchItemView, SecretBytes, UniqueIdentifier,
};

const OPERATION: u32 = 0x0000_0023;

/// A MAC request from KMIP v2.1 §6.1.32, Table 259.
#[derive(Default)]
pub struct MacRequest {
    identifier: Option<UniqueIdentifier>,
    parameters: Option<Structure>,
    data: Option<OperationData>,
    correlation: Option<SecretBytes>,
    init: Option<bool>,
    final_part: Option<bool>,
}

impl MacRequest {
    /// Creates a MAC request with all optional fields omitted.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            identifier: None,
            parameters: None,
            data: None,
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

    /// Supplies single-part request Data (§7.9).
    #[must_use]
    pub fn with_data(mut self, value: OperationData) -> Self {
        self.data = Some(value);
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

    /// Builds a TTLV payload for Table 259.
    ///
    /// # Errors
    ///
    /// Returns a sanitized error for invalid Cryptographic Parameters or
    /// invalid single-part/multipart framing.
    pub fn to_ttlv_payload(self) -> Result<Structure, ProtocolError> {
        common::validate_framing(
            self.data.is_some(),
            self.correlation.is_some(),
            self.init,
            self.final_part,
        )?;
        let mut payload = Structure::new();
        common::encode_common_fields(
            &mut payload,
            self.identifier,
            self.parameters,
            self.data,
            self.correlation,
            self.init,
            self.final_part,
        )?;
        Ok(payload)
    }
}

impl fmt::Debug for MacRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MacRequest")
            .field("has_unique_identifier", &self.identifier.is_some())
            .field("has_cryptographic_parameters", &self.parameters.is_some())
            .field("has_data", &self.data.is_some())
            .field("has_correlation_value", &self.correlation.is_some())
            .field("init_indicator", &self.init)
            .field("final_indicator", &self.final_part)
            .finish()
    }
}

/// A typed MAC response for one KMIP response batch item.
#[derive(Debug)]
pub struct MacResponse {
    result: KmipOperationResult,
}

impl MacResponse {
    /// Converts a Pending MAC response result.
    pub fn try_from_pending_response_item(
        item: ResponseBatchItemView<'_>,
    ) -> Result<Self, MacError> {
        Ok(Self {
            result: common::parse_result(item, OPERATION, "MAC", Some(true))?,
        })
    }

    /// Converts a completed MAC response item.
    pub fn try_from_response_item(item: ResponseBatchItemView<'_>) -> Result<Self, MacError> {
        Ok(Self {
            result: common::parse_result(item, OPERATION, "MAC", Some(false))?,
        })
    }

    /// Returns the exact server-reported operation result.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }
}

/// A sanitized error converting a MAC response item.
pub type MacError = CryptographicOperationError;
