//! C ABI bridge to the Rust Cryptographic Parameters model.
use super::*;

fn valid_presence_flags(fields: &kmipkit_cryptographic_parameters_fields_v1_t) -> bool {
    [
        fields.block_cipher_mode_present,
        fields.padding_method_present,
        fields.hashing_algorithm_present,
        fields.key_role_type_present,
        fields.digital_signature_algorithm_present,
        fields.cryptographic_algorithm_present,
        fields.random_iv_present,
        fields.iv_length_present,
        fields.tag_length_present,
        fields.fixed_field_length_present,
        fields.invocation_field_length_present,
        fields.counter_length_present,
        fields.initial_counter_value_present,
        fields.salt_length_present,
        fields.mask_generator_present,
        fields.mask_generator_hashing_algorithm_present,
        fields.p_source_present,
        fields.trailer_field_present,
    ]
    .into_iter()
    .all(|value| value <= 1)
        && fields.random_iv <= 1
}

fn build_parameters(
    fields: &kmipkit_cryptographic_parameters_fields_v1_t,
) -> Result<CryptographicParameters, i32> {
    let mut builder = CryptographicParametersBuilder::new();
    if fields.block_cipher_mode_present != 0 {
        builder = builder.with_block_cipher_mode(fields.block_cipher_mode);
    }
    if fields.padding_method_present != 0 {
        builder = builder.with_padding_method(fields.padding_method);
    }
    if fields.hashing_algorithm_present != 0 {
        builder = builder.with_hashing_algorithm(fields.hashing_algorithm);
    }
    if fields.key_role_type_present != 0 {
        builder = builder.with_key_role_type(fields.key_role_type);
    }
    if fields.digital_signature_algorithm_present != 0 {
        builder = builder.with_digital_signature_algorithm(fields.digital_signature_algorithm);
    }
    if fields.cryptographic_algorithm_present != 0 {
        builder = builder.with_cryptographic_algorithm(fields.cryptographic_algorithm);
    }
    if fields.random_iv_present != 0 {
        builder = builder.with_random_iv(fields.random_iv != 0);
    }
    if fields.iv_length_present != 0 {
        builder = builder.with_iv_length(fields.iv_length);
    }
    if fields.tag_length_present != 0 {
        builder = builder.with_tag_length(fields.tag_length);
    }
    if fields.fixed_field_length_present != 0 {
        builder = builder.with_fixed_field_length(fields.fixed_field_length);
    }
    if fields.invocation_field_length_present != 0 {
        builder = builder.with_invocation_field_length(fields.invocation_field_length);
    }
    if fields.counter_length_present != 0 {
        builder = builder.with_counter_length(fields.counter_length);
    }
    if fields.initial_counter_value_present != 0 {
        builder = builder.with_initial_counter_value(fields.initial_counter_value);
    }
    if fields.salt_length_present != 0 {
        builder = builder.with_salt_length(fields.salt_length);
    }
    if fields.mask_generator_present != 0 {
        builder = builder.with_mask_generator(fields.mask_generator);
    }
    if fields.mask_generator_hashing_algorithm_present != 0 {
        builder =
            builder.with_mask_generator_hashing_algorithm(fields.mask_generator_hashing_algorithm);
    }
    if fields.p_source_present != 0 {
        let source = input_span(
            fields.p_source,
            fields.p_source_length,
            MAX_TRANSPORT_MATERIAL_BYTES,
        )?;
        let mut copy = Vec::new();
        copy.try_reserve_exact(source.len())
            .map_err(|_| ERROR_RESOURCE_LIMIT)?;
        copy.extend_from_slice(source);
        builder = builder.with_p_source(copy);
    }
    if fields.trailer_field_present != 0 {
        builder = builder.with_trailer_field(fields.trailer_field);
    }
    builder.build().map_err(|error| protocol_status(&error))
}

/// Builds a KMIP 2.1 Cryptographic Parameters value from a versioned field record.
///
/// Raw Enumeration values remain open to future and vendor assignments. Rust constructs and
/// validates the canonical Table 59 order. The caller retains ownership of the input record and
/// its P Source bytes.
///
/// # Safety
///
/// `fields` must point to an aligned, readable version 1 record for the duration of the call.
/// When P Source is present, its pointer must reference `p_source_length` readable bytes. The
/// output must point to a writable, aligned handle slot that does not overlap the input record or
/// P Source bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kmipkit_cryptographic_parameters_from_fields_v1(
    fields: *const kmipkit_cryptographic_parameters_fields_v1_t,
    out_parameters: *mut *mut kmipkit_cryptographic_parameters_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_parameters);
        if fields.is_null() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        // SAFETY: the caller promises an aligned, readable v1 fields record for this call.
        let fields = unsafe { &*fields };
        if fields.struct_size as usize != size_of::<kmipkit_cryptographic_parameters_fields_v1_t>()
            || fields.abi_version != 1
            || !valid_presence_flags(fields)
            || (fields.p_source_present == 0
                && (!fields.p_source.is_null() || fields.p_source_length != 0))
        {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        if fields.p_source_present != 0 && fields.p_source_length > MAX_TRANSPORT_MATERIAL_BYTES {
            return record_ffi_error(ERROR_RESOURCE_LIMIT);
        }
        let parameters = match build_parameters(fields) {
            Ok(parameters) => parameters,
            Err(status) => return record_ffi_error(status),
        };
        *out = try_ffi!(try_make_handle(
            Kind::CryptographicParameters,
            HandleValue::CryptographicParameters(parameters),
        ));
        SUCCESS
    })
}

/// Copies and validates a generic TTLV Structure as Cryptographic Parameters.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_cryptographic_parameters_from_ttlv(
    structure: *mut kmipkit_ttlv_structure_t,
    out_parameters: *mut *mut kmipkit_cryptographic_parameters_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_parameters);
        let structure = try_ffi!(copy_required_structure_handle(structure));
        let parameters = match CryptographicParameters::try_from_ttlv(structure) {
            Ok(parameters) => parameters,
            Err(error) => return record_ffi_error(protocol_status(&error)),
        };
        *out = try_ffi!(try_make_handle(
            Kind::CryptographicParameters,
            HandleValue::CryptographicParameters(parameters),
        ));
        SUCCESS
    })
}

/// Consumes a typed Cryptographic Parameters value into its generic TTLV Structure.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_cryptographic_parameters_into_ttlv(
    parameters: *mut kmipkit_cryptographic_parameters_t,
    out_structure: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_structure);
        let payload = try_ffi!(owned_payload(parameters, Kind::CryptographicParameters));
        let HandleValue::CryptographicParameters(parameters) = payload else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = try_ffi!(try_make_handle(
            Kind::Structure,
            HandleValue::Structure(parameters.into_ttlv()),
        ));
        SUCCESS
    })
}

/// Releases a typed Cryptographic Parameters value and zeroizes KMIPKit-owned P Source storage.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_cryptographic_parameters_release(
    parameters: *mut kmipkit_cryptographic_parameters_t,
) {
    ffi_entrypoint(|| release(parameters, Kind::CryptographicParameters));
}
