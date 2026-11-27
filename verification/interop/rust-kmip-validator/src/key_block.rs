//! Key Block shape verification shared by generic and typed Get routes.

use crate::driver::{item, structure};
use crate::generic::node;
use kmipkit::ttlv::{Item, Structure, StructureView, Value, ValueView};

#[derive(PartialEq, Eq)]
pub(crate) enum KeyBlockShape {
    Transparent(u32),
    Wrapped(String),
}

pub(crate) fn wrapping_fields(identifier: &str) -> Result<Vec<Item>, &'static str> {
    Ok(vec![
        item(0x0042_009e, Value::enumeration(1))?,
        node(
            0x0042_0036,
            vec![
                item(0x0042_0094, Value::text_string(identifier.to_owned()))?,
                node(
                    0x0042_002b,
                    vec![item(0x0042_0011, Value::enumeration(12))?],
                )?,
            ],
        )?,
        item(0x0042_00a3, Value::enumeration(1))?,
    ])
}

pub(crate) fn wrapping_specification(identifier: &str) -> Result<Structure, &'static str> {
    structure(0x0042_0047, wrapping_fields(identifier)?)
}

fn field<'a>(view: &StructureView<'a>, tag: u32) -> Option<&'a Item> {
    let mut hits = view
        .children()
        .iter()
        .filter(|field| field.tag().raw() == tag);
    let first = hits.next()?;
    hits.next().is_none().then_some(first)
}

pub(crate) fn key_block<'a>(
    payload: &StructureView<'a>,
    object_type: u32,
) -> Option<StructureView<'a>> {
    let tag = match object_type {
        2 => 0x0042_008f,
        3 => 0x0042_006d,
        4 => 0x0042_0064,
        _ => return None,
    };
    let key = field(payload, tag)?.as_structure()?;
    field(&key, 0x0042_0040)?.as_structure()
}

pub(crate) fn asymmetric_format(requires: &serde_json::Value) -> Option<u32> {
    match requires["key_material_format"].as_str() {
        Some("10") => Some(10),
        Some("11") => Some(11),
        Some("20") => Some(20),
        Some("21") => Some(21),
        _ => None,
    }
}

pub(crate) fn pair_attributes(format: u32) -> Result<[Vec<Item>; 3], &'static str> {
    let ec = matches!(format, 20 | 21);
    let mut common = vec![
        item(0x0042_0028, Value::enumeration(if ec { 26 } else { 4 }))?,
        item(0x0042_002a, Value::integer(if ec { 256 } else { 2048 }))?,
    ];
    if ec {
        common.push(node(
            0x0042_0029,
            vec![item(0x0042_0075, Value::enumeration(7))?],
        )?);
    }
    let own = |mask| -> Result<Vec<Item>, &'static str> {
        Ok(vec![
            item(0x0042_002c, Value::integer(mask))?,
            item(0x0042_0122, Value::boolean(true))?,
        ])
    };
    Ok([common, own(1)?, own(2)?])
}

pub(crate) fn shape(block: &StructureView<'_>) -> Option<KeyBlockShape> {
    let enumeration = |view: &StructureView<'_>, tag| {
        field(view, tag)?.with_value(|value| match value {
            ValueView::Enumeration(number) => Some(*number),
            _ => None,
        })
    };
    let format = enumeration(block, 0x0042_0042)?;
    let value = field(block, 0x0042_0045)?;
    let wrapping_fields = block
        .children()
        .iter()
        .filter(|field| field.tag().raw() == 0x0042_0046)
        .count();
    let algorithm = enumeration(block, 0x0042_0028)?;
    let length = field(block, 0x0042_002a)?.with_value(|value| match value {
        ValueView::Integer(length) => Some(*length),
        _ => None,
    })?;
    if wrapping_fields > 0 {
        if wrapping_fields != 1 || algorithm != 3 || length != 256 {
            return None;
        }
        let wrapping = field(block, 0x0042_0046)?.as_structure()?;
        if !value
            .with_value(|value| matches!(value, ValueView::ByteString(bytes) if bytes.len() == 40))
            || enumeration(&wrapping, 0x0042_009e) != Some(1)
            || enumeration(&wrapping, 0x0042_00a3) != Some(1)
        {
            return None;
        }
        let encryption_key = field(&wrapping, 0x0042_0036)?.as_structure()?;
        return field(&encryption_key, 0x0042_0094)?.with_value(|value| match value {
            ValueView::TextString(identifier) => {
                Some(KeyBlockShape::Wrapped(identifier.to_owned()))
            }
            _ => None,
        });
    }
    let value = value.as_structure()?;
    let material = field(&value, 0x0042_0043)?.as_structure()?;
    let bytes = |tag, predicate: fn(usize) -> bool| {
        field(&material, tag).is_some_and(|value| {
            value.with_value(
                |value| matches!(value, ValueView::ByteString(bytes) if predicate(bytes.len())),
            )
        })
    };
    let integer = |tag, predicate: fn(usize) -> bool| {
        field(&material, tag).is_some_and(|value| {
            value.with_value(
                |value| matches!(value, ValueView::BigInteger(bytes) if predicate(bytes.len())),
            )
        })
    };
    let matches = match (format, algorithm, length) {
        (7, 3, 256) => bytes(0x0042_003f, |size| size == 32),
        (10 | 11, 4, 2048) => {
            let exponent = if format == 10 {
                0x0042_0063
            } else {
                0x0042_006c
            };
            integer(0x0042_0052, |size| matches!(size, 256 | 264))
                && integer(exponent, |size| size > 0)
        }
        (20 | 21, 6 | 14 | 26, 256) => {
            enumeration(&material, 0x0042_0075) == Some(7)
                && if format == 20 {
                    integer(0x0042_002e, |size| (1..=40).contains(&size))
                } else {
                    bytes(0x0042_0072, |size| size == 65)
                }
        }
        _ => false,
    };
    matches.then_some(KeyBlockShape::Transparent(format))
}
