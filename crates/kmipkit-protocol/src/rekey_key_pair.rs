//! Typed KMIP 2.1 Re-key Key Pair request and response models.

use std::error::Error;
use std::fmt;

use kmipkit_ttlv::{Item, ModelError, RawTag, Structure, StructureView, Tag, Value};

use crate::cryptographic_operation::{self as common, OwnedResponseTtlv};
use crate::{
    AttributeSet, KmipOperationResult, ProtocolCauseCategory, ProtocolError, ProtocolErrorKind,
    ResponseBatchItemView, ResultMessage, ResultValidationError, UniqueIdentifier,
};

const OPERATION: u32 = 0x0000_001d;
const SUCCESS: u32 = 0;
const PRIVATE_KEY_UNIQUE_IDENTIFIER: u32 = 0x0042_0066;
const PUBLIC_KEY_UNIQUE_IDENTIFIER: u32 = 0x0042_006f;
const OFFSET: u32 = 0x0042_0058;
const COMMON_ATTRIBUTES: u32 = 0x0042_0126;
const PRIVATE_KEY_ATTRIBUTES: u32 = 0x0042_0127;
const PUBLIC_KEY_ATTRIBUTES: u32 = 0x0042_0128;
const COMMON_PROTECTION_STORAGE_MASKS: u32 = 0x0042_0163;
const PRIVATE_PROTECTION_STORAGE_MASKS: u32 = 0x0042_0164;
const PUBLIC_PROTECTION_STORAGE_MASKS: u32 = 0x0042_0165;
const PUBLIC_KEY_OBJECT_TYPE: u32 = 3;
const PRIVATE_KEY_OBJECT_TYPE: u32 = 4;

/// A typed Re-key Key Pair request from §6.1.47, Table 310.
///
/// The server creates the replacement key pair, applies the source-defined
/// attribute and lifecycle-date rules, and maintains replacement links.
/// `KMIPKit` forwards the optional private-key identifier, Offset, attribute
/// groups, and protection masks; it does not generate key material or track
/// remote lifecycle state.
pub struct RekeyKeyPairRequest {
    private_key_unique_identifier: Option<UniqueIdentifier>,
    offset: Option<u32>,
    common_attributes: Option<AttributeSet>,
    private_key_attributes: Option<AttributeSet>,
    public_key_attributes: Option<AttributeSet>,
    common_protection_storage_masks: Option<Structure>,
    private_protection_storage_masks: Option<Structure>,
    public_protection_storage_masks: Option<Structure>,
}

impl RekeyKeyPairRequest {
    /// Creates a request with all optional Table 310 fields omitted.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            private_key_unique_identifier: None,
            offset: None,
            common_attributes: None,
            private_key_attributes: None,
            public_key_attributes: None,
            common_protection_storage_masks: None,
            private_protection_storage_masks: None,
            public_protection_storage_masks: None,
        }
    }

    /// Selects the existing key pair by its Private Key Unique Identifier.
    #[must_use]
    pub fn with_private_key_unique_identifier(mut self, value: UniqueIdentifier) -> Self {
        self.private_key_unique_identifier = Some(value);
        self
    }

    /// Returns the optional explicit existing Private Key Unique Identifier.
    #[must_use]
    pub const fn private_key_unique_identifier(&self) -> Option<&UniqueIdentifier> {
        self.private_key_unique_identifier.as_ref()
    }

    /// Supplies the optional server-side lifecycle date difference.
    #[must_use]
    pub const fn with_offset(mut self, value: u32) -> Self {
        self.offset = Some(value);
        self
    }

    /// Supplies the optional Attributes shared by the replacement key pair.
    #[must_use]
    pub fn with_common_attributes(mut self, value: AttributeSet) -> Self {
        self.common_attributes = Some(value);
        self
    }

    /// Supplies the optional Attributes for the replacement Private Key.
    #[must_use]
    pub fn with_private_key_attributes(mut self, value: AttributeSet) -> Self {
        self.private_key_attributes = Some(value);
        self
    }

    /// Supplies the optional Attributes for the replacement Public Key.
    #[must_use]
    pub fn with_public_key_attributes(mut self, value: AttributeSet) -> Self {
        self.public_key_attributes = Some(value);
        self
    }

    /// Supplies the optional masks shared by both replacement keys.
    #[must_use]
    pub fn with_common_protection_storage_masks(mut self, value: Structure) -> Self {
        self.common_protection_storage_masks = Some(value);
        self
    }

    /// Supplies the optional masks for the replacement Private Key.
    #[must_use]
    pub fn with_private_protection_storage_masks(mut self, value: Structure) -> Self {
        self.private_protection_storage_masks = Some(value);
        self
    }

    /// Supplies the optional masks for the replacement Public Key.
    #[must_use]
    pub fn with_public_protection_storage_masks(mut self, value: Structure) -> Self {
        self.public_protection_storage_masks = Some(value);
        self
    }

    /// Builds the ordered Request Payload from §6.1.47, Table 310.
    ///
    /// # Errors
    ///
    /// Returns a sanitized protocol error if source-defined Attribute
    /// cardinality rules are violated or a generic TTLV model constraint
    /// prevents representing the payload.
    pub fn into_ttlv_payload(self) -> Result<Structure, ProtocolError> {
        validate_attribute_group(
            self.common_attributes.as_ref(),
            Some(PRIVATE_KEY_OBJECT_TYPE),
        )?;
        validate_attribute_group(
            self.private_key_attributes.as_ref(),
            Some(PRIVATE_KEY_OBJECT_TYPE),
        )?;
        validate_attribute_group(
            self.public_key_attributes.as_ref(),
            Some(PUBLIC_KEY_OBJECT_TYPE),
        )?;

        let mut payload = Structure::new();
        if let Some(identifier) = self.private_key_unique_identifier {
            push(
                &mut payload,
                PRIVATE_KEY_UNIQUE_IDENTIFIER,
                common::unique_identifier_value(identifier),
            )?;
        }
        if let Some(offset) = self.offset {
            push(&mut payload, OFFSET, Value::interval(offset))?;
        }
        push_attribute_group(&mut payload, COMMON_ATTRIBUTES, self.common_attributes)?;
        push_attribute_group(
            &mut payload,
            PRIVATE_KEY_ATTRIBUTES,
            self.private_key_attributes,
        )?;
        push_attribute_group(
            &mut payload,
            PUBLIC_KEY_ATTRIBUTES,
            self.public_key_attributes,
        )?;
        push_structure_group(
            &mut payload,
            COMMON_PROTECTION_STORAGE_MASKS,
            self.common_protection_storage_masks,
        )?;
        push_structure_group(
            &mut payload,
            PRIVATE_PROTECTION_STORAGE_MASKS,
            self.private_protection_storage_masks,
        )?;
        push_structure_group(
            &mut payload,
            PUBLIC_PROTECTION_STORAGE_MASKS,
            self.public_protection_storage_masks,
        )?;
        Ok(payload)
    }
}

impl Default for RekeyKeyPairRequest {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for RekeyKeyPairRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RekeyKeyPairRequest")
            .field(
                "has_private_key_unique_identifier",
                &self.private_key_unique_identifier.is_some(),
            )
            .field("offset", &self.offset)
            .field("has_common_attributes", &self.common_attributes.is_some())
            .field(
                "has_private_key_attributes",
                &self.private_key_attributes.is_some(),
            )
            .field(
                "has_public_key_attributes",
                &self.public_key_attributes.is_some(),
            )
            .field(
                "has_common_protection_storage_masks",
                &self.common_protection_storage_masks.is_some(),
            )
            .field(
                "has_private_protection_storage_masks",
                &self.private_protection_storage_masks.is_some(),
            )
            .field(
                "has_public_protection_storage_masks",
                &self.public_protection_storage_masks.is_some(),
            )
            .finish()
    }
}

/// A typed Re-key Key Pair result from §6.1.47, Table 311.
#[derive(Debug)]
pub struct RekeyKeyPairResponse {
    result: KmipOperationResult,
    response_ttlv: OwnedResponseTtlv,
    private_key_unique_identifier: Option<UniqueIdentifier>,
    public_key_unique_identifier: Option<UniqueIdentifier>,
}

impl RekeyKeyPairResponse {
    /// Converts one validated Re-key Key Pair response batch item.
    ///
    /// Success requires one well-formed identifier for each replacement key.
    /// Non-success results preserve their common status, reason, and message.
    ///
    /// # Errors
    ///
    /// Returns [`RekeyKeyPairError`] for a different operation, malformed
    /// result, or malformed successful payload.
    pub fn try_from_response_item(
        item: ResponseBatchItemView<'_>,
    ) -> Result<Self, RekeyKeyPairError> {
        if item.operation() != Some(OPERATION) {
            return Err(RekeyKeyPairError::UnexpectedOperation);
        }
        let status = item
            .result_status()
            .ok_or(RekeyKeyPairError::MissingResultStatus)?;
        let result_message = item.with_result_message(|text| ResultMessage::new(text.to_owned()));
        let result = KmipOperationResult::new(status, item.result_reason(), result_message)
            .map_err(RekeyKeyPairError::InvalidOperationResult)?;
        let response_ttlv = common::clone_response_item(item, "Re-key Key Pair")
            .map_err(|_| RekeyKeyPairError::MalformedResponseItem)?;

        let (private_key_unique_identifier, public_key_unique_identifier) =
            if status.raw() == SUCCESS {
                let identifiers = item
                    .with_response_payload(|payload| parse_success_identifiers(&payload))
                    .ok_or(RekeyKeyPairError::MalformedSuccessPayload)??;
                (Some(identifiers.0), Some(identifiers.1))
            } else {
                (None, None)
            };

        Ok(Self {
            result,
            response_ttlv,
            private_key_unique_identifier,
            public_key_unique_identifier,
        })
    }

    /// Returns the exact KMIP operation result.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }

    /// Returns the replacement Private Key Unique Identifier on success.
    #[must_use]
    pub const fn private_key_unique_identifier(&self) -> Option<&UniqueIdentifier> {
        self.private_key_unique_identifier.as_ref()
    }

    /// Returns the replacement Public Key Unique Identifier on success.
    #[must_use]
    pub const fn public_key_unique_identifier(&self) -> Option<&UniqueIdentifier> {
        self.public_key_unique_identifier.as_ref()
    }

    /// Lends the complete ordered generic response batch item.
    pub fn with_ttlv<R>(&self, callback: impl for<'a> FnOnce(StructureView<'a>) -> R) -> R {
        self.response_ttlv.with_ttlv(callback)
    }
}

/// A payload-free error converting a validated Re-key Key Pair response item.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RekeyKeyPairError {
    /// The response item is not the client-to-server Re-key Key Pair operation.
    UnexpectedOperation,
    /// The validated response item did not expose a Result Status.
    MissingResultStatus,
    /// The represented KMIP result violates the shared result contract.
    InvalidOperationResult(ResultValidationError),
    /// The generic response item could not be retained.
    MalformedResponseItem,
    /// A successful response omitted or malformed either required identifier.
    MalformedSuccessPayload,
}

impl fmt::Display for RekeyKeyPairError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedOperation => {
                formatter.write_str("response item is not Re-key Key Pair")
            }
            Self::MissingResultStatus => {
                formatter.write_str("Re-key Key Pair result status is missing")
            }
            Self::InvalidOperationResult(cause) => {
                write!(
                    formatter,
                    "Re-key Key Pair operation result is invalid: {cause}"
                )
            }
            Self::MalformedResponseItem => {
                formatter.write_str("Re-key Key Pair response item is malformed")
            }
            Self::MalformedSuccessPayload => {
                formatter.write_str("successful Re-key Key Pair payload is malformed")
            }
        }
    }
}

impl Error for RekeyKeyPairError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidOperationResult(cause) => Some(cause),
            _ => None,
        }
    }
}

fn parse_success_identifiers(
    payload: &StructureView<'_>,
) -> Result<(UniqueIdentifier, UniqueIdentifier), RekeyKeyPairError> {
    let mut private_identifier = None;
    let mut public_identifier = None;
    for field in payload.children() {
        match field.tag().raw() {
            PRIVATE_KEY_UNIQUE_IDENTIFIER => {
                if private_identifier.is_some() {
                    return Err(RekeyKeyPairError::MalformedSuccessPayload);
                }
                private_identifier = crate::lifecycle::parse_unique_identifier(field);
                if private_identifier.is_none() {
                    return Err(RekeyKeyPairError::MalformedSuccessPayload);
                }
            }
            PUBLIC_KEY_UNIQUE_IDENTIFIER => {
                if public_identifier.is_some() {
                    return Err(RekeyKeyPairError::MalformedSuccessPayload);
                }
                public_identifier = crate::lifecycle::parse_unique_identifier(field);
                if public_identifier.is_none() {
                    return Err(RekeyKeyPairError::MalformedSuccessPayload);
                }
            }
            _ => {}
        }
    }
    match (private_identifier, public_identifier) {
        (Some(private), Some(public)) => Ok((private, public)),
        _ => Err(RekeyKeyPairError::MalformedSuccessPayload),
    }
}

fn push_attribute_group(
    payload: &mut Structure,
    raw_tag: u32,
    attributes: Option<AttributeSet>,
) -> Result<(), ProtocolError> {
    let Some(attributes) = attributes else {
        return Ok(());
    };
    let mut group = Structure::new();
    for attribute in attributes.into_items() {
        group.try_push(attribute).map_err(model_error)?;
    }
    push(payload, raw_tag, Value::structure(group))
}

fn validate_attribute_group(
    attributes: Option<&AttributeSet>,
    object_type: Option<u32>,
) -> Result<(), ProtocolError> {
    if let Some(attributes) = attributes {
        crate::attribute::validate_single_instance_attribute_items(
            attributes.as_items(),
            object_type,
            true,
        )
        .map_err(|_| common::request_error())?;
    }
    Ok(())
}

fn push_structure_group(
    payload: &mut Structure,
    raw_tag: u32,
    group: Option<Structure>,
) -> Result<(), ProtocolError> {
    if let Some(group) = group {
        push(payload, raw_tag, Value::structure(group))?;
    }
    Ok(())
}

fn push(payload: &mut Structure, raw_tag: u32, value: Value) -> Result<(), ProtocolError> {
    payload
        .try_push(Item::new(tag(raw_tag)?, value).map_err(model_error)?)
        .map_err(model_error)
}

fn tag(raw_tag: u32) -> Result<Tag, ProtocolError> {
    RawTag::new(raw_tag)
        .and_then(|raw| raw.try_checked())
        .map_err(model_error)
}

fn model_error(error: ModelError) -> ProtocolError {
    ProtocolError::new(
        ProtocolErrorKind::InvalidValue,
        ProtocolCauseCategory::InvalidValue,
        error,
    )
}
