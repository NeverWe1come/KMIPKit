//! XML 1.0 character and zeroizing attribute handling.
use super::{ALLOCATION, INVALID_DOCUMENT, WireCodecError};
use zeroize::Zeroizing;

pub(super) fn xml_space(c: char) -> bool {
    matches!(c, ' ' | '\r' | '\n' | '\t')
}
pub(super) fn xml_char(c: char) -> bool {
    matches!(c, '\t' | '\n' | '\r' | '\u{20}'..='\u{D7FF}' | '\u{E000}'..='\u{FFFD}' | '\u{10000}'..='\u{10FFFF}')
}

pub(super) fn attribute(value: &str) -> Result<Zeroizing<String>, WireCodecError> {
    let mut output = Zeroizing::new(String::new());
    output
        .try_reserve_exact(value.len())
        .map_err(|_| ALLOCATION)?;
    attribute_chars(value, |c| output.push(c))?;
    Ok(output)
}

/// Compare normalized type metadata without allocating during lexical preflight.
pub(super) fn attribute_matches(value: &str, expected: &str) -> Result<bool, WireCodecError> {
    let mut expected = expected.chars();
    let mut matches = true;
    attribute_chars(value, |c| matches &= expected.next() == Some(c))?;
    Ok(matches && expected.next().is_none())
}

fn attribute_chars(value: &str, mut emit: impl FnMut(char)) -> Result<(), WireCodecError> {
    let mut chars = value.chars();
    while let Some(c) = chars.next() {
        match c {
            '<' => return Err(INVALID_DOCUMENT),
            '\r' => {
                if chars.as_str().starts_with('\n') {
                    chars.next();
                }
                emit(' ');
            }
            '\t' | '\n' => emit(' '),
            '&' => {
                let remaining = chars.as_str();
                let end = remaining.find(';').ok_or(INVALID_DOCUMENT)?;
                let c = match &remaining[..end] {
                    "lt" => '<',
                    "gt" => '>',
                    "amp" => '&',
                    "apos" => '\'',
                    "quot" => '"',
                    entity => {
                        let number = if let Some(hex) = entity.strip_prefix("#x") {
                            if hex.is_empty() || !hex.bytes().all(|c| c.is_ascii_hexdigit()) {
                                return Err(INVALID_DOCUMENT);
                            }
                            u32::from_str_radix(hex, 16)
                        } else if let Some(decimal) = entity.strip_prefix('#') {
                            if decimal.is_empty() || !decimal.bytes().all(|c| c.is_ascii_digit()) {
                                return Err(INVALID_DOCUMENT);
                            }
                            decimal.parse()
                        } else {
                            return Err(INVALID_DOCUMENT);
                        }
                        .map_err(|_| INVALID_DOCUMENT)?;
                        char::from_u32(number)
                            .filter(|c| xml_char(*c))
                            .ok_or(INVALID_DOCUMENT)?
                    }
                };
                for _ in 0..=end {
                    chars.next();
                }
                emit(c);
            }
            _ => emit(c),
        }
    }
    Ok(())
}
