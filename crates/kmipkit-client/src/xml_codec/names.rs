//! Catalog-only XML names; Profiles v2.1 §§5.4.1.1–5.4.1.3.
use super::{INVALID_VALUE, WireCodecError};
use crate::wire_names_generated::WIRE_NAMES;

// Profiles §5.4.1.1: preserve acronym case and move initial digits.
pub(super) fn normalize(source: &str) -> String {
    let chars: Vec<_> = source.chars().collect();
    let mut tokens = String::new();
    for (i, &c) in chars.iter().enumerate() {
        if c.is_whitespace()
            || c == '('
            || c == ')'
            || (!c.is_alphanumeric()
                && c != '_'
                && chars.get(i + 1).is_some_and(|n| n.is_alphabetic())
                && chars.get(i + 2).is_some_and(|n| n.is_lowercase()))
        {
            tokens.push(' ');
        } else if c.is_alphanumeric() || c == '_' {
            tokens.push(c);
        } else {
            tokens.push('_');
        }
    }
    let mut output = String::new();
    for (i, token) in tokens.split_whitespace().enumerate() {
        let digits = if i == 0 {
            token.bytes().take_while(u8::is_ascii_digit).count()
        } else {
            0
        };
        let token = if digits > 0 {
            {
                let mut moved = token[digits..].to_owned();
                moved.push_str(&token[..digits]);
                moved
            }
        } else {
            token.to_owned()
        };
        let mut chars = token.chars();
        if let Some(first) = chars.next() {
            output.extend(first.to_uppercase());
            output.extend(chars);
        }
    }
    output
}

pub(super) fn rows(kind: &str) -> impl Iterator<Item = [&'static str; 4]> {
    WIRE_NAMES.lines().filter_map(move |line| {
        let mut fields = line.split('\t');
        let record = fields.next()?;
        if record != kind {
            return None;
        }
        Some([
            record,
            fields.next()?,
            fields.next()?,
            fields.next().unwrap_or_default(),
        ])
    })
}

pub(super) fn tag_name(raw: u32) -> Option<String> {
    rows("T")
        .find(|r| u32::from_str_radix(r[1], 16) == Ok(raw))
        .map(|r| normalize(r[2]))
}

pub(super) fn tag_value(name: &str) -> Result<u32, WireCodecError> {
    if name.starts_with("0x") {
        return hex_number(name, 6).and_then(|v| u32::try_from(v).map_err(|_| INVALID_VALUE));
    }
    rows("T")
        .find(|r| normalize(r[2]) == name)
        .and_then(|r| u32::from_str_radix(r[1], 16).ok())
        .ok_or(INVALID_VALUE)
}

pub(super) fn family(raw: u32, kind: &str) -> Option<&'static str> {
    rows(kind)
        .find(|r| u32::from_str_radix(r[1], 16) == Ok(raw))
        .map(|r| r[2])
}

pub(super) fn enum_name(raw: u32, value: u32) -> Option<String> {
    let family = family(raw, "F")?;
    rows("E")
        .find(|r| r[1] == family && u32::from_str_radix(r[2], 16) == Ok(value))
        .map(|r| normalize(r[3]))
}

pub(super) fn enum_value(raw: u32, value: &str) -> Result<u32, WireCodecError> {
    if value.starts_with("0x") {
        return hex_number(value, 8).and_then(|v| u32::try_from(v).map_err(|_| INVALID_VALUE));
    }
    let family = family(raw, "F").ok_or(INVALID_VALUE)?;
    rows("E")
        .find(|r| r[1] == family && normalize(r[3]) == value)
        .and_then(|r| u32::from_str_radix(r[2], 16).ok())
        .ok_or(INVALID_VALUE)
}

pub(super) fn hex_number(value: &str, digits: usize) -> Result<u64, WireCodecError> {
    let hex = value.strip_prefix("0x").ok_or(INVALID_VALUE)?;
    if hex.len() != digits || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(INVALID_VALUE);
    }
    u64::from_str_radix(hex, 16).map_err(|_| INVALID_VALUE)
}
