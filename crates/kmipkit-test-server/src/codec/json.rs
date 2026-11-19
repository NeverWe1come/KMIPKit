//! JSON encoding and decoding under OASIS KMIP Profiles.

use super::datetime::{format_datetime, hex_to_octets, octets_to_hex, parse_datetime};
use super::names::{NameKind, family, named_value, normalize, tag_name, tag_value, value_name};
use kmipkit_ttlv::{Item, RawTag, Structure, Value, ValueView};
use serde_json::{Value as JsonVal, json};

pub fn decode_json(bytes: &[u8]) -> Result<Item, String> {
    let json_val: JsonVal =
        serde_json::from_slice(bytes).map_err(|e| format!("invalid JSON: {e}"))?;
    decode_json_element(&json_val)
}

pub fn encode_json(item: &Item) -> Result<String, String> {
    let val = encode_json_element(item)?;
    serde_json::to_string(&val).map_err(|e| format!("JSON serialization error: {e}"))
}

fn decode_json_element(v: &JsonVal) -> Result<Item, String> {
    let obj = v
        .as_object()
        .ok_or_else(|| "expected JSON object".to_string())?;
    let tag_str = obj
        .get("tag")
        .and_then(JsonVal::as_str)
        .ok_or_else(|| "missing tag string".to_string())?;

    let tag_raw = parse_tag(tag_str)?;
    let raw_tag = RawTag::new(tag_raw).map_err(|e| format!("invalid tag: {e}"))?;
    let tag = raw_tag
        .try_checked()
        .map_err(|e| format!("unchecked tag: {e}"))?;

    let type_str = obj.get("type").and_then(JsonVal::as_str);
    let value_elem = obj
        .get("value")
        .ok_or_else(|| "missing value field".to_string())?;

    let item_val = if type_str == Some("Structure") || (type_str.is_none() && value_elem.is_array())
    {
        let arr = value_elem
            .as_array()
            .ok_or_else(|| "expected array for Structure".to_string())?;
        let mut structure = Structure::new();
        for child_json in arr {
            let child = decode_json_element(child_json)?;
            structure
                .try_push(child)
                .map_err(|e| format!("structure push failed: {e}"))?;
        }
        Value::structure(structure)
    } else {
        match type_str {
            Some("Integer") => {
                let n = if let Some(i) = value_elem.as_i64() {
                    i32::try_from(i).map_err(|e| format!("integer overflow: {e}"))?
                } else if let Some(s) = value_elem.as_str() {
                    s.parse::<i32>()
                        .map_err(|e| format!("invalid integer: {e}"))?
                } else {
                    return Err("invalid Integer value".to_string());
                };
                Value::integer(n)
            }
            Some("LongInteger") => {
                let n = if let Some(i) = value_elem.as_i64() {
                    i
                } else if let Some(s) = value_elem.as_str() {
                    s.parse::<i64>()
                        .map_err(|e| format!("invalid long integer: {e}"))?
                } else {
                    return Err("invalid LongInteger value".to_string());
                };
                Value::long_integer(n)
            }
            Some("BigInteger") => {
                let s = value_elem
                    .as_str()
                    .ok_or_else(|| "expected hex string for BigInteger".to_string())?;
                let octets = hex_to_octets(s)?;
                Value::big_integer(octets)
            }
            Some("Enumeration") => {
                let val_u32 = if let Some(s) = value_elem.as_str() {
                    if let Some(stripped) = s.strip_prefix("0x") {
                        u32::from_str_radix(stripped, 16)
                            .map_err(|e| format!("invalid hex enum: {e}"))?
                    } else if let Some(f) = family(tag_raw, NameKind::Enumeration) {
                        named_value(f, s, NameKind::Enumeration)
                            .or_else(|| named_value(f, &normalize(s), NameKind::Enumeration))
                            .ok_or_else(|| format!("unknown enum name {s} in family {f}"))?
                    } else {
                        s.parse::<u32>()
                            .map_err(|e| format!("invalid enumeration string: {e}"))?
                    }
                } else if let Some(i) = value_elem.as_u64() {
                    u32::try_from(i).map_err(|e| format!("enum overflow: {e}"))?
                } else {
                    return Err("invalid Enumeration value".to_string());
                };
                Value::enumeration(val_u32)
            }
            Some("Boolean") => {
                let b = value_elem
                    .as_bool()
                    .ok_or_else(|| "expected boolean".to_string())?;
                Value::boolean(b)
            }
            Some("TextString") => {
                let s = value_elem
                    .as_str()
                    .ok_or_else(|| "expected string for TextString".to_string())?;
                Value::text_string(s.to_string())
            }
            Some("ByteString") => {
                let s = value_elem
                    .as_str()
                    .ok_or_else(|| "expected hex string for ByteString".to_string())?;
                let octets = hex_to_octets(s)?;
                Value::byte_string(octets)
            }
            Some("DateTime") => {
                let secs = if let Some(s) = value_elem.as_str() {
                    parse_datetime(s).or_else(|_| {
                        let clean_hex = s.strip_prefix("0x").unwrap_or(s);
                        if let Ok(v) = u64::from_str_radix(clean_hex, 16) {
                            Ok(v as i64)
                        } else {
                            s.parse::<i64>()
                                .map_err(|e| format!("invalid datetime int: {e}"))
                        }
                    })?
                } else if let Some(i) = value_elem.as_i64() {
                    i
                } else {
                    return Err("invalid DateTime value".to_string());
                };
                Value::date_time(secs)
            }
            Some("Interval") => {
                let i = value_elem
                    .as_u64()
                    .ok_or_else(|| "expected unsigned integer for Interval".to_string())?;
                let val_u32 = u32::try_from(i).map_err(|e| format!("interval overflow: {e}"))?;
                Value::interval(val_u32)
            }
            Some("DateTimeExtended") => {
                let i = value_elem
                    .as_i64()
                    .ok_or_else(|| "expected integer for DateTimeExtended".to_string())?;
                Value::date_time_extended(i)
            }
            _ => {
                if let Some(b) = value_elem.as_bool() {
                    Value::boolean(b)
                } else if let Some(i) = value_elem.as_i64() {
                    Value::integer(i32::try_from(i).unwrap_or(0))
                } else if let Some(s) = value_elem.as_str() {
                    Value::text_string(s.to_string())
                } else {
                    return Err(format!(
                        "unsupported value type without explicit type tag: {value_elem:?}"
                    ));
                }
            }
        }
    };

    Item::new(tag, item_val).map_err(|e| format!("item creation failed: {e}"))
}

fn encode_json_element(item: &Item) -> Result<JsonVal, String> {
    let raw = item.tag().raw();
    let tag_str = tag_name(raw)
        .map(normalize)
        .unwrap_or_else(|| format!("0x{raw:06X}"));

    item.with_value(|view| match view {
        ValueView::Structure(s) => {
            let mut arr = Vec::with_capacity(s.children().len());
            for child in s.children() {
                arr.push(encode_json_element(child)?);
            }
            Ok(json!({
                "tag": tag_str,
                "value": arr
            }))
        }
        ValueView::Integer(i) => Ok(json!({
            "tag": tag_str,
            "type": "Integer",
            "value": *i
        })),
        ValueView::LongInteger(i) => Ok(json!({
            "tag": tag_str,
            "type": "LongInteger",
            "value": *i
        })),
        ValueView::BigInteger(b) => Ok(json!({
            "tag": tag_str,
            "type": "BigInteger",
            "value": octets_to_hex(b)
        })),
        ValueView::Enumeration(e) => {
            let enum_name = family(raw, NameKind::Enumeration)
                .and_then(|f| value_name(f, *e, NameKind::Enumeration))
                .map(normalize);
            let val = match enum_name {
                Some(name) => json!(name),
                None => json!(*e),
            };
            Ok(json!({
                "tag": tag_str,
                "type": "Enumeration",
                "value": val
            }))
        }
        ValueView::Boolean(b) => Ok(json!({
            "tag": tag_str,
            "type": "Boolean",
            "value": *b
        })),
        ValueView::TextString(s) => Ok(json!({
            "tag": tag_str,
            "type": "TextString",
            "value": *s
        })),
        ValueView::ByteString(b) => Ok(json!({
            "tag": tag_str,
            "type": "ByteString",
            "value": octets_to_hex(b)
        })),
        ValueView::DateTime(dt) => Ok(json!({
            "tag": tag_str,
            "type": "DateTime",
            "value": format_datetime(*dt)
        })),
        ValueView::Interval(i) => Ok(json!({
            "tag": tag_str,
            "type": "Interval",
            "value": *i
        })),
        ValueView::DateTimeExtended(dt) => Ok(json!({
            "tag": tag_str,
            "type": "DateTimeExtended",
            "value": *dt
        })),
        _ => Err(format!("unsupported value view for tag 0x{raw:06X}")),
    })
}

fn parse_tag(tag_str: &str) -> Result<u32, String> {
    if let Some(stripped) = tag_str.strip_prefix("0x") {
        u32::from_str_radix(stripped, 16).map_err(|e| format!("invalid hex tag: {e}"))
    } else if let Some(val) = tag_value(tag_str).or_else(|| tag_value(&normalize(tag_str))) {
        Ok(val)
    } else {
        Err(format!("unknown tag name: {tag_str}"))
    }
}
