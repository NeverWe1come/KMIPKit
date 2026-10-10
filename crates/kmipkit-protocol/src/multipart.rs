//! Shared validation for multipart cryptographic operation request shapes.

use crate::{ProtocolCauseCategory, ProtocolError, ProtocolErrorKind};

/// Validates Data and multipart indicators for Encrypt and Decrypt requests.
///
/// KMIP Specification v2.1 §6.1 defines the single-part and multipart
/// sequences; Tables 196 and 214 define operation Data as required for a
/// single-part request. The unresolved Data-omission conflict for a framed
/// single request is bounded by KMIPKIT-DISC-045.
pub(crate) fn validate_data_multipart_shape(
    has_data: bool,
    has_correlation_value: bool,
    init_indicator: Option<bool>,
    final_indicator: Option<bool>,
) -> Result<(), ProtocolError> {
    let invalid = if init_indicator == Some(true) {
        has_correlation_value || (final_indicator == Some(true) && !has_data)
    } else if final_indicator == Some(true) {
        !has_correlation_value
    } else if init_indicator == Some(false) || final_indicator == Some(false) {
        // Explicit false marks a multipart middle part; only absent indicators
        // without correlation can use the unframed single-part shape.
        !has_correlation_value || !has_data
    } else {
        !has_data
    };

    if invalid {
        Err(ProtocolError::categorized(
            ProtocolErrorKind::InvalidValue,
            ProtocolCauseCategory::InvalidValue,
        ))
    } else {
        Ok(())
    }
}
