//! XML encoding and decoding under OASIS KMIP Profiles.

use std::fmt::Write;

use super::datetime::{format_datetime, hex_to_octets, octets_to_hex, parse_datetime};
use super::names::{NameKind, family, named_value, normalize, tag_name, tag_value, value_name};
use kmipkit_ttlv::{Item, RawTag, Structure, Value, ValueView};
use roxmltree::{Document, Node};

pub fn decode_xml(bytes: &[u8]) -> Result<Item, String> {
    let text = std::str::from_utf8(bytes).map_err(|e| format!("invalid UTF-8 in XML: {e}"))?;
    let doc = Document::parse(text).map_err(|e| format!("XML parse error: {e}"))?;
    decode_xml_node(doc.root_element())
}

pub fn encode_xml(item: &Item) -> Result<String, String> {
    let mut out = String::new();
    encode_xml_node(item, &mut out)?;
    Ok(out)
}

fn decode_xml_node(node: Node<'_, '_>) -> Result<Item, String> {
    let tag_name_str = node.tag_name().name();
    let tag_raw = parse_tag(tag_name_str)?;
    let raw_tag = RawTag::new(tag_raw).map_err(|e| format!("invalid tag: {e}"))?;
    let tag = raw_tag
        .try_checked()
        .map_err(|e| format!("unchecked tag: {e}"))?;

    let element_children: Vec<Node<'_, '_>> = node.children().filter(Node::is_element).collect();

    let type_attr = node.attribute("type");
    let val_attr = node.attribute("value").or_else(|| node.text());

    let item_val = if !element_children.is_empty() {
        let mut structure = Structure::new();
        for child in element_children {
            let child_item = decode_xml_node(child)?;
            structure
                .try_push(child_item)
                .map_err(|e| format!("structure push failed: {e}"))?;
        }
        Value::structure(structure)
    } else if let Some(ty) = type_attr {
        let val_str = val_attr.unwrap_or("").trim();
        match ty {
            "Integer" => {
                let n = val_str
                    .parse::<i32>()
                    .map_err(|e| format!("invalid integer: {e}"))?;
                Value::integer(n)
            }
            "LongInteger" => {
                let n = val_str
                    .parse::<i64>()
                    .map_err(|e| format!("invalid long integer: {e}"))?;
                Value::long_integer(n)
            }
            "BigInteger" => {
                let octets = hex_to_octets(val_str)?;
                Value::big_integer(octets)
            }
            "Enumeration" => {
                let val_u32 = if let Some(stripped) = val_str.strip_prefix("0x") {
                    u32::from_str_radix(stripped, 16)
                        .map_err(|e| format!("invalid hex enum: {e}"))?
                } else if let Some(f) = family(tag_raw, NameKind::Enumeration) {
                    named_value(f, val_str, NameKind::Enumeration)
                        .or_else(|| named_value(f, &normalize(val_str), NameKind::Enumeration))
                        .ok_or_else(|| format!("unknown enum name {val_str} in family {f}"))?
                } else {
                    val_str
                        .parse::<u32>()
                        .map_err(|e| format!("invalid enumeration string: {e}"))?
                };
                Value::enumeration(val_u32)
            }
            "Boolean" => {
                let b = match val_str {
                    "true" | "1" => true,
                    "false" | "0" => false,
                    _ => return Err(format!("invalid boolean: {val_str}")),
                };
                Value::boolean(b)
            }
            "TextString" => Value::text_string(val_str.to_string()),
            "ByteString" => {
                let octets = hex_to_octets(val_str)?;
                Value::byte_string(octets)
            }
            "DateTime" => {
                let secs = parse_datetime(val_str)?;
                Value::date_time(secs)
            }
            "Interval" => {
                let n = val_str
                    .parse::<u32>()
                    .map_err(|e| format!("invalid interval: {e}"))?;
                Value::interval(n)
            }
            "DateTimeExtended" => {
                let n = val_str
                    .parse::<i64>()
                    .map_err(|e| format!("invalid date time extended: {e}"))?;
                Value::date_time_extended(n)
            }
            _ => return Err(format!("unknown XML type attribute: {ty}")),
        }
    } else {
        // Empty element like <ResponsePayload/> is an empty Structure
        Value::structure(Structure::new())
    };

    Item::new(tag, item_val).map_err(|e| format!("item creation failed: {e}"))
}

fn encode_xml_node(item: &Item, out: &mut String) -> Result<(), String> {
    let raw = item.tag().raw();
    let tag_str = tag_name(raw)
        .map(normalize)
        .unwrap_or_else(|| format!("Tag_0x{raw:06X}"));

    item.with_value(|view| match view {
        ValueView::Structure(s) => {
            if s.children().is_empty() {
                let _ = write!(out, "<{tag_str}/>");
            } else {
                let _ = write!(out, "<{tag_str}>");
                for child in s.children() {
                    encode_xml_node(child, out)?;
                }
                let _ = write!(out, "</{tag_str}>");
            }
            Ok(())
        }
        ValueView::Integer(i) => {
            let _ = write!(out, "<{tag_str} type=\"Integer\" value=\"{i}\"/>");
            Ok(())
        }
        ValueView::LongInteger(i) => {
            let _ = write!(out, "<{tag_str} type=\"LongInteger\" value=\"{i}\"/>");
            Ok(())
        }
        ValueView::BigInteger(b) => {
            let hex = octets_to_hex(b);
            let _ = write!(out, "<{tag_str} type=\"BigInteger\" value=\"{hex}\"/>");
            Ok(())
        }
        ValueView::Enumeration(e) => {
            let enum_name = family(raw, NameKind::Enumeration)
                .and_then(|f| value_name(f, *e, NameKind::Enumeration))
                .map(normalize);
            let val = match enum_name {
                Some(name) => name,
                None => e.to_string(),
            };
            let _ = write!(out, "<{tag_str} type=\"Enumeration\" value=\"{val}\"/>");
            Ok(())
        }
        ValueView::Boolean(b) => {
            let _ = write!(out, "<{tag_str} type=\"Boolean\" value=\"{b}\"/>");
            Ok(())
        }
        ValueView::TextString(s) => {
            let escaped = escape_xml(s);
            let _ = write!(out, "<{tag_str} type=\"TextString\" value=\"{escaped}\"/>");
            Ok(())
        }
        ValueView::ByteString(b) => {
            let hex = octets_to_hex(b);
            let _ = write!(out, "<{tag_str} type=\"ByteString\" value=\"{hex}\"/>");
            Ok(())
        }
        ValueView::DateTime(dt) => {
            let val = format_datetime(*dt);
            let _ = write!(out, "<{tag_str} type=\"DateTime\" value=\"{val}\"/>");
            Ok(())
        }
        ValueView::Interval(i) => {
            let _ = write!(out, "<{tag_str} type=\"Interval\" value=\"{i}\"/>");
            Ok(())
        }
        ValueView::DateTimeExtended(dt) => {
            let _ = write!(out, "<{tag_str} type=\"DateTimeExtended\" value=\"{dt}\"/>");
            Ok(())
        }
        _ => Err(format!("unsupported value view for tag 0x{raw:06X}")),
    })
}

fn escape_xml(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

fn parse_tag(tag_str: &str) -> Result<u32, String> {
    if let Some(stripped) = tag_str
        .strip_prefix("0x")
        .or_else(|| tag_str.strip_prefix("Tag_0x"))
    {
        u32::from_str_radix(stripped, 16).map_err(|e| format!("invalid hex tag: {e}"))
    } else if let Some(val) = tag_value(tag_str).or_else(|| tag_value(&normalize(tag_str))) {
        Ok(val)
    } else {
        Err(format!("unknown tag name: {tag_str}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xml_codec_roundtrip() {
        let xml_input = br#"<ResponseMessage><ResponseHeader><ProtocolVersion><ProtocolVersionMajor type="Integer" value="2"/><ProtocolVersionMinor type="Integer" value="1"/></ProtocolVersion><TimeStamp type="DateTime" value="1970-01-01T00:00:01Z"/><BatchCount type="Integer" value="1"/></ResponseHeader><BatchItem><Operation type="Enumeration" value="DiscoverVersions"/><ResultStatus type="Enumeration" value="Success"/><ResponsePayload/></BatchItem></ResponseMessage>"#;

        let item = decode_xml(xml_input).expect("XML decode succeeds");
        assert_eq!(item.tag().raw(), 0x0042_007B);

        let encoded = encode_xml(&item).expect("XML encode succeeds");
        assert!(encoded.contains("<ResponseMessage>"));
        assert!(encoded.contains("<Operation type=\"Enumeration\" value=\"DiscoverVersions\"/>"));
        assert!(encoded.contains("<ResponsePayload/>"));
    }
}
