//! Safe C ABI conversions for the generated direct-attribute catalog.
use super::*;

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
        if item.tag().raw() != expected_tag
            || !typed_attributes_catalog_generated::is_catalogued_attribute_tag(expected_tag)
        {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let mut attributes = AttributeSet::new();
        if attributes.try_push(item).is_err() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let Some(item) = attributes.as_items().first() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        let item = try_ffi!(item.try_clone().map_err(model_status));
        *out = try_ffi!(try_make_handle(Kind::Item, HandleValue::Item(item)));
        SUCCESS
    })
}
