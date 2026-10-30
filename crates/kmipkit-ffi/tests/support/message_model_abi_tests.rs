use super::*;
use crate::kmipkit_protocol_version_v1_t;

const REQUEST_HEADER_TAG: u32 = 0x0042_0077;
const REQUEST_PAYLOAD_TAG: u32 = 0x0042_0079;
const PROTOCOL_VERSION_TAG: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR_TAG: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR_TAG: u32 = 0x0042_006B;
const BATCH_COUNT_TAG: u32 = 0x0042_000D;
const BATCH_ITEM_TAG: u32 = 0x0042_000F;
const OPERATION_TAG: u32 = 0x0042_005C;

fn message_tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("the fixture tag fits the KMIP tag width")
        .try_checked()
        .expect("the fixture tag is allocated by KMIP 2.1")
}

fn message_item(raw: u32, value: Value) -> Item {
    Item::new(message_tag(raw), value).expect("the fixture item is valid")
}

fn message_structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut structure = Structure::new();
    for item in items {
        structure
            .try_push(item)
            .expect("the fixture stays within TTLV limits");
    }
    structure
}

fn valid_request_message_tree() -> Structure {
    let version = message_structure([
        message_item(PROTOCOL_VERSION_MAJOR_TAG, Value::integer(2)),
        message_item(PROTOCOL_VERSION_MINOR_TAG, Value::integer(1)),
    ]);
    let header = message_structure([
        message_item(PROTOCOL_VERSION_TAG, Value::structure(version)),
        message_item(BATCH_COUNT_TAG, Value::integer(1)),
    ]);
    let batch = message_structure([
        message_item(OPERATION_TAG, Value::enumeration(0xF123_4567)),
        message_item(REQUEST_PAYLOAD_TAG, Value::structure(Structure::new())),
    ]);
    message_structure([
        message_item(REQUEST_HEADER_TAG, Value::structure(header)),
        message_item(BATCH_ITEM_TAG, Value::structure(batch)),
    ])
}

fn request_message_missing_multi_item_id() -> Structure {
    let version = message_structure([
        message_item(PROTOCOL_VERSION_MAJOR_TAG, Value::integer(2)),
        message_item(PROTOCOL_VERSION_MINOR_TAG, Value::integer(1)),
    ]);
    let header = message_structure([
        message_item(PROTOCOL_VERSION_TAG, Value::structure(version)),
        message_item(BATCH_COUNT_TAG, Value::integer(2)),
    ]);
    let first_batch = message_structure([
        message_item(OPERATION_TAG, Value::enumeration(1)),
        message_item(0x0042_0093, Value::byte_string(vec![0xA1])),
        message_item(REQUEST_PAYLOAD_TAG, Value::structure(Structure::new())),
    ]);
    let second_batch = message_structure([
        message_item(OPERATION_TAG, Value::enumeration(1)),
        message_item(REQUEST_PAYLOAD_TAG, Value::structure(Structure::new())),
    ]);
    message_structure([
        message_item(REQUEST_HEADER_TAG, Value::structure(header)),
        message_item(BATCH_ITEM_TAG, Value::structure(first_batch)),
        message_item(BATCH_ITEM_TAG, Value::structure(second_batch)),
    ])
}

#[test]
fn request_message_handle_exposes_validated_header_and_open_operation() {
    let structure = try_make_handle(
        Kind::Structure,
        HandleValue::Structure(valid_request_message_tree()),
    )
    .expect("the generic TTLV input handle is allocated");
    let mut message = ptr::null_mut();
    let mut validation_error = ptr::dangling_mut();

    assert_eq!(
        kmipkit_request_message_from_ttlv(structure, &raw mut message, &raw mut validation_error,),
        SUCCESS
    );
    assert!(!message.is_null());
    assert!(validation_error.is_null());

    let mut header = ptr::null_mut();
    assert_eq!(
        kmipkit_request_message_header(message, &raw mut header),
        SUCCESS
    );
    let mut version = kmipkit_protocol_version_v1_t { major: 0, minor: 0 };
    assert_eq!(
        kmipkit_request_header_view_protocol_version(header, &raw mut version),
        SUCCESS
    );
    assert_eq!(
        version,
        kmipkit_protocol_version_v1_t { major: 2, minor: 1 }
    );

    let mut batch_count = 0_i32;
    assert_eq!(
        kmipkit_request_header_view_batch_count(header, &raw mut batch_count),
        SUCCESS
    );
    assert_eq!(batch_count, 1);

    let mut item_count = 0_u64;
    assert_eq!(
        kmipkit_request_message_batch_item_count(message, &raw mut item_count),
        SUCCESS
    );
    assert_eq!(item_count, 1);

    let mut batch_item = ptr::null_mut();
    assert_eq!(
        kmipkit_request_message_batch_item_at(message, 0, &raw mut batch_item),
        SUCCESS
    );
    let mut has_operation = u8::MAX;
    let mut operation = 0;
    assert_eq!(
        kmipkit_request_batch_item_view_operation(
            batch_item,
            &raw mut has_operation,
            &raw mut operation,
        ),
        SUCCESS
    );
    assert_eq!(has_operation, 1);
    assert_eq!(operation, 0xF123_4567);

    let mut original_ttlv = ptr::null_mut();
    assert_eq!(
        kmipkit_request_message_ttlv(message, &raw mut original_ttlv),
        SUCCESS
    );
    let original_ttlv_handle = reference_handle(original_ttlv, Kind::Structure)
        .expect("the message TTLV copy is an owned generic Structure");
    let HandleValue::Structure(original_tree) = &original_ttlv_handle.value else {
        panic!("the message TTLV accessor returns a Structure");
    };
    let original_view = original_tree.view();
    let root_items = original_view.children();
    assert_eq!(
        root_items
            .iter()
            .map(|item| item.tag().raw())
            .collect::<Vec<_>>(),
        [REQUEST_HEADER_TAG, BATCH_ITEM_TAG]
    );
    let raw_operation = root_items[1].with_value(|batch| match batch {
        ValueView::Structure(batch) => {
            batch.children()[0].with_value(|operation| match operation {
                ValueView::Enumeration(raw) => Some(*raw),
                _ => None,
            })
        }
        _ => None,
    });
    assert_eq!(raw_operation, Some(0xF123_4567));

    kmipkit_request_batch_item_release(batch_item);
    kmipkit_request_header_view_release(header);
    kmipkit_request_message_release(message);
    kmipkit_ttlv_structure_release(original_ttlv);
}

#[test]
fn invalid_request_message_returns_only_payload_free_validation_metadata() {
    let structure = try_make_handle(Kind::Structure, HandleValue::Structure(Structure::new()))
        .expect("the generic TTLV input handle is allocated");
    let mut message = ptr::dangling_mut();
    let mut validation_error = ptr::null_mut();

    assert_eq!(
        kmipkit_request_message_from_ttlv(structure, &raw mut message, &raw mut validation_error,),
        ERROR_PROTOCOL
    );
    assert!(message.is_null());
    assert!(!validation_error.is_null());

    let mut kind = u32::MAX;
    assert_eq!(
        kmipkit_message_validation_error_kind(validation_error, &raw mut kind),
        SUCCESS
    );
    assert_eq!(kind, 1, "missing required field is the stable kind value 1");
    kmipkit_message_validation_error_release(validation_error);
}

#[test]
fn validation_error_exposes_optional_numeric_indexes_without_payload() {
    let structure = try_make_handle(
        Kind::Structure,
        HandleValue::Structure(request_message_missing_multi_item_id()),
    )
    .expect("the generic TTLV input handle is allocated");
    let mut message = ptr::null_mut();
    let mut validation_error = ptr::null_mut();

    assert_eq!(
        kmipkit_request_message_from_ttlv(structure, &raw mut message, &raw mut validation_error,),
        ERROR_PROTOCOL
    );
    assert!(message.is_null());
    assert!(!validation_error.is_null());

    let mut has_structure_index = u8::MAX;
    let mut structure_index = u64::MAX;
    assert_eq!(
        kmipkit_message_validation_error_structure_index(
            validation_error,
            &raw mut has_structure_index,
            &raw mut structure_index,
        ),
        SUCCESS
    );
    assert_eq!(has_structure_index, 1);
    assert_eq!(structure_index, 2);

    let mut has_field_index = u8::MAX;
    let mut field_index = u64::MAX;
    assert_eq!(
        kmipkit_message_validation_error_field_index(
            validation_error,
            &raw mut has_field_index,
            &raw mut field_index,
        ),
        SUCCESS
    );
    assert_eq!(has_field_index, 0);
    assert_eq!(field_index, 0);

    kmipkit_message_validation_error_release(validation_error);
}

#[test]
fn ffi_output_region_validator_rejects_overlapping_slots() {
    let mut output = 0_u64;
    let shared = (&raw mut output).cast::<u8>().cast_const();

    assert_eq!(
        validate_output_regions(&[
            (shared, std::mem::size_of::<u64>()),
            (shared, std::mem::size_of::<u32>()),
        ]),
        Err(ERROR_INVALID_INPUT)
    );
}

#[test]
fn request_message_constructor_rejects_aliased_output_slots_before_consuming_input() {
    let structure = try_make_handle(
        Kind::Structure,
        HandleValue::Structure(valid_request_message_tree()),
    )
    .expect("the generic TTLV input handle is allocated");
    let mut output: *mut crate::kmipkit_request_message_t = ptr::null_mut();
    let output_error = (&raw mut output).cast();

    assert_eq!(
        kmipkit_request_message_from_ttlv(structure, &raw mut output, output_error),
        ERROR_INVALID_INPUT
    );
    assert!(output.is_null());
    assert!(reference_handle(structure, Kind::Structure).is_ok());
    kmipkit_ttlv_structure_release(structure);
}
