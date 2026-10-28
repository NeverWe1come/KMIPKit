use super::*;
use crate::kmipkit_attribute_reference_v1_t;
use kmipkit_client::ClientRequest;
use kmipkit_protocol::{AttributeReference, GetAttributesRequest};

#[link(name = "kmipkit_client_abi_c_consumer", kind = "static")]
unsafe extern "C" {
    fn kmipkit_client_abi_c_consumer_build_get_attributes_request() -> i32;
    fn kmipkit_client_abi_c_consumer_validate_get_attributes_response(
        item: *mut kmipkit_client_batch_item_response_t,
        expected_present: u8,
    ) -> i32;
}

#[test]
fn get_attributes_constructor_preserves_reference_order_and_optional_identifier() {
    let name_vendor = b"KMIPKit";
    let name_attribute = b"Comment";
    let mut references = [
        kmipkit_attribute_reference_v1_t {
            struct_size: std::mem::size_of::<kmipkit_attribute_reference_v1_t>() as u32,
            abi_version: 1,
            kind: 1,
            reserved: 0,
            reserved2: 0,
            tag_value: 0x0042_002f,
            vendor_identification_data: ptr::null(),
            vendor_identification_length: 0,
            attribute_name_data: ptr::null(),
            attribute_name_length: 0,
        },
        kmipkit_attribute_reference_v1_t {
            struct_size: std::mem::size_of::<kmipkit_attribute_reference_v1_t>() as u32,
            abi_version: 1,
            kind: 2,
            reserved: 0,
            reserved2: 0,
            tag_value: 0,
            vendor_identification_data: name_vendor.as_ptr(),
            vendor_identification_length: name_vendor.len() as u64,
            attribute_name_data: name_attribute.as_ptr(),
            attribute_name_length: name_attribute.len() as u64,
        },
    ];
    let mut item = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_item_get_attributes(
            ptr::null(),
            references.as_mut_ptr(),
            references.len() as u64,
            &raw mut item,
        ),
        SUCCESS
    );

    let handle = reference_handle(item, Kind::BatchItem)
        .expect("Get Attributes returns a batch-item handle");
    let HandleValue::BatchItem(batch_item) = &handle.value else {
        panic!("the constructor returns a typed batch item");
    };
    let ClientRequest::GetAttributes(request) = batch_item.request() else {
        panic!("the batch item contains Get Attributes");
    };
    let payload = request.to_ttlv_payload().expect("the request is valid");
    let payload_view = payload.view();
    let fields = payload_view.children();
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].tag().raw(), 0x0042_013b);
    assert_eq!(fields[1].tag().raw(), 0x0042_013b);
    assert_eq!(
        fields[0].with_value(|value| matches!(
            value,
            kmipkit_ttlv::ValueView::Enumeration(tag) if *tag == 0x0042_002f
        )),
        true
    );
    kmipkit_client_batch_item_release(item);

    let duplicate_record = || kmipkit_attribute_reference_v1_t {
        struct_size: std::mem::size_of::<kmipkit_attribute_reference_v1_t>() as u32,
        abi_version: 1,
        kind: 1,
        reserved: 0,
        reserved2: 0,
        tag_value: 0x0042_002f,
        vendor_identification_data: ptr::null(),
        vendor_identification_length: 0,
        attribute_name_data: ptr::null(),
        attribute_name_length: 0,
    };
    let mut duplicate_records = [duplicate_record(), duplicate_record()];
    let mut duplicate_item = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_item_get_attributes(
            ptr::null(),
            duplicate_records.as_mut_ptr(),
            duplicate_records.len() as u64,
            &raw mut duplicate_item,
        ),
        ERROR_INVALID_INPUT,
        "§6.1.20 forbids duplicate identical Attribute References"
    );
    assert!(duplicate_item.is_null());
}

#[test]
fn get_attributes_response_returns_owned_attributes_and_hides_failure_fields() {
    let request = || {
        ClientRequest::get_attributes(
            GetAttributesRequest::try_new(None, [AttributeReference::tag(0x0042_002f)])
                .expect("the request reference is unique"),
        )
    };
    let success = successful_cryptographic_item(
        request(),
        0x0b,
        b"get-attributes-success",
        vec![
            ttlv_item(0x0042_0094, 0x07, b"object-7"),
            ttlv_structure(
                0x0042_0125,
                [
                    ttlv_item(0x0042_002f, 0x09, &7_i64.to_be_bytes()),
                    ttlv_item(0x0042_002f, 0x09, &19_i64.to_be_bytes()),
                ],
            ),
        ],
    );
    let mut present = 0;
    let mut identifier = kmipkit_unique_identifier_v1_t {
        struct_size: std::mem::size_of::<kmipkit_unique_identifier_v1_t>() as u32,
        abi_version: 1,
        kind: 0,
        reserved: 0,
        enumeration_value: 0,
        integer_value: 0,
        text_data: ptr::null(),
        text_length: 0,
    };
    let mut attributes = ptr::null_mut();
    assert_eq!(
        client_abi::kmipkit_client_batch_item_response_get_attributes(
            success,
            &raw mut present,
            &raw mut identifier,
            &raw mut attributes,
        ),
        SUCCESS
    );
    assert_eq!(present, 1);
    assert_eq!(identifier.kind, 1);
    assert_eq!(
        unsafe {
            std::slice::from_raw_parts(identifier.text_data, identifier.text_length as usize)
        },
        b"object-7"
    );
    let attributes_handle = reference_handle(attributes, Kind::Structure)
        .expect("the successful result returns caller-owned generic TTLV");
    let HandleValue::Structure(structure) = &attributes_handle.value else {
        panic!("the successful result contains an Attributes structure");
    };
    assert_eq!(structure.view().children().len(), 2);
    assert_eq!(
        unsafe { kmipkit_client_abi_c_consumer_validate_get_attributes_response(success, 1) },
        1
    );
    kmipkit_ttlv_structure_release(attributes);
    kmipkit_client_batch_item_response_release(success);

    let failure = failed_cryptographic_item(request(), 0x0b, b"get-attributes-failure");
    let mut present = 9;
    let mut identifier = kmipkit_unique_identifier_v1_t {
        struct_size: std::mem::size_of::<kmipkit_unique_identifier_v1_t>() as u32,
        abi_version: 1,
        kind: 9,
        reserved: 0,
        enumeration_value: 0,
        integer_value: 0,
        text_data: ptr::null(),
        text_length: 0,
    };
    let mut attributes = ptr::dangling_mut();
    assert_eq!(
        client_abi::kmipkit_client_batch_item_response_get_attributes(
            failure,
            &raw mut present,
            &raw mut identifier,
            &raw mut attributes,
        ),
        SUCCESS
    );
    assert_eq!(present, 0);
    assert!(attributes.is_null());
    assert_eq!(identifier.kind, 0);
    assert_eq!(
        unsafe { kmipkit_client_abi_c_consumer_validate_get_attributes_response(failure, 0) },
        1
    );
    kmipkit_client_batch_item_response_release(failure);
}

#[test]
fn get_attributes_response_preserves_success_with_identifier_only_payload() {
    let request = || {
        ClientRequest::get_attributes(
            GetAttributesRequest::try_new(None, [AttributeReference::tag(0x0042_002f)])
                .expect("the request reference is unique"),
        )
    };
    let success = successful_cryptographic_item(
        request(),
        0x0b,
        b"get-attributes-no-match",
        vec![ttlv_item(0x0042_0094, 0x07, b"object-7")],
    );
    let mut present = 0;
    let mut identifier = kmipkit_unique_identifier_v1_t {
        struct_size: std::mem::size_of::<kmipkit_unique_identifier_v1_t>() as u32,
        abi_version: 1,
        kind: 0,
        reserved: 0,
        enumeration_value: 0,
        integer_value: 0,
        text_data: ptr::null(),
        text_length: 0,
    };
    let mut attributes = ptr::dangling_mut();

    assert_eq!(
        client_abi::kmipkit_client_batch_item_response_get_attributes(
            success,
            &raw mut present,
            &raw mut identifier,
            &raw mut attributes,
        ),
        SUCCESS
    );
    assert_eq!(present, 1, "the operation completed successfully");
    assert_eq!(
        identifier.kind,
        crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING
    );
    assert_eq!(
        unsafe {
            std::slice::from_raw_parts(identifier.text_data, identifier.text_length as usize)
        },
        b"object-7"
    );
    assert!(
        attributes.is_null(),
        "an omitted wire Attributes field must not be synthesized"
    );
    kmipkit_client_batch_item_response_release(success);
}

#[test]
fn compiled_c_consumer_constructs_get_attributes_from_multiple_reference_forms() {
    assert_eq!(
        unsafe { kmipkit_client_abi_c_consumer_build_get_attributes_request() },
        1
    );
}
