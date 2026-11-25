//! TTLV inspector and encoder built on `kmipkit-ttlv`.

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use kmipkit_ttlv::codec::{CodecLimits, decode_with_limits};
use kmipkit_ttlv::{Item, ItemType, KmipTag, ValueView};
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use crate::security::redaction::{REDACTED_DISPLAY, is_sensitive_tag, redact_hex_slice};
use crate::spec::catalog::resolve_enumeration_name;

#[derive(Debug, Clone, Copy, Deserialize, Default, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DecodeFormat {
    #[default]
    Auto,
    Hex,
    Base64,
}

#[derive(Debug, Clone, Default)]
pub struct DecodeOptions {
    pub format: DecodeFormat,
    pub reveal_all: bool,
    pub reveal_offsets: Vec<usize>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComplianceAlert {
    pub severity: String,
    pub code: String,
    pub message: String,
    pub offset: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TtlvNode {
    pub tag_hex: String,
    pub tag_name: String,
    pub item_type: String,
    pub offset: usize,
    pub header_length: usize,
    pub value_offset: usize,
    pub value_length: usize,
    pub padded_length: usize,
    pub display_value: String,
    pub enum_name: Option<String>,
    pub redacted: bool,
    pub sensitive: bool,
    pub children: Vec<TtlvNode>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecodeResult {
    pub raw_hex: String,
    pub byte_length: usize,
    pub root: TtlvNode,
    pub alerts: Vec<ComplianceAlert>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncodeNodeInput {
    pub tag_hex: String,
    pub item_type: String,
    pub value: Option<serde_json::Value>,
    #[serde(default)]
    pub children: Vec<EncodeNodeInput>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EncodeResult {
    pub raw_hex: String,
    pub base64: String,
    pub byte_length: usize,
}

/// Parses hex or base64 input into raw TTLV bytes, validates with `kmipkit-ttlv`,
/// and produces an offset-mapped, redacted-by-default inspection tree.
///
/// # Errors
///
/// Returns an error message if the input cannot be parsed as hex/base64 or fails
/// `kmipkit-ttlv` framing/model validation.
pub fn decode_ttlv(input: &str, options: &DecodeOptions) -> Result<DecodeResult, String> {
    let bytes = parse_input_bytes(input, options.format)?;
    if bytes.is_empty() {
        return Err("Empty TTLV input".to_string());
    }

    let limits = CodecLimits::defaults();
    let root_item = decode_with_limits(&bytes, &limits)
        .map_err(|err| format!("TTLV validation failed: {err}"))?;

    let mut raw_hex_chars = hex::encode(&bytes);
    let mut alerts = Vec::new();

    let root = build_node_from_item(
        &root_item,
        &bytes,
        0,
        options,
        &mut raw_hex_chars,
        &mut alerts,
    )?;

    Ok(DecodeResult {
        raw_hex: raw_hex_chars,
        byte_length: bytes.len(),
        root,
        alerts,
    })
}

fn parse_input_bytes(input: &str, format: DecodeFormat) -> Result<Zeroizing<Vec<u8>>, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("Input is empty".to_string());
    }

    match format {
        DecodeFormat::Hex => decode_hex_flexible(trimmed),
        DecodeFormat::Base64 => {
            let cleaned: String = trimmed.chars().filter(|c| !c.is_whitespace()).collect();
            let decoded = BASE64_STANDARD
                .decode(&cleaned)
                .map_err(|e| format!("Invalid Base64 input: {e}"))?;
            Ok(Zeroizing::new(decoded))
        }
        DecodeFormat::Auto => {
            if let Ok(hex_bytes) = decode_hex_flexible(trimmed) {
                return Ok(hex_bytes);
            }
            let cleaned: String = trimmed.chars().filter(|c| !c.is_whitespace()).collect();
            let decoded = BASE64_STANDARD
                .decode(&cleaned)
                .map_err(|_| "Input is neither valid hexadecimal nor Base64 TTLV".to_string())?;
            Ok(Zeroizing::new(decoded))
        }
    }
}

fn decode_hex_flexible(input: &str) -> Result<Zeroizing<Vec<u8>>, String> {
    let mut cleaned = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '0' && matches!(chars.peek(), Some('x' | 'X')) {
            let _ = chars.next();
            continue;
        }
        if c.is_whitespace() || c == ':' || c == '-' || c == '_' {
            continue;
        }
        if !c.is_ascii_hexdigit() {
            return Err(format!("Unexpected non-hex character '{c}'"));
        }
        cleaned.push(c);
    }

    if cleaned.is_empty() || !cleaned.len().is_multiple_of(2) {
        return Err("Hex input must contain an even number of hex digits".to_string());
    }

    let decoded = hex::decode(&cleaned).map_err(|e| format!("Hex decode failed: {e}"))?;
    Ok(Zeroizing::new(decoded))
}

fn padded_len(len: usize) -> usize {
    let rem = len % 8;
    if rem == 0 { len } else { len + (8 - rem) }
}

fn build_node_from_item(
    item: &Item,
    wire: &[u8],
    offset: usize,
    options: &DecodeOptions,
    raw_hex_chars: &mut String,
    alerts: &mut Vec<ComplianceAlert>,
) -> Result<TtlvNode, String> {
    if offset + 8 > wire.len() {
        return Err(format!("Truncated TTLV header at offset {offset}"));
    }

    let raw_tag_u32 = u32::from_be_bytes([0, wire[offset], wire[offset + 1], wire[offset + 2]]);
    let value_length =
        u32::from_be_bytes([wire[offset + 4], wire[offset + 5], wire[offset + 6], wire[offset + 7]])
            as usize;
    let value_offset = offset + 8;
    let padded_length = if item.item_type() == ItemType::Structure {
        value_length
    } else {
        padded_len(value_length)
    };

    let known_tag = KmipTag::from_u32(raw_tag_u32);
    let tag_hex = format!("0x{raw_tag_u32:06X}");
    let tag_name = known_tag
        .map(|t| t.name().to_string())
        .unwrap_or_else(|| format!("Tag({tag_hex})"));
    let item_type = format_item_type(item.item_type());

    let sensitive = is_sensitive_tag(known_tag, raw_tag_u32);
    let should_reveal = options.reveal_all || options.reveal_offsets.contains(&offset);
    let redacted = sensitive && !should_reveal;

    if redacted && value_offset + padded_length <= wire.len() {
        *raw_hex_chars = redact_hex_slice(raw_hex_chars, value_offset, value_length);
    }

    let mut children = Vec::new();
    let mut enum_name = None;

    let display_value = item.with_value(|view| match view {
        ValueView::Structure(structure) => {
            let mut cursor = value_offset;
            for child in structure.children() {
                if let Ok(child_node) = build_node_from_item(
                    child,
                    wire,
                    cursor,
                    options,
                    raw_hex_chars,
                    alerts,
                ) {
                    cursor = child_node.value_offset + child_node.padded_length;
                    children.push(child_node);
                }
            }
            format!("Structure ({} items)", children.len())
        }
        ValueView::Integer(v) => {
            check_integer_compliance(known_tag, *v, offset, alerts);
            if redacted {
                REDACTED_DISPLAY.to_string()
            } else {
                format!("{v} (0x{:08X})", *v)
            }
        }
        ValueView::LongInteger(v) => {
            if redacted {
                REDACTED_DISPLAY.to_string()
            } else {
                v.to_string()
            }
        }
        ValueView::BigInteger(bytes) => {
            if redacted {
                REDACTED_DISPLAY.to_string()
            } else {
                format!("0x{}", hex::encode(bytes))
            }
        }
        ValueView::Enumeration(v) => {
            let resolved = resolve_enumeration_name(known_tag, *v).map(str::to_string);
            if let Some(ref name) = resolved {
                check_enum_compliance(known_tag, name, offset, alerts);
            }
            enum_name = resolved.clone();
            if redacted {
                REDACTED_DISPLAY.to_string()
            } else if let Some(name) = resolved {
                format!("{name} (0x{v:08X})")
            } else {
                format!("0x{v:08X}")
            }
        }
        ValueView::Boolean(b) => {
            if redacted {
                REDACTED_DISPLAY.to_string()
            } else {
                b.to_string()
            }
        }
        ValueView::TextString(s)
        | ValueView::Identifier(s)
        | ValueView::Reference(s)
        | ValueView::NameReference(s) => {
            if redacted {
                REDACTED_DISPLAY.to_string()
            } else {
                (*s).to_string()
            }
        }
        ValueView::ByteString(bytes) => {
            if redacted {
                format!("{REDACTED_DISPLAY} ({} bytes)", bytes.len())
            } else {
                format!("0x{} ({} bytes)", hex::encode(bytes), bytes.len())
            }
        }
        ValueView::DateTime(ts) | ValueView::DateTimeExtended(ts) => {
            if redacted {
                REDACTED_DISPLAY.to_string()
            } else {
                format!("{ts} (Unix epoch s)")
            }
        }
        ValueView::Interval(secs) => {
            if redacted {
                REDACTED_DISPLAY.to_string()
            } else {
                format!("{secs}s")
            }
        }
        _ => {
            if redacted {
                REDACTED_DISPLAY.to_string()
            } else {
                "<unsupported value>".to_string()
            }
        }
    });

    Ok(TtlvNode {
        tag_hex,
        tag_name,
        item_type,
        offset,
        header_length: 8,
        value_offset,
        value_length,
        padded_length,
        display_value,
        enum_name,
        redacted,
        sensitive,
        children,
    })
}

fn check_enum_compliance(
    tag: Option<KmipTag>,
    name: &str,
    offset: usize,
    alerts: &mut Vec<ComplianceAlert>,
) {
    if matches!(tag, Some(KmipTag::CryptographicAlgorithm)) {
        let lower = name.to_ascii_lowercase();
        if matches!(lower.as_str(), "des" | "3des" | "tripledes" | "rc4" | "skipjack") {
            alerts.push(ComplianceAlert {
                severity: "warning".to_string(),
                code: "WEAK_CIPHER".to_string(),
                message: format!("Deprecated/weak cryptographic algorithm '{name}' detected; prefer AES-256."),
                offset,
            });
        }
    } else if matches!(tag, Some(KmipTag::HashingAlgorithm)) {
        let lower = name.to_ascii_lowercase();
        if matches!(lower.as_str(), "md5" | "sha-1" | "sha1" | "md2" | "md4") {
            alerts.push(ComplianceAlert {
                severity: "warning".to_string(),
                code: "WEAK_HASH".to_string(),
                message: format!("Weak hash algorithm '{name}' detected; prefer SHA-256 or stronger."),
                offset,
            });
        }
    }
}

fn check_integer_compliance(
    tag: Option<KmipTag>,
    value: i32,
    offset: usize,
    alerts: &mut Vec<ComplianceAlert>,
) {
    if matches!(tag, Some(KmipTag::CryptographicLength)) && value > 0 && value < 112 {
        alerts.push(ComplianceAlert {
            severity: "warning".to_string(),
            code: "SHORT_KEY_LENGTH".to_string(),
            message: format!("Cryptographic length {value} bits is below modern 112/128-bit security baselines."),
            offset,
        });
    }
}

fn format_item_type(ty: ItemType) -> String {
    match ty {
        ItemType::Structure => "Structure",
        ItemType::Integer => "Integer",
        ItemType::LongInteger => "LongInteger",
        ItemType::BigInteger => "BigInteger",
        ItemType::Enumeration => "Enumeration",
        ItemType::Boolean => "Boolean",
        ItemType::TextString => "TextString",
        ItemType::ByteString => "ByteString",
        ItemType::DateTime => "DateTime",
        ItemType::Interval => "Interval",
        ItemType::DateTimeExtended => "DateTimeExtended",
        ItemType::Identifier => "Identifier",
        ItemType::Reference => "Reference",
        ItemType::NameReference => "NameReference",
        _ => "Unknown",
    }
    .to_string()
}

/// Encodes a JSON TTLV node tree into canonical wire TTLV bytes (8-byte padded).
///
/// # Errors
///
/// Returns an error string if a tag, type, or value is invalid.
pub fn encode_ttlv(node: &EncodeNodeInput) -> Result<EncodeResult, String> {
    let bytes = encode_node_bytes(node)?;
    let _ = decode_with_limits(&bytes, &CodecLimits::defaults())
        .map_err(|e| format!("Encoded TTLV failed validation: {e}"))?;

    Ok(EncodeResult {
        raw_hex: hex::encode(&bytes),
        base64: BASE64_STANDARD.encode(&bytes),
        byte_length: bytes.len(),
    })
}

fn encode_node_bytes(node: &EncodeNodeInput) -> Result<Vec<u8>, String> {
    let tag_clean = node
        .tag_hex
        .trim()
        .strip_prefix("0x")
        .or_else(|| node.tag_hex.trim().strip_prefix("0X"))
        .unwrap_or(node.tag_hex.trim());
    let tag_u32 = u32::from_str_radix(tag_clean, 16)
        .map_err(|e| format!("Invalid tag '{}': {e}", node.tag_hex))?;
    if tag_u32 > 0x00FF_FFFF {
        return Err(format!("Tag '{}' exceeds 24 bits", node.tag_hex));
    }

    let (type_code, payload, pad_payload): (u8, Vec<u8>, bool) =
        match node.item_type.as_str() {
            "Structure" => {
                let mut buf = Vec::new();
                for child in &node.children {
                    buf.extend_from_slice(&encode_node_bytes(child)?);
                }
                (0x01, buf, false)
            }
            "Integer" => {
                let v = node
                    .value
                    .as_ref()
                    .and_then(serde_json::Value::as_i64)
                    .ok_or_else(|| "Integer node requires numeric value".to_string())?;
                let i = i32::try_from(v).map_err(|_| "Integer value out of i32 range")?;
                let mut buf = Vec::with_capacity(8);
                buf.extend_from_slice(&i.to_be_bytes());
                buf.extend_from_slice(&[0u8; 4]);
                (0x02, buf, false)
            }
            "Enumeration" => {
                let v = node
                    .value
                    .as_ref()
                    .and_then(serde_json::Value::as_u64)
                    .ok_or_else(|| "Enumeration node requires u32 numeric value".to_string())?;
                let u = u32::try_from(v).map_err(|_| "Enumeration value out of u32 range")?;
                let mut buf = Vec::with_capacity(8);
                buf.extend_from_slice(&u.to_be_bytes());
                buf.extend_from_slice(&[0u8; 4]);
                (0x05, buf, false)
            }
            "Boolean" => {
                let b = node
                    .value
                    .as_ref()
                    .and_then(serde_json::Value::as_bool)
                    .ok_or_else(|| "Boolean node requires boolean value".to_string())?;
                let val: u64 = u64::from(b);
                (0x06, val.to_be_bytes().to_vec(), false)
            }
            "TextString" => {
                let s = node
                    .value
                    .as_ref()
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| "TextString node requires string value".to_string())?;
                (0x07, s.as_bytes().to_vec(), true)
            }
            "ByteString" => {
                let s = node
                    .value
                    .as_ref()
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| "ByteString node requires hex string value".to_string())?;
                let raw = decode_hex_flexible(s)?;
                (0x08, raw.to_vec(), true)
            }
            other => return Err(format!("Unsupported encode itemType '{other}'")),
        };

    let logical_len: u32 = match node.item_type.as_str() {
        "Integer" | "Enumeration" => 4,
        "Boolean" => 8,
        _ => u32::try_from(payload.len()).map_err(|_| "Payload too large")?,
    };

    let mut out = Vec::with_capacity(8 + payload.len() + 8);
    let tag_bytes = tag_u32.to_be_bytes();
    out.extend_from_slice(&tag_bytes[1..4]);
    out.push(type_code);
    out.extend_from_slice(&logical_len.to_be_bytes());
    out.extend_from_slice(&payload);
    if pad_payload {
        let rem = payload.len() % 8;
        if rem != 0 {
            out.resize(out.len() + (8 - rem), 0u8);
        }
    }
    Ok(out)
}
