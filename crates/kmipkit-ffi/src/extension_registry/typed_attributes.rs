//! Safe C ABI conversions for the generated direct-attribute catalog.
use super::{
    AttributeSet, ERROR_INTERNAL, ERROR_INVALID_INPUT, ERROR_TYPED_ATTRIBUTE_TAG_MISMATCH,
    ERROR_TYPED_ATTRIBUTE_VALUE_TYPE_MISMATCH, FfiResult, HandleValue, Item, Kind, RawTag, SUCCESS,
    Structure, Value, ValueView, copy_required_item_handle, ffi_entrypoint, kmipkit_ttlv_item_t,
    kmipkit_ttlv_value_t, model_status, record_ffi_error, reference_handle, try_make_handle,
    typed_attributes_catalog_generated,
};
use std::ptr;

/// Validates one Item with the protocol `AttributeSet`'s catalog-backed rules.
pub(super) fn validate_typed_item(expected_tag: u32, item: Item) -> FfiResult<Item> {
    if !typed_attributes_catalog_generated::is_catalogued_attribute_tag(expected_tag) {
        return Err(ERROR_INVALID_INPUT);
    }
    if item.tag().raw() != expected_tag {
        return Err(ERROR_TYPED_ATTRIBUTE_TAG_MISMATCH);
    }
    let attributes = match AttributeSet::try_new([item]) {
        Ok(attributes) => attributes,
        Err(kmipkit_protocol::AttributeSetError::AttributeTtlvTypeMismatch) => {
            return Err(ERROR_TYPED_ATTRIBUTE_VALUE_TYPE_MISMATCH);
        }
        Err(_) => return Err(ERROR_INVALID_INPUT),
    };
    attributes
        .into_items()
        .into_iter()
        .next()
        .ok_or(ERROR_INTERNAL)
}

/// Returns a copied catalogued direct attribute Item after validating its tag and TTLV type.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_typed_attribute_from_item(
    expected_tag: u32,
    item: *mut kmipkit_ttlv_item_t,
    out_item: *mut *mut kmipkit_ttlv_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let item = try_ffi!(copy_required_item_handle(item));
        let item = try_ffi!(validate_typed_item(expected_tag, item));
        *out = try_ffi!(try_make_handle(Kind::Item, HandleValue::Item(item)));
        SUCCESS
    })
}

/// Constructs an owned typed Item by pairing a catalogued tag with a copied TTLV value.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_typed_attribute_from_value(
    expected_tag: u32,
    value: *mut kmipkit_ttlv_value_t,
    out_item: *mut *mut kmipkit_ttlv_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let source = try_ffi!(reference_handle(value, Kind::Value));
        let HandleValue::Value(source) = &source.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let value = try_ffi!(kmipkit_ttlv::try_clone_value(source).map_err(model_status));
        let tag = try_ffi!(
            RawTag::new(expected_tag)
                .and_then(|tag| tag.try_checked())
                .map_err(|_| ERROR_INVALID_INPUT)
        );
        let item = try_ffi!(Item::new(tag, value).map_err(|_| ERROR_INVALID_INPUT));
        let item = try_ffi!(validate_typed_item(expected_tag, item));
        *out = try_ffi!(try_make_handle(Kind::Item, HandleValue::Item(item)));
        SUCCESS
    })
}

/// Returns an owned generic TTLV value copy after checking its direct attribute marker.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_typed_attribute_value(
    expected_tag: u32,
    item: *mut kmipkit_ttlv_item_t,
    out_value: *mut *mut kmipkit_ttlv_value_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_value);
        let item = try_ffi!(copy_required_item_handle(item));
        let item = try_ffi!(validate_typed_item(expected_tag, item));
        let value = try_ffi!(item.with_value(clone_value_view).map_err(model_status));
        *out = try_ffi!(try_make_handle(Kind::Value, HandleValue::Value(value)));
        SUCCESS
    })
}

fn clone_value_view(value: ValueView<'_>) -> Result<Value, kmipkit_ttlv::ModelError> {
    match value {
        ValueView::Structure(source) => {
            let mut structure = Structure::new();
            for child in source.children() {
                let value = child.with_value(clone_value_view)?;
                structure.try_push(Item::new(child.tag(), value)?)?;
            }
            Ok(Value::structure(structure))
        }
        ValueView::Integer(value) => Ok(Value::integer(*value)),
        ValueView::LongInteger(value) => Ok(Value::long_integer(*value)),
        ValueView::BigInteger(value) => Ok(Value::big_integer(value.to_vec())),
        ValueView::Enumeration(value) => Ok(Value::enumeration(*value)),
        ValueView::Boolean(value) => Ok(Value::boolean(*value)),
        ValueView::TextString(value) => Ok(Value::text_string((*value).to_owned())),
        ValueView::ByteString(value) => Ok(Value::byte_string(value.to_vec())),
        ValueView::DateTime(value) => Ok(Value::date_time(*value)),
        ValueView::Interval(value) => Ok(Value::interval(*value)),
        ValueView::DateTimeExtended(value) => Ok(Value::date_time_extended(*value)),
        _ => Err(kmipkit_ttlv::ModelError::UnsupportedValue),
    }
}
