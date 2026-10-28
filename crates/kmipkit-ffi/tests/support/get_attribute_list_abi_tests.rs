use super::*;
use kmipkit_client::ClientRequest;
use kmipkit_protocol::GetAttributeListRequest;

#[link(name = "kmipkit_client_abi_c_consumer", kind = "static")]
unsafe extern "C" {
    fn kmipkit_client_abi_c_consumer_build_get_attribute_list_request() -> i32;
    fn kmipkit_client_abi_c_consumer_validate_get_attribute_list_response(
        item: *mut kmipkit_client_batch_item_response_t,
        expected_present: u8,
    ) -> i32;
}

#[test]
fn get_attribute_list_constructor_preserves_omitted_identifier() {
    let mut item = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_item_get_attribute_list(ptr::null(), &raw mut item),
        SUCCESS
    );
    assert!(!item.is_null());

    let handle = reference_handle(item, Kind::BatchItem)
        .expect("the Get Attribute List constructor returns a valid batch item");
    let HandleValue::BatchItem(batch_item) = &handle.value else {
        panic!("the constructor returns a batch item");
    };
    let ClientRequest::GetAttributeList(request) = batch_item.request() else {
        panic!("the batch item contains Get Attribute List");
    };
    assert!(
        request
            .to_ttlv_payload()
            .expect("the request is valid")
            .view()
            .children()
            .is_empty()
    );
    kmipkit_client_batch_item_release(item);
}

#[test]
fn get_attribute_list_constructor_preserves_text_identifier() {
    let identifier_text = b"object-7";
    let identifier = import_test_identifier(identifier_text);
    let mut item = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_item_get_attribute_list(&identifier, &raw mut item),
        SUCCESS
    );
    let handle = reference_handle(item, Kind::BatchItem)
        .expect("the constructor returns a valid batch item");
    let HandleValue::BatchItem(batch_item) = &handle.value else {
        panic!("the constructor returns a batch item");
    };
    let ClientRequest::GetAttributeList(request) = batch_item.request() else {
        panic!("the batch item contains Get Attribute List");
    };
    let payload = request.to_ttlv_payload().expect("the request is valid");
    let payload_view = payload.view();
    let children = payload_view.children();
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].tag().raw(), 0x0042_0094);
    assert!(children[0].with_value(|value| matches!(
        value,
        kmipkit_ttlv::ValueView::TextString(text) if text.as_bytes() == identifier_text
    )));
    kmipkit_client_batch_item_release(item);
}

#[test]
fn get_attribute_list_response_ttlv_is_success_only_and_c_owned() {
    let request = || ClientRequest::get_attribute_list(GetAttributeListRequest::new(None));
    let success = successful_cryptographic_item(
        request(),
        0x0c,
        b"attribute-list-success",
        vec![
            ttlv_item(0x0042_0094, 0x07, b"object-7"),
            ttlv_enumeration(0x0042_013b, 0x0042_002f),
        ],
    );
    let mut present = 0;
    let mut response_handle = ptr::null_mut();
    assert_eq!(
        client_abi::kmipkit_client_batch_item_response_get_attribute_list_payload_ttlv(
            success,
            &raw mut present,
            &raw mut response_handle,
        ),
        SUCCESS
    );
    assert_eq!(present, 1);
    let response = reference_handle(response_handle, Kind::Structure)
        .expect("the response accessor returns an owned TTLV structure");
    let HandleValue::Structure(response_structure) = &response.value else {
        panic!("the response accessor returns a structure");
    };
    assert_eq!(response_structure.view().children().len(), 2);
    kmipkit_ttlv_structure_release(response_handle);
    kmipkit_client_batch_item_response_release(success);

    let failure = failed_cryptographic_item(request(), 0x0c, b"attribute-list-failure");
    let mut present = 9;
    let mut response_ttlv = ptr::dangling_mut();
    assert_eq!(
        client_abi::kmipkit_client_batch_item_response_get_attribute_list_payload_ttlv(
            failure,
            &raw mut present,
            &raw mut response_ttlv,
        ),
        SUCCESS
    );
    assert_eq!(present, 0);
    assert!(response_ttlv.is_null());
    kmipkit_client_batch_item_response_release(failure);
}

#[test]
fn compiled_c_consumer_builds_get_attribute_list() {
    assert_eq!(
        unsafe { kmipkit_client_abi_c_consumer_build_get_attribute_list_request() },
        1
    );
}

#[test]
fn compiled_c_consumer_reads_successful_response_payload_and_hides_failure_payload() {
    let request = || ClientRequest::get_attribute_list(GetAttributeListRequest::new(None));
    let success = successful_cryptographic_item(
        request(),
        0x0c,
        b"attribute-list-c-success",
        vec![
            ttlv_item(0x0042_0094, 0x07, b"object-7"),
            ttlv_enumeration(0x0042_013b, 0x0042_002f),
        ],
    );
    assert_eq!(
        unsafe { kmipkit_client_abi_c_consumer_validate_get_attribute_list_response(success, 1) },
        1
    );
    kmipkit_client_batch_item_response_release(success);

    let failure = failed_cryptographic_item(request(), 0x0c, b"attribute-list-c-failure");
    assert_eq!(
        unsafe { kmipkit_client_abi_c_consumer_validate_get_attribute_list_response(failure, 0) },
        1
    );
    kmipkit_client_batch_item_response_release(failure);
}
