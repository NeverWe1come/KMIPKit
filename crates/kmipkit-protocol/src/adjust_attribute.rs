//! Typed KMIP 2.1 Adjust Attribute request and response models.

use std::error::Error;
use std::fmt;

use kmipkit_ttlv::{Structure, StructureView, Value};

use crate::asynchronous::{is_success, item, model_error};
use crate::attribute::copy_text_string;
use crate::{
    AttributeReference, KmipOperationResult, ProtocolCauseCategory, ProtocolError,
    ProtocolErrorKind, ResponseBatchItemView, ResultMessage, ResultValidationError,
};

const ADJUST_ATTRIBUTE_OPERATION: u32 = 0x0000_0030;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const ADJUSTMENT_TYPE: u32 = 0x0042_0158;
const ADJUSTMENT_VALUE: u32 = 0x0042_0162;

/// An Adjustment Type Enumeration that retains unknown raw values.
///
/// OASIS KMIP v2.1 §11.1, Table 429 assigns Increment (1), Decrement (2), and
/// Negate (3), plus the extension range `0x80000000..=0x8FFFFFFF`. Other raw
/// values remain representable for inspection but cannot be sent by a typed
/// Adjust Attribute request.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct AdjustmentType(u32);

impl AdjustmentType {
    /// Increment an eligible attribute value.
    pub const INCREMENT: Self = Self(1);
    /// Decrement an eligible attribute value.
    pub const DECREMENT: Self = Self(2);
    /// Negate an eligible Boolean attribute value.
    pub const NEGATE: Self = Self(3);

    /// Creates an Adjustment Type retaining the exact raw Enumeration value.
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Returns the exact raw Enumeration value.
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }

    const fn is_assigned_or_extension(self) -> bool {
        matches!(self.0, 1..=3 | 0x8000_0000..=0x8FFF_FFFF)
    }
}

/// A client-to-server Adjust Attribute request from OASIS KMIP v2.1 §6.1.3.
///
/// Table 170 preserves the optional Unique Identifier and Adjustment Value,
/// followed by the required Attribute Reference and Adjustment Type. The
/// client does not apply the adjustment locally or synthesize a default value.
#[derive(Debug)]
pub struct AdjustAttributeRequest {
    unique_identifier: Option<String>,
    attribute_reference: AttributeReference,
    adjustment_type: AdjustmentType,
    adjustment_value: Option<Value>,
}

impl AdjustAttributeRequest {
    /// Creates an Adjust Attribute request from the exact caller-supplied
    /// reference, type, and optional generic Adjustment Value.
    #[must_use]
    pub fn new(
        unique_identifier: Option<String>,
        attribute_reference: AttributeReference,
        adjustment_type: AdjustmentType,
        adjustment_value: Option<Value>,
    ) -> Self {
        Self {
            unique_identifier,
            attribute_reference,
            adjustment_type,
            adjustment_value,
        }
    }

    /// Returns the optional exact Unique Identifier.
    #[must_use]
    pub fn unique_identifier(&self) -> Option<&str> {
        self.unique_identifier.as_deref()
    }

    /// Returns the exact Attribute Reference selected by the caller.
    #[must_use]
    pub const fn attribute_reference(&self) -> &AttributeReference {
        &self.attribute_reference
    }

    /// Returns the raw-preserving Adjustment Type.
    #[must_use]
    pub const fn adjustment_type(&self) -> AdjustmentType {
        self.adjustment_type
    }

    /// Returns the optional generic Adjustment Value.
    #[must_use]
    pub const fn adjustment_value(&self) -> Option<&Value> {
        self.adjustment_value.as_ref()
    }

    /// Builds the ordered Request Payload Structure from §6.1.3, Table 170.
    ///
    /// # Errors
    ///
    /// Returns a sanitized protocol error if an Attribute Reference or fixed
    /// field cannot be represented, the generic value cannot be copied, or the
    /// Adjustment Type is Reserved by §11.1, Table 429.
    pub fn to_ttlv_payload(&self) -> Result<Structure, ProtocolError> {
        if !self.adjustment_type.is_assigned_or_extension() {
            return Err(ProtocolError::categorized(
                ProtocolErrorKind::InvalidValue,
                ProtocolCauseCategory::InvalidValue,
            ));
        }

        let mut payload = Structure::new();
        if let Some(unique_identifier) = &self.unique_identifier {
            payload
                .try_push(item(
                    UNIQUE_IDENTIFIER,
                    Value::text_string(unique_identifier.clone()),
                )?)
                .map_err(model_error)?;
        }
        payload
            .try_push(self.attribute_reference.to_ttlv_item()?)
            .map_err(model_error)?;
        payload
            .try_push(item(
                ADJUSTMENT_TYPE,
                Value::enumeration(self.adjustment_type.raw()),
            )?)
            .map_err(model_error)?;
        if let Some(adjustment_value) = &self.adjustment_value {
            let value = kmipkit_ttlv::try_clone_value(adjustment_value).map_err(model_error)?;
            payload
                .try_push(item(ADJUSTMENT_VALUE, value)?)
                .map_err(model_error)?;
        }

        Ok(payload)
    }
}

/// A typed Adjust Attribute response preserving the common result and
/// successful Unique Identifier.
#[derive(Debug)]
pub struct AdjustAttributeResponse {
    result: KmipOperationResult,
    unique_identifier: Option<String>,
}

impl AdjustAttributeResponse {
    /// Converts one validated Adjust Attribute response batch item.
    ///
    /// Successful payloads follow OASIS KMIP v2.1 §6.1.3, Table 171 and contain
    /// exactly the required Unique Identifier. Non-success results preserve
    /// Result Status, optional Result Reason, and optional Result Message.
    ///
    /// # Errors
    ///
    /// Returns [`AdjustAttributeError`] for another operation, invalid common
    /// result metadata, a missing successful payload, or a malformed payload.
    pub fn try_from_response_item(
        item: ResponseBatchItemView<'_>,
    ) -> Result<Self, AdjustAttributeError> {
        if item.operation() != Some(ADJUST_ATTRIBUTE_OPERATION) {
            return Err(AdjustAttributeError::UnexpectedOperation);
        }

        let status = item
            .result_status()
            .ok_or(AdjustAttributeError::MissingResultStatus)?;
        let message = item.with_result_message(|text| ResultMessage::new(text.to_owned()));
        let result = KmipOperationResult::new(status, item.result_reason(), message)
            .map_err(AdjustAttributeError::InvalidOperationResult)?;

        if !is_success(status) {
            return Ok(Self {
                result,
                unique_identifier: None,
            });
        }

        let unique_identifier = item
            .with_response_payload(|payload| parse_success_payload(&payload))
            .ok_or(AdjustAttributeError::MissingSuccessPayload)??;

        Ok(Self {
            result,
            unique_identifier: Some(unique_identifier),
        })
    }

    /// Returns the complete server-reported KMIP operation result.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }

    /// Returns the successful response's exact Unique Identifier, if present.
    ///
    /// Non-success results return `None`.
    #[must_use]
    pub fn unique_identifier(&self) -> Option<&str> {
        self.unique_identifier.as_deref()
    }
}

/// A payload-free Adjust Attribute response conversion error.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdjustAttributeError {
    /// The response item identifies a different KMIP operation.
    UnexpectedOperation,
    /// The validated response item does not expose Result Status.
    MissingResultStatus,
    /// Result Status, Result Reason, and Result Message violate the common contract.
    InvalidOperationResult(ResultValidationError),
    /// A successful response omitted its Response Payload.
    MissingSuccessPayload,
    /// A successful response payload does not match Table 171.
    MalformedSuccessPayload,
}

impl fmt::Display for AdjustAttributeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedOperation => {
                formatter.write_str("response item is not Adjust Attribute")
            }
            Self::MissingResultStatus => {
                formatter.write_str("Adjust Attribute result status is missing")
            }
            Self::InvalidOperationResult(cause) => {
                write!(
                    formatter,
                    "Adjust Attribute operation result is invalid: {cause}"
                )
            }
            Self::MissingSuccessPayload => {
                formatter.write_str("successful Adjust Attribute response has no payload")
            }
            Self::MalformedSuccessPayload => {
                formatter.write_str("Adjust Attribute response payload is malformed")
            }
        }
    }
}

impl Error for AdjustAttributeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidOperationResult(cause) => Some(cause),
            _ => None,
        }
    }
}

fn parse_success_payload(payload: &StructureView<'_>) -> Result<String, AdjustAttributeError> {
    let fields = payload.children();
    if fields.len() != 1 {
        return Err(AdjustAttributeError::MalformedSuccessPayload);
    }
    let field = fields
        .first()
        .ok_or(AdjustAttributeError::MalformedSuccessPayload)?;
    if field.tag().raw() != UNIQUE_IDENTIFIER {
        return Err(AdjustAttributeError::MalformedSuccessPayload);
    }
    field
        .with_value(|value| copy_text_string(&value))
        .ok_or(AdjustAttributeError::MalformedSuccessPayload)
}
