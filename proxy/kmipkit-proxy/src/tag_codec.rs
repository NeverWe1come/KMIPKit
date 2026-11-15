//! Universal Cloud Tag-Stuffing Codec (KMIP Attributes <-> Cloud Tags/Labels).
#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use crate::providers::CloudTagMap;

pub const PREFIX_NAME: &str = "kmip_name_";
pub const TAG_OBJ_TYPE: &str = "kmip_obj_type";
pub const TAG_STATE: &str = "kmip_state";
pub const TAG_USAGE_MASK: &str = "kmip_usage_mask";
pub const TAG_ACT_DATE: &str = "kmip_act_date";
pub const TAG_STOP_DATE: &str = "kmip_stop_date";
pub const TAG_DEACT_DATE: &str = "kmip_deact_date";
pub const TAG_REV_REASON: &str = "kmip_rev_reason";
pub const TAG_MODE: &str = "kmip_mode";
pub const TAG_LINK_PREFIX: &str = "kmip_link_";

const BASE32HEX_ALPHABET: &[u8; 32] = b"0123456789abcdefghijklmnopqrstuv";

/// Encodes binary data into lowercase RFC 4648 Base32Hex without padding.
#[must_use]
pub fn base32hex_encode(data: &[u8]) -> String {
    let mut out = String::with_capacity((data.len() * 8).div_ceil(5));
    let mut buffer: u64 = 0;
    let mut bits_left = 0;

    for &byte in data {
        buffer = (buffer << 8) | u64::from(byte);
        bits_left += 8;
        while bits_left >= 5 {
            bits_left -= 5;
            let index = ((buffer >> bits_left) & 0x1F) as usize;
            out.push(BASE32HEX_ALPHABET[index] as char);
        }
    }

    if bits_left > 0 {
        let index = ((buffer << (5 - bits_left)) & 0x1F) as usize;
        out.push(BASE32HEX_ALPHABET[index] as char);
    }

    out
}

/// Decodes lowercase RFC 4648 Base32Hex without padding into bytes.
/// Returns None if invalid characters are encountered.
#[must_use]
pub fn base32hex_decode(s: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity((s.len() * 5) / 8);
    let mut buffer: u64 = 0;
    let mut bits_left = 0;

    for b in s.bytes() {
        let val = match b {
            b'0'..=b'9' => b - b'0',
            b'a'..=b'v' => b - b'a' + 10,
            b'A'..=b'V' => b - b'A' + 10,
            _ => return None,
        };
        buffer = (buffer << 5) | u64::from(val);
        bits_left += 5;
        if bits_left >= 8 {
            bits_left -= 8;
            out.push(((buffer >> bits_left) & 0xFF) as u8);
        }
    }

    Some(out)
}

/// High-level KMIP object attributes for tag-stuffing.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct KmipAttributes {
    pub object_type: Option<String>,
    pub state: Option<String>,
    pub cryptographic_usage_mask: Option<u32>,
    pub name: Option<String>,
    pub activation_date: Option<i64>,
    pub protect_stop_date: Option<i64>,
    pub deactivation_date: Option<i64>,
    pub revocation_reason: Option<String>,
    pub custody_mode: Option<String>,
    pub links: BTreeMap<String, String>,
}

pub struct TagCodec;

impl TagCodec {
    /// Encodes KMIP attributes into normalized cloud tags.
    #[must_use]
    pub fn encode(attrs: &KmipAttributes) -> CloudTagMap {
        let mut map = BTreeMap::new();

        if let Some(ref obj_type) = attrs.object_type {
            map.insert(TAG_OBJ_TYPE.to_string(), obj_type.to_ascii_lowercase());
        }
        if let Some(ref state) = attrs.state {
            map.insert(TAG_STATE.to_string(), state.to_ascii_lowercase());
        }
        if let Some(mask) = attrs.cryptographic_usage_mask {
            map.insert(TAG_USAGE_MASK.to_string(), format!("{mask:08x}"));
        }
        if let Some(ref name) = attrs.name {
            let encoded = base32hex_encode(name.as_bytes());
            // Split into <= 60 character chunks for strict cloud limits (e.g. GCP 63 char limit)
            let chunks: Vec<&str> = encoded
                .as_bytes()
                .chunks(60)
                .map(|c| std::str::from_utf8(c).unwrap_or(""))
                .collect();
            for (idx, chunk) in chunks.into_iter().enumerate() {
                map.insert(format!("{PREFIX_NAME}{idx}"), chunk.to_string());
            }
        }
        if let Some(date) = attrs.activation_date {
            map.insert(TAG_ACT_DATE.to_string(), date.to_string());
        }
        if let Some(date) = attrs.protect_stop_date {
            map.insert(TAG_STOP_DATE.to_string(), date.to_string());
        }
        if let Some(date) = attrs.deactivation_date {
            map.insert(TAG_DEACT_DATE.to_string(), date.to_string());
        }
        if let Some(ref reason) = attrs.revocation_reason {
            map.insert(TAG_REV_REASON.to_string(), reason.to_ascii_lowercase());
        }
        if let Some(ref mode) = attrs.custody_mode {
            map.insert(TAG_MODE.to_string(), mode.to_ascii_lowercase());
        }
        for (link_type, target_id) in &attrs.links {
            let key = format!("{TAG_LINK_PREFIX}{}", link_type.to_ascii_lowercase());
            map.insert(key, target_id.clone());
        }

        CloudTagMap(map)
    }

    /// Decodes cloud tags back into high-level KMIP attributes.
    #[must_use]
    pub fn decode(tags: &CloudTagMap) -> KmipAttributes {
        let mut attrs = KmipAttributes::default();

        if let Some(obj_type) = tags.0.get(TAG_OBJ_TYPE) {
            attrs.object_type = Some(obj_type.clone());
        }
        if let Some(state) = tags.0.get(TAG_STATE) {
            attrs.state = Some(state.clone());
        }
        if let Some(mask_str) = tags.0.get(TAG_USAGE_MASK) {
            if let Ok(mask) = u32::from_str_radix(mask_str, 16) {
                attrs.cryptographic_usage_mask = Some(mask);
            }
        }
        // Reassemble name chunks
        let mut name_encoded = String::new();
        let mut idx = 0;
        while let Some(chunk) = tags.0.get(&format!("{PREFIX_NAME}{idx}")) {
            name_encoded.push_str(chunk);
            idx += 1;
        }
        if !name_encoded.is_empty() {
            if let Some(bytes) = base32hex_decode(&name_encoded) {
                if let Ok(name) = String::from_utf8(bytes) {
                    attrs.name = Some(name);
                }
            }
        }
        if let Some(date_str) = tags.0.get(TAG_ACT_DATE) {
            if let Ok(date) = date_str.parse::<i64>() {
                attrs.activation_date = Some(date);
            }
        }
        if let Some(date_str) = tags.0.get(TAG_STOP_DATE) {
            if let Ok(date) = date_str.parse::<i64>() {
                attrs.protect_stop_date = Some(date);
            }
        }
        if let Some(date_str) = tags.0.get(TAG_DEACT_DATE) {
            if let Ok(date) = date_str.parse::<i64>() {
                attrs.deactivation_date = Some(date);
            }
        }
        if let Some(reason) = tags.0.get(TAG_REV_REASON) {
            attrs.revocation_reason = Some(reason.clone());
        }
        if let Some(mode) = tags.0.get(TAG_MODE) {
            attrs.custody_mode = Some(mode.clone());
        }
        for (key, val) in &tags.0 {
            if let Some(link_type) = key.strip_prefix(TAG_LINK_PREFIX) {
                attrs.links.insert(link_type.to_string(), val.clone());
            }
        }

        attrs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base32hex_roundtrip() {
        let input = "Database-AES-Encryption-Key_2026.01#Special!";
        let encoded = base32hex_encode(input.as_bytes());
        assert!(!encoded.contains('='));
        assert!(
            encoded
                .chars()
                .all(|c| c.is_ascii_digit() || (c >= 'a' && c <= 'v'))
        );
        let decoded = base32hex_decode(&encoded).expect("valid base32hex");
        assert_eq!(String::from_utf8(decoded).unwrap(), input);
    }

    #[test]
    fn test_tag_codec_roundtrip() {
        let mut links = BTreeMap::new();
        links.insert("public_key".to_string(), "uuid-pub-key-1234".to_string());

        let attrs = KmipAttributes {
            object_type: Some("symmetric_key".to_string()),
            state: Some("active".to_string()),
            cryptographic_usage_mask: Some(0x0000_000C),
            name: Some("Production MySQL DB Master Key (Sensitive)".to_string()),
            activation_date: Some(1767225600),
            protect_stop_date: Some(1798761600),
            deactivation_date: None,
            revocation_reason: None,
            custody_mode: Some("auto_envelope".to_string()),
            links,
        };

        let tags = TagCodec::encode(&attrs);
        assert_eq!(tags.0.get(TAG_OBJ_TYPE).unwrap(), "symmetric_key");
        assert_eq!(tags.0.get(TAG_STATE).unwrap(), "active");
        assert_eq!(tags.0.get(TAG_USAGE_MASK).unwrap(), "0000000c");
        assert_eq!(tags.0.get(TAG_ACT_DATE).unwrap(), "1767225600");
        assert_eq!(tags.0.get(TAG_STOP_DATE).unwrap(), "1798761600");
        assert_eq!(tags.0.get(TAG_MODE).unwrap(), "auto_envelope");
        assert_eq!(
            tags.0.get("kmip_link_public_key").unwrap(),
            "uuid-pub-key-1234"
        );
        assert!(tags.0.contains_key("kmip_name_0"));

        let decoded = TagCodec::decode(&tags);
        assert_eq!(decoded, attrs);
    }
}
