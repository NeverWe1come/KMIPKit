use super::*;
use kmipkit_protocol::SetConstraintsRequest;

#[test]
fn set_constraints_constructor_copies_the_caller_structure() {
    let mut constraints = Structure::new();
    constraints
        .try_push(
            Item::new(
                RawTag::new(0x0054_1234)
                    .expect("test tag is representable")
                    .try_checked()
                    .expect("test tag is valid"),
                Value::integer(17),
            )
            .expect("test item is valid"),
        )
        .expect("test structure accepts the item");
    let source = make_handle(Kind::Structure, HandleValue::Structure(constraints));
    let mut item = ptr::null_mut();

    assert_eq!(
        kmipkit_client_batch_item_set_constraints(source, &raw mut item),
        SUCCESS
    );
    assert!(!item.is_null());
    kmipkit_ttlv_structure_release(source);

    let handle = reference_handle(item, Kind::BatchItem)
        .expect("the returned Set Constraints batch item handle is valid");
    let HandleValue::BatchItem(batch_item) = &handle.value else {
        panic!("the constructor returns a batch item handle");
    };
    let ClientRequest::SetConstraints(request) = batch_item.request() else {
        panic!("the batch item contains Set Constraints");
    };
    assert_eq!(request.constraints().children().len(), 1);
    assert!(format!("{request:?}").contains("[REDACTED]"));

    kmipkit_client_batch_item_release(item);
}

#[test]
fn set_constraints_constructor_rejects_a_missing_structure_and_clears_output() {
    let mut item = ptr::dangling_mut();

    assert_eq!(
        kmipkit_client_batch_item_set_constraints(ptr::null_mut(), &raw mut item),
        ERROR_INVALID_INPUT
    );
    assert!(item.is_null());
}

#[test]
fn set_constraints_ttlv_getter_clears_outputs_for_an_invalid_item() {
    let mut present = 9;
    let mut response_ttlv = ptr::dangling_mut();

    assert_eq!(
        client_abi::kmipkit_client_batch_item_response_set_constraints_ttlv(
            ptr::null_mut(),
            &raw mut present,
            &raw mut response_ttlv,
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!(present, 0);
    assert!(response_ttlv.is_null());
}

#[test]
fn set_constraints_ttlv_getter_exposes_success_only() {
    let request = || ClientRequest::set_constraints(SetConstraintsRequest::new(Structure::new()));
    let success =
        successful_cryptographic_item(request(), 0x37, b"set-constraints-success", Vec::new());
    let failed = failed_cryptographic_item(request(), 0x37, b"set-constraints-failure");

    for (item, expected_present) in [(success, 1), (failed, 0)] {
        let mut present = 9;
        let mut response_ttlv = ptr::null_mut();
        assert_eq!(
            client_abi::kmipkit_client_batch_item_response_set_constraints_ttlv(
                item,
                &raw mut present,
                &raw mut response_ttlv,
            ),
            SUCCESS
        );
        assert_eq!(present, expected_present);
        assert_eq!(!response_ttlv.is_null(), expected_present == 1);
        if !response_ttlv.is_null() {
            kmipkit_ttlv_structure_release(response_ttlv);
        }
        kmipkit_client_batch_item_response_release(item);
    }
}

#[link(name = "kmipkit_client_abi_c_consumer", kind = "static")]
unsafe extern "C" {
    fn kmipkit_client_abi_c_consumer_build_set_constraints_request(
        out_item: *mut *mut kmipkit_client_batch_item_t,
    ) -> i32;
    fn kmipkit_client_abi_c_consumer_validate_set_constraints_response_ttlv(
        item: *mut kmipkit_client_batch_item_response_t,
        expected_present: u8,
    ) -> i32;
    fn kmipkit_client_abi_c_consumer_validate_set_constraints_failure_metadata(
        item: *mut kmipkit_client_batch_item_response_t,
    ) -> i32;
    fn kmipkit_client_abi_c_consumer_reject_set_constraints_ttlv_output_overlap() -> i32;
}

#[test]
fn compiled_c_consumer_builds_a_set_constraints_request_from_a_caller_structure() {
    let mut item = ptr::null_mut();
    assert_eq!(
        unsafe { kmipkit_client_abi_c_consumer_build_set_constraints_request(&raw mut item) },
        1
    );
    assert!(!item.is_null());

    let handle = reference_handle(item, Kind::BatchItem)
        .expect("the C consumer returned a valid batch item handle");
    let HandleValue::BatchItem(batch_item) = &handle.value else {
        panic!("the C consumer returns a batch item handle");
    };
    let ClientRequest::SetConstraints(request) = batch_item.request() else {
        panic!("the C consumer built Set Constraints");
    };
    assert_eq!(request.constraints().children().len(), 0);
    kmipkit_client_batch_item_release(item);
}

#[test]
fn compiled_c_consumer_reads_owned_success_ttlv_and_hides_failure_ttlv() {
    let request = || ClientRequest::set_constraints(SetConstraintsRequest::new(Structure::new()));
    let success =
        successful_cryptographic_item(request(), 0x37, b"set-constraints-success", Vec::new());
    assert_eq!(
        unsafe { kmipkit_client_abi_c_consumer_validate_set_constraints_response_ttlv(success, 1) },
        1
    );

    let failure = failed_cryptographic_item(request(), 0x37, b"set-constraints-failure");
    assert_eq!(
        unsafe { kmipkit_client_abi_c_consumer_validate_set_constraints_response_ttlv(failure, 0) },
        1
    );

    let correlation = b"set-constraints-pending-correlation";
    let pending_id = b"set-constraints-pending";
    let pending_response = execute_single_abi_response_with_async(
        request(),
        pending_id,
        pending_response_for_operation(0x37, pending_id, correlation),
        Some(1),
    )
    .expect("the explicitly asynchronous Set Constraints batch permits Pending");
    let pending = client_batch_item_response_from(pending_response);
    assert_pending_abi_item_metadata(pending, correlation);
    // SAFETY: the C consumer receives a live item response and releases its owned handle.
    assert_eq!(
        unsafe { kmipkit_client_abi_c_consumer_validate_set_constraints_response_ttlv(pending, 0) },
        1
    );
}

#[test]
fn compiled_c_consumer_reads_set_constraints_failure_metadata_without_ttlv() {
    let request = || ClientRequest::set_constraints(SetConstraintsRequest::new(Structure::new()));
    let failure = failed_cryptographic_item(request(), 0x37, b"set-constraints-failure");
    // SAFETY: the compiled C caller borrows this live Failure item and does not release it.
    assert_eq!(
        unsafe { kmipkit_client_abi_c_consumer_validate_set_constraints_failure_metadata(failure) },
        1
    );
    kmipkit_client_batch_item_response_release(failure);
}

#[test]
fn set_constraints_ttlv_getter_rejects_overlapping_c_outputs_without_writes() {
    // SAFETY: the C consumer deliberately aliases the two writable output ranges to verify the
    // ABI rejects overlap before creating Rust mutable references or writing either output.
    assert_eq!(
        unsafe { kmipkit_client_abi_c_consumer_reject_set_constraints_ttlv_output_overlap() },
        1
    );
}
