use super::*;

#[test]
fn poll_execution_clears_output_before_rejecting_an_invalid_correlation_span() {
    let mut output = ptr::dangling_mut::<crate::kmipkit_client_async_operation_outcome_t>();

    let status = kmipkit_client_execute_poll(
        ptr::null_mut(),
        ptr::null(),
        1,
        ptr::null_mut(),
        &mut output,
    );

    assert_eq!(status, ERROR_INVALID_INPUT);
    assert!(output.is_null());
}

#[test]
fn poll_execution_returns_exact_pending_metadata_and_correlation_without_polling_again() {
    let expected_correlation = [0x00, 0xff, 0x80, 0x13, 0x00];
    let response = ttlv_structure(
        0x0042_007b,
        [
            ttlv_structure(
                0x0042_007a,
                [
                    ttlv_structure(
                        0x0042_0069,
                        [ttlv_integer(0x0042_006a, 2), ttlv_integer(0x0042_006b, 1)],
                    ),
                    ttlv_datetime(0x0042_0092, 1),
                    ttlv_integer(0x0042_000d, 1),
                ],
            ),
            ttlv_structure(
                0x0042_000f,
                [
                    ttlv_enumeration(0x0042_005c, 0x1a),
                    ttlv_enumeration(0x0042_007f, 2),
                    ttlv_item(0x0042_0006, 0x08, &expected_correlation),
                ],
            ),
        ],
    );
    let (client, peer) = client_with_one_response(response);
    let client_handle = make_handle(
        Kind::Client,
        HandleValue::Client(Box::new(Mutex::new(client))),
    );
    let limits_handle = make_handle(
        Kind::CodecLimits,
        HandleValue::CodecLimits(Arc::new(CodecLimits::defaults())),
    );
    let mut outcome = ptr::null_mut();

    let status = kmipkit_client_execute_poll(
        client_handle,
        expected_correlation.as_ptr(),
        u64::try_from(expected_correlation.len()).expect("correlation length fits ABI"),
        limits_handle,
        &mut outcome,
    );
    peer.join().expect("the one-exchange test peer completes");

    assert_eq!(status, SUCCESS);
    assert!(!outcome.is_null());
    assert_pending_metadata(outcome);
    assert_exact_correlation(outcome, &expected_correlation);
    assert_response_payload_is_absent(outcome);

    kmipkit_client_async_operation_outcome_release(outcome);
    kmipkit_codec_limits_release(limits_handle);
    kmipkit_client_release(client_handle);
}

#[test]
// OASIS KMIP v2.1 §9.13, Table 418: preserve a non-critical extension as generic TTLV.
fn async_outcome_extension_abi_returns_owned_ordered_generic_ttlv_copy() {
    let correlation = [0x00, 0xff, 0x80, 0x13, 0x00];
    let extension = ttlv_structure(
        0x0042_0051,
        [
            ttlv_item(0x0042_009d, 0x07, b"OpaqueVendor"),
            ttlv_item(0x0042_0026, 0x06, &[0; 8]),
            ttlv_structure(
                0x0042_009c,
                [
                    ttlv_item(0x0054_0173, 0x07, b"opaque-v1"),
                    ttlv_enumeration(0x0054_0174, u32::MAX),
                ],
            ),
        ],
    );
    let response = ttlv_structure(
        0x0042_007b,
        [
            ttlv_structure(
                0x0042_007a,
                [
                    ttlv_structure(
                        0x0042_0069,
                        [ttlv_integer(0x0042_006a, 2), ttlv_integer(0x0042_006b, 1)],
                    ),
                    ttlv_datetime(0x0042_0092, 1),
                    ttlv_integer(0x0042_000d, 1),
                ],
            ),
            ttlv_structure(
                0x0042_000f,
                [
                    ttlv_enumeration(0x0042_005c, 0x1a),
                    ttlv_enumeration(0x0042_007f, 2),
                    ttlv_item(0x0042_0006, 0x08, &correlation),
                    extension,
                ],
            ),
        ],
    );
    let (client, peer) = client_with_one_response(response);
    let client_handle = make_handle(
        Kind::Client,
        HandleValue::Client(Box::new(Mutex::new(client))),
    );
    let limits_handle = make_handle(
        Kind::CodecLimits,
        HandleValue::CodecLimits(Arc::new(CodecLimits::defaults())),
    );
    let mut outcome = ptr::null_mut();

    let status = kmipkit_client_execute_poll(
        client_handle,
        correlation.as_ptr(),
        u64::try_from(correlation.len()).expect("correlation length fits ABI"),
        limits_handle,
        &mut outcome,
    );
    peer.join().expect("the one-exchange test peer completes");
    assert_eq!(status, SUCCESS);
    assert!(!outcome.is_null());

    let mut count = u64::MAX;
    assert_eq!(
        kmipkit_client_async_operation_outcome_extension_count(outcome, &raw mut count),
        SUCCESS
    );
    assert_eq!(count, 1);

    let mut absent = u8::MAX;
    let mut out_of_range = ptr::dangling_mut();
    assert_eq!(
        kmipkit_client_async_operation_outcome_extension_ttlv_at(
            outcome,
            1,
            &raw mut absent,
            &raw mut out_of_range,
        ),
        SUCCESS
    );
    assert_eq!(absent, 0);
    assert!(out_of_range.is_null());

    let mut present = u8::MAX;
    let mut extension_copy = ptr::null_mut();
    assert_eq!(
        kmipkit_client_async_operation_outcome_extension_ttlv_at(
            outcome,
            0,
            &raw mut present,
            &raw mut extension_copy,
        ),
        SUCCESS
    );
    assert_eq!(present, 1);
    assert!(!extension_copy.is_null());
    kmipkit_client_async_operation_outcome_release(outcome);
    kmipkit_codec_limits_release(limits_handle);
    kmipkit_client_release(client_handle);

    assert_async_extension_ttlv_contents(extension_copy);
}

fn assert_async_extension_ttlv_contents(extension_copy: *mut kmipkit_ttlv_structure_t) {
    let mut view = ptr::null_mut();
    assert_eq!(
        kmipkit_ttlv_structure_view(extension_copy, &raw mut view),
        SUCCESS
    );
    kmipkit_ttlv_structure_release(extension_copy);
    let mut item_count = u64::MAX;
    assert_eq!(
        kmipkit_ttlv_structure_view_item_count(view, &raw mut item_count),
        SUCCESS
    );
    assert_eq!(item_count, 3);
    assert_ttlv_text_value(view, 0, 0x0042_009d, b"OpaqueVendor");
    assert_ttlv_boolean_value(view, 1, 0x0042_0026, false);
    let vendor_extension = ttlv_nested_structure(view, 2, 0x0042_009c);
    let mut vendor_extension_count = u64::MAX;
    assert_eq!(
        kmipkit_ttlv_structure_view_item_count(vendor_extension, &raw mut vendor_extension_count,),
        SUCCESS
    );
    assert_eq!(vendor_extension_count, 2);
    assert_ttlv_text_value(vendor_extension, 0, 0x0054_0173, b"opaque-v1");
    assert_ttlv_enumeration_value(vendor_extension, 1, 0x0054_0174, u32::MAX);
    kmipkit_ttlv_structure_view_release(vendor_extension);
    kmipkit_ttlv_structure_view_release(view);
}

fn ttlv_item_with_tag(
    view: *mut kmipkit_ttlv_structure_view_t,
    index: u64,
    expected_tag: u32,
) -> *mut kmipkit_ttlv_item_view_t {
    let mut item = ptr::null_mut();
    assert_eq!(
        kmipkit_ttlv_structure_view_item_at(view, index, &raw mut item),
        SUCCESS
    );
    assert!(!item.is_null());
    let mut tag = ptr::null_mut();
    assert_eq!(kmipkit_ttlv_item_view_tag(item, &raw mut tag), SUCCESS);
    assert!(!tag.is_null());
    let mut actual_tag = u32::MAX;
    assert_eq!(kmipkit_ttlv_tag_value(tag, &raw mut actual_tag), SUCCESS);
    kmipkit_tag_release(tag);
    assert_eq!(actual_tag, expected_tag);
    item
}

fn assert_ttlv_text_value(
    view: *mut kmipkit_ttlv_structure_view_t,
    index: u64,
    expected_tag: u32,
    expected_bytes: &[u8],
) {
    let item = ttlv_item_with_tag(view, index, expected_tag);
    let mut item_type = u8::MAX;
    assert_eq!(
        kmipkit_ttlv_item_view_type(item, &raw mut item_type),
        SUCCESS
    );
    assert_eq!(item_type, 7);
    let mut value = ptr::null_mut();
    assert_eq!(kmipkit_ttlv_item_view_value(item, &raw mut value), SUCCESS);
    let mut length = u64::MAX;
    assert_eq!(
        kmipkit_ttlv_value_view_byte_length(value, &raw mut length),
        SUCCESS
    );
    assert_eq!(
        length,
        u64::try_from(expected_bytes.len()).expect("test length fits u64")
    );
    let mut bytes = Vec::with_capacity(expected_bytes.len());
    for index in 0..length {
        let mut byte = u8::MAX;
        assert_eq!(
            kmipkit_ttlv_value_view_byte_at(value, index, &raw mut byte),
            SUCCESS
        );
        bytes.push(byte);
    }
    assert_eq!(bytes, expected_bytes);
    kmipkit_ttlv_value_view_release(value);
    kmipkit_ttlv_item_view_release(item);
}

fn assert_ttlv_boolean_value(
    view: *mut kmipkit_ttlv_structure_view_t,
    index: u64,
    expected_tag: u32,
    expected_value: bool,
) {
    let item = ttlv_item_with_tag(view, index, expected_tag);
    let mut item_type = u8::MAX;
    assert_eq!(
        kmipkit_ttlv_item_view_type(item, &raw mut item_type),
        SUCCESS
    );
    assert_eq!(item_type, 6);
    let mut value = ptr::null_mut();
    assert_eq!(kmipkit_ttlv_item_view_value(item, &raw mut value), SUCCESS);
    let mut actual = u8::MAX;
    assert_eq!(
        kmipkit_ttlv_value_view_boolean(value, &raw mut actual),
        SUCCESS
    );
    assert_eq!(actual, u8::from(expected_value));
    kmipkit_ttlv_value_view_release(value);
    kmipkit_ttlv_item_view_release(item);
}

fn assert_ttlv_enumeration_value(
    view: *mut kmipkit_ttlv_structure_view_t,
    index: u64,
    expected_tag: u32,
    expected_value: u32,
) {
    let item = ttlv_item_with_tag(view, index, expected_tag);
    let mut item_type = u8::MAX;
    assert_eq!(
        kmipkit_ttlv_item_view_type(item, &raw mut item_type),
        SUCCESS
    );
    assert_eq!(item_type, 5);
    let mut value = ptr::null_mut();
    assert_eq!(kmipkit_ttlv_item_view_value(item, &raw mut value), SUCCESS);
    let mut actual = u32::MAX;
    assert_eq!(
        kmipkit_ttlv_value_view_enumeration(value, &raw mut actual),
        SUCCESS
    );
    assert_eq!(actual, expected_value);
    kmipkit_ttlv_value_view_release(value);
    kmipkit_ttlv_item_view_release(item);
}

fn ttlv_nested_structure(
    view: *mut kmipkit_ttlv_structure_view_t,
    index: u64,
    expected_tag: u32,
) -> *mut kmipkit_ttlv_structure_view_t {
    let item = ttlv_item_with_tag(view, index, expected_tag);
    let mut item_type = u8::MAX;
    assert_eq!(
        kmipkit_ttlv_item_view_type(item, &raw mut item_type),
        SUCCESS
    );
    assert_eq!(item_type, 1);
    let mut value = ptr::null_mut();
    assert_eq!(kmipkit_ttlv_item_view_value(item, &raw mut value), SUCCESS);
    let mut nested = ptr::null_mut();
    assert_eq!(
        kmipkit_ttlv_value_view_structure(value, &raw mut nested),
        SUCCESS
    );
    kmipkit_ttlv_value_view_release(value);
    kmipkit_ttlv_item_view_release(item);
    assert!(!nested.is_null());
    nested
}

#[test]
fn poll_execution_with_options_returns_one_exact_async_outcome() {
    let expected_correlation = [0x00, 0xff, 0x80, 0x13, 0x00];
    let response = ttlv_structure(
        0x0042_007b,
        [
            ttlv_structure(
                0x0042_007a,
                [
                    ttlv_structure(
                        0x0042_0069,
                        [ttlv_integer(0x0042_006a, 2), ttlv_integer(0x0042_006b, 1)],
                    ),
                    ttlv_datetime(0x0042_0092, 1),
                    ttlv_integer(0x0042_000d, 1),
                ],
            ),
            ttlv_structure(
                0x0042_000f,
                [
                    ttlv_enumeration(0x0042_005c, 0x1a),
                    ttlv_enumeration(0x0042_007f, 2),
                    ttlv_item(0x0042_0006, 0x08, &expected_correlation),
                ],
            ),
        ],
    );
    let (client, peer, exchange_count) = client_with_responses(vec![response]);
    let client_handle = make_handle(
        Kind::Client,
        HandleValue::Client(Box::new(Mutex::new(client))),
    );
    let limits_handle = make_handle(
        Kind::CodecLimits,
        HandleValue::CodecLimits(Arc::new(CodecLimits::defaults())),
    );
    let options = request_options(
        timeout(1, 3, 0),
        timeout(1, 4, 0),
        timeout(1, 5, 0),
        timeout(1, 6, 0),
    );
    let mut outcome = ptr::null_mut();

    let status = kmipkit_client_execute_poll_with_options_v1(
        client_handle,
        expected_correlation.as_ptr(),
        u64::try_from(expected_correlation.len()).expect("correlation length fits ABI"),
        limits_handle,
        &options,
        &mut outcome,
    );
    peer.join().expect("the one-exchange TLS peer completes");

    assert_eq!(status, SUCCESS);
    assert_eq!(exchange_count.load(Ordering::Relaxed), 1);
    assert!(!outcome.is_null());
    assert_pending_metadata(outcome);
    assert_exact_correlation(outcome, &expected_correlation);
    assert_response_payload_is_absent(outcome);

    kmipkit_client_async_operation_outcome_release(outcome);
    kmipkit_codec_limits_release(limits_handle);
    kmipkit_client_release(client_handle);
}

fn assert_pending_metadata(outcome: *mut kmipkit_client_async_operation_outcome_t) {
    let mut metadata = kmipkit_client_async_operation_metadata_v1_t {
        struct_size: 0,
        abi_version: 0,
        operation: 0,
        outcome_kind: 0,
        result_status: 0,
        has_result_reason: 0,
        result_reason: 0,
        has_cancellation_result: 0,
        cancellation_result: 0,
        has_result_message: 0,
        reserved: 0,
        result_message_data: ptr::null(),
        result_message_length: 0,
    };
    assert_eq!(
        test_pointer_ffi_call!(kmipkit_client_async_operation_outcome_metadata_v1(
            outcome,
            &mut metadata
        )),
        SUCCESS
    );
    assert_eq!(metadata.operation, 0x1a);
    assert_eq!(metadata.outcome_kind, BATCH_OUTCOME_PENDING);
    assert_eq!(metadata.result_status, 2);
}

fn assert_exact_correlation(
    outcome: *mut kmipkit_client_async_operation_outcome_t,
    expected: &[u8],
) {
    let mut present = 0;
    let mut correlation = kmipkit_byte_span_v1_t {
        data: ptr::null(),
        length: 0,
    };
    assert_eq!(
        kmipkit_client_async_operation_outcome_correlation_value(
            outcome,
            &mut present,
            &mut correlation,
        ),
        SUCCESS
    );
    assert_eq!(present, 1);
    // SAFETY: the live outcome owns the response tree backing this borrowed span.
    assert_eq!(
        unsafe {
            std::slice::from_raw_parts(
                correlation.data,
                usize::try_from(correlation.length).expect("correlation length fits usize"),
            )
        },
        expected
    );
}

fn assert_response_payload_is_absent(outcome: *mut kmipkit_client_async_operation_outcome_t) {
    let mut present = 0;
    let mut payload = ptr::dangling_mut::<kmipkit_ttlv_structure_t>();
    assert_eq!(
        kmipkit_client_async_operation_outcome_response_payload_ttlv(
            outcome,
            &mut present,
            &mut payload,
        ),
        SUCCESS
    );
    assert_eq!(present, 0);
    assert!(payload.is_null());
}
