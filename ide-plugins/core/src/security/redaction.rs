//! Mandatory redaction of sensitive cryptographic material and credentials.

use kmipkit_ttlv::KmipTag;

/// Placeholder displayed when sensitive TTLV material is redacted.
pub const REDACTED_DISPLAY: &str = "[REDACTED - Click to reveal]";

/// Returns `true` if a KMIP tag carries sensitive cryptographic or credential material
/// that must be redacted by default in IDE UIs and never logged in cleartext.
#[must_use]
pub fn is_sensitive_tag(tag: Option<KmipTag>, raw_tag: u32) -> bool {
    if let Some(known) = tag {
        return matches!(
            known,
            KmipTag::KeyMaterial
                | KmipTag::KeyValue
                | KmipTag::SecretData
                | KmipTag::PrivateKey
                | KmipTag::Password
                | KmipTag::CredentialValue
                | KmipTag::D
                | KmipTag::P
                | KmipTag::Q
                | KmipTag::CRTCoefficient
                | KmipTag::X
                | KmipTag::NonceValue
                | KmipTag::Salt
                | KmipTag::DerivationData
                | KmipTag::OpaqueDataValue
                | KmipTag::SymmetricKey
                | KmipTag::Seed
        );
    }

    // Fallback for raw tag values if not matched via enum variant
    matches!(
        raw_tag,
        0x0042_0043 // KeyMaterial
            | 0x0042_0045 // KeyValue
            | 0x0042_006C // PrivateKey
            | 0x0042_0081 // SecretData
            | 0x0042_0025 // CredentialValue
    )
}

/// Masks a hex string corresponding to a byte range when redaction is active.
#[must_use]
pub fn redact_hex_slice(hex_str: &str, byte_offset: usize, byte_len: usize) -> String {
    let char_start = byte_offset.saturating_mul(2);
    let char_end = byte_offset.saturating_add(byte_len).saturating_mul(2);
    if char_end > hex_str.len() || char_start >= char_end {
        return hex_str.to_string();
    }
    let mut out = String::with_capacity(hex_str.len());
    out.push_str(&hex_str[..char_start]);
    for _ in char_start..char_end {
        out.push('*');
    }
    out.push_str(&hex_str[char_end..]);
    out
}
