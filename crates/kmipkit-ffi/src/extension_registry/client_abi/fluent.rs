//! Stable C adapters for the Rust-owned fluent request builders.

use super::*;
use std::sync::Arc;

type SymmetricBuilder = kmipkit_client::fluent::CreateSymmetricKeyBuilder<'static>;
type KeyPairBuilder = kmipkit_client::fluent::CreateKeyPairBuilder<'static>;
type SetAttributeBuilder = kmipkit_client::fluent::SetAttributeBuilder<'static>;

fn symmetric_builder_validation_status(operation: impl FnOnce() -> i32) -> i32 {
    let status = operation();
    if status != SUCCESS {
        LAST_ERROR.with(|last_error| {
            let mut error = last_error.get();
            // These two local builder calls cannot reach transport. Preserve
            // the generic recorder for other operations and error categories.
            if error.status == status && error.category == ERROR_CATEGORY_VALIDATION {
                error.delivery = DELIVERY_STATE_NOT_SENT;
                last_error.set(error);
            }
        });
    }
    status
}

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

/// Creates a client using the compact DER mutual TLS configuration.
///
/// # Safety
///
/// `configuration` must be null or a live configuration handle. `input` must
/// point to a readable, aligned v1 record whose non-empty spans and certificate
/// array remain readable for this call. `out_client` must point to an aligned,
/// writable handle slot that does not overlap the input record.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kmipkit_client_create_mtls_v1(
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
            .with_wire_encoding(configuration.wire_encoding())
            .with_protocol_version(configuration.protocol_version());
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

pub(super) fn execute_fluent_item(
    client: *mut kmipkit_client_t,
    request: ClientRequest,
    limits: *mut kmipkit_codec_limits_t,
    options: &RequestOptions,
    asynchronous_indicator: Option<u32>,
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
        let mut batch = ClientBatch::new(ClientBatchItem::new(request));
        if let Some(indicator) = asynchronous_indicator {
            batch = batch.with_asynchronous_indicator(indicator);
        }
        match client.execute_with_options(batch, limits, options) {
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

fn update_key_pair_builder(
    builder: *mut kmipkit_create_key_pair_builder_t,
    update: impl FnOnce(KeyPairBuilder) -> KeyPairBuilder,
) -> FfiResult<()> {
    let handle = reference_handle(builder, Kind::CreateKeyPairBuilder)?;
    let HandleValue::CreateKeyPairBuilder(builder) = &handle.value else {
        return Err(ERROR_INVALID_INPUT);
    };
    let mut slot = builder.lock().map_err(|_| ERROR_INTERNAL)?;
    let mut current = slot.take().ok_or(ERROR_INVALID_INPUT)?;
    current.builder = update(current.builder);
    *slot = Some(current);
    Ok(())
}

fn update_set_attribute_builder(
    builder: *mut kmipkit_set_attribute_builder_t,
    update: impl FnOnce(SetAttributeBuilder) -> SetAttributeBuilder,
) -> FfiResult<()> {
    let handle = reference_handle(builder, Kind::SetAttributeBuilder)?;
    let HandleValue::SetAttributeBuilder(builder) = &handle.value else {
        return Err(ERROR_INVALID_INPUT);
    };
    let mut slot = builder.lock().map_err(|_| ERROR_INTERNAL)?;
    let mut current = slot.take().ok_or(ERROR_INVALID_INPUT)?;
    current.builder = update(current.builder);
    *slot = Some(current);
    Ok(())
}

fn set_attribute_request(
    builder: *mut kmipkit_set_attribute_builder_t,
) -> FfiResult<(SetAttributeRequest, Option<u32>)> {
    let handle = reference_handle(builder, Kind::SetAttributeBuilder)?;
    let HandleValue::SetAttributeBuilder(builder) = &handle.value else {
        return Err(ERROR_INVALID_INPUT);
    };
    let slot = builder.lock().map_err(|_| ERROR_INTERNAL)?;
    let state = slot.as_ref().ok_or(ERROR_INVALID_INPUT)?;
    let request = state
        .builder
        .build_request()
        .map_err(|_| ERROR_INVALID_INPUT)?;
    Ok((request, state.asynchronous_indicator))
}

fn key_pair_attribute_set(item: Item) -> FfiResult<AttributeSet> {
    AttributeSet::try_new([item]).map_err(|_| ERROR_INVALID_INPUT)
}

fn clone_fluent_value(value: ValueView<'_>) -> Result<Value, kmipkit_ttlv::ModelError> {
    match value {
        ValueView::Structure(structure) => Ok(Value::structure(structure.try_clone()?)),
        _ => super::super::typed_attributes::clone_scalar_value_view(&value),
    }
}

fn update_key_pair_attribute(
    builder: *mut kmipkit_create_key_pair_builder_t,
    group: u32,
    item: Item,
) -> FfiResult<()> {
    if group > 2 {
        return Err(ERROR_INVALID_INPUT);
    }
    let attributes = key_pair_attribute_set(item)?;
    match group {
        0 => update_key_pair_builder(builder, |builder| builder.common_attributes(attributes)),
        1 => update_key_pair_builder(builder, |builder| {
            builder.private_key_attributes(attributes)
        }),
        2 => update_key_pair_builder(builder, |builder| builder.public_key_attributes(attributes)),
        _ => Err(ERROR_INVALID_INPUT),
    }
}

fn key_pair_request(
    builder: *mut kmipkit_create_key_pair_builder_t,
) -> FfiResult<(CreateKeyPairRequest, Option<u32>)> {
    let handle = reference_handle(builder, Kind::CreateKeyPairBuilder)?;
    let HandleValue::CreateKeyPairBuilder(builder) = &handle.value else {
        return Err(ERROR_INVALID_INPUT);
    };
    let slot = builder.lock().map_err(|_| ERROR_INTERNAL)?;
    let state = slot.as_ref().ok_or(ERROR_INVALID_INPUT)?;
    let request = state
        .builder
        .build_request()
        .map_err(|_| ERROR_INVALID_INPUT)?;
    Ok((request, state.asynchronous_indicator))
}

fn symmetric_key_create_request(
    builder: *mut kmipkit_symmetric_key_builder_t,
) -> FfiResult<ClientRequest> {
    let handle = reference_handle(builder, Kind::SymmetricKeyBuilder)?;
    let HandleValue::SymmetricKeyBuilder(builder) = &handle.value else {
        return Err(ERROR_INVALID_INPUT);
    };
    let slot = builder.lock().map_err(|_| ERROR_INTERNAL)?;
    let builder = slot.as_ref().ok_or(ERROR_INVALID_INPUT)?;
    let request = builder.build_request().map_err(|_| ERROR_INVALID_INPUT)?;
    Ok(ClientRequest::Create(request))
}

fn execute_key_pair_builder(
    builder: *mut kmipkit_create_key_pair_builder_t,
    client: *mut kmipkit_client_t,
    limits: *mut kmipkit_codec_limits_t,
    options: Option<&kmipkit_request_options_v1_t>,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    let (request, asynchronous_indicator) = match key_pair_request(builder) {
        Ok(parts) => parts,
        Err(status) => return record_ffi_error(status),
    };
    let options = match options {
        Some(options) => match super::request_options_from_v1(options) {
            Ok(options) => options,
            Err(status) => return record_ffi_error(status),
        },
        None => RequestOptions::default(),
    };
    execute_fluent_item(
        client,
        ClientRequest::CreateKeyPair(request),
        limits,
        &options,
        asynchronous_indicator,
        out_response,
    )
}

fn execute_set_attribute_builder(
    builder: *mut kmipkit_set_attribute_builder_t,
    client: *mut kmipkit_client_t,
    limits: *mut kmipkit_codec_limits_t,
    options: Option<&kmipkit_request_options_v1_t>,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    let (request, asynchronous_indicator) = match set_attribute_request(builder) {
        Ok(parts) => parts,
        Err(status) => return record_ffi_error(status),
    };
    let options = match options {
        Some(options) => match super::request_options_from_v1(options) {
            Ok(options) => options,
            Err(status) => return record_ffi_error(status),
        },
        None => RequestOptions::default(),
    };
    execute_fluent_item(
        client,
        ClientRequest::SetAttribute(request),
        limits,
        &options,
        asynchronous_indicator,
        out_response,
    )
}

fn key_pair_vendor_attribute(
    vendor_data: *const u8,
    vendor_length: u64,
    name_data: *const u8,
    name_length: u64,
    value_handle: *mut kmipkit_ttlv_item_t,
) -> FfiResult<Item> {
    const VENDOR_ATTRIBUTE_TAG: u32 = 0x0042_0008;
    const ATTRIBUTE_NAME_TAG: u32 = 0x0042_000A;
    const ATTRIBUTE_VALUE_TAG: u32 = 0x0042_000B;
    const VENDOR_IDENTIFICATION_TAG: u32 = 0x0042_009D;

    let vendor = input_span(vendor_data, vendor_length, MAX_FFI_INPUT_BYTES)?;
    let vendor = std::str::from_utf8(vendor).map_err(|_| ERROR_INVALID_INPUT)?;
    let name = input_span(name_data, name_length, MAX_FFI_INPUT_BYTES)?;
    let name = std::str::from_utf8(name).map_err(|_| ERROR_INVALID_INPUT)?;
    let value = super::super::copy_required_item_handle(value_handle)?;
    let mut fields = Structure::new();
    fields
        .try_push(
            Item::new(
                RawTag::new(VENDOR_IDENTIFICATION_TAG)
                    .and_then(|tag| tag.try_checked())
                    .map_err(|_| ERROR_INVALID_INPUT)?,
                Value::text_string(vendor.to_owned()),
            )
            .map_err(model_status)?,
        )
        .map_err(model_status)?;
    fields
        .try_push(
            Item::new(
                RawTag::new(ATTRIBUTE_NAME_TAG)
                    .and_then(|tag| tag.try_checked())
                    .map_err(|_| ERROR_INVALID_INPUT)?,
                Value::text_string(name.to_owned()),
            )
            .map_err(model_status)?,
        )
        .map_err(model_status)?;
    fields
        .try_push(
            Item::new(
                RawTag::new(ATTRIBUTE_VALUE_TAG)
                    .and_then(|tag| tag.try_checked())
                    .map_err(|_| ERROR_INVALID_INPUT)?,
                value.with_value(clone_fluent_value).map_err(model_status)?,
            )
            .map_err(model_status)?,
        )
        .map_err(model_status)?;
    Item::new(
        RawTag::new(VENDOR_ATTRIBUTE_TAG)
            .and_then(|tag| tag.try_checked())
            .map_err(|_| ERROR_INVALID_INPUT)?,
        Value::structure(fields),
    )
    .map_err(model_status)
}

fn validated_expert_attribute_item(item: *mut kmipkit_ttlv_item_t) -> FfiResult<Item> {
    let item = super::super::copy_required_item_handle(item)?;
    let tag = item.tag().raw();
    if super::super::typed_attributes_catalog_generated::is_catalogued_attribute_tag(tag) {
        super::super::typed_attributes::validate_typed_item(tag, item)
    } else {
        Ok(item)
    }
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
    execute_fluent_item(client, request, limits, &options, None, out_response)
}

fn fluent_get_attributes_request(
    unique_identifier: *const kmipkit_unique_identifier_v1_t,
    attribute_references: *const crate::kmipkit_attribute_reference_v1_t,
    attribute_reference_count: u64,
) -> FfiResult<ClientRequest> {
    let unique_identifier = super::super::copy_optional_text_identifier(unique_identifier)?
        .ok_or(ERROR_INVALID_INPUT)?;
    let maximum = MAX_GET_ATTRIBUTES_REFERENCES;
    let source = input_array(attribute_references, attribute_reference_count, maximum)?;
    let mut references = Vec::new();
    references
        .try_reserve_exact(source.len())
        .map_err(|_| ERROR_RESOURCE_LIMIT)?;
    for reference in source {
        let reference_pointer = ptr::from_ref(reference);
        let copied_reference = super::super::copy_attribute_reference(reference_pointer)?;
        references.push(copied_reference);
    }
    let request = GetAttributesRequest::try_new(Some(unique_identifier), references)
        .map_err(|_| ERROR_INVALID_INPUT)?;
    Ok(ClientRequest::get_attributes(request))
}

fn fluent_get_attribute_list_request(
    unique_identifier: *const kmipkit_unique_identifier_v1_t,
) -> FfiResult<ClientRequest> {
    let unique_identifier = super::super::copy_optional_text_identifier(unique_identifier)?
        .ok_or(ERROR_INVALID_INPUT)?;
    Ok(ClientRequest::get_attribute_list(
        GetAttributeListRequest::new(Some(unique_identifier)),
    ))
}

/// Executes one typed Get Attributes request with ordered references.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_get_attributes_v1(
    client: *mut kmipkit_client_t,
    unique_identifier: *const kmipkit_unique_identifier_v1_t,
    attribute_references: *const crate::kmipkit_attribute_reference_v1_t,
    attribute_reference_count: u64,
    limits: *mut kmipkit_codec_limits_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        let request = match fluent_get_attributes_request(
            unique_identifier,
            attribute_references,
            attribute_reference_count,
        ) {
            Ok(request) => request,
            Err(status) => return record_ffi_error(status),
        };
        execute_simple_request(client, request, limits, None, out_response)
    })
}

/// Executes Get Attributes with explicit exchange-local timeout options.
///
/// # Safety
///
/// `options` must point to a readable, aligned v1 options record. All input
/// spans and arrays must remain readable for this call, and `out_response`
/// must be a writable aligned output slot that does not overlap the inputs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kmipkit_client_get_attributes_with_options_v1(
    client: *mut kmipkit_client_t,
    unique_identifier: *const kmipkit_unique_identifier_v1_t,
    attribute_references: *const crate::kmipkit_attribute_reference_v1_t,
    attribute_reference_count: u64,
    limits: *mut kmipkit_codec_limits_t,
    options: *const kmipkit_request_options_v1_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        if options.is_null() || !options.is_aligned() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        // SAFETY: the aligned, non-null pointer is borrowed for this call.
        let options = unsafe { &*options };
        let request = match fluent_get_attributes_request(
            unique_identifier,
            attribute_references,
            attribute_reference_count,
        ) {
            Ok(request) => request,
            Err(status) => return record_ffi_error(status),
        };
        execute_simple_request(client, request, limits, Some(options), out_response)
    })
}

/// Executes one typed Get Attribute List request for a required object identifier.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_get_attribute_list_v1(
    client: *mut kmipkit_client_t,
    unique_identifier: *const kmipkit_unique_identifier_v1_t,
    limits: *mut kmipkit_codec_limits_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        let request = match fluent_get_attribute_list_request(unique_identifier) {
            Ok(request) => request,
            Err(status) => return record_ffi_error(status),
        };
        execute_simple_request(client, request, limits, None, out_response)
    })
}

/// Executes Get Attribute List for a required identifier with exchange-local timeout options.
///
/// # Safety
///
/// `options` must point to a readable, aligned v1 options record. `out_response`
/// must be a writable aligned output slot that does not overlap any input.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kmipkit_client_get_attribute_list_with_options_v1(
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
        // SAFETY: the aligned, non-null pointer is borrowed for this call.
        let options = unsafe { &*options };
        let request = match fluent_get_attribute_list_request(unique_identifier) {
            Ok(request) => request,
            Err(status) => return record_ffi_error(status),
        };
        execute_simple_request(client, request, limits, Some(options), out_response)
    })
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
///
/// # Safety
///
/// `client` and `limits` must be null or live handles of the declared types.
/// Every non-null request and options pointer must refer to an aligned,
/// initialized record readable for this call. `out_response` must be an
/// aligned, writable handle slot. Input records and the output slot must not
/// overlap.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kmipkit_client_create_symmetric_key_with_options_v1(
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

/// Executes an identifier-based request with explicit exchange-local options.
///
/// # Safety
///
/// The client and limits must be null or live handles of the declared types.
/// `unique_identifier` and `options` must be readable initialized records for
/// this call, including any referenced byte spans. `out_response` must be an
/// aligned, writable handle slot. Input records and the output slot must not
/// overlap.
unsafe fn execute_identifier_request_with_options(
    client: *mut kmipkit_client_t,
    unique_identifier: *const kmipkit_unique_identifier_v1_t,
    limits: *mut kmipkit_codec_limits_t,
    options: *const kmipkit_request_options_v1_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
    request_for_identifier: impl FnOnce(UniqueIdentifier) -> ClientRequest,
) -> i32 {
    if options.is_null() || !options.is_aligned() {
        return record_ffi_error(ERROR_INVALID_INPUT);
    }
    // SAFETY: the caller guarantees that options points to a readable v1 record.
    let options = unsafe { &*options };
    let identifier = match super::super::copy_unique_identifier(unique_identifier) {
        Ok(Some(identifier)) => identifier,
        Ok(None) => return record_ffi_error(ERROR_INVALID_INPUT),
        Err(status) => return record_ffi_error(status),
    };
    execute_simple_request(
        client,
        request_for_identifier(identifier),
        limits,
        Some(options),
        out_response,
    )
}

/// Executes Get with explicit exchange-local timeout options.
///
/// # Safety
///
/// `client` and `limits` must be null or live handles of the declared types.
/// `unique_identifier` and `options` must point to aligned, initialized
/// records readable for this call, including any referenced byte spans.
/// `out_response` must be an aligned, writable handle slot. Input records and
/// the output slot must not overlap.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kmipkit_client_get_by_id_with_options_v1(
    client: *mut kmipkit_client_t,
    unique_identifier: *const kmipkit_unique_identifier_v1_t,
    limits: *mut kmipkit_codec_limits_t,
    options: *const kmipkit_request_options_v1_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        // SAFETY: these exported arguments satisfy the documented pointer contract.
        unsafe {
            execute_identifier_request_with_options(
                client,
                unique_identifier,
                limits,
                options,
                out_response,
                |identifier| {
                    ClientRequest::Get(GetRequest::new().with_unique_identifier(identifier))
                },
            )
        }
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
///
/// # Safety
///
/// `client` and `limits` must be null or live handles of the declared types.
/// `unique_identifier` and `options` must point to aligned, initialized
/// records readable for this call, including any referenced byte spans.
/// `out_response` must be an aligned, writable handle slot. Input records and
/// the output slot must not overlap.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kmipkit_client_destroy_by_id_with_options_v1(
    client: *mut kmipkit_client_t,
    unique_identifier: *const kmipkit_unique_identifier_v1_t,
    limits: *mut kmipkit_codec_limits_t,
    options: *const kmipkit_request_options_v1_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        // SAFETY: these exported arguments satisfy the documented pointer contract.
        unsafe {
            execute_identifier_request_with_options(
                client,
                unique_identifier,
                limits,
                options,
                out_response,
                |identifier| ClientRequest::Destroy(DestroyRequest::new(Some(identifier))),
            )
        }
    })
}

/// Creates a detached, reusable Create Key Pair builder.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_create_key_pair_builder_create(
    out_builder: *mut *mut kmipkit_create_key_pair_builder_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_builder);
        *out = try_ffi!(try_make_handle(
            Kind::CreateKeyPairBuilder,
            HandleValue::CreateKeyPairBuilder(Mutex::new(Some(
                super::super::CreateKeyPairBuilderState {
                    builder: KeyPairBuilder::detached(),
                    asynchronous_indicator: None,
                },
            ))),
        ));
        SUCCESS
    })
}

/// Sets the raw KMIP Cryptographic Algorithm Enumeration.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_create_key_pair_builder_set_algorithm(
    builder: *mut kmipkit_create_key_pair_builder_t,
    algorithm: u32,
) -> i32 {
    ffi_entrypoint(|| {
        try_ffi!(update_key_pair_builder(builder, |builder| {
            builder.algorithm(CryptographicAlgorithm::from_raw(algorithm))
        }));
        SUCCESS
    })
}

/// Sets the signed KMIP Cryptographic Length in bits.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_create_key_pair_builder_set_length(
    builder: *mut kmipkit_create_key_pair_builder_t,
    length: i32,
) -> i32 {
    ffi_entrypoint(|| {
        try_ffi!(update_key_pair_builder(builder, |builder| builder.length(length)));
        SUCCESS
    })
}

/// Sets the explicit private-key Cryptographic Usage Mask bits.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_create_key_pair_builder_set_private_usage_mask(
    builder: *mut kmipkit_create_key_pair_builder_t,
    usage_mask: i32,
) -> i32 {
    ffi_entrypoint(|| {
        try_ffi!(update_key_pair_builder(builder, |builder| {
            builder.private_usage_mask(kmipkit_client::fluent::CryptographicUsageMask::from_raw(
                usage_mask,
            ))
        }));
        SUCCESS
    })
}

/// Sets the explicit public-key Cryptographic Usage Mask bits.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_create_key_pair_builder_set_public_usage_mask(
    builder: *mut kmipkit_create_key_pair_builder_t,
    usage_mask: i32,
) -> i32 {
    ffi_entrypoint(|| {
        try_ffi!(update_key_pair_builder(builder, |builder| {
            builder.public_usage_mask(kmipkit_client::fluent::CryptographicUsageMask::from_raw(
                usage_mask,
            ))
        }));
        SUCCESS
    })
}

fn set_key_pair_name(
    builder: *mut kmipkit_create_key_pair_builder_t,
    name_data: *const u8,
    name_length: u64,
    private_key: bool,
) -> i32 {
    ffi_entrypoint(|| {
        let name = try_ffi!(input_span(name_data, name_length, MAX_FFI_INPUT_BYTES));
        let name = try_ffi!(std::str::from_utf8(name).map_err(|_| ERROR_INVALID_INPUT));
        let mut owned = String::new();
        if owned.try_reserve_exact(name.len()).is_err() {
            return record_ffi_error(ERROR_RESOURCE_LIMIT);
        }
        owned.push_str(name);
        try_ffi!(update_key_pair_builder(builder, |builder| {
            if private_key {
                builder.private_name(&owned)
            } else {
                builder.public_name(&owned)
            }
        }));
        SUCCESS
    })
}

/// Copies a UTF-8 Name to the private-key attributes.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_create_key_pair_builder_set_private_name(
    builder: *mut kmipkit_create_key_pair_builder_t,
    name_data: *const u8,
    name_length: u64,
) -> i32 {
    set_key_pair_name(builder, name_data, name_length, true)
}

/// Copies a UTF-8 Name to the public-key attributes.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_create_key_pair_builder_set_public_name(
    builder: *mut kmipkit_create_key_pair_builder_t,
    name_data: *const u8,
    name_length: u64,
) -> i32 {
    set_key_pair_name(builder, name_data, name_length, false)
}

fn add_key_pair_attribute_item(
    builder: *mut kmipkit_create_key_pair_builder_t,
    group: u32,
    item: *mut kmipkit_ttlv_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let item = try_ffi!(validated_expert_attribute_item(item));
        try_ffi!(update_key_pair_attribute(builder, group, item));
        SUCCESS
    })
}

/// Adds an explicit TTLV Item to Common Attributes.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_create_key_pair_builder_add_common_attribute_item(
    builder: *mut kmipkit_create_key_pair_builder_t,
    item: *mut kmipkit_ttlv_item_t,
) -> i32 {
    add_key_pair_attribute_item(builder, 0, item)
}

/// Adds an explicit TTLV Item to Private Key Attributes.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_create_key_pair_builder_add_private_attribute_item(
    builder: *mut kmipkit_create_key_pair_builder_t,
    item: *mut kmipkit_ttlv_item_t,
) -> i32 {
    add_key_pair_attribute_item(builder, 1, item)
}

/// Adds an explicit TTLV Item to Public Key Attributes.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_create_key_pair_builder_add_public_attribute_item(
    builder: *mut kmipkit_create_key_pair_builder_t,
    item: *mut kmipkit_ttlv_item_t,
) -> i32 {
    add_key_pair_attribute_item(builder, 2, item)
}

fn add_key_pair_catalogued_attribute(
    builder: *mut kmipkit_create_key_pair_builder_t,
    group: u32,
    tag: u32,
    value: *mut kmipkit_ttlv_value_t,
) -> i32 {
    ffi_entrypoint(|| {
        let item = try_ffi!(super::super::typed_attributes::catalogued_item_from_value(
            tag,
            value,
            model_status,
        ));
        try_ffi!(update_key_pair_attribute(builder, group, item));
        SUCCESS
    })
}

/// Adds a catalogued primitive attribute to Common Attributes after type validation.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_create_key_pair_builder_add_common_catalogued_attribute(
    builder: *mut kmipkit_create_key_pair_builder_t,
    tag: u32,
    value: *mut kmipkit_ttlv_value_t,
) -> i32 {
    add_key_pair_catalogued_attribute(builder, 0, tag, value)
}

/// Adds a catalogued primitive attribute to Private Key Attributes after type validation.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_create_key_pair_builder_add_private_catalogued_attribute(
    builder: *mut kmipkit_create_key_pair_builder_t,
    tag: u32,
    value: *mut kmipkit_ttlv_value_t,
) -> i32 {
    add_key_pair_catalogued_attribute(builder, 1, tag, value)
}

/// Adds a catalogued primitive attribute to Public Key Attributes after type validation.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_create_key_pair_builder_add_public_catalogued_attribute(
    builder: *mut kmipkit_create_key_pair_builder_t,
    tag: u32,
    value: *mut kmipkit_ttlv_value_t,
) -> i32 {
    add_key_pair_catalogued_attribute(builder, 2, tag, value)
}

fn add_key_pair_vendor_attribute(
    builder: *mut kmipkit_create_key_pair_builder_t,
    group: u32,
    vendor_data: *const u8,
    vendor_length: u64,
    name_data: *const u8,
    name_length: u64,
    value: *mut kmipkit_ttlv_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let item = try_ffi!(key_pair_vendor_attribute(
            vendor_data,
            vendor_length,
            name_data,
            name_length,
            value,
        ));
        try_ffi!(update_key_pair_attribute(builder, group, item));
        SUCCESS
    })
}

/// Builds and appends a KMIP Vendor Attribute to Common Attributes.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_create_key_pair_builder_add_common_vendor_attribute(
    builder: *mut kmipkit_create_key_pair_builder_t,
    vendor_data: *const u8,
    vendor_length: u64,
    name_data: *const u8,
    name_length: u64,
    value: *mut kmipkit_ttlv_item_t,
) -> i32 {
    add_key_pair_vendor_attribute(
        builder,
        0,
        vendor_data,
        vendor_length,
        name_data,
        name_length,
        value,
    )
}

/// Builds and appends a KMIP Vendor Attribute to Private Key Attributes.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_create_key_pair_builder_add_private_vendor_attribute(
    builder: *mut kmipkit_create_key_pair_builder_t,
    vendor_data: *const u8,
    vendor_length: u64,
    name_data: *const u8,
    name_length: u64,
    value: *mut kmipkit_ttlv_item_t,
) -> i32 {
    add_key_pair_vendor_attribute(
        builder,
        1,
        vendor_data,
        vendor_length,
        name_data,
        name_length,
        value,
    )
}

/// Builds and appends a KMIP Vendor Attribute to Public Key Attributes.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_create_key_pair_builder_add_public_vendor_attribute(
    builder: *mut kmipkit_create_key_pair_builder_t,
    vendor_data: *const u8,
    vendor_length: u64,
    name_data: *const u8,
    name_length: u64,
    value: *mut kmipkit_ttlv_item_t,
) -> i32 {
    add_key_pair_vendor_attribute(
        builder,
        2,
        vendor_data,
        vendor_length,
        name_data,
        name_length,
        value,
    )
}

/// Sets the Request Header Asynchronous Indicator for the next exchange.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_create_key_pair_builder_set_asynchronous_indicator(
    builder: *mut kmipkit_create_key_pair_builder_t,
    value: u32,
) -> i32 {
    ffi_entrypoint(|| {
        let handle = try_ffi!(reference_handle(builder, Kind::CreateKeyPairBuilder));
        let HandleValue::CreateKeyPairBuilder(builder) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(mut slot) = builder.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        let Some(state) = slot.as_mut() else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        state.asynchronous_indicator = Some(value);
        SUCCESS
    })
}

/// Validates the explicit Create Key Pair inputs without sending a request.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_create_key_pair_builder_validate(
    builder: *mut kmipkit_create_key_pair_builder_t,
) -> i32 {
    ffi_entrypoint(|| {
        let _ = try_ffi!(key_pair_request(builder));
        SUCCESS
    })
}

/// Builds an independently owned typed Create Key Pair batch item.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_create_key_pair_builder_build_batch_item(
    builder: *mut kmipkit_create_key_pair_builder_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let (request, _) = match key_pair_request(builder) {
            Ok(parts) => parts,
            Err(status) => return record_ffi_error(status),
        };
        *out = try_ffi!(try_make_handle(
            Kind::BatchItem,
            HandleValue::BatchItem(Box::new(ClientBatchItem::new(
                ClientRequest::CreateKeyPair(request)
            ))),
        ));
        SUCCESS
    })
}

/// Executes the typed request with the client's default exchange options.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_create_key_pair_builder_execute_v1(
    builder: *mut kmipkit_create_key_pair_builder_t,
    client: *mut kmipkit_client_t,
    limits: *mut kmipkit_codec_limits_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    ffi_entrypoint(|| execute_key_pair_builder(builder, client, limits, None, out_response))
}

/// Executes the typed request with explicit exchange-local options.
///
/// # Safety
///
/// `options` must reference an aligned, initialized v1 record. All handles
/// and the output pointer must satisfy their declared C ABI contracts.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kmipkit_create_key_pair_builder_execute_with_options_v1(
    builder: *mut kmipkit_create_key_pair_builder_t,
    client: *mut kmipkit_client_t,
    limits: *mut kmipkit_codec_limits_t,
    options: *const kmipkit_request_options_v1_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        if options.is_null() || !options.is_aligned() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        // SAFETY: The non-null aligned pointer must reference one readable v1 record.
        let options = unsafe { &*options };
        execute_key_pair_builder(builder, client, limits, Some(options), out_response)
    })
}

/// Releases the opaque Create Key Pair builder.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_create_key_pair_builder_release(
    builder: *mut kmipkit_create_key_pair_builder_t,
) {
    ffi_entrypoint(|| release(builder, Kind::CreateKeyPairBuilder));
}

/// Creates a detached, reusable Set Attribute builder.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_set_attribute_builder_create(
    out_builder: *mut *mut kmipkit_set_attribute_builder_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_builder);
        *out = try_ffi!(try_make_handle(
            Kind::SetAttributeBuilder,
            HandleValue::SetAttributeBuilder(Mutex::new(Some(
                super::super::SetAttributeBuilderState {
                    builder: SetAttributeBuilder::detached(),
                    asynchronous_indicator: None,
                },
            ))),
        ));
        SUCCESS
    })
}

/// Sets an optional UTF-8 Unique Identifier.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_set_attribute_builder_set_unique_identifier(
    builder: *mut kmipkit_set_attribute_builder_t,
    value_data: *const u8,
    value_length: u64,
) -> i32 {
    ffi_entrypoint(|| {
        let value = match input_span(value_data, value_length, MAX_FFI_INPUT_BYTES) {
            Ok(value) => match std::str::from_utf8(value) {
                Ok(value) => value,
                Err(_) => return record_ffi_error(ERROR_INVALID_INPUT),
            },
            Err(status) => return record_ffi_error(status),
        };
        try_ffi!(update_set_attribute_builder(builder, |builder| {
            builder.unique_identifier(value)
        }));
        SUCCESS
    })
}

/// Adds one explicitly typed TTLV Item as the New Attribute.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_set_attribute_builder_add_attribute_item(
    builder: *mut kmipkit_set_attribute_builder_t,
    item: *mut kmipkit_ttlv_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let item = try_ffi!(super::super::copy_required_item_handle(item));
        try_ffi!(update_set_attribute_builder(builder, |builder| {
            builder.attribute_item(item)
        }));
        SUCCESS
    })
}

/// Adds a catalogued primitive attribute after validating its TTLV type.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_set_attribute_builder_add_catalogued_attribute(
    builder: *mut kmipkit_set_attribute_builder_t,
    tag: u32,
    value: *mut kmipkit_ttlv_value_t,
) -> i32 {
    ffi_entrypoint(|| {
        let item = try_ffi!(super::super::typed_attributes::catalogued_item_from_value(
            tag,
            value,
            model_status,
        ));
        try_ffi!(update_set_attribute_builder(builder, |builder| {
            builder.attribute_item(item)
        }));
        SUCCESS
    })
}

/// Adds a KMIP Vendor Attribute with an explicit typed value Item.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_set_attribute_builder_add_vendor_attribute(
    builder: *mut kmipkit_set_attribute_builder_t,
    vendor_data: *const u8,
    vendor_length: u64,
    name_data: *const u8,
    name_length: u64,
    value: *mut kmipkit_ttlv_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let item = try_ffi!(key_pair_vendor_attribute(
            vendor_data,
            vendor_length,
            name_data,
            name_length,
            value,
        ));
        try_ffi!(update_set_attribute_builder(builder, |builder| {
            builder.attribute_item(item)
        }));
        SUCCESS
    })
}

/// Sets the Request Header Asynchronous Indicator.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_set_attribute_builder_set_asynchronous_indicator(
    builder: *mut kmipkit_set_attribute_builder_t,
    value: u32,
) -> i32 {
    ffi_entrypoint(|| {
        let handle = try_ffi!(reference_handle(builder, Kind::SetAttributeBuilder));
        let HandleValue::SetAttributeBuilder(builder) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(mut slot) = builder.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        let Some(mut state) = slot.take() else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        state.builder = state.builder.with_asynchronous_indicator(value);
        state.asynchronous_indicator = Some(value);
        *slot = Some(state);
        SUCCESS
    })
}

/// Validates the typed Set Attribute request without transport.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_set_attribute_builder_validate(
    builder: *mut kmipkit_set_attribute_builder_t,
) -> i32 {
    ffi_entrypoint(|| {
        symmetric_builder_validation_status(|| match set_attribute_request(builder) {
            Ok(_) => SUCCESS,
            Err(status) => record_ffi_error(status),
        })
    })
}

/// Builds an independently owned typed Set Attribute request.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_set_attribute_builder_build_request(
    builder: *mut kmipkit_set_attribute_builder_t,
    out_request: *mut *mut kmipkit_set_attribute_request_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_request);
        let (request, _) = match set_attribute_request(builder) {
            Ok(parts) => parts,
            Err(status) => return record_ffi_error(status),
        };
        *out = try_ffi!(try_make_handle(
            Kind::SetAttributeRequest,
            HandleValue::SetAttributeRequest(Box::new(request)),
        ));
        SUCCESS
    })
}

/// Builds an independently owned typed Set Attribute batch item.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_set_attribute_builder_build_batch_item(
    builder: *mut kmipkit_set_attribute_builder_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let (request, _) = match set_attribute_request(builder) {
            Ok(parts) => parts,
            Err(status) => return record_ffi_error(status),
        };
        *out = try_ffi!(try_make_handle(
            Kind::BatchItem,
            HandleValue::BatchItem(Box::new(ClientBatchItem::new(ClientRequest::SetAttribute(
                request
            ),))),
        ));
        SUCCESS
    })
}

/// Executes the typed request with the client's default exchange options.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_set_attribute_builder_execute_v1(
    builder: *mut kmipkit_set_attribute_builder_t,
    client: *mut kmipkit_client_t,
    limits: *mut kmipkit_codec_limits_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    ffi_entrypoint(|| execute_set_attribute_builder(builder, client, limits, None, out_response))
}

/// Executes the typed request with explicit exchange-local options.
///
/// # Safety
/// `options` must reference an aligned, initialized v1 record.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kmipkit_set_attribute_builder_execute_with_options_v1(
    builder: *mut kmipkit_set_attribute_builder_t,
    client: *mut kmipkit_client_t,
    limits: *mut kmipkit_codec_limits_t,
    options: *const kmipkit_request_options_v1_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        if options.is_null() || !options.is_aligned() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        // SAFETY: The non-null aligned pointer must reference one readable v1 record.
        let options = unsafe { &*options };
        execute_set_attribute_builder(builder, client, limits, Some(options), out_response)
    })
}

/// Releases the opaque Set Attribute builder.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_set_attribute_builder_release(
    builder: *mut kmipkit_set_attribute_builder_t,
) {
    ffi_entrypoint(|| release(builder, Kind::SetAttributeBuilder));
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
        let item = try_ffi!(super::super::typed_attributes::catalogued_item_from_value(
            tag,
            value,
            model_status,
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
        let item = try_ffi!(validated_expert_attribute_item(item));
        try_ffi!(update_symmetric_builder(builder, |builder| {
            builder.attribute_item(item)
        }));
        SUCCESS
    })
}

/// Inserts an explicit expert attribute at its final Object Attributes index.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_symmetric_key_builder_insert_attribute_item(
    builder: *mut kmipkit_symmetric_key_builder_t,
    index: u64,
    item: *mut kmipkit_ttlv_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        symmetric_builder_validation_status(|| {
            let item = try_ffi!(validated_expert_attribute_item(item));
            let Ok(index) = usize::try_from(index) else {
                return record_ffi_error(ERROR_INVALID_INPUT); // COVERAGE-EXCLUDE: The guard is unreachable on tested 64-bit targets.
            };
            try_ffi!(update_symmetric_builder(builder, |builder| {
                builder.attribute_at(
                    kmipkit_client::fluent::FluentAttributePosition::ObjectAttributes { index },
                    item,
                )
            }));
            SUCCESS
        })
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
        symmetric_builder_validation_status(|| {
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
        let request = match symmetric_key_create_request(builder) {
            Ok(request) => request,
            Err(status) => return record_ffi_error(status),
        };
        execute_simple_request(client, request, limits, None, out_response)
    })
}

/// Executes the builder with explicit exchange-local timeout options.
///
/// # Safety
///
/// `builder`, `client`, and `limits` must be null or live handles of the
/// declared types. `options` must point to an aligned, initialized v1 record
/// readable for this call. `out_response` must be an aligned, writable handle
/// slot. The options record and output slot must not overlap.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kmipkit_symmetric_key_builder_execute_with_options_v1(
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
        let request = match symmetric_key_create_request(builder) {
            Ok(request) => request,
            Err(status) => return record_ffi_error(status),
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
