use super::*;

fn request() -> kmipkit_create_key_pair_request_v1_t {
    kmipkit_create_key_pair_request_v1_t {
        struct_size: u32::try_from(std::mem::size_of::<kmipkit_create_key_pair_request_v1_t>())
            .expect("the ABI input record fits uint32"),
        abi_version: 1,
        cryptographic_algorithm: 6,
        cryptographic_length: 256,
        private_usage_mask_present: 1,
        public_usage_mask_present: 1,
        reserved_0: 0,
        reserved_1: 0,
        private_usage_mask: 1,
        public_usage_mask: 2,
    }
}

#[test]
fn key_pair_v1_preserves_explicit_and_absent_attribute_groups() {
    // KMIPKIT-0093-FR-011/013; KMIP 2.1 section 6.1.2.
    let mut input = request();
    let parsed = create_key_pair_request_v1(&raw const input).expect("valid semantic input");
    assert!(parsed.has_common_attributes());
    assert!(parsed.has_private_key_attributes());
    assert!(parsed.has_public_key_attributes());
    input.private_usage_mask_present = 0;
    input.public_usage_mask_present = 0;
    input.private_usage_mask = 0;
    input.public_usage_mask = 0;
    let parsed = create_key_pair_request_v1(&raw const input).expect("optional usage masks");
    assert!(!parsed.has_private_key_attributes());
    assert!(!parsed.has_public_key_attributes());
}

#[test]
fn key_pair_v1_additional_common_attributes_preserve_layout_and_reject_duplicates() {
    // The original public v1 record stays 28 bytes with four-byte alignment.
    assert_eq!(
        std::mem::size_of::<kmipkit_create_key_pair_request_v1_t>(),
        28
    );
    let input = request();
    let mut additional = AttributeSet::new();
    additional
        .try_push_typed(
            Attribute::<typed::CryptographicDomainParameters>::new(Structure::new())
                .expect("typed domain parameters"),
        )
        .expect("one common attribute");
    assert!(
        create_key_pair_request_with_attributes_v1(&raw const input, Some(&additional)).is_ok()
    );
    additional
        .try_push_typed(Attribute::<typed::CryptographicAlgorithm>::new(6).expect("algorithm"))
        .expect("additional algorithm");
    assert!(matches!(
        create_key_pair_request_with_attributes_v1(&raw const input, Some(&additional)),
        Err(ERROR_INVALID_INPUT)
    ));
}

#[test]
fn key_pair_v1_rejects_invalid_record_fields_before_execution() {
    let mut invalid = Vec::new();
    let mut input = request();
    input.struct_size = 0;
    invalid.push(input);
    input = request();
    input.abi_version = 2;
    invalid.push(input);
    input = request();
    input.reserved_0 = 1;
    invalid.push(input);
    input = request();
    input.reserved_1 = 1;
    invalid.push(input);
    input = request();
    input.private_usage_mask_present = 2;
    invalid.push(input);
    input = request();
    input.public_usage_mask_present = 2;
    invalid.push(input);
    input = request();
    input.private_usage_mask_present = 0;
    invalid.push(input);
    input = request();
    input.public_usage_mask_present = 0;
    invalid.push(input);
    for (index, input) in invalid.into_iter().enumerate() {
        assert!(
            matches!(
                create_key_pair_request_v1(&raw const input),
                Err(ERROR_INVALID_INPUT)
            ),
            "invalid field case {index} was accepted"
        );
    }
    assert!(matches!(
        create_key_pair_request_v1(std::ptr::null()),
        Err(ERROR_INVALID_INPUT)
    ));
    assert!(matches!(
        create_key_pair_request_v1(std::ptr::dangling::<u8>().cast()),
        Err(ERROR_INVALID_INPUT)
    ));
}

#[test]
fn key_pair_v1_execution_entrypoints_validate_inputs_before_missing_client() {
    let input = request();
    let mut response = std::ptr::null_mut();
    assert_eq!(
        kmipkit_client_create_key_pair_v1(
            std::ptr::null_mut(),
            &raw const input,
            std::ptr::null_mut(),
            &raw mut response,
        ),
        ERROR_INVALID_INPUT
    );
    assert!(response.is_null());

    let mut invalid_input = request();
    invalid_input.abi_version = 2;
    assert_eq!(
        kmipkit_client_create_key_pair_v1(
            std::ptr::null_mut(),
            &raw const invalid_input,
            std::ptr::null_mut(),
            &raw mut response,
        ),
        ERROR_INVALID_INPUT
    );

    let inherited_timeout = || kmipkit_timeout_limit_v1_t {
        kind: 0,
        reserved: 0,
        seconds: 0,
        nanoseconds: 0,
        reserved2: 0,
    };
    let options = kmipkit_request_options_v1_t {
        struct_size: u32::try_from(std::mem::size_of::<kmipkit_request_options_v1_t>())
            .expect("the ABI options record fits uint32"),
        abi_version: 1,
        connect_timeout: inherited_timeout(),
        write_timeout: inherited_timeout(),
        read_timeout: inherited_timeout(),
        total_timeout: inherited_timeout(),
    };
    // SAFETY: both input records and the output slot are live, aligned, and non-overlapping.
    assert_eq!(
        unsafe {
            kmipkit_client_create_key_pair_with_options_v1(
                std::ptr::null_mut(),
                &raw const input,
                std::ptr::null_mut(),
                std::ptr::null(),
                &raw mut response,
            )
        },
        ERROR_INVALID_INPUT
    );
    // SAFETY: both input records and the output slot are live, aligned, and non-overlapping.
    assert_eq!(
        unsafe {
            kmipkit_client_create_key_pair_with_options_v1(
                std::ptr::null_mut(),
                &raw const input,
                std::ptr::null_mut(),
                &raw const options,
                &raw mut response,
            )
        },
        ERROR_INVALID_INPUT
    );

    let source = make_handle(Kind::Structure, HandleValue::Structure(Structure::new()));
    let mut attributes = std::ptr::null_mut();
    assert_eq!(
        kmipkit_attribute_set_create(source, &raw mut attributes),
        SUCCESS
    );
    assert_eq!(
        kmipkit_client_create_key_pair_with_common_attributes_v1(
            std::ptr::null_mut(),
            &raw const input,
            attributes,
            std::ptr::null_mut(),
            &raw mut response,
        ),
        ERROR_INVALID_INPUT
    );
    kmipkit_attribute_set_release(attributes);
    kmipkit_ttlv_structure_release(source);
}
