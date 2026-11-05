//! Bounded zeroizing XML output; Profiles v2.1 §5.4.1.
use super::characters::xml_char;
use super::names::{enum_name, family, normalize, rows, tag_name};
use super::values::datetime;
use super::{ALLOCATION, INVALID_VALUE, LIMIT, Result};
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::{Item, ValueView};
use zeroize::Zeroizing;

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
