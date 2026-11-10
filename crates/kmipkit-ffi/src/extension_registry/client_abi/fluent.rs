//! Stable C adapters for the Rust-owned fluent request builders.

use super::*;
use std::sync::Arc;

type SymmetricBuilder = kmipkit_client::fluent::CreateSymmetricKeyBuilder<'static>;

fn inherited_timeout() -> kmipkit_timeout_limit_v1_t {
    kmipkit_timeout_limit_v1_t {
        kind: 0,
        reserved: 0,
        seconds: 0,
        nanoseconds: 0,
        reserved2: 0,
    }
}

fn der_transport_inputs(
    spans: *const kmipkit_byte_span_v1_t,
    count: u64,
) -> FfiResult<Vec<kmipkit_transport_input_v1_t>> {
    let count = usize::try_from(count).map_err(|_| ERROR_RESOURCE_LIMIT)?;
    if count == 0 || count > 64 || spans.is_null() || !spans.is_aligned() {
        return Err(ERROR_INVALID_INPUT);
    }
    // SAFETY: pointer, alignment, and bounded element count are checked above; the array is
    // borrowed only for this synchronous constructor call.
    let spans = unsafe { std::slice::from_raw_parts(spans, count) };
    spans
        .iter()
        .map(|span| {
            input_span(span.data, span.length, MAX_FFI_INPUT_BYTES)?;
            Ok(kmipkit_transport_input_v1_t {
                source_kind: TRANSPORT_SOURCE_MEMORY,
                encoding: TRANSPORT_ENCODING_DER,
                data: *span,
            })
        })
        .collect()
}

/// Creates a client from a compact raw-TLS/mTLS input record using DER material and default
/// timeouts. The supplied extension configuration remains caller-owned.
fn transport_from_mtls_v1(input: &kmipkit_mtls_client_config_v1_t) -> FfiResult<TransportConfig> {
    let required_size = u32::try_from(std::mem::size_of::<kmipkit_mtls_client_config_v1_t>())
        .map_err(|_| ERROR_INVALID_INPUT)?;
    if input.struct_size < required_size
        || input.abi_version != 1
        || input.reserved != 0
        || input.endpoint_port == 0
    {
        return Err(ERROR_INVALID_INPUT);
    }
    let certificates =
        der_transport_inputs(input.client_certificates, input.client_certificate_count)?;
    let authorities =
        der_transport_inputs(input.trust_certificates, input.trust_certificate_count)?;
    let max_request_bytes = if input.max_request_bytes == 0 {
        8 * 1024 * 1024
    } else {
        input.max_request_bytes
    };
    let transport = kmipkit_transport_config_v1_t {
        struct_size: u32::try_from(std::mem::size_of::<kmipkit_transport_config_v1_t>())
            .map_err(|_| ERROR_INVALID_INPUT)?,
        abi_version: 1,
        endpoint_kind: TRANSPORT_ENDPOINT_RAW_TLS,
        trust_kind: TRANSPORT_TRUST_CERTIFICATES,
        flags: TRANSPORT_FLAG_TLS_SERVER_NAME,
        reserved: 0,
        endpoint: input.endpoint_host,
        endpoint_port: input.endpoint_port,
        reserved2: 0,
        client_certificates: certificates.as_ptr(),
        client_certificate_count: u64::try_from(certificates.len())
            .map_err(|_| ERROR_RESOURCE_LIMIT)?,
        client_private_key: kmipkit_transport_input_v1_t {
            source_kind: TRANSPORT_SOURCE_MEMORY,
            encoding: TRANSPORT_ENCODING_DER,
            data: input.client_private_key,
        },
        trust_certificates: authorities.as_ptr(),
        trust_certificate_count: u64::try_from(authorities.len())
            .map_err(|_| ERROR_RESOURCE_LIMIT)?,
        revocation_lists: ptr::null(),
        revocation_list_count: 0,
        tls_server_name: input.tls_server_name,
        target_uri: kmipkit_byte_span_v1_t::default(),
        connect_timeout: inherited_timeout(),
        write_timeout: inherited_timeout(),
        read_timeout: inherited_timeout(),
        total_timeout: inherited_timeout(),
        max_request_bytes,
    };
    super::transport_config_from_v1(ptr::addr_of!(transport))
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_create_mtls_v1(
    configuration: *mut kmipkit_client_configuration_t,
    input: *const kmipkit_mtls_client_config_v1_t,
    out_client: *mut *mut kmipkit_client_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_client);
        if input.is_null() || !input.is_aligned() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        // SAFETY: the non-null aligned pointer is borrowed for this synchronous constructor call.
        let input = unsafe { input.read() };
        let transport = match transport_from_mtls_v1(&input) {
            Ok(transport) => transport,
            Err(status) => return record_ffi_error(status),
        };
        let configuration = read_handle!(configuration, Configuration);
        let HandleValue::Configuration(configuration) = &configuration.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let client_configuration =
            kmipkit_client::extension_registry::ClientConfiguration::from_shared_registry(
                configuration.shared_extension_registry(),
            )
            .with_wire_encoding(configuration.wire_encoding());
        match kmipkit_client::Client::new(client_configuration, transport) {
            Ok(client) => {
                *out = try_ffi!(try_make_handle(
                    Kind::Client,
                    HandleValue::Client(Box::new(Mutex::new(client))),
                ));
                SUCCESS
            }
            Err(error) => record_client_error(&error),
        }
    })
}

fn execute_fluent_item(
    client: *mut kmipkit_client_t,
    request: ClientRequest,
    limits: *mut kmipkit_codec_limits_t,
    options: &RequestOptions,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_response);
        let client_handle = read_handle!(client, Client);
        let HandleValue::Client(client) = &client_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let limits_handle = read_handle!(limits, CodecLimits);
        let HandleValue::CodecLimits(limits) = &limits_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(mut client) = client.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        match client.execute_with_options(
            ClientBatch::new(ClientBatchItem::new(request)),
            limits,
            options,
        ) {
            Ok(response) => {
                *out = try_ffi!(try_make_handle(
                    Kind::ClientBatchResponse,
                    HandleValue::ClientBatchResponse(Arc::new(response)),
                ));
                SUCCESS
            }
            Err(error) => record_client_error(&error),
        }
    })
}

fn update_symmetric_builder(
    builder: *mut kmipkit_symmetric_key_builder_t,
    update: impl FnOnce(SymmetricBuilder) -> SymmetricBuilder,
) -> FfiResult<()> {
    let handle = reference_handle(builder, Kind::SymmetricKeyBuilder)?;
    let HandleValue::SymmetricKeyBuilder(builder) = &handle.value else {
        return Err(ERROR_INVALID_INPUT);
    };
    let mut slot = builder.lock().map_err(|_| ERROR_INTERNAL)?;
    let current = slot.take().ok_or(ERROR_INVALID_INPUT)?;
    *slot = Some(update(current));
    Ok(())
}

fn symmetric_builder_request(
    request: *const kmipkit_symmetric_key_request_v1_t,
) -> FfiResult<SymmetricBuilder> {
    if request.is_null() || !request.is_aligned() {
        return Err(ERROR_INVALID_INPUT);
    }
    // SAFETY: a non-null pointer must reference one readable v1 request record.
    let request = unsafe { request.read() };
    if (request.struct_size as usize) < std::mem::size_of::<kmipkit_symmetric_key_request_v1_t>()
        || request.abi_version != 1
        || request.reserved_0 != 0
        || request.reserved_1 != 0
        || request.reserved_2 != 0
    {
        return Err(ERROR_INVALID_INPUT);
    }
    let builder = SymmetricBuilder::detached()
        .algorithm(CryptographicAlgorithm::from_raw(
            request.cryptographic_algorithm,
        ))
        .length(request.cryptographic_length)
        .usage_mask(kmipkit_client::fluent::CryptographicUsageMask::from_raw(
            request.cryptographic_usage_mask,
        ));
    match request.name_present {
        0 if request.name.data.is_null() && request.name.length == 0 => Ok(builder),
        1 => {
            let name = input_span(request.name.data, request.name.length, MAX_FFI_INPUT_BYTES)?;
            let name = std::str::from_utf8(name).map_err(|_| ERROR_INVALID_INPUT)?;
            Ok(builder.name(name))
        }
        _ => Err(ERROR_INVALID_INPUT),
    }
}

fn execute_simple_request(
    client: *mut kmipkit_client_t,
    request: ClientRequest,
    limits: *mut kmipkit_codec_limits_t,
    options: Option<&kmipkit_request_options_v1_t>,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    let options = match options {
        Some(options) => match super::request_options_from_v1(options) {
            Ok(options) => options,
            Err(status) => return record_ffi_error(status),
        },
        None => RequestOptions::default(),
    };
    execute_fluent_item(client, request, limits, &options, out_response)
}

/// Executes one semantic Create Symmetric Key request.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_create_symmetric_key_v1(
    client: *mut kmipkit_client_t,
    request: *const kmipkit_symmetric_key_request_v1_t,
    limits: *mut kmipkit_codec_limits_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        let builder = match symmetric_builder_request(request) {
            Ok(builder) => builder,
            Err(status) => return record_ffi_error(status),
        };
        let request = match builder.build() {
            Ok(request) => ClientRequest::Create(request),
            Err(_) => return record_ffi_error(ERROR_INVALID_INPUT),
        };
        execute_simple_request(client, request, limits, None, out_response)
    })
}

/// Executes Create Symmetric Key with explicit exchange-local timeout options.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_create_symmetric_key_with_options_v1(
    client: *mut kmipkit_client_t,
    request: *const kmipkit_symmetric_key_request_v1_t,
    limits: *mut kmipkit_codec_limits_t,
    options: *const kmipkit_request_options_v1_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        if options.is_null() || !options.is_aligned() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        // SAFETY: a non-null options pointer must reference one readable v1 record.
        let options = unsafe { &*options };
        let builder = match symmetric_builder_request(request) {
            Ok(builder) => builder,
            Err(status) => return record_ffi_error(status),
        };
        let request = match builder.build() {
            Ok(request) => ClientRequest::Create(request),
            Err(_) => return record_ffi_error(ERROR_INVALID_INPUT),
        };
        execute_simple_request(client, request, limits, Some(options), out_response)
    })
}

/// Executes Get with an explicit versioned Unique Identifier.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_get_by_id_v1(
    client: *mut kmipkit_client_t,
    unique_identifier: *const kmipkit_unique_identifier_v1_t,
    limits: *mut kmipkit_codec_limits_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        let identifier = match super::super::copy_unique_identifier(unique_identifier) {
            Ok(Some(identifier)) => identifier,
            Ok(None) => return record_ffi_error(ERROR_INVALID_INPUT),
            Err(status) => return record_ffi_error(status),
        };
        execute_simple_request(
            client,
            ClientRequest::Get(GetRequest::new().with_unique_identifier(identifier)),
            limits,
            None,
            out_response,
        )
    })
}

/// Executes Get with explicit exchange-local timeout options.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_get_by_id_with_options_v1(
    client: *mut kmipkit_client_t,
    unique_identifier: *const kmipkit_unique_identifier_v1_t,
    limits: *mut kmipkit_codec_limits_t,
    options: *const kmipkit_request_options_v1_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        if options.is_null() || !options.is_aligned() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        // SAFETY: a non-null options pointer must reference one readable v1 record.
        let options = unsafe { &*options };
        let identifier = match super::super::copy_unique_identifier(unique_identifier) {
            Ok(Some(identifier)) => identifier,
            Ok(None) => return record_ffi_error(ERROR_INVALID_INPUT),
            Err(status) => return record_ffi_error(status),
        };
        execute_simple_request(
            client,
            ClientRequest::Get(GetRequest::new().with_unique_identifier(identifier)),
            limits,
            Some(options),
            out_response,
        )
    })
}

/// Executes Destroy with an explicit versioned Unique Identifier.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_destroy_by_id_v1(
    client: *mut kmipkit_client_t,
    unique_identifier: *const kmipkit_unique_identifier_v1_t,
    limits: *mut kmipkit_codec_limits_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        let identifier = match super::super::copy_unique_identifier(unique_identifier) {
            Ok(Some(identifier)) => identifier,
            Ok(None) => return record_ffi_error(ERROR_INVALID_INPUT),
            Err(status) => return record_ffi_error(status),
        };
        execute_simple_request(
            client,
            ClientRequest::Destroy(DestroyRequest::new(Some(identifier))),
            limits,
            None,
            out_response,
        )
    })
}

/// Executes Destroy with explicit exchange-local timeout options.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_destroy_by_id_with_options_v1(
    client: *mut kmipkit_client_t,
    unique_identifier: *const kmipkit_unique_identifier_v1_t,
    limits: *mut kmipkit_codec_limits_t,
    options: *const kmipkit_request_options_v1_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        if options.is_null() || !options.is_aligned() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        // SAFETY: a non-null options pointer must reference one readable v1 record.
        let options = unsafe { &*options };
        let identifier = match super::super::copy_unique_identifier(unique_identifier) {
            Ok(Some(identifier)) => identifier,
            Ok(None) => return record_ffi_error(ERROR_INVALID_INPUT),
            Err(status) => return record_ffi_error(status),
        };
        execute_simple_request(
            client,
            ClientRequest::Destroy(DestroyRequest::new(Some(identifier))),
            limits,
            Some(options),
            out_response,
        )
    })
}

/// Creates an opaque, reusable Create Symmetric Key builder.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_symmetric_key_builder_create(
    out_builder: *mut *mut kmipkit_symmetric_key_builder_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_builder);
        *out = try_ffi!(try_make_handle(
            Kind::SymmetricKeyBuilder,
            HandleValue::SymmetricKeyBuilder(Mutex::new(Some(SymmetricBuilder::detached()))),
        ));
        SUCCESS
    })
}

/// Sets the raw KMIP Cryptographic Algorithm Enumeration.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_symmetric_key_builder_set_algorithm(
    builder: *mut kmipkit_symmetric_key_builder_t,
    algorithm: u32,
) -> i32 {
    ffi_entrypoint(|| {
        try_ffi!(update_symmetric_builder(builder, |builder| {
            builder.algorithm(CryptographicAlgorithm::from_raw(algorithm))
        }));
        SUCCESS
    })
}

/// Sets the signed KMIP Cryptographic Length in bits.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_symmetric_key_builder_set_length(
    builder: *mut kmipkit_symmetric_key_builder_t,
    length: i32,
) -> i32 {
    ffi_entrypoint(|| {
        try_ffi!(update_symmetric_builder(builder, |builder| builder.length(length)));
        SUCCESS
    })
}

/// Sets the exact signed Cryptographic Usage Mask bits.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_symmetric_key_builder_set_usage_mask(
    builder: *mut kmipkit_symmetric_key_builder_t,
    usage_mask: i32,
) -> i32 {
    ffi_entrypoint(|| {
        try_ffi!(update_symmetric_builder(builder, |builder| {
            builder.usage_mask(kmipkit_client::fluent::CryptographicUsageMask::from_raw(
                usage_mask,
            ))
        }));
        SUCCESS
    })
}

/// Copies a UTF-8 Name value into the builder.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_symmetric_key_builder_set_name(
    builder: *mut kmipkit_symmetric_key_builder_t,
    name_data: *const u8,
    name_length: u64,
) -> i32 {
    ffi_entrypoint(|| {
        let name = try_ffi!(input_span(name_data, name_length, MAX_FFI_INPUT_BYTES));
        let name = try_ffi!(std::str::from_utf8(name).map_err(|_| ERROR_INVALID_INPUT));
        let mut owned = String::new();
        if owned.try_reserve_exact(name.len()).is_err() {
            return record_ffi_error(ERROR_RESOURCE_LIMIT);
        }
        owned.push_str(name);
        try_ffi!(update_symmetric_builder(builder, |builder| builder.name(&owned)));
        SUCCESS
    })
}

/// Adds a validated catalogued attribute from an explicit TTLV Value.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_symmetric_key_builder_add_catalogued_attribute(
    builder: *mut kmipkit_symmetric_key_builder_t,
    tag: u32,
    value: *mut kmipkit_ttlv_value_t,
) -> i32 {
    ffi_entrypoint(|| {
        let value_handle = try_ffi!(reference_handle(value, Kind::Value));
        let HandleValue::Value(value) = &value_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let value = try_ffi!(kmipkit_ttlv::try_clone_value(value).map_err(model_status));
        let raw_tag = try_ffi!(
            RawTag::new(tag)
                .and_then(|raw| raw.try_checked())
                .map_err(|_| ERROR_INVALID_INPUT)
        );
        let item = try_ffi!(Item::new(raw_tag, value).map_err(model_status));
        let item = try_ffi!(super::super::typed_attributes::validate_typed_item(
            tag, item
        ));
        try_ffi!(update_symmetric_builder(builder, |builder| {
            builder.attribute_item(item)
        }));
        SUCCESS
    })
}

/// Adds an explicit TTLV Item; unknown tags retain their encoded type.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_symmetric_key_builder_add_attribute_item(
    builder: *mut kmipkit_symmetric_key_builder_t,
    item: *mut kmipkit_ttlv_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let item = try_ffi!(super::super::copy_required_item_handle(item));
        let raw_tag = item.tag().raw();
        if (0x0054_0000..=0x0054_FFFF).contains(&raw_tag) {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let item = if super::super::typed_attributes_catalog_generated::is_catalogued_attribute_tag(
            raw_tag,
        ) {
            try_ffi!(super::super::typed_attributes::validate_typed_item(
                raw_tag, item
            ))
        } else {
            item
        };
        try_ffi!(update_symmetric_builder(builder, |builder| {
            builder.attribute_item(item)
        }));
        SUCCESS
    })
}

/// Adds a valid Vendor Attribute using explicit vendor, name, and TTLV value fields.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_symmetric_key_builder_add_vendor_attribute(
    builder: *mut kmipkit_symmetric_key_builder_t,
    vendor_data: *const u8,
    vendor_length: u64,
    name_data: *const u8,
    name_length: u64,
    attribute_value: *mut kmipkit_ttlv_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let vendor = try_ffi!(input_span(vendor_data, vendor_length, MAX_FFI_INPUT_BYTES));
        let vendor = try_ffi!(std::str::from_utf8(vendor).map_err(|_| ERROR_INVALID_INPUT));
        let name = try_ffi!(input_span(name_data, name_length, MAX_FFI_INPUT_BYTES));
        let name = try_ffi!(std::str::from_utf8(name).map_err(|_| ERROR_INVALID_INPUT));
        let value = try_ffi!(super::super::copy_required_item_handle(attribute_value));
        let mut vendor_owned = String::new();
        let mut name_owned = String::new();
        if vendor_owned.try_reserve_exact(vendor.len()).is_err()
            || name_owned.try_reserve_exact(name.len()).is_err()
        {
            return record_ffi_error(ERROR_RESOURCE_LIMIT);
        }
        vendor_owned.push_str(vendor);
        name_owned.push_str(name);
        try_ffi!(update_symmetric_builder(builder, |builder| {
            builder.vendor_attribute(&vendor_owned, &name_owned, &value)
        }));
        SUCCESS
    })
}

/// Builds an independently owned Create batch item without consuming the builder.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_symmetric_key_builder_build_batch_item(
    builder: *mut kmipkit_symmetric_key_builder_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let handle = read_handle!(builder, SymmetricKeyBuilder);
        let HandleValue::SymmetricKeyBuilder(builder) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(builder) = builder.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        let Some(builder) = builder.as_ref() else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(request) = builder.build_request() else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = try_ffi!(try_make_handle(
            Kind::BatchItem,
            HandleValue::BatchItem(Box::new(ClientBatchItem::new(ClientRequest::Create(
                request,
            )))),
        ));
        SUCCESS
    })
}

/// Executes the builder with the client's configured timeout policy.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_symmetric_key_builder_execute_v1(
    builder: *mut kmipkit_symmetric_key_builder_t,
    client: *mut kmipkit_client_t,
    limits: *mut kmipkit_codec_limits_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        let handle = read_handle!(builder, SymmetricKeyBuilder);
        let HandleValue::SymmetricKeyBuilder(builder) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(builder) = builder.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        let Some(builder) = builder.as_ref() else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let request = match builder.build_request() {
            Ok(request) => ClientRequest::Create(request),
            Err(_) => return record_ffi_error(ERROR_INVALID_INPUT),
        };
        execute_simple_request(client, request, limits, None, out_response)
    })
}

/// Executes the builder with explicit exchange-local timeout options.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_symmetric_key_builder_execute_with_options_v1(
    builder: *mut kmipkit_symmetric_key_builder_t,
    client: *mut kmipkit_client_t,
    limits: *mut kmipkit_codec_limits_t,
    options: *const kmipkit_request_options_v1_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        if options.is_null() || !options.is_aligned() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        // SAFETY: a non-null options pointer must reference one readable v1 record.
        let options = unsafe { &*options };
        let handle = read_handle!(builder, SymmetricKeyBuilder);
        let HandleValue::SymmetricKeyBuilder(builder) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(builder) = builder.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        let Some(builder) = builder.as_ref() else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let request = match builder.build_request() {
            Ok(request) => ClientRequest::Create(request),
            Err(_) => return record_ffi_error(ERROR_INVALID_INPUT),
        };
        execute_simple_request(client, request, limits, Some(options), out_response)
    })
}

/// Releases the opaque builder.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_symmetric_key_builder_release(
    builder: *mut kmipkit_symmetric_key_builder_t,
) {
    ffi_entrypoint(|| release(builder, Kind::SymmetricKeyBuilder));
}
