use super::*;
use kmipkit_client::ClientRequest;
use kmipkit_protocol::DeleteAttributeRequest;

#[link(name = "kmipkit_client_abi_c_consumer", kind = "static")]
// SAFETY: these declarations match the C consumer test helpers compiled by `build.rs`.
unsafe extern "C" {
    fn kmipkit_client_abi_c_consumer_build_delete_attribute(
        out_request: *mut *mut kmipkit_client_batch_item_t,
    ) -> i32;
    fn kmipkit_client_abi_c_consumer_validate_delete_attribute_response(
        response: *mut kmipkit_client_batch_item_response_t,
        expected_present: u8,
    ) -> i32;
}

fn empty_identifier_record() -> kmipkit_unique_identifier_v1_t {
    kmipkit_unique_identifier_v1_t {
        struct_size: u32::try_from(std::mem::size_of::<kmipkit_unique_identifier_v1_t>())
            .expect("identifier record size fits the ABI"),
        abi_version: 1,
        kind: 0,
        reserved: 0,
        text_data: ptr::null(),
        text_length: 0,
        enumeration_value: 0,
        integer_value: 0,
    }
}

#[test]
fn compiled_c_consumer_builds_delete_attribute_with_omitted_selectors() {
    let mut request_handle = ptr::null_mut();
    assert_eq!(
        // SAFETY: the C helper initializes this writable output slot before returning.
        unsafe { kmipkit_client_abi_c_consumer_build_delete_attribute(&raw mut request_handle) },
        1,
        "the C consumer can construct Delete Attribute with all optional selectors omitted"
    );
    assert!(!request_handle.is_null());

    let handle = reference_handle(request_handle, Kind::BatchItem)
        .expect("the C consumer returns a Delete Attribute batch item");
    let HandleValue::BatchItem(batch_item) = &handle.value else {
        panic!("the returned C handle contains a batch item");
    };
    let ClientRequest::DeleteAttribute(request) = batch_item.request() else {
        panic!("the C consumer constructs Delete Attribute");
    };
    assert!(request.unique_identifier().is_none());
    assert!(request.current_attribute().is_none());
    assert!(request.attribute_reference().is_none());
    assert!(
        request
            .to_ttlv_payload()
            .expect("an empty optional-selector request is valid")
            .view()
            .children()
            .is_empty()
    );

    kmipkit_client_batch_item_release(request_handle);
}

#[test]
fn delete_attribute_response_copies_success_identifier_and_hides_failure_payload() {
    let request = || ClientRequest::DeleteAttribute(DeleteAttributeRequest::new(None, None, None));
    let success = successful_cryptographic_item(
        request(),
        0x0f,
        b"delete-attribute-success",
        vec![ttlv_item(0x0042_0094, 0x07, b"deleted-object-1")],
    );
    let mut present = 0;
    let mut identifier = empty_identifier_record();
    assert_eq!(
        client_abi::kmipkit_client_batch_item_response_delete_attribute(
            success,
            &raw mut present,
            &raw mut identifier,
        ),
        SUCCESS
    );
    assert_eq!(present, 1);
    assert_eq!(
        // SAFETY: the ABI borrows response text while `success` remains live.
        unsafe {
            std::slice::from_raw_parts(
                identifier.text_data,
                usize::try_from(identifier.text_length)
                    .expect("fixture identifier length fits usize"),
            )
        },
        b"deleted-object-1"
    );
    assert_eq!(
        // SAFETY: the response handle remains live while the C consumer reads it.
        unsafe { kmipkit_client_abi_c_consumer_validate_delete_attribute_response(success, 1) },
        1
    );
    kmipkit_client_batch_item_response_release(success);

    let failure = failed_cryptographic_item(request(), 0x0f, b"delete-attribute-failure");
    present = 9;
    identifier = empty_identifier_record();
    assert_eq!(
        client_abi::kmipkit_client_batch_item_response_delete_attribute(
            failure,
            &raw mut present,
            &raw mut identifier,
        ),
        SUCCESS
    );
    assert_eq!(present, 0);
    assert_eq!(identifier.kind, 0);
    assert!(identifier.text_data.is_null());
    assert_eq!(identifier.text_length, 0);
    assert_eq!(
        // SAFETY: the failure response remains live while the C consumer checks redaction.
        unsafe { kmipkit_client_abi_c_consumer_validate_delete_attribute_response(failure, 0) },
        1
    );
    kmipkit_client_batch_item_response_release(failure);
}
