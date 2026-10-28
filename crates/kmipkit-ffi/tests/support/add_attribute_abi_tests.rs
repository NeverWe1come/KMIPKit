use super::*;
use kmipkit_client::ClientRequest;
use kmipkit_protocol::{AddAttributeRequest, NewAttribute};
use kmipkit_ttlv::{Item, RawTag, Structure, Value, ValueView};

#[link(name = "kmipkit_client_abi_c_consumer", kind = "static")]
// SAFETY: these declarations match the C consumer test helpers compiled by `build.rs`.
unsafe extern "C" {
    fn kmipkit_client_abi_c_consumer_build_add_attribute_request(
        out_request: *mut *mut kmipkit_client_batch_item_t,
    ) -> i32;
    fn kmipkit_client_abi_c_consumer_validate_add_attribute_response(
        response: *mut kmipkit_client_batch_item_response_t,
        expected_present: u8,
    ) -> i32;
}

#[test]
fn compiled_c_consumer_builds_add_attribute_and_preserves_the_borrowed_item() {
    let mut request_handle = ptr::null_mut();
    assert_eq!(
        // SAFETY: the helper initializes the valid, writable output slot before returning.
        unsafe {
            kmipkit_client_abi_c_consumer_build_add_attribute_request(&raw mut request_handle)
        },
        1,
        "the C consumer can pass its New Attribute Item without transferring ownership"
    );
    assert!(!request_handle.is_null());

    let handle = reference_handle(request_handle, Kind::BatchItem)
        .expect("the C consumer returns an Add Attribute batch item");
    let HandleValue::BatchItem(batch_item) = &handle.value else {
        panic!("the returned C handle contains a batch item");
    };
    let ClientRequest::AddAttribute(request) = batch_item.request() else {
        panic!("the C consumer constructs Add Attribute");
    };
    let payload = request
        .to_ttlv_payload()
        .expect("the C-built Add Attribute payload is valid");
    let payload_view = payload.view();
    let fields = payload_view.children();

    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].tag().raw(), 0x0042_0094);
    assert!(fields[0].with_value(|value| matches!(
        value,
        ValueView::TextString(text) if text.as_bytes() == b"object-7"
    )));
    assert_eq!(fields[1].tag().raw(), 0x0042_013d);
    assert!(fields[1].with_value(|value| match value {
        ValueView::Structure(new_attribute) => {
            let children = new_attribute.children();
            children.len() == 1
                && children[0].tag().raw() == 0x0042_000a
                && children[0].with_value(|name| match name {
                    ValueView::Structure(name) => {
                        let name_fields = name.children();
                        name_fields.len() == 2
                            && name_fields[0].tag().raw() == 0x0042_00a5
                            && name_fields[0].with_value(|value| {
                                matches!(
                                    value,
                                    ValueView::TextString(text) if text.as_bytes() == b"new name"
                                )
                            })
                            && name_fields[1].tag().raw() == 0x0042_00a6
                            && name_fields[1]
                                .with_value(|value| matches!(value, ValueView::Enumeration(1)))
                    }
                    _ => false,
                })
        }
        _ => false,
    }));

    kmipkit_client_batch_item_release(request_handle);
}

#[test]
fn add_attribute_response_copies_success_identifier_and_hides_failure_payload() {
    let request = || {
        ClientRequest::AddAttribute(AddAttributeRequest::new(
            None,
            NewAttribute::new(add_attribute_name_item()),
        ))
    };
    let success = successful_cryptographic_item(
        request(),
        0x0d,
        b"add-attribute-success",
        vec![ttlv_item(0x0042_0094, 0x07, b"object-7")],
    );
    let mut present = 0;
    let mut identifier = empty_identifier_record();
    assert_eq!(
        client_abi::kmipkit_client_batch_item_response_add_attribute(
            success,
            &raw mut present,
            &raw mut identifier,
        ),
        SUCCESS
    );
    assert_eq!(present, 1);
    assert_eq!(
        identifier.kind,
        crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING
    );
    assert_eq!(identifier.text_length, 8);
    // SAFETY: the ABI borrows the identifier bytes from `success`, which remains live here.
    assert_eq!(
        unsafe {
            std::slice::from_raw_parts(
                identifier.text_data,
                usize::try_from(identifier.text_length)
                    .expect("fixture identifier length fits usize"),
            )
        },
        b"object-7"
    );
    assert_eq!(
        // SAFETY: `success` is a live response handle and the helper only reads its typed result.
        unsafe { kmipkit_client_abi_c_consumer_validate_add_attribute_response(success, 1) },
        1,
        "a C consumer reads the successful Add Attribute identifier"
    );
    kmipkit_client_batch_item_response_release(success);

    let failure = failed_cryptographic_item(request(), 0x0d, b"add-attribute-failure");
    present = 9;
    identifier = empty_identifier_record();
    assert_eq!(
        client_abi::kmipkit_client_batch_item_response_add_attribute(
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
        // SAFETY: `failure` is a live response handle and the helper only reads its typed result.
        unsafe { kmipkit_client_abi_c_consumer_validate_add_attribute_response(failure, 0) },
        1,
        "a C consumer observes no typed payload for a failed operation"
    );
    kmipkit_client_batch_item_response_release(failure);
}

fn add_attribute_name_item() -> Item {
    let tag = |raw| {
        RawTag::new(raw)
            .expect("fixture tag fits the KMIP tag field")
            .try_checked()
            .expect("fixture tag is allocated")
    };
    let mut name_fields = Structure::new();
    name_fields
        .try_push(
            Item::new(tag(0x0042_00a5), Value::text_string("new name".to_owned()))
                .expect("Name Value is a valid Item"),
        )
        .expect("Name Value fits the Structure");
    name_fields
        .try_push(
            Item::new(tag(0x0042_00a6), Value::enumeration(1)).expect("Name Type is a valid Item"),
        )
        .expect("Name Type fits the Structure");
    Item::new(tag(0x0042_000a), Value::structure(name_fields))
        .expect("the New Attribute test item uses a valid Name")
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
