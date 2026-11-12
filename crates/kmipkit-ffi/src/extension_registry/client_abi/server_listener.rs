//! C ABI for TLS listeners receiving typed KMIP server-to-client requests.
#![allow(clippy::missing_safety_doc, clippy::too_many_arguments)]

use super::*;
use crate::kmipkit_attribute_reference_v1_t;
use std::time::Duration;

const SERVER_LISTENER_RAW_TLS: u32 = 1;
const SERVER_LISTENER_HTTPS: u32 = 2;
const SERVER_LISTENER_ENCODING_TTLV: u32 = 1;
const SERVER_LISTENER_ENCODING_JSON: u32 = 2;
const SERVER_LISTENER_ENCODING_XML: u32 = 3;
const SERVER_OPERATION_DISCOVER_VERSIONS: u32 = 0x0000_001e;
const SERVER_OPERATION_NOTIFY: u32 = 0x0000_001b;
const SERVER_OPERATION_PUT: u32 = 0x0000_001c;
const SERVER_OPERATION_QUERY: u32 = 0x0000_0018;
const SERVER_OPERATION_SET_ENDPOINT_ROLE: u32 = 0x0000_0032;
const SERVER_PUT_OBJECT_KEY_BLOCK: u32 = 0x0042_0040;
const SERVER_KEY_BLOCK_FORMAT: u32 = 0x0042_0042;
const SERVER_KEY_BLOCK_KEY_VALUE: u32 = 0x0042_0045;
const SERVER_KEY_BLOCK_WRAPPING_DATA: u32 = 0x0042_0046;
const SERVER_KEY_VALUE_KEY_MATERIAL: u32 = 0x0042_0043;
const SERVER_READ_PRESENT: u32 = 1;
const SERVER_READ_TYPE: u32 = 2;
const SERVER_READ_ENUMERATION: u32 = 3;
const SERVER_READ_INTEGER: u32 = 4;
const SERVER_READ_SECRET_BYTES: u32 = 5;
const SERVER_READ_STRUCTURE_VIEW: u32 = 6;
const SERVER_READ_SECRET_TEXT: u32 = 7;
const SERVER_NOTIFY_VENDOR_IDENTIFICATION: u32 = 0x0042_009D;
const SERVER_NOTIFY_ATTRIBUTE_NAME: u32 = 0x0042_000A;

fn listener_config_from_v1(
    input: *const kmipkit_server_listener_config_v1_t,
) -> FfiResult<kmipkit_transport::ListenerTransportConfig> {
    if input.is_null() {
        return Err(ERROR_INVALID_INPUT);
    }
    // SAFETY: the caller supplies a readable versioned config for this call.
    let input = unsafe { &*input };
    let required_size = u32::try_from(std::mem::size_of::<kmipkit_server_listener_config_v1_t>())
        .map_err(|_| ERROR_INVALID_INPUT)?;
    if input.abi_version != 1
        || input.struct_size < required_size
        || input.reserved != 0
        || input.reserved2 != 0
        || !matches!(
            input.transport_kind,
            SERVER_LISTENER_RAW_TLS | SERVER_LISTENER_HTTPS
        )
    {
        return Err(ERROR_INVALID_INPUT);
    }

    let host = transport_text(input.bind_host)?;
    let certificates = transport_input_slice(
        input.listener_certificates,
        input.listener_certificate_count,
    )?;
    if certificates.is_empty() {
        return Err(ERROR_INVALID_INPUT);
    }
    let identity = ClientIdentity::new(
        client_certificate_chain(certificates)?,
        private_key_input(&input.listener_private_key)?,
    );

    let trust_certificates =
        transport_input_slice(input.trust_certificates, input.trust_certificate_count)?;
    if trust_certificates.is_empty() {
        return Err(ERROR_INVALID_INPUT);
    }
    let trust_materials = certificate_input_materials(trust_certificates)?;
    let authorities = trust_certificates
        .iter()
        .zip(trust_materials)
        .map(|(source, bytes)| match transport_input_encoding(source)? {
            TRANSPORT_ENCODING_PEM => Ok(CertificateInput::from_pem(bytes)),
            TRANSPORT_ENCODING_DER => Ok(CertificateInput::from_der(vec![bytes])),
            _ => Err(ERROR_INVALID_INPUT),
        })
        .collect::<FfiResult<Vec<_>>>()?;
    let trust = TrustSource::certificate_authorities(authorities);

    let revocation_inputs =
        transport_input_slice(input.revocation_lists, input.revocation_list_count)?;
    validate_transport_input_total(revocation_inputs)?;
    let revocation_lists = revocation_inputs
        .iter()
        .map(revocation_input)
        .collect::<FfiResult<Vec<_>>>()?;

    let mut builder = kmipkit_transport::ListenerTransportConfig::builder(host, input.bind_port)
        .listener_identity(identity)
        .trust_source(trust)
        .revocation_lists(revocation_lists);
    match input.wire_encoding {
        0 if input.transport_kind == SERVER_LISTENER_HTTPS => {}
        0 | SERVER_LISTENER_ENCODING_TTLV if input.transport_kind == SERVER_LISTENER_RAW_TLS => {
            builder = builder.wire_encoding(WireEncoding::Ttlv);
        }
        SERVER_LISTENER_ENCODING_TTLV => builder = builder.wire_encoding(WireEncoding::Ttlv),
        SERVER_LISTENER_ENCODING_JSON => builder = builder.wire_encoding(WireEncoding::Json),
        SERVER_LISTENER_ENCODING_XML => builder = builder.wire_encoding(WireEncoding::Xml),
        _ => return Err(ERROR_INVALID_INPUT),
    }
    if input.target_path.length != 0 || !input.target_path.data.is_null() {
        if input.transport_kind != SERVER_LISTENER_HTTPS {
            return Err(ERROR_INVALID_INPUT);
        }
        builder = builder.target_path(transport_text(input.target_path)?);
    }
    if input.max_request_bytes != 0 {
        let max_request_bytes =
            usize::try_from(input.max_request_bytes).map_err(|_| ERROR_RESOURCE_LIMIT)?;
        builder = builder.max_request_bytes(max_request_bytes);
    }
    builder.build().map_err(|_| ERROR_INVALID_INPUT)
}

fn timeout(seconds: u64, nanoseconds: u32) -> FfiResult<Option<Duration>> {
    if nanoseconds >= 1_000_000_000 {
        return Err(ERROR_INVALID_INPUT);
    }
    if seconds == 0 && nanoseconds == 0 {
        Ok(None)
    } else {
        Ok(Some(Duration::new(seconds, nanoseconds)))
    }
}

fn request_item_view(
    batch: &Arc<kmipkit_client::IncomingServerBatch>,
    index: usize,
) -> FfiResult<kmipkit_client::ServerRequestItemView<'_>> {
    batch
        .items()
        .into_iter()
        .nth(index)
        .ok_or(ERROR_INVALID_INPUT)
}

fn request_item_owner(
    item: *mut kmipkit_server_request_item_t,
) -> FfiResult<(Arc<kmipkit_client::IncomingServerBatch>, usize)> {
    let handle = reference_handle(item, Kind::ServerRequestItem)?;
    let HandleValue::ServerRequestItem(owner) = &handle.value else {
        return Err(ERROR_INVALID_INPUT);
    };
    Ok((Arc::clone(&owner.batch), owner.index))
}

fn managed_object_type_for_tag(tag: u32) -> kmipkit_protocol::ObjectType {
    let raw = match tag {
        0x0042_0013 => 1,
        0x0042_008F => 2,
        0x0042_006D => 3,
        0x0042_0064 => 4,
        0x0042_0089 => 5,
        0x0042_0085 => 7,
        0x0042_005B => 8,
        0x0042_00BD => 9,
        0x0042_0018 => 10,
        0x0042_01A1 => 13,
        0x0042_01A6 => 14,
        0x0042_01A7 => 15,
        0x0042_01AF => 16,
        _ => 0,
    };
    kmipkit_protocol::ObjectType::from_raw(raw)
}

fn put_object_child_kind(
    parent: &ServerPutObjectOwner,
    field_tag: u32,
    child: &Item,
) -> FfiResult<kmipkit_protocol::ManagedValueKind> {
    use kmipkit_protocol::ManagedValueKind;

    let kind = match parent.kind {
        ManagedValueKind::ManagedObject(_) if field_tag == SERVER_PUT_OBJECT_KEY_BLOCK => {
            ManagedValueKind::KeyBlock
        }
        ManagedValueKind::KeyBlock if field_tag == SERVER_KEY_BLOCK_KEY_VALUE => {
            let Some(ManagedFieldValue::Enumeration(format)) =
                parent.reader.read_field(SERVER_KEY_BLOCK_FORMAT)
            else {
                return Err(ERROR_INVALID_INPUT);
            };
            ManagedValueKind::KeyValue(KeyFormatType::from_raw(format))
        }
        ManagedValueKind::KeyBlock if field_tag == SERVER_KEY_BLOCK_WRAPPING_DATA => {
            ManagedValueKind::KeyWrappingData
        }
        ManagedValueKind::KeyValue(format) if field_tag == SERVER_KEY_VALUE_KEY_MATERIAL => {
            ManagedValueKind::KeyMaterial(format)
        }
        ManagedValueKind::ManagedObject(_) | ManagedValueKind::KeyMaterial(_) => {
            ManagedValueKind::ManagedObject(managed_object_type_for_tag(child.tag().raw()))
        }
        _ => ManagedValueKind::ManagedObject(managed_object_type_for_tag(child.tag().raw())),
    };
    Ok(kind)
}

fn server_put_object_handle(
    kind: kmipkit_protocol::ManagedValueKind,
    item: Item,
) -> FfiResult<*mut kmipkit_server_put_object_t> {
    let tag = item.tag().raw();
    let reader = kmipkit_protocol::ManagedValueReader::try_from_item(kind, item)
        .map_err(|_| ERROR_INVALID_INPUT)?;
    try_make_handle(
        Kind::ServerPutObject,
        HandleValue::ServerPutObject(ServerPutObjectOwner { kind, tag, reader }),
    )
}

fn server_response_item(
    operation: u32,
    payload: Result<Structure, kmipkit_protocol::ProtocolError>,
) -> FfiResult<*mut kmipkit_server_response_item_t> {
    let payload = payload.map_err(|_| ERROR_PROTOCOL)?;
    try_make_handle(
        Kind::ServerResponseItem,
        HandleValue::ServerResponseItem(ServerResponseItemValue { operation, payload }),
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_listener_config_create_v1(
    configuration: *const kmipkit_server_listener_config_v1_t,
    out_configuration: *mut *mut kmipkit_server_listener_config_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_configuration);
        let configuration = try_ffi!(listener_config_from_v1(configuration));
        *out = try_ffi!(try_make_handle(
            Kind::ServerListenerConfig,
            HandleValue::ServerListenerConfig(Box::new(configuration)),
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_listener_bind_raw_tls(
    configuration: *mut kmipkit_server_listener_config_t,
    out_listener: *mut *mut kmipkit_server_listener_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_listener);
        let configuration = read_handle!(configuration, ServerListenerConfig);
        let HandleValue::ServerListenerConfig(configuration) = &configuration.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        match kmipkit_client::ClientListener::bind_raw_tls((**configuration).clone()) {
            Ok(listener) => {
                *out = try_ffi!(try_make_handle(
                    Kind::ServerListener,
                    HandleValue::ServerListener(Box::new(Mutex::new(listener))),
                ));
                SUCCESS
            }
            Err(error) => record_client_error(&error),
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_listener_bind_https(
    configuration: *mut kmipkit_server_listener_config_t,
    out_listener: *mut *mut kmipkit_server_listener_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_listener);
        let configuration = read_handle!(configuration, ServerListenerConfig);
        let HandleValue::ServerListenerConfig(configuration) = &configuration.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        match kmipkit_client::ClientListener::bind_https((**configuration).clone()) {
            Ok(listener) => {
                *out = try_ffi!(try_make_handle(
                    Kind::ServerListener,
                    HandleValue::ServerListener(Box::new(Mutex::new(listener))),
                ));
                SUCCESS
            }
            Err(error) => record_client_error(&error),
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_listener_port(
    listener: *mut kmipkit_server_listener_t,
    out_port: *mut u16,
) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_port);
        *out = 0;
        let listener = read_handle!(listener, ServerListener);
        let HandleValue::ServerListener(listener) = &listener.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(listener) = listener.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        match listener.local_addr() {
            Ok(address) => {
                *out = address.port();
                SUCCESS
            }
            Err(_) => record_ffi_error(ERROR_TRANSPORT),
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_listener_accept(
    listener: *mut kmipkit_server_listener_t,
    timeout_seconds: u64,
    timeout_nanoseconds: u32,
    out_connection: *mut *mut kmipkit_server_connection_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_connection);
        let accept_timeout = try_ffi!(timeout(timeout_seconds, timeout_nanoseconds));
        let listener = read_handle!(listener, ServerListener);
        let HandleValue::ServerListener(listener) = &listener.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(mut listener) = listener.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        match listener.accept(accept_timeout) {
            Ok(connection) => {
                *out = try_ffi!(try_make_handle(
                    Kind::ServerConnection,
                    HandleValue::ServerConnection(Box::new(Mutex::new(ServerConnectionState {
                        connection,
                        active_batch: None,
                    }))),
                ));
                SUCCESS
            }
            Err(error) => record_client_error(&error),
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_connection_recv_batch(
    connection: *mut kmipkit_server_connection_t,
    limits: *mut kmipkit_codec_limits_t,
    timeout_seconds: u64,
    timeout_nanoseconds: u32,
    out_batch: *mut *mut kmipkit_server_incoming_batch_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_batch);
        let read_timeout = try_ffi!(timeout(timeout_seconds, timeout_nanoseconds));
        let limits = read_handle!(limits, CodecLimits);
        let HandleValue::CodecLimits(limits) = &limits.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let connection = read_handle!(connection, ServerConnection);
        let HandleValue::ServerConnection(connection) = &connection.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(mut connection) = connection.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        match connection.connection.recv_batch(limits, read_timeout) {
            Ok(batch) => {
                let batch = Arc::new(batch);
                connection.active_batch = Some(Arc::clone(&batch));
                *out = try_ffi!(try_make_handle(
                    Kind::ServerIncomingBatch,
                    HandleValue::ServerIncomingBatch(batch),
                ));
                SUCCESS
            }
            Err(error) => record_client_error(&error),
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_incoming_batch_count(
    batch: *mut kmipkit_server_incoming_batch_t,
    out_count: *mut u64,
) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_count);
        *out = 0;
        let batch = read_handle!(batch, ServerIncomingBatch);
        let HandleValue::ServerIncomingBatch(batch) = &batch.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = batch.items().len() as u64;
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_incoming_batch_protocol_version(
    batch: *mut kmipkit_server_incoming_batch_t,
    out_version: *mut kmipkit_protocol_version_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_version);
        *out = kmipkit_protocol_version_v1_t { major: 0, minor: 0 };
        let batch = read_handle!(batch, ServerIncomingBatch);
        let HandleValue::ServerIncomingBatch(batch) = &batch.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let version = batch.protocol_version();
        *out = kmipkit_protocol_version_v1_t {
            major: version.major(),
            minor: version.minor(),
        };
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_incoming_batch_item_at(
    batch: *mut kmipkit_server_incoming_batch_t,
    index: u64,
    out_item: *mut *mut kmipkit_server_request_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let index = try_ffi!(usize::try_from(index).map_err(|_| ERROR_RESOURCE_LIMIT));
        let batch = read_handle!(batch, ServerIncomingBatch);
        let HandleValue::ServerIncomingBatch(batch) = &batch.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        if index >= batch.items().len() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        *out = try_ffi!(try_make_handle(
            Kind::ServerRequestItem,
            HandleValue::ServerRequestItem(ServerRequestItemOwner {
                batch: Arc::clone(batch),
                index,
            }),
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_request_item_operation(
    item: *mut kmipkit_server_request_item_t,
    out_operation: *mut u32,
) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_operation);
        *out = 0;
        let (batch, index) = try_ffi!(request_item_owner(item));
        let view = try_ffi!(request_item_view(&batch, index));
        *out = view.operation();
        SUCCESS
    })
}

/// Returns a validated typed view of the Managed Object in a server Put request.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_request_item_put_object(
    request: *mut kmipkit_server_request_item_t,
    out_object: *mut *mut kmipkit_server_put_object_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_object);
        let (batch, index) = try_ffi!(request_item_owner(request));
        let view = try_ffi!(request_item_view(&batch, index));
        let Some(request) = view.as_put() else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let item = try_ffi!(request.object_item().try_clone().map_err(model_status));
        let kind = kmipkit_protocol::ManagedValueKind::ManagedObject(managed_object_type_for_tag(
            item.tag().raw(),
        ));
        *out = try_ffi!(server_put_object_handle(kind, item));
        SUCCESS
    })
}

/// Returns the KMIP Object Type for a root Put Managed Object view.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_put_object_type(
    object: *mut kmipkit_server_put_object_t,
    out_object_type: *mut u32,
) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_object_type);
        *out = 0;
        let handle = read_handle!(object, ServerPutObject);
        let HandleValue::ServerPutObject(owner) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let kmipkit_protocol::ManagedValueKind::ManagedObject(object_type) = owner.kind else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = object_type.raw();
        SUCCESS
    })
}

/// Returns the KMIP Structure tag for the current opaque Put-object view.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_put_object_tag(
    object: *mut kmipkit_server_put_object_t,
    out_tag: *mut u32,
) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_tag);
        *out = 0;
        let handle = read_handle!(object, ServerPutObject);
        let HandleValue::ServerPutObject(owner) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = owner.tag;
        SUCCESS
    })
}

/// Reads a typed Put-object field. Byte strings and big integers are returned only in a zeroizing
/// `kmipkit_secret_bytes_t`; nested structures produce another opaque validated view.
enum PutObjectFieldReadValue {
    Present,
    Enumeration(u32),
    Integer(i32),
    Child(*mut kmipkit_server_put_object_t),
    SecretBytes(*mut kmipkit_secret_bytes_t),
    SecretText(*mut kmipkit_secret_text_t),
}

fn read_put_object_field_value(
    owner: &ServerPutObjectOwner,
    field_tag: u32,
    read_kind: u32,
    field: ManagedFieldValue,
) -> FfiResult<PutObjectFieldReadValue> {
    match read_kind {
        SERVER_READ_PRESENT | SERVER_READ_TYPE => Ok(PutObjectFieldReadValue::Present),
        SERVER_READ_ENUMERATION => match field {
            ManagedFieldValue::Enumeration(value) => {
                Ok(PutObjectFieldReadValue::Enumeration(value))
            }
            _ => Err(ERROR_INVALID_INPUT),
        },
        SERVER_READ_INTEGER => match field {
            ManagedFieldValue::Integer(value) => Ok(PutObjectFieldReadValue::Integer(value)),
            _ => Err(ERROR_INVALID_INPUT),
        },
        SERVER_READ_SECRET_BYTES => match field {
            ManagedFieldValue::ByteString(bytes) | ManagedFieldValue::BigInteger(bytes) => {
                let mut copy = Zeroizing::new(Vec::new());
                copy.try_reserve_exact(bytes.len())
                    .map_err(|_| ERROR_RESOURCE_LIMIT)?;
                copy.extend_from_slice(&bytes);
                let secret = try_make_handle(
                    Kind::SecretBytes,
                    HandleValue::SecretBytes(SecretBytes::new(std::mem::take(&mut *copy))),
                )?;
                Ok(PutObjectFieldReadValue::SecretBytes(secret))
            }
            _ => Err(ERROR_INVALID_INPUT),
        },
        SERVER_READ_SECRET_TEXT => match field {
            ManagedFieldValue::TextString(bytes) => {
                let text = std::str::from_utf8(&bytes).map_err(|_| ERROR_INVALID_INPUT)?;
                let mut copy = Zeroizing::new(String::new());
                copy.try_reserve_exact(text.len())
                    .map_err(|_| ERROR_RESOURCE_LIMIT)?;
                copy.push_str(text);
                let secret = try_make_handle(
                    Kind::SecretText,
                    HandleValue::SecretText(SecretText::new(std::mem::take(&mut *copy))),
                )?;
                Ok(PutObjectFieldReadValue::SecretText(secret))
            }
            _ => Err(ERROR_INVALID_INPUT),
        },
        SERVER_READ_STRUCTURE_VIEW => {
            if !matches!(field, ManagedFieldValue::Structure) {
                return Err(ERROR_INVALID_INPUT);
            }
            let item = owner
                .reader
                .read_field_item(field_tag)
                .map_err(|_| ERROR_INVALID_INPUT)?
                .ok_or(ERROR_INVALID_INPUT)?;
            let child_kind = put_object_child_kind(owner, field_tag, &item)?;
            let child = server_put_object_handle(child_kind, item)?;
            Ok(PutObjectFieldReadValue::Child(child))
        }
        _ => Err(ERROR_INVALID_INPUT),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_put_object_field_read(
    object: *mut kmipkit_server_put_object_t,
    field_tag: u32,
    read_kind: u32,
    out_present: *mut u8,
    out_item_type: *mut u8,
    out_enumeration: *mut u32,
    out_integer: *mut i32,
    out_child: *mut *mut kmipkit_server_put_object_t,
    out_secret: *mut *mut kmipkit_secret_bytes_t,
    out_secret_text: *mut *mut kmipkit_secret_text_t,
) -> i32 {
    ffi_entrypoint(|| {
        if out_present.is_null() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let valid_outputs = match read_kind {
            SERVER_READ_PRESENT => true,
            SERVER_READ_TYPE => !out_item_type.is_null(),
            SERVER_READ_ENUMERATION => !out_enumeration.is_null(),
            SERVER_READ_INTEGER => !out_integer.is_null(),
            SERVER_READ_SECRET_BYTES => !out_secret.is_null(),
            SERVER_READ_STRUCTURE_VIEW => !out_item_type.is_null() && !out_child.is_null(),
            SERVER_READ_SECRET_TEXT => !out_secret_text.is_null(),
            _ => false,
        };
        if !valid_outputs {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        // SAFETY: The non-null output slots are writable caller storage per the C contract.
        unsafe { out_present.write(0) };
        if !out_item_type.is_null() {
            // SAFETY: Optional non-null output is caller-provided writable storage.
            unsafe { out_item_type.write(0) };
        }
        if !out_enumeration.is_null() {
            // SAFETY: Optional non-null output is caller-provided writable storage.
            unsafe { out_enumeration.write(0) };
        }
        if !out_integer.is_null() {
            // SAFETY: Optional non-null output is caller-provided writable storage.
            unsafe { out_integer.write(0) };
        }
        if !out_child.is_null() {
            // SAFETY: Optional non-null output is caller-provided writable storage.
            unsafe { out_child.write(ptr::null_mut()) };
        }
        if !out_secret.is_null() {
            // SAFETY: Optional non-null output is caller-provided writable storage.
            unsafe { out_secret.write(ptr::null_mut()) };
        }
        if !out_secret_text.is_null() {
            // SAFETY: Optional non-null output is caller-provided writable storage.
            unsafe { out_secret_text.write(ptr::null_mut()) };
        }
        let handle = read_handle!(object, ServerPutObject);
        let HandleValue::ServerPutObject(owner) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(field) = owner.reader.read_field(field_tag) else {
            return if read_kind == SERVER_READ_PRESENT {
                SUCCESS
            } else {
                record_ffi_error(ERROR_INVALID_INPUT)
            };
        };
        // SAFETY: The required output pointer was checked above.
        unsafe { out_present.write(1) };
        if !out_item_type.is_null() {
            // SAFETY: A non-null optional output points to writable caller storage.
            unsafe { out_item_type.write(item_type_code(field.item_type())) };
        }
        let value = match read_put_object_field_value(owner, field_tag, read_kind, field) {
            Ok(value) => value,
            Err(status) => return record_ffi_error(status),
        };
        match value {
            PutObjectFieldReadValue::Present => SUCCESS,
            PutObjectFieldReadValue::Enumeration(value) => {
                // SAFETY: The required enumeration output was checked above.
                unsafe { out_enumeration.write(value) };
                SUCCESS
            }
            PutObjectFieldReadValue::Integer(value) => {
                // SAFETY: The required integer output was checked above.
                unsafe { out_integer.write(value) };
                SUCCESS
            }
            PutObjectFieldReadValue::Child(child) => {
                // SAFETY: The required child output was checked above.
                unsafe { out_child.write(child) };
                SUCCESS
            }
            PutObjectFieldReadValue::SecretBytes(secret) => {
                // SAFETY: The required secret output was checked above.
                unsafe { out_secret.write(secret) };
                SUCCESS
            }
            PutObjectFieldReadValue::SecretText(secret) => {
                // SAFETY: The required secret-text output was checked above.
                unsafe { out_secret_text.write(secret) };
                SUCCESS
            }
        }
    })
}

/// Returns the non-secret Unique Identifier on a typed Notify or Put request.
/// The span borrows the request item and remains valid while it is retained.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_request_item_unique_identifier(
    item: *mut kmipkit_server_request_item_t,
    out_identifier: *mut kmipkit_byte_span_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_identifier);
        *out = kmipkit_byte_span_v1_t::default();
        let (batch, index) = try_ffi!(request_item_owner(item));
        let view = try_ffi!(request_item_view(&batch, index));
        let value = if let Some(request) = view.as_notify() {
            request.unique_identifier()
        } else if let Some(request) = view.as_put() {
            request.unique_identifier()
        } else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = kmipkit_byte_span_v1_t {
            data: value.as_ptr(),
            length: value.len() as u64,
        };
        SUCCESS
    })
}

/// Returns the changed attributes from a typed Notify request as a validated Attribute Set.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_request_item_notify_attributes(
    request: *mut kmipkit_server_request_item_t,
    out_present: *mut u8,
    out_attributes: *mut *mut kmipkit_attribute_set_t,
) -> i32 {
    ffi_entrypoint(|| {
        if out_present.is_null() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let out = out_slot!(out_attributes);
        // SAFETY: The required presence output was checked and is caller-owned storage.
        unsafe { out_present.write(0) };
        let (batch, index) = try_ffi!(request_item_owner(request));
        let view = try_ffi!(request_item_view(&batch, index));
        let Some(notify) = view.as_notify() else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(attributes) = notify.attributes() else {
            return SUCCESS;
        };
        let mut items = Vec::new();
        for item in attributes.view().children() {
            items.push(try_ffi!(clone_item(item)));
        }
        let attributes = try_ffi!(AttributeSet::try_new(items).map_err(|_| ERROR_INVALID_INPUT));
        *out = try_ffi!(try_make_handle(
            Kind::AttributeSet,
            HandleValue::AttributeSet(Mutex::new(attributes)),
        ));
        // SAFETY: The required presence output was checked and is caller-owned storage.
        unsafe { out_present.write(1) };
        SUCCESS
    })
}

/// Returns the number of Deleted Attribute References in a typed Notify request.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_request_item_notify_deleted_attribute_reference_count(
    request: *mut kmipkit_server_request_item_t,
    out_count: *mut u64,
) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_count);
        *out = 0;
        let (batch, index) = try_ffi!(request_item_owner(request));
        let view = try_ffi!(request_item_view(&batch, index));
        let Some(notify) = view.as_notify() else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = notify.deleted_attribute_references().len() as u64;
        SUCCESS
    })
}

/// Reads one Deleted Attribute Reference from Notify into a versioned borrowed-span record.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_request_item_notify_deleted_attribute_reference_at(
    request: *mut kmipkit_server_request_item_t,
    index: u64,
    output: *mut kmipkit_attribute_reference_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        try_ffi!(validate_attribute_reference_output(output));
        // SAFETY: `validate_attribute_reference_output` checked pointer alignment, record size,
        // and ABI version before this fixed-header read.
        let struct_size = unsafe { std::ptr::addr_of!((*output).struct_size).read() };
        let mut record = kmipkit_attribute_reference_v1_t {
            struct_size,
            abi_version: 1,
            kind: 0,
            reserved: 0,
            tag_value: 0,
            reserved2: 0,
            vendor_identification_data: ptr::null(),
            vendor_identification_length: 0,
            attribute_name_data: ptr::null(),
            attribute_name_length: 0,
        };
        let index = try_ffi!(usize::try_from(index).map_err(|_| ERROR_RESOURCE_LIMIT));
        let (batch, item_index) = try_ffi!(request_item_owner(request));
        let view = try_ffi!(request_item_view(&batch, item_index));
        let Some(notify) = view.as_notify() else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(reference) = notify.deleted_attribute_references().get(index) else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let fields = reference.view().children();
        if fields.len() == 1 && fields[0].tag().raw() == SERVER_NOTIFY_ATTRIBUTE_NAME {
            let Some(raw_tag) = fields[0].with_value(|value| match value {
                ValueView::Enumeration(raw) => Some(*raw),
                _ => None,
            }) else {
                return record_ffi_error(ERROR_INVALID_INPUT);
            };
            record.kind = crate::KMIPKIT_ATTRIBUTE_REFERENCE_KIND_TAG;
            record.tag_value = raw_tag;
        } else if fields.len() == 2
            && fields[0].tag().raw() == SERVER_NOTIFY_VENDOR_IDENTIFICATION
            && fields[1].tag().raw() == SERVER_NOTIFY_ATTRIBUTE_NAME
        {
            let Some((vendor_ptr, vendor_len, name_ptr, name_len)) =
                fields[0].with_value(|vendor_value| {
                    fields[1].with_value(|name_value| match (vendor_value, name_value) {
                        (ValueView::TextString(vendor), ValueView::TextString(name)) => {
                            Some((vendor.as_ptr(), vendor.len(), name.as_ptr(), name.len()))
                        }
                        _ => None,
                    })
                })
            else {
                return record_ffi_error(ERROR_INVALID_INPUT);
            };
            record.kind = crate::KMIPKIT_ATTRIBUTE_REFERENCE_KIND_NAME;
            record.vendor_identification_data = vendor_ptr;
            record.vendor_identification_length = vendor_len as u64;
            record.attribute_name_data = name_ptr;
            record.attribute_name_length = name_len as u64;
        } else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        // SAFETY: The record was validated for the caller's v1 ABI layout above.
        unsafe { output.write(record) };
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_request_item_put_function(
    item: *mut kmipkit_server_request_item_t,
    out_put_function: *mut u32,
) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_put_function);
        *out = 0;
        let (batch, index) = try_ffi!(request_item_owner(item));
        let view = try_ffi!(request_item_view(&batch, index));
        let Some(request) = view.as_put() else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = request.put_function().raw();
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_request_item_endpoint_role(
    item: *mut kmipkit_server_request_item_t,
    out_endpoint_role: *mut u32,
) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_endpoint_role);
        *out = 0;
        let (batch, index) = try_ffi!(request_item_owner(item));
        let view = try_ffi!(request_item_view(&batch, index));
        let Some(request) = view.as_set_endpoint_role() else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = request.endpoint_role().raw();
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_request_item_discover_versions_count(
    item: *mut kmipkit_server_request_item_t,
    out_count: *mut u64,
) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_count);
        *out = 0;
        let (batch, index) = try_ffi!(request_item_owner(item));
        let view = try_ffi!(request_item_view(&batch, index));
        let Some(request) = view.as_discover_versions() else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = request.protocol_versions().len() as u64;
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_request_item_discover_version_at(
    item: *mut kmipkit_server_request_item_t,
    index: u64,
    out_version: *mut kmipkit_protocol_version_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_version);
        *out = kmipkit_protocol_version_v1_t { major: 0, minor: 0 };
        let index = try_ffi!(usize::try_from(index).map_err(|_| ERROR_RESOURCE_LIMIT));
        let (batch, item_index) = try_ffi!(request_item_owner(item));
        let view = try_ffi!(request_item_view(&batch, item_index));
        let Some(request) = view.as_discover_versions() else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(version) = request.protocol_versions().get(index) else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = kmipkit_protocol_version_v1_t {
            major: version.major(),
            minor: version.minor(),
        };
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_request_item_query_functions_count(
    item: *mut kmipkit_server_request_item_t,
    out_count: *mut u64,
) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_count);
        *out = 0;
        let (batch, index) = try_ffi!(request_item_owner(item));
        let view = try_ffi!(request_item_view(&batch, index));
        let Some(request) = view.as_query() else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = request.query_functions().len() as u64;
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_request_item_query_function_at(
    item: *mut kmipkit_server_request_item_t,
    index: u64,
    out_query_function: *mut u32,
) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_query_function);
        *out = 0;
        let index = try_ffi!(usize::try_from(index).map_err(|_| ERROR_RESOURCE_LIMIT));
        let (batch, item_index) = try_ffi!(request_item_owner(item));
        let view = try_ffi!(request_item_view(&batch, item_index));
        let Some(request) = view.as_query() else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(function) = request.query_functions().get(index) else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = function.raw();
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_response_builder_create(
    batch: *mut kmipkit_server_incoming_batch_t,
    out_builder: *mut *mut kmipkit_server_response_builder_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_builder);
        let batch = read_handle!(batch, ServerIncomingBatch);
        let HandleValue::ServerIncomingBatch(batch) = &batch.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let item_count = batch.items().len();
        let builder = kmipkit_protocol::server_to_client::ServerResponseBuilder::with_version(
            batch.protocol_version(),
        );
        *out = try_ffi!(try_make_handle(
            Kind::ServerResponseBuilder,
            HandleValue::ServerResponseBuilder(Box::new(Mutex::new(Some(
                ServerResponseAccumulator {
                    batch: Arc::clone(batch),
                    builder,
                    added_items: vec![false; item_count],
                },
            )))),
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_notify_response_create(
    request: *mut kmipkit_server_request_item_t,
    out_response_item: *mut *mut kmipkit_server_response_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_response_item);
        let (batch, index) = try_ffi!(request_item_owner(request));
        let view = try_ffi!(request_item_view(&batch, index));
        if view.as_notify().is_none() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        *out = try_ffi!(server_response_item(
            SERVER_OPERATION_NOTIFY,
            kmipkit_protocol::server_to_client::ServerNotifyResponse::new().to_ttlv_payload(),
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_put_response_create(
    request: *mut kmipkit_server_request_item_t,
    out_response_item: *mut *mut kmipkit_server_response_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_response_item);
        let (batch, index) = try_ffi!(request_item_owner(request));
        let view = try_ffi!(request_item_view(&batch, index));
        if view.as_put().is_none() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        *out = try_ffi!(server_response_item(
            SERVER_OPERATION_PUT,
            kmipkit_protocol::server_to_client::ServerPutResponse::new().to_ttlv_payload(),
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_discover_versions_response_create(
    request: *mut kmipkit_server_request_item_t,
    versions: *const kmipkit_protocol_version_v1_t,
    version_count: u64,
    out_response_item: *mut *mut kmipkit_server_response_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_response_item);
        let versions = try_ffi!(input_array(
            versions,
            version_count,
            MAX_TRANSPORT_CONFIG_ITEMS
        ));
        let (batch, index) = try_ffi!(request_item_owner(request));
        let view = try_ffi!(request_item_view(&batch, index));
        if view.as_discover_versions().is_none() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let versions = versions
            .iter()
            .map(|version| {
                kmipkit_protocol::ProtocolVersion::from_raw(version.major, version.minor)
            })
            .collect();
        *out = try_ffi!(server_response_item(
            SERVER_OPERATION_DISCOVER_VERSIONS,
            kmipkit_protocol::server_to_client::ServerDiscoverVersionsResponse::new(versions)
                .to_ttlv_payload(),
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_query_response_create(
    request: *mut kmipkit_server_request_item_t,
    operations: *const u32,
    operation_count: u64,
    object_types: *const u32,
    object_type_count: u64,
    extension_tags: *const u32,
    extension_tag_count: u64,
    vendor_identification: kmipkit_byte_span_v1_t,
    out_response_item: *mut *mut kmipkit_server_response_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_response_item);
        let operations = try_ffi!(input_array(
            operations,
            operation_count,
            MAX_VALIDATE_INPUT_ITEMS
        ));
        let object_types = try_ffi!(input_array(
            object_types,
            object_type_count,
            MAX_VALIDATE_INPUT_ITEMS
        ));
        let extension_tags = try_ffi!(input_array(
            extension_tags,
            extension_tag_count,
            MAX_VALIDATE_INPUT_ITEMS
        ));
        let vendor_identification = if vendor_identification.length == 0
            && vendor_identification.data.is_null()
        {
            None
        } else {
            let bytes = try_ffi!(input_span(
                vendor_identification.data,
                vendor_identification.length,
                MAX_TEXT_BYTES
            ));
            Some(try_ffi!(std::str::from_utf8(bytes).map_err(|_| ERROR_INVALID_INPUT)).to_owned())
        };
        let (batch, index) = try_ffi!(request_item_owner(request));
        let view = try_ffi!(request_item_view(&batch, index));
        let Some(request) = view.as_query() else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let mut capabilities = kmipkit_protocol::server_to_client::ServerCapabilities::new();
        capabilities.set_operations(operations.to_vec());
        capabilities.set_object_types(object_types.to_vec());
        capabilities.set_extension_tags(extension_tags.to_vec());
        if let Some(vendor_identification) = vendor_identification {
            capabilities.set_vendor_identification(vendor_identification);
        }
        *out = try_ffi!(server_response_item(
            SERVER_OPERATION_QUERY,
            kmipkit_protocol::server_to_client::ServerQueryResponse::respond(
                request,
                &capabilities
            )
            .to_ttlv_payload(),
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_set_endpoint_role_response_create(
    request: *mut kmipkit_server_request_item_t,
    endpoint_role: u32,
    vendor_identification: kmipkit_byte_span_v1_t,
    out_response_item: *mut *mut kmipkit_server_response_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_response_item);
        let vendor_identification = if vendor_identification.length == 0
            && vendor_identification.data.is_null()
        {
            None
        } else {
            let bytes = try_ffi!(input_span(
                vendor_identification.data,
                vendor_identification.length,
                MAX_TEXT_BYTES
            ));
            Some(try_ffi!(std::str::from_utf8(bytes).map_err(|_| ERROR_INVALID_INPUT)).to_owned())
        };
        let (batch, index) = try_ffi!(request_item_owner(request));
        let view = try_ffi!(request_item_view(&batch, index));
        if view.as_set_endpoint_role().is_none() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        *out = try_ffi!(server_response_item(
            SERVER_OPERATION_SET_ENDPOINT_ROLE,
            kmipkit_protocol::SetEndpointRoleResponse::success(
                kmipkit_protocol::EndpointRole::from_raw(endpoint_role),
                vendor_identification,
            )
            .to_ttlv_payload(),
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_response_builder_add_success(
    builder: *mut kmipkit_server_response_builder_t,
    request: *mut kmipkit_server_request_item_t,
    response_item: *mut kmipkit_server_response_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let request_handle = read_handle!(request, ServerRequestItem);
        let HandleValue::ServerRequestItem(request) = &request_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let response_item = read_handle!(response_item, ServerResponseItem);
        let HandleValue::ServerResponseItem(response_item) = &response_item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let builder_handle = read_handle!(builder, ServerResponseBuilder);
        let HandleValue::ServerResponseBuilder(builder) = &builder_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(mut builder) = builder.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        let Some(accumulator) = builder.as_mut() else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        if !Arc::ptr_eq(&accumulator.batch, &request.batch)
            || request.index >= accumulator.added_items.len()
            || accumulator.added_items[request.index]
        {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let view = try_ffi!(request_item_view(&request.batch, request.index));
        if view.operation() != response_item.operation {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let unique_id = view.unique_batch_item_id().map(<[u8]>::to_vec);
        let Ok(payload) = response_item.payload.view().try_clone() else {
            return record_ffi_error(ERROR_RESOURCE_LIMIT);
        };
        if accumulator
            .builder
            .add_success_item(response_item.operation, unique_id, payload)
            .is_err()
        {
            return record_ffi_error(ERROR_PROTOCOL);
        }
        accumulator.added_items[request.index] = true;
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_response_builder_add_failure(
    builder: *mut kmipkit_server_response_builder_t,
    request: *mut kmipkit_server_request_item_t,
    result_status: u32,
    has_result_reason: u8,
    result_reason: u32,
    result_message: kmipkit_byte_span_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        let result_status_value = kmipkit_protocol::ResultStatus::from_raw(result_status);
        if has_result_reason > 1
            || result_status_value.known_name() == Some("Success")
            || (result_status_value.known_name() == Some("Operation Failed")
                && has_result_reason == 0)
        {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let message = if result_message.length == 0 && result_message.data.is_null() {
            None
        } else {
            let bytes = try_ffi!(input_span(
                result_message.data,
                result_message.length,
                MAX_TEXT_BYTES
            ));
            Some(try_ffi!(std::str::from_utf8(bytes).map_err(|_| ERROR_INVALID_INPUT)).to_owned())
        };
        let request_handle = read_handle!(request, ServerRequestItem);
        let HandleValue::ServerRequestItem(request) = &request_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let builder_handle = read_handle!(builder, ServerResponseBuilder);
        let HandleValue::ServerResponseBuilder(builder) = &builder_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(mut builder) = builder.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        let Some(accumulator) = builder.as_mut() else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        if !Arc::ptr_eq(&accumulator.batch, &request.batch)
            || request.index >= accumulator.added_items.len()
            || accumulator.added_items[request.index]
        {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let view = try_ffi!(request_item_view(&request.batch, request.index));
        let reason = (has_result_reason == 1)
            .then(|| kmipkit_protocol::ResultReason::from_raw(result_reason));
        if accumulator
            .builder
            .add_failure_item(
                view.operation(),
                view.unique_batch_item_id().map(<[u8]>::to_vec),
                kmipkit_protocol::ResultStatus::from_raw(result_status),
                reason,
                message,
            )
            .is_err()
        {
            return record_ffi_error(ERROR_PROTOCOL);
        }
        accumulator.added_items[request.index] = true;
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_response_builder_build(
    builder: *mut kmipkit_server_response_builder_t,
    out_response: *mut *mut kmipkit_server_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_response);
        let builder = read_handle!(builder, ServerResponseBuilder);
        let HandleValue::ServerResponseBuilder(builder) = &builder.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(mut builder) = builder.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        let Some(accumulator) = builder.as_mut() else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        if accumulator.added_items.iter().any(|added| !added) {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let Some(accumulator) = builder.take() else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let batch = accumulator.batch;
        let Ok(message) = accumulator.builder.build() else {
            return record_ffi_error(ERROR_PROTOCOL);
        };
        *out = try_ffi!(try_make_handle(
            Kind::ServerResponse,
            HandleValue::ServerResponse(Box::new(ServerResponseOwner { batch, message })),
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_connection_send_response(
    connection: *mut kmipkit_server_connection_t,
    response: *mut kmipkit_server_response_t,
    timeout_seconds: u64,
    timeout_nanoseconds: u32,
) -> i32 {
    ffi_entrypoint(|| {
        let send_timeout = try_ffi!(timeout(timeout_seconds, timeout_nanoseconds));
        let connection = read_handle!(connection, ServerConnection);
        let HandleValue::ServerConnection(connection) = &connection.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let response = read_handle!(response, ServerResponse);
        let HandleValue::ServerResponse(response) = &response.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(mut connection) = connection.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        let Some(active_batch) = connection.active_batch.as_ref() else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        if !Arc::ptr_eq(active_batch, &response.batch) {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        connection.active_batch = None;
        match connection
            .connection
            .send_response(&response.message, send_timeout)
        {
            Ok(()) => SUCCESS,
            Err(error) => record_client_error(&error),
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_listener_config_release(
    configuration: *mut kmipkit_server_listener_config_t,
) {
    ffi_entrypoint(|| release(configuration, Kind::ServerListenerConfig));
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_listener_release(listener: *mut kmipkit_server_listener_t) {
    ffi_entrypoint(|| release(listener, Kind::ServerListener));
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_connection_release(connection: *mut kmipkit_server_connection_t) {
    ffi_entrypoint(|| release(connection, Kind::ServerConnection));
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_incoming_batch_release(
    batch: *mut kmipkit_server_incoming_batch_t,
) {
    ffi_entrypoint(|| release(batch, Kind::ServerIncomingBatch));
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_request_item_release(item: *mut kmipkit_server_request_item_t) {
    ffi_entrypoint(|| release(item, Kind::ServerRequestItem));
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_put_object_release(object: *mut kmipkit_server_put_object_t) {
    ffi_entrypoint(|| release(object, Kind::ServerPutObject));
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_response_builder_release(
    builder: *mut kmipkit_server_response_builder_t,
) {
    ffi_entrypoint(|| release(builder, Kind::ServerResponseBuilder));
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_response_item_release(item: *mut kmipkit_server_response_item_t) {
    ffi_entrypoint(|| release(item, Kind::ServerResponseItem));
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_response_release(response: *mut kmipkit_server_response_t) {
    ffi_entrypoint(|| release(response, Kind::ServerResponse));
}
