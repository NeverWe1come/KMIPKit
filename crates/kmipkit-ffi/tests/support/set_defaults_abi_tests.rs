use super::*;
use kmipkit_client::ClientRequest;
use kmipkit_protocol::SetDefaultsRequest;

#[link(name = "kmipkit_client_abi_c_consumer", kind = "static")]
unsafe extern "C" {
    fn kmipkit_client_abi_c_consumer_build_set_defaults_requests(
        out_omitted: *mut *mut kmipkit_client_batch_item_t,
        out_present_empty: *mut *mut kmipkit_client_batch_item_t,
    ) -> i32;
    fn kmipkit_client_abi_c_consumer_validate_set_defaults_response_ttlv(
        item: *mut kmipkit_client_batch_item_response_t,
        expected_present: u8,
    ) -> i32;
    fn kmipkit_client_abi_c_consumer_validate_set_defaults_failure_metadata(
        item: *mut kmipkit_client_batch_item_response_t,
    ) -> i32;
}

#[test]
fn set_defaults_constructor_preserves_omitted_and_present_empty_information() {
    let mut omitted = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_item_set_defaults(ptr::null_mut(), &raw mut omitted),
        SUCCESS
    );
    assert!(!omitted.is_null());
    let omitted_handle = reference_handle(omitted, Kind::BatchItem)
        .expect("the omitted Set Defaults batch item handle is valid");
    let HandleValue::BatchItem(omitted_item) = &omitted_handle.value else {
        panic!("the omitted constructor returns a batch item");
    };
    let ClientRequest::SetDefaults(omitted_request) = omitted_item.request() else {
        panic!("the omitted batch item contains Set Defaults");
    };
    assert!(omitted_request.defaults_information().is_none());
    kmipkit_client_batch_item_release(omitted);

    let source = make_handle(Kind::Structure, HandleValue::Structure(Structure::new()));
    let mut present_empty = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_item_set_defaults(source, &raw mut present_empty),
        SUCCESS
    );
    kmipkit_ttlv_structure_release(source);
    let present_handle = reference_handle(present_empty, Kind::BatchItem)
        .expect("the present-empty Set Defaults batch item handle is valid");
    let HandleValue::BatchItem(present_item) = &present_handle.value else {
        panic!("the present constructor returns a batch item");
    };
    let ClientRequest::SetDefaults(present_request) = present_item.request() else {
        panic!("the present batch item contains Set Defaults");
    };
    assert!(
        present_request
            .defaults_information()
            .is_some_and(|information| information.children().is_empty())
    );
    kmipkit_client_batch_item_release(present_empty);
}

#[test]
fn set_defaults_constructor_rejects_invalid_structure_and_clears_output() {
    let mut item = ptr::dangling_mut();
    assert_eq!(
        kmipkit_client_batch_item_set_defaults(ptr::dangling_mut(), &raw mut item),
        ERROR_INVALID_INPUT
    );
    assert!(item.is_null());
}

#[test]
fn compiled_c_consumer_preserves_set_defaults_absence_and_empty_structure() {
    let mut omitted = ptr::null_mut();
    let mut present_empty = ptr::null_mut();
    // SAFETY: both pointers address writable output slots for the synchronous C call.
    let status = unsafe {
        kmipkit_client_abi_c_consumer_build_set_defaults_requests(
            &raw mut omitted,
            &raw mut present_empty,
        )
    };
    assert_eq!(status, 1);

    let omitted_handle = reference_handle(omitted, Kind::BatchItem)
        .expect("the compiled C consumer returned an omitted request");
    let HandleValue::BatchItem(omitted_item) = &omitted_handle.value else {
        panic!("the C omitted handle is a batch item");
    };
    let ClientRequest::SetDefaults(omitted_request) = omitted_item.request() else {
        panic!("the C omitted handle contains Set Defaults");
    };
    assert!(omitted_request.defaults_information().is_none());

    let present_handle = reference_handle(present_empty, Kind::BatchItem)
        .expect("the compiled C consumer returned a present-empty request");
    let HandleValue::BatchItem(present_item) = &present_handle.value else {
        panic!("the C present handle is a batch item");
    };
    let ClientRequest::SetDefaults(present_request) = present_item.request() else {
        panic!("the C present handle contains Set Defaults");
    };
    assert!(
        present_request
            .defaults_information()
            .is_some_and(|information| information.children().is_empty())
    );

    kmipkit_client_batch_item_release(omitted);
    kmipkit_client_batch_item_release(present_empty);
}

#[test]
fn set_defaults_response_ttlv_is_success_only_and_c_owned_copy_is_independent() {
    let request = || ClientRequest::set_defaults(SetDefaultsRequest::new());
    let success =
        successful_cryptographic_item(request(), 0x36, b"set-defaults-success", Vec::new());
    // SAFETY: the C consumer owns the copy and releases it after the source response is freed.
    assert_eq!(
        unsafe { kmipkit_client_abi_c_consumer_validate_set_defaults_response_ttlv(success, 1) },
        1
    );

    let failure = failed_cryptographic_item(request(), 0x36, b"set-defaults-failure");
    // SAFETY: the consumer borrows this valid failure response and checks its metadata only.
    assert_eq!(
        unsafe { kmipkit_client_abi_c_consumer_validate_set_defaults_failure_metadata(failure) },
        1
    );
    kmipkit_client_batch_item_response_release(failure);
}

#[test]
fn set_defaults_ttlv_getter_clears_outputs_for_invalid_item() {
    let mut present = 9;
    let mut response_ttlv = ptr::dangling_mut();
    assert_eq!(
        client_abi::kmipkit_client_batch_item_response_set_defaults_ttlv(
            ptr::null_mut(),
            &raw mut present,
            &raw mut response_ttlv,
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!(present, 0);
    assert!(response_ttlv.is_null());
}
