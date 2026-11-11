//! Consumer-side TTLV framing through public scoped views; bytes remain ephemeral.
use crate::models::Result;
use kmipkit::ttlv::{Item, ValueView};
use serde_json::{Value, json};
use zeroize::Zeroizing;

pub(crate) fn encode(item: &Item) -> Result<Zeroizing<Vec<u8>>> {
    let (kind, body) = item.with_value(|value| -> Result<_> {
        let (kind, body) = match value {
            ValueView::Structure(value) => {
                let mut body = Zeroizing::new(Vec::new());
                for child in value.children() {
                    body.extend_from_slice(&encode(child)?);
                }
                (1, body)
            }
            ValueView::Integer(v) => (2, Zeroizing::new(v.to_be_bytes().to_vec())),
            ValueView::LongInteger(v) => (3, Zeroizing::new(v.to_be_bytes().to_vec())),
            ValueView::BigInteger(v) => {
                let sign = if v.first().is_some_and(|b| b & 0x80 != 0) {
                    0xff
                } else {
                    0
                };
                let mut body = Zeroizing::new(vec![sign; (8 - v.len() % 8) % 8]);
                body.extend_from_slice(v);
                (4, body)
            }
            ValueView::Enumeration(v) => (5, Zeroizing::new(v.to_be_bytes().to_vec())),
            ValueView::Boolean(v) => (6, Zeroizing::new(u64::from(*v).to_be_bytes().to_vec())),
            ValueView::TextString(v) => (7, Zeroizing::new(v.as_bytes().to_vec())),
            ValueView::ByteString(v) => (8, Zeroizing::new(v.to_vec())),
            ValueView::DateTime(v) => (9, Zeroizing::new(v.to_be_bytes().to_vec())),
            ValueView::Interval(v) => (10, Zeroizing::new(v.to_be_bytes().to_vec())),
            ValueView::DateTimeExtended(v) => (11, Zeroizing::new(v.to_be_bytes().to_vec())),
            _ => return Err("unknown local value type"),
        };
        Ok((kind, body))
    })?;
    let mut bytes = Zeroizing::new(item.tag().raw().to_be_bytes()[1..].to_vec());
    bytes.push(kind);
    bytes.extend_from_slice(
        &u32::try_from(body.len())
            .map_err(|_| "local value too large")?
            .to_be_bytes(),
    );
    bytes.extend_from_slice(&body);
    let padded = bytes.len().next_multiple_of(8);
    bytes.resize(padded, 0);
    Ok(bytes)
}

pub(crate) fn normalized(item: &Item) -> Result<Value> {
    item.with_value(|value| {
        let (kind, safe) = match value {
            ValueView::Structure(value) => (
                1,
                value
                    .children()
                    .iter()
                    .map(normalized)
                    .collect::<Result<Vec<_>>>()
                    .map(Value::Array)?,
            ),
            ValueView::Integer(v) => (2, json!(v)),
            ValueView::LongInteger(v) => (3, json!(v)),
            ValueView::BigInteger(_) => (4, json!("<redacted>")),
            ValueView::Enumeration(v) => (5, json!(v)),
            ValueView::Boolean(v) => (6, json!(u8::from(*v))),
            ValueView::TextString(_) => (7, json!("<redacted>")),
            ValueView::ByteString(_) => (8, json!("<redacted>")),
            ValueView::DateTime(v) => (9, json!(v)),
            ValueView::Interval(v) => (10, json!(v)),
            ValueView::DateTimeExtended(v) => (11, json!(v)),
            _ => return Err("unknown local value type"),
        };
        let safe = if [0x0042_0025, 0x0042_0043, 0x0042_000b].contains(&item.tag().raw()) {
            json!("<redacted>")
        } else {
            safe
        };
        Ok(json!([item.tag().raw(), kind, safe]))
    })
}

pub(crate) fn roundtrip(item: &Item) -> Result<String> {
    let bytes = encode(item)?;
    let decoded = kmipkit::ttlv::codec::decode(&bytes).map_err(|_| "local decode rejected")?;
    let reencoded = encode(&decoded)?;
    if bytes != reencoded || normalized(item)? != normalized(&decoded)? {
        return Err("local roundtrip value mismatch");
    }
    let normalized = json!([normalized(&decoded)?]);
    Ok(crate::models::hash(
        &serde_json::to_vec(&normalized).map_err(|_| "local normalization failed")?,
    ))
}
