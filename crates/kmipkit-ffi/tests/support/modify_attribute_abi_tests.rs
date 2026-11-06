use super::*;
use kmipkit_client::ClientRequest;
use kmipkit_protocol::{ModifyAttributeRequest, NewAttribute};
use kmipkit_ttlv::{Item, RawTag, Value, ValueView};

#[link(name = "kmipkit_client_abi_c_consumer", kind = "static")]
// SAFETY: these declarations match the C consumer helpers compiled by `build.rs`.
unsafe extern "C" {
    fn kmipkit_client_abi_c_consumer_build_modify_attribute(
        out_request: *mut *mut kmipkit_client_batch_item_t,
    ) -> i32;
    fn kmipkit_client_abi_c_consumer_validate_modify_attribute_response(
        response: *mut kmipkit_client_batch_item_response_t,
        expected_present: u8,
    ) -> i32;
}

fn item_handle(raw_tag: u32, text: &str) -> *mut kmipkit_ttlv_item_t {
    let tag = RawTag::new(raw_tag)
        .expect("test attribute tag fits the KMIP tag field")
        .try_checked()
        .expect("test attribute tag is assigned");
    let item = Item::new(tag, Value::text_string(text.to_owned()))
        .expect("test attribute is a valid generic TTLV item");
    try_make_handle(Kind::Item, HandleValue::Item(item)).expect("test item handle is registered")
}

fn unique_identifier() -> kmipkit_unique_identifier_v1_t {
    kmipkit_unique_identifier_v1_t {
        struct_size: u32::try_from(std::mem::size_of::<kmipkit_unique_identifier_v1_t>())
            .expect("identifier record size fits the ABI"),
        abi_version: 1,
        kind: crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING,
        reserved: 0,
        text_data: b"object-7".as_ptr(),
        text_length: 8,
        enumeration_value: 0,
        integer_value: 0,
    }
}

#[test]
fn modify_attribute_constructor_preserves_optional_current_and_distinct_new_value() {
    let current = item_handle(0x0042_000d, "old value");
    let new = item_handle(0x0042_000d, "replacement");
    let mut identifier = unique_identifier();
    let mut request_handle = ptr::null_mut();

    assert_eq!(
        client_abi::kmipkit_client_batch_item_modify_attribute(
            &raw mut identifier,
            current,
            new,
            &raw mut request_handle,
        ),
        SUCCESS
    );
    let handle = reference_handle(request_handle, Kind::BatchItem)
        .expect("Modify Attribute returns an opaque batch item");
    let HandleValue::BatchItem(batch_item) = &handle.value else {
        panic!("the returned handle contains a batch item");
    };
    let ClientRequest::ModifyAttribute(request) = batch_item.request() else {
        panic!("the constructor creates Modify Attribute");
    };
    assert_eq!(request.unique_identifier(), Some("object-7"));
    assert!(request.current_attribute().is_some());
    let payload = request
        .to_ttlv_payload()
        .expect("Modify Attribute payload is valid");
    let payload_view = payload.view();
    let fields = payload_view.children();
    assert_eq!(fields.len(), 3);
    assert_eq!(fields[0].tag().raw(), 0x0042_0094);
    assert_eq!(fields[1].tag().raw(), 0x0042_013c);
    assert!(fields[1].with_value(|value| match value {
        ValueView::Structure(current) => {
            let children = current.children();
            children.len() == 1
                && children[0].tag().raw() == 0x0042_000d
                && children[0].with_value(|value| {
                    matches!(value, ValueView::TextString(text) if text.as_bytes() == b"old value")
                })
        }
        _ => false,
    }));
    assert_eq!(fields[2].tag().raw(), 0x0042_013d);
    assert!(fields[2].with_value(|value| match value {
        ValueView::Structure(new) => {
            let children = new.children();
            children.len() == 1
                && children[0].tag().raw() == 0x0042_000d
                && children[0].with_value(|value| {
                    matches!(value, ValueView::TextString(text) if text.as_bytes() == b"replacement")
                })
        }
        _ => false,
    }));

    kmipkit_client_batch_item_release(request_handle);
    kmipkit_ttlv_item_release(current);
    kmipkit_ttlv_item_release(new);
}

#[test]
fn modify_attribute_constructor_preserves_omitted_current_attribute() {
    let new = item_handle(0x0042_000d, "replacement");
    let mut request_handle = ptr::null_mut();

    assert_eq!(
        client_abi::kmipkit_client_batch_item_modify_attribute(
            ptr::null(),
            ptr::null_mut(),
            new,
            &raw mut request_handle,
        ),
        SUCCESS
    );
    let handle = reference_handle(request_handle, Kind::BatchItem)
        .expect("Modify Attribute returns an opaque batch item");
    let HandleValue::BatchItem(batch_item) = &handle.value else {
        panic!("the returned handle contains a batch item");
    };
    let ClientRequest::ModifyAttribute(request) = batch_item.request() else {
        panic!("the constructor creates Modify Attribute");
    };
    assert!(request.unique_identifier().is_none());
    assert!(request.current_attribute().is_none());
    assert_eq!(
        request
            .to_ttlv_payload()
            .expect("omitted Current Attribute is valid")
            .view()
            .children()
            .len(),
        1
    );

    kmipkit_client_batch_item_release(request_handle);
    kmipkit_ttlv_item_release(new);
}

#[test]
fn compiled_c_consumer_builds_modify_attribute_and_preserves_borrowed_inputs() {
    let mut request_handle = ptr::null_mut();
    assert_eq!(
        // SAFETY: the C helper initializes the writable output slot before returning.
        unsafe { kmipkit_client_abi_c_consumer_build_modify_attribute(&raw mut request_handle) },
        1,
        "the C constructor copies both Attribute Items and preserves caller ownership"
    );
    assert!(!request_handle.is_null());

    let handle = reference_handle(request_handle, Kind::BatchItem)
        .expect("the C consumer returns a Modify Attribute batch item");
    let HandleValue::BatchItem(batch_item) = &handle.value else {
        panic!("the returned C handle contains a batch item");
    };
    let ClientRequest::ModifyAttribute(request) = batch_item.request() else {
        panic!("the C consumer constructs Modify Attribute");
    };
    assert_eq!(request.unique_identifier(), Some("object-7"));
    assert!(request.current_attribute().is_some());
    let payload = request
        .to_ttlv_payload()
        .expect("the C-built Modify Attribute payload is valid");
    let payload_view = payload.view();
    let fields = payload_view.children();
    assert_eq!(
        fields
            .iter()
            .map(|field| field.tag().raw())
            .collect::<Vec<_>>(),
        [0x0042_0094, 0x0042_013C, 0x0042_013D]
    );
    assert!(fields[1].with_value(|value| matches!(value, ValueView::Structure(_))));
    assert!(fields[2].with_value(|value| matches!(value, ValueView::Structure(_))));

    kmipkit_client_batch_item_release(request_handle);
}

#[test]
fn modify_attribute_response_copies_success_identifier_and_hides_failure_fields() {
    let request = || {
        let tag = RawTag::new(0x0042_000d)
            .expect("test tag fits")
            .try_checked()
            .expect("test tag is assigned");
        let attribute = Item::new(tag, Value::text_string("replacement".to_owned()))
            .expect("test attribute is valid");
        ClientRequest::ModifyAttribute(ModifyAttributeRequest::new(
            Some(RESPONSE_FIXTURE_UID.to_owned()),
            None,
            NewAttribute::new(attribute),
        ))
    };
    let success = successful_cryptographic_item(
        request(),
        0x0e,
        b"modify-attribute-success",
        vec![ttlv_item(0x0042_0094, 0x07, b"object-7")],
    );
    let mut present = 0;
    let mut identifier = empty_identifier_record();
    assert_eq!(
        client_abi::kmipkit_client_batch_item_response_modify_attribute(
            success,
            &raw mut present,
            &raw mut identifier,
        ),
        SUCCESS
    );
    assert_eq!(present, 1);
    // SAFETY: response text is borrowed while `success` remains live.
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
        // SAFETY: `success` remains live while the C consumer reads its typed response.
        unsafe { kmipkit_client_abi_c_consumer_validate_modify_attribute_response(success, 1) },
        1,
        "a C consumer reads the successful Modify Attribute identifier"
    );
    kmipkit_client_batch_item_response_release(success);

    let failure = failed_cryptographic_item(request(), 0x0e, b"modify-attribute-failure");
    present = 9;
    identifier = empty_identifier_record();
    assert_eq!(
        client_abi::kmipkit_client_batch_item_response_modify_attribute(
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
        // SAFETY: `failure` remains live while the C consumer checks absent typed fields.
        unsafe { kmipkit_client_abi_c_consumer_validate_modify_attribute_response(failure, 0) },
        1,
        "a C consumer observes no typed fields for a failed operation"
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
