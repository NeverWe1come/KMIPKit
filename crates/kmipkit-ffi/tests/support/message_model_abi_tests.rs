use super::*;
use crate::kmipkit_protocol_version_v1_t;

const SHARED_PARITY_FIXTURE: &str =
    include_str!("../../../../tests/fixtures/api/shared-capability-parity.json");

#[link(name = "kmipkit_message_model_abi_c_consumer", kind = "static")]
unsafe extern "C" {
    fn kmipkit_message_model_c_consumer_validate_request(
        structure: *mut crate::kmipkit_ttlv_structure_t,
    ) -> i32;
    fn kmipkit_message_model_c_consumer_validate_response(
        structure: *mut crate::kmipkit_ttlv_structure_t,
    ) -> i32;
}

const REQUEST_HEADER_TAG: u32 = 0x0042_0077;
const REQUEST_PAYLOAD_TAG: u32 = 0x0042_0079;
const RESPONSE_HEADER_TAG: u32 = 0x0042_007A;
const PROTOCOL_VERSION_TAG: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR_TAG: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR_TAG: u32 = 0x0042_006B;
const BATCH_COUNT_TAG: u32 = 0x0042_000D;
const BATCH_ITEM_TAG: u32 = 0x0042_000F;
const OPERATION_TAG: u32 = 0x0042_005C;
const TIME_STAMP_TAG: u32 = 0x0042_0092;
const RESULT_STATUS_TAG: u32 = 0x0042_007F;
const ASYNC_CORRELATION_VALUE_TAG: u32 = 0x0042_0006;
const CRITICALITY_INDICATOR_TAG: u32 = 0x0042_0026;
const MESSAGE_EXTENSION_TAG: u32 = 0x0042_0051;
const VENDOR_EXTENSION_TAG: u32 = 0x0042_009C;
const VENDOR_IDENTIFICATION_TAG: u32 = 0x0042_009D;

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

#[test]
fn typed_message_parity_case_is_shared_with_rust_and_compiled_c_consumers() {
    assert!(
        SHARED_PARITY_FIXTURE
            .contains("\"id\": \"typed-message.ordered-open-values-and-lifetime.v1\"")
    );
}

fn assert_structure_tags(
    handle: *mut crate::kmipkit_ttlv_structure_t,
    expected_tags: &[u32],
) -> Arc<Handle> {
    let structure_handle =
        reference_handle(handle, Kind::Structure).expect("the accessor returns a Structure");
    let HandleValue::Structure(structure) = &structure_handle.value else {
        panic!("the accessor returns a Structure");
    };
    assert_eq!(
        structure
            .view()
            .children()
            .iter()
            .map(|item| item.tag().raw())
            .collect::<Vec<_>>(),
        expected_tags
    );
    structure_handle
}

fn assert_response_extension_is_lossless(
    batch_item: *mut crate::kmipkit_response_batch_item_view_t,
) {
    let mut extension_count = 0_u64;
    assert_eq!(
        kmipkit_response_batch_item_extension_count(batch_item, &raw mut extension_count),
        SUCCESS
    );
    assert_eq!(extension_count, 1);
    let mut extension_view: *mut crate::kmipkit_message_extension_view_t = ptr::null_mut();
    assert_eq!(
        kmipkit_response_batch_item_extension_at(batch_item, 0, &raw mut extension_view),
        SUCCESS
    );
    let mut extension_ttlv: *mut crate::kmipkit_ttlv_structure_t = ptr::null_mut();
    assert_eq!(
        kmipkit_message_extension_view_ttlv(extension_view, &raw mut extension_ttlv),
        SUCCESS
    );
    let extension_handle = assert_structure_tags(
        extension_ttlv,
        &[
            VENDOR_IDENTIFICATION_TAG,
            CRITICALITY_INDICATOR_TAG,
            VENDOR_EXTENSION_TAG,
        ],
    );
    let HandleValue::Structure(extension_tree) = &extension_handle.value else {
        panic!("the extension TTLV accessor returns a Structure");
    };
    let vendor_value = extension_tree.view().children()[2].with_value(|value| match value {
        ValueView::Structure(vendor_extension) => {
            vendor_extension.children()[0].with_value(|value| match value {
                ValueView::ByteString(bytes) => Some(bytes.to_vec()),
                _ => None,
            })
        }
        _ => None,
    });
    assert_eq!(vendor_value, Some(vec![0x00, 0xFE, 0x80]));
    kmipkit_ttlv_structure_release(extension_ttlv);
    kmipkit_message_extension_view_release(extension_view);
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

fn request_message_with_two_items_and_omitted_defaults() -> Structure {
    let version = message_structure([
        message_item(PROTOCOL_VERSION_MAJOR_TAG, Value::integer(2)),
        message_item(PROTOCOL_VERSION_MINOR_TAG, Value::integer(1)),
    ]);
    let header = message_structure([
        message_item(PROTOCOL_VERSION_TAG, Value::structure(version)),
        message_item(BATCH_COUNT_TAG, Value::integer(2)),
    ]);
    let first = message_structure([
        message_item(OPERATION_TAG, Value::enumeration(0xF123_4567)),
        message_item(0x0042_0093, Value::byte_string(vec![0x00, 0xFF, 0x80])),
        message_item(REQUEST_PAYLOAD_TAG, Value::structure(Structure::new())),
    ]);
    let second = message_structure([
        message_item(OPERATION_TAG, Value::enumeration(0x0000_000A)),
        message_item(0x0042_0093, Value::byte_string(vec![0x02, 0x03, 0x04])),
        message_item(REQUEST_PAYLOAD_TAG, Value::structure(Structure::new())),
    ]);
    message_structure([
        message_item(REQUEST_HEADER_TAG, Value::structure(header)),
        message_item(BATCH_ITEM_TAG, Value::structure(first)),
        message_item(BATCH_ITEM_TAG, Value::structure(second)),
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

fn valid_pending_response_message_tree() -> Structure {
    let version = message_structure([
        message_item(PROTOCOL_VERSION_MAJOR_TAG, Value::integer(2)),
        message_item(PROTOCOL_VERSION_MINOR_TAG, Value::integer(1)),
    ]);
    let header = message_structure([
        message_item(PROTOCOL_VERSION_TAG, Value::structure(version)),
        message_item(TIME_STAMP_TAG, Value::date_time(5678)),
        message_item(BATCH_COUNT_TAG, Value::integer(1)),
    ]);
    let extension = message_structure([
        message_item(
            VENDOR_IDENTIFICATION_TAG,
            Value::text_string("Acme".to_owned()),
        ),
        message_item(CRITICALITY_INDICATOR_TAG, Value::boolean(false)),
        message_item(
            VENDOR_EXTENSION_TAG,
            Value::structure(message_structure([message_item(
                0x0054_0001,
                Value::byte_string(vec![0x00, 0xFE, 0x80]),
            )])),
        ),
    ]);
    let batch = message_structure([
        message_item(OPERATION_TAG, Value::enumeration(0x0000_001A)),
        message_item(RESULT_STATUS_TAG, Value::enumeration(2)),
        message_item(
            ASYNC_CORRELATION_VALUE_TAG,
            Value::byte_string(vec![0x00, 0xFF, 0x80]),
        ),
        message_item(MESSAGE_EXTENSION_TAG, Value::structure(extension)),
    ]);
    message_structure([
        message_item(RESPONSE_HEADER_TAG, Value::structure(header)),
        message_item(BATCH_ITEM_TAG, Value::structure(batch)),
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

    let mut header_ttlv: *mut crate::kmipkit_ttlv_structure_t = ptr::null_mut();
    assert_eq!(
        kmipkit_request_header_view_ttlv(header, &raw mut header_ttlv),
        SUCCESS
    );
    let _header_tree = assert_structure_tags(header_ttlv, &[PROTOCOL_VERSION_TAG, BATCH_COUNT_TAG]);
    kmipkit_ttlv_structure_release(header_ttlv);

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
    let mut batch_item_ttlv: *mut crate::kmipkit_ttlv_structure_t = ptr::null_mut();
    assert_eq!(
        kmipkit_request_batch_item_view_ttlv(batch_item, &raw mut batch_item_ttlv),
        SUCCESS
    );
    let _batch_tree = assert_structure_tags(batch_item_ttlv, &[OPERATION_TAG, REQUEST_PAYLOAD_TAG]);
    kmipkit_ttlv_structure_release(batch_item_ttlv);
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
    let original_ttlv_handle =
        assert_structure_tags(original_ttlv, &[REQUEST_HEADER_TAG, BATCH_ITEM_TAG]);
    let HandleValue::Structure(original_tree) = &original_ttlv_handle.value else {
        panic!("the message TTLV accessor returns a Structure");
    };
    let original_view = original_tree.view();
    let root_items = original_view.children();
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
fn request_header_c_abi_exposes_effective_kmip_defaults_without_materializing_fields() {
    let structure = try_make_handle(
        Kind::Structure,
        HandleValue::Structure(request_message_with_two_items_and_omitted_defaults()),
    )
    .expect("the generic TTLV input handle is allocated");
    let mut message = ptr::null_mut();
    let mut validation_error = ptr::null_mut();

    assert_eq!(
        kmipkit_request_message_from_ttlv(structure, &raw mut message, &raw mut validation_error,),
        SUCCESS
    );
    assert!(validation_error.is_null());
    let mut header = ptr::null_mut();
    assert_eq!(
        kmipkit_request_message_header(message, &raw mut header),
        SUCCESS
    );

    let mut asynchronous_indicator = u32::MAX;
    assert_eq!(
        kmipkit_request_header_view_effective_asynchronous_indicator(
            header,
            &raw mut asynchronous_indicator,
        ),
        SUCCESS
    );
    assert_eq!(asynchronous_indicator, 3);

    let mut attestation_capable = u8::MAX;
    assert_eq!(
        kmipkit_request_header_view_effective_attestation_capable_indicator(
            header,
            &raw mut attestation_capable,
        ),
        SUCCESS
    );
    assert_eq!(attestation_capable, 0);

    let mut batch_error_continuation = u32::MAX;
    assert_eq!(
        kmipkit_request_header_view_effective_batch_error_continuation_option(
            header,
            &raw mut batch_error_continuation,
        ),
        SUCCESS
    );
    assert_eq!(batch_error_continuation, 2);

    let mut batch_order = u8::MAX;
    assert_eq!(
        kmipkit_request_header_view_effective_batch_order_option(header, &raw mut batch_order,),
        SUCCESS
    );
    assert_eq!(batch_order, 1);

    let mut header_ttlv: *mut crate::kmipkit_ttlv_structure_t = ptr::null_mut();
    assert_eq!(
        kmipkit_request_header_view_ttlv(header, &raw mut header_ttlv),
        SUCCESS
    );
    let _header_tree = assert_structure_tags(header_ttlv, &[PROTOCOL_VERSION_TAG, BATCH_COUNT_TAG]);
    kmipkit_ttlv_structure_release(header_ttlv);
    kmipkit_request_header_view_release(header);
    kmipkit_request_message_release(message);
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

#[test]
fn response_message_handle_exposes_typed_header_and_raw_result_status() {
    let structure = try_make_handle(
        Kind::Structure,
        HandleValue::Structure(valid_pending_response_message_tree()),
    )
    .expect("the generic TTLV input handle is allocated");
    let mut message: *mut crate::kmipkit_response_message_t = ptr::null_mut();
    let mut validation_error = ptr::null_mut();

    assert_eq!(
        kmipkit_response_message_from_ttlv(structure, &raw mut message, &raw mut validation_error,),
        SUCCESS
    );
    assert!(!message.is_null());
    assert!(validation_error.is_null());

    let mut header: *mut crate::kmipkit_response_header_view_t = ptr::null_mut();
    assert_eq!(
        kmipkit_response_message_header(message, &raw mut header),
        SUCCESS
    );
    let mut version = kmipkit_protocol_version_v1_t { major: 0, minor: 0 };
    assert_eq!(
        kmipkit_response_header_view_protocol_version(header, &raw mut version),
        SUCCESS
    );
    assert_eq!(
        version,
        kmipkit_protocol_version_v1_t { major: 2, minor: 1 }
    );
    let mut header_ttlv: *mut crate::kmipkit_ttlv_structure_t = ptr::null_mut();
    assert_eq!(
        kmipkit_response_header_view_ttlv(header, &raw mut header_ttlv),
        SUCCESS
    );
    let _response_header_tree = assert_structure_tags(
        header_ttlv,
        &[PROTOCOL_VERSION_TAG, TIME_STAMP_TAG, BATCH_COUNT_TAG],
    );
    kmipkit_ttlv_structure_release(header_ttlv);

    let mut time_stamp = 0_i64;
    assert_eq!(
        kmipkit_response_header_view_time_stamp(header, &raw mut time_stamp),
        SUCCESS
    );
    assert_eq!(time_stamp, 5678);

    let mut item_count = 0_u64;
    assert_eq!(
        kmipkit_response_message_batch_item_count(message, &raw mut item_count),
        SUCCESS
    );
    assert_eq!(item_count, 1);

    let mut batch_item: *mut crate::kmipkit_response_batch_item_view_t = ptr::null_mut();
    assert_eq!(
        kmipkit_response_message_batch_item_at(message, 0, &raw mut batch_item),
        SUCCESS
    );
    let mut batch_item_ttlv: *mut crate::kmipkit_ttlv_structure_t = ptr::null_mut();
    assert_eq!(
        kmipkit_response_batch_item_view_ttlv(batch_item, &raw mut batch_item_ttlv),
        SUCCESS
    );
    let _response_batch_tree = assert_structure_tags(
        batch_item_ttlv,
        &[
            OPERATION_TAG,
            RESULT_STATUS_TAG,
            ASYNC_CORRELATION_VALUE_TAG,
            MESSAGE_EXTENSION_TAG,
        ],
    );
    kmipkit_ttlv_structure_release(batch_item_ttlv);
    let mut has_status = u8::MAX;
    let mut raw_status = 0_u32;
    assert_eq!(
        kmipkit_response_batch_item_view_result_status(
            batch_item,
            &raw mut has_status,
            &raw mut raw_status,
        ),
        SUCCESS
    );
    assert_eq!(has_status, 1);
    assert_eq!(raw_status, 2);

    assert_response_extension_is_lossless(batch_item);
    kmipkit_response_batch_item_release(batch_item);
    kmipkit_response_header_view_release(header);
    kmipkit_response_message_release(message);
}

#[test]
fn compiled_c_consumer_uses_request_and_response_message_abi() {
    let request = try_make_handle(
        Kind::Structure,
        HandleValue::Structure(valid_request_message_tree()),
    )
    .expect("the request TTLV handle is allocated");
    let response = try_make_handle(
        Kind::Structure,
        HandleValue::Structure(valid_pending_response_message_tree()),
    )
    .expect("the response TTLV handle is allocated");

    // SAFETY: Both pointers refer to live Structure handles. The compiled C consumer passes each
    // to the corresponding constructor, which consumes the handle exactly once.
    assert_eq!(
        unsafe { kmipkit_message_model_c_consumer_validate_request(request,) },
        0
    );
    // SAFETY: The response handle is live and is consumed once by the response constructor.
    assert_eq!(
        unsafe { kmipkit_message_model_c_consumer_validate_response(response,) },
        0
    );
}
