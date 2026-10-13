//! Common validated client and transport C ABI implementation.
#![allow(clippy::wildcard_imports)] // Shares the parent module's private FFI handle invariants.

use super::*;

#[cfg(test)]
// These tests intentionally pass Rust references as raw C pointers and slice the byte above the
// exact allocation bound to exercise the FFI's one-byte-over rejection path.
#[allow(clippy::borrow_as_ptr, clippy::range_plus_one)]
#[path = "../../tests/support/client_abi_tests.rs"]
mod client_abi_tests;

fn copy_transport_span(data: *const u8, length: u64, maximum: u64) -> FfiResult<Vec<u8>> {
    let bytes = input_span(data, length, maximum)?;
    let mut owned = Vec::new();
    owned
        .try_reserve_exact(bytes.len())
        .map_err(|_| ERROR_RESOURCE_LIMIT)?;
    owned.extend_from_slice(bytes);
    Ok(owned)
}

fn read_transport_file(path: &str) -> FfiResult<Vec<u8>> {
    let metadata = fs::metadata(path).map_err(|_| ERROR_INVALID_INPUT)?;
    if metadata.len() > MAX_TRANSPORT_MATERIAL_BYTES {
        return Err(ERROR_RESOURCE_LIMIT);
    }
    let initial_length = usize::try_from(metadata.len()).map_err(|_| ERROR_RESOURCE_LIMIT)?;
    let mut file = fs::File::open(path).map_err(|_| ERROR_INVALID_INPUT)?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(initial_length)
        .map_err(|_| ERROR_RESOURCE_LIMIT)?;
    file.by_ref()
        .take(MAX_TRANSPORT_MATERIAL_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| ERROR_INVALID_INPUT)?;
    if u64::try_from(bytes.len()).map_err(|_| ERROR_RESOURCE_LIMIT)? > MAX_TRANSPORT_MATERIAL_BYTES
    {
        return Err(ERROR_RESOURCE_LIMIT);
    }
    Ok(bytes)
}

fn validate_transport_input_total(inputs: &[kmipkit_transport_input_v1_t]) -> FfiResult<()> {
    let mut total = 0_u64;
    for input in inputs {
        let length = match input.source_kind {
            TRANSPORT_SOURCE_MEMORY => input.data.length,
            TRANSPORT_SOURCE_FILE => {
                let path = transport_input_path(input)?;
                fs::metadata(path).map_err(|_| ERROR_INVALID_INPUT)?.len()
            }
            _ => return Err(ERROR_INVALID_INPUT),
        };
        if length > MAX_TRANSPORT_MATERIAL_BYTES {
            return Err(ERROR_RESOURCE_LIMIT);
        }
        total = total.checked_add(length).ok_or(ERROR_RESOURCE_LIMIT)?;
        if total > MAX_TRANSPORT_MATERIAL_BYTES {
            return Err(ERROR_RESOURCE_LIMIT);
        }
    }
    Ok(())
}

fn transport_input_slice<'a>(
    inputs: *const kmipkit_transport_input_v1_t,
    count: u64,
) -> FfiResult<&'a [kmipkit_transport_input_v1_t]> {
    if count > MAX_TRANSPORT_CONFIG_ITEMS {
        return Err(ERROR_RESOURCE_LIMIT);
    }
    let count = usize::try_from(count).map_err(|_| ERROR_RESOURCE_LIMIT)?;
    if count == 0 {
        return Ok(&[]);
    }
    if inputs.is_null() {
        return Err(ERROR_INVALID_INPUT);
    }
    // SAFETY: for a nonzero count the caller must provide that many readable
    // contiguous v1 input records; count is bounded and converted above.
    Ok(unsafe { std::slice::from_raw_parts(inputs, count) })
}

fn transport_input_encoding(input: &kmipkit_transport_input_v1_t) -> FfiResult<u32> {
    match input.encoding {
        TRANSPORT_ENCODING_PEM | TRANSPORT_ENCODING_DER => Ok(input.encoding),
        _ => Err(ERROR_INVALID_INPUT),
    }
}

fn transport_input_bytes(input: &kmipkit_transport_input_v1_t) -> FfiResult<Vec<u8>> {
    copy_transport_span(
        input.data.data,
        input.data.length,
        MAX_TRANSPORT_MATERIAL_BYTES,
    )
}

fn transport_input_path(input: &kmipkit_transport_input_v1_t) -> FfiResult<String> {
    let bytes = copy_transport_span(input.data.data, input.data.length, MAX_TRANSPORT_TEXT_BYTES)?;
    let path = std::str::from_utf8(&bytes).map_err(|_| ERROR_INVALID_INPUT)?;
    if path.is_empty() {
        return Err(ERROR_INVALID_INPUT);
    }
    Ok(path.to_owned())
}

fn certificate_input(input: &kmipkit_transport_input_v1_t) -> FfiResult<CertificateInput> {
    let encoding = transport_input_encoding(input)?;
    match (input.source_kind, encoding) {
        (TRANSPORT_SOURCE_MEMORY, TRANSPORT_ENCODING_PEM) => {
            Ok(CertificateInput::from_pem(transport_input_bytes(input)?))
        }
        (TRANSPORT_SOURCE_MEMORY, TRANSPORT_ENCODING_DER) => {
            Ok(CertificateInput::from_der(vec![transport_input_bytes(
                input,
            )?]))
        }
        (TRANSPORT_SOURCE_FILE, TRANSPORT_ENCODING_PEM) => {
            Ok(CertificateInput::from_pem(certificate_input_bytes(input)?))
        }
        (TRANSPORT_SOURCE_FILE, TRANSPORT_ENCODING_DER) => {
            Ok(CertificateInput::from_der(vec![certificate_input_bytes(
                input,
            )?]))
        }
        _ => Err(ERROR_INVALID_INPUT),
    }
}

fn certificate_input_bytes(input: &kmipkit_transport_input_v1_t) -> FfiResult<Vec<u8>> {
    match input.source_kind {
        TRANSPORT_SOURCE_MEMORY => transport_input_bytes(input),
        TRANSPORT_SOURCE_FILE => {
            let path = transport_input_path(input)?;
            read_transport_file(&path)
        }
        _ => Err(ERROR_INVALID_INPUT),
    }
}

fn client_certificate_chain(
    inputs: &[kmipkit_transport_input_v1_t],
) -> FfiResult<CertificateInput> {
    let Some(first) = inputs.first() else {
        return Err(ERROR_INVALID_INPUT);
    };
    let encoding = transport_input_encoding(first)?;
    if inputs
        .iter()
        .any(|input| transport_input_encoding(input).ok() != Some(encoding))
    {
        return Err(ERROR_INVALID_INPUT);
    }
    validate_transport_input_total(inputs)?;
    if encoding == TRANSPORT_ENCODING_DER {
        let certificates = inputs
            .iter()
            .map(certificate_input_bytes)
            .collect::<FfiResult<Vec<_>>>()?;
        return Ok(CertificateInput::from_der(certificates));
    }
    let mut pem = Vec::new();
    for input in inputs {
        let bytes = certificate_input_bytes(input)?;
        pem.try_reserve(bytes.len())
            .map_err(|_| ERROR_RESOURCE_LIMIT)?;
        pem.extend_from_slice(&bytes);
    }
    Ok(CertificateInput::from_pem(pem))
}

fn private_key_input(input: &kmipkit_transport_input_v1_t) -> FfiResult<PrivateKeyInput> {
    let encoding = transport_input_encoding(input)?;
    match (input.source_kind, encoding) {
        (TRANSPORT_SOURCE_MEMORY, TRANSPORT_ENCODING_PEM) => {
            Ok(PrivateKeyInput::from_pem(transport_input_bytes(input)?))
        }
        (TRANSPORT_SOURCE_MEMORY, TRANSPORT_ENCODING_DER) => {
            Ok(PrivateKeyInput::from_der(transport_input_bytes(input)?))
        }
        (TRANSPORT_SOURCE_FILE, TRANSPORT_ENCODING_PEM) => {
            Ok(PrivateKeyInput::from_pem(certificate_input_bytes(input)?))
        }
        (TRANSPORT_SOURCE_FILE, TRANSPORT_ENCODING_DER) => {
            Ok(PrivateKeyInput::from_der(certificate_input_bytes(input)?))
        }
        _ => Err(ERROR_INVALID_INPUT),
    }
}

fn revocation_input(input: &kmipkit_transport_input_v1_t) -> FfiResult<RevocationListInput> {
    let encoding = transport_input_encoding(input)?;
    if input.source_kind != TRANSPORT_SOURCE_MEMORY {
        return Err(ERROR_INVALID_INPUT);
    }
    match encoding {
        TRANSPORT_ENCODING_PEM => Ok(RevocationListInput::from_pem(transport_input_bytes(input)?)),
        TRANSPORT_ENCODING_DER => Ok(RevocationListInput::from_der(vec![transport_input_bytes(
            input,
        )?])),
        _ => Err(ERROR_INVALID_INPUT),
    }
}

fn timeout_limit_v1(
    input: &kmipkit_timeout_limit_v1_t,
    default: TimeoutLimit,
) -> FfiResult<TimeoutLimit> {
    if input.reserved != 0 || input.reserved2 != 0 {
        return Err(ERROR_INVALID_INPUT);
    }
    match input.kind {
        0 if input.seconds == 0 && input.nanoseconds == 0 => Ok(default),
        TRANSPORT_TIMEOUT_BOUNDED if input.nanoseconds < 1_000_000_000 => Ok(
            TimeoutLimit::Bounded(Duration::new(input.seconds, input.nanoseconds)),
        ),
        TRANSPORT_TIMEOUT_UNBOUNDED if input.seconds == 0 && input.nanoseconds == 0 => {
            Ok(TimeoutLimit::Unbounded)
        }
        _ => Err(ERROR_INVALID_INPUT),
    }
}

fn transport_text(span: crate::kmipkit_byte_span_v1_t) -> FfiResult<String> {
    let bytes = copy_transport_span(span.data, span.length, MAX_TRANSPORT_TEXT_BYTES)?;
    let text = std::str::from_utf8(&bytes).map_err(|_| ERROR_INVALID_INPUT)?;
    if text.is_empty() {
        return Err(ERROR_INVALID_INPUT);
    }
    Ok(text.to_owned())
}

fn transport_config_from_v1(
    input: *const kmipkit_transport_config_v1_t,
) -> FfiResult<TransportConfig> {
    if input.is_null() {
        return Err(ERROR_INVALID_INPUT);
    }
    // SAFETY: a non-null pointer must reference a readable transport-config v1
    // prefix for the duration of this constructor call.
    let input = unsafe { &*input };
    let required_size = u32::try_from(std::mem::size_of::<kmipkit_transport_config_v1_t>())
        .map_err(|_| ERROR_INVALID_INPUT)?;
    if input.abi_version != 1
        || input.struct_size < required_size
        || input.reserved != 0
        || input.reserved2 != 0
        || input.flags & !(TRANSPORT_FLAG_TLS_SERVER_NAME | TRANSPORT_FLAG_TARGET_URI) != 0
    {
        return Err(ERROR_INVALID_INPUT);
    }

    let endpoint = match input.endpoint_kind {
        TRANSPORT_ENDPOINT_RAW_TLS => {
            let host = transport_text(input.endpoint)?;
            Endpoint::raw_tls(host, input.endpoint_port)
        }
        TRANSPORT_ENDPOINT_HTTPS if input.endpoint_port == 0 => {
            Endpoint::https(transport_text(input.endpoint)?)
        }
        _ => return Err(ERROR_INVALID_INPUT),
    };
    let client_certificates =
        transport_input_slice(input.client_certificates, input.client_certificate_count)?;
    if client_certificates.is_empty() {
        return Err(ERROR_INVALID_INPUT);
    }
    let identity = ClientIdentity::new(
        client_certificate_chain(client_certificates)?,
        private_key_input(&input.client_private_key)?,
    );

    let trust_certificates =
        transport_input_slice(input.trust_certificates, input.trust_certificate_count)?;
    let trust = match input.trust_kind {
        TRANSPORT_TRUST_PLATFORM if trust_certificates.is_empty() => TrustSource::platform(),
        TRANSPORT_TRUST_CERTIFICATES if !trust_certificates.is_empty() => {
            validate_transport_input_total(trust_certificates)?;
            TrustSource::certificate_authorities(
                trust_certificates
                    .iter()
                    .map(certificate_input)
                    .collect::<FfiResult<Vec<_>>>()?,
            )
        }
        _ => return Err(ERROR_INVALID_INPUT),
    };
    let revocation_inputs =
        transport_input_slice(input.revocation_lists, input.revocation_list_count)?;
    validate_transport_input_total(revocation_inputs)?;
    let revocation_lists = revocation_inputs
        .iter()
        .map(revocation_input)
        .collect::<FfiResult<Vec<_>>>()?;
    let defaults = TimeoutPolicy::default();
    let timeouts = defaults
        .with_connect(timeout_limit_v1(
            &input.connect_timeout,
            defaults.connect(),
        )?)
        .with_write(timeout_limit_v1(&input.write_timeout, defaults.write())?)
        .with_read(timeout_limit_v1(&input.read_timeout, defaults.read())?)
        .with_total(timeout_limit_v1(&input.total_timeout, defaults.total())?);
    let max_request_bytes =
        usize::try_from(input.max_request_bytes).map_err(|_| ERROR_RESOURCE_LIMIT)?;

    let mut builder = TransportConfig::builder(endpoint)
        .client_identity(identity)
        .trust_source(trust)
        .revocation_lists(revocation_lists)
        .timeouts(timeouts)
        .max_request_bytes(max_request_bytes);
    if input.flags & TRANSPORT_FLAG_TLS_SERVER_NAME != 0 {
        builder = builder.tls_server_name(transport_text(input.tls_server_name)?);
    } else if input.tls_server_name.length != 0 || !input.tls_server_name.data.is_null() {
        return Err(ERROR_INVALID_INPUT);
    }
    if input.flags & TRANSPORT_FLAG_TARGET_URI != 0 {
        builder = builder.target_uri(transport_text(input.target_uri)?);
    } else if input.target_uri.length != 0 || !input.target_uri.data.is_null() {
        return Err(ERROR_INVALID_INPUT);
    }
    builder.build().map_err(|_| ERROR_INVALID_INPUT)
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_transport_config_create_v1(
    configuration: *const kmipkit_transport_config_v1_t,
    out_configuration: *mut *mut kmipkit_transport_config_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_configuration);
        let configuration = try_ffi!(transport_config_from_v1(configuration));
        *out = make_handle(
            Kind::TransportConfig,
            HandleValue::TransportConfig(configuration),
        );
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_create(
    configuration: *mut kmipkit_client_configuration_t,
    transport_configuration: *mut *mut kmipkit_transport_config_t,
    out_client: *mut *mut kmipkit_client_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_client);
        if transport_configuration.is_null() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        // SAFETY: the caller supplies a writable in/out slot containing its
        // owned transport-config handle for this consuming constructor.
        let transport_slot = unsafe { &mut *transport_configuration };
        let transport_handle = *transport_slot;
        if transport_handle.is_null() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let configuration = read_handle!(configuration, Configuration);
        let HandleValue::Configuration(configuration) = &configuration.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        if let Err(status) = reference_handle(&transport_handle, Kind::TransportConfig) {
            return record_ffi_error(status);
        }
        let client_configuration = client_extension::ClientConfiguration::from_shared_registry(
            configuration.shared_extension_registry(),
        );
        let transport_configuration = match owned_payload(transport_handle, Kind::TransportConfig) {
            Ok(HandleValue::TransportConfig(configuration)) => configuration,
            Ok(_) => return record_ffi_error(ERROR_INVALID_INPUT),
            Err(status) => return record_ffi_error(status),
        };
        *transport_slot = ptr::null_mut();
        match Client::new(client_configuration, transport_configuration) {
            Ok(client) => {
                *out = make_handle(
                    Kind::Client,
                    HandleValue::Client(Box::new(std::sync::Mutex::new(client))),
                );
                SUCCESS
            }
            Err(error) => record_client_error(&error),
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_create(out_batch: *mut *mut kmipkit_client_batch_t) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_batch);
        *out = make_handle(
            Kind::ClientBatch,
            HandleValue::ClientBatch(std::sync::Mutex::new(ClientBatchBuilder::new())),
        );
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_add_item(
    batch: *mut kmipkit_client_batch_t,
    inout_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        if inout_item.is_null() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        // SAFETY: the caller provides a writable in/out slot containing its
        // owned typed item handle, which this call consumes on success.
        let item_slot = unsafe { &mut *inout_item };
        let item_handle = *item_slot;
        if item_handle.is_null() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let batch = read_handle!(batch, ClientBatch);
        let HandleValue::ClientBatch(batch) = &batch.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(mut batch) = batch.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        if let Err(status) = reference_handle(&item_handle, Kind::BatchItem) {
            return record_ffi_error(status);
        }
        if batch.items.try_reserve(1).is_err() {
            return record_ffi_error(ERROR_RESOURCE_LIMIT);
        }
        let item = match owned_payload(item_handle, Kind::BatchItem) {
            Ok(HandleValue::BatchItem(item)) => *item,
            Ok(_) => return record_ffi_error(ERROR_INVALID_INPUT),
            Err(status) => return record_ffi_error(status),
        };
        *item_slot = ptr::null_mut();
        batch.items.push(item);
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_set_request_time_stamp(
    batch: *mut kmipkit_client_batch_t,
    time_stamp: i64,
) -> i32 {
    ffi_entrypoint(|| {
        let batch = read_handle!(batch, ClientBatch);
        let HandleValue::ClientBatch(batch) = &batch.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(mut batch) = batch.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        batch.request_time_stamp = Some(time_stamp);
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_set_order_option(
    batch: *mut kmipkit_client_batch_t,
    enabled: u8,
) -> i32 {
    ffi_entrypoint(|| {
        if enabled > 1 {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let batch = read_handle!(batch, ClientBatch);
        let HandleValue::ClientBatch(batch) = &batch.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(mut batch) = batch.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        batch.batch_order_option = Some(enabled != 0);
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_add_error_continuation_value(
    batch: *mut kmipkit_client_batch_t,
    value: u32,
) -> i32 {
    ffi_entrypoint(|| {
        let batch = read_handle!(batch, ClientBatch);
        let HandleValue::ClientBatch(batch) = &batch.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(mut batch) = batch.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        if batch.batch_error_continuation_values.len()
            >= usize::try_from(MAX_TRANSPORT_CONFIG_ITEMS).unwrap_or(usize::MAX)
        {
            return record_ffi_error(ERROR_RESOURCE_LIMIT);
        }
        if batch
            .batch_error_continuation_values
            .try_reserve(1)
            .is_err()
        {
            return record_ffi_error(ERROR_RESOURCE_LIMIT);
        }
        batch.batch_error_continuation_values.push(value);
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_set_asynchronous_indicator(
    batch: *mut kmipkit_client_batch_t,
    indicator: u32,
) -> i32 {
    ffi_entrypoint(|| {
        let batch = read_handle!(batch, ClientBatch);
        let HandleValue::ClientBatch(batch) = &batch.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(mut batch) = batch.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        batch.asynchronous_indicator = Some(indicator);
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_set_client_correlation_value(
    batch: *mut kmipkit_client_batch_t,
    value: *const u8,
    value_length: u64,
) -> i32 {
    ffi_entrypoint(|| {
        let value = try_ffi!(input_span(value, value_length, u64::MAX));
        let value = try_ffi!(std::str::from_utf8(value).map_err(|_| ERROR_INVALID_INPUT));
        let batch = read_handle!(batch, ClientBatch);
        let HandleValue::ClientBatch(batch) = &batch.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(mut batch) = batch.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        batch.client_correlation_value = Some(value.to_owned());
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_execute(
    client: *mut kmipkit_client_t,
    batch: *mut *mut kmipkit_client_batch_t,
    limits: *mut kmipkit_codec_limits_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_response);
        if batch.is_null() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        // SAFETY: the caller supplies a writable in/out slot holding the
        // one-shot ordered batch consumed by execution.
        let batch_slot = unsafe { &mut *batch };
        let batch_handle = *batch_slot;
        if batch_handle.is_null() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let client = read_handle!(client, Client);
        let HandleValue::Client(client) = &client.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let limits_handle = read_handle!(limits, CodecLimits);
        let HandleValue::CodecLimits(limits) = &limits_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        if let Err(status) = reference_handle(&batch_handle, Kind::ClientBatch) {
            return record_ffi_error(status);
        }
        let batch_payload = match owned_payload(batch_handle, Kind::ClientBatch) {
            Ok(HandleValue::ClientBatch(batch)) => batch,
            Ok(_) => return record_ffi_error(ERROR_INVALID_INPUT),
            Err(status) => return record_ffi_error(status),
        };
        *batch_slot = ptr::null_mut();
        let batch = match batch_payload.into_inner() {
            Ok(batch) => batch.into_batch(),
            Err(_) => return record_ffi_error(ERROR_INTERNAL),
        };
        let Ok(mut client) = client.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        match client.execute(batch, limits) {
            Ok(response) => {
                *out = make_handle(
                    Kind::ClientBatchResponse,
                    HandleValue::ClientBatchResponse(Arc::new(response)),
                );
                SUCCESS
            }
            Err(error) => record_client_error(&error),
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_response_release(
    handle: *mut kmipkit_client_batch_response_t,
) {
    ffi_entrypoint(|| release(handle, Kind::ClientBatchResponse));
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_release(
    handle: *mut kmipkit_client_batch_item_response_t,
) {
    ffi_entrypoint(|| release(handle, Kind::ClientBatchItemResponse));
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_release(handle: *mut kmipkit_client_t) {
    ffi_entrypoint(|| release(handle, Kind::Client));
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_release(handle: *mut kmipkit_client_batch_t) {
    ffi_entrypoint(|| release(handle, Kind::ClientBatch));
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_response_item_count(
    response: *mut kmipkit_client_batch_response_t,
    out_count: *mut u64,
) -> i32 {
    ffi_entrypoint(|| {
        let count = scalar_out!(out_count);
        let response = read_handle!(response, ClientBatchResponse);
        let HandleValue::ClientBatchResponse(response) = &response.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *count = u64::try_from(response.len()).unwrap_or(u64::MAX);
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_response_item_at(
    response: *mut kmipkit_client_batch_response_t,
    index: u64,
    out_item: *mut *mut kmipkit_client_batch_item_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let response = read_handle!(response, ClientBatchResponse);
        let HandleValue::ClientBatchResponse(response) = &response.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(index) = usize::try_from(index) else {
            return SUCCESS;
        };
        if response.get(index).is_some() {
            *out = make_handle(
                Kind::ClientBatchItemResponse,
                HandleValue::ClientBatchItemResponse {
                    response: Arc::clone(response),
                    index,
                },
            );
        }
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_metadata_v1(
    item: *mut kmipkit_client_batch_item_response_t,
    out_metadata: *mut kmipkit_client_batch_item_metadata_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        if out_metadata.is_null() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        let outcome = item.outcome();
        let result = outcome.result();
        let Ok(struct_size) =
            u32::try_from(std::mem::size_of::<kmipkit_client_batch_item_metadata_v1_t>())
        else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        // SAFETY: the caller supplies a writable pointer to the complete version 1 record.
        unsafe {
            *out_metadata = kmipkit_client_batch_item_metadata_v1_t {
                struct_size,
                abi_version: 1,
                outcome_kind: if outcome.asynchronous_correlation_value().is_some() {
                    BATCH_OUTCOME_PENDING
                } else {
                    BATCH_OUTCOME_COMPLETED
                },
                result_status: result.status().raw(),
                has_result_reason: u32::from(result.reason().is_some()),
                result_reason: result
                    .reason()
                    .map_or(0, kmipkit_protocol::ResultReason::raw),
            };
        }
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_unique_batch_item_id(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_value: *mut crate::kmipkit_byte_span_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        let value = scalar_out!(out_value);
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        if let Some(bytes) = item.unique_batch_item_id() {
            *present = 1;
            *value = crate::kmipkit_byte_span_v1_t {
                data: bytes.as_ptr(),
                length: u64::try_from(bytes.len()).unwrap_or(u64::MAX),
            };
        } else {
            *present = 0;
            *value = crate::kmipkit_byte_span_v1_t {
                data: ptr::null(),
                length: 0,
            };
        }
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_pending_correlation_value(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_value: *mut crate::kmipkit_byte_span_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        let value = scalar_out!(out_value);
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        if let Some(bytes) = item.outcome().asynchronous_correlation_value() {
            *present = 1;
            *value = crate::kmipkit_byte_span_v1_t {
                data: bytes.as_ptr(),
                length: u64::try_from(bytes.len()).unwrap_or(u64::MAX),
            };
        } else {
            *present = 0;
            *value = crate::kmipkit_byte_span_v1_t {
                data: ptr::null(),
                length: 0,
            };
        }
        SUCCESS
    })
}
