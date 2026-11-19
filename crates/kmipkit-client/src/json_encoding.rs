//! Private KMIP JSON encoding: OASIS KMIP Profiles v2.1 §5.5.1.
//!
//! A bounded lexical pass validates the whole document before
//! constructing payload owners. The second pass borrows field spans rather than
//! creating a second JSON tree. Output uses a counting pass and one reservation
//! so owned initialized wire bytes never move through a growable allocation.
use crate::wire_codec_common::{WireCodecError, WireCodecErrorKind};
use kmipkit_ttlv::{Item, RawTag, Structure, Value, codec::CodecLimits};
use zeroize::Zeroizing;

mod names;
mod scalars;
mod writer;
use names::{NameKind, family, named_value, tag_value};
use scalars::{date_time, hex_bytes, hex_u64, signed_number};
use writer::Writer;

const fn error(kind: WireCodecErrorKind) -> WireCodecError {
    WireCodecError::new(kind)
}

pub(crate) fn decode(bytes: &[u8], limits: &CodecLimits) -> Result<Item, WireCodecError> {
    if bytes.len() > limits.max_message_bytes() {
        return Err(error(WireCodecErrorKind::LimitExceeded));
    }
    let input =
        std::str::from_utf8(bytes).map_err(|_| error(WireCodecErrorKind::InvalidDocument))?;
    let mut scan = Scanner::new(input, limits);
    scan.value(0)?;
    scan.space();
    if scan.pos != input.len() {
        return Err(error(WireCodecErrorKind::InvalidDocument));
    }
    read_item(input, limits)
}

pub(crate) fn encode(
    item: &Item,
    limits: &CodecLimits,
) -> Result<Zeroizing<Vec<u8>>, WireCodecError> {
    let mut count = Writer {
        output: None,
        length: 0,
        elements: 0,
        limits,
    };
    count.item(item, 0)?;
    let mut bytes = Zeroizing::new(Vec::new());
    bytes
        .try_reserve_exact(count.length)
        .map_err(|_| error(WireCodecErrorKind::AllocationFailed))?;
    let mut writer = Writer {
        output: Some(bytes),
        length: 0,
        elements: 0,
        limits,
    };
    writer.item(item, 0)?;
    writer
        .output
        .ok_or_else(|| error(WireCodecErrorKind::AllocationFailed))
}

struct Scanner<'a, 'l> {
    input: &'a str,
    pos: usize,
    elements: usize,
    nesting: usize,
    limits: &'l CodecLimits,
}
impl<'a, 'l> Scanner<'a, 'l> {
    fn new(input: &'a str, limits: &'l CodecLimits) -> Self {
        Self {
            input,
            pos: 0,
            elements: 0,
            nesting: 0,
            limits,
        }
    }
    fn space(&mut self) {
        while self
            .peek()
            .is_some_and(|b| matches!(b, b' ' | b'\t' | b'\r' | b'\n'))
        {
            self.pos += 1;
        }
    }
    fn peek(&self) -> Option<u8> {
        self.input.as_bytes().get(self.pos).copied()
    }
    fn take(&mut self, byte: u8) -> Result<(), WireCodecError> {
        self.space();
        if self.peek() != Some(byte) {
            return Err(error(WireCodecErrorKind::InvalidDocument));
        }
        self.pos += 1;
        Ok(())
    }
    fn string(&mut self) -> Result<&'a str, WireCodecError> {
        self.space();
        let start = self.pos;
        self.take(b'"')?;
        loop {
            let b = self
                .peek()
                .ok_or_else(|| error(WireCodecErrorKind::InvalidDocument))?;
            self.pos += 1;
            match b {
                b'"' => return Ok(&self.input[start..self.pos]),
                0..=31 => return Err(error(WireCodecErrorKind::InvalidDocument)),
                b'\\' => {
                    let escaped = self
                        .peek()
                        .ok_or_else(|| error(WireCodecErrorKind::InvalidDocument))?;
                    self.pos += 1;
                    match escaped {
                        b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't' => (),
                        b'u' => {
                            let first = self.codepoint()?;
                            if (0xd800..=0xdbff).contains(&first) {
                                if self.input.as_bytes().get(self.pos..self.pos + 2) != Some(b"\\u")
                                {
                                    return Err(error(WireCodecErrorKind::InvalidDocument));
                                }
                                self.pos += 2;
                                let second = self.codepoint()?;
                                if !(0xdc00..=0xdfff).contains(&second) {
                                    return Err(error(WireCodecErrorKind::InvalidDocument));
                                }
                            } else if (0xdc00..=0xdfff).contains(&first) {
                                return Err(error(WireCodecErrorKind::InvalidDocument));
                            }
                        }
                        _ => return Err(error(WireCodecErrorKind::InvalidDocument)),
                    }
                }
                _ => (),
            }
        }
    }
    fn codepoint(&mut self) -> Result<u32, WireCodecError> {
        let end = self
            .pos
            .checked_add(4)
            .ok_or_else(|| error(WireCodecErrorKind::InvalidDocument))?;
        let text = self
            .input
            .get(self.pos..end)
            .ok_or_else(|| error(WireCodecErrorKind::InvalidDocument))?;
        if !text.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(error(WireCodecErrorKind::InvalidDocument));
        }
        let n = u32::from_str_radix(text, 16)
            .map_err(|_| error(WireCodecErrorKind::InvalidDocument))?;
        self.pos = end;
        Ok(n)
    }
    fn value(&mut self, depth: usize) -> Result<&'a str, WireCodecError> {
        // Invalid JSON object nesting must not bypass the Structure-array bound.
        if self.nesting >= self.limits.max_structure_depth() * 2 + 4 {
            return Err(error(WireCodecErrorKind::LimitExceeded));
        }
        self.nesting += 1;
        self.space();
        let start = self.pos;
        match self.peek() {
            Some(b'{') => {
                self.object(depth)?;
            }
            Some(b'[') => {
                // Every JSON array is a Structure's value in the KMIP grammar.
                if depth >= self.limits.max_structure_depth() {
                    return Err(error(WireCodecErrorKind::LimitExceeded));
                }
                self.pos += 1;
                self.space();
                if self.peek() != Some(b']') {
                    loop {
                        self.value(depth + 1)?;
                        self.space();
                        if self.peek() != Some(b',') {
                            break;
                        }
                        self.pos += 1;
                    }
                }
                self.take(b']')?;
            }
            Some(b'"') => {
                self.string()?;
            }
            Some(b'-' | b'0'..=b'9') => {
                if self.peek() == Some(b'-') {
                    self.pos += 1;
                }
                match self.peek() {
                    Some(b'0') => self.pos += 1,
                    Some(b'1'..=b'9') => {
                        self.pos += 1;
                        while self.peek().is_some_and(|b| b.is_ascii_digit()) {
                            self.pos += 1;
                        }
                    }
                    _ => return Err(error(WireCodecErrorKind::InvalidDocument)),
                }
                // KMIP numeric values are integers, never fractional/exponent JSON numbers.
            }
            _ => {
                let token = ["true", "false", "null"]
                    .into_iter()
                    .find(|t| self.input[self.pos..].starts_with(t))
                    .ok_or_else(|| error(WireCodecErrorKind::InvalidDocument))?;
                self.pos += token.len();
            }
        }
        self.nesting -= 1;
        Ok(&self.input[start..self.pos])
    }
    fn object(&mut self, depth: usize) -> Result<[Option<&'a str>; 4], WireCodecError> {
        self.elements = self
            .elements
            .checked_add(1)
            .ok_or_else(|| error(WireCodecErrorKind::LimitExceeded))?;
        if self.elements > self.limits.max_elements() {
            return Err(error(WireCodecErrorKind::LimitExceeded));
        }
        self.take(b'{')?;
        self.space();
        let mut fields = [None; 4];
        if self.peek() != Some(b'}') {
            loop {
                let key = text(self.string()?)?;
                let slot = match key.as_str() {
                    "tag" => 0,
                    "name" => 1,
                    "type" => 2,
                    "value" => 3,
                    _ => return Err(error(WireCodecErrorKind::InvalidDocument)),
                };
                if fields[slot].is_some() {
                    return Err(error(WireCodecErrorKind::InvalidDocument));
                }
                self.take(b':')?;
                fields[slot] = Some(self.value(depth)?);
                self.space();
                if self.peek() != Some(b',') {
                    break;
                }
                self.pos += 1;
            }
        }
        self.take(b'}')?;
        // A null value still denotes a Structure and consumes one Structure level.
        if fields[3] == Some("null") && depth >= self.limits.max_structure_depth() {
            return Err(error(WireCodecErrorKind::LimitExceeded));
        }
        Ok(fields)
    }
}

fn text(source: &str) -> Result<Zeroizing<String>, WireCodecError> {
    let inside = source
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .ok_or_else(|| error(WireCodecErrorKind::InvalidValue))?;
    let mut result = Zeroizing::new(String::new());
    result
        .try_reserve_exact(inside.len())
        .map_err(|_| error(WireCodecErrorKind::AllocationFailed))?;
    let mut chars = inside.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            result.push(c);
            continue;
        }
        let escaped = chars
            .next()
            .ok_or_else(|| error(WireCodecErrorKind::InvalidDocument))?;
        result.push(match escaped {
            '"' => '"',
            '\\' => '\\',
            '/' => '/',
            'b' => '\u{8}',
            'f' => '\u{c}',
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            'u' => {
                fn quad(chars: &mut std::str::Chars<'_>) -> Result<u32, WireCodecError> {
                    let mut n = 0;
                    for _ in 0..4 {
                        n = n * 16
                            + chars
                                .next()
                                .and_then(|c| c.to_digit(16))
                                .ok_or_else(|| error(WireCodecErrorKind::InvalidDocument))?;
                    }
                    Ok(n)
                }
                let first = quad(&mut chars)?;
                let code = if (0xd800..=0xdbff).contains(&first) {
                    if chars.next() != Some('\\') || chars.next() != Some('u') {
                        return Err(error(WireCodecErrorKind::InvalidDocument));
                    }
                    let second = quad(&mut chars)?;
                    if !(0xdc00..=0xdfff).contains(&second) {
                        return Err(error(WireCodecErrorKind::InvalidDocument));
                    }
                    0x10000 + ((first - 0xd800) << 10) + (second - 0xdc00)
                } else {
                    first
                };
                char::from_u32(code).ok_or_else(|| error(WireCodecErrorKind::InvalidDocument))?
            }
            _ => return Err(error(WireCodecErrorKind::InvalidDocument)),
        });
    }
    Ok(result)
}

fn read_item(source: &str, limits: &CodecLimits) -> Result<Item, WireCodecError> {
    let mut scanner = Scanner::new(source, limits);
    let [tag, name, ty, value] = scanner.object(0)?;
    if let Some(name) = name {
        text(name)?;
    }
    let tag_text = text(tag.ok_or_else(|| error(WireCodecErrorKind::InvalidDocument))?)?;
    let raw = if tag_text.starts_with("0x") {
        u32::try_from(hex_u64(&tag_text, 3)?)
            .map_err(|_| error(WireCodecErrorKind::InvalidValue))?
    } else {
        tag_value(&tag_text).ok_or_else(|| error(WireCodecErrorKind::InvalidValue))?
    };
    let tag = RawTag::new(raw)
        .and_then(|tag| tag.try_checked())
        .map_err(|_| error(WireCodecErrorKind::InvalidValue))?;
    let ty = ty.map(text).transpose()?;
    let ty = ty.as_deref().map_or("Structure", String::as_str);
    let source = value.ok_or_else(|| error(WireCodecErrorKind::InvalidDocument))?;
    let value = if ty == "Structure" {
        let mut structure = Structure::new();
        if source != "null" {
            let mut scanner = Scanner::new(source, limits);
            scanner.take(b'[')?;
            scanner.space();
            if scanner.peek() != Some(b']') {
                loop {
                    let span = scanner.value(0)?;
                    structure
                        .try_push(read_item(span, limits)?)
                        .map_err(|_| error(WireCodecErrorKind::LimitExceeded))?;
                    scanner.space();
                    if scanner.peek() != Some(b',') {
                        break;
                    }
                    scanner.pos += 1;
                }
            }
            scanner.take(b']')?;
        }
        Value::structure(structure)
    } else {
        scalar(raw, ty, source)?
    };
    Item::new(tag, value).map_err(|_| error(WireCodecErrorKind::InvalidValue))
}

fn integer_value(tag: u32, source: &str, string: Option<&str>) -> Result<Value, WireCodecError> {
    let invalid = || error(WireCodecErrorKind::InvalidValue);

    let n = if let Some(s) = string {
        if let Some(family) = family(tag, NameKind::Mask) {
            let mut n = 0u32;
            for component in s.split('|') {
                n |= if component.starts_with("0x") {
                    u32::try_from(hex_u64(component, 4)?).map_err(|_| invalid())?
                } else {
                    named_value(family, component, NameKind::Mask).ok_or_else(invalid)?
                };
            }
            n
        } else {
            u32::try_from(hex_u64(s, 4)?).map_err(|_| invalid())?
        }
    } else {
        i32::try_from(signed_number(source)?)
            .map_err(|_| invalid())?
            .cast_unsigned()
    };
    Ok(Value::integer(n.cast_signed()))
}

fn scalar(tag: u32, ty: &str, source: &str) -> Result<Value, WireCodecError> {
    let string = if source.starts_with('"') {
        Some(text(source)?)
    } else {
        None
    };
    let string = string.as_deref().map(String::as_str);
    let invalid = || error(WireCodecErrorKind::InvalidValue);
    Ok(match ty {
        "Integer" => integer_value(tag, source, string)?,
        "LongInteger" | "DateTimeExtended" => {
            let n = if let Some(s) = string {
                hex_u64(s, 8)?.cast_signed()
            } else {
                signed_number(source)?
            };
            if ty == "LongInteger" {
                Value::long_integer(n)
            } else {
                Value::date_time_extended(n)
            }
        }
        "BigInteger" => {
            let mut bytes = if let Some(s) = string {
                let s = s.strip_prefix("0x").ok_or_else(invalid)?;
                if s.is_empty() || !s.len().is_multiple_of(16) {
                    return Err(invalid());
                }
                hex_bytes(s)?
            } else {
                let mut v = Vec::new();
                v.try_reserve_exact(8)
                    .map_err(|_| error(WireCodecErrorKind::AllocationFailed))?;
                v.extend_from_slice(&signed_number(source)?.to_be_bytes());
                Zeroizing::new(v)
            };
            Value::big_integer(std::mem::take(&mut *bytes))
        }
        "Enumeration" => {
            let n = if let Some(s) = string {
                if s.starts_with("0x") {
                    u32::try_from(hex_u64(s, 4)?).map_err(|_| invalid())?
                } else {
                    family(tag, NameKind::Enumeration)
                        .and_then(|f| named_value(f, s, NameKind::Enumeration))
                        .ok_or_else(invalid)?
                }
            } else {
                source.parse::<u32>().map_err(|_| invalid())?
            };
            Value::enumeration(n)
        }
        "Boolean" => Value::boolean(match (string, source) {
            (None, "true") => true,
            (None, "false") => false,
            (Some(s), _) => match hex_u64(s, 8)? {
                0 => false,
                1 => true,
                _ => return Err(invalid()),
            },
            _ => return Err(invalid()),
        }),
        "TextString" => {
            let mut value = text(source)?;
            Value::text_string(std::mem::take(&mut *value))
        }
        "ByteString" => {
            let mut value = hex_bytes(string.ok_or_else(invalid)?)?;
            Value::byte_string(std::mem::take(&mut *value))
        }
        "DateTime" => {
            let s = string.ok_or_else(invalid)?;
            Value::date_time(if s.starts_with("0x") {
                hex_u64(s, 8)?.cast_signed()
            } else {
                date_time(s)?
            })
        }
        "Interval" => Value::interval(if let Some(s) = string {
            u32::try_from(hex_u64(s, 4)?).map_err(|_| invalid())?
        } else {
            source.parse().map_err(|_| invalid())?
        }),
        _ => return Err(invalid()),
    })
}

#[cfg(test)]
#[path = "../tests/unit/json_encoding_tests.rs"]
mod tests;
