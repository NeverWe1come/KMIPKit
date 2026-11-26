//! Key Block shape verification shared by generic and typed Get routes.

use crate::driver::{item, structure};
use crate::generic::node;
use kmipkit::ttlv::{Item, Structure, StructureView, Value, ValueView};

#[derive(PartialEq, Eq)]
pub(crate) enum KeyBlockShape {
    Transparent,
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

pub(crate) fn symmetric_key_block<'a>(payload: &StructureView<'a>) -> Option<StructureView<'a>> {
    let key = field(payload, 0x0042_008f)?.as_structure()?;
    field(&key, 0x0042_0040)?.as_structure()
}

pub(crate) fn shape(block: &StructureView<'_>) -> Option<KeyBlockShape> {
    let algorithm_and_length = field(block, 0x0042_0028)?
        .with_value(|value| matches!(value, ValueView::Enumeration(3)))
        && field(block, 0x0042_002a)?.with_value(|value| matches!(value, ValueView::Integer(256)));
    if !algorithm_and_length {
        return None;
    }
    let value = field(block, 0x0042_0045)?;
    if let Some(wrapping) = field(block, 0x0042_0046) {
        let wrapping = wrapping.as_structure()?;
        if !value
            .with_value(|value| matches!(value, ValueView::ByteString(bytes) if bytes.len() == 40))
            || !field(&wrapping, 0x0042_009e)?
                .with_value(|value| matches!(value, ValueView::Enumeration(1)))
            || !field(&wrapping, 0x0042_00a3)?
                .with_value(|value| matches!(value, ValueView::Enumeration(1)))
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
    if !field(block, 0x0042_0042)?.with_value(|value| matches!(value, ValueView::Enumeration(7))) {
        return None;
    }
    let value = value.as_structure()?;
    let material = field(&value, 0x0042_0043)?.as_structure()?;
    field(&material, 0x0042_003f)?
        .with_value(|value| matches!(value, ValueView::ByteString(bytes) if bytes.len() == 32))
        .then_some(KeyBlockShape::Transparent)
}
