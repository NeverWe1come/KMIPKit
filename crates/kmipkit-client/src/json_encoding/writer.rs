//! Bounded two-pass JSON emission without wire-buffer growth.
use super::names::{NameKind, family, mask_components, normalize, tag_name, value_name};
use super::{WireCodecError, WireCodecErrorKind, error};
use kmipkit_ttlv::{Item, ValueView, codec::CodecLimits};
use zeroize::Zeroizing;

pub(super) struct Writer<'l> {
    pub(super) output: Option<Zeroizing<Vec<u8>>>,
    pub(super) length: usize,
    pub(super) elements: usize,
    pub(super) limits: &'l CodecLimits,
}
impl Writer<'_> {
    fn put(&mut self, bytes: &[u8]) -> Result<(), WireCodecError> {
        self.length = self
            .length
            .checked_add(bytes.len())
            .ok_or_else(|| error(WireCodecErrorKind::LimitExceeded))?;
        if self.length > self.limits.max_message_bytes() {
            return Err(error(WireCodecErrorKind::LimitExceeded));
        }
        if let Some(out) = &mut self.output {
            out.extend_from_slice(bytes);
        }
        Ok(())
    }
    fn quoted(&mut self, value: &str) -> Result<(), WireCodecError> {
        self.put(b"\"")?;
        for ch in value.chars() {
            match ch {
                '"' => self.put(b"\\\"")?,
                '\\' => self.put(b"\\\\")?,
                '\n' => self.put(b"\\n")?,
                '\r' => self.put(b"\\r")?,
                '\t' => self.put(b"\\t")?,
                c if c < ' ' => {
                    self.put(b"\\u00")?;
                    self.hex(&[u8::try_from(u32::from(c))
                        .map_err(|_| error(WireCodecErrorKind::InvalidValue))?])?;
                }
                c => {
                    let mut encoded = [0; 4];
                    self.put(c.encode_utf8(&mut encoded).as_bytes())?;
                }
            }
        }
        self.put(b"\"")
    }
    fn hex(&mut self, bytes: &[u8]) -> Result<(), WireCodecError> {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        for b in bytes {
            self.put(&[HEX[usize::from(b >> 4)], HEX[usize::from(b & 15)]])?;
        }
        Ok(())
    }
    fn hex_value(&mut self, bytes: &[u8], prefix: bool) -> Result<(), WireCodecError> {
        self.put(b"\"")?;
        if prefix {
            self.put(b"0x")?;
        }
        self.hex(bytes)?;
        self.put(b"\"")
    }
    fn number(&mut self, number: impl std::fmt::Display) -> Result<(), WireCodecError> {
        let text = Zeroizing::new(number.to_string());
        self.put(text.as_bytes())
    }
    pub(super) fn item(&mut self, item: &Item, depth: usize) -> Result<(), WireCodecError> {
        self.elements = self
            .elements
            .checked_add(1)
            .ok_or_else(|| error(WireCodecErrorKind::LimitExceeded))?;
        if self.elements > self.limits.max_elements() {
            return Err(error(WireCodecErrorKind::LimitExceeded));
        }
        self.put(b"{\"tag\":")?;
        let tag = item.tag().raw();
        if let Some(name) = tag_name(tag) {
            self.quoted(&normalize(name))?;
        } else {
            self.hex_value(&tag.to_be_bytes()[1..], true)?;
        }
        self.put(b",\"type\":")?;
        item.with_value(|v| -> Result<(), WireCodecError> {
            let ty = match &v {
                ValueView::Structure(_) => "Structure",
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
                _ => return Err(error(WireCodecErrorKind::InvalidValue)),
            };
            self.quoted(ty)?;
            self.put(b",\"value\":")?;
            match v {
                ValueView::Structure(s) => {
                    if depth >= self.limits.max_structure_depth() {
                        return Err(error(WireCodecErrorKind::LimitExceeded));
                    }
                    self.put(b"[")?;
                    for (i, child) in s.children().iter().enumerate() {
                        if i > 0 {
                            self.put(b",")?;
                        }
                        self.item(child, depth + 1)?;
                    }
                    self.put(b"]")?;
                }
                ValueView::Integer(n) => {
                    if let Some(f) = family(tag, NameKind::Mask) {
                        self.mask(f, n.cast_unsigned())?;
                    } else {
                        self.number(n)?;
                    }
                }
                ValueView::LongInteger(n) | ValueView::DateTimeExtended(n) => {
                    if n.unsigned_abs() < (1u64 << 52) {
                        self.number(n)?;
                    } else {
                        self.hex_value(&n.to_be_bytes(), true)?;
                    }
                }
                ValueView::BigInteger(bytes) => {
                    if bytes.is_empty() || !bytes.len().is_multiple_of(8) {
                        return Err(error(WireCodecErrorKind::InvalidValue));
                    }
                    self.hex_value(bytes, true)?;
                }
                ValueView::Enumeration(n) => {
                    if let Some(name) = family(tag, NameKind::Enumeration)
                        .and_then(|f| value_name(f, *n, NameKind::Enumeration))
                    {
                        self.quoted(&normalize(name))?;
                    } else {
                        self.hex_value(&n.to_be_bytes(), true)?;
                    }
                }
                ValueView::Boolean(b) => self.put(if *b { b"true" } else { b"false" })?,
                ValueView::TextString(s) => self.quoted(s)?,
                ValueView::ByteString(b) => self.hex_value(b, false)?,
                ValueView::DateTime(n) => self.hex_value(&n.to_be_bytes(), true)?,
                ValueView::Interval(n) => self.number(n)?,
                _ => return Err(error(WireCodecErrorKind::InvalidValue)),
            }
            Ok(())
        })?;
        self.put(b"}")
    }
    fn mask(&mut self, family: &str, mut bits: u32) -> Result<(), WireCodecError> {
        self.put(b"\"")?;
        let mut any = false;
        for (raw, name) in mask_components(family) {
            if raw != 0 && bits & raw == raw {
                if any {
                    self.put(b"|")?;
                }
                self.put(normalize(name).as_bytes())?;
                any = true;
                bits &= !raw;
            }
        }
        if bits != 0 || !any {
            if any {
                self.put(b"|")?;
            }
            self.put(b"0x")?;
            self.hex(&bits.to_be_bytes())?;
        }
        self.put(b"\"")
    }
}
