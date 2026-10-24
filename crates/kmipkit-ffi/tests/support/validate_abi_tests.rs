use super::*;
use crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;

#[test]
fn validate_constructor_preserves_identifier_and_optional_date() {
    let identifier_text = b"validate-chain-private-sentinel";
    let identifier = kmipkit_unique_identifier_v1_t {
        struct_size: u32::try_from(std::mem::size_of::<kmipkit_unique_identifier_v1_t>())
            .expect("versioned identifier record size fits u32"),
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
            &raw const identifier,
            1,
            ptr::null_mut(),
            0,
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
fn validate_constructor_preserves_nonempty_certificate_array() {
    let mut certificate = Structure::new();
    for (tag, value) in [
        (0x0042_001d, Value::enumeration(1)),
        (
            0x0042_001e,
            Value::byte_string(b"validate-certificate-ffi-sentinel".to_vec()),
        ),
    ] {
        certificate
            .try_push(
                Item::new(
                    RawTag::new(tag)
                        .expect("Validate certificate tag is a valid raw KMIP tag")
                        .try_checked()
                        .expect("Validate certificate tag is allocated"),
                    value,
                )
                .expect("Validate certificate field is valid TTLV"),
            )
            .expect("Validate certificate fits within the default limits");
    }
    let certificate_handle = make_handle(Kind::Structure, HandleValue::Structure(certificate));
    let mut certificates = [certificate_handle];
    let mut item = ptr::null_mut();

    assert_eq!(
        kmipkit_client_batch_item_validate(
            certificates.as_mut_ptr(),
            certificates.len() as u64,
            ptr::null(),
            0,
            ptr::null_mut(),
            0,
            0,
            0,
            &raw mut item,
        ),
        SUCCESS
    );
    assert!(!item.is_null());

    let handle = reference_handle(item, Kind::BatchItem)
        .expect("Validate constructor returns a live batch-item handle");
    let HandleValue::BatchItem(batch_item) = &handle.value else {
        panic!("Validate constructor returns a batch item");
    };
    let ClientRequest::Validate(request) = batch_item.request() else {
        panic!("batch item contains a Validate request");
    };
    assert_eq!(request.chain_len(), 1);
    assert!(reference_handle(certificate_handle, Kind::Structure).is_ok());

    kmipkit_client_batch_item_release(item);
    kmipkit_ttlv_structure_release(certificate_handle);
}

#[test]
fn validate_constructor_borrows_and_copies_payload_extensions() {
    let extension: *mut kmipkit_ttlv_item_t = make_handle(
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
    let mut extension_handles = [extension];
    let mut item = ptr::null_mut();

    assert_eq!(
        kmipkit_client_batch_item_validate(
            ptr::null_mut(),
            0,
            ptr::null(),
            0,
            extension_handles.as_mut_ptr(),
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
fn validate_constructor_rejects_output_overlapping_payload_extension_array() {
    let extension: *mut kmipkit_ttlv_item_t = make_handle(
        Kind::Item,
        HandleValue::Item(
            Item::new(
                RawTag::new(0x0054_1234)
                    .expect("vendor extension tag fits the raw tag field")
                    .try_checked()
                    .expect("vendor extension tag is allocated"),
                Value::text_string("validate-overlap-sentinel".to_owned()),
            )
            .expect("valid text extension forms a TTLV item"),
        ),
    );
    let mut extension_handles = [extension];
    let overlapping_output = extension_handles
        .as_mut_ptr()
        .cast::<*mut kmipkit_client_batch_item_t>();

    assert_eq!(
        kmipkit_client_batch_item_validate(
            ptr::null_mut(),
            0,
            ptr::null(),
            0,
            extension_handles.as_mut_ptr(),
            extension_handles.len() as u64,
            0,
            0,
            overlapping_output,
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!(extension_handles[0], extension);
    kmipkit_ttlv_item_release(extension);
}

#[test]
fn validate_constructor_rejects_output_overlapping_identifier_text() {
    let mut item: *mut kmipkit_client_batch_item_t = ptr::dangling_mut();
    let original_item = item;
    let mut identifiers = [kmipkit_unique_identifier_v1_t {
        struct_size: u32::try_from(std::mem::size_of::<kmipkit_unique_identifier_v1_t>())
            .expect("versioned identifier record size fits u32"),
        abi_version: 1,
        kind: KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING,
        reserved: 0,
        text_data: (&raw mut item).cast(),
        text_length: u64::try_from(std::mem::size_of::<*mut kmipkit_client_batch_item_t>())
            .expect("output pointer size fits u64"),
        enumeration_value: 0,
        integer_value: 0,
    }];

    let status = kmipkit_client_batch_item_validate(
        ptr::null_mut(),
        0,
        identifiers.as_mut_ptr(),
        identifiers.len() as u64,
        ptr::null_mut(),
        0,
        0,
        0,
        &raw mut item,
    );
    if status == SUCCESS && !item.is_null() {
        kmipkit_client_batch_item_release(item);
    }

    assert_eq!(status, ERROR_INVALID_INPUT);
    assert_eq!(item, original_item);
}

#[test]
fn validate_constructor_enforces_one_aggregate_ttlv_element_limit() {
    let tag = RawTag::new(0x0054_1234)
        .expect("vendor tag fits the raw tag field")
        .try_checked()
        .expect("vendor tag is allocated");
    let mut certificate_handles = Vec::new();
    for _ in 0..2 {
        let mut certificate = Structure::new();
        certificate
            .try_push(
                Item::new(
                    RawTag::new(0x0042_001d)
                        .expect("Certificate Type tag fits the raw tag field")
                        .try_checked()
                        .expect("Certificate Type tag is allocated"),
                    Value::enumeration(1),
                )
                .expect("Certificate Type is a valid item"),
            )
            .expect("Certificate Type fits the structure");
        certificate
            .try_push(
                Item::new(
                    RawTag::new(0x0042_001e)
                        .expect("Certificate Value tag fits the raw tag field")
                        .try_checked()
                        .expect("Certificate Value tag is allocated"),
                    Value::byte_string(vec![0]),
                )
                .expect("Certificate Value is a valid item"),
            )
            .expect("Certificate Value fits the structure");
        for _ in 0..49_998 {
            certificate
                .try_push(
                    Item::new(tag, Value::integer(0)).expect("vendor integer is a valid item"),
                )
                .expect("vendor integer fits the certificate");
        }
        certificate_handles.push(make_handle(
            Kind::Structure,
            HandleValue::Structure(certificate),
        ));
    }

    let mut item = ptr::dangling_mut();
    assert_eq!(
        kmipkit_client_batch_item_validate(
            certificate_handles.as_mut_ptr(),
            certificate_handles.len() as u64,
            ptr::null(),
            0,
            ptr::null_mut(),
            0,
            0,
            0,
            &raw mut item,
        ),
        ERROR_RESOURCE_LIMIT
    );
    assert!(item.is_null());
    for handle in certificate_handles {
        assert!(reference_handle(handle, Kind::Structure).is_ok());
        kmipkit_ttlv_structure_release(handle);
    }
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
            ptr::null_mut(),
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
