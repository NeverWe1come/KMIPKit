//! Shared structural validation for typed Credential wrappers.

use kmipkit_ttlv::{StructureView, ValueView};

use super::{CredentialType, CredentialValidationError, CredentialValidationErrorKind};

const CREDENTIAL_TYPE: u32 = 0x0042_0024;
const CREDENTIAL_VALUE: u32 = 0x0042_0025;

/// Validates the ordered outer fields shared by standalone and nested Credentials.
///
/// Unknown children remain permitted and are retained by the owning generic
/// TTLV tree. Credential Value contents are deliberately left opaque here.
pub(super) fn credential_type_from_view(
    view: &StructureView<'_>,
) -> Result<CredentialType, CredentialValidationError> {
    let mut credential_type = None;
    let mut credential_value_seen = false;

    for field in view.children() {
        match field.tag().raw() {
            CREDENTIAL_TYPE => {
                if credential_type.is_some() {
                    return Err(CredentialValidationError::new(
                        CredentialValidationErrorKind::DuplicateField,
                    ));
                }
                if credential_value_seen {
                    return Err(CredentialValidationError::new(
                        CredentialValidationErrorKind::FieldOutOfOrder,
                    ));
                }
                let raw = field.with_value(|value| match value {
                    ValueView::Enumeration(raw) => Some(*raw),
                    _ => None,
                });
                let Some(raw) = raw else {
                    return Err(CredentialValidationError::new(
                        CredentialValidationErrorKind::WrongFieldType,
                    ));
                };
                credential_type = Some(CredentialType::from_raw(raw));
            }
            CREDENTIAL_VALUE => {
                if credential_value_seen {
                    return Err(CredentialValidationError::new(
                        CredentialValidationErrorKind::DuplicateField,
                    ));
                }
                if credential_type.is_none() {
                    return Err(CredentialValidationError::new(
                        CredentialValidationErrorKind::FieldOutOfOrder,
                    ));
                }
                let is_structure =
                    field.with_value(|value| matches!(value, ValueView::Structure(_)));
                if !is_structure {
                    return Err(CredentialValidationError::new(
                        CredentialValidationErrorKind::WrongFieldType,
                    ));
                }
                credential_value_seen = true;
            }
            _ => {}
        }
    }

    let Some(credential_type) = credential_type else {
        return Err(CredentialValidationError::new(
            CredentialValidationErrorKind::MissingField,
        ));
    };
    if !credential_value_seen {
        return Err(CredentialValidationError::new(
            CredentialValidationErrorKind::MissingField,
        ));
    }
    Ok(credential_type)
}
