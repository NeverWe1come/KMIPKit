use super::*;

fn abi_size<T>() -> u32 {
    u32::try_from(size_of::<T>()).expect("test ABI record size fits u32")
}

#[test]
fn attribute_reference_copy_rejects_malformed_records_without_dereferencing_misaligned_data() {
    assert_eq!(
        copy_attribute_reference(ptr::null()),
        Err(ERROR_INVALID_INPUT)
    );

    let storage = [0_u64; 4];
    let misaligned = ptr::from_ref(&storage).cast::<u8>().wrapping_add(1).cast();
    assert_eq!(
        copy_attribute_reference(misaligned),
        Err(ERROR_INVALID_INPUT)
    );

    let valid_tag = crate::kmipkit_attribute_reference_v1_t {
        struct_size: abi_size::<crate::kmipkit_attribute_reference_v1_t>(),
        abi_version: 1,
        kind: crate::KMIPKIT_ATTRIBUTE_REFERENCE_KIND_TAG,
        reserved: 0,
        reserved2: 0,
        tag_value: 0x0042_0001,
        vendor_identification_data: ptr::null(),
        vendor_identification_length: 0,
        attribute_name_data: ptr::null(),
        attribute_name_length: 0,
    };
    assert_eq!(
        copy_attribute_reference(&raw const valid_tag),
        Ok(AttributeReference::tag(0x0042_0001))
    );

    let vendor = b"Vendor";
    let name = b"Attribute";
    let invalid_utf8 = [0xff];
    let valid_name = crate::kmipkit_attribute_reference_v1_t {
        kind: crate::KMIPKIT_ATTRIBUTE_REFERENCE_KIND_NAME,
        tag_value: 0,
        vendor_identification_data: vendor.as_ptr(),
        vendor_identification_length: vendor.len() as u64,
        attribute_name_data: name.as_ptr(),
        attribute_name_length: name.len() as u64,
        ..valid_tag
    };
    assert_eq!(
        copy_attribute_reference(&raw const valid_name),
        Ok(AttributeReference::name("Vendor", "Attribute"))
    );

    let invalid = [
        crate::kmipkit_attribute_reference_v1_t {
            struct_size: 1,
            ..valid_tag
        },
        crate::kmipkit_attribute_reference_v1_t {
            abi_version: 2,
            ..valid_tag
        },
        crate::kmipkit_attribute_reference_v1_t {
            reserved: 1,
            ..valid_tag
        },
        crate::kmipkit_attribute_reference_v1_t {
            reserved2: 1,
            ..valid_tag
        },
        crate::kmipkit_attribute_reference_v1_t {
            vendor_identification_data: vendor.as_ptr(),
            vendor_identification_length: vendor.len() as u64,
            ..valid_tag
        },
        crate::kmipkit_attribute_reference_v1_t {
            kind: u32::MAX,
            ..valid_tag
        },
        crate::kmipkit_attribute_reference_v1_t {
            tag_value: 1,
            ..valid_name
        },
        crate::kmipkit_attribute_reference_v1_t {
            vendor_identification_data: ptr::null(),
            vendor_identification_length: 1,
            ..valid_name
        },
        crate::kmipkit_attribute_reference_v1_t {
            attribute_name_data: ptr::null(),
            attribute_name_length: 1,
            ..valid_name
        },
        crate::kmipkit_attribute_reference_v1_t {
            vendor_identification_data: invalid_utf8.as_ptr(),
            vendor_identification_length: 1,
            ..valid_name
        },
    ];
    for reference in invalid {
        assert_eq!(
            copy_attribute_reference(&raw const reference),
            Err(ERROR_INVALID_INPUT)
        );
    }
}

#[test]
fn consuming_handle_arrays_reject_duplicates_wrong_kinds_shared_and_released_handles() {
    let empty: [*mut crate::kmipkit_raw_tag_t; 0] = [];
    assert!(
        consume_unshared_handles(&empty, Kind::RawTag)
            .expect("an empty input consumes no handles")
            .is_empty()
    );

    let duplicate = make_handle::<crate::kmipkit_raw_tag_t>(
        Kind::RawTag,
        HandleValue::RawTag(RawTag::new(0x0042_0001).expect("tag is valid")),
    );
    assert!(matches!(
        consume_unshared_handles(&[duplicate, duplicate], Kind::RawTag),
        Err(ERROR_INVALID_INPUT)
    ));
    release(duplicate, Kind::RawTag);

    let wrong_kind = make_handle::<crate::kmipkit_raw_tag_t>(
        Kind::RawTag,
        HandleValue::RawTag(RawTag::new(0x0042_0002).expect("tag is valid")),
    );
    assert!(matches!(
        consume_unshared_handles(&[wrong_kind], Kind::Tag),
        Err(ERROR_INVALID_INPUT)
    ));
    release(wrong_kind, Kind::RawTag);

    let shared = make_handle::<crate::kmipkit_raw_tag_t>(
        Kind::RawTag,
        HandleValue::RawTag(RawTag::new(0x0042_0003).expect("tag is valid")),
    );
    let retained = reference_handle(shared, Kind::RawTag).expect("test retains one shared owner");
    assert!(matches!(
        consume_unshared_handles(&[shared], Kind::RawTag),
        Err(ERROR_SHARED_OWNER)
    ));
    drop(retained);
    let consumed = consume_unshared_handles(&[shared], Kind::RawTag)
        .expect("the handle becomes consumable after its shared owner is dropped");
    assert_eq!(consumed.len(), 1);

    let stale = make_handle::<crate::kmipkit_raw_tag_t>(
        Kind::RawTag,
        HandleValue::RawTag(RawTag::new(0x0042_0004).expect("tag is valid")),
    );
    release(stale, Kind::RawTag);
    assert!(matches!(
        consume_unshared_handles(&[stale], Kind::RawTag),
        Err(ERROR_INVALID_INPUT)
    ));
}

#[test]
fn split_key_reference_copy_checks_thresholds_and_preserves_identifier_value_forms() {
    let missing_threshold = make_handle::<crate::kmipkit_split_key_reference_t>(
        Kind::SplitKeyReference,
        HandleValue::SplitKeyReference(SplitKeyReference::new(UniqueIdentifier::TextString(
            "missing-threshold".to_owned(),
        ))),
    );
    let mut missing_threshold_handle = [missing_threshold];
    assert!(matches!(
        copy_join_split_key_references(missing_threshold_handle.as_mut_ptr(), 1),
        Err(ERROR_INVALID_INPUT)
    ));
    release(missing_threshold, Kind::SplitKeyReference);

    let text_reference = make_handle::<crate::kmipkit_split_key_reference_t>(
        Kind::SplitKeyReference,
        HandleValue::SplitKeyReference(
            SplitKeyReference::new(UniqueIdentifier::TextString("text-id".to_owned()))
                .with_split_key_threshold(2),
        ),
    );
    let integer_reference = make_handle::<crate::kmipkit_split_key_reference_t>(
        Kind::SplitKeyReference,
        HandleValue::SplitKeyReference(
            SplitKeyReference::new(UniqueIdentifier::Integer(17)).with_split_key_threshold(3),
        ),
    );
    let mut handles = [text_reference, integer_reference];
    let copied = copy_join_split_key_references(handles.as_mut_ptr(), 2)
        .expect("complete reference context copies");
    assert_eq!(copied.len(), 2);
    assert_eq!(copied[0].split_key_threshold(), Some(2));
    assert_eq!(copied[1].split_key_threshold(), Some(3));
    release(text_reference, Kind::SplitKeyReference);
    release(integer_reference, Kind::SplitKeyReference);
}

#[test]
fn ttlv_decode_ffi_checks_record_limits_input_and_valid_payloads() {
    let bytes = [
        0x42, 0x00, 0x01, // Tag
        0x02, // Integer
        0x00, 0x00, 0x00, 0x04, // Length
        0x00, 0x00, 0x00, 0x01, // Value
        0x00, 0x00, 0x00, 0x00, // Padding
    ];
    let mut error = crate::kmipkit_ttlv_decode_error_v1_t {
        struct_size: abi_size::<crate::kmipkit_ttlv_decode_error_v1_t>(),
        abi_version: 1,
        kind: u32::MAX,
        byte_offset: u64::MAX,
    };
    let mut output = ptr::dangling_mut();

    let mut wrong_size = error;
    wrong_size.struct_size -= 1;
    assert_eq!(
        decode_ttlv_into_handle(
            bytes.as_ptr(),
            bytes.len() as u64,
            &CodecLimits::defaults(),
            &raw mut wrong_size,
            &raw mut output,
        ),
        ERROR_INVALID_INPUT
    );
    assert!(output.is_null());

    assert_eq!(
        decode_ttlv_into_handle(
            bytes.as_ptr(),
            u64::try_from(CodecLimits::defaults().max_message_bytes()).expect("limit fits ABI") + 1,
            &CodecLimits::defaults(),
            &raw mut error,
            &raw mut output,
        ),
        ERROR_RESOURCE_LIMIT
    );
    assert_eq!(
        error.kind,
        decode_error_kind_code(DecodeErrorKind::MessageTooLarge)
    );

    assert_eq!(
        decode_ttlv_into_handle(
            ptr::null(),
            1,
            &CodecLimits::defaults(),
            &raw mut error,
            &raw mut output,
        ),
        ERROR_INVALID_INPUT
    );

    assert_eq!(
        decode_ttlv_into_handle(
            bytes[..1].as_ptr(),
            1,
            &CodecLimits::defaults(),
            &raw mut error,
            &raw mut output,
        ),
        ERROR_PROTOCOL
    );
    assert_ne!(error.kind, 0);
    assert_eq!(error.byte_offset, 0);

    assert_eq!(
        decode_ttlv_into_handle(
            bytes.as_ptr(),
            bytes.len() as u64,
            &CodecLimits::defaults(),
            &raw mut error,
            &raw mut output,
        ),
        SUCCESS
    );
    assert!(!output.is_null());
    release(output, Kind::Item);
}

#[test]
fn extension_identity_creation_rejects_invalid_utf8_and_null_nonempty_spans() {
    let invalid_utf8 = [0xff];
    let name = b"extension";
    let version = b"1";
    let mut identity = ptr::dangling_mut();

    assert_eq!(
        kmipkit_extension_identity_create(
            invalid_utf8.as_ptr(),
            invalid_utf8.len() as u64,
            name.as_ptr(),
            name.len() as u64,
            version.as_ptr(),
            version.len() as u64,
            &raw mut identity,
        ),
        ERROR_INVALID_IDENTITY
    );
    assert!(identity.is_null());

    assert_eq!(
        kmipkit_extension_identity_create(
            ptr::null(),
            1,
            name.as_ptr(),
            name.len() as u64,
            version.as_ptr(),
            version.len() as u64,
            &raw mut identity,
        ),
        ERROR_INVALID_INPUT
    );
    assert!(identity.is_null());
}

#[test]
fn ttlv_path_child_tag_rejects_out_of_range_and_unallocated_tags() {
    let mut path = ptr::null_mut();
    assert_eq!(
        kmipkit_ttlv_path_create(0x0042_0001, &raw mut path),
        SUCCESS
    );

    let mut child = ptr::dangling_mut();
    assert_eq!(
        kmipkit_ttlv_path_with_child_tag(path, 0x0100_0000, &raw mut child),
        ERROR_INVALID_INPUT
    );
    assert!(child.is_null());

    assert_eq!(
        kmipkit_ttlv_path_create(0x0042_0001, &raw mut path),
        SUCCESS
    );
    assert_eq!(
        kmipkit_ttlv_path_with_child_tag(path, 0x00ff_fffe, &raw mut child),
        ERROR_INVALID_SCHEMA
    );
    assert!(child.is_null());
}
