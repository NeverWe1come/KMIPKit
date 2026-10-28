use super::*;
use kmipkit_client::ClientRequest;
use kmipkit_protocol::{ModifyAttributeRequest, NewAttribute};
use kmipkit_ttlv::{Item, RawTag, Value, ValueView};

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
fn modify_attribute_response_copies_success_identifier_and_hides_failure_fields() {
    let request = || {
        let tag = RawTag::new(0x0042_000d)
            .expect("test tag fits")
            .try_checked()
            .expect("test tag is assigned");
        let attribute = Item::new(tag, Value::text_string("replacement".to_owned()))
            .expect("test attribute is valid");
        ClientRequest::ModifyAttribute(ModifyAttributeRequest::new(
            None,
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
