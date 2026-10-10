//! Shared, locally knowable Cryptographic Parameters validation.

use kmipkit_ttlv::{Structure, ValueView};

use crate::{ProtocolCauseCategory, ProtocolError, ProtocolErrorKind};

const BLOCK_CIPHER_MODE: u32 = 0x0042_0011;
const IV_LENGTH: u32 = 0x0042_00CD;
const TAG_LENGTH: u32 = 0x0042_00CE;

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

    let view = supplied.view();
    let mut requires_iv_length = false;
    let mut requires_tag_length = false;
    let mut has_iv_length = false;
    let mut has_tag_length = false;

    for item in view.children() {
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

    Ok(parameters)
}
