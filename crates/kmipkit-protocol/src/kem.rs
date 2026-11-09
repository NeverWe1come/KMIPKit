//! KMIP 3.0 Deactivate and KEM operation payload models (KMIPKIT-0050).

use std::fmt;

use kmipkit_ttlv::{Item, RawTag, Structure, StructureView, Value, ValueView};

use crate::cryptographic_operation::OwnedResponseTtlv;
use crate::{
    AttributeSet, CryptographicParameters, KmipOperationResult, ResponseBatchItemView,
    UniqueIdentifier,
    kmip30_operations::{
        Kmip30OperationError, parse_success_identifier, parse_typed_success_response,
    },
};

/// TTLV Tag for Unique Identifier (0x00420094).
pub const UNIQUE_IDENTIFIER_TAG: u32 = 0x0042_0094;
/// TTLV Tag for Data (0x004200C2), used by the KEM operations.
pub const DATA_TAG: u32 = 0x0042_00C2;
const CRYPTOGRAPHIC_PARAMETERS_TAG: u32 = 0x0042_002B;
const ATTRIBUTES_TAG: u32 = 0x0042_0125;
const DEACTIVATION_REASON_TAG: u32 = 0x0042_01B8;
const DEACTIVATION_DATE_TAG: u32 = 0x0042_002F;
const DEACTIVATE_OPERATION: u32 = 0x0000_0040;
const ENCAPSULATE_OPERATION: u32 = 0x0000_0041;
const DECAPSULATE_OPERATION: u32 = 0x0000_0042;
fn ttlv_item(raw_tag: u32, value: Value) -> Result<Item, String> {
    let tag = RawTag::new(raw_tag)
        .map_err(|error| error.to_string())?
        .try_checked()
        .map_err(|error| error.to_string())?;
    Item::new(tag, value).map_err(|error| error.to_string())
}

fn push(payload: &mut Structure, raw_tag: u32, value: Value) -> Result<(), String> {
    payload
        .try_push(ttlv_item(raw_tag, value)?)
        .map_err(|error| error.to_string())
}

fn identifier_value(identifier: UniqueIdentifier) -> Value {
    match identifier {
        UniqueIdentifier::TextString(value) => Value::text_string(value),
        UniqueIdentifier::Enumeration(value) => Value::enumeration(value),
        UniqueIdentifier::Integer(value) => Value::integer(value),
    }
}

fn single_uid_payload(identifier: UniqueIdentifier) -> Result<Structure, String> {
    let mut payload = Structure::new();
    push(
        &mut payload,
        UNIQUE_IDENTIFIER_TAG,
        identifier_value(identifier),
    )?;
    Ok(payload)
}

fn append_attributes(payload: &mut Structure, attributes: AttributeSet) -> Result<(), String> {
    let mut fields = Structure::new();
    for attribute in attributes.into_items() {
        fields
            .try_push(attribute)
            .map_err(|error| error.to_string())?;
    }
    push(payload, ATTRIBUTES_TAG, Value::structure(fields))
}

fn append_cryptographic_parameters(
    payload: &mut Structure,
    parameters: Structure,
) -> Result<(), String> {
    crate::cryptographic_parameters::validate_known_cryptographic_parameters_view(
        &parameters.view(),
    )
    .map_err(|error| error.to_string())?;
    let parameters =
        crate::cryptographic_parameters::validate_cryptographic_parameters(Some(parameters))
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "missing Cryptographic Parameters".to_owned())?;
    push(
        payload,
        CRYPTOGRAPHIC_PARAMETERS_TAG,
        Value::structure(parameters),
    )
}

/// A KMIP 3.0 Deactivate request from §6.1.14, Table 293.
pub struct DeactivateRequest {
    unique_identifier: UniqueIdentifier,
    deactivation_reason: Option<Structure>,
    deactivation_date: Option<i64>,
}

impl DeactivateRequest {
    /// Creates a Deactivate request with its required Unique Identifier.
    #[must_use]
    pub fn new(unique_identifier: impl Into<UniqueIdentifier>) -> Self {
        Self {
            unique_identifier: unique_identifier.into(),
            deactivation_reason: None,
            deactivation_date: None,
        }
    }

    /// Returns the exact Unique Identifier value.
    #[must_use]
    pub const fn unique_identifier(&self) -> &UniqueIdentifier {
        &self.unique_identifier
    }

    /// Adds the optional Deactivation Reason Structure.
    #[must_use]
    pub fn with_deactivation_reason(mut self, reason: Structure) -> Self {
        self.deactivation_reason = Some(reason);
        self
    }

    /// Adds the optional Deactivation Date in KMIP `DateTime` seconds since epoch.
    #[must_use]
    pub const fn with_deactivation_date(mut self, unix_seconds: i64) -> Self {
        self.deactivation_date = Some(unix_seconds);
        self
    }

    /// Serializes Table 293 fields in their defined order.
    ///
    /// # Errors
    ///
    /// Returns a sanitized TTLV model error.
    pub fn into_ttlv_payload(self) -> Result<Structure, String> {
        let mut payload = single_uid_payload(self.unique_identifier)?;
        if let Some(reason) = self.deactivation_reason {
            push(
                &mut payload,
                DEACTIVATION_REASON_TAG,
                Value::structure(reason),
            )?;
        }
        if let Some(date) = self.deactivation_date {
            push(&mut payload, DEACTIVATION_DATE_TAG, Value::date_time(date))?;
        }
        Ok(payload)
    }
}

impl fmt::Debug for DeactivateRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DeactivateRequest")
            .field("unique_identifier", &self.unique_identifier)
            .field(
                "has_deactivation_reason",
                &self.deactivation_reason.is_some(),
            )
            .field("has_deactivation_date", &self.deactivation_date.is_some())
            .finish()
    }
}

/// A typed KMIP 3.0 Deactivate response.
pub struct DeactivateResponse {
    result: KmipOperationResult,
    unique_identifier: Option<UniqueIdentifier>,
    response_ttlv: Option<OwnedResponseTtlv>,
}

impl DeactivateResponse {
    /// Creates a successful Deactivate response payload.
    #[must_use]
    pub fn new(unique_identifier: impl Into<UniqueIdentifier>) -> Self {
        Self {
            result: KmipOperationResult::success(),
            unique_identifier: Some(unique_identifier.into()),
            response_ttlv: None,
        }
    }

    /// Converts one validated Deactivate response item, retaining Pending results.
    ///
    /// # Errors
    ///
    /// Returns a redacted operation-response error.
    pub fn try_from_response_item(
        item: ResponseBatchItemView<'_>,
    ) -> Result<Self, Kmip30OperationError> {
        let (result, unique_identifier, response_ttlv) =
            parse_typed_success_response(item, DEACTIVATE_OPERATION, "Deactivate", |payload| {
                parse_success_identifier(&payload)
            })?;
        Ok(Self {
            result,
            unique_identifier,
            response_ttlv,
        })
    }

    /// Returns the exact KMIP operation result.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }

    /// Returns the successful response's exact Unique Identifier form.
    #[must_use]
    pub const fn unique_identifier(&self) -> Option<&UniqueIdentifier> {
        self.unique_identifier.as_ref()
    }

    /// Lends the complete original successful response item, including unknown fields.
    ///
    /// Failure and Pending results do not retain a response item.
    pub fn with_ttlv<R>(&self, callback: impl for<'a> FnOnce(StructureView<'a>) -> R) -> Option<R> {
        self.response_ttlv
            .as_ref()
            .map(|response_ttlv| response_ttlv.with_ttlv(callback))
    }

    /// Serializes the response payload.
    ///
    /// # Errors
    ///
    /// Returns a sanitized TTLV model error.
    pub fn into_ttlv_payload(self) -> Result<Structure, String> {
        match self.unique_identifier {
            Some(identifier) => single_uid_payload(identifier),
            None => Ok(Structure::new()),
        }
    }
}

impl fmt::Debug for DeactivateResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DeactivateResponse")
            .field("result", &self.result)
            .field("unique_identifier", &self.unique_identifier)
            .finish_non_exhaustive()
    }
}

/// A KMIP 3.0 Encapsulate request (PQC KEM), serialized in Table 317 order.
pub struct EncapsulateRequest {
    unique_identifier: UniqueIdentifier,
    cryptographic_parameters: Option<Structure>,
    attributes: Option<AttributeSet>,
}

impl EncapsulateRequest {
    /// Creates a request with its required Unique Identifier.
    #[must_use]
    pub fn new(unique_identifier: impl Into<UniqueIdentifier>) -> Self {
        Self {
            unique_identifier: unique_identifier.into(),
            cryptographic_parameters: None,
            attributes: None,
        }
    }

    /// Returns the request's exact Unique Identifier form.
    #[must_use]
    pub const fn unique_identifier(&self) -> &UniqueIdentifier {
        &self.unique_identifier
    }

    /// Supplies optional generic Cryptographic Parameters.
    #[must_use]
    pub fn with_cryptographic_parameters(mut self, parameters: Structure) -> Self {
        self.cryptographic_parameters = Some(parameters);
        self
    }

    /// Supplies optional typed Cryptographic Parameters.
    #[must_use]
    pub fn with_typed_cryptographic_parameters(self, parameters: CryptographicParameters) -> Self {
        self.with_cryptographic_parameters(parameters.into_ttlv())
    }

    /// Supplies the optional ordered Attributes Structure contents.
    #[must_use]
    pub fn with_attributes(mut self, attributes: AttributeSet) -> Self {
        self.attributes = Some(attributes);
        self
    }

    /// Serializes Table 317 members in source order.
    ///
    /// # Errors
    ///
    /// Returns a sanitized TTLV or Cryptographic Parameters validation error.
    pub fn into_ttlv_payload(self) -> Result<Structure, String> {
        let mut payload = single_uid_payload(self.unique_identifier)?;
        if let Some(parameters) = self.cryptographic_parameters {
            append_cryptographic_parameters(&mut payload, parameters)?;
        }
        if let Some(attributes) = self.attributes {
            append_attributes(&mut payload, attributes)?;
        }
        Ok(payload)
    }
}

impl fmt::Debug for EncapsulateRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EncapsulateRequest")
            .field("unique_identifier", &self.unique_identifier)
            .field(
                "has_cryptographic_parameters",
                &self.cryptographic_parameters.is_some(),
            )
            .field("has_attributes", &self.attributes.is_some())
            .finish()
    }
}

/// A typed KMIP 3.0 Encapsulate response (PQC KEM).
///
/// Table 318 returns the new Secret Data object's identifier and public
/// encapsulated output `Data`; the shared secret remains server-managed.
pub struct EncapsulateResponse {
    result: KmipOperationResult,
    unique_identifier: Option<UniqueIdentifier>,
    data: Option<Vec<u8>>,
    response_ttlv: Option<OwnedResponseTtlv>,
}

impl EncapsulateResponse {
    /// Creates a successful Encapsulate response payload.
    #[must_use]
    pub fn new(unique_identifier: impl Into<UniqueIdentifier>, data: Vec<u8>) -> Self {
        Self {
            result: KmipOperationResult::success(),
            unique_identifier: Some(unique_identifier.into()),
            data: Some(data),
            response_ttlv: None,
        }
    }

    /// Converts one validated Encapsulate response item, retaining Pending results.
    ///
    /// # Errors
    ///
    /// Returns a redacted operation-response error.
    pub fn try_from_response_item(
        item: ResponseBatchItemView<'_>,
    ) -> Result<Self, Kmip30OperationError> {
        let (result, parsed, response_ttlv) =
            parse_typed_success_response(item, ENCAPSULATE_OPERATION, "Encapsulate", |payload| {
                parse_encapsulate_payload(&payload)
            })?;
        let (unique_identifier, data) = parsed.map_or((None, None), |(identifier, data)| {
            (Some(identifier), Some(data))
        });
        Ok(Self {
            result,
            unique_identifier,
            data,
            response_ttlv,
        })
    }

    /// Returns the exact KMIP operation result.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }

    /// Returns the successful response's exact Unique Identifier form.
    #[must_use]
    pub const fn unique_identifier(&self) -> Option<&UniqueIdentifier> {
        self.unique_identifier.as_ref()
    }

    /// Returns Encapsulate's public peer-facing ciphertext output, if successful.
    #[must_use]
    pub fn data(&self) -> Option<&[u8]> {
        self.data.as_deref()
    }

    /// Lends the complete original successful response item, including unknown fields.
    ///
    /// Failure and Pending results do not retain a response item.
    pub fn with_ttlv<R>(&self, callback: impl for<'a> FnOnce(StructureView<'a>) -> R) -> Option<R> {
        self.response_ttlv
            .as_ref()
            .map(|response_ttlv| response_ttlv.with_ttlv(callback))
    }

    /// Serializes the response fields in Table 318 order.
    ///
    /// # Errors
    ///
    /// Returns a sanitized TTLV model error.
    pub fn into_ttlv_payload(self) -> Result<Structure, String> {
        let mut payload = Structure::new();
        if let Some(identifier) = self.unique_identifier {
            push(
                &mut payload,
                UNIQUE_IDENTIFIER_TAG,
                identifier_value(identifier),
            )?;
        }
        if let Some(data) = self.data {
            push(&mut payload, DATA_TAG, Value::byte_string(data))?;
        }
        Ok(payload)
    }
}

impl fmt::Debug for EncapsulateResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EncapsulateResponse")
            .field("result", &self.result)
            .field("unique_identifier", &self.unique_identifier)
            .field("data_length", &self.data.as_ref().map(Vec::len))
            .finish_non_exhaustive()
    }
}

fn parse_encapsulate_payload(
    payload: &StructureView<'_>,
) -> Result<(UniqueIdentifier, Vec<u8>), Kmip30OperationError> {
    let mut identifier = None;
    let mut data = None;
    for field in payload.children() {
        match field.tag().raw() {
            UNIQUE_IDENTIFIER_TAG => {
                if identifier.is_some() {
                    return Err(Kmip30OperationError::MalformedSuccessPayload);
                }
                identifier = parse_identifier(field);
                if identifier.is_none() {
                    return Err(Kmip30OperationError::MalformedSuccessPayload);
                }
            }
            DATA_TAG => {
                if data.is_some() {
                    return Err(Kmip30OperationError::MalformedSuccessPayload);
                }
                data = field.with_value(|value| match value {
                    ValueView::ByteString(bytes) => Some(bytes.to_vec()),
                    _ => None,
                });
                if data.is_none() {
                    return Err(Kmip30OperationError::MalformedSuccessPayload);
                }
            }
            _ => {}
        }
    }
    match (identifier, data) {
        (Some(identifier), Some(data)) => Ok((identifier, data)),
        _ => Err(Kmip30OperationError::MalformedSuccessPayload),
    }
}

fn parse_decapsulate_response_payload(
    payload: &StructureView<'_>,
) -> Result<UniqueIdentifier, Kmip30OperationError> {
    let mut identifier = None;
    for field in payload.children() {
        match field.tag().raw() {
            UNIQUE_IDENTIFIER_TAG => {
                if identifier.is_some() {
                    return Err(Kmip30OperationError::MalformedSuccessPayload);
                }
                identifier = parse_identifier(field);
                if identifier.is_none() {
                    return Err(Kmip30OperationError::MalformedSuccessPayload);
                }
            }
            // KMIP 3.0 Table 297 returns only Unique Identifier. In
            // particular, Decapsulate must never return its shared secret.
            DATA_TAG => return Err(Kmip30OperationError::MalformedSuccessPayload),
            // Preserve forward-compatible/vendor extensions in the retained
            // response TTLV, but do not interpret them as standard fields.
            _ => {}
        }
    }
    identifier.ok_or(Kmip30OperationError::MalformedSuccessPayload)
}

fn parse_identifier(field: &Item) -> Option<UniqueIdentifier> {
    if field.tag().raw() != UNIQUE_IDENTIFIER_TAG {
        return None;
    }
    field.with_value(|value| match value {
        ValueView::TextString(value) => Some(UniqueIdentifier::TextString(value.to_owned())),
        ValueView::Enumeration(value) => Some(UniqueIdentifier::Enumeration(*value)),
        ValueView::Integer(value) => Some(UniqueIdentifier::Integer(*value)),
        _ => None,
    })
}

/// A KMIP 3.0 Decapsulate request (PQC KEM), serialized in Table 296 order.
pub struct DecapsulateRequest {
    unique_identifier: UniqueIdentifier,
    cryptographic_parameters: Option<Structure>,
    ciphertext: Vec<u8>,
    attributes: Option<AttributeSet>,
}

impl DecapsulateRequest {
    /// Creates a request with required Unique Identifier and encapsulated Data.
    #[must_use]
    pub fn new(unique_identifier: impl Into<UniqueIdentifier>, ciphertext: Vec<u8>) -> Self {
        Self {
            unique_identifier: unique_identifier.into(),
            cryptographic_parameters: None,
            ciphertext,
            attributes: None,
        }
    }

    /// Returns the exact Unique Identifier form.
    #[must_use]
    pub const fn unique_identifier(&self) -> &UniqueIdentifier {
        &self.unique_identifier
    }

    /// Returns the supplied encapsulated Data bytes.
    #[must_use]
    pub fn ciphertext(&self) -> &[u8] {
        &self.ciphertext
    }

    /// Supplies optional generic Cryptographic Parameters.
    #[must_use]
    pub fn with_cryptographic_parameters(mut self, parameters: Structure) -> Self {
        self.cryptographic_parameters = Some(parameters);
        self
    }

    /// Supplies optional typed Cryptographic Parameters.
    #[must_use]
    pub fn with_typed_cryptographic_parameters(self, parameters: CryptographicParameters) -> Self {
        self.with_cryptographic_parameters(parameters.into_ttlv())
    }

    /// Supplies the optional ordered Attributes Structure contents.
    #[must_use]
    pub fn with_attributes(mut self, attributes: AttributeSet) -> Self {
        self.attributes = Some(attributes);
        self
    }

    /// Serializes Table 296 members in their defined order.
    ///
    /// # Errors
    ///
    /// Returns a sanitized TTLV or Cryptographic Parameters validation error.
    pub fn into_ttlv_payload(self) -> Result<Structure, String> {
        let mut payload = single_uid_payload(self.unique_identifier)?;
        if let Some(parameters) = self.cryptographic_parameters {
            append_cryptographic_parameters(&mut payload, parameters)?;
        }
        push(&mut payload, DATA_TAG, Value::byte_string(self.ciphertext))?;
        if let Some(attributes) = self.attributes {
            append_attributes(&mut payload, attributes)?;
        }
        Ok(payload)
    }
}

impl fmt::Debug for DecapsulateRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DecapsulateRequest")
            .field("unique_identifier", &self.unique_identifier)
            .field("ciphertext_length", &self.ciphertext.len())
            .field(
                "has_cryptographic_parameters",
                &self.cryptographic_parameters.is_some(),
            )
            .field("has_attributes", &self.attributes.is_some())
            .finish()
    }
}

/// A typed KMIP 3.0 Decapsulate response (PQC KEM).
///
/// The response returns only the new Secret Data identifier; its shared secret
/// remains server-managed. See §6.1.15, Table 297.
pub struct DecapsulateResponse {
    result: KmipOperationResult,
    unique_identifier: Option<UniqueIdentifier>,
    response_ttlv: Option<OwnedResponseTtlv>,
}

impl DecapsulateResponse {
    /// Creates a successful Decapsulate response payload.
    #[must_use]
    pub fn new(unique_identifier: impl Into<UniqueIdentifier>) -> Self {
        Self {
            result: KmipOperationResult::success(),
            unique_identifier: Some(unique_identifier.into()),
            response_ttlv: None,
        }
    }

    /// Converts one validated Decapsulate response item, retaining Pending results.
    ///
    /// # Errors
    ///
    /// Returns a redacted operation-response error.
    pub fn try_from_response_item(
        item: ResponseBatchItemView<'_>,
    ) -> Result<Self, Kmip30OperationError> {
        let (result, unique_identifier, response_ttlv) =
            parse_typed_success_response(item, DECAPSULATE_OPERATION, "Decapsulate", |payload| {
                parse_decapsulate_response_payload(&payload)
            })?;
        Ok(Self {
            result,
            unique_identifier,
            response_ttlv,
        })
    }

    /// Returns the exact KMIP operation result.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }

    /// Returns the successful response's exact Unique Identifier form.
    #[must_use]
    pub const fn unique_identifier(&self) -> Option<&UniqueIdentifier> {
        self.unique_identifier.as_ref()
    }

    /// Lends the complete original successful response item, including unknown fields.
    ///
    /// Failure and Pending results do not retain a response item.
    pub fn with_ttlv<R>(&self, callback: impl for<'a> FnOnce(StructureView<'a>) -> R) -> Option<R> {
        self.response_ttlv
            .as_ref()
            .map(|response_ttlv| response_ttlv.with_ttlv(callback))
    }

    /// Serializes the Table 297 response payload.
    ///
    /// # Errors
    ///
    /// Returns a sanitized TTLV model error.
    pub fn into_ttlv_payload(self) -> Result<Structure, String> {
        match self.unique_identifier {
            Some(identifier) => single_uid_payload(identifier),
            None => Ok(Structure::new()),
        }
    }
}

impl fmt::Debug for DecapsulateResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DecapsulateResponse")
            .field("result", &self.result)
            .field("unique_identifier", &self.unique_identifier)
            .finish_non_exhaustive()
    }
}
