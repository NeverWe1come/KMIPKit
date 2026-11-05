//! Private bounded KMIP 2.1 XML codec; OASIS Profiles v2.1 §5.4.1.
//!
//! Events borrow the caller's bytes. DTDs are rejected, with no entity
//! resolver, schema processor, network path, or document DOM. Attribute
//! scratch, open element metadata and output use zeroizing owners.
use crate::wire_codec_common::{WireCodecError, WireCodecErrorKind};
use crate::wire_names_generated::WIRE_NAMES;
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::{Item, RawTag, Structure, Value, ValueView};
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use zeroize::Zeroizing;

type Result<T> = std::result::Result<T, WireCodecError>;
const NAMESPACE: &str = "urn:oasis:tc:kmip:xmlns";
const INVALID_DOCUMENT: WireCodecError = WireCodecError::new(WireCodecErrorKind::InvalidDocument);
const INVALID_VALUE: WireCodecError = WireCodecError::new(WireCodecErrorKind::InvalidValue);
const LIMIT: WireCodecError = WireCodecError::new(WireCodecErrorKind::LimitExceeded);
const ALLOCATION: WireCodecError = WireCodecError::new(WireCodecErrorKind::AllocationFailed);

// Profiles §5.4.1.1: preserve acronym case and move initial digits.
fn normalize(source: &str) -> String {
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
            format!("{}{}", &token[digits..], &token[..digits])
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

fn rows(kind: &str) -> impl Iterator<Item = Vec<&'static str>> {
    WIRE_NAMES
        .lines()
        .filter(move |line| line.split('\t').next() == Some(kind))
        .map(|line| line.split('\t').collect())
}

fn tag_name(raw: u32) -> Option<String> {
    rows("T")
        .find(|r| u32::from_str_radix(r[1], 16) == Ok(raw))
        .map(|r| normalize(r[2]))
}

fn tag_value(name: &str) -> Result<u32> {
    if name.starts_with("0x") {
        return hex_number(name, 6).and_then(|v| u32::try_from(v).map_err(|_| INVALID_VALUE));
    }
    rows("T")
        .find(|r| normalize(r[2]) == name)
        .and_then(|r| u32::from_str_radix(r[1], 16).ok())
        .ok_or(INVALID_VALUE)
}

fn family(raw: u32, kind: &str) -> Option<&'static str> {
    rows(kind)
        .find(|r| u32::from_str_radix(r[1], 16) == Ok(raw))
        .map(|r| r[2])
}

fn enum_name(raw: u32, value: u32) -> Option<String> {
    let family = family(raw, "F")?;
    rows("E")
        .find(|r| r[1] == family && u32::from_str_radix(r[2], 16) == Ok(value))
        .map(|r| normalize(r[3]))
}

fn enum_value(raw: u32, value: &str) -> Result<u32> {
    if value.starts_with("0x") {
        return hex_number(value, 8).and_then(|v| u32::try_from(v).map_err(|_| INVALID_VALUE));
    }
    let family = family(raw, "F").ok_or(INVALID_VALUE)?;
    rows("E")
        .find(|r| r[1] == family && normalize(r[3]) == value)
        .and_then(|r| u32::from_str_radix(r[2], 16).ok())
        .ok_or(INVALID_VALUE)
}

fn hex_number(value: &str, digits: usize) -> Result<u64> {
    let hex = value.strip_prefix("0x").ok_or(INVALID_VALUE)?;
    if hex.len() != digits || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(INVALID_VALUE);
    }
    u64::from_str_radix(hex, 16).map_err(|_| INVALID_VALUE)
}

fn integer(raw: u32, value: &str) -> Result<i32> {
    if let Ok(number) = value.parse::<i32>() {
        return Ok(number);
    }
    let mut bits = 0_u32;
    let mut found = false;
    let mask_family = family(raw, "M");
    for component in value.split_ascii_whitespace() {
        if found && mask_family.is_none() {
            return Err(INVALID_VALUE);
        }
        let part = if component.starts_with("0x") {
            u32::try_from(hex_number(component, 8)?).map_err(|_| INVALID_VALUE)?
        } else {
            let family = mask_family.ok_or(INVALID_VALUE)?;
            rows("B")
                .find(|r| r[1] == family && normalize(r[3]) == component)
                .and_then(|r| u32::from_str_radix(r[2], 16).ok())
                .ok_or(INVALID_VALUE)?
        };
        found = true;
        bits |= part;
    }
    if !found {
        return Err(INVALID_VALUE);
    }
    Ok(i32::from_be_bytes(bits.to_be_bytes()))
}

fn signed_long(value: &str) -> Result<i64> {
    if value.starts_with("0x") {
        return Ok(i64::from_be_bytes(hex_number(value, 16)?.to_be_bytes()));
    }
    value.parse().map_err(|_| INVALID_VALUE)
}

fn xml_space(c: char) -> bool {
    matches!(c, ' ' | '\r' | '\n' | '\t')
}
fn xml_char(c: char) -> bool {
    matches!(c, '\t' | '\n' | '\r' | '\u{20}'..='\u{D7FF}' | '\u{E000}'..='\u{FFFD}' | '\u{10000}'..='\u{10FFFF}')
}

fn attribute(value: &str) -> Result<Zeroizing<String>> {
    let mut output = Zeroizing::new(String::new());
    output
        .try_reserve_exact(value.len())
        .map_err(|_| ALLOCATION)?;
    let mut chars = value.chars();
    while let Some(c) = chars.next() {
        match c {
            '<' => return Err(INVALID_DOCUMENT),
            '\r' => {
                if chars.as_str().starts_with('\n') {
                    chars.next();
                }
                output.push(' ');
            }
            '\t' | '\n' => output.push(' '),
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
                output.push(c);
            }
            _ => output.push(c),
        }
    }
    Ok(output)
}

fn octets(text: &str, big: bool) -> Result<Vec<u8>> {
    if !text.len().is_multiple_of(2)
        || (big && (text.is_empty() || !text.len().is_multiple_of(16)))
        || !text.bytes().all(|c| c.is_ascii_hexdigit())
    {
        return Err(INVALID_VALUE);
    }
    let mut bytes = Zeroizing::new(Vec::new());
    bytes
        .try_reserve_exact(text.len() / 2)
        .map_err(|_| ALLOCATION)?;
    for pair in text.as_bytes().as_chunks::<2>().0 {
        let digit = |c: u8| {
            if c.is_ascii_digit() {
                c - b'0'
            } else {
                c.to_ascii_uppercase() - b'A' + 10
            }
        };
        bytes.push(digit(pair[0]) * 16 + digit(pair[1]));
    }
    Ok(std::mem::take(&mut *bytes))
}

fn scalar(raw: u32, ty: &str, value: &str) -> Result<Value> {
    let collapsed = value.trim_matches(xml_space);
    Ok(match ty {
        "Integer" => Value::integer(integer(raw, collapsed)?),
        "LongInteger" => Value::long_integer(signed_long(collapsed)?),
        "BigInteger" => Value::big_integer(octets(collapsed, true)?),
        "Enumeration" => Value::enumeration(enum_value(raw, value)?),
        "Boolean" => Value::boolean(match collapsed {
            "true" | "1" => true,
            "false" | "0" => false,
            _ => return Err(INVALID_VALUE),
        }),
        "TextString" => {
            let mut text = Zeroizing::new(String::new());
            text.try_reserve_exact(value.len())
                .map_err(|_| ALLOCATION)?;
            text.push_str(value);
            Value::text_string(std::mem::take(&mut *text))
        }
        "ByteString" => Value::byte_string(octets(collapsed, false)?),
        "DateTime" => Value::date_time(parse_datetime(collapsed)?),
        "Interval" => Value::interval(collapsed.parse().map_err(|_| INVALID_VALUE)?),
        "DateTimeExtended" => {
            Value::date_time_extended(collapsed.parse().map_err(|_| INVALID_VALUE)?)
        }
        _ => return Err(INVALID_VALUE),
    })
}

// Gregorian conversion uses Euclidean 400-year eras. i128 arithmetic covers
// every i64 epoch second without narrowing its XML year (§5.4.1.6.11).
fn days_from_civil(year: i128, month: i128, day: i128) -> i128 {
    let y = year - i128::from(month <= 2);
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let m = month + if month > 2 { -3 } else { 9 };
    era * 146_097 + yoe * 365 + yoe / 4 - yoe / 100 + (153 * m + 2) / 5 + day - 1 - 719_468
}

fn civil_from_days(days: i128) -> (i128, i128, i128) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    (y + i128::from(m <= 2), m, d)
}

fn two_digits(value: &str) -> Result<i128> {
    if value.len() != 2 || !value.bytes().all(|c| c.is_ascii_digit()) {
        return Err(INVALID_VALUE);
    }
    value.parse().map_err(|_| INVALID_VALUE)
}

fn parse_datetime(text: &str) -> Result<i64> {
    let (date, time) = text.split_once('T').ok_or(INVALID_VALUE)?;
    let (year_month, day) = date.rsplit_once('-').ok_or(INVALID_VALUE)?;
    let (year, month) = year_month.rsplit_once('-').ok_or(INVALID_VALUE)?;
    let unsigned_year = year.strip_prefix('-').unwrap_or(year);
    if unsigned_year.len() < 4
        || !unsigned_year.bytes().all(|c| c.is_ascii_digit())
        || unsigned_year.len() > 18
        || (unsigned_year.len() > 4 && unsigned_year.starts_with('0'))
    {
        return Err(INVALID_VALUE);
    }
    let year: i128 = year.parse().map_err(|_| INVALID_VALUE)?;
    if year == 0 {
        return Err(INVALID_VALUE);
    }
    let astronomical = if year < 0 { year + 1 } else { year };
    let month = two_digits(month)?;
    let day = two_digits(day)?;
    let leap = astronomical.rem_euclid(4) == 0
        && (astronomical.rem_euclid(100) != 0 || astronomical.rem_euclid(400) == 0);
    let max = match month {
        2 => {
            if leap {
                29
            } else {
                28
            }
        }
        4 | 6 | 9 | 11 => 30,
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        _ => return Err(INVALID_VALUE),
    };
    if day < 1 || day > max {
        return Err(INVALID_VALUE);
    }
    let (clock, zone) = if let Some(clock) = time.strip_suffix('Z') {
        (clock, 0)
    } else {
        let index = time.find(['+', '-']).ok_or(INVALID_VALUE)?;
        let (h, m) = time[index + 1..].split_once(':').ok_or(INVALID_VALUE)?;
        let h = two_digits(h)?;
        let m = two_digits(m)?;
        if h > 14 || m > 59 || (h == 14 && m != 0) {
            return Err(INVALID_VALUE);
        }
        (
            &time[..index],
            (h * 3600 + m * 60)
                * if time.as_bytes()[index] == b'-' {
                    -1
                } else {
                    1
                },
        )
    };
    let (h, rest) = clock.split_once(':').ok_or(INVALID_VALUE)?;
    let (m, second) = rest.split_once(':').ok_or(INVALID_VALUE)?;
    let second = if let Some((s, fraction)) = second.split_once('.') {
        if fraction.is_empty() || !fraction.bytes().all(|c| c == b'0') {
            return Err(INVALID_VALUE);
        }
        s
    } else {
        second
    };
    let h = two_digits(h)?;
    let m = two_digits(m)?;
    let s = two_digits(second)?;
    if h > 24 || m > 59 || s > 59 || (h == 24 && (m != 0 || s != 0)) {
        return Err(INVALID_VALUE);
    }
    let seconds = days_from_civil(astronomical, month, day) * 86_400 + h * 3600 + m * 60 + s - zone;
    i64::try_from(seconds).map_err(|_| INVALID_VALUE)
}

fn datetime(seconds: i64) -> Zeroizing<String> {
    let seconds = i128::from(seconds);
    let (year, month, day) = civil_from_days(seconds.div_euclid(86_400));
    let clock = seconds.rem_euclid(86_400);
    let h = clock / 3600;
    let m = clock / 60 % 60;
    let s = clock % 60;
    if year <= 0 {
        Zeroizing::new(format!(
            "-{:04}-{month:02}-{day:02}T{h:02}:{m:02}:{s:02}Z",
            1 - year
        ))
    } else {
        Zeroizing::new(format!(
            "{year:04}-{month:02}-{day:02}T{h:02}:{m:02}:{s:02}Z"
        ))
    }
}

struct Output {
    bytes: Zeroizing<Vec<u8>>,
    cap: usize,
}
impl Output {
    fn push(&mut self, text: &str) -> Result<()> {
        let required = self
            .bytes
            .len()
            .checked_add(text.len())
            .filter(|n| *n <= self.cap)
            .ok_or(LIMIT)?;
        if required > self.bytes.capacity() {
            let capacity = self
                .bytes
                .capacity()
                .saturating_mul(2)
                .max(required)
                .min(self.cap);
            let additional = capacity - self.bytes.len();
            self.bytes
                .try_reserve_exact(additional)
                .map_err(|_| ALLOCATION)?;
        }
        self.bytes.extend_from_slice(text.as_bytes());
        Ok(())
    }

    fn escaped(&mut self, text: &str) -> Result<()> {
        for c in text.chars() {
            match c {
                '&' => self.push("&amp;")?,
                '<' => self.push("&lt;")?,
                '>' => self.push("&gt;")?,
                '\'' => self.push("&apos;")?,
                '"' => self.push("&quot;")?,
                '\t' => self.push("&#x9;")?,
                '\r' => self.push("&#xD;")?,
                '\n' => self.push("&#xA;")?,
                _ if !xml_char(c) => return Err(INVALID_VALUE),
                _ => self.push(c.encode_utf8(&mut [0; 4]))?,
            }
        }
        Ok(())
    }

    fn octets(&mut self, bytes: &[u8]) -> Result<()> {
        const HEX: &[u8; 16] = b"0123456789ABCDEF";
        for &byte in bytes {
            self.push(
                std::str::from_utf8(&[HEX[usize::from(byte >> 4)], HEX[usize::from(byte & 15)]])
                    .map_err(|_| INVALID_VALUE)?,
            )?;
        }
        Ok(())
    }
}

fn emit_mask(output: &mut Output, raw: u32, value: i32) -> Result<()> {
    let Some(family) = family(raw, "M") else {
        return output.push(&Zeroizing::new(value.to_string()));
    };
    let mut bits = u32::from_be_bytes(value.to_be_bytes());
    let mut first = true;
    for row in rows("B").filter(|r| r[1] == family) {
        let flag = u32::from_str_radix(row[2], 16).map_err(|_| INVALID_VALUE)?;
        if flag != 0 && bits & flag == flag {
            if !first {
                output.push(" ")?;
            }
            output.push(&normalize(row[3]))?;
            first = false;
            bits &= !flag;
        }
    }
    if bits != 0 || first {
        if !first {
            output.push(" ")?;
        }
        output.push(&Zeroizing::new(format!("0x{bits:08X}")))?;
    }
    Ok(())
}

fn emit(
    output: &mut Output,
    item: &Item,
    limits: &CodecLimits,
    depth: usize,
    count: &mut usize,
) -> Result<()> {
    if *count >= limits.max_elements() {
        return Err(LIMIT);
    }
    *count += 1;
    let raw = item.tag().raw();
    let name = tag_name(raw);
    let element = name.as_deref().unwrap_or("TTLV");
    output.push("<")?;
    output.push(element)?;
    if name.is_none() {
        output.push(&format!(" tag=\"0x{raw:06X}\""))?;
    }
    item.with_value(|view| {
        if let ValueView::Structure(s) = view {
            if depth >= limits.max_structure_depth() {
                return Err(LIMIT);
            }
            output.push(">")?;
            for child in s.children() {
                emit(output, child, limits, depth + 1, count)?;
            }
            output.push("</")?;
            output.push(element)?;
            return output.push(">");
        }
        let ty = match view {
            ValueView::Integer(_) => "Integer",
            ValueView::LongInteger(_) => "LongInteger",
            ValueView::BigInteger(_) => "BigInteger",
            ValueView::Enumeration(_) => "Enumeration",
            ValueView::Boolean(_) => "Boolean",
            ValueView::TextString(_) => "TextString",
            ValueView::ByteString(_) => "ByteString",
            ValueView::DateTime(_) => "DateTime",
            ValueView::Interval(_) => "Interval",
            ValueView::DateTimeExtended(_) => "DateTimeExtended",
            _ => return Err(INVALID_VALUE),
        };
        output.push(" type=\"")?;
        output.push(ty)?;
        output.push("\" value=\"")?;
        match view {
            ValueView::Integer(v) => emit_mask(output, raw, *v)?,
            ValueView::LongInteger(v) | ValueView::DateTimeExtended(v) => {
                output.push(&Zeroizing::new(v.to_string()))?;
            }
            ValueView::BigInteger(v) => {
                if v.is_empty() || !v.len().is_multiple_of(8) {
                    return Err(INVALID_VALUE);
                }
                output.octets(v)?;
            }
            ValueView::Enumeration(v) => {
                if let Some(name) = enum_name(raw, *v) {
                    output.push(&name)?;
                } else {
                    output.push(&Zeroizing::new(format!("0x{v:08X}")))?;
                }
            }
            ValueView::Boolean(v) => output.push(if *v { "true" } else { "false" })?,
            ValueView::TextString(v) => output.escaped(v)?,
            ValueView::ByteString(v) => output.octets(v)?,
            ValueView::DateTime(v) => output.push(&datetime(*v))?,
            ValueView::Interval(v) => output.push(&Zeroizing::new(v.to_string()))?,
            _ => return Err(INVALID_VALUE),
        }
        output.push("\"/>")
    })
}

pub(crate) fn encode(item: &Item, limits: &CodecLimits) -> Result<Zeroizing<Vec<u8>>> {
    let mut output = Output {
        bytes: Zeroizing::new(Vec::new()),
        cap: limits.max_message_bytes(),
    };
    emit(&mut output, item, limits, 0, &mut 0)?;
    Ok(output.bytes)
}

struct Frame {
    name: Zeroizing<String>,
    tag: kmipkit_ttlv::Tag,
    value: Option<Value>,
    children: Structure,
    namespaces: Vec<(Zeroizing<String>, Zeroizing<String>)>,
}

fn qname(name: &str) -> Result<(&str, &str)> {
    fn valid(name: &str) -> bool {
        let mut chars = name.chars();
        chars.next().is_some_and(|c| c == '_' || c.is_alphabetic())
            && chars.all(|c| c == '_' || c == '-' || c == '.' || c.is_alphanumeric())
    }
    let (prefix, local) = name.split_once(':').unwrap_or(("", name));
    if (!prefix.is_empty() && !valid(prefix)) || !valid(local) || local.contains(':') {
        return Err(INVALID_DOCUMENT);
    }
    Ok((prefix, local))
}

fn start(event: &BytesStart<'_>, frames: &[Frame]) -> Result<Frame> {
    let name = event.name();
    let name = name.as_ref();
    let (prefix, local) = qname(name)?;
    let mut attributes: [Option<Zeroizing<String>>; 4] = [None, None, None, None];
    let mut namespaces = Vec::new();
    for attr in event.attributes() {
        let attr = attr.map_err(|_| INVALID_DOCUMENT)?;
        let key = attr.key.as_ref();
        if key == "xmlns" || key.starts_with("xmlns:") {
            let namespace_prefix = if key == "xmlns" { "" } else { &key[6..] };
            if !namespace_prefix.is_empty() {
                qname(namespace_prefix)?;
            }
            let uri = attribute(&attr.value)?;
            if namespace_prefix == "xmlns"
                || uri.as_str() == "http://www.w3.org/2000/xmlns/"
                || (namespace_prefix == "xml"
                    && uri.as_str() != "http://www.w3.org/XML/1998/namespace")
                || (namespace_prefix != "xml"
                    && uri.as_str() == "http://www.w3.org/XML/1998/namespace")
                || (!namespace_prefix.is_empty() && uri.is_empty())
            {
                return Err(INVALID_DOCUMENT);
            }
            namespaces.try_reserve(1).map_err(|_| ALLOCATION)?;
            namespaces.push((attribute(namespace_prefix)?, uri));
        } else {
            let index = match key {
                "tag" => 0,
                "name" => 1,
                "type" => 2,
                "value" => 3,
                _ => return Err(INVALID_DOCUMENT),
            };
            if attributes[index].is_some() {
                return Err(INVALID_DOCUMENT);
            }
            attributes[index] = Some(attribute(&attr.value)?);
        }
    }
    let namespace = namespaces
        .iter()
        .rev()
        .chain(
            frames
                .iter()
                .rev()
                .flat_map(|frame| frame.namespaces.iter().rev()),
        )
        .find(|(p, _)| p.as_str() == prefix)
        .map(|(_, uri)| uri.as_str());
    if prefix == "xml"
        || match namespace {
            Some("") | None => !prefix.is_empty(),
            Some(uri) => uri != NAMESPACE,
        }
    {
        return Err(INVALID_DOCUMENT);
    }
    let raw = if local == "TTLV" {
        tag_value(
            attributes[0]
                .as_deref()
                .map(String::as_str)
                .ok_or(INVALID_DOCUMENT)?,
        )?
    } else {
        if attributes[0].is_some() || attributes[1].is_some() {
            return Err(INVALID_DOCUMENT);
        }
        tag_value(local)?
    };
    let tag = RawTag::new(raw)
        .and_then(|t| t.try_checked())
        .map_err(|_| INVALID_VALUE)?;
    let ty = attributes[2].as_deref().map_or("Structure", String::as_str);
    let value = if ty == "Structure" {
        if attributes[3].is_some() {
            return Err(INVALID_DOCUMENT);
        }
        None
    } else {
        Some(scalar(
            raw,
            ty,
            attributes[3]
                .as_deref()
                .map(String::as_str)
                .ok_or(INVALID_DOCUMENT)?,
        )?)
    };
    Ok(Frame {
        name: attribute(name)?,
        tag,
        value,
        children: Structure::new(),
        namespaces,
    })
}

fn finish(frame: Frame) -> Result<Item> {
    Item::new(
        frame.tag,
        frame
            .value
            .unwrap_or_else(|| Value::structure(frame.children)),
    )
    .map_err(|_| INVALID_VALUE)
}

fn attach(item: Item, frames: &mut [Frame], root: &mut Option<Item>) -> Result<()> {
    if let Some(parent) = frames.last_mut() {
        parent.children.try_push(item).map_err(|_| INVALID_VALUE)
    } else if root.is_some() {
        Err(INVALID_DOCUMENT)
    } else {
        *root = Some(item);
        Ok(())
    }
}

fn preflight(event: &BytesStart<'_>, depth: usize, limits: &CodecLimits) -> Result<()> {
    if depth < limits.max_structure_depth() {
        return Ok(());
    }
    // Only type metadata is needed to decide whether this element adds a
    // Structure. Never decode its payload or namespace attributes first.
    for attr in event.attributes().with_checks(false) {
        let attr = attr.map_err(|_| INVALID_DOCUMENT)?;
        if attr.key.as_ref() == "type" {
            return if attribute(&attr.value)?.as_str() == "Structure" {
                Err(LIMIT)
            } else {
                Ok(())
            };
        }
    }
    Err(LIMIT)
}

pub(crate) fn decode(bytes: &[u8], limits: &CodecLimits) -> Result<Item> {
    if bytes.len() > limits.max_message_bytes() {
        return Err(LIMIT);
    }
    let text = std::str::from_utf8(bytes).map_err(|_| INVALID_DOCUMENT)?;
    if !text.chars().all(xml_char) {
        return Err(INVALID_DOCUMENT);
    }
    let mut reader = Reader::from_str(text);
    reader.config_mut().check_comments = true;
    // Compare our own zeroizing names rather than copying to a parser stack.
    reader.config_mut().check_end_names = false;
    let mut frames: Vec<Frame> = Vec::new();
    let mut root = None;
    let mut count = 0;
    let mut declaration = false;
    loop {
        let before = reader.buffer_position();
        let event = reader.read_event().map_err(|_| INVALID_DOCUMENT)?;
        let empty = matches!(event, Event::Empty(_));
        match event {
            Event::Start(event) | Event::Empty(event) => {
                if count >= limits.max_elements() {
                    return Err(LIMIT);
                }
                if frames.last().is_some_and(|f| f.value.is_some())
                    || (frames.is_empty() && root.is_some())
                {
                    return Err(INVALID_DOCUMENT);
                }
                // Every ancestor is a Structure; preflight before attributes
                // or model payloads are allocated for this item.
                if frames.len() > limits.max_structure_depth() {
                    return Err(LIMIT);
                }
                count += 1;
                preflight(&event, frames.len(), limits)?;
                let frame = start(&event, &frames)?;
                if frame.value.is_none() && frames.len() >= limits.max_structure_depth() {
                    return Err(LIMIT);
                }
                if empty {
                    attach(finish(frame)?, &mut frames, &mut root)?;
                } else {
                    frames.try_reserve(1).map_err(|_| ALLOCATION)?;
                    frames.push(frame);
                }
            }
            Event::End(event) => {
                let frame = frames.pop().ok_or(INVALID_DOCUMENT)?;
                if event.name().as_ref() != frame.name.as_str() {
                    return Err(INVALID_DOCUMENT);
                }
                attach(finish(frame)?, &mut frames, &mut root)?;
            }
            Event::Text(event) if event.as_ref().chars().all(xml_space) => {}
            Event::CData(event) if !frames.is_empty() && event.as_ref().chars().all(xml_space) => {}
            Event::Comment(_) => {}
            Event::PI(event) => {
                if event.target().eq_ignore_ascii_case("xml") {
                    return Err(INVALID_DOCUMENT);
                }
            }
            Event::Decl(event) => {
                if declaration
                    || before != 0
                    || count != 0
                    || root.is_some()
                    || event.version().map_err(|_| INVALID_DOCUMENT)?.as_ref() != "1.0"
                {
                    return Err(INVALID_DOCUMENT);
                }
                if let Some(encoding) = event.encoding()
                    && !encoding
                        .map_err(|_| INVALID_DOCUMENT)?
                        .eq_ignore_ascii_case("UTF-8")
                {
                    return Err(INVALID_DOCUMENT);
                }
                declaration = true;
            }
            Event::Eof => {
                if !frames.is_empty() {
                    return Err(INVALID_DOCUMENT);
                }
                return root.ok_or(INVALID_DOCUMENT);
            }
            _ => return Err(INVALID_DOCUMENT),
        }
    }
}

#[cfg(test)]
#[path = "../tests/unit/xml_codec_tests.rs"]
mod tests;
