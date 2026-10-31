use super::*;
use crate::kmipkit_attribute_reference_v1_t;
use kmipkit_client::ClientRequest;
use kmipkit_protocol::GetAttributeListRequest;

#[link(name = "kmipkit_client_abi_c_consumer", kind = "static")]
unsafe extern "C" {
    fn kmipkit_client_abi_c_consumer_build_get_attribute_list_request() -> i32;
    fn kmipkit_client_abi_c_consumer_validate_get_attribute_list_response(
        item: *mut kmipkit_client_batch_item_response_t,
        expected_present: u8,
    ) -> i32;
    fn kmipkit_client_abi_c_consumer_validate_typed_get_attribute_list_response(
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
    let request = || {
        ClientRequest::get_attribute_list(GetAttributeListRequest::new(Some(
            RESPONSE_FIXTURE_UID.to_owned(),
        )))
    };
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
        test_pointer_ffi_call!(
            client_abi::kmipkit_client_batch_item_response_get_attribute_list_payload_ttlv(
                success,
                &raw mut present,
                &raw mut response_handle,
            )
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
        test_pointer_ffi_call!(
            client_abi::kmipkit_client_batch_item_response_get_attribute_list_payload_ttlv(
                failure,
                &raw mut present,
                &raw mut response_ttlv,
            )
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
    let request = || {
        ClientRequest::get_attribute_list(GetAttributeListRequest::new(Some(
            RESPONSE_FIXTURE_UID.to_owned(),
        )))
    };
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
    assert_eq!(
        unsafe {
            kmipkit_client_abi_c_consumer_validate_typed_get_attribute_list_response(failure, 0)
        },
        1
    );
    kmipkit_client_batch_item_response_release(failure);
}

#[test]
fn get_attribute_list_uid_span_remains_readable_while_response_is_open() {
    let success = successful_cryptographic_item(
        ClientRequest::get_attribute_list(GetAttributeListRequest::new(Some(
            RESPONSE_FIXTURE_UID.to_owned(),
        ))),
        0x0c,
        b"attribute-list-uid-span",
        vec![
            ttlv_item(0x0042_0094, 0x07, b"object-7"),
            ttlv_enumeration(0x0042_013b, 0x0054_0001),
        ],
    );
    let mut present = 0;
    let mut identifier = empty_unique_identifier_record();
    let mut count = 0;
    assert_eq!(
        test_pointer_ffi_call!(
            client_abi::kmipkit_client_batch_item_response_get_attribute_list(
                success,
                &raw mut present,
                &raw mut identifier,
                ptr::null_mut(),
                0,
                &raw mut count,
            )
        ),
        SUCCESS
    );
    assert_eq!(present, 1);
    assert_eq!(count, 1);
    let handle = reference_handle(success, Kind::ClientBatchItemResponse)
        .expect("the response handle keeps the typed identifier alive");
    let HandleValue::ClientBatchItemResponse { response, index } = &handle.value else {
        panic!("the response handle contains a batch item response");
    };
    let item = response
        .get(*index)
        .expect("the response item remains owned by the response handle");
    let kmipkit_client::ClientBatchOutcome::GetAttributeList(result) = item.outcome() else {
        panic!("the response contains a typed Get Attribute List outcome");
    };
    let expected_text = result
        .unique_identifier()
        .expect("successful Get Attribute List returns a Unique Identifier");
    assert_eq!(
        identifier.text_data,
        expected_text.as_ptr(),
        "FFI must borrow the UID span directly from the retained response"
    );
    kmipkit_client_batch_item_response_release(success);
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
        ClientRequest::get_attribute_list(GetAttributeListRequest::new(Some(
            RESPONSE_FIXTURE_UID.to_owned(),
        ))),
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
        test_pointer_ffi_call!(
            client_abi::kmipkit_client_batch_item_response_get_attribute_list(
                success,
                &raw mut present,
                &raw mut identifier,
                ptr::null_mut(),
                0,
                &raw mut reference_count,
            )
        ),
        SUCCESS
    );
    assert_eq!(present, 1);
    assert_eq!(
        identifier.kind,
        crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING
    );
    assert_eq!(reference_count, 3);
    assert_eq!(unique_identifier_text(&identifier), b"object-7");

    let mut references = vec![
        empty_attribute_reference_record();
        usize::try_from(reference_count)
            .expect("the test reference count fits usize")
    ];
    assert_eq!(
        test_pointer_ffi_call!(
            client_abi::kmipkit_client_batch_item_response_get_attribute_list(
                success,
                &raw mut present,
                &raw mut identifier,
                references.as_mut_ptr(),
                references.len() as u64,
                &raw mut reference_count,
            )
        ),
        SUCCESS
    );
    assert_eq!(present, 1);
    assert_eq!(
        references[0].kind,
        crate::KMIPKIT_ATTRIBUTE_REFERENCE_KIND_TAG
    );
    assert_eq!(references[0].tag_value, 0x0054_0001);
    assert_attribute_name_reference(&references[1], b"vendor", b"Opaque.Name");
    assert_attribute_name_reference(&references[2], b"vendor", b"Opaque.Name");
    assert_eq!(
        unsafe {
            kmipkit_client_abi_c_consumer_validate_typed_get_attribute_list_response(success, 1)
        },
        1
    );
    kmipkit_client_batch_item_response_release(success);
}

fn assert_attribute_name_reference(
    reference: &kmipkit_attribute_reference_v1_t,
    vendor: &[u8],
    name: &[u8],
) {
    assert_eq!(reference.kind, crate::KMIPKIT_ATTRIBUTE_REFERENCE_KIND_NAME);
    assert_eq!(
        span_bytes(
            reference.vendor_identification_data,
            reference.vendor_identification_length,
        ),
        vendor
    );
    assert_eq!(
        span_bytes(
            reference.attribute_name_data,
            reference.attribute_name_length
        ),
        name
    );
}

fn unique_identifier_text(identifier: &kmipkit_unique_identifier_v1_t) -> Vec<u8> {
    let length =
        usize::try_from(identifier.text_length).expect("the test identifier length fits usize");
    // SAFETY: the ABI record points into the live response item in this test.
    unsafe { std::slice::from_raw_parts(identifier.text_data, length).to_vec() }
}

#[test]
fn typed_get_attribute_list_rejects_short_capacity_and_bad_output_versions() {
    let success = successful_cryptographic_item(
        ClientRequest::get_attribute_list(GetAttributeListRequest::new(Some(
            RESPONSE_FIXTURE_UID.to_owned(),
        ))),
        0x0c,
        b"attribute-list-invalid-output",
        vec![
            ttlv_item(0x0042_0094, 0x07, b"object-7"),
            ttlv_enumeration(0x0042_013b, 0x0054_0001),
            ttlv_enumeration(0x0042_013b, 0x0054_0002),
            ttlv_enumeration(0x0042_013b, 0x0054_0003),
        ],
    );
    let mut present = 0;
    let mut identifier = empty_unique_identifier_record();
    let mut references = [
        empty_attribute_reference_record(),
        empty_attribute_reference_record(),
        empty_attribute_reference_record(),
    ];
    let mut count = 0;
    assert_eq!(
        test_pointer_ffi_call!(
            client_abi::kmipkit_client_batch_item_response_get_attribute_list(
                success,
                &raw mut present,
                &raw mut identifier,
                references.as_mut_ptr(),
                2,
                &raw mut count,
            )
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!(present, 1);
    assert_eq!(count, 3);

    references[1].abi_version = 2;
    assert_eq!(
        test_pointer_ffi_call!(
            client_abi::kmipkit_client_batch_item_response_get_attribute_list(
                success,
                &raw mut present,
                &raw mut identifier,
                references.as_mut_ptr(),
                references.len() as u64,
                &raw mut count,
            )
        ),
        ERROR_INVALID_INPUT
    );

    identifier.abi_version = 2;
    assert_eq!(
        test_pointer_ffi_call!(
            client_abi::kmipkit_client_batch_item_response_get_attribute_list(
                success,
                &raw mut present,
                &raw mut identifier,
                ptr::null_mut(),
                0,
                &raw mut count,
            )
        ),
        ERROR_INVALID_INPUT
    );
    kmipkit_client_batch_item_response_release(success);
}

#[test]
fn typed_get_attribute_list_hides_operation_fields_for_failure() {
    let failure = failed_cryptographic_item(
        ClientRequest::get_attribute_list(GetAttributeListRequest::new(Some(
            RESPONSE_FIXTURE_UID.to_owned(),
        ))),
        0x0c,
        b"attribute-list-typed-failure",
    );
    let mut present = 1;
    let mut identifier = empty_unique_identifier_record();
    let mut count = 1;
    assert_eq!(
        test_pointer_ffi_call!(
            client_abi::kmipkit_client_batch_item_response_get_attribute_list(
                failure,
                &raw mut present,
                &raw mut identifier,
                ptr::null_mut(),
                0,
                &raw mut count,
            )
        ),
        SUCCESS
    );
    assert_eq!(present, 0);
    assert_eq!(count, 0);
    assert_eq!(identifier.kind, 0);
    assert!(identifier.text_data.is_null());
    assert_eq!(identifier.text_length, 0);
    kmipkit_client_batch_item_response_release(failure);
}

fn empty_unique_identifier_record() -> kmipkit_unique_identifier_v1_t {
    kmipkit_unique_identifier_v1_t {
        struct_size: abi_struct_size::<kmipkit_unique_identifier_v1_t>(),
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
        struct_size: abi_struct_size::<kmipkit_attribute_reference_v1_t>(),
        abi_version: 1,
        kind: 0,
        reserved: 0,
        tag_value: 0,
        reserved2: 0,
        vendor_identification_data: ptr::null(),
        vendor_identification_length: 0,
        attribute_name_data: ptr::null(),
        attribute_name_length: 0,
    }
}

fn abi_struct_size<T>() -> u32 {
    u32::try_from(std::mem::size_of::<T>()).expect("ABI test record size fits u32")
}

fn span_bytes(data: *const u8, length: u64) -> Vec<u8> {
    if length == 0 {
        return Vec::new();
    }
    let length = usize::try_from(length).expect("test response span length fits usize");
    // SAFETY: response spans borrow from the live response item in this test.
    unsafe { std::slice::from_raw_parts(data, length).to_vec() }
}
