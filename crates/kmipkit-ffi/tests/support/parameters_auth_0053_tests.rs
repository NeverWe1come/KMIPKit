use super::client_abi::kmipkit_client_batch_create;
use super::*;

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
