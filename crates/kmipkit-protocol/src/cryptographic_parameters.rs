//! Shared, locally knowable Cryptographic Parameters validation.

use kmipkit_ttlv::{Structure, StructureView, ValueView};

use crate::{ProtocolCauseCategory, ProtocolError, ProtocolErrorKind};

const BLOCK_CIPHER_MODE: u32 = 0x0042_0011;
const CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;
const HASHING_ALGORITHM: u32 = 0x0042_0038;
const PADDING_METHOD: u32 = 0x0042_005F;
const KEY_ROLE_TYPE: u32 = 0x0042_0083;
const DIGITAL_SIGNATURE_ALGORITHM: u32 = 0x0042_00AE;
const RANDOM_IV: u32 = 0x0042_00C5;
const IV_LENGTH: u32 = 0x0042_00CD;
const TAG_LENGTH: u32 = 0x0042_00CE;
const FIXED_FIELD_LENGTH: u32 = 0x0042_00CF;
const COUNTER_LENGTH: u32 = 0x0042_00D0;
const INITIAL_COUNTER_VALUE: u32 = 0x0042_00D1;
const INVOCATION_FIELD_LENGTH: u32 = 0x0042_00D2;
const SALT_LENGTH: u32 = 0x0042_0100;
const MASK_GENERATOR: u32 = 0x0042_0101;
const MASK_GENERATOR_HASHING_ALGORITHM: u32 = 0x0042_0102;
const P_SOURCE: u32 = 0x0042_0103;
const TRAILER_FIELD: u32 = 0x0042_0104;

// OASIS KMIP v2.1 §11.6, Table 436. NIST SP 800-38C Appendix A.1 supports
// classifying CCM as variable-IV through its permitted example nonce lengths.
const CTR_MODE: u32 = 6;
const CCM_MODE: u32 = 8;
const GCM_MODE: u32 = 9;

/// Validates only §4.16 requirements that can be established from the supplied
/// Cryptographic Parameters Structure and returns the original Structure.
///
/// The supplied tree is never rebuilt or reordered. Unknown members, unknown
/// mode values, and the original encoded values remain intact. This validator
/// recognizes CTR (6), CCM (8), and GCM (9) as variable-IV modes requiring IV
/// Length. GCM also requires Tag Length; CCM does not. Other mode values are
/// preserved without inferred requirements. An omitted operation-level value
/// remains omitted, because the server may obtain parameters from object
/// attributes that the client cannot inspect.
///
/// This helper checks required-member presence only. It does not validate the
/// TTLV Item Types or multiplicity of known parameter members; typed request
/// conversion is responsible for those structural checks.
///
/// # Errors
///
/// Returns a sanitized `InvalidValue` error when supplied parameters identify
/// CTR, CCM, or GCM without IV Length, or GCM without Tag Length. The error
/// contains no member values or raw parameter data.
pub(crate) fn validate_cryptographic_parameters(
    parameters: Option<Structure>,
) -> Result<Option<Structure>, ProtocolError> {
    let Some(ref supplied) = parameters else {
        return Ok(None);
    };

    validate_cryptographic_parameters_view(&supplied.view())?;
    Ok(parameters)
}

/// Validates required-member presence while preserving the caller's borrowed
/// ordered Structure. Typed request boundaries use this when they validate a
/// generic payload without taking ownership of its nested values.
pub(crate) fn validate_cryptographic_parameters_view(
    supplied: &StructureView<'_>,
) -> Result<(), ProtocolError> {
    let mut requires_iv_length = false;
    let mut requires_tag_length = false;
    let mut has_iv_length = false;
    let mut has_tag_length = false;

    for item in supplied.children() {
        match item.tag().raw() {
            BLOCK_CIPHER_MODE => {
                let mode = item.with_value(|value| match value {
                    ValueView::Enumeration(mode) => Some(*mode),
                    _ => None,
                });
                match mode {
                    Some(CTR_MODE | CCM_MODE) => requires_iv_length = true,
                    Some(GCM_MODE) => {
                        requires_iv_length = true;
                        requires_tag_length = true;
                    }
                    Some(_) | None => {}
                }
            }
            IV_LENGTH => has_iv_length = true,
            TAG_LENGTH => has_tag_length = true,
            _ => {}
        }
    }

    if (requires_iv_length && !has_iv_length) || (requires_tag_length && !has_tag_length) {
        return Err(ProtocolError::categorized(
            ProtocolErrorKind::InvalidValue,
            ProtocolCauseCategory::InvalidValue,
        ));
    }

    Ok(())
}

/// Validates Table 59 member types and singleton cardinality while retaining
/// the original ordered Structure and every unknown member.
pub(super) fn validate_known_cryptographic_parameters_view(
    parameters: &StructureView<'_>,
) -> Result<(), ProtocolError> {
    let mut seen = [false; 18];
    for field in parameters.children() {
        let (index, valid_type) = match field.tag().raw() {
            BLOCK_CIPHER_MODE => (
                Some(0),
                field.with_value(|value| matches!(value, ValueView::Enumeration(_))),
            ),
            CRYPTOGRAPHIC_ALGORITHM => (
                Some(1),
                field.with_value(|value| matches!(value, ValueView::Enumeration(_))),
            ),
            HASHING_ALGORITHM => (
                Some(2),
                field.with_value(|value| matches!(value, ValueView::Enumeration(_))),
            ),
            PADDING_METHOD => (
                Some(3),
                field.with_value(|value| matches!(value, ValueView::Enumeration(_))),
            ),
            KEY_ROLE_TYPE => (
                Some(4),
                field.with_value(|value| matches!(value, ValueView::Enumeration(_))),
            ),
            DIGITAL_SIGNATURE_ALGORITHM => (
                Some(5),
                field.with_value(|value| matches!(value, ValueView::Enumeration(_))),
            ),
            RANDOM_IV => (
                Some(6),
                field.with_value(|value| matches!(value, ValueView::Boolean(_))),
            ),
            IV_LENGTH => (
                Some(7),
                field.with_value(|value| matches!(value, ValueView::Integer(_))),
            ),
            TAG_LENGTH => (
                Some(8),
                field.with_value(|value| matches!(value, ValueView::Integer(_))),
            ),
            FIXED_FIELD_LENGTH => (
                Some(9),
                field.with_value(|value| matches!(value, ValueView::Integer(_))),
            ),
            COUNTER_LENGTH => (
                Some(10),
                field.with_value(|value| matches!(value, ValueView::Integer(_))),
            ),
            INITIAL_COUNTER_VALUE => (
                Some(11),
                field.with_value(|value| matches!(value, ValueView::Integer(_))),
            ),
            INVOCATION_FIELD_LENGTH => (
                Some(12),
                field.with_value(|value| matches!(value, ValueView::Integer(_))),
            ),
            SALT_LENGTH => (
                Some(13),
                field.with_value(|value| matches!(value, ValueView::Integer(_))),
            ),
            MASK_GENERATOR => (
                Some(14),
                field.with_value(|value| matches!(value, ValueView::Enumeration(_))),
            ),
            MASK_GENERATOR_HASHING_ALGORITHM => (
                Some(15),
                field.with_value(|value| matches!(value, ValueView::Enumeration(_))),
            ),
            P_SOURCE => (
                Some(16),
                field.with_value(|value| matches!(value, ValueView::ByteString(_))),
            ),
            TRAILER_FIELD => (
                Some(17),
                field.with_value(|value| matches!(value, ValueView::Integer(_))),
            ),
            _ => (None, true),
        };
        if !valid_type {
            return Err(ProtocolError::categorized(
                ProtocolErrorKind::InvalidValue,
                ProtocolCauseCategory::InvalidValue,
            ));
        }
        if let Some(index) = index {
            if seen[index] {
                return Err(ProtocolError::categorized(
                    ProtocolErrorKind::InvalidValue,
                    ProtocolCauseCategory::InvalidValue,
                ));
            }
            seen[index] = true;
        }
    }

    Ok(())
}
