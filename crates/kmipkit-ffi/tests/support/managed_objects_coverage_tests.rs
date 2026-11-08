use super::*;
use crate::extension_registry::release;
use kmipkit_ttlv::ItemType;

fn fixture_item(raw_tag: u32, value: Value) -> Item {
    let tag = RawTag::new(raw_tag)
        .expect("fixture tag is representable")
        .try_checked()
        .expect("fixture tag is allocated");
    Item::new(tag, value).expect("fixture Item is valid")
}

fn fixture_certificate() -> Item {
    fixture_item(
        0x0042_0013,
        Value::structure({
            let mut fields = Structure::new();
            fields
                .try_push(fixture_item(0x0042_001d, Value::enumeration(0x8000_0042)))
                .expect("certificate type fits");
            fields
                .try_push(fixture_item(
                    0x0042_001e,
                    Value::byte_string(vec![0x41, 0x00, 0xff]),
                ))
                .expect("certificate value fits");
            fields
        }),
    )
}

fn item_handle(item: Item) -> *mut kmipkit_ttlv_item_t {
    try_make_handle(Kind::Item, HandleValue::Item(item)).expect("fixture handle is allocated")
}

#[test]
fn private_schema_and_field_helpers_cover_open_contexts_and_limits() {
    assert!(managed_value_kind(FIELD_OWNER_MANAGED_OBJECT, 0x8000_0001).is_some());
    assert!(managed_value_kind(FIELD_OWNER_KEY_BLOCK, 0).is_some());
    assert!(managed_value_kind(FIELD_OWNER_KEY_VALUE, 0x8000_0002).is_some());
    assert!(managed_value_kind(FIELD_OWNER_KEY_MATERIAL, 0x8000_0003).is_some());
    assert!(managed_value_kind(FIELD_OWNER_KEY_WRAPPING_DATA, 0).is_some());
    assert!(managed_value_kind(0, 0).is_none());
    assert!(managed_value_kind(FIELD_OWNER_KEY_BLOCK, 1).is_none());
    assert!(managed_value_kind(FIELD_OWNER_KEY_WRAPPING_DATA, 1).is_none());

    for object_type in [1, 2, 3, 4, 5, 7, 8, 9, 10] {
        assert!(object_schema(object_type).is_some());
    }
    assert!(object_schema(6).is_none());
    assert!(
        copy_field_items(ptr::null_mut(), 0)
            .expect("empty field list is valid")
            .is_empty()
    );
    assert!(matches!(
        copy_field_items(ptr::null_mut(), MAX_TYPED_FIELD_COUNT + 1),
        Err(ERROR_RESOURCE_LIMIT)
    ));
    assert!(matches!(
        copy_field_items(ptr::null_mut(), 1),
        Err(ERROR_INVALID_INPUT)
    ));

    let unknown = fixture_item(0x0054_0001, Value::integer(9));
    let later_known = fixture_item(0x0042_0002, Value::integer(2));
    let earlier_known = fixture_item(0x0042_0001, Value::integer(1));
    let ordered = order_known_fields(
        vec![later_known, unknown, earlier_known],
        &[0x0042_0001, 0x0042_0002],
    )
    .expect("known fields reorder without moving unknown fields");
    assert_eq!(
        ordered
            .iter()
            .map(|item| item.tag().raw())
            .collect::<Vec<_>>(),
        [0x0042_0001, 0x0054_0001, 0x0042_0002]
    );
    assert!(matches!(
        structure_item(u32::MAX, Vec::new(), &[]),
        Err(ManagedObjectError::TtlvModelLimit)
    ));
}

#[test]
fn typed_item_builders_map_invalid_limits_and_success() {
    let mut output = ptr::null_mut();
    assert_eq!(
        typed_fields_to_item(ptr::null_mut(), ptr::null_mut(), 0, |_| {
            Ok(fixture_item(0x0042_0001, Value::integer(1)))
        }),
        ERROR_INVALID_INPUT
    );
    assert_eq!(
        typed_fields_to_item(&raw mut output, ptr::null_mut(), 1, |_| {
            Ok(fixture_item(0x0042_0001, Value::integer(1)))
        }),
        ERROR_INVALID_INPUT
    );
    assert_eq!(
        typed_fields_to_item(
            &raw mut output,
            ptr::null_mut(),
            MAX_TYPED_FIELD_COUNT + 1,
            |_| Ok(fixture_item(0x0042_0001, Value::integer(1))),
        ),
        ERROR_RESOURCE_LIMIT
    );
    assert_eq!(
        typed_fields_to_item(&raw mut output, ptr::null_mut(), 0, |_| {
            Err(ManagedObjectError::TtlvModelLimit)
        }),
        ERROR_RESOURCE_LIMIT
    );
    assert_eq!(
        typed_fields_to_item(&raw mut output, ptr::null_mut(), 0, |_| {
            Err(ManagedObjectError::InvalidStructure)
        }),
        ERROR_INVALID_INPUT
    );
    assert_eq!(
        typed_fields_to_item(&raw mut output, ptr::null_mut(), 0, |_| {
            Ok(fixture_item(0x0042_0001, Value::integer(1)))
        }),
        SUCCESS
    );
    release(output, Kind::Item);
    assert_eq!(
        managed_object_from_fields_impl(6, ptr::null_mut(), 0, &raw mut output),
        ERROR_INVALID_INPUT
    );
    assert_eq!(
        convert_typed_item(ptr::null_mut(), &raw mut output, Ok),
        ERROR_INVALID_INPUT
    );
    let input = item_handle(fixture_item(0x0042_0001, Value::integer(1)));
    assert_eq!(
        convert_typed_item(input, ptr::null_mut(), Ok),
        ERROR_INVALID_INPUT
    );
    assert_eq!(
        convert_typed_item(input, &raw mut output, |_| {
            Err(ManagedObjectError::TtlvModelLimit)
        }),
        ERROR_RESOURCE_LIMIT
    );
    assert_eq!(
        convert_typed_item(input, &raw mut output, |_| {
            Err(ManagedObjectError::InvalidStructure)
        }),
        ERROR_INVALID_INPUT
    );
    assert_eq!(convert_typed_item(input, &raw mut output, Ok), SUCCESS);
    release(output, Kind::Item);
    release(input, Kind::Item);
}

#[test]
fn managed_value_reader_abi_rejects_invalid_arguments_and_contexts() {
    let input = item_handle(fixture_certificate());
    let mut present = 0_u8;
    let mut byte_length = 0_u64;
    // SAFETY: Null output cases are explicitly rejected before dereferencing any pointer.
    assert_eq!(
        unsafe {
            kmipkit_managed_value_field_read(
                FIELD_OWNER_MANAGED_OBJECT,
                1,
                input,
                0x0042_001d,
                FIELD_READ_PRESENT,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                0,
                ptr::null_mut(),
            )
        },
        ERROR_INVALID_INPUT
    );
    // SAFETY: All non-null outputs point to writable local variables; invalid selectors are rejected.
    assert_eq!(
        unsafe {
            kmipkit_managed_value_field_read(
                FIELD_OWNER_MANAGED_OBJECT,
                1,
                input,
                0x0042_001d,
                99,
                &raw mut present,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                0,
                ptr::null_mut(),
            )
        },
        ERROR_INVALID_INPUT
    );
    // SAFETY: The non-zero capacity with a null byte span is rejected before use.
    assert_eq!(
        unsafe {
            kmipkit_managed_value_field_read(
                FIELD_OWNER_MANAGED_OBJECT,
                1,
                input,
                0x0042_001e,
                FIELD_READ_BYTES,
                &raw mut present,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                1,
                &raw mut byte_length,
            )
        },
        ERROR_INVALID_INPUT
    );
    // SAFETY: Output is writable; unsupported owner/context combinations are rejected.
    assert_eq!(
        unsafe {
            kmipkit_managed_value_field_read(
                99,
                0,
                input,
                0x0042_001d,
                FIELD_READ_PRESENT,
                &raw mut present,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                0,
                ptr::null_mut(),
            )
        },
        ERROR_INVALID_INPUT
    );
    // SAFETY: Valid outputs are writable; invalid schema contexts and missing required fields return errors.
    assert_eq!(
        unsafe {
            kmipkit_managed_value_field_read(
                FIELD_OWNER_KEY_BLOCK,
                0,
                input,
                0x0042_001d,
                FIELD_READ_PRESENT,
                &raw mut present,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                0,
                ptr::null_mut(),
            )
        },
        ERROR_INVALID_INPUT
    );
    release(input, Kind::Item);
}

#[test]
fn managed_value_reader_abi_reads_presence_and_typed_values() {
    let input = item_handle(fixture_certificate());
    let mut present = 0_u8;
    let mut item_type = 0_u8;
    let mut enumeration = 0_u32;
    let mut integer = 0_i32;
    // SAFETY: `present` is writable and a presence-only query succeeds for a missing tag.
    assert_eq!(
        unsafe {
            kmipkit_managed_value_field_read(
                FIELD_OWNER_MANAGED_OBJECT,
                1,
                input,
                0x0054_0001,
                FIELD_READ_PRESENT,
                &raw mut present,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                0,
                ptr::null_mut(),
            )
        },
        SUCCESS
    );
    assert_eq!(present, 0);
    // SAFETY: The requested absent enumeration cannot be written and returns invalid input.
    assert_eq!(
        unsafe {
            kmipkit_managed_value_field_read(
                FIELD_OWNER_MANAGED_OBJECT,
                1,
                input,
                0x0054_0001,
                FIELD_READ_ENUMERATION,
                &raw mut present,
                ptr::null_mut(),
                &raw mut enumeration,
                ptr::null_mut(),
                ptr::null_mut(),
                0,
                ptr::null_mut(),
            )
        },
        ERROR_INVALID_INPUT
    );
    // SAFETY: Each output points to a valid local slot and the certificate type is an Enumeration.
    assert_eq!(
        unsafe {
            kmipkit_managed_value_field_read(
                FIELD_OWNER_MANAGED_OBJECT,
                1,
                input,
                0x0042_001d,
                FIELD_READ_ENUMERATION,
                &raw mut present,
                &raw mut item_type,
                &raw mut enumeration,
                ptr::null_mut(),
                ptr::null_mut(),
                0,
                ptr::null_mut(),
            )
        },
        SUCCESS
    );
    assert_eq!(present, 1);
    assert_eq!(enumeration, 0x8000_0042);
    assert_eq!(item_type, item_type_code(ItemType::Enumeration));
    // SAFETY: `integer` is writable but Certificate Type is not an Integer.
    assert_eq!(
        unsafe {
            kmipkit_managed_value_field_read(
                FIELD_OWNER_MANAGED_OBJECT,
                1,
                input,
                0x0042_001d,
                FIELD_READ_INTEGER,
                &raw mut present,
                ptr::null_mut(),
                ptr::null_mut(),
                &raw mut integer,
                ptr::null_mut(),
                0,
                ptr::null_mut(),
            )
        },
        ERROR_INVALID_INPUT
    );
    release(input, Kind::Item);
}

#[test]
fn managed_value_reader_abi_checks_byte_buffer_capacity() {
    let input = item_handle(fixture_certificate());
    let mut present = 0_u8;
    let mut byte_length = 0_u64;
    // SAFETY: A null output span and zero capacity query the required Byte String length.
    assert_eq!(
        unsafe {
            kmipkit_managed_value_field_read(
                FIELD_OWNER_MANAGED_OBJECT,
                1,
                input,
                0x0042_001e,
                FIELD_READ_BYTES,
                &raw mut present,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                0,
                &raw mut byte_length,
            )
        },
        SUCCESS
    );
    assert_eq!(byte_length, 3);
    let mut short = [0_u8; 2];
    // SAFETY: `short` is writable but too small; the ABI reports required size without copying.
    assert_eq!(
        unsafe {
            kmipkit_managed_value_field_read(
                FIELD_OWNER_MANAGED_OBJECT,
                1,
                input,
                0x0042_001e,
                FIELD_READ_BYTES,
                &raw mut present,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                short.as_mut_ptr(),
                short.len() as u64,
                &raw mut byte_length,
            )
        },
        ERROR_INVALID_INPUT
    );
    assert_eq!(short, [0; 2]);
    let mut complete = [0_u8; 3];
    // SAFETY: `complete` is a writable span with capacity equal to the Byte String length.
    assert_eq!(
        unsafe {
            kmipkit_managed_value_field_read(
                FIELD_OWNER_MANAGED_OBJECT,
                1,
                input,
                0x0042_001e,
                FIELD_READ_BYTES,
                &raw mut present,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                complete.as_mut_ptr(),
                complete.len() as u64,
                &raw mut byte_length,
            )
        },
        SUCCESS
    );
    assert_eq!(complete, [0x41, 0x00, 0xff]);
    release(input, Kind::Item);
}

#[test]
fn managed_value_item_reader_checks_context_and_absence() {
    let input = item_handle(fixture_certificate());
    let mut present = 0_u8;
    let mut output = ptr::null_mut();
    // SAFETY: Null outputs and unsupported contexts are rejected without dereference.
    assert_eq!(
        unsafe {
            kmipkit_managed_value_field_item_read(
                FIELD_OWNER_MANAGED_OBJECT,
                1,
                input,
                0x0042_001d,
                ptr::null_mut(),
                &raw mut output,
            )
        },
        ERROR_INVALID_INPUT
    );
    // SAFETY: `present` is writable, but a null Item output fails the required out-slot check.
    assert_eq!(
        unsafe {
            kmipkit_managed_value_field_item_read(
                FIELD_OWNER_MANAGED_OBJECT,
                1,
                input,
                0x0042_001d,
                &raw mut present,
                ptr::null_mut(),
            )
        },
        ERROR_INVALID_INPUT
    );
    // SAFETY: Both outputs are valid; an absent field returns a null owned handle.
    assert_eq!(
        unsafe {
            kmipkit_managed_value_field_item_read(
                FIELD_OWNER_MANAGED_OBJECT,
                1,
                input,
                0x0054_0001,
                &raw mut present,
                &raw mut output,
            )
        },
        SUCCESS
    );
    assert_eq!(present, 0);
    assert!(output.is_null());
    // SAFETY: Valid outputs are passed with an invalid owner kind, which is rejected.
    assert_eq!(
        unsafe {
            kmipkit_managed_value_field_item_read(
                99,
                0,
                input,
                0x0042_001d,
                &raw mut present,
                &raw mut output,
            )
        },
        ERROR_INVALID_INPUT
    );
    release(input, Kind::Item);
}
