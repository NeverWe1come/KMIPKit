//! KMIP 1.0-1.4 to 2.x protocol version and attribute compatibility adapter.

use kmipkit_ttlv::{Item, RawTag, Structure, Value, ValueView};

pub const V1_0: (u32, u32) = (1, 0);
pub const V1_1: (u32, u32) = (1, 1);
pub const V1_2: (u32, u32) = (1, 2);
pub const V1_3: (u32, u32) = (1, 3);
pub const V1_4: (u32, u32) = (1, 4);
pub const V2_0: (u32, u32) = (2, 0);
pub const V2_1: (u32, u32) = (2, 1);

pub const ALL_VERSIONS: &[(u32, u32)] = &[V1_0, V1_1, V1_2, V1_3, V1_4, V2_0, V2_1];

/// Verifies whether the specified version is supported by the test server.
pub fn is_version_supported(major: u32, minor: u32) -> bool {
    matches!(
        (major, minor),
        V1_0 | V1_1 | V1_2 | V1_3 | V1_4 | V2_0 | V2_1
    )
}

/// Tag constants for 1.x and 2.x translation.
pub const TAG_ATTRIBUTE: u32 = 0x0042_0008;
pub const TAG_ATTRIBUTE_INDEX: u32 = 0x0042_0009;
pub const TAG_ATTRIBUTE_NAME: u32 = 0x0042_000A;
pub const TAG_ATTRIBUTE_VALUE: u32 = 0x0042_000B;
pub const TAG_ATTRIBUTES: u32 = 0x0042_0125;
pub const TAG_TEMPLATE_ATTRIBUTE: u32 = 0x0042_0091;

/// Normalizes attributes inside a request structure to canonical 2.1 format.
/// If 1.x `TemplateAttribute` or `Attribute` structures are present, they are unpacked
/// into direct attribute items.
pub fn normalize_attributes_to_v2(item: &Item) -> Option<Item> {
    let raw = item.tag().raw();
    if raw == TAG_TEMPLATE_ATTRIBUTE || raw == TAG_ATTRIBUTES {
        let mut normalized = Structure::new();
        item.with_value(|view| {
            if let ValueView::Structure(s) = view {
                for child in s.children() {
                    if child.tag().raw() == TAG_ATTRIBUTE {
                        if let Some(direct_item) = unpack_v1_attribute(child) {
                            let _ = normalized.try_push(direct_item);
                        }
                    } else if let Some(norm_child) = normalize_attributes_to_v2(child) {
                        let _ = normalized.try_push(norm_child);
                    }
                }
            }
        });
        let tag = RawTag::new(TAG_ATTRIBUTES).ok()?.try_checked().ok()?;
        Item::new(tag, Value::structure(normalized)).ok()
    } else {
        item.try_clone().ok()
    }
}

fn unpack_v1_attribute(attr_item: &Item) -> Option<Item> {
    let mut name = None;
    let mut value_item = None;

    attr_item.with_value(|view| {
        if let ValueView::Structure(s) = view {
            for child in s.children() {
                if child.tag().raw() == TAG_ATTRIBUTE_NAME {
                    child.with_value(|v| {
                        if let ValueView::TextString(s) = v {
                            name = Some(s.to_string());
                        }
                    });
                } else if child.tag().raw() == TAG_ATTRIBUTE_VALUE {
                    value_item = child.try_clone().ok();
                }
            }
        }
    });

    let attr_name = name?;
    let val = value_item?;

    // Map attribute name to tag
    let tag_num = crate::codec::names::tag_value(&attr_name)
        .or_else(|| crate::codec::names::tag_value(&crate::codec::names::normalize(&attr_name)))?;
    let raw_tag = RawTag::new(tag_num).ok()?.try_checked().ok()?;

    // The unpacked item has tag_num and the payload from AttributeValue
    let mut item_to_return = None;
    val.with_value(|v| match v {
        ValueView::Structure(s) => {
            let mut new_s = Structure::new();
            for c in s.children() {
                if let Ok(cloned_c) = c.try_clone() {
                    let _ = new_s.try_push(cloned_c);
                }
            }
            item_to_return = Item::new(raw_tag, Value::structure(new_s)).ok();
        }
        ValueView::Integer(i) => item_to_return = Item::new(raw_tag, Value::integer(*i)).ok(),
        ValueView::LongInteger(i) => {
            item_to_return = Item::new(raw_tag, Value::long_integer(*i)).ok()
        }
        ValueView::BigInteger(b) => {
            item_to_return = Item::new(raw_tag, Value::big_integer(b.to_vec())).ok()
        }
        ValueView::Enumeration(e) => {
            item_to_return = Item::new(raw_tag, Value::enumeration(*e)).ok()
        }
        ValueView::Boolean(b) => item_to_return = Item::new(raw_tag, Value::boolean(*b)).ok(),
        ValueView::TextString(s) => {
            item_to_return = Item::new(raw_tag, Value::text_string(s.to_string())).ok()
        }
        ValueView::ByteString(b) => {
            item_to_return = Item::new(raw_tag, Value::byte_string(b.to_vec())).ok()
        }
        ValueView::DateTime(dt) => item_to_return = Item::new(raw_tag, Value::date_time(*dt)).ok(),
        ValueView::Interval(i) => item_to_return = Item::new(raw_tag, Value::interval(*i)).ok(),
        ValueView::DateTimeExtended(dt) => {
            item_to_return = Item::new(raw_tag, Value::date_time_extended(*dt)).ok()
        }
        _ => {}
    });

    item_to_return
}

/// Converts a 2.1 attribute item into a 1.x `Attribute` structure:
/// Structure {
///   AttributeName: TextString,
///   AttributeValue: <original value>
/// }
pub fn convert_v2_to_v1_attribute(item: &Item) -> Option<Item> {
    let raw = item.tag().raw();
    let name_str = crate::codec::names::tag_name(raw)?;

    let attr_tag = RawTag::new(TAG_ATTRIBUTE).ok()?.try_checked().ok()?;
    let name_tag = RawTag::new(TAG_ATTRIBUTE_NAME).ok()?.try_checked().ok()?;
    let val_tag = RawTag::new(TAG_ATTRIBUTE_VALUE).ok()?.try_checked().ok()?;

    let mut attr_struct = Structure::new();
    let name_item = Item::new(name_tag, Value::text_string(name_str.to_string())).ok()?;
    attr_struct.try_push(name_item).ok()?;

    let mut val_item_wrapped = None;
    item.with_value(|v| match v {
        ValueView::Structure(s) => {
            let mut new_s = Structure::new();
            for c in s.children() {
                if let Ok(cloned_c) = c.try_clone() {
                    let _ = new_s.try_push(cloned_c);
                }
            }
            val_item_wrapped = Item::new(val_tag, Value::structure(new_s)).ok();
        }
        ValueView::Integer(i) => val_item_wrapped = Item::new(val_tag, Value::integer(*i)).ok(),
        ValueView::LongInteger(i) => {
            val_item_wrapped = Item::new(val_tag, Value::long_integer(*i)).ok();
        }
        ValueView::BigInteger(b) => {
            val_item_wrapped = Item::new(val_tag, Value::big_integer(b.to_vec())).ok();
        }
        ValueView::Enumeration(e) => {
            val_item_wrapped = Item::new(val_tag, Value::enumeration(*e)).ok();
        }
        ValueView::Boolean(b) => val_item_wrapped = Item::new(val_tag, Value::boolean(*b)).ok(),
        ValueView::TextString(s) => {
            val_item_wrapped = Item::new(val_tag, Value::text_string(s.to_string())).ok();
        }
        ValueView::ByteString(b) => {
            val_item_wrapped = Item::new(val_tag, Value::byte_string(b.to_vec())).ok();
        }
        ValueView::DateTime(dt) => {
            val_item_wrapped = Item::new(val_tag, Value::date_time(*dt)).ok();
        }
        ValueView::Interval(i) => val_item_wrapped = Item::new(val_tag, Value::interval(*i)).ok(),
        ValueView::DateTimeExtended(dt) => {
            val_item_wrapped = Item::new(val_tag, Value::date_time_extended(*dt)).ok();
        }
        _ => {}
    });

    attr_struct.try_push(val_item_wrapped?).ok()?;
    Item::new(attr_tag, Value::structure(attr_struct)).ok()
}
