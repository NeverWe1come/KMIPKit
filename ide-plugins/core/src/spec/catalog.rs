//! OASIS KMIP (1.0 - 3.0) specification catalog backed by `kmipkit-ttlv` and `kmipkit-protocol`.

use kmipkit_protocol::named_values_generated::values::enumeration;
use kmipkit_ttlv::KmipTag;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecEntry {
    pub tag_hex: String,
    pub name: String,
    pub oasis_name: String,
    pub expected_type: String,
    pub versions: String,
    pub summary: String,
    pub deprecation_warning: Option<String>,
}

type CuratedSpecRow = (&'static str, &'static str, &'static str, &'static str, &'static str, Option<&'static str>);

/// Curated catalog of high-frequency KMIP tags, operations, and attributes
/// with version ranges and expected TTLV Item Types.
const CURATED_SPECS: &[CuratedSpecRow] = &[
    ("0x420078", "RequestMessage", "Request Message", "Structure", "1.0 - 3.0", None),
    ("0x420077", "RequestHeader", "Request Header", "Structure", "1.0 - 3.0", None),
    ("0x42007B", "ResponseMessage", "Response Message", "Structure", "1.0 - 3.0", None),
    ("0x42007A", "ResponseHeader", "Response Header", "Structure", "1.0 - 3.0", None),
    ("0x42000F", "BatchItem", "Batch Item", "Structure", "1.0 - 3.0", None),
    ("0x42000D", "BatchCount", "Batch Count", "Integer", "1.0 - 3.0", None),
    ("0x420069", "ProtocolVersion", "Protocol Version", "Structure", "1.0 - 3.0", None),
    ("0x42006A", "ProtocolVersionMajor", "Protocol Version Major", "Integer", "1.0 - 3.0", None),
    ("0x42006B", "ProtocolVersionMinor", "Protocol Version Minor", "Integer", "1.0 - 3.0", None),
    ("0x42005C", "Operation", "Operation", "Enumeration", "1.0 - 3.0", None),
    ("0x42007F", "ResultStatus", "Result Status", "Enumeration", "1.0 - 3.0", None),
    ("0x42007E", "ResultReason", "Result Reason", "Enumeration", "1.0 - 3.0", None),
    ("0x42007D", "ResultMessage", "Result Message", "TextString", "1.0 - 3.0", None),
    ("0x420057", "ObjectType", "Object Type", "Enumeration", "1.0 - 3.0", None),
    ("0x420094", "UniqueIdentifier", "Unique Identifier", "TextString / Identifier (3.0)", "1.0 - 3.0", None),
    ("0x420028", "CryptographicAlgorithm", "Cryptographic Algorithm", "Enumeration", "1.0 - 3.0", None),
    ("0x42002A", "CryptographicLength", "Cryptographic Length", "Integer", "1.0 - 3.0", None),
    ("0x42002C", "CryptographicUsageMask", "Cryptographic Usage Mask", "Integer (BitMask)", "1.0 - 3.0", None),
    ("0x420042", "KeyBlock", "Key Block", "Structure", "1.0 - 3.0", None),
    ("0x420043", "KeyMaterial", "Key Material", "Structure / ByteString", "1.0 - 3.0", None),
    ("0x420045", "KeyValue", "Key Value", "Structure / ByteString", "1.0 - 3.0", None),
    ("0x420008", "Attribute", "Attribute", "Structure", "1.0 - 3.0", None),
    ("0x42000A", "AttributeName", "Attribute Name", "TextString", "1.0 - 2.1", Some("Deprecated in KMIP 2.0+ in favor of Attributes Structure.")),
    ("0x42000B", "AttributeValue", "Attribute Value", "Any", "1.0 - 2.1", Some("Deprecated in KMIP 2.0+ in favor of typed attribute tags.")),
    ("0x4200C0", "Attributes", "Attributes", "Structure", "2.0 - 3.0", None),
    ("0x42008E", "TemplateAttribute", "Template Attribute", "Structure", "1.0 - 1.4", Some("Removed in KMIP 2.0+; use Attributes (0x4200C0).")),
    ("0x42014F", "ProtectionLevel", "Protection Level", "Enumeration", "2.0 - 3.0", None),
    ("0x420195", "KemAlgorithm", "KEM Algorithm", "Enumeration", "3.0+", None),
];

/// Resolves a hex tag (e.g. `0x420028` or `420028`) or a KMIP name to specification entries.
#[must_use]
pub fn lookup(query: &str) -> Vec<SpecEntry> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return CURATED_SPECS
            .iter()
            .take(15)
            .map(|(tag, name, oasis, ty, ver, dep)| SpecEntry {
                tag_hex: (*tag).to_string(),
                name: (*name).to_string(),
                oasis_name: (*oasis).to_string(),
                expected_type: (*ty).to_string(),
                versions: (*ver).to_string(),
                summary: format!("OASIS KMIP {oasis} ({tag}) — expected TTLV type: {ty}."),
                deprecation_warning: dep.map(str::to_string),
            })
            .collect();
    }

    let normalized_hex = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed)
        .replace('_', "");

    if let Ok(raw_u32) = u32::from_str_radix(&normalized_hex, 16)
        && let Some(tag) = KmipTag::from_u32(raw_u32)
    {
        let tag_hex = format!("0x{:06X}", tag.as_u32());
        if let Some(curated) = CURATED_SPECS
            .iter()
            .find(|(h, ..)| h.eq_ignore_ascii_case(&tag_hex))
        {
            return vec![SpecEntry {
                tag_hex: curated.0.to_string(),
                name: curated.1.to_string(),
                oasis_name: curated.2.to_string(),
                expected_type: curated.3.to_string(),
                versions: curated.4.to_string(),
                summary: format!(
                    "OASIS KMIP {} ({}) — expected TTLV type: {}.",
                    curated.2, curated.0, curated.3
                ),
                deprecation_warning: curated.5.map(str::to_string),
            }];
        }

        return vec![SpecEntry {
            tag_hex: tag_hex.clone(),
            name: format!("{tag:?}"),
            oasis_name: tag.name().to_string(),
            expected_type: "See OASIS Spec".to_string(),
            versions: "1.0 - 3.0".to_string(),
            summary: format!("OASIS KMIP standard tag {} ({tag_hex}).", tag.name()),
            deprecation_warning: None,
        }];
    }

    let q_lower = trimmed.to_ascii_lowercase();
    let mut results: Vec<SpecEntry> = CURATED_SPECS
        .iter()
        .filter(|(hex_t, name, oasis, ..)| {
            hex_t.to_ascii_lowercase().contains(&q_lower)
                || name.to_ascii_lowercase().contains(&q_lower)
                || oasis.to_ascii_lowercase().contains(&q_lower)
        })
        .map(|(tag, name, oasis, ty, ver, dep)| SpecEntry {
            tag_hex: (*tag).to_string(),
            name: (*name).to_string(),
            oasis_name: (*oasis).to_string(),
            expected_type: (*ty).to_string(),
            versions: (*ver).to_string(),
            summary: format!("OASIS KMIP {oasis} ({tag}) — expected TTLV type: {ty}."),
            deprecation_warning: dep.map(str::to_string),
        })
        .collect();

    if results.is_empty() && matches_known_algo_or_op(&q_lower) {
        results.push(SpecEntry {
            tag_hex: "Enumeration".to_string(),
            name: trimmed.to_string(),
            oasis_name: trimmed.to_string(),
            expected_type: "Enumeration".to_string(),
            versions: "1.0 - 3.0".to_string(),
            summary: format!("OASIS KMIP standard enumeration value `{trimmed}`."),
            deprecation_warning: weak_algorithm_warning(&q_lower),
        });
    }

    results
}

fn matches_known_algo_or_op(q_lower: &str) -> bool {
    matches!(
        q_lower,
        "aes"
            | "rsa"
            | "ecdsa"
            | "ecdh"
            | "3des"
            | "des"
            | "rc4"
            | "md5"
            | "sha-1"
            | "sha1"
            | "sha-256"
            | "ml-kem"
            | "ml-dsa"
            | "slh-dsa"
            | "create"
            | "createkeypair"
            | "locate"
            | "get"
            | "rekey"
            | "destroy"
            | "discoverversions"
            | "activate"
            | "revoke"
    )
}

#[must_use]
pub fn weak_algorithm_warning(q_lower: &str) -> Option<String> {
    match q_lower {
        "3des" | "des" | "rc4" => Some("Weak/deprecated symmetric cipher. Prefer AES-256 or ChaCha20Poly1305.".to_string()),
        "md5" | "sha-1" | "sha1" => Some("Cryptographically broken hash algorithm. Prefer SHA-256 / SHA-384 / SHA-512.".to_string()),
        _ => None,
    }
}

/// Resolves an enumeration raw value for common KMIP tags using `kmipkit-protocol`'s generated tables.
#[must_use]
pub fn resolve_enumeration_name(tag: Option<KmipTag>, raw_value: u32) -> Option<&'static str> {
    match tag? {
        KmipTag::Operation => enumeration::operation::name(raw_value),
        KmipTag::ObjectType => enumeration::object_type::name(raw_value),
        KmipTag::CryptographicAlgorithm => enumeration::cryptographic_algorithm::name(raw_value),
        KmipTag::BlockCipherMode => enumeration::block_cipher_mode::name(raw_value),
        KmipTag::PaddingMethod => enumeration::padding_method::name(raw_value),
        KmipTag::HashingAlgorithm => enumeration::hashing_algorithm::name(raw_value),
        KmipTag::ResultStatus => enumeration::result_status::name(raw_value),
        KmipTag::ResultReason => enumeration::result_reason::name(raw_value),
        KmipTag::State => enumeration::state::name(raw_value),
        KmipTag::RevocationReasonCode => enumeration::revocation_reason_code::name(raw_value),
        KmipTag::CredentialType => enumeration::credential_type::name(raw_value),
        KmipTag::KeyFormatType => enumeration::key_format_type::name(raw_value),
        KmipTag::KeyCompressionType => enumeration::key_compression_type::name(raw_value),
        KmipTag::WrappingMethod => enumeration::wrapping_method::name(raw_value),
        KmipTag::RecommendedCurve => enumeration::recommended_curve::name(raw_value),
        KmipTag::DigitalSignatureAlgorithm => {
            enumeration::digital_signature_algorithm::name(raw_value)
        }
        KmipTag::ProtectionLevel => enumeration::protection_level::name(raw_value),
        _ => None,
    }
}
