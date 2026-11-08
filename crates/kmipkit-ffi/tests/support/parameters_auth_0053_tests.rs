use super::client_abi::kmipkit_client_batch_create;
use super::*;

fn empty_cryptographic_parameters_fields() -> kmipkit_cryptographic_parameters_fields_v1_t {
    // SAFETY: The versioned record contains only integer fields and raw pointers, for which zero is valid.
    unsafe { std::mem::zeroed() }
}

unsafe extern "C" {
    fn kmipkit_parameters_authentication_c_consumer_parameters() -> i32;
    fn kmipkit_parameters_authentication_c_consumer_reads() -> i32;
    fn kmipkit_parameters_authentication_c_consumer_from_ttlv() -> i32;
    fn kmipkit_parameters_authentication_c_consumer_attach_authentication(
        batch: *mut kmipkit_client_batch_t,
        authentication: *mut kmipkit_authentication_t,
    ) -> i32;
    fn kmipkit_parameters_authentication_c_consumer_reject_authentication(
        authentication: *mut kmipkit_authentication_t,
    ) -> i32;
}

#[test]
fn compiled_c_consumer_builds_all_table_59_parameters_in_source_order() {
    // SAFETY: This no-argument consumer owns and releases every handle it creates.
    let status = unsafe { kmipkit_parameters_authentication_c_consumer_parameters() };
    assert_eq!(status, SUCCESS);
}

#[test]
fn compiled_c_consumer_reads_versioned_cryptographic_parameter_fields() {
    // SAFETY: The consumer owns and releases its Cryptographic Parameters handle.
    let status = unsafe { kmipkit_parameters_authentication_c_consumer_reads() };
    assert_eq!(status, SUCCESS);
}

#[test]
fn compiled_c_consumer_parses_and_preserves_generic_vendor_parameters() {
    // SAFETY: The C consumer owns and releases all handles it creates.
    let status = unsafe { kmipkit_parameters_authentication_c_consumer_from_ttlv() };
    assert_eq!(status, SUCCESS);
}

#[test]
fn cryptographic_parameters_reader_returns_versioned_fields_and_borrowed_p_source() {
    let parameters = CryptographicParametersBuilder::new()
        .with_block_cipher_mode(0x8000_0001)
        .with_random_iv(false)
        .with_iv_length(12)
        .with_p_source(vec![0x00, 0xff, 0x81])
        .build()
        .expect("the parameter combination is structurally valid");
    let handle = make_handle(
        Kind::CryptographicParameters,
        HandleValue::CryptographicParameters(parameters),
    );
    // SAFETY: the record contains integer fields and a nullable pointer; an all-zero
    // representation is valid before the ABI fills its versioned output fields.
    let mut output: kmipkit_cryptographic_parameters_fields_v1_t = unsafe { std::mem::zeroed() };
    output.struct_size = u32::try_from(std::mem::size_of::<
        kmipkit_cryptographic_parameters_fields_v1_t,
    >())
    .expect("the versioned test record size fits u32");
    output.abi_version = 1;

    // SAFETY: `handle` is live and `output` is an exclusive writable v1 record.
    let status = unsafe {
        super::cryptographic_parameters::kmipkit_cryptographic_parameters_read_fields_v1(
            handle,
            &raw mut output,
        )
    };
    assert_eq!(status, SUCCESS);
    assert_eq!(output.block_cipher_mode_present, 1);
    assert_eq!(output.block_cipher_mode, 0x8000_0001);
    assert_eq!(output.random_iv_present, 1);
    assert_eq!(output.random_iv, 0);
    assert_eq!(output.iv_length_present, 1);
    assert_eq!(output.iv_length, 12);
    assert_eq!(output.p_source_present, 1);
    assert_eq!(output.p_source_length, 3);
    // SAFETY: this span is borrowed from the live CryptographicParameters handle and is
    // valid until that handle is released.
    let p_source_length = usize::try_from(output.p_source_length)
        .expect("the test P Source length fits the platform usize");
    let p_source = unsafe { std::slice::from_raw_parts(output.p_source, p_source_length) };
    assert_eq!(p_source, [0x00, 0xff, 0x81]);

    release(handle, Kind::CryptographicParameters);
}

#[test]
#[allow(clippy::too_many_lines)] // Keep the complete versioned-field round-trip in one auditable fixture.
fn cryptographic_parameters_abi_roundtrips_every_versioned_field() {
    let p_source = [0x00, 0x7f, 0xff];
    let mut input = empty_cryptographic_parameters_fields();
    input.struct_size = u32::try_from(std::mem::size_of::<
        kmipkit_cryptographic_parameters_fields_v1_t,
    >())
    .expect("versioned record size fits u32");
    input.abi_version = 1;
    input.block_cipher_mode_present = 1;
    input.block_cipher_mode = 0x8000_0001;
    input.padding_method_present = 1;
    input.padding_method = 0x8000_0002;
    input.hashing_algorithm_present = 1;
    input.hashing_algorithm = 0x8000_0003;
    input.key_role_type_present = 1;
    input.key_role_type = 0x8000_0004;
    input.digital_signature_algorithm_present = 1;
    input.digital_signature_algorithm = 0x8000_0005;
    input.cryptographic_algorithm_present = 1;
    input.cryptographic_algorithm = 0x8000_0006;
    input.random_iv_present = 1;
    input.random_iv = 1;
    input.iv_length_present = 1;
    input.iv_length = 12;
    input.tag_length_present = 1;
    input.tag_length = 16;
    input.fixed_field_length_present = 1;
    input.fixed_field_length = 2;
    input.invocation_field_length_present = 1;
    input.invocation_field_length = 4;
    input.counter_length_present = 1;
    input.counter_length = 32;
    input.initial_counter_value_present = 1;
    input.initial_counter_value = 1;
    input.salt_length_present = 1;
    input.salt_length = 8;
    input.mask_generator_present = 1;
    input.mask_generator = 0x8000_0007;
    input.mask_generator_hashing_algorithm_present = 1;
    input.mask_generator_hashing_algorithm = 0x8000_0008;
    input.p_source_present = 1;
    input.p_source = p_source.as_ptr();
    input.p_source_length = p_source.len() as u64;
    input.trailer_field_present = 1;
    input.trailer_field = i32::MIN + 9;

    let mut parameters = std::ptr::null_mut();
    // SAFETY: Input and its P Source span are readable and the output handle slot is writable.
    assert_eq!(
        unsafe {
            super::cryptographic_parameters::kmipkit_cryptographic_parameters_from_fields_v1(
                &raw const input,
                &raw mut parameters,
            )
        },
        SUCCESS
    );
    let mut output = empty_cryptographic_parameters_fields();
    output.struct_size = input.struct_size;
    output.abi_version = 1;
    // SAFETY: The handle is live and `output` is an initialized writable v1 record.
    assert_eq!(
        unsafe {
            super::cryptographic_parameters::kmipkit_cryptographic_parameters_read_fields_v1(
                parameters,
                &raw mut output,
            )
        },
        SUCCESS
    );
    assert_eq!(output.block_cipher_mode, input.block_cipher_mode);
    assert_eq!(output.padding_method, input.padding_method);
    assert_eq!(output.hashing_algorithm, input.hashing_algorithm);
    assert_eq!(output.key_role_type, input.key_role_type);
    assert_eq!(
        output.digital_signature_algorithm,
        input.digital_signature_algorithm
    );
    assert_eq!(
        output.cryptographic_algorithm,
        input.cryptographic_algorithm
    );
    assert_eq!(output.random_iv, input.random_iv);
    assert_eq!(output.iv_length, input.iv_length);
    assert_eq!(output.tag_length, input.tag_length);
    assert_eq!(output.fixed_field_length, input.fixed_field_length);
    assert_eq!(
        output.invocation_field_length,
        input.invocation_field_length
    );
    assert_eq!(output.counter_length, input.counter_length);
    assert_eq!(output.initial_counter_value, input.initial_counter_value);
    assert_eq!(output.salt_length, input.salt_length);
    assert_eq!(output.mask_generator, input.mask_generator);
    assert_eq!(
        output.mask_generator_hashing_algorithm,
        input.mask_generator_hashing_algorithm
    );
    assert_eq!(output.p_source_present, 1);
    assert_eq!(output.p_source_length, p_source.len() as u64);
    // SAFETY: The returned P Source is borrowed from the live typed parameters handle.
    assert_eq!(
        unsafe { std::slice::from_raw_parts(output.p_source, p_source.len()) },
        p_source
    );
    assert_eq!(output.trailer_field, input.trailer_field);
    release(parameters, Kind::CryptographicParameters);
}

#[test]
fn cryptographic_parameters_abi_rejects_invalid_flags_inconsistent_spans_and_size() {
    let expected_size = u32::try_from(std::mem::size_of::<
        kmipkit_cryptographic_parameters_fields_v1_t,
    >())
    .expect("versioned record size fits u32");
    let mut input = empty_cryptographic_parameters_fields();
    input.struct_size = expected_size;
    input.abi_version = 1;
    let mut output = std::ptr::null_mut();

    for invalid in [
        {
            let mut value = input;
            value.struct_size = expected_size - 1;
            value
        },
        {
            let mut value = input;
            value.abi_version = 2;
            value
        },
        {
            let mut value = input;
            value.block_cipher_mode_present = 2;
            value
        },
        {
            let mut value = input;
            value.random_iv = 2;
            value
        },
        {
            let mut value = input;
            value.p_source_length = 1;
            value
        },
    ] {
        // SAFETY: Each malformed but readable record is rejected before dereferencing optional spans.
        assert_eq!(
            unsafe {
                super::cryptographic_parameters::kmipkit_cryptographic_parameters_from_fields_v1(
                    &raw const invalid,
                    &raw mut output,
                )
            },
            ERROR_INVALID_INPUT
        );
        assert!(output.is_null());
    }

    input.p_source_present = 1;
    input.p_source_length = MAX_TRANSPORT_MATERIAL_BYTES + 1;
    // SAFETY: The maximum-size check rejects this length before reading the null pointer.
    assert_eq!(
        unsafe {
            super::cryptographic_parameters::kmipkit_cryptographic_parameters_from_fields_v1(
                &raw const input,
                &raw mut output,
            )
        },
        ERROR_RESOURCE_LIMIT
    );
    assert!(output.is_null());
}

#[test]
fn compiled_c_consumer_can_attach_authentication_to_a_batch() {
    let credential = Credential::try_from_ttlv({
        let mut fields = Structure::new();
        fields
            .try_push(
                Item::new(
                    RawTag::new(0x0042_0024)
                        .expect("tag fits")
                        .try_checked()
                        .expect("tag is assigned"),
                    Value::enumeration(0x8000_0001),
                )
                .expect("credential type item is valid"),
            )
            .expect("credential type fits");
        fields
            .try_push(
                Item::new(
                    RawTag::new(0x0042_0025)
                        .expect("tag fits")
                        .try_checked()
                        .expect("tag is assigned"),
                    Value::structure(Structure::new()),
                )
                .expect("credential value item is valid"),
            )
            .expect("credential value fits");
        fields
    })
    .expect("credential envelope is valid");
    let authentication = Authentication::new(vec![credential])
        .expect("one credential forms non-empty Authentication");
    let authentication = make_handle(
        Kind::Authentication,
        HandleValue::Authentication(authentication),
    );
    // SAFETY: The C consumer passes a live Authentication handle with an intentionally null
    // batch. The setter must reject the call without taking ownership.
    let status = unsafe {
        kmipkit_parameters_authentication_c_consumer_reject_authentication(authentication)
    };
    assert_eq!(status, ERROR_INVALID_INPUT);
    assert!(reference_handle(authentication, Kind::Authentication).is_ok());

    let mut batch = ptr::null_mut();
    assert_eq!(kmipkit_client_batch_create(&raw mut batch), SUCCESS);

    // SAFETY: The C wrapper receives live handles and the setter consumes Authentication on success.
    let status = unsafe {
        kmipkit_parameters_authentication_c_consumer_attach_authentication(batch, authentication)
    };
    assert_eq!(status, SUCCESS);
    assert!(reference_handle(authentication, Kind::Authentication).is_err());
    release(batch, Kind::ClientBatch);
}
