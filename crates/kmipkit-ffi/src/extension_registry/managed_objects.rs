//! Safe C ABI conversions for typed managed objects and key structures.
use super::{
    ERROR_INVALID_INPUT, ERROR_RESOURCE_LIMIT, HandleValue, Item, KeyBlock, KeyFormatType,
    KeyMaterial, KeyValue, KeyWrappingData, Kind, ManagedObject, ObjectType, RawTag, SUCCESS,
    Structure, Value, copy_required_item_handle, ffi_entrypoint, item_type_code,
    kmipkit_ttlv_item_t, record_ffi_error, try_make_handle,
};
use kmipkit_protocol::{
    ManagedFieldValue, ManagedObjectError, ManagedValueKind, ManagedValueReader,
};
use std::ptr;

const MAX_TYPED_FIELD_COUNT: u64 = 100_000;
const KEY_BLOCK_TAG: u32 = 0x0042_0040;
const KEY_VALUE_TAG: u32 = 0x0042_0045;
const KEY_MATERIAL_TAG: u32 = 0x0042_0043;
const KEY_WRAPPING_DATA_TAG: u32 = 0x0042_0046;
const FIELD_OWNER_MANAGED_OBJECT: u32 = 1;
const FIELD_OWNER_KEY_BLOCK: u32 = 2;
const FIELD_OWNER_KEY_VALUE: u32 = 3;
const FIELD_OWNER_KEY_MATERIAL: u32 = 4;
const FIELD_OWNER_KEY_WRAPPING_DATA: u32 = 5;
const FIELD_READ_PRESENT: u32 = 1;
const FIELD_READ_TYPE: u32 = 2;
const FIELD_READ_ENUMERATION: u32 = 3;
const FIELD_READ_INTEGER: u32 = 4;
const FIELD_READ_BYTES: u32 = 5;

fn managed_value_kind(owner_kind: u32, context_raw: u32) -> Option<ManagedValueKind> {
    match owner_kind {
        FIELD_OWNER_MANAGED_OBJECT => Some(ManagedValueKind::ManagedObject(ObjectType::from_raw(
            context_raw,
        ))),
        FIELD_OWNER_KEY_BLOCK if context_raw == 0 => Some(ManagedValueKind::KeyBlock),
        FIELD_OWNER_KEY_VALUE => Some(ManagedValueKind::KeyValue(KeyFormatType::from_raw(
            context_raw,
        ))),
        FIELD_OWNER_KEY_MATERIAL => Some(ManagedValueKind::KeyMaterial(KeyFormatType::from_raw(
            context_raw,
        ))),
        FIELD_OWNER_KEY_WRAPPING_DATA if context_raw == 0 => {
            Some(ManagedValueKind::KeyWrappingData)
        }
        _ => None,
    }
}

fn copy_field_items(
    fields: *mut *mut kmipkit_ttlv_item_t,
    field_count: u64,
) -> Result<Vec<Item>, i32> {
    if field_count > MAX_TYPED_FIELD_COUNT {
        return Err(ERROR_RESOURCE_LIMIT);
    }
    let field_count = usize::try_from(field_count).map_err(|_| ERROR_RESOURCE_LIMIT)?;
    if field_count == 0 {
        return Ok(Vec::new());
    }
    if fields.is_null() {
        return Err(ERROR_INVALID_INPUT);
    }
    // SAFETY: The C ABI requires `fields` to address `field_count` readable Item handles.
    let field_handles = unsafe { std::slice::from_raw_parts(fields, field_count) };
    field_handles
        .iter()
        .map(|field| copy_required_item_handle(*field))
        .collect()
}

fn order_known_fields(fields: Vec<Item>, schema_order: &[u32]) -> Vec<Item> {
    let mut slots = Vec::new();
    let mut known_positions = Vec::new();
    let mut known_fields = Vec::new();
    for (index, item) in fields.into_iter().enumerate() {
        if let Some(position) = schema_order.iter().position(|tag| *tag == item.tag().raw()) {
            known_positions.push(index);
            known_fields.push((position, item));
            slots.push(None);
        } else {
            slots.push(Some(item));
        }
    }
    known_fields.sort_by_key(|(position, _)| *position);
    for (index, (_, item)) in known_positions.into_iter().zip(known_fields) {
        slots[index] = Some(item);
    }
    slots.into_iter().flatten().collect()
}

fn structure_item(
    raw_tag: u32,
    fields: Vec<Item>,
    schema_order: &[u32],
) -> Result<Item, ManagedObjectError> {
    let fields = order_known_fields(fields, schema_order);
    let tag = RawTag::new(raw_tag)
        .map_err(|_| ManagedObjectError::TtlvModelLimit)?
        .try_checked()
        .map_err(|_| ManagedObjectError::TtlvModelLimit)?;
    let mut structure = Structure::new();
    for field in fields {
        structure
            .try_push(field)
            .map_err(|_| ManagedObjectError::TtlvModelLimit)?;
    }
    Item::new(tag, Value::structure(structure)).map_err(|_| ManagedObjectError::TtlvModelLimit)
}

fn object_schema(object_type: u32) -> Option<(u32, &'static [u32])> {
    match object_type {
        1 => Some((0x0042_0013, &[0x0042_001d, 0x0042_001e])),
        2 => Some((0x0042_008f, &[KEY_BLOCK_TAG])),
        3 => Some((0x0042_006d, &[KEY_BLOCK_TAG])),
        4 => Some((0x0042_0064, &[KEY_BLOCK_TAG])),
        5 => Some((
            0x0042_0089,
            &[
                0x0042_008b,
                0x0042_0044,
                0x0042_008c,
                0x0042_008a,
                0x0042_0062,
                KEY_BLOCK_TAG,
            ],
        )),
        7 => Some((0x0042_0085, &[0x0042_0086, KEY_BLOCK_TAG])),
        8 => Some((0x0042_005b, &[0x0042_0059, 0x0042_005a])),
        9 => Some((0x0042_00bd, &[0x0042_00be, KEY_BLOCK_TAG])),
        10 => Some((0x0042_0018, &[0x0042_0019, 0x0042_0140])),
        _ => None,
    }
}

fn typed_fields_to_item(
    out_item: *mut *mut kmipkit_ttlv_item_t,
    fields: *mut *mut kmipkit_ttlv_item_t,
    field_count: u64,
    build: impl FnOnce(Vec<Item>) -> Result<Item, kmipkit_protocol::ManagedObjectError>,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let fields = match copy_field_items(fields, field_count) {
            Ok(fields) => fields,
            Err(status) => return record_ffi_error(status),
        };
        let item = match build(fields) {
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

fn managed_object_from_fields_impl(
    object_type: u32,
    fields: *mut *mut kmipkit_ttlv_item_t,
    field_count: u64,
    out_item: *mut *mut kmipkit_ttlv_item_t,
) -> i32 {
    typed_fields_to_item(out_item, fields, field_count, |fields| {
        let (object_tag, schema_order) = object_schema(object_type)
            .ok_or(kmipkit_protocol::ManagedObjectError::MismatchedObjectType)?;
        let item = structure_item(object_tag, fields, schema_order)?;
        ManagedObject::try_from_item(ObjectType::from_raw(object_type), item)
            .map(ManagedObject::into_item)
    })
}

/// Builds and validates a known managed-object Structure from caller-owned field Items.
///
/// Known fields are placed in their KMIP table order while unknown Items retain
/// their relative positions and repetitions. Input handles remain caller-owned.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_managed_object_from_fields(
    object_type: u32,
    fields: *mut *mut kmipkit_ttlv_item_t,
    field_count: u64,
    out_item: *mut *mut kmipkit_ttlv_item_t,
) -> i32 {
    managed_object_from_fields_impl(object_type, fields, field_count, out_item)
}

macro_rules! managed_object_constructor {
    ($name:ident, $object_type:expr) => {
        #[doc = "Builds the named managed-object Structure from caller-owned field Items."]
        #[unsafe(no_mangle)]
        pub extern "C" fn $name(
            fields: *mut *mut kmipkit_ttlv_item_t,
            field_count: u64,
            out_item: *mut *mut kmipkit_ttlv_item_t,
        ) -> i32 {
            managed_object_from_fields_impl($object_type, fields, field_count, out_item)
        }
    };
}

managed_object_constructor!(kmipkit_certificate_from_fields, 1);
managed_object_constructor!(kmipkit_symmetric_key_from_fields, 2);
managed_object_constructor!(kmipkit_public_key_from_fields, 3);
managed_object_constructor!(kmipkit_private_key_from_fields, 4);
managed_object_constructor!(kmipkit_split_key_from_fields, 5);
managed_object_constructor!(kmipkit_secret_data_from_fields, 7);
managed_object_constructor!(kmipkit_opaque_object_from_fields, 8);
managed_object_constructor!(kmipkit_pgp_key_from_fields, 9);
managed_object_constructor!(kmipkit_certificate_request_from_fields, 10);

/// Builds a source-ordered Key Block from caller-owned field Items.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_key_block_from_fields(
    fields: *mut *mut kmipkit_ttlv_item_t,
    field_count: u64,
    out_item: *mut *mut kmipkit_ttlv_item_t,
) -> i32 {
    typed_fields_to_item(out_item, fields, field_count, |fields| {
        let item = structure_item(
            KEY_BLOCK_TAG,
            fields,
            &[
                0x0042_0042,
                0x0042_0041,
                0x0042_0045,
                0x0042_0028,
                0x0042_002a,
                0x0042_0046,
            ],
        )?;
        KeyBlock::try_from_item(item).and_then(KeyBlock::try_into_item)
    })
}

/// Builds a plaintext Key Value Structure from caller-owned field Items.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_key_value_from_fields(
    format_raw: u32,
    fields: *mut *mut kmipkit_ttlv_item_t,
    field_count: u64,
    out_item: *mut *mut kmipkit_ttlv_item_t,
) -> i32 {
    typed_fields_to_item(out_item, fields, field_count, |fields| {
        let item = structure_item(KEY_VALUE_TAG, fields, &[KEY_MATERIAL_TAG, 0x0042_0125])?;
        KeyValue::try_from_item(item, KeyFormatType::from_raw(format_raw)).map(KeyValue::into_item)
    })
}

/// Builds transparent Key Material Structure fields for the explicit format.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_key_material_from_fields(
    format_raw: u32,
    fields: *mut *mut kmipkit_ttlv_item_t,
    field_count: u64,
    out_item: *mut *mut kmipkit_ttlv_item_t,
) -> i32 {
    typed_fields_to_item(out_item, fields, field_count, |fields| {
        if (1..=6).contains(&format_raw) {
            let mut fields = fields.into_iter();
            let Some(item) = fields.next() else {
                return Err(ManagedObjectError::InvalidStructure);
            };
            if fields.next().is_some() {
                return Err(ManagedObjectError::InvalidStructure);
            }
            return KeyMaterial::try_from_item(item, KeyFormatType::from_raw(format_raw))
                .map(KeyMaterial::into_item);
        }
        // KMIP 2.1 §11.25, Table 461 assigns transparent EC formats 0x14/0x15;
        // unassigned values such as 0x0E/0x0F retain generic Structure children.
        if !matches!(format_raw, 7..=13 | 0x14 | 0x15) {
            let item = structure_item(KEY_MATERIAL_TAG, fields, &[])?;
            return KeyMaterial::try_from_item(item, KeyFormatType::from_raw(format_raw))
                .map(KeyMaterial::into_item);
        }
        let schema_order: &[u32] = match format_raw {
            7 => &[0x0042_003f],
            8 => &[0x0042_005e, 0x0042_0071, 0x0042_0037, 0x0042_009f],
            9 => &[0x0042_005e, 0x0042_0071, 0x0042_0037, 0x0042_00a0],
            10 => &[
                0x0042_0052,
                0x0042_006c,
                0x0042_0063,
                0x0042_005e,
                0x0042_0071,
                0x0042_0060,
                0x0042_0061,
                0x0042_0027,
            ],
            11 => &[0x0042_0052, 0x0042_006c],
            12 => &[
                0x0042_005e,
                0x0042_0071,
                0x0042_0037,
                0x0042_003e,
                0x0042_009f,
            ],
            13 => &[
                0x0042_005e,
                0x0042_0071,
                0x0042_0037,
                0x0042_003e,
                0x0042_00a0,
            ],
            0x14 => &[0x0042_0075, 0x0042_002e],
            0x15 => &[0x0042_0075, 0x0042_0072],
            _ => &[],
        };
        let item = structure_item(KEY_MATERIAL_TAG, fields, schema_order)?;
        KeyMaterial::try_from_item(item, KeyFormatType::from_raw(format_raw))
            .map(KeyMaterial::into_item)
    })
}

/// Builds Key Wrapping Data from caller-owned field Items.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_key_wrapping_data_from_fields(
    fields: *mut *mut kmipkit_ttlv_item_t,
    field_count: u64,
    out_item: *mut *mut kmipkit_ttlv_item_t,
) -> i32 {
    typed_fields_to_item(out_item, fields, field_count, |fields| {
        let item = structure_item(
            KEY_WRAPPING_DATA_TAG,
            fields,
            &[
                0x0042_009e,
                0x0042_0036,
                0x0042_004e,
                0x0042_004d,
                0x0042_003d,
                0x0042_00a3,
            ],
        )?;
        KeyWrappingData::try_from_item(item).and_then(KeyWrappingData::into_item)
    })
}

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

/// Reads one direct typed field after validating its managed-object or key-structure context.
///
/// `owner_kind` is 1 for a managed object, 2 for Key Block, 3 for Key Value,
/// 4 for Key Material, and 5 for Key Wrapping Data. `context_raw` is the raw
/// Object Type for kind 1 or raw Key Format Type for kinds 3 and 4; it is zero
/// for kinds 2 and 5. `read_kind` selects 1=presence, 2=TTLV type,
/// 3=Enumeration, 4=Integer, or 5=Byte String/Big Integer bytes. The caller
/// owns all output storage. For byte reads, a null buffer with zero capacity
/// queries the required length; a short buffer is rejected without partial copy.
/// The input Item remains caller-owned and the returned bytes are caller-owned.
///
/// # Safety
///
/// `item` must be null or a live Item handle created by `KMIPKit`. `out_present`
/// must point to writable, properly aligned `u8` storage. Every other non-null
/// output pointer must point to writable, properly aligned storage of its
/// declared type, and output regions must not overlap. If `out_bytes` is
/// non-null, it must cover `byte_capacity` writable bytes and must not overlap
/// the input handle or any other output region.
#[allow(clippy::too_many_arguments)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kmipkit_managed_value_field_read(
    owner_kind: u32,
    context_raw: u32,
    item: *mut kmipkit_ttlv_item_t,
    field_tag: u32,
    read_kind: u32,
    out_present: *mut u8,
    out_item_type: *mut u8,
    out_enumeration: *mut u32,
    out_integer: *mut i32,
    out_bytes: *mut u8,
    byte_capacity: u64,
    out_byte_length: *mut u64,
) -> i32 {
    ffi_entrypoint(|| {
        if out_present.is_null() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let output_is_valid = match read_kind {
            FIELD_READ_PRESENT => true,
            FIELD_READ_TYPE => !out_item_type.is_null(),
            FIELD_READ_ENUMERATION => !out_enumeration.is_null(),
            FIELD_READ_INTEGER => !out_integer.is_null(),
            FIELD_READ_BYTES => !out_byte_length.is_null(),
            _ => false,
        };
        if !output_is_valid || (out_bytes.is_null() && byte_capacity != 0) {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let Some(kind) = managed_value_kind(owner_kind, context_raw) else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let item = try_ffi!(copy_required_item_handle(item));
        let Ok(reader) = ManagedValueReader::try_from_item(kind, item) else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let field = reader.read_field(field_tag);
        if field.is_none() && read_kind != FIELD_READ_PRESENT {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }

        // SAFETY: `out_present` was checked non-null and is a caller-owned output slot.
        unsafe { out_present.write(u8::from(field.is_some())) };
        let Some(field) = field else {
            return SUCCESS;
        };
        if !out_item_type.is_null() {
            // SAFETY: A non-null output slot is caller-owned and valid for a single byte.
            unsafe { out_item_type.write(item_type_code(field.item_type())) };
        }
        match read_kind {
            FIELD_READ_PRESENT | FIELD_READ_TYPE => SUCCESS,
            FIELD_READ_ENUMERATION => match field {
                ManagedFieldValue::Enumeration(raw) => {
                    // SAFETY: The required output slot was checked before reading the field.
                    unsafe { out_enumeration.write(raw) };
                    SUCCESS
                }
                _ => record_ffi_error(ERROR_INVALID_INPUT),
            },
            FIELD_READ_INTEGER => match field {
                ManagedFieldValue::Integer(raw) => {
                    // SAFETY: The required output slot was checked before reading the field.
                    unsafe { out_integer.write(raw) };
                    SUCCESS
                }
                _ => record_ffi_error(ERROR_INVALID_INPUT),
            },
            FIELD_READ_BYTES => match field {
                ManagedFieldValue::ByteString(bytes) | ManagedFieldValue::BigInteger(bytes) => {
                    let Ok(length) = u64::try_from(bytes.len()) else {
                        return record_ffi_error(ERROR_RESOURCE_LIMIT);
                    };
                    // SAFETY: The required length output was checked before reading the field.
                    unsafe { out_byte_length.write(length) };
                    if !out_bytes.is_null() && length > byte_capacity {
                        return record_ffi_error(ERROR_INVALID_INPUT);
                    }
                    if length > 0 && !out_bytes.is_null() {
                        // SAFETY: The caller supplied a non-null buffer whose declared capacity
                        // is at least the complete byte length, and it cannot alias the Item.
                        unsafe {
                            std::ptr::copy_nonoverlapping(bytes.as_ptr(), out_bytes, bytes.len());
                        };
                    }
                    SUCCESS
                }
                _ => record_ffi_error(ERROR_INVALID_INPUT),
            },
            _ => record_ffi_error(ERROR_INVALID_INPUT),
        }
    })
}

/// Returns an independently owned copy of one direct Item child after typed model validation.
///
/// `owner_kind` and `context_raw` use the same values as
/// `kmipkit_managed_value_field_read`. When the field is absent, `out_present` is zero and
/// `out_item` is null. When present, the caller owns the copied Item and must release it.
///
/// # Safety
///
/// `item` must be null or a live Item handle created by `KMIPKit`. `out_present`
/// and `out_item` must point to distinct, writable, properly aligned output
/// storage of their declared types. Neither output region may overlap the input
/// handle or another output region.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kmipkit_managed_value_field_item_read(
    owner_kind: u32,
    context_raw: u32,
    item: *mut kmipkit_ttlv_item_t,
    field_tag: u32,
    out_present: *mut u8,
    out_item: *mut *mut kmipkit_ttlv_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        if out_present.is_null() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let out = out_slot!(out_item);
        // SAFETY: The output pointer was checked and is writable caller-owned storage.
        unsafe { out_present.write(0) };

        let Some(kind) = managed_value_kind(owner_kind, context_raw) else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let item = try_ffi!(copy_required_item_handle(item));
        let Ok(reader) = ManagedValueReader::try_from_item(kind, item) else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(nested) = reader.read_field_item(field_tag) else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(nested) = nested else {
            return SUCCESS;
        };
        *out = try_ffi!(try_make_handle(Kind::Item, HandleValue::Item(nested)));
        // SAFETY: The output pointer was checked and is writable caller-owned storage.
        unsafe { out_present.write(1) };
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

#[cfg(test)]
#[path = "../../tests/support/managed_objects_coverage_tests.rs"]
mod coverage_tests;
