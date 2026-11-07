//! Allocation-free lexical resource checks before constructing the XML reader.
use super::characters::{attribute_matches, xml_space};
use super::{INVALID_DOCUMENT, LIMIT, WireCodecError};
use kmipkit_ttlv::codec::CodecLimits;
use quick_xml::events::BytesStart;

/// Bound the reader's internal name stack before it can copy any input names.
/// This is a resource pass, not a replacement for XML or KMIP validation.
pub(super) fn check(mut text: &str, limits: &CodecLimits) -> Result<(), WireCodecError> {
    let mut depth = 0_usize;
    let mut count = 0_usize;
    while let Some(offset) = text.find('<') {
        text = &text[offset..];
        let special = if text.starts_with("<!--") {
            Some((4, "-->"))
        } else if text.starts_with("<![CDATA[") {
            Some((9, "]]>"))
        } else if text.starts_with("<?") {
            Some((2, "?>"))
        } else {
            None
        };
        if let Some((start, terminator)) = special {
            let end = text[start..].find(terminator).ok_or(INVALID_DOCUMENT)?;
            text = &text[start + end + terminator.len()..];
            continue;
        }
        if text.starts_with("<!") {
            return Err(INVALID_DOCUMENT);
        }
        if text.starts_with("</") {
            let end = text.find('>').ok_or(INVALID_DOCUMENT)?;
            depth = depth.checked_sub(1).ok_or(INVALID_DOCUMENT)?;
            text = &text[end + 1..];
            continue;
        }
        let end = start_end(text)?;
        let content = &text[1..end];
        let empty = content.ends_with('/');
        let content = content.strip_suffix('/').unwrap_or(content);
        if count >= limits.max_elements() || depth > limits.max_structure_depth() {
            return Err(LIMIT);
        }
        count += 1;
        if depth == limits.max_structure_depth() && is_structure(content)? {
            return Err(LIMIT);
        }
        if !empty {
            depth = depth.checked_add(1).ok_or(LIMIT)?;
        }
        text = &text[end + 1..];
    }
    if depth != 0 {
        return Err(INVALID_DOCUMENT);
    }
    Ok(())
}

fn start_end(text: &str) -> Result<usize, WireCodecError> {
    let mut quote = None;
    for (offset, byte) in text.bytes().enumerate().skip(1) {
        match (quote, byte) {
            (Some(open), close) if open == close => quote = None,
            (None, b'\'' | b'"') => quote = Some(byte),
            (None, b'>') => return Ok(offset),
            _ => {}
        }
    }
    Err(INVALID_DOCUMENT)
}

fn is_structure(content: &str) -> Result<bool, WireCodecError> {
    let name_len = content.find(xml_space).unwrap_or(content.len());
    let event = BytesStart::from_content(content, name_len);
    // Duplicate checking allocates; full validation retains it after preflight.
    for attribute in event.attributes().with_checks(false) {
        let attribute = attribute.map_err(|_| INVALID_DOCUMENT)?;
        if attribute.key.as_ref() == "type" {
            return attribute_matches(&attribute.value, "Structure");
        }
    }
    Ok(true)
}
