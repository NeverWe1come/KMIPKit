//! Safe C ABI conversions for typed managed objects and key structures.
use super::*;

fn convert_typed_item(
    item: *mut kmipkit_ttlv_item_t,
    out_item: *mut *mut kmipkit_ttlv_item_t,
    convert: impl FnOnce(Item) -> Result<Item, kmipkit_protocol::ManagedObjectError>,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let item = try_ffi!(copy_required_item_handle(item));
        let item = match convert(item) {
            Ok(item) => item,
            Err(kmipkit_protocol::ManagedObjectError::TtlvModelLimit) => {
                return record_ffi_error(ERROR_RESOURCE_LIMIT);
            }
            Err(_) => return record_ffi_error(ERROR_INVALID_INPUT),
        };
        *out = try_ffi!(try_make_handle(Kind::Item, HandleValue::Item(item)));
        SUCCESS
    })
}

/// Copies an Item after validating it as the selected KMIP managed Object Type.
///
/// Unknown Object Types and Structure tags remain in the generic TTLV model.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_managed_object_from_item(
    object_type: u32,
    item: *mut kmipkit_ttlv_item_t,
    out_item: *mut *mut kmipkit_ttlv_item_t,
) -> i32 {
    convert_typed_item(item, out_item, |item| {
        ManagedObject::try_from_item(ObjectType::from_raw(object_type), item)
            .map(ManagedObject::into_item)
    })
}

/// Copies an Item after validating it as a KMIP Key Block.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_key_block_from_item(
    item: *mut kmipkit_ttlv_item_t,
    out_item: *mut *mut kmipkit_ttlv_item_t,
) -> i32 {
    convert_typed_item(item, out_item, |item| {
        KeyBlock::try_from_item(item).and_then(KeyBlock::try_into_item)
    })
}

/// Copies an Item after validating it as a plaintext KMIP Key Value for a format.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_key_value_from_item(
    format_raw: u32,
    item: *mut kmipkit_ttlv_item_t,
    out_item: *mut *mut kmipkit_ttlv_item_t,
) -> i32 {
    convert_typed_item(item, out_item, |item| {
        KeyValue::try_from_item(item, KeyFormatType::from_raw(format_raw)).map(KeyValue::into_item)
    })
}

/// Copies an Item after validating it as KMIP Key Material for a format.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_key_material_from_item(
    format_raw: u32,
    item: *mut kmipkit_ttlv_item_t,
    out_item: *mut *mut kmipkit_ttlv_item_t,
) -> i32 {
    convert_typed_item(item, out_item, |item| {
        KeyMaterial::try_from_item(item, KeyFormatType::from_raw(format_raw))
            .map(KeyMaterial::into_item)
    })
}

/// Copies an Item after validating it as KMIP Key Wrapping Data.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_key_wrapping_data_from_item(
    item: *mut kmipkit_ttlv_item_t,
    out_item: *mut *mut kmipkit_ttlv_item_t,
) -> i32 {
    convert_typed_item(item, out_item, |item| {
        KeyWrappingData::try_from_item(item).and_then(KeyWrappingData::into_item)
    })
}
