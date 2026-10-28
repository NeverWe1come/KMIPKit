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

#[test]
fn typed_get_attribute_list_response_preserves_unknown_tags_and_duplicate_name_order() {
    let name_reference = ttlv_structure(
        0x0042_013b,
        [
            ttlv_item(0x0042_009d, 0x07, b"vendor"),
            ttlv_item(0x0042_000a, 0x07, b"Opaque.Name"),
        ],
    );
    let success = successful_cryptographic_item(
        ClientRequest::get_attribute_list(GetAttributeListRequest::new(None)),
        0x0c,
        b"attribute-list-typed-success",
        vec![
            ttlv_item(0x0042_0094, 0x07, b"object-7"),
            ttlv_enumeration(0x0042_013b, 0x0054_0001),
            name_reference.clone(),
            name_reference,
        ],
    );

    let mut present = 0;
    let mut identifier = empty_unique_identifier_record();
    let mut reference_count = 0;
    assert_eq!(
        client_abi::kmipkit_client_batch_item_response_get_attribute_list(
            success,
            &raw mut present,
            &raw mut identifier,
            ptr::null_mut(),
            0,
            &raw mut reference_count,
        ),
        SUCCESS
    );
    assert_eq!(present, 1);
    assert_eq!(identifier.kind, crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING);
    assert_eq!(reference_count, 3);
    assert_eq!(
        unsafe { std::slice::from_raw_parts(identifier.text_data, identifier.text_length as usize) },
        b"object-7"
    );

    let mut references = vec![empty_attribute_reference_record(); reference_count as usize];
    assert_eq!(
        client_abi::kmipkit_client_batch_item_response_get_attribute_list(
            success,
            &raw mut present,
            &raw mut identifier,
            references.as_mut_ptr(),
            references.len() as u64,
            &raw mut reference_count,
        ),
        SUCCESS
    );
    assert_eq!(present, 1);
    assert_eq!(references[0].kind, crate::KMIPKIT_ATTRIBUTE_REFERENCE_KIND_TAG);
    assert_eq!(references[0].tag_value, 0x0054_0001);
    assert_eq!(references[1].kind, crate::KMIPKIT_ATTRIBUTE_REFERENCE_KIND_NAME);
    assert_eq!(span_bytes(references[1].vendor_identification), b"vendor".to_vec());
    assert_eq!(
        span_bytes(references[1].attribute_name),
        b"Opaque.Name".to_vec()
    );
    assert_eq!(span_bytes(references[2].vendor_identification), b"vendor".to_vec());
    assert_eq!(
        span_bytes(references[2].attribute_name),
        b"Opaque.Name".to_vec()
    );
    kmipkit_client_batch_item_response_release(success);
}

fn empty_unique_identifier_record() -> kmipkit_unique_identifier_v1_t {
    kmipkit_unique_identifier_v1_t {
        struct_size: std::mem::size_of::<kmipkit_unique_identifier_v1_t>() as u32,
        abi_version: 1,
        kind: 0,
        reserved: 0,
        text_data: ptr::null(),
        text_length: 0,
        enumeration_value: 0,
        integer_value: 0,
    }
}

fn empty_attribute_reference_record() -> kmipkit_attribute_reference_v1_t {
    kmipkit_attribute_reference_v1_t {
        struct_size: std::mem::size_of::<kmipkit_attribute_reference_v1_t>() as u32,
        abi_version: 1,
        kind: 0,
        reserved: 0,
        tag_value: 0,
        reserved2: 0,
        vendor_identification: byte_span(&[]),
        attribute_name: byte_span(&[]),
    }
}

fn span_bytes(span: kmipkit_byte_span_v1_t) -> Vec<u8> {
    if span.length == 0 {
        return Vec::new();
    }
    // SAFETY: response spans borrow from the live response item in this test.
    unsafe { std::slice::from_raw_parts(span.data, span.length as usize).to_vec() }
}
