use super::*;

#[test]
fn common_abi_handles_preserve_ownership_reject_null_and_stale_inputs_and_redact_values() {
    const SECRET_SENTINEL: &str = "KMIPKIT-0053-SECRET-REDACTION-SENTINEL";
    let parameters = CryptographicParametersBuilder::new()
        .with_p_source(SECRET_SENTINEL.as_bytes().to_vec())
        .build()
        .expect("P Source is a valid caller-provided byte string");
    let rendered = format!("{parameters:?}");
    assert!(!rendered.contains(SECRET_SENTINEL));

    let handle = make_handle(
        Kind::CryptographicParameters,
        HandleValue::CryptographicParameters(parameters),
    );
    // SAFETY: The null output is rejected before any write; the live handle remains valid.
    let null_output_status = unsafe {
        super::cryptographic_parameters::kmipkit_cryptographic_parameters_read_fields_v1(
            handle,
            ptr::null_mut(),
        )
    };
    assert_eq!(null_output_status, ERROR_INVALID_INPUT);
    assert!(reference_handle(handle, Kind::CryptographicParameters).is_ok());

    // SAFETY: An all-zero versioned record is writable, but its unsupported version is rejected.
    let mut invalid_record: kmipkit_cryptographic_parameters_fields_v1_t =
        unsafe { std::mem::zeroed() };
    invalid_record.struct_size = u32::try_from(std::mem::size_of::<
        kmipkit_cryptographic_parameters_fields_v1_t,
    >())
    .expect("the versioned test record size fits u32");
    invalid_record.abi_version = 2;
    let invalid_version_status = unsafe {
        super::cryptographic_parameters::kmipkit_cryptographic_parameters_read_fields_v1(
            handle,
            &raw mut invalid_record,
        )
    };
    assert_eq!(invalid_version_status, ERROR_INVALID_INPUT);
    assert!(reference_handle(handle, Kind::CryptographicParameters).is_ok());

    release(handle, Kind::CryptographicParameters);
    // SAFETY: The address is stale by design. The ABI checks its handle registry before access.
    let stale_handle_status = unsafe {
        super::cryptographic_parameters::kmipkit_cryptographic_parameters_read_fields_v1(
            handle,
            &raw mut invalid_record,
        )
    };
    assert_eq!(stale_handle_status, ERROR_INVALID_INPUT);
}
