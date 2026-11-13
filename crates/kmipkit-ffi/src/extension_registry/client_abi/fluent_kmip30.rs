//! Semantic C builders for the seven approved KMIP 3.0 operations (KMIPKIT-0116).
//!
//! KMIP 3.0 WD 2026-07-13 §§6.1.9/10/13/14/15/22/41. Every constructor copies
//! borrowed inputs. Builders and credential objects own their native copies and
//! may be released independently; caller-created copies remain caller-owned.

use super::*;
use crate::{kmipkit_create_credential_object_t, kmipkit_kmip30_operation_builder_t};
use kmipkit_client::Kmip30OperationResponse;
use kmipkit_protocol::{
    Certificate, CreateCredentialObject, CreateCredentialRequest, CreateGroupRequest,
    CreateUserRequest, DeactivateRequest, DecapsulateRequest, DeviceCredentialObject,
    DeviceCredentialObjectFields, EncapsulateRequest, HashedPasswordCredentialObject,
    HashedPasswordCredentialObjectFields, ObliterateRequest, OneTimePasswordCredentialObject,
    OneTimePasswordCredentialObjectFields, PasswordCredentialObject,
    PasswordCredentialObjectFields,
};

// Validate alignment before the shared output-slot helper creates a Rust reference.
macro_rules! kmip30_out_slot {
    ($out:expr) => {{
        if !$out.is_aligned() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        out_slot!($out)
    }};
}

pub(in crate::extension_registry) struct Kmip30OperationBuilder {
    request: BuilderRequest,
    asynchronous_indicator: Option<u32>,
}

enum BuilderRequest {
    CreateCredential {
        credential_type: CredentialType,
        attributes: AttributeSet,
        object: Item,
    },
    CreateGroup(AttributeSet),
    CreateUser(AttributeSet),
    Deactivate {
        identifier: UniqueIdentifier,
        reason: Option<Structure>,
        date: Option<i64>,
    },
    Decapsulate {
        identifier: UniqueIdentifier,
        ciphertext: SecretBytes,
        parameters: Option<Structure>,
        attributes: Option<AttributeSet>,
    },
    Encapsulate {
        identifier: UniqueIdentifier,
        parameters: Option<Structure>,
        attributes: Option<AttributeSet>,
    },
    Obliterate(UniqueIdentifier),
}

fn copy_attributes(source: &AttributeSet) -> FfiResult<AttributeSet> {
    let mut items = Vec::new();
    items
        .try_reserve_exact(source.len())
        .map_err(|_| ERROR_RESOURCE_LIMIT)?;
    for item in source.as_items() {
        items.push(item.try_clone().map_err(model_status)?);
    }
    AttributeSet::try_new(items).map_err(|_| ERROR_INVALID_INPUT)
}

fn input_attributes(attributes: *mut kmipkit_attribute_set_t) -> FfiResult<AttributeSet> {
    let handle = reference_handle(attributes, Kind::AttributeSet)?;
    let HandleValue::AttributeSet(attributes) = &handle.value else {
        return Err(ERROR_INVALID_INPUT);
    };
    let attributes = attributes.lock().map_err(|_| ERROR_INTERNAL)?;
    copy_attributes(&attributes)
}

fn identifier_input(
    identifier: *const kmipkit_unique_identifier_v1_t,
) -> FfiResult<UniqueIdentifier> {
    if identifier.is_null() || !identifier.is_aligned() {
        return Err(ERROR_INVALID_INPUT);
    }
    copy_unique_identifier(identifier)?.ok_or(ERROR_INVALID_INPUT)
}

fn bytes_input(span: *const kmipkit_byte_span_v1_t) -> FfiResult<SecretBytes> {
    if span.is_null() || !span.is_aligned() {
        return Err(ERROR_INVALID_INPUT);
    }
    // SAFETY: the caller lends a readable, aligned span for this synchronous call.
    let span = unsafe { span.read() };
    let input = input_span(span.data, span.length, MAX_FFI_INPUT_BYTES)?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(input.len())
        .map_err(|_| ERROR_RESOURCE_LIMIT)?;
    bytes.extend_from_slice(input);
    Ok(SecretBytes::new(bytes))
}

fn optional_bytes(span: *const kmipkit_byte_span_v1_t) -> FfiResult<Option<SecretBytes>> {
    if span.is_null() {
        Ok(None)
    } else {
        bytes_input(span).map(Some)
    }
}

fn optional_text(span: *const kmipkit_byte_span_v1_t) -> FfiResult<Option<SecretText>> {
    optional_bytes(span)?
        .map(|bytes| {
            bytes.with_bytes(|bytes| {
                let input = std::str::from_utf8(bytes).map_err(|_| ERROR_INVALID_INPUT)?;
                let mut text = Zeroizing::new(String::new());
                text.try_reserve_exact(input.len())
                    .map_err(|_| ERROR_RESOURCE_LIMIT)?;
                text.push_str(input);
                Ok(SecretText::new(std::mem::take(&mut *text)))
            })
        })
        .transpose()
}

fn optional_number<T: Default + PartialEq>(present: u8, value: T) -> FfiResult<Option<T>> {
    match present {
        0 if value == T::default() => Ok(None),
        1 => Ok(Some(value)),
        _ => Err(ERROR_INVALID_INPUT),
    }
}

fn credential_object(item: Item) -> FfiResult<CreateCredentialObject> {
    match item.tag().raw() {
        0x0042_0013 => Certificate::try_from_item(item).map(CreateCredentialObject::Certificate),
        0x0042_01A1 => PasswordCredentialObject::try_from_item(item)
            .map(CreateCredentialObject::PasswordCredential),
        0x0042_01A6 => DeviceCredentialObject::try_from_item(item)
            .map(CreateCredentialObject::DeviceCredential),
        0x0042_01AF => HashedPasswordCredentialObject::try_from_item(item)
            .map(CreateCredentialObject::HashedPasswordCredential),
        0x0042_01A7 => OneTimePasswordCredentialObject::try_from_item(item)
            .map(CreateCredentialObject::OneTimePasswordCredential),
        _ => return Err(ERROR_INVALID_INPUT),
    }
    .map_err(|_| ERROR_INVALID_INPUT)
}

fn input_object(object: *mut kmipkit_create_credential_object_t) -> FfiResult<Item> {
    let handle = reference_handle(object, Kind::CreateCredentialObject)?;
    let HandleValue::CreateCredentialObject(item) = &handle.value else {
        return Err(ERROR_INVALID_INPUT);
    };
    item.try_clone().map_err(model_status)
}

fn object_handle(item: Item) -> FfiResult<*mut kmipkit_create_credential_object_t> {
    try_make_handle(
        Kind::CreateCredentialObject,
        HandleValue::CreateCredentialObject(item),
    )
}

/// Copies an existing typed Certificate into the closed Create Credential object owner.
/// Other object tags are rejected. The input Item remains caller-owned.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_create_credential_object_certificate(
    certificate: *mut kmipkit_ttlv_item_t,
    out_object: *mut *mut kmipkit_create_credential_object_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = kmip30_out_slot!(out_object);
        let item = try_ffi!(copy_required_item_handle(certificate));
        let certificate =
            try_ffi!(Certificate::try_from_item(item).map_err(|_| ERROR_INVALID_INPUT));
        *out = try_ffi!(object_handle(certificate.into_item()));
        SUCCESS
    })
}

/// Copies optional password representation fields into an owned zeroizing object.
/// Null spans mean absent; a non-null empty span means an explicitly empty value.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_create_credential_object_password(
    password: *const kmipkit_byte_span_v1_t,
    password_salt: *const kmipkit_byte_span_v1_t,
    has_salt_algorithm: u8,
    salt_algorithm: u32,
    salted_password: *const kmipkit_byte_span_v1_t,
    has_iteration_count: u8,
    iteration_count: i32,
    out_object: *mut *mut kmipkit_create_credential_object_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = kmip30_out_slot!(out_object);
        let fields = PasswordCredentialObjectFields {
            password: try_ffi!(optional_text(password)),
            password_salt: try_ffi!(optional_bytes(password_salt)),
            password_salt_algorithm: try_ffi!(optional_number(has_salt_algorithm, salt_algorithm)),
            salted_password: try_ffi!(optional_bytes(salted_password)),
            iteration_count: try_ffi!(optional_number(has_iteration_count, iteration_count)),
        };
        let object =
            try_ffi!(PasswordCredentialObject::try_new(fields).map_err(|_| ERROR_INVALID_INPUT));
        *out = try_ffi!(object_handle(object.into_item()));
        SUCCESS
    })
}

/// Copies the optional UTF-8 device fields; the typed model requires at least one identifier.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_create_credential_object_device(
    serial: *const kmipkit_byte_span_v1_t,
    identifier: *const kmipkit_byte_span_v1_t,
    network: *const kmipkit_byte_span_v1_t,
    machine: *const kmipkit_byte_span_v1_t,
    media: *const kmipkit_byte_span_v1_t,
    out_object: *mut *mut kmipkit_create_credential_object_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = kmip30_out_slot!(out_object);
        let fields = DeviceCredentialObjectFields {
            device_serial_number: try_ffi!(optional_text(serial)),
            device_identifier: try_ffi!(optional_text(identifier)),
            network_identifier: try_ffi!(optional_text(network)),
            machine_identifier: try_ffi!(optional_text(machine)),
            media_identifier: try_ffi!(optional_text(media)),
        };
        let object =
            try_ffi!(DeviceCredentialObject::try_new(fields).map_err(|_| ERROR_INVALID_INPUT));
        *out = try_ffi!(object_handle(object.into_item()));
        SUCCESS
    })
}

/// Copies the two required precomputed hashed-password byte strings without hashing locally.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_create_credential_object_hashed_password(
    has_hashing_algorithm: u8,
    hashing_algorithm: u32,
    hashed_username_password: *const kmipkit_byte_span_v1_t,
    hashed_password_username: *const kmipkit_byte_span_v1_t,
    out_object: *mut *mut kmipkit_create_credential_object_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = kmip30_out_slot!(out_object);
        let fields = HashedPasswordCredentialObjectFields {
            hashing_algorithm: try_ffi!(optional_number(has_hashing_algorithm, hashing_algorithm)),
            hashed_username_password: Some(try_ffi!(bytes_input(hashed_username_password))),
            hashed_password_username: Some(try_ffi!(bytes_input(hashed_password_username))),
        };
        let object = try_ffi!(
            HashedPasswordCredentialObject::try_new(fields).map_err(|_| ERROR_INVALID_INPUT)
        );
        *out = try_ffi!(object_handle(object.into_item()));
        SUCCESS
    })
}

/// Copies the caller-selected OTP parameters and optional seed; no OTP is generated locally.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_create_credential_object_one_time_password(
    algorithm: u32,
    has_digest: u8,
    digest: u32,
    serial: *const kmipkit_byte_span_v1_t,
    seed: *const kmipkit_byte_span_v1_t,
    has_interval: u8,
    interval: u32,
    has_digits: u8,
    digits: i32,
    out_object: *mut *mut kmipkit_create_credential_object_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = kmip30_out_slot!(out_object);
        let fields = OneTimePasswordCredentialObjectFields {
            otp_algorithm: Some(algorithm),
            otp_digest: try_ffi!(optional_number(has_digest, digest)),
            otp_serial: try_ffi!(optional_text(serial)),
            otp_seed: try_ffi!(optional_bytes(seed)),
            otp_interval: try_ffi!(optional_number(has_interval, interval)),
            otp_digits: try_ffi!(optional_number(has_digits, digits)),
        };
        let object = try_ffi!(
            OneTimePasswordCredentialObject::try_new(fields).map_err(|_| ERROR_INVALID_INPUT)
        );
        *out = try_ffi!(object_handle(object.into_item()));
        SUCCESS
    })
}

/// Releases this native credential copy. Borrowed input and external copies remain caller-owned.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_create_credential_object_release(
    object: *mut kmipkit_create_credential_object_t,
) {
    ffi_entrypoint(|| release(object, Kind::CreateCredentialObject));
}

impl Kmip30OperationBuilder {
    fn new(request: BuilderRequest) -> Self {
        Self {
            request,
            asynchronous_indicator: None,
        }
    }

    fn build_request(&self) -> FfiResult<ClientRequest> {
        Ok(match &self.request {
            BuilderRequest::CreateCredential {
                credential_type,
                attributes,
                object,
            } => ClientRequest::create_credential(CreateCredentialRequest::new(
                *credential_type,
                copy_attributes(attributes)?,
                credential_object(object.try_clone().map_err(model_status)?)?,
            )),
            BuilderRequest::CreateGroup(attributes) => {
                ClientRequest::create_group(CreateGroupRequest::new(copy_attributes(attributes)?))
            }
            BuilderRequest::CreateUser(attributes) => {
                ClientRequest::create_user(CreateUserRequest::new(copy_attributes(attributes)?))
            }
            BuilderRequest::Deactivate {
                identifier,
                reason,
                date,
            } => {
                let mut request = DeactivateRequest::new(identifier.clone());
                if let Some(reason) = reason {
                    request = request
                        .with_deactivation_reason(reason.view().try_clone().map_err(model_status)?);
                }
                if let Some(date) = date {
                    request = request.with_deactivation_date(*date);
                }
                ClientRequest::deactivate(request)
            }
            BuilderRequest::Decapsulate {
                identifier,
                ciphertext,
                parameters,
                attributes,
            } => {
                let bytes = ciphertext.with_bytes(|input| {
                    let mut bytes = Vec::new();
                    bytes
                        .try_reserve_exact(input.len())
                        .map_err(|_| ERROR_RESOURCE_LIMIT)?;
                    bytes.extend_from_slice(input);
                    Ok::<_, i32>(bytes)
                })?;
                let mut request = DecapsulateRequest::new(identifier.clone(), bytes);
                if let Some(parameters) = parameters {
                    request =
                        request.with_typed_cryptographic_parameters(copy_parameters(parameters)?);
                }
                if let Some(attributes) = attributes {
                    request = request.with_attributes(copy_attributes(attributes)?);
                }
                ClientRequest::decapsulate(request)
            }
            BuilderRequest::Encapsulate {
                identifier,
                parameters,
                attributes,
            } => {
                let mut request = EncapsulateRequest::new(identifier.clone());
                if let Some(parameters) = parameters {
                    request =
                        request.with_typed_cryptographic_parameters(copy_parameters(parameters)?);
                }
                if let Some(attributes) = attributes {
                    request = request.with_attributes(copy_attributes(attributes)?);
                }
                ClientRequest::encapsulate(request)
            }
            BuilderRequest::Obliterate(identifier) => {
                ClientRequest::obliterate(ObliterateRequest::new(identifier.clone()))
            }
        })
    }
}

fn copy_parameters(parameters: &Structure) -> FfiResult<CryptographicParameters> {
    CryptographicParameters::try_from_ttlv(parameters.view().try_clone().map_err(model_status)?)
        .map_err(|_| ERROR_INVALID_INPUT)
}

fn builder_handle(request: BuilderRequest) -> FfiResult<*mut kmipkit_kmip30_operation_builder_t> {
    try_make_handle(
        Kind::Kmip30OperationBuilder,
        HandleValue::Kmip30OperationBuilder(Mutex::new(Kmip30OperationBuilder::new(request))),
    )
}

fn update_builder(
    builder: *mut kmipkit_kmip30_operation_builder_t,
    update: impl FnOnce(&mut Kmip30OperationBuilder) -> FfiResult<()>,
) -> FfiResult<()> {
    let handle = reference_handle(builder, Kind::Kmip30OperationBuilder)?;
    let HandleValue::Kmip30OperationBuilder(builder) = &handle.value else {
        return Err(ERROR_INVALID_INPUT);
    };
    let mut builder = builder.lock().map_err(|_| ERROR_INTERNAL)?;
    update(&mut builder)
}

fn create_credential_input(
    attributes: *mut kmipkit_attribute_set_t,
    object: *mut kmipkit_create_credential_object_t,
) -> FfiResult<BuilderRequest> {
    let object = input_object(object)?;
    let credential_type = match object.tag().raw() {
        0x0042_0013 => CredentialType::Attestation,
        0x0042_01A1 => CredentialType::UsernameAndPassword,
        0x0042_01A6 => CredentialType::Device,
        0x0042_01AF => CredentialType::HashedPassword,
        0x0042_01A7 => CredentialType::OneTimePassword,
        _ => return Err(ERROR_INVALID_INPUT),
    };
    Ok(BuilderRequest::CreateCredential {
        credential_type,
        attributes: input_attributes(attributes)?,
        object,
    })
}

/// Creates a reusable Create Credential builder from copied semantic inputs.
/// The caller retains every input handle and byte span.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_create_credential_builder_create(
    attributes: *mut kmipkit_attribute_set_t,
    object: *mut kmipkit_create_credential_object_t,
    out_builder: *mut *mut kmipkit_kmip30_operation_builder_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = kmip30_out_slot!(out_builder);
        let request = try_ffi!(create_credential_input(attributes, object));
        *out = try_ffi!(builder_handle(request));
        SUCCESS
    })
}

/// Executes one typed Create Credential exchange using the client's explicit version.
/// The 2.1 default is retained and rejects this 3.0 route before transport.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_create_credential_v1(
    client: *mut kmipkit_client_t,
    attributes: *mut kmipkit_attribute_set_t,
    object: *mut kmipkit_create_credential_object_t,
    limits: *mut kmipkit_codec_limits_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        let _out = kmip30_out_slot!(out_response);
        let request = try_ffi!(create_credential_input(attributes, object));
        execute_request(request, client, limits, None, out_response)
    })
}

/// Executes one typed Create Credential exchange with caller-selected timeout options.
/// Input records and spans are borrowed synchronously; returned response handles are caller-owned.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_create_credential_with_options_v1(
    client: *mut kmipkit_client_t,
    attributes: *mut kmipkit_attribute_set_t,
    object: *mut kmipkit_create_credential_object_t,
    limits: *mut kmipkit_codec_limits_t,
    options: *const kmipkit_request_options_v1_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        let _out = kmip30_out_slot!(out_response);
        let options = try_ffi!(options_input(options));
        let request = try_ffi!(create_credential_input(attributes, object));
        execute_request(request, client, limits, Some(options), out_response)
    })
}

fn create_group_input(attributes: *mut kmipkit_attribute_set_t) -> FfiResult<BuilderRequest> {
    Ok(BuilderRequest::CreateGroup(input_attributes(attributes)?))
}

/// Creates a reusable Create Group builder from copied semantic inputs.
/// The caller retains every input handle and byte span.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_create_group_builder_create(
    attributes: *mut kmipkit_attribute_set_t,
    out_builder: *mut *mut kmipkit_kmip30_operation_builder_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = kmip30_out_slot!(out_builder);
        let request = try_ffi!(create_group_input(attributes));
        *out = try_ffi!(builder_handle(request));
        SUCCESS
    })
}

/// Executes one typed Create Group exchange using the client's explicit version.
/// The 2.1 default is retained and rejects this 3.0 route before transport.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_create_group_v1(
    client: *mut kmipkit_client_t,
    attributes: *mut kmipkit_attribute_set_t,
    limits: *mut kmipkit_codec_limits_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        let _out = kmip30_out_slot!(out_response);
        let request = try_ffi!(create_group_input(attributes));
        execute_request(request, client, limits, None, out_response)
    })
}

/// Executes one typed Create Group exchange with caller-selected timeout options.
/// Input records and spans are borrowed synchronously; returned response handles are caller-owned.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_create_group_with_options_v1(
    client: *mut kmipkit_client_t,
    attributes: *mut kmipkit_attribute_set_t,
    limits: *mut kmipkit_codec_limits_t,
    options: *const kmipkit_request_options_v1_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        let _out = kmip30_out_slot!(out_response);
        let options = try_ffi!(options_input(options));
        let request = try_ffi!(create_group_input(attributes));
        execute_request(request, client, limits, Some(options), out_response)
    })
}

fn create_user_input(attributes: *mut kmipkit_attribute_set_t) -> FfiResult<BuilderRequest> {
    Ok(BuilderRequest::CreateUser(input_attributes(attributes)?))
}

/// Creates a reusable Create User builder from copied semantic inputs.
/// The caller retains every input handle and byte span.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_create_user_builder_create(
    attributes: *mut kmipkit_attribute_set_t,
    out_builder: *mut *mut kmipkit_kmip30_operation_builder_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = kmip30_out_slot!(out_builder);
        let request = try_ffi!(create_user_input(attributes));
        *out = try_ffi!(builder_handle(request));
        SUCCESS
    })
}

/// Executes one typed Create User exchange using the client's explicit version.
/// The 2.1 default is retained and rejects this 3.0 route before transport.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_create_user_v1(
    client: *mut kmipkit_client_t,
    attributes: *mut kmipkit_attribute_set_t,
    limits: *mut kmipkit_codec_limits_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        let _out = kmip30_out_slot!(out_response);
        let request = try_ffi!(create_user_input(attributes));
        execute_request(request, client, limits, None, out_response)
    })
}

/// Executes one typed Create User exchange with caller-selected timeout options.
/// Input records and spans are borrowed synchronously; returned response handles are caller-owned.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_create_user_with_options_v1(
    client: *mut kmipkit_client_t,
    attributes: *mut kmipkit_attribute_set_t,
    limits: *mut kmipkit_codec_limits_t,
    options: *const kmipkit_request_options_v1_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        let _out = kmip30_out_slot!(out_response);
        let options = try_ffi!(options_input(options));
        let request = try_ffi!(create_user_input(attributes));
        execute_request(request, client, limits, Some(options), out_response)
    })
}

fn deactivate_input(
    identifier: *const kmipkit_unique_identifier_v1_t,
) -> FfiResult<BuilderRequest> {
    Ok(BuilderRequest::Deactivate {
        identifier: identifier_input(identifier)?,
        reason: None,
        date: None,
    })
}

/// Creates a reusable Deactivate builder from copied semantic inputs.
/// The caller retains every input handle and byte span.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_deactivate_builder_create(
    identifier: *const kmipkit_unique_identifier_v1_t,
    out_builder: *mut *mut kmipkit_kmip30_operation_builder_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = kmip30_out_slot!(out_builder);
        let request = try_ffi!(deactivate_input(identifier));
        *out = try_ffi!(builder_handle(request));
        SUCCESS
    })
}

/// Executes one typed Deactivate exchange using the client's explicit version.
/// The 2.1 default is retained and rejects this 3.0 route before transport.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_deactivate_v1(
    client: *mut kmipkit_client_t,
    identifier: *const kmipkit_unique_identifier_v1_t,
    limits: *mut kmipkit_codec_limits_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        let _out = kmip30_out_slot!(out_response);
        let request = try_ffi!(deactivate_input(identifier));
        execute_request(request, client, limits, None, out_response)
    })
}

/// Executes one typed Deactivate exchange with caller-selected timeout options.
/// Input records and spans are borrowed synchronously; returned response handles are caller-owned.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_deactivate_with_options_v1(
    client: *mut kmipkit_client_t,
    identifier: *const kmipkit_unique_identifier_v1_t,
    limits: *mut kmipkit_codec_limits_t,
    options: *const kmipkit_request_options_v1_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        let _out = kmip30_out_slot!(out_response);
        let options = try_ffi!(options_input(options));
        let request = try_ffi!(deactivate_input(identifier));
        execute_request(request, client, limits, Some(options), out_response)
    })
}

fn decapsulate_input(
    identifier: *const kmipkit_unique_identifier_v1_t,
    ciphertext: *const kmipkit_byte_span_v1_t,
) -> FfiResult<BuilderRequest> {
    Ok(BuilderRequest::Decapsulate {
        identifier: identifier_input(identifier)?,
        ciphertext: bytes_input(ciphertext)?,
        parameters: None,
        attributes: None,
    })
}

/// Creates a reusable Decapsulate builder from copied semantic inputs.
/// The caller retains every input handle and byte span.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_decapsulate_builder_create(
    identifier: *const kmipkit_unique_identifier_v1_t,
    ciphertext: *const kmipkit_byte_span_v1_t,
    out_builder: *mut *mut kmipkit_kmip30_operation_builder_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = kmip30_out_slot!(out_builder);
        let request = try_ffi!(decapsulate_input(identifier, ciphertext));
        *out = try_ffi!(builder_handle(request));
        SUCCESS
    })
}

/// Executes one typed Decapsulate exchange using the client's explicit version.
/// The 2.1 default is retained and rejects this 3.0 route before transport.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_decapsulate_v1(
    client: *mut kmipkit_client_t,
    identifier: *const kmipkit_unique_identifier_v1_t,
    ciphertext: *const kmipkit_byte_span_v1_t,
    limits: *mut kmipkit_codec_limits_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        let _out = kmip30_out_slot!(out_response);
        let request = try_ffi!(decapsulate_input(identifier, ciphertext));
        execute_request(request, client, limits, None, out_response)
    })
}

/// Executes one typed Decapsulate exchange with caller-selected timeout options.
/// Input records and spans are borrowed synchronously; returned response handles are caller-owned.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_decapsulate_with_options_v1(
    client: *mut kmipkit_client_t,
    identifier: *const kmipkit_unique_identifier_v1_t,
    ciphertext: *const kmipkit_byte_span_v1_t,
    limits: *mut kmipkit_codec_limits_t,
    options: *const kmipkit_request_options_v1_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        let _out = kmip30_out_slot!(out_response);
        let options = try_ffi!(options_input(options));
        let request = try_ffi!(decapsulate_input(identifier, ciphertext));
        execute_request(request, client, limits, Some(options), out_response)
    })
}

fn encapsulate_input(
    identifier: *const kmipkit_unique_identifier_v1_t,
) -> FfiResult<BuilderRequest> {
    Ok(BuilderRequest::Encapsulate {
        identifier: identifier_input(identifier)?,
        parameters: None,
        attributes: None,
    })
}

/// Creates a reusable Encapsulate builder from copied semantic inputs.
/// The caller retains every input handle and byte span.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_encapsulate_builder_create(
    identifier: *const kmipkit_unique_identifier_v1_t,
    out_builder: *mut *mut kmipkit_kmip30_operation_builder_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = kmip30_out_slot!(out_builder);
        let request = try_ffi!(encapsulate_input(identifier));
        *out = try_ffi!(builder_handle(request));
        SUCCESS
    })
}

/// Executes one typed Encapsulate exchange using the client's explicit version.
/// The 2.1 default is retained and rejects this 3.0 route before transport.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_encapsulate_v1(
    client: *mut kmipkit_client_t,
    identifier: *const kmipkit_unique_identifier_v1_t,
    limits: *mut kmipkit_codec_limits_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        let _out = kmip30_out_slot!(out_response);
        let request = try_ffi!(encapsulate_input(identifier));
        execute_request(request, client, limits, None, out_response)
    })
}

/// Executes one typed Encapsulate exchange with caller-selected timeout options.
/// Input records and spans are borrowed synchronously; returned response handles are caller-owned.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_encapsulate_with_options_v1(
    client: *mut kmipkit_client_t,
    identifier: *const kmipkit_unique_identifier_v1_t,
    limits: *mut kmipkit_codec_limits_t,
    options: *const kmipkit_request_options_v1_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        let _out = kmip30_out_slot!(out_response);
        let options = try_ffi!(options_input(options));
        let request = try_ffi!(encapsulate_input(identifier));
        execute_request(request, client, limits, Some(options), out_response)
    })
}

fn obliterate_input(
    identifier: *const kmipkit_unique_identifier_v1_t,
) -> FfiResult<BuilderRequest> {
    Ok(BuilderRequest::Obliterate(identifier_input(identifier)?))
}

/// Creates a reusable Obliterate builder from copied semantic inputs.
/// The caller retains every input handle and byte span.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_obliterate_builder_create(
    identifier: *const kmipkit_unique_identifier_v1_t,
    out_builder: *mut *mut kmipkit_kmip30_operation_builder_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = kmip30_out_slot!(out_builder);
        let request = try_ffi!(obliterate_input(identifier));
        *out = try_ffi!(builder_handle(request));
        SUCCESS
    })
}

/// Executes one typed Obliterate exchange using the client's explicit version.
/// The 2.1 default is retained and rejects this 3.0 route before transport.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_obliterate_v1(
    client: *mut kmipkit_client_t,
    identifier: *const kmipkit_unique_identifier_v1_t,
    limits: *mut kmipkit_codec_limits_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        let _out = kmip30_out_slot!(out_response);
        let request = try_ffi!(obliterate_input(identifier));
        execute_request(request, client, limits, None, out_response)
    })
}

/// Executes one typed Obliterate exchange with caller-selected timeout options.
/// Input records and spans are borrowed synchronously; returned response handles are caller-owned.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_obliterate_with_options_v1(
    client: *mut kmipkit_client_t,
    identifier: *const kmipkit_unique_identifier_v1_t,
    limits: *mut kmipkit_codec_limits_t,
    options: *const kmipkit_request_options_v1_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        let _out = kmip30_out_slot!(out_response);
        let options = try_ffi!(options_input(options));
        let request = try_ffi!(obliterate_input(identifier));
        execute_request(request, client, limits, Some(options), out_response)
    })
}

/// Copies an ordered Attribute Set into Create Credential/Group/User or either KEM builder.
/// Other operation kinds reject this setter without changing their request.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_kmip30_operation_builder_set_attributes(
    builder: *mut kmipkit_kmip30_operation_builder_t,
    attributes: *mut kmipkit_attribute_set_t,
) -> i32 {
    ffi_entrypoint(|| {
        let attributes = try_ffi!(input_attributes(attributes));
        try_ffi!(update_builder(builder, |builder| {
            match &mut builder.request {
                BuilderRequest::CreateCredential {
                    attributes: target, ..
                }
                | BuilderRequest::CreateGroup(target)
                | BuilderRequest::CreateUser(target) => *target = attributes,
                BuilderRequest::Encapsulate {
                    attributes: target, ..
                }
                | BuilderRequest::Decapsulate {
                    attributes: target, ..
                } => *target = Some(attributes),
                _ => return Err(ERROR_INVALID_INPUT),
            }
            Ok(())
        }));
        SUCCESS
    })
}

/// Copies existing typed Cryptographic Parameters into a KEM builder.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_kmip30_operation_builder_set_cryptographic_parameters(
    builder: *mut kmipkit_kmip30_operation_builder_t,
    parameters: *mut kmipkit_cryptographic_parameters_t,
) -> i32 {
    ffi_entrypoint(|| {
        let handle = read_handle!(parameters, CryptographicParameters);
        let HandleValue::CryptographicParameters(parameters) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let parameters = try_ffi!(
            parameters
                .with_ttlv(|value| value.try_clone())
                .map_err(model_status)
        );
        try_ffi!(update_builder(builder, |builder| {
            match &mut builder.request {
                BuilderRequest::Encapsulate {
                    parameters: target, ..
                }
                | BuilderRequest::Decapsulate {
                    parameters: target, ..
                } => *target = Some(parameters),
                _ => return Err(ERROR_INVALID_INPUT),
            }
            Ok(())
        }));
        SUCCESS
    })
}

/// Copies the typed request model's optional Deactivation Reason Structure.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_kmip30_operation_builder_set_deactivation_reason(
    builder: *mut kmipkit_kmip30_operation_builder_t,
    reason: *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let reason = try_ffi!(copy_required_structure_handle(reason));
        try_ffi!(update_builder(builder, |builder| {
            let BuilderRequest::Deactivate { reason: target, .. } = &mut builder.request else {
                return Err(ERROR_INVALID_INPUT);
            };
            *target = Some(reason);
            Ok(())
        }));
        SUCCESS
    })
}

/// Sets the optional Deactivation Date in seconds since the Unix epoch.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_kmip30_operation_builder_set_deactivation_date(
    builder: *mut kmipkit_kmip30_operation_builder_t,
    unix_seconds: i64,
) -> i32 {
    ffi_entrypoint(|| {
        try_ffi!(update_builder(builder, |builder| {
            let BuilderRequest::Deactivate { date, .. } = &mut builder.request else {
                return Err(ERROR_INVALID_INPUT);
            };
            *date = Some(unix_seconds);
            Ok(())
        }));
        SUCCESS
    })
}

/// Sets an assigned or extension Asynchronous Indicator without retrying or polling.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_kmip30_operation_builder_set_asynchronous_indicator(
    builder: *mut kmipkit_kmip30_operation_builder_t,
    indicator: u32,
) -> i32 {
    ffi_entrypoint(|| {
        if !(1..=3).contains(&indicator) && !(0x8000_0000..=0x8FFF_FFFF).contains(&indicator) {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        try_ffi!(update_builder(builder, |builder| {
            builder.asynchronous_indicator = Some(indicator);
            Ok(())
        }));
        SUCCESS
    })
}

/// Validates the required typed semantic inputs locally without transport.
/// The client's version, catalog rules, and request are checked again during execution.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_kmip30_operation_builder_validate(
    builder: *mut kmipkit_kmip30_operation_builder_t,
) -> i32 {
    ffi_entrypoint(|| {
        try_ffi!(update_builder(builder, |builder| builder
            .build_request()
            .map(|_| ())));
        SUCCESS
    })
}

/// Copies this builder into one caller-owned typed batch item without consuming it.
/// Header options, including the Asynchronous Indicator, apply when the builder executes.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_kmip30_operation_builder_build_batch_item(
    builder: *mut kmipkit_kmip30_operation_builder_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = kmip30_out_slot!(out_item);
        try_ffi!(update_builder(builder, |builder| {
            *out = batch_item_handle(builder.build_request()?)?;
            Ok(())
        }));
        SUCCESS
    })
}

fn options_input(options: *const kmipkit_request_options_v1_t) -> FfiResult<RequestOptions> {
    if options.is_null() || !options.is_aligned() {
        return Err(ERROR_INVALID_INPUT);
    }
    // SAFETY: the caller lends an aligned, readable v1 options record for this call.
    request_options_from_v1(unsafe { &*options })
}

fn execute_request(
    request: BuilderRequest,
    client: *mut kmipkit_client_t,
    limits: *mut kmipkit_codec_limits_t,
    options: Option<RequestOptions>,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    let builder = Kmip30OperationBuilder::new(request);
    let request = try_ffi!(builder.build_request());
    fluent::execute_fluent_item(
        client,
        request,
        limits,
        &options.unwrap_or_default(),
        builder.asynchronous_indicator,
        out_response,
    )
}

fn execute_builder(
    builder: *mut kmipkit_kmip30_operation_builder_t,
    client: *mut kmipkit_client_t,
    limits: *mut kmipkit_codec_limits_t,
    options: Option<RequestOptions>,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    // Copy the request and release the builder lock before the untrusted transport runs.
    let mut request = None;
    let mut indicator = None;
    try_ffi!(update_builder(builder, |builder| {
        request = Some(builder.build_request()?);
        indicator = builder.asynchronous_indicator;
        Ok(())
    }));
    let Some(request) = request else {
        return record_ffi_error(ERROR_INTERNAL);
    };
    fluent::execute_fluent_item(
        client,
        request,
        limits,
        &options.unwrap_or_default(),
        indicator,
        out_response,
    )
}

/// Executes exactly one exchange through the existing typed client; no retry or implicit 3.0.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_kmip30_operation_builder_execute_v1(
    builder: *mut kmipkit_kmip30_operation_builder_t,
    client: *mut kmipkit_client_t,
    limits: *mut kmipkit_codec_limits_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        let _out = kmip30_out_slot!(out_response);
        execute_builder(builder, client, limits, None, out_response)
    })
}

/// Executes one typed exchange with borrowed exchange-local timeout options.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_kmip30_operation_builder_execute_with_options_v1(
    builder: *mut kmipkit_kmip30_operation_builder_t,
    client: *mut kmipkit_client_t,
    limits: *mut kmipkit_codec_limits_t,
    options: *const kmipkit_request_options_v1_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        let _out = kmip30_out_slot!(out_response);
        let options = try_ffi!(options_input(options));
        execute_builder(builder, client, limits, Some(options), out_response)
    })
}

/// Releases the native builder and all its owned copies; null is accepted.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_kmip30_operation_builder_release(
    builder: *mut kmipkit_kmip30_operation_builder_t,
) {
    ffi_entrypoint(|| release(builder, Kind::Kmip30OperationBuilder));
}

/// Borrows the actual typed 3.0 response identifier until the response item is released.
/// Pending, Failure, Obliterate, and absent identifiers return `out_present == 0`.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_kmip30_identifier(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_identifier: *mut kmipkit_unique_identifier_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        if out_present.is_null()
            || !out_present.is_aligned()
            || out_identifier.is_null()
            || !out_identifier.is_aligned()
            || ffi_output_ranges_overlap(out_present, out_identifier)
        {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let present = scalar_out!(out_present);
        *present = 0;
        try_ffi!(write_unique_identifier_v1(out_identifier, None));
        let handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&handle));
        let identifier = match item.kmip30_response() {
            Some(Kmip30OperationResponse::CreateCredential(value)) => value.unique_identifier(),
            Some(Kmip30OperationResponse::CreateGroup(value)) => value.unique_identifier(),
            Some(Kmip30OperationResponse::CreateUser(value)) => value.unique_identifier(),
            Some(Kmip30OperationResponse::Deactivate(value)) => value.unique_identifier(),
            Some(Kmip30OperationResponse::Decapsulate(value)) => value.unique_identifier(),
            Some(Kmip30OperationResponse::Encapsulate(value)) => value.unique_identifier(),
            _ => None,
        };
        try_ffi!(write_unique_identifier_v1(out_identifier, identifier));
        *present = u8::from(identifier.is_some());
        SUCCESS
    })
}

/// Borrows only Encapsulate's public ciphertext Data until the item is released.
/// Decapsulate never exposes server-managed shared-secret material through this accessor.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_encapsulate_data(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_data: *mut kmipkit_byte_span_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        if out_present.is_null()
            || !out_present.is_aligned()
            || out_data.is_null()
            || !out_data.is_aligned()
            || ffi_output_ranges_overlap(out_present, out_data)
        {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let present = scalar_out!(out_present);
        let data = scalar_out!(out_data);
        *present = 0;
        *data = kmipkit_byte_span_v1_t::default();
        let handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&handle));
        if let Some(Kmip30OperationResponse::Encapsulate(response)) = item.kmip30_response()
            && let Some(bytes) = response.data()
        {
            *data = kmipkit_byte_span_v1_t {
                data: bytes.as_ptr(),
                length: try_ffi!(u64::try_from(bytes.len()).map_err(|_| ERROR_RESOURCE_LIMIT)),
            };
            *present = 1;
        }
        SUCCESS
    })
}
