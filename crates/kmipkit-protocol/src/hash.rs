//! Typed KMIP 2.1 Hash request and response models.

use std::fmt;

use kmipkit_ttlv::{Item, Structure, StructureView, Value, ValueView};

use crate::cryptographic_operation as common;
use crate::{
    CryptographicOperationError, CryptographicOperationErrorKind,
    CryptographicOperationResponseContext, KmipOperationResult, OperationData, ProtocolError,
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
    ///
    /// # Errors
    /// Returns an error if required single-part Data is absent or multipart
    /// framing conflicts with the supplied Data.
    pub fn validate_multipart_shape(&self) -> Result<(), ProtocolError> {
        common::validate_framing(
            self.data.is_some(),
            self.correlation.is_some(),
            self.init,
            self.final_part,
        )
    }

    /// Returns the request-derived response framing context.
    #[must_use]
    pub fn response_context(&self) -> CryptographicOperationResponseContext {
        common::response_context_from_framing(
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
        validate_hashing_parameters(&self.parameters)?;
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
            .finish_non_exhaustive()
    }
}

/// A typed Hash response for one KMIP response batch item.
#[derive(Debug)]
pub struct HashResponse {
    result: KmipOperationResult,
    response_ttlv: Structure,
    data: Option<SecretBytes>,
    correlation_value: Option<SecretBytes>,
}

impl HashResponse {
    /// Converts a Pending Hash response result.
    ///
    /// # Errors
    /// Returns an error if the item is not a Pending Hash result or its common
    /// result fields are malformed.
    pub fn try_from_pending_response_item(
        item: ResponseBatchItemView<'_>,
    ) -> Result<Self, HashError> {
        let result = common::parse_result(item, OPERATION, "Hash", Some(true))?;
        let response_ttlv = common::clone_response_item(item, "Hash")?;
        Ok(Self {
            result,
            response_ttlv,
            data: None,
            correlation_value: None,
        })
    }

    /// Converts a completed Hash response item.
    ///
    /// # Errors
    /// Returns an error if the item is not a completed Hash result or its
    /// successful single-part response omits Data or its payload is malformed.
    pub fn try_from_response_item(item: ResponseBatchItemView<'_>) -> Result<Self, HashError> {
        Self::try_from_response_item_with_context(
            item,
            CryptographicOperationResponseContext::SinglePart,
        )
    }

    /// Converts a completed Hash response using the original request framing.
    ///
    /// The response-only [`Self::try_from_response_item`] convenience uses the
    /// single-part interpretation. Multipart callers should pass the context
    /// returned by [`HashRequest::response_context`]. Per KMIP v2.1 §6.1.24,
    /// Table 236, Data is required for single-part responses and absent for
    /// multipart responses.
    ///
    /// # Errors
    /// Returns an error if the item is not a completed Hash result or its
    /// successful response payload disagrees with the request context.
    pub fn try_from_response_item_with_context(
        item: ResponseBatchItemView<'_>,
        context: CryptographicOperationResponseContext,
    ) -> Result<Self, HashError> {
        let result = common::parse_result(item, OPERATION, "Hash", Some(false))?;
        let response_ttlv = common::clone_response_item(item, "Hash")?;
        if result.status().raw() != common::SUCCESS {
            return Ok(Self {
                result,
                response_ttlv,
                data: None,
                correlation_value: None,
            });
        }

        let parsed = item
            .with_response_payload(|payload| parse_success_payload(&payload))
            .ok_or_else(|| {
                response_error(CryptographicOperationErrorKind::MissingSuccessPayload)
            })??;
        common::validate_operation_output_shape(parsed.data.is_some(), context, "Hash")?;
        Ok(Self {
            result,
            response_ttlv,
            data: parsed.data,
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

    /// Returns the optional digest Data returned by the server.
    #[must_use]
    pub const fn data(&self) -> Option<&SecretBytes> {
        self.data.as_ref()
    }

    /// Returns the optional multipart Correlation Value returned by the server.
    #[must_use]
    pub const fn correlation_value(&self) -> Option<&SecretBytes> {
        self.correlation_value.as_ref()
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

struct ParsedHashPayload {
    data: Option<SecretBytes>,
    correlation_value: Option<SecretBytes>,
}

fn parse_success_payload(payload: &StructureView<'_>) -> Result<ParsedHashPayload, HashError> {
    let mut data = None;
    let mut correlation_value = None;

    for field in payload.children() {
        match field.tag().raw() {
            common::DATA => parse_optional_secret(&mut data, field)?,
            common::CORRELATION_VALUE => parse_optional_secret(&mut correlation_value, field)?,
            _ => {}
        }
    }

    Ok(ParsedHashPayload {
        data,
        correlation_value,
    })
}

fn malformed_response() -> HashError {
    response_error(CryptographicOperationErrorKind::MalformedPayload)
}

fn response_error(kind: CryptographicOperationErrorKind) -> HashError {
    CryptographicOperationError::new("Hash", kind)
}

fn parse_optional_secret(slot: &mut Option<SecretBytes>, field: &Item) -> Result<(), HashError> {
    if slot.is_some() {
        return Err(malformed_response());
    }
    *slot = Some(common::parse_secret(field).ok_or_else(malformed_response)?);
    Ok(())
}

fn validate_hashing_parameters(parameters: &Structure) -> Result<(), ProtocolError> {
    common::validate_parameters(parameters)?;
    let includes_hashing_algorithm = parameters.view().children().iter().any(|field| {
        field.tag().raw() == common::HASHING_ALGORITHM
            && field.with_value(|value| matches!(value, ValueView::Enumeration(_)))
    });
    if includes_hashing_algorithm {
        Ok(())
    } else {
        Err(common::request_error())
    }
}
