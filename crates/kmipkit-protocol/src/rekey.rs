//! Typed KMIP 2.1 Re-key request and response models.

use std::error::Error;
use std::fmt;

use kmipkit_ttlv::{Structure, StructureView, Value};

use crate::cryptographic_operation::{self as common, OwnedResponseTtlv};
use crate::{
    KmipOperationResult, ProtocolError, ResponseBatchItemView, ResultMessage,
    ResultValidationError, UniqueIdentifier,
};

const OPERATION: u32 = 0x0000_0004;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const OFFSET: u32 = 0x0042_0058;
const ATTRIBUTES: u32 = 0x0042_0125;
const PROTECTION_STORAGE_MASKS: u32 = 0x0042_015F;
const SYMMETRIC_KEY_OBJECT_TYPE: u32 = 2;
const SUCCESS: u32 = 0;

/// A KMIP 2.1 Re-key request from §6.1.46, Table 305.
///
/// The server creates the replacement key, applies source-defined date and
/// attribute rules, and maintains the Replacement Object and Replaced Key
/// links. `KMIPKit` forwards the selected identifier, optional Offset, and
/// optional attributes without generating key material or tracking remote
/// object state.
pub struct RekeyRequest {
    unique_identifier: Option<UniqueIdentifier>,
    offset: Option<u32>,
    attributes: Option<Structure>,
    protection_storage_masks: Option<Structure>,
}

impl RekeyRequest {
    /// Creates a Re-key request with all optional fields omitted.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            unique_identifier: None,
            offset: None,
            attributes: None,
            protection_storage_masks: None,
        }
    }

    /// Selects the existing Symmetric Key to re-key.
    #[must_use]
    pub fn with_unique_identifier(mut self, value: UniqueIdentifier) -> Self {
        self.unique_identifier = Some(value);
        self
    }

    /// Returns the optional explicit existing key identifier.
    #[must_use]
    pub const fn unique_identifier(&self) -> Option<&UniqueIdentifier> {
        self.unique_identifier.as_ref()
    }

    /// Supplies the server-side date difference for the replacement key.
    #[must_use]
    pub const fn with_offset(mut self, value: u32) -> Self {
        self.offset = Some(value);
        self
    }

    /// Supplies the ordered generic Attributes Structure.
    #[must_use]
    pub fn with_attributes(mut self, value: Structure) -> Self {
        self.attributes = Some(value);
        self
    }

    /// Supplies the ordered generic Protection Storage Masks Structure.
    #[must_use]
    pub fn with_protection_storage_masks(mut self, value: Structure) -> Self {
        self.protection_storage_masks = Some(value);
        self
    }

    /// Builds the ordered Request Payload from §6.1.46, Table 305.
    ///
    /// # Errors
    ///
    /// Returns a sanitized protocol error if source-defined Attribute
    /// cardinality rules are violated or the generic TTLV model cannot
    /// represent the payload.
    pub fn into_ttlv_payload(self) -> Result<Structure, ProtocolError> {
        if let Some(attributes) = self.attributes.as_ref() {
            let view = attributes.view();
            crate::attribute::validate_single_instance_attribute_items(
                view.children(),
                Some(SYMMETRIC_KEY_OBJECT_TYPE),
                true,
            )
            .map_err(|_| common::request_error())?;
        }

        let mut payload = Structure::new();
        if let Some(identifier) = self.unique_identifier {
            common::push(
                &mut payload,
                UNIQUE_IDENTIFIER,
                common::unique_identifier_value(identifier),
            )?;
        }
        if let Some(offset) = self.offset {
            common::push(&mut payload, OFFSET, Value::interval(offset))?;
        }
        if let Some(attributes) = self.attributes {
            common::push(&mut payload, ATTRIBUTES, Value::structure(attributes))?;
        }
        if let Some(masks) = self.protection_storage_masks {
            common::push(
                &mut payload,
                PROTECTION_STORAGE_MASKS,
                Value::structure(masks),
            )?;
        }
        Ok(payload)
    }
}

impl Default for RekeyRequest {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for RekeyRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RekeyRequest")
            .field("has_unique_identifier", &self.unique_identifier.is_some())
            .field("offset", &self.offset)
            .field("has_attributes", &self.attributes.is_some())
            .field(
                "has_protection_storage_masks",
                &self.protection_storage_masks.is_some(),
            )
            .finish()
    }
}

/// A typed Re-key result from §6.1.46, Table 306.
///
/// The complete generic response item is retained for callback-scoped
/// inspection, including unknown or future fields.
#[derive(Debug)]
pub struct RekeyResponse {
    result: KmipOperationResult,
    response_ttlv: OwnedResponseTtlv,
    unique_identifier: Option<UniqueIdentifier>,
}

impl RekeyResponse {
    /// Converts one already validated Re-key response batch item.
    ///
    /// A successful result requires exactly one well-formed Unique Identifier.
    /// Non-success results preserve their common status, reason, and message.
    ///
    /// # Errors
    ///
    /// Returns [`RekeyError`] for a different operation, malformed result, or
    /// malformed successful payload. Error text never includes payload values.
    pub fn try_from_response_item(item: ResponseBatchItemView<'_>) -> Result<Self, RekeyError> {
        if item.operation() != Some(OPERATION) {
            return Err(RekeyError::UnexpectedOperation);
        }
        let status = item
            .result_status()
            .ok_or(RekeyError::MissingResultStatus)?;
        let result_message = item.with_result_message(|text| ResultMessage::new(text.to_owned()));
        let result = KmipOperationResult::new(status, item.result_reason(), result_message)
            .map_err(RekeyError::InvalidOperationResult)?;
        let response_ttlv = common::clone_response_item(item, "Re-key")
            .map_err(|_| RekeyError::MalformedResponseItem)?;

        let unique_identifier = if status.raw() == SUCCESS {
            Some(parse_success_identifier(item)?)
        } else {
            None
        };

        Ok(Self {
            result,
            response_ttlv,
            unique_identifier,
        })
    }

    /// Returns the exact KMIP operation result.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }

    /// Returns the replacement Symmetric Key Unique Identifier on success.
    #[must_use]
    pub const fn unique_identifier(&self) -> Option<&UniqueIdentifier> {
        self.unique_identifier.as_ref()
    }

    /// Lends the complete ordered generic response batch item.
    pub fn with_ttlv<R>(&self, callback: impl for<'a> FnOnce(StructureView<'a>) -> R) -> R {
        self.response_ttlv.with_ttlv(callback)
    }
}

/// A payload-free error converting a validated Re-key response item.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RekeyError {
    /// The response item is not the client-to-server Re-key operation.
    UnexpectedOperation,
    /// The validated response item did not expose a Result Status.
    MissingResultStatus,
    /// The represented KMIP result violates the shared result contract.
    InvalidOperationResult(ResultValidationError),
    /// The generic response item could not be retained.
    MalformedResponseItem,
    /// A successful response omitted or malformed its required identifier.
    MalformedSuccessPayload,
}

impl fmt::Display for RekeyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedOperation => formatter.write_str("response item is not Re-key"),
            Self::MissingResultStatus => formatter.write_str("Re-key result status is missing"),
            Self::InvalidOperationResult(cause) => {
                write!(formatter, "Re-key operation result is invalid: {cause}")
            }
            Self::MalformedResponseItem => formatter.write_str("Re-key response item is malformed"),
            Self::MalformedSuccessPayload => {
                formatter.write_str("successful Re-key response payload is malformed")
            }
        }
    }
}

impl Error for RekeyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidOperationResult(cause) => Some(cause),
            _ => None,
        }
    }
}

fn parse_success_identifier(
    item: ResponseBatchItemView<'_>,
) -> Result<UniqueIdentifier, RekeyError> {
    let payload = item
        .with_response_payload(|payload| super::lifecycle::successful_response_identifier(&payload))
        .ok_or(RekeyError::MalformedSuccessPayload)?;
    payload.ok_or(RekeyError::MalformedSuccessPayload)
}
