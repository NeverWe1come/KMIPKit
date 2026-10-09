//! Typed KMIP 2.1 Get Attribute List request and response models.

use std::error::Error;
use std::fmt;

use kmipkit_ttlv::{Structure, StructureView, Value};

use crate::asynchronous::{is_success, item, model_error};
use crate::attribute::copy_text_string;
use crate::{
    AttributeReference, KmipOperationResult, ProtocolError, ResponseBatchItemView, ResultMessage,
    ResultValidationError,
};

const GET_ATTRIBUTE_LIST_OPERATION: u32 = 0x0000_000C;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const ATTRIBUTE_REFERENCE: u32 = 0x0042_013B;

/// A typed client-to-server Get Attribute List request.
///
/// OASIS KMIP v2.1 §6.1.21, Table 226 permits only an optional Unique
/// Identifier in this request. With no Attribute Reference selector, the
/// server returns the object's complete attribute-name list.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GetAttributeListRequest {
    unique_identifier: Option<String>,
}

impl GetAttributeListRequest {
    /// Creates a request preserving the optional Unique Identifier exactly.
    #[must_use]
    pub fn new(unique_identifier: Option<String>) -> Self {
        Self { unique_identifier }
    }

    /// Builds the ordered Get Attribute List Request Payload from §6.1.21,
    /// Table 226.
    ///
    /// The Unique Identifier is omitted when `None`; no Attribute Reference
    /// selector is added.
    ///
    /// # Errors
    ///
    /// Returns a sanitized protocol error if the fixed Unique Identifier tag
    /// is not an allocated KMIP tag or the generic TTLV Structure rejects the
    /// field.
    pub fn to_ttlv_payload(&self) -> Result<Structure, ProtocolError> {
        let mut payload = Structure::new();
        if let Some(unique_identifier) = &self.unique_identifier {
            payload
                .try_push(item(
                    UNIQUE_IDENTIFIER,
                    Value::text_string(unique_identifier.clone()),
                )?)
                .map_err(model_error)?;
        }
        Ok(payload)
    }
}

/// A typed result for one client-to-server Get Attribute List response item.
///
/// Successful responses expose the required Unique Identifier and the
/// complete ordered list of Attribute References. Repeated references are
/// retained. Non-success results preserve the common KMIP result and expose no
/// operation payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GetAttributeListResponse {
    result: KmipOperationResult,
    unique_identifier: Option<String>,
    attribute_references: Option<Vec<AttributeReference>>,
}

impl GetAttributeListResponse {
    /// Converts one validated Get Attribute List response batch item.
    ///
    /// Successful payloads follow OASIS KMIP v2.1 §6.1.21, Table 227: a
    /// required Unique Identifier followed by one or more Attribute Reference
    /// values. References are parsed using the two lossless forms in §5.5,
    /// Table 161 and kept in wire order. Non-success results retain their
    /// Result Status, optional Result Reason, and optional Result Message
    /// without requiring an operation payload.
    ///
    /// # Errors
    ///
    /// Returns [`GetAttributeListError`] for a different operation, an
    /// invalid common result, a missing successful payload, or a malformed
    /// successful payload.
    pub fn try_from_response_item(
        item: ResponseBatchItemView<'_>,
    ) -> Result<Self, GetAttributeListError> {
        if item.operation() != Some(GET_ATTRIBUTE_LIST_OPERATION) {
            return Err(GetAttributeListError::UnexpectedOperation);
        }

        let status = item
            .result_status()
            .ok_or(GetAttributeListError::MissingResultStatus)?;
        let result_message = item.with_result_message(|text| ResultMessage::new(text.to_owned()));
        let result = KmipOperationResult::new(status, item.result_reason(), result_message)
            .map_err(GetAttributeListError::InvalidOperationResult)?;

        if !is_success(status) {
            return Ok(Self {
                result,
                unique_identifier: None,
                attribute_references: None,
            });
        }

        let (unique_identifier, attribute_references) = item
            .with_response_payload(|payload| parse_success_payload(&payload))
            .ok_or(GetAttributeListError::MissingSuccessPayload)??;

        Ok(Self {
            result,
            unique_identifier: Some(unique_identifier),
            attribute_references: Some(attribute_references),
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

    /// Returns all successful Attribute References in wire order, if present.
    ///
    /// Table 227 permits repeated Attribute Reference values. This accessor
    /// retains every returned entry, including duplicates; non-success results
    /// return `None`. Inspect each entry with
    /// [`AttributeReference::tag_value`] for the tag form or
    /// [`AttributeReference::name_parts`] for the name form; exactly one form
    /// is present for each reference.
    #[must_use]
    pub fn attribute_references(&self) -> Option<&[AttributeReference]> {
        self.attribute_references.as_deref()
    }
}

/// A payload-free error converting a validated Get Attribute List response.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GetAttributeListError {
    /// A response item did not identify the client-to-server Get Attribute List operation.
    UnexpectedOperation,
    /// A validated response item did not expose a Result Status.
    MissingResultStatus,
    /// The Result Status, Reason, or Message combination is invalid.
    ///
    /// The typed [`ResultValidationError`] is retained as the error source.
    InvalidOperationResult(ResultValidationError),
    /// A successful response omitted its required Response Payload Structure.
    MissingSuccessPayload,
    /// A successful payload violated Table 227 field ordering, cardinality, or value types.
    MalformedSuccessPayload,
    /// An Attribute Reference value did not satisfy §5.5, Table 161.
    ///
    /// The source is a sanitized [`ProtocolError`] and contains no payload text.
    MalformedAttributeReference(ProtocolError),
}

impl fmt::Display for GetAttributeListError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedOperation => {
                formatter.write_str("response item is not Get Attribute List")
            }
            Self::MissingResultStatus => {
                formatter.write_str("Get Attribute List result status is missing")
            }
            Self::InvalidOperationResult(cause) => write!(
                formatter,
                "Get Attribute List operation result is invalid: {cause}"
            ),
            Self::MissingSuccessPayload => {
                formatter.write_str("successful Get Attribute List response has no payload")
            }
            Self::MalformedSuccessPayload => {
                formatter.write_str("Get Attribute List response payload is malformed")
            }
            Self::MalformedAttributeReference(cause) => write!(
                formatter,
                "Get Attribute List response contains a malformed Attribute Reference: {cause}"
            ),
        }
    }
}

impl Error for GetAttributeListError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidOperationResult(cause) => Some(cause),
            Self::MalformedAttributeReference(cause) => Some(cause),
            _ => None,
        }
    }
}

fn parse_success_payload(
    payload: &StructureView<'_>,
) -> Result<(String, Vec<AttributeReference>), GetAttributeListError> {
    let mut unique_identifier = None;
    let mut attribute_references = Vec::new();

    for child in payload.children() {
        match child.tag().raw() {
            UNIQUE_IDENTIFIER if unique_identifier.is_none() && attribute_references.is_empty() => {
                unique_identifier = child.with_value(|value| copy_text_string(&value));
                if unique_identifier.is_none() {
                    return Err(GetAttributeListError::MalformedSuccessPayload);
                }
            }
            ATTRIBUTE_REFERENCE if unique_identifier.is_some() => {
                let reference = AttributeReference::try_from_ttlv_item(child)
                    .map_err(GetAttributeListError::MalformedAttributeReference)?;
                attribute_references.push(reference);
            }
            _ => return Err(GetAttributeListError::MalformedSuccessPayload),
        }
    }

    match (unique_identifier, attribute_references.is_empty()) {
        (Some(identifier), false) => Ok((identifier, attribute_references)),
        _ => Err(GetAttributeListError::MalformedSuccessPayload),
    }
}
