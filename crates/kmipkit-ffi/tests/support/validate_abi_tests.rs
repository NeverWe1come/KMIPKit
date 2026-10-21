use super::*;
use crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;

#[test]
fn validate_constructor_preserves_identifier_and_optional_date() {
    let identifier_text = b"validate-chain-private-sentinel";
    let identifier = kmipkit_unique_identifier_v1_t {
        struct_size: std::mem::size_of::<kmipkit_unique_identifier_v1_t>() as u32,
        abi_version: 1,
        kind: KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING,
        reserved: 0,
        text_data: identifier_text.as_ptr(),
        text_length: identifier_text.len() as u64,
        enumeration_value: 0,
        integer_value: 0,
    };
    let mut item = ptr::null_mut();

    assert_eq!(
        kmipkit_client_batch_item_validate(
            ptr::null_mut(),
            0,
            &identifier,
            1,
            1,
            i64::MIN,
            &raw mut item,
        ),
        SUCCESS
    );
    assert!(!item.is_null());

    let handle = reference_handle(item, Kind::BatchItem)
        .expect("Validate constructor returns a batch item handle");
    let HandleValue::BatchItem(batch_item) = &handle.value else {
        panic!("Validate constructor returns a batch item");
    };
    let ClientRequest::Validate(request) = batch_item.request() else {
        panic!("batch item contains a Validate request");
    };
    assert_eq!(request.chain_len(), 1);
    assert_eq!(request.validity_date(), Some(i64::MIN));
    assert!(!format!("{request:?}").contains("validate-chain-private-sentinel"));

    kmipkit_client_batch_item_release(item);
}

#[test]
fn validate_constructor_borrows_and_copies_payload_extensions() {
    let extension = make_handle(
        Kind::Item,
        HandleValue::Item(
            Item::new(
                RawTag::new(0x0054_1234)
                    .expect("vendor extension tag fits the raw tag field")
                    .try_checked()
                    .expect("vendor extension tag is allocated"),
                Value::text_string("validate-extension-sentinel".to_owned()),
            )
            .expect("valid text extension forms a TTLV item"),
        ),
    );
    let extension_handles = [extension];
    let mut item = ptr::null_mut();

    assert_eq!(
        kmipkit_client_batch_item_validate(
            ptr::null_mut(),
            0,
            ptr::null(),
            0,
            extension_handles.as_ptr(),
            extension_handles.len() as u64,
            0,
            0,
            &raw mut item,
        ),
        SUCCESS
    );
    assert!(!item.is_null());
    assert!(reference_handle(extension, Kind::Item).is_ok());

    kmipkit_client_batch_item_release(item);
    kmipkit_ttlv_item_release(extension);
}

#[test]
fn validate_constructor_rejects_noncanonical_absent_date_and_clears_output() {
    let mut item = ptr::dangling_mut();

    assert_eq!(
        kmipkit_client_batch_item_validate(
            ptr::null_mut(),
            0,
            ptr::null(),
            0,
            0,
            1,
            &raw mut item,
        ),
        ERROR_INVALID_INPUT
    );
    assert!(item.is_null());
}

#[test]
fn validate_response_accessors_clear_outputs_for_invalid_item() {
    let mut present = u8::MAX;
    let mut indicator = u32::MAX;

    assert_eq!(
        client_abi::kmipkit_client_batch_item_response_validate(
            ptr::null_mut(),
            &raw mut present,
            &raw mut indicator,
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!(present, 0);
    assert_eq!(indicator, 0);

    let mut response_ttlv = ptr::dangling_mut();
    assert_eq!(
        client_abi::kmipkit_client_batch_item_response_validate_ttlv(
            ptr::null_mut(),
            &raw mut present,
            &raw mut response_ttlv,
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!(present, 0);
    assert!(response_ttlv.is_null());
}
