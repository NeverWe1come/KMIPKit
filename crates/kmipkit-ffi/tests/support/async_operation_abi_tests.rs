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
