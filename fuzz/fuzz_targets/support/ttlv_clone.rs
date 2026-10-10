//! Bounded owned cloning for decoded TTLV trees used by fuzz targets.

use kmipkit_ttlv::{Item, Structure, Value, ValueView};

pub(crate) fn clone_structure(source: &kmipkit_ttlv::StructureView<'_>) -> Option<Structure> {
    let mut copy = Structure::new();
    for child in source.children() {
        let value = child.with_value(clone_value)?;
        copy.try_push(Item::new(child.tag(), value).ok()?).ok()?;
    }
    Some(copy)
}

fn clone_value(source: ValueView<'_>) -> Option<Value> {
    match source {
        ValueView::Structure(value) => Some(Value::structure(clone_structure(&value)?)),
        ValueView::Integer(value) => Some(Value::integer(*value)),
        ValueView::LongInteger(value) => Some(Value::long_integer(*value)),
        ValueView::BigInteger(value) => Some(Value::big_integer(value.to_vec())),
        ValueView::Enumeration(value) => Some(Value::enumeration(*value)),
        ValueView::Boolean(value) => Some(Value::boolean(*value)),
        ValueView::TextString(value) => Some(Value::text_string((*value).to_owned())),
        ValueView::ByteString(value) => Some(Value::byte_string(value.to_vec())),
        ValueView::DateTime(value) => Some(Value::date_time(*value)),
        ValueView::Interval(value) => Some(Value::interval(*value)),
        ValueView::DateTimeExtended(value) => Some(Value::date_time_extended(*value)),
        _ => None,
    }
}
