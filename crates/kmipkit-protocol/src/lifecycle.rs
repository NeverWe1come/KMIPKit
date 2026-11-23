//! Shared TTLV mechanics for managed-object lifecycle operations.

use kmipkit_ttlv::{Item, ModelError, RawTag, Structure, StructureView, Tag, Value};

use crate::{ProtocolCauseCategory, ProtocolError, ProtocolErrorKind, UniqueIdentifier};

const UNIQUE_IDENTIFIER: u32 = 0x0042_0094; // §11.56 Table 487.

pub(crate) fn request_payload(
    unique_identifier: Option<&UniqueIdentifier>,
) -> Result<Structure, ProtocolError> {
    let mut payload = Structure::new();
    if let Some(identifier) = unique_identifier {
        let value = identifier.clone().into_value();
        payload.try_push(item(value)?).map_err(model_error)?;
    }
    Ok(payload)
}

pub(crate) fn successful_response_identifier(
    payload: &StructureView<'_>,
) -> Option<UniqueIdentifier> {
    let mut identifier = None;
    for field in payload.children() {
        if field.tag().raw() == UNIQUE_IDENTIFIER {
            if identifier.is_some() {
                return None;
            }
            identifier = parse_unique_identifier(field);
            identifier.as_ref()?;
        }
    }
    identifier
}

pub(crate) fn parse_unique_identifier(field: &Item) -> Option<UniqueIdentifier> {
    field.with_value(UniqueIdentifier::from_value)
}

fn item(value: Value) -> Result<Item, ProtocolError> {
    Item::new(tag()?, value).map_err(model_error)
}

fn tag() -> Result<Tag, ProtocolError> {
    RawTag::new(UNIQUE_IDENTIFIER)
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

/// NIST SP 800-57 Key Lifecycle State operational permission guard.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NistKeyLifecycleGuard;

impl NistKeyLifecycleGuard {
    /// Validates whether an originating operation (encrypt, sign) is permitted.
    ///
    /// According to NIST SP 800-57 Part 1 Rev 5:
    /// - Only keys in the `Active` state are permitted to generate new signatures or ciphertexts.
    #[must_use]
    pub const fn can_originate(state: crate::KeyLifecycleState) -> bool {
        matches!(state, crate::KeyLifecycleState::Active)
    }

    /// Validates whether a recipient or processing operation (decrypt, verify) is permitted.
    ///
    /// According to NIST SP 800-57 Part 1 Rev 5:
    /// - Keys in `Active` or `Deactivated` states may decrypt or verify existing data.
    /// - Keys in `Compromised`, `Destroyed`, or `DestroyedCompromised` states are strictly prohibited.
    #[must_use]
    pub const fn can_process(state: crate::KeyLifecycleState) -> bool {
        matches!(
            state,
            crate::KeyLifecycleState::Active | crate::KeyLifecycleState::Deactivated
        )
    }
}
