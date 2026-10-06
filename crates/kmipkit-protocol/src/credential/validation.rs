//! Shared structural validation for typed Credential wrappers.

use kmipkit_ttlv::{StructureView, ValueView};

use super::{CredentialType, CredentialValidationError, CredentialValidationErrorKind};

const CREDENTIAL_TYPE: u32 = 0x0042_0024;
const CREDENTIAL_VALUE: u32 = 0x0042_0025;

/// The KMIP TTLV Item Type required by a known credential member.
#[derive(Clone, Copy)]
pub(super) enum FieldKind {
    /// A KMIP Text String.
    TextString,
    /// A KMIP Structure.
    Structure,
    /// A KMIP Enumeration.
    Enumeration,
    /// A KMIP Byte String.
    ByteString,
    /// A KMIP Date Time Extended value.
    DateTimeExtended,
}

/// A known member in one of the KMIP 2.1 credential tables.
#[derive(Clone, Copy)]
pub(super) struct FieldRule {
    pub(super) tag: u32,
    pub(super) kind: FieldKind,
    pub(super) required: bool,
}

pub(super) fn duplicate_field_error() -> CredentialValidationError {
    CredentialValidationError::new(CredentialValidationErrorKind::DuplicateField)
}

pub(super) fn missing_field_error() -> CredentialValidationError {
    CredentialValidationError::new(CredentialValidationErrorKind::MissingField)
}

pub(super) fn wrong_field_type_error() -> CredentialValidationError {
    CredentialValidationError::new(CredentialValidationErrorKind::WrongFieldType)
}

pub(super) fn field_out_of_order_error() -> CredentialValidationError {
    CredentialValidationError::new(CredentialValidationErrorKind::FieldOutOfOrder)
}

pub(super) fn empty_authentication_error() -> CredentialValidationError {
    CredentialValidationError::new(CredentialValidationErrorKind::EmptyAuthentication)
}

pub(super) fn invalid_ttlv_structure_error() -> CredentialValidationError {
    CredentialValidationError::new(CredentialValidationErrorKind::InvalidTtlvStructure)
}

/// Checks singleton cardinality, required members, and Item Types for a schema.
///
/// Unknown members are intentionally ignored here and remain in the owned TTLV
/// tree. Their order and payloads are therefore preserved by the typed wrapper.
pub(super) fn validate_fields(
    view: &StructureView<'_>,
    rules: &[FieldRule],
    at_least_one: &[u32],
) -> Result<(), CredentialValidationError> {
    let mut seen = vec![false; rules.len()];
    let mut group_member_seen = false;
    let mut last_known_member_index = None;

    for field in view.children() {
        let Some((rule_index, rule)) = rules
            .iter()
            .enumerate()
            .find(|(_, rule)| rule.tag == field.tag().raw())
        else {
            continue;
        };

        if seen[rule_index] {
            return Err(duplicate_field_error());
        }
        if last_known_member_index.is_some_and(|last| rule_index < last) {
            return Err(field_out_of_order_error());
        }
        seen[rule_index] = true;
        last_known_member_index = Some(rule_index);
        group_member_seen |= at_least_one.contains(&rule.tag);

        let matches_kind = field.with_value(|value| {
            matches!(
                (rule.kind, value),
                (FieldKind::TextString, ValueView::TextString(_))
                    | (FieldKind::Structure, ValueView::Structure(_))
                    | (FieldKind::Enumeration, ValueView::Enumeration(_))
                    | (FieldKind::ByteString, ValueView::ByteString(_))
                    | (FieldKind::DateTimeExtended, ValueView::DateTimeExtended(_))
            )
        });
        if !matches_kind {
            return Err(wrong_field_type_error());
        }
    }

    if rules
        .iter()
        .zip(seen.iter())
        .any(|(rule, was_seen)| rule.required && !was_seen)
        || (!at_least_one.is_empty() && !group_member_seen)
    {
        return Err(missing_field_error());
    }

    Ok(())
}

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
                    return Err(duplicate_field_error());
                }
                if credential_value_seen {
                    return Err(field_out_of_order_error());
                }
                let raw = field.with_value(|value| match value {
                    ValueView::Enumeration(raw) => Some(*raw),
                    _ => None,
                });
                let Some(raw) = raw else {
                    return Err(wrong_field_type_error());
                };
                credential_type = Some(CredentialType::from_raw(raw));
            }
            CREDENTIAL_VALUE => {
                if credential_value_seen {
                    return Err(duplicate_field_error());
                }
                if credential_type.is_none() {
                    return Err(field_out_of_order_error());
                }
                let is_structure =
                    field.with_value(|value| matches!(value, ValueView::Structure(_)));
                if !is_structure {
                    return Err(wrong_field_type_error());
                }
                credential_value_seen = true;
            }
            _ => {}
        }
    }

    let Some(credential_type) = credential_type else {
        return Err(missing_field_error());
    };
    if !credential_value_seen {
        return Err(missing_field_error());
    }
    Ok(credential_type)
}
