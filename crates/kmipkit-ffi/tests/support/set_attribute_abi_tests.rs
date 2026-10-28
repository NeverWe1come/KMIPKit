use super::*;
use kmipkit_client::ClientRequest;
use kmipkit_protocol::{NewAttribute, SetAttributeRequest};
use kmipkit_ttlv::{Item, RawTag, Value, ValueView};

#[link(name = "kmipkit_client_abi_c_consumer", kind = "static")]
// SAFETY: these declarations match the C consumer helpers compiled by `build.rs`.
unsafe extern "C" {
    fn kmipkit_client_abi_c_consumer_build_set_attribute_request(
        out_request: *mut *mut kmipkit_client_batch_item_t,
    ) -> i32;
    fn kmipkit_client_abi_c_consumer_validate_set_attribute_response(
        response: *mut kmipkit_client_batch_item_response_t,
        expected_present: u8,
    ) -> i32;
}

fn set_attribute_request() -> ClientRequest {
    let tag = RawTag::new(0x0042_00fd)
        .expect("Comment tag fits the KMIP field")
        .try_checked()
        .expect("Comment is assigned");
    let item = Item::new(tag, Value::text_string("updated comment".to_owned()))
        .expect("attribute Item is valid");
    ClientRequest::SetAttribute(SetAttributeRequest::new(None, NewAttribute::new(item)))
}

#[test]
fn set_attribute_ffi_constructor_preserves_optional_identifier_and_direct_new_item() {
    let attribute_tag = RawTag::new(0x0042_00fd)
        .expect("attribute tag fits")
        .try_checked()
        .expect("attribute tag is assigned");
    let attribute = Item::new(
        attribute_tag,
        Value::text_string("updated comment".to_owned()),
    )
    .expect("New Attribute Item is valid");
    let attribute_handle = try_make_handle(Kind::Item, HandleValue::Item(attribute))
        .expect("attribute handle is registered");
    let mut identifier = kmipkit_unique_identifier_v1_t {
        struct_size: u32::try_from(std::mem::size_of::<kmipkit_unique_identifier_v1_t>())
            .expect("identifier size fits"),
        abi_version: 1,
        kind: crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING,
        reserved: 0,
        text_data: b"object-7".as_ptr(),
        text_length: 8,
        enumeration_value: 0,
        integer_value: 0,
    };
    let mut request_handle = ptr::null_mut();

    assert_eq!(
        client_abi::kmipkit_client_batch_item_set_attribute(
            &raw mut identifier,
            attribute_handle,
            &raw mut request_handle,
        ),
        SUCCESS
    );
    let handle = reference_handle(request_handle, Kind::BatchItem)
        .expect("Set Attribute returns an opaque batch item");
    let HandleValue::BatchItem(batch_item) = &handle.value else {
        panic!("the result handle contains a batch item");
    };
    let ClientRequest::SetAttribute(request) = batch_item.request() else {
        panic!("the constructor creates Set Attribute");
    };
    assert_eq!(request.unique_identifier(), Some("object-7"));
    let payload = request.to_ttlv_payload().expect("payload is valid");
    let payload_view = payload.view();
    let fields = payload_view.children();
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].tag().raw(), 0x0042_0094);
    assert_eq!(fields[1].tag().raw(), 0x0042_013d);
    assert!(fields[1].with_value(|value| match value {
        ValueView::Structure(attribute) => {
            let children = attribute.children();
            children.len() == 1
                && children[0].tag().raw() == 0x0042_00fd
                && children[0].with_value(|value| matches!(value, ValueView::TextString(_)))
        }
        _ => false,
    }));

    kmipkit_client_batch_item_release(request_handle);
    kmipkit_ttlv_item_release(attribute_handle);
}

#[test]
fn compiled_c_consumer_builds_set_attribute_and_preserves_borrowed_input() {
    let mut request_handle = ptr::null_mut();
    assert_eq!(
        // SAFETY: the C helper initializes the writable output slot before returning.
        unsafe {
            kmipkit_client_abi_c_consumer_build_set_attribute_request(&raw mut request_handle)
        },
        1,
        "the C constructor copies New Attribute and leaves caller ownership intact"
    );
    assert!(!request_handle.is_null());
    let handle = reference_handle(request_handle, Kind::BatchItem)
        .expect("the C consumer returns a Set Attribute request");
    let HandleValue::BatchItem(batch_item) = &handle.value else {
        panic!("the request handle contains a batch item");
    };
    let ClientRequest::SetAttribute(request) = batch_item.request() else {
        panic!("the C consumer constructs Set Attribute");
    };
    assert_eq!(request.unique_identifier(), Some("object-7"));
    assert_eq!(request.new_attribute().item().tag().raw(), 0x0042_00fd);
    kmipkit_client_batch_item_release(request_handle);
}

#[test]
fn set_attribute_response_exposes_success_identifier_and_hides_failures() {
    let request = set_attribute_request;
    let success = successful_cryptographic_item(
        request(),
        0x31,
        b"set-attribute-success",
        vec![ttlv_item(0x0042_0094, 0x07, b"object-7")],
    );
    let mut present = 0;
    let mut identifier = empty_identifier_record();
    assert_eq!(
        client_abi::kmipkit_client_batch_item_response_set_attribute(
            success,
            &raw mut present,
            &raw mut identifier,
        ),
        SUCCESS
    );
    assert_eq!(present, 1);
    // SAFETY: the output span borrows from the live response handle.
    assert_eq!(
        unsafe {
            std::slice::from_raw_parts(
                identifier.text_data,
                usize::try_from(identifier.text_length).expect("fixture length fits usize"),
            )
        },
        b"object-7"
    );
    assert_eq!(
        // SAFETY: the C helper reads the typed response while the response handle remains live.
        unsafe { kmipkit_client_abi_c_consumer_validate_set_attribute_response(success, 1) },
        1
    );
    kmipkit_client_batch_item_response_release(success);

    let failure = failed_cryptographic_item(request(), 0x31, b"set-attribute-failure");
    present = 9;
    identifier = empty_identifier_record();
    assert_eq!(
        client_abi::kmipkit_client_batch_item_response_set_attribute(
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
        // SAFETY: the C helper sees a live failed response and verifies absent typed fields.
        unsafe { kmipkit_client_abi_c_consumer_validate_set_attribute_response(failure, 0) },
        1
    );
    kmipkit_client_batch_item_response_release(failure);
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
