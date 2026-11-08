use super::*;

#[test]
fn cancel_echo_is_exposed_only_for_success_with_a_validated_result() {
    assert!(cancel_response_echo_is_exposed(0, true));
    assert!(!cancel_response_echo_is_exposed(1, false));
    assert!(!cancel_response_echo_is_exposed(0, false));
}

#[test]
fn standalone_poll_request_exposes_its_owned_correlation_bytes() {
    let expected_correlation = [0x00, 0xff, 0x80, 0x13, 0x00];
    let mut request = ptr::null_mut();
    assert_eq!(
        client_abi::kmipkit_poll_request_create(
            expected_correlation.as_ptr(),
            u64::try_from(expected_correlation.len()).expect("correlation length fits ABI"),
            &raw mut request,
        ),
        SUCCESS
    );
    assert!(!request.is_null());

    let mut observed = kmipkit_byte_span_v1_t {
        data: ptr::null(),
        length: u64::MAX,
    };
    assert_eq!(
        client_abi::kmipkit_poll_request_asynchronous_correlation_value(request, &raw mut observed,),
        SUCCESS
    );
    assert_eq!(observed.length, expected_correlation.len() as u64);
    assert!(!observed.data.is_null());
    // SAFETY: the borrowed span remains valid while the live Poll request handle is retained.
    let observed_bytes =
        unsafe { std::slice::from_raw_parts(observed.data, expected_correlation.len()) };
    assert_eq!(observed_bytes, expected_correlation);

    client_abi::kmipkit_poll_request_release(request);
}

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
    let mut typed_present = u8::MAX;
    let mut typed_poll = kmipkit_client_async_operation_response_v1_t::default();
    assert_eq!(
        // SAFETY: `outcome` is live and the output values are separate writable locals.
        unsafe {
            kmipkit_client_async_operation_outcome_poll_response_v1(
                outcome,
                &raw mut typed_present,
                &raw mut typed_poll,
            )
        },
        SUCCESS
    );
    assert_eq!(typed_present, 1);
    assert_eq!(typed_poll.abi_version, 1);
    assert_eq!(typed_poll.operation, 0x1a);
    assert_eq!(typed_poll.is_pending, 1);
    assert_eq!(typed_poll.has_asynchronous_correlation_value, 1);
    assert_eq!(
        typed_poll.asynchronous_correlation_value.length,
        expected_correlation.len() as u64
    );
    assert_eq!(typed_poll.has_response_payload, 0);
    // SAFETY: the borrowed correlation span is backed by the live outcome handle.
    assert_eq!(
        unsafe {
            std::slice::from_raw_parts(
                typed_poll.asynchronous_correlation_value.data,
                expected_correlation.len(),
            )
        },
        expected_correlation
    );
    assert_c_consumer_typed_response(outcome, 0x1a, 2, 1, 0, 0, &expected_correlation, 0, 0);
    let mut typed_cancel = kmipkit_client_async_operation_response_v1_t::default();
    assert_eq!(
        // SAFETY: `outcome` is live and the output values are separate writable locals.
        unsafe {
            kmipkit_client_async_operation_outcome_cancel_response_v1(
                outcome,
                &raw mut typed_present,
                &raw mut typed_cancel,
            )
        },
        SUCCESS
    );
    assert_eq!(typed_present, 0);
    assert_eq!(typed_cancel.abi_version, 0);
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

#[test]
fn query_async_requests_executes_once_and_has_no_operation_correlation_value() {
    let first_correlation = [0x00, 0xff, 0x80];
    let response = query_async_requests_response(&first_correlation, 0);
    let (client, peer, exchange_count) = client_with_responses(vec![response]);
    let client_handle = make_handle(
        Kind::Client,
        HandleValue::Client(Box::new(Mutex::new(client))),
    );
    let limits_handle = make_handle(
        Kind::CodecLimits,
        HandleValue::CodecLimits(Arc::new(CodecLimits::defaults())),
    );
    let second_correlation = [0x13, 0x00];
    let correlation_values = [first_correlation.as_slice(), second_correlation.as_slice()];
    let correlation_spans = correlation_values.map(|value| crate::kmipkit_byte_span_v1_t {
        data: value.as_ptr(),
        length: u64::try_from(value.len()).expect("correlation length fits ABI"),
    });
    let operations = [0x1a, 0xf123_4567];
    let mut outcome = ptr::null_mut();

    let status = kmipkit_client_execute_query_async_requests(
        client_handle,
        1,
        correlation_spans.as_ptr(),
        u64::try_from(correlation_spans.len()).expect("span count fits ABI"),
        1,
        operations.as_ptr(),
        u64::try_from(operations.len()).expect("operation count fits ABI"),
        0,
        0,
        limits_handle,
        &raw mut outcome,
    );
    peer.join().expect("the one-exchange test peer completes");

    assert_eq!(status, SUCCESS);
    assert_eq!(exchange_count.load(Ordering::Relaxed), 1);
    assert!(!outcome.is_null());
    assert_query_async_requests_metadata(outcome, 0, BATCH_OUTCOME_COMPLETED);
    assert_query_async_requests_has_no_correlation(outcome);
    assert_c_consumer_typed_response(outcome, 0x39, 0, 0, 0, 0, &[], 1, 0);

    kmipkit_client_async_operation_outcome_release(outcome);
    kmipkit_codec_limits_release(limits_handle);
    kmipkit_client_release(client_handle);
}

#[test]
fn cancel_and_process_typed_readers_are_called_from_the_compiled_c_consumer() {
    let correlation = [0x00, 0xff, 0x80, 0x23];
    let responses = vec![
        cancel_response(&correlation),
        process_pending_response(&correlation),
    ];
    let (client, peer, exchange_count) = client_with_responses(responses);
    let client_handle = make_handle(
        Kind::Client,
        HandleValue::Client(Box::new(Mutex::new(client))),
    );
    let limits_handle = make_handle(
        Kind::CodecLimits,
        HandleValue::CodecLimits(Arc::new(CodecLimits::defaults())),
    );
    let mut cancel_outcome = ptr::null_mut();
    assert_eq!(
        kmipkit_client_execute_cancel(
            client_handle,
            correlation.as_ptr(),
            u64::try_from(correlation.len()).expect("correlation length fits ABI"),
            limits_handle,
            &raw mut cancel_outcome,
        ),
        SUCCESS
    );
    assert_c_consumer_typed_response(cancel_outcome, 0x19, 0, 0, 1, 1, &correlation, 1, 0);
    kmipkit_client_async_operation_outcome_release(cancel_outcome);

    let mut process_outcome = ptr::null_mut();
    assert_eq!(
        kmipkit_client_execute_process(
            client_handle,
            correlation.as_ptr(),
            u64::try_from(correlation.len()).expect("correlation length fits ABI"),
            1,
            1,
            limits_handle,
            &raw mut process_outcome,
        ),
        SUCCESS
    );
    assert_c_consumer_typed_response(process_outcome, 0x3a, 2, 1, 0, 0, &correlation, 1, 0);
    kmipkit_client_async_operation_outcome_release(process_outcome);

    peer.join().expect("the two-exchange test peer completes");
    assert_eq!(exchange_count.load(Ordering::Relaxed), 2);
    kmipkit_codec_limits_release(limits_handle);
    kmipkit_client_release(client_handle);
}

#[test]
fn failed_cancel_response_does_not_expose_success_only_echo_fields() {
    let correlation = [0x00, 0xff, 0x80, 0x23];
    let response = failed_cancel_response();
    let (client, peer, exchange_count) = client_with_responses(vec![response]);
    let client_handle = make_handle(
        Kind::Client,
        HandleValue::Client(Box::new(Mutex::new(client))),
    );
    let limits_handle = make_handle(
        Kind::CodecLimits,
        HandleValue::CodecLimits(Arc::new(CodecLimits::defaults())),
    );
    let mut outcome = ptr::null_mut();

    assert_eq!(
        kmipkit_client_execute_cancel(
            client_handle,
            correlation.as_ptr(),
            u64::try_from(correlation.len()).expect("correlation length fits ABI"),
            limits_handle,
            &raw mut outcome,
        ),
        SUCCESS
    );
    peer.join().expect("the one-exchange test peer completes");

    assert_eq!(exchange_count.load(Ordering::Relaxed), 1);
    assert!(!outcome.is_null());
    let mut present = u8::MAX;
    let mut response = kmipkit_client_async_operation_response_v1_t::default();
    assert_eq!(
        // SAFETY: `outcome` is live and the output values are separate writable locals.
        unsafe {
            kmipkit_client_async_operation_outcome_cancel_response_v1(
                outcome,
                &raw mut present,
                &raw mut response,
            )
        },
        SUCCESS
    );
    assert_eq!(present, 1);
    assert_eq!(response.result_status, 1);
    assert_eq!(response.has_cancellation_result, 0);
    assert_eq!(response.has_asynchronous_correlation_value, 0);
    assert!(response.asynchronous_correlation_value.data.is_null());
    assert_eq!(response.asynchronous_correlation_value.length, 0);

    kmipkit_client_async_operation_outcome_release(outcome);
    kmipkit_codec_limits_release(limits_handle);
    kmipkit_client_release(client_handle);
}

#[test]
fn query_async_requests_with_options_executes_and_returns_one_outcome() {
    let expected_correlation = [0x00, 0xff, 0x80];
    let response = query_async_requests_response(&expected_correlation, 2);
    let (client, peer, exchange_count) = client_with_responses(vec![response]);
    let client_handle = make_handle(
        Kind::Client,
        HandleValue::Client(Box::new(Mutex::new(client))),
    );
    let limits_handle = make_handle(
        Kind::CodecLimits,
        HandleValue::CodecLimits(Arc::new(CodecLimits::defaults())),
    );
    let other_correlation = [0x13, 0x00];
    let correlation_values = [
        expected_correlation.as_slice(),
        other_correlation.as_slice(),
    ];
    let correlation_spans = correlation_values.map(|value| crate::kmipkit_byte_span_v1_t {
        data: value.as_ptr(),
        length: u64::try_from(value.len()).expect("correlation length fits ABI"),
    });
    let operations = [0x1a, 0xf123_4567];
    let options = request_options(
        timeout(1, 3, 0),
        timeout(1, 4, 0),
        timeout(1, 5, 0),
        timeout(1, 6, 0),
    );
    let mut outcome = ptr::null_mut();

    let status = kmipkit_client_execute_query_async_requests_with_options_v1(
        client_handle,
        1,
        correlation_spans.as_ptr(),
        u64::try_from(correlation_spans.len()).expect("span count fits ABI"),
        1,
        operations.as_ptr(),
        u64::try_from(operations.len()).expect("operation count fits ABI"),
        1,
        1,
        limits_handle,
        &options,
        &raw mut outcome,
    );
    peer.join().expect("the one-exchange peer completes");

    assert_eq!(status, SUCCESS);
    assert_eq!(exchange_count.load(Ordering::Relaxed), 1);
    assert!(!outcome.is_null());
    assert_query_async_requests_metadata(outcome, 2, BATCH_OUTCOME_PENDING);
    assert_exact_correlation(outcome, &expected_correlation);
    assert_c_consumer_typed_response(outcome, 0x39, 2, 1, 0, 0, &expected_correlation, 1, 0);

    kmipkit_client_async_operation_outcome_release(outcome);
    kmipkit_codec_limits_release(limits_handle);
    kmipkit_client_release(client_handle);
}

#[test]
fn query_async_requests_with_options_rejects_invalid_presence_flags_and_counts() {
    let mut outcome = ptr::dangling_mut();
    let invalid_flag = kmipkit_client_execute_query_async_requests_with_options_v1(
        ptr::null_mut(),
        2,
        ptr::null(),
        0,
        0,
        ptr::null(),
        0,
        0,
        0,
        ptr::null_mut(),
        ptr::null(),
        &raw mut outcome,
    );
    assert_eq!(invalid_flag, ERROR_INVALID_INPUT);
    assert!(outcome.is_null());

    let correlation = crate::kmipkit_byte_span_v1_t {
        data: ptr::NonNull::<u8>::dangling().as_ptr(),
        length: 1,
    };
    outcome = ptr::dangling_mut();
    let unexpected_correlation = kmipkit_client_execute_query_async_requests_with_options_v1(
        ptr::null_mut(),
        0,
        &raw const correlation,
        1,
        0,
        ptr::null(),
        0,
        0,
        0,
        ptr::null_mut(),
        ptr::null(),
        &raw mut outcome,
    );
    assert_eq!(unexpected_correlation, ERROR_INVALID_INPUT);
    assert!(outcome.is_null());

    let operation = 0x1a;
    outcome = ptr::dangling_mut();
    let unexpected_operation = kmipkit_client_execute_query_async_requests_with_options_v1(
        ptr::null_mut(),
        0,
        ptr::null(),
        0,
        0,
        &raw const operation,
        1,
        0,
        0,
        ptr::null_mut(),
        ptr::null(),
        &raw mut outcome,
    );
    assert_eq!(unexpected_operation, ERROR_INVALID_INPUT);
    assert!(outcome.is_null());
}

fn query_async_requests_response(correlation: &[u8], result_status: u32) -> Vec<u8> {
    let mut response_fields = vec![
        ttlv_enumeration(0x0042_005c, 0x39),
        ttlv_enumeration(0x0042_007f, result_status),
    ];
    if result_status == 2 {
        response_fields.push(ttlv_item(0x0042_0006, 0x08, correlation));
    }
    response_fields.push(ttlv_structure(
        0x0042_007c,
        [ttlv_structure(
            0x0042_0176,
            [ttlv_item(0x0042_0006, 0x08, correlation)],
        )],
    ));
    ttlv_structure(
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
            ttlv_structure_from_children(0x0042_000f, response_fields),
        ],
    )
}

fn cancel_response(correlation: &[u8]) -> Vec<u8> {
    response_message(
        0x19,
        0,
        None,
        Some(ttlv_structure(
            0x0042_007c,
            [
                ttlv_item(0x0042_0006, 0x08, correlation),
                ttlv_enumeration(0x0042_0012, 1),
            ],
        )),
    )
}

fn failed_cancel_response() -> Vec<u8> {
    ttlv_structure(
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
                    ttlv_enumeration(0x0042_005c, 0x19),
                    ttlv_enumeration(0x0042_007f, 1),
                    ttlv_enumeration(0x0042_007e, 5),
                ],
            ),
        ],
    )
}

fn process_pending_response(correlation: &[u8]) -> Vec<u8> {
    response_message(
        0x3a,
        2,
        Some(correlation),
        Some(ttlv_structure(0x0042_007c, [])),
    )
}

fn response_message(
    operation: u32,
    result_status: u32,
    correlation: Option<&[u8]>,
    payload: Option<Vec<u8>>,
) -> Vec<u8> {
    let operation_item = ttlv_enumeration(0x0042_005c, operation);
    let status_item = ttlv_enumeration(0x0042_007f, result_status);
    let response_item = match (correlation, payload) {
        (Some(correlation), Some(payload)) => ttlv_structure(
            0x0042_000f,
            [
                operation_item,
                status_item,
                ttlv_item(0x0042_0006, 0x08, correlation),
                payload,
            ],
        ),
        (Some(correlation), None) => ttlv_structure(
            0x0042_000f,
            [
                operation_item,
                status_item,
                ttlv_item(0x0042_0006, 0x08, correlation),
            ],
        ),
        (None, Some(payload)) => {
            ttlv_structure(0x0042_000f, [operation_item, status_item, payload])
        }
        (None, None) => ttlv_structure(0x0042_000f, [operation_item, status_item]),
    };
    ttlv_structure(
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
            response_item,
        ],
    )
}

fn ttlv_structure_from_children(tag: u32, children: Vec<Vec<u8>>) -> Vec<u8> {
    ttlv_item(
        tag,
        0x01,
        &children.into_iter().flatten().collect::<Vec<_>>(),
    )
}

fn assert_c_consumer_typed_response(
    outcome: *mut kmipkit_client_async_operation_outcome_t,
    operation: u32,
    result_status: u32,
    is_pending: u32,
    has_cancellation_result: u32,
    cancellation_result: u32,
    correlation: &[u8],
    has_payload: u32,
    payload_member_count: u64,
) {
    let (correlation_data, correlation_length) = if correlation.is_empty() {
        (ptr::null(), 0)
    } else {
        (
            correlation.as_ptr(),
            u64::try_from(correlation.len()).expect("correlation length fits ABI"),
        )
    };
    // SAFETY: C receives the live outcome and a correlation slice retained for this call.
    let result = unsafe {
        super::kmipkit_client_abi_c_consumer_reads_async_response_v1(
            outcome,
            operation,
            result_status,
            is_pending,
            has_cancellation_result,
            cancellation_result,
            correlation_data,
            correlation_length,
            has_payload,
            payload_member_count,
        )
    };
    assert_eq!(
        result, 1,
        "compiled C consumer rejected typed operation {operation:#x}"
    );
}

fn assert_query_async_requests_metadata(
    outcome: *mut crate::kmipkit_client_async_operation_outcome_t,
    expected_status: u32,
    expected_outcome: u32,
) {
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
    assert_eq!(metadata.operation, 0x39);
    assert_eq!(metadata.outcome_kind, expected_outcome);
    assert_eq!(metadata.result_status, expected_status);
}

fn assert_query_async_requests_has_no_correlation(
    outcome: *mut crate::kmipkit_client_async_operation_outcome_t,
) {
    let mut present = u8::MAX;
    let mut correlation = crate::kmipkit_byte_span_v1_t {
        data: std::ptr::dangling(),
        length: u64::MAX,
    };
    assert_eq!(
        kmipkit_client_async_operation_outcome_correlation_value(
            outcome,
            &raw mut present,
            &raw mut correlation,
        ),
        SUCCESS
    );
    assert_eq!(present, 0);
    assert!(correlation.data.is_null());
    assert_eq!(correlation.length, 0);
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
