//! Common validated client and transport C ABI implementation.
#![allow(clippy::wildcard_imports)] // Shares the parent module's private FFI handle invariants.

use super::*;
use crate::kmipkit_server_request_handler_v2_t;
use crate::{
    kmipkit_listener_options_v1_t, kmipkit_protocol_version_v1_t, kmipkit_s2c_operation_result_v1_t,
};
use crate::{
    kmipkit_s2c_capabilities_v1_t, kmipkit_s2c_capabilities_v2_t,
    kmipkit_server_request_handler_v5_t, kmipkit_server_request_handler_v6_t,
};
use crate::{kmipkit_server_request_handler_v3_t, kmipkit_server_request_handler_v4_t};
use zeroize::{Zeroize, Zeroizing};

#[path = "client_abi/fluent.rs"]
mod fluent;

#[path = "client_abi/fluent_crypto.rs"]
mod fluent_crypto;

#[cfg(test)]
// These tests intentionally pass Rust references as raw C pointers and slice the byte above the
// exact allocation bound to exercise the FFI's one-byte-over rejection path.
#[allow(clippy::borrow_as_ptr, clippy::range_plus_one)]
#[path = "../../tests/support/client_abi_tests.rs"]
mod client_abi_tests;

fn batch_response_item_from_handle(
    handle: &Handle,
) -> FfiResult<&kmipkit_client::ClientBatchItemResponse> {
    let HandleValue::ClientBatchItemResponse { response, index } = &handle.value else {
        return Err(ERROR_INVALID_INPUT);
    };
    response.get(*index).ok_or(ERROR_INTERNAL)
}

fn copy_attribute_set_model(
    attributes: *mut crate::kmipkit_attribute_set_t,
) -> FfiResult<AttributeSet> {
    let handle = reference_handle(attributes, Kind::AttributeSet)?;
    let HandleValue::AttributeSet(source) = &handle.value else {
        return Err(ERROR_INVALID_INPUT);
    };
    let source = source.lock().map_err(|_| ERROR_INTERNAL)?;
    let mut items = Vec::new();
    items
        .try_reserve_exact(source.len())
        .map_err(|_| ERROR_RESOURCE_LIMIT)?;
    for item in source.as_items() {
        items.push(item.try_clone().map_err(model_status)?);
    }
    AttributeSet::try_new(items).map_err(|_| ERROR_INVALID_INPUT)
}

fn copy_optional_attribute_set_model(
    attributes: *mut crate::kmipkit_attribute_set_t,
) -> FfiResult<Option<AttributeSet>> {
    if attributes.is_null() {
        return Ok(None);
    }
    copy_attribute_set_model(attributes).map(Some)
}

fn copy_optional_cryptographic_parameters_structure(
    parameters: *mut crate::kmipkit_cryptographic_parameters_t,
) -> FfiResult<Option<Structure>> {
    if parameters.is_null() {
        return Ok(None);
    }
    let handle = reference_handle(parameters, Kind::CryptographicParameters)?;
    let HandleValue::CryptographicParameters(source) = &handle.value else {
        return Err(ERROR_INVALID_INPUT);
    };
    source
        .with_ttlv(|view| clone_structure_view(&view))
        .map(Some)
}

fn create_credential_object_from_item(
    object_type_raw: u32,
    item: Item,
) -> FfiResult<CreateCredentialObject> {
    let object = ManagedObject::try_from_item(ObjectType::from_raw(object_type_raw), item)
        .map_err(|error| match error {
            ManagedObjectError::TtlvModelLimit => ERROR_RESOURCE_LIMIT,
            _ => ERROR_INVALID_INPUT,
        })?;
    match object {
        ManagedObject::Certificate(value) => Ok(CreateCredentialObject::Certificate(value)),
        ManagedObject::PasswordCredential(value) => {
            Ok(CreateCredentialObject::PasswordCredential(value))
        }
        ManagedObject::DeviceCredential(value) => {
            Ok(CreateCredentialObject::DeviceCredential(value))
        }
        ManagedObject::HashedPasswordCredential(value) => {
            Ok(CreateCredentialObject::HashedPasswordCredential(value))
        }
        ManagedObject::OneTimePasswordCredential(value) => {
            Ok(CreateCredentialObject::OneTimePasswordCredential(value))
        }
        _ => Err(ERROR_INVALID_INPUT),
    }
}

fn require_success_response_ttlv<T>(response_ttlv: Option<FfiResult<T>>) -> FfiResult<T> {
    match response_ttlv {
        Some(response_ttlv) => response_ttlv,
        None => Err(ERROR_INTERNAL),
    }
}

fn clone_query_response_field_item(
    field: &kmipkit_protocol::QueryResponseField,
) -> FfiResult<Option<Item>> {
    // Callers enumerate fields individually; rebuilding the full payload here would be quadratic.
    use kmipkit_protocol::QueryResponseField;

    match field.kind() {
        QueryResponseFieldKind::Enumeration => Ok(None),
        QueryResponseFieldKind::TextString => {
            let (QueryResponseField::VendorIdentification(value)
            | QueryResponseField::ApplicationNamespace(value)) = field
            else {
                return Err(ERROR_INTERNAL);
            };
            let tag = RawTag::new(field.tag())
                .and_then(|raw| raw.try_checked())
                .map_err(model_status)?;
            Item::new(tag, Value::text_string(value.clone()))
                .map(Some)
                .map_err(model_status)
        }
        QueryResponseFieldKind::Structure | QueryResponseFieldKind::Unknown => field
            .with_ttlv(super::clone_item)
            .ok_or(ERROR_INTERNAL)?
            .map(Some),
    }
}

fn outcome_has_success_or_pending_response(outcome: &kmipkit_client::ClientBatchOutcome) -> bool {
    outcome.result().status().raw() == 0 || outcome.asynchronous_correlation_value().is_some()
}

fn copy_transport_span(data: *const u8, length: u64, maximum: u64) -> FfiResult<Vec<u8>> {
    let bytes = input_span(data, length, maximum)?;
    let mut owned = Vec::new();
    owned
        .try_reserve_exact(bytes.len())
        .map_err(|_| ERROR_RESOURCE_LIMIT)?;
    owned.extend_from_slice(bytes);
    Ok(owned)
}

/// Copies the complete ordered PKCS#11 response for Success, Failure, or Pending.
///
/// The typed result handle is available only for a valid server response. Local transport,
/// decoding, and validation errors do not expose a response body.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_pkcs11_ttlv(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_response_ttlv: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        let response_ttlv = out_slot!(out_response_ttlv);
        *present = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        let Some(pkcs11) = item.outcome().response().pkcs11() else {
            return SUCCESS;
        };
        let structure = try_ffi!(pkcs11.with_ttlv(|view| super::clone_structure_view(&view)));
        *response_ttlv = try_ffi!(try_make_handle(
            Kind::Structure,
            HandleValue::Structure(structure),
        ));
        *present = 1;
        SUCCESS
    })
}

/// Copies the optional PKCS#11 Interface Structure from a successful result.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_pkcs11_interface_ttlv(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_interface: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        *present = 0;
        let interface = out_slot!(out_interface);
        let item = read_handle!(item, ClientBatchItemResponse);
        let Some(pkcs11) = try_ffi!(pkcs11_response_from_handle(&item)) else {
            return SUCCESS;
        };
        let Some(structure) = pkcs11.with_interface(|view| super::clone_structure_view(&view))
        else {
            return SUCCESS;
        };
        let structure = try_ffi!(structure);
        *interface = try_ffi!(try_make_handle(
            Kind::Structure,
            HandleValue::Structure(structure),
        ));
        *present = 1;
        SUCCESS
    })
}

/// Copies the optional PKCS#11 Function Structure from a successful result.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_pkcs11_function_ttlv(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_function: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        *present = 0;
        let function = out_slot!(out_function);
        let item = read_handle!(item, ClientBatchItemResponse);
        let Some(pkcs11) = try_ffi!(pkcs11_response_from_handle(&item)) else {
            return SUCCESS;
        };
        let Some(structure) = pkcs11.with_function(|view| super::clone_structure_view(&view))
        else {
            return SUCCESS;
        };
        let structure = try_ffi!(structure);
        *function = try_ffi!(try_make_handle(
            Kind::Structure,
            HandleValue::Structure(structure),
        ));
        *present = 1;
        SUCCESS
    })
}

/// Returns the exact Correlation Value bytes borrowed from `item`.
///
/// `out_value.data` remains valid only while the response item handle remains
/// live. The caller must copy the span before releasing that handle. A present
/// empty Byte String is reported as `out_present == 1` and a zero-length span;
/// absence is `out_present == 0` with the same empty span.
///
/// # Safety
///
/// `item` must be null or a live response-item handle. `out_present` and
/// `out_value` must be aligned, writable pointers to their respective output
/// types and must not overlap each other or the handle storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kmipkit_client_batch_item_response_pkcs11_correlation_value(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_value: *mut crate::kmipkit_byte_span_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        *present = 0;
        if out_value.is_null() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        // SAFETY: `out_value` is checked non-null and is the caller's writable output slot.
        let value = unsafe { &mut *out_value };
        *value = crate::kmipkit_byte_span_v1_t {
            data: ptr::null(),
            length: 0,
        };
        let item = read_handle!(item, ClientBatchItemResponse);
        let Some(pkcs11) = try_ffi!(pkcs11_response_from_handle(&item)) else {
            return SUCCESS;
        };
        let Some(status) =
            pkcs11.with_correlation_value(|bytes| set_borrowed_span(present, value, Some(bytes)))
        else {
            return SUCCESS;
        };
        try_ffi!(status);
        SUCCESS
    })
}

/// Copies the optional PKCS#11 Output Parameters Structure from a result.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_pkcs11_output_parameters_ttlv(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_output_parameters: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        *present = 0;
        let output_parameters = out_slot!(out_output_parameters);
        let item = read_handle!(item, ClientBatchItemResponse);
        let Some(pkcs11) = try_ffi!(pkcs11_response_from_handle(&item)) else {
            return SUCCESS;
        };
        let Some(structure) =
            pkcs11.with_output_parameters(|view| super::clone_structure_view(&view))
        else {
            return SUCCESS;
        };
        let structure = try_ffi!(structure);
        *output_parameters = try_ffi!(try_make_handle(
            Kind::Structure,
            HandleValue::Structure(structure),
        ));
        *present = 1;
        SUCCESS
    })
}

/// Copies the optional PKCS#11 Return Code Structure from a result.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_pkcs11_return_code_ttlv(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_return_code: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        *present = 0;
        let return_code = out_slot!(out_return_code);
        let item = read_handle!(item, ClientBatchItemResponse);
        let Some(pkcs11) = try_ffi!(pkcs11_response_from_handle(&item)) else {
            return SUCCESS;
        };
        let Some(structure) = pkcs11.with_return_code(|view| super::clone_structure_view(&view))
        else {
            return SUCCESS;
        };
        let structure = try_ffi!(structure);
        *return_code = try_ffi!(try_make_handle(
            Kind::Structure,
            HandleValue::Structure(structure),
        ));
        *present = 1;
        SUCCESS
    })
}

fn pkcs11_response_from_handle(
    handle: &Handle,
) -> FfiResult<Option<&kmipkit_protocol::Pkcs11Response>> {
    let HandleValue::ClientBatchItemResponse { response, index } = &handle.value else {
        return Err(ERROR_INVALID_INPUT);
    };
    let item = response.get(*index).ok_or(ERROR_INTERNAL)?;
    Ok(item.outcome().response().pkcs11())
}

/// Copies the complete typed Interop response for Success, Failure, or Pending.
///
/// The generic response is available only through the successful result handle;
/// local transport, decoding, and validation errors do not expose response bodies.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_interop_ttlv(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_response_ttlv: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        let response_ttlv = out_slot!(out_response_ttlv);
        *present = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        let outcome = item.outcome();
        let response = outcome.response();
        let Some(interop) = response.interop() else {
            return SUCCESS;
        };
        let structure = try_ffi!(interop.with_ttlv(|view| super::clone_structure_view(&view)));
        *response_ttlv = try_ffi!(try_make_handle(
            Kind::Structure,
            HandleValue::Structure(structure),
        ));
        *present = 1;
        SUCCESS
    })
}

/// Copies the complete ordered Log response TTLV for Success, Failure, or Pending.
///
/// The response is available only through a valid result handle; local
/// transport, decoding, and validation errors do not expose response bodies.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_log_ttlv(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_response_ttlv: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        let response_ttlv = out_slot!(out_response_ttlv);
        *present = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        let outcome = item.outcome();
        let response = outcome.response();
        let Some(log) = response.log() else {
            return SUCCESS;
        };
        let structure = try_ffi!(log.with_ttlv(|view| super::clone_structure_view(&view)));
        *response_ttlv = try_ffi!(try_make_handle(
            Kind::Structure,
            HandleValue::Structure(structure),
        ));
        *present = 1;
        SUCCESS
    })
}

fn read_transport_file_with_budget(path: &str, budget: u64) -> FfiResult<Vec<u8>> {
    let metadata = fs::metadata(path).map_err(|_| ERROR_INVALID_INPUT)?;
    if metadata.len() > budget {
        return Err(ERROR_RESOURCE_LIMIT);
    }
    let file = fs::File::open(path).map_err(|_| ERROR_INVALID_INPUT)?;
    read_transport_reader_with_budget(file, metadata.len(), budget)
}

fn read_transport_reader_with_budget<R: Read>(
    mut reader: R,
    expected_length: u64,
    budget: u64,
) -> FfiResult<Vec<u8>> {
    if expected_length > budget {
        return Err(ERROR_RESOURCE_LIMIT);
    }
    let expected_length = usize::try_from(expected_length).map_err(|_| ERROR_RESOURCE_LIMIT)?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(expected_length)
        .map_err(|_| ERROR_RESOURCE_LIMIT)?;
    let mut scratch = [0_u8; 4096];
    loop {
        let current_length = u64::try_from(bytes.len()).map_err(|_| ERROR_RESOURCE_LIMIT)?;
        let remaining = budget
            .checked_sub(current_length)
            .ok_or(ERROR_RESOURCE_LIMIT)?;
        if remaining == 0 {
            let mut overflow_probe = [0_u8; 1];
            return match reader.read(&mut overflow_probe) {
                Ok(0) => Ok(bytes),
                Ok(_) => Err(ERROR_RESOURCE_LIMIT),
                Err(_) => Err(ERROR_INVALID_INPUT),
            };
        }
        let chunk_limit = usize::try_from(remaining.min(scratch.len() as u64))
            .map_err(|_| ERROR_RESOURCE_LIMIT)?;
        let count = reader
            .read(&mut scratch[..chunk_limit])
            .map_err(|_| ERROR_INVALID_INPUT)?;
        if count == 0 {
            return Ok(bytes);
        }
        bytes.try_reserve(count).map_err(|_| ERROR_RESOURCE_LIMIT)?;
        bytes.extend_from_slice(&scratch[..count]);
    }
}

struct SecretBuffer {
    bytes: Vec<u8>,
    #[cfg(test)]
    observer: Option<SecretBufferObserver>,
}

impl SecretBuffer {
    fn read_from<R: Read>(reader: R, maximum: u64) -> FfiResult<Self> {
        Self::read_from_inner(
            reader,
            maximum,
            #[cfg(test)]
            None,
        )
    }

    #[allow(dead_code)]
    #[cfg(test)]
    fn read_from_with_observer_for_test<R: Read>(
        reader: R,
        maximum: u64,
        observer: SecretBufferObserver,
    ) -> FfiResult<Self> {
        Self::read_from_inner(reader, maximum, Some(observer))
    }

    fn read_from_inner<R: Read>(
        mut reader: R,
        maximum: u64,
        #[cfg(test)] observer: Option<SecretBufferObserver>,
    ) -> FfiResult<Self> {
        let mut owned = Self {
            bytes: Vec::new(),
            #[cfg(test)]
            observer,
        };
        let mut scratch = Zeroizing::new([0_u8; 4096]);
        loop {
            let count = reader
                .read(&mut scratch[..])
                .map_err(|_| ERROR_INVALID_INPUT)?;
            if count == 0 {
                return Ok(owned);
            }
            owned.append(&scratch[..count], maximum)?;
            scratch[..count].zeroize();
        }
    }

    fn append(&mut self, bytes: &[u8], maximum: u64) -> FfiResult<()> {
        let next_length = self
            .bytes
            .len()
            .checked_add(bytes.len())
            .ok_or(ERROR_RESOURCE_LIMIT)?;
        if u64::try_from(next_length).map_err(|_| ERROR_RESOURCE_LIMIT)? > maximum {
            return Err(ERROR_RESOURCE_LIMIT);
        }
        if next_length > self.bytes.capacity() {
            let mut replacement = Vec::new();
            replacement
                .try_reserve_exact(next_length)
                .map_err(|_| ERROR_RESOURCE_LIMIT)?;
            replacement.extend_from_slice(&self.bytes);
            replacement.extend_from_slice(bytes);
            self.scrub_current_allocation();
            let previous = std::mem::replace(&mut self.bytes, replacement);
            drop(previous);
        } else {
            self.bytes.extend_from_slice(bytes);
        }
        Ok(())
    }

    fn scrub_current_allocation(&mut self) {
        self.bytes.zeroize();
        #[cfg(test)]
        if let Some(observer) = &self.observer {
            observer.record(self.bytes.iter().all(|byte| *byte == 0));
        }
    }

    fn into_vec(mut self) -> Vec<u8> {
        std::mem::take(&mut self.bytes)
    }
}

impl Drop for SecretBuffer {
    fn drop(&mut self) {
        self.scrub_current_allocation();
    }
}

#[allow(dead_code)]
#[cfg(test)]
#[derive(Clone, Default)]
struct SecretBufferObserver(std::sync::Arc<std::sync::Mutex<Vec<bool>>>);

#[allow(dead_code)]
#[cfg(test)]
impl SecretBufferObserver {
    fn new() -> Self {
        Self::default()
    }

    fn record(&self, scrubbed: bool) {
        if let Ok(mut records) = self.0.lock() {
            records.push(scrubbed);
        }
    }

    fn initialized_bytes_were_zeroized(&self) -> bool {
        self.0
            .lock()
            .is_ok_and(|records| !records.is_empty() && records.iter().all(|record| *record))
    }
}

fn private_key_file_bytes(path: &str) -> FfiResult<Vec<u8>> {
    let metadata = fs::metadata(path).map_err(|_| ERROR_INVALID_INPUT)?;
    if metadata.len() > MAX_TRANSPORT_MATERIAL_BYTES {
        return Err(ERROR_RESOURCE_LIMIT);
    }
    let file = fs::File::open(path).map_err(|_| ERROR_INVALID_INPUT)?;
    let secret = SecretBuffer::read_from(file, MAX_TRANSPORT_MATERIAL_BYTES)?;
    Ok(secret.into_vec())
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

#[allow(dead_code)]
#[cfg(test)]
fn certificate_input_bytes(input: &kmipkit_transport_input_v1_t) -> FfiResult<Vec<u8>> {
    certificate_input_bytes_with_budget(input, MAX_TRANSPORT_MATERIAL_BYTES)
}

fn certificate_input_bytes_with_budget(
    input: &kmipkit_transport_input_v1_t,
    budget: u64,
) -> FfiResult<Vec<u8>> {
    match input.source_kind {
        TRANSPORT_SOURCE_MEMORY => copy_transport_span(input.data.data, input.data.length, budget),
        TRANSPORT_SOURCE_FILE => {
            let path = transport_input_path(input)?;
            read_transport_file_with_budget(&path, budget)
        }
        _ => Err(ERROR_INVALID_INPUT),
    }
}

fn certificate_input_materials(inputs: &[kmipkit_transport_input_v1_t]) -> FfiResult<Vec<Vec<u8>>> {
    validate_transport_input_total(inputs)?;
    let mut remaining = MAX_TRANSPORT_MATERIAL_BYTES;
    let mut materials = Vec::new();
    materials
        .try_reserve_exact(inputs.len())
        .map_err(|_| ERROR_RESOURCE_LIMIT)?;
    for input in inputs {
        let bytes = certificate_input_bytes_with_budget(input, remaining)?;
        remaining = remaining
            .checked_sub(u64::try_from(bytes.len()).map_err(|_| ERROR_RESOURCE_LIMIT)?)
            .ok_or(ERROR_RESOURCE_LIMIT)?;
        materials.push(bytes);
    }
    Ok(materials)
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
    let materials = certificate_input_materials(inputs)?;
    if encoding == TRANSPORT_ENCODING_DER {
        return Ok(CertificateInput::from_der(materials));
    }
    let mut pem = Vec::new();
    for bytes in materials {
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
            let path = transport_input_path(input)?;
            Ok(PrivateKeyInput::from_pem(private_key_file_bytes(&path)?))
        }
        (TRANSPORT_SOURCE_FILE, TRANSPORT_ENCODING_DER) => {
            let path = transport_input_path(input)?;
            Ok(PrivateKeyInput::from_der(private_key_file_bytes(&path)?))
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

fn request_timeout_override_v1(
    input: &kmipkit_timeout_limit_v1_t,
) -> FfiResult<Option<TimeoutLimit>> {
    if input.reserved != 0 || input.reserved2 != 0 {
        return Err(ERROR_INVALID_INPUT);
    }
    match input.kind {
        0 if input.seconds == 0 && input.nanoseconds == 0 => Ok(None),
        TRANSPORT_TIMEOUT_BOUNDED if input.nanoseconds < 1_000_000_000 => Ok(Some(
            TimeoutLimit::Bounded(Duration::new(input.seconds, input.nanoseconds)),
        )),
        TRANSPORT_TIMEOUT_UNBOUNDED if input.seconds == 0 && input.nanoseconds == 0 => {
            Ok(Some(TimeoutLimit::Unbounded))
        }
        _ => Err(ERROR_INVALID_INPUT),
    }
}

fn request_options_from_v1(input: &kmipkit_request_options_v1_t) -> FfiResult<RequestOptions> {
    let required_size = u32::try_from(std::mem::size_of::<kmipkit_request_options_v1_t>())
        .map_err(|_| ERROR_INVALID_INPUT)?;
    if input.abi_version != 1 || input.struct_size < required_size {
        return Err(ERROR_INVALID_INPUT);
    }

    let connect = request_timeout_override_v1(&input.connect_timeout)?;
    let write = request_timeout_override_v1(&input.write_timeout)?;
    let read = request_timeout_override_v1(&input.read_timeout)?;
    let total = request_timeout_override_v1(&input.total_timeout)?;
    let options = RequestOptions::default();
    let options = connect.map_or(options, |value| options.with_connect(value));
    let options = write.map_or(options, |value| options.with_write(value));
    let options = read.map_or(options, |value| options.with_read(value));
    let options = total.map_or(options, |value| options.with_total(value));
    Ok(options)
}

fn request_options_from_pointer_v1(
    input: *const kmipkit_request_options_v1_t,
) -> FfiResult<RequestOptions> {
    if input.is_null() {
        return Err(ERROR_INVALID_INPUT);
    }
    // SAFETY: the versioned C ABI requires a non-null input pointer to reference
    // one readable request-options record for the duration of this call.
    request_options_from_v1(unsafe { &*input })
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
        || input.flags
            & !(TRANSPORT_FLAG_TLS_SERVER_NAME
                | TRANSPORT_FLAG_TARGET_URI
                | TRANSPORT_FLAG_TLS12_COMPAT)
            != 0
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
            let materials = certificate_input_materials(trust_certificates)?;
            let authorities = trust_certificates
                .iter()
                .zip(materials)
                .map(|(input, bytes)| match transport_input_encoding(input)? {
                    TRANSPORT_ENCODING_PEM => Ok(CertificateInput::from_pem(bytes)),
                    TRANSPORT_ENCODING_DER => Ok(CertificateInput::from_der(vec![bytes])),
                    _ => Err(ERROR_INVALID_INPUT),
                })
                .collect::<FfiResult<Vec<_>>>()?;
            TrustSource::certificate_authorities(authorities)
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
    if input.flags & TRANSPORT_FLAG_TLS12_COMPAT != 0 {
        builder = builder.tls_version_policy(kmipkit_transport::TlsVersionPolicy::Tls12And13Compat);
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
        *out = try_ffi!(try_make_handle(
            Kind::TransportConfig,
            HandleValue::TransportConfig(Box::new(configuration)),
        ));
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
        if let Err(status) = reference_handle(transport_handle, Kind::TransportConfig) {
            return record_ffi_error(status);
        }
        let client_configuration = client_extension::ClientConfiguration::from_shared_registry(
            configuration.shared_extension_registry(),
        )
        .with_wire_encoding(configuration.wire_encoding())
        .with_protocol_version(configuration.protocol_version());
        let transport_configuration = match owned_payload(transport_handle, Kind::TransportConfig) {
            Ok(HandleValue::TransportConfig(configuration)) => *configuration,
            Ok(_) => return record_ffi_error(ERROR_INVALID_INPUT),
            Err(status) => return record_ffi_error(status),
        };
        *transport_slot = ptr::null_mut();
        match Client::new(client_configuration, transport_configuration) {
            Ok(client) => {
                *out = try_ffi!(try_make_handle(
                    Kind::Client,
                    HandleValue::Client(Box::new(std::sync::Mutex::new(client))),
                ));
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
        *out = try_ffi!(try_make_handle(
            Kind::ClientBatch,
            HandleValue::ClientBatch(Arc::new(Mutex::new(Some(ClientBatchBuilder::new())))),
        ));
        SUCCESS
    })
}

/// Attaches Authentication to this Request Message and takes ownership on success.
///
/// Authentication is encoded once in the Request Header and applies to every Batch Item. It is
/// not stored in Client configuration. If validation fails, the Authentication handle remains
/// owned by the caller.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_set_authentication(
    batch: *mut kmipkit_client_batch_t,
    authentication: *mut kmipkit_authentication_t,
) -> i32 {
    ffi_entrypoint(|| {
        if authentication.is_null() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let batch_handle = read_handle!(batch, ClientBatch);
        let HandleValue::ClientBatch(batch) = &batch_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(mut batch) = batch.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        let Some(batch) = batch.as_mut() else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let authentication_handle = match reference_handle(authentication, Kind::Authentication) {
            Ok(handle) => handle,
            Err(status) => return record_ffi_error(status),
        };
        if !matches!(&authentication_handle.value, HandleValue::Authentication(_)) {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        drop(authentication_handle);
        let authentication = match owned_payload(authentication, Kind::Authentication) {
            Ok(HandleValue::Authentication(authentication)) => authentication,
            Ok(_) => return record_ffi_error(ERROR_INVALID_INPUT),
            Err(status) => return record_ffi_error(status),
        };
        batch.authentication = Some(authentication);
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
        let item = read_handle!(item_handle, BatchItem);
        if !matches!(&item.value, HandleValue::BatchItem(_)) {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        drop(item);
        let batch = read_handle!(batch, ClientBatch);
        let HandleValue::ClientBatch(batch) = &batch.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(mut batch) = batch.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        let Some(batch) = batch.as_mut() else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        if let Err(status) = reference_handle(item_handle, Kind::BatchItem) {
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
pub extern "C" fn kmipkit_client_batch_item_count(
    batch: *mut kmipkit_client_batch_t,
    out_count: *mut u64,
) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_count);
        *out = 0;
        let batch_handle = read_handle!(batch, ClientBatch);
        let HandleValue::ClientBatch(batch) = &batch_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(batch) = batch.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        let Some(batch) = batch.as_ref() else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = try_ffi!(u64::try_from(batch.items.len()).map_err(|_| ERROR_RESOURCE_LIMIT));
        SUCCESS
    })
}

/// Returns the KMIP 2.1 Operation Enumeration for a typed request batch item.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_operation(
    item: *mut kmipkit_client_batch_item_t,
    out_operation: *mut u32,
) -> i32 {
    ffi_entrypoint(|| {
        let operation = scalar_out!(out_operation);
        *operation = 0;
        let handle = read_handle!(item, BatchItem);
        *operation = match &handle.value {
            HandleValue::BatchItem(item) => item.operation(),
            HandleValue::BatchItemReadback(item) if item.owner_index_is_valid() => {
                item.metadata.operation
            }
            HandleValue::BatchItemReadback(_) => return record_ffi_error(ERROR_INTERNAL),
            _ => return record_ffi_error(ERROR_INVALID_INPUT),
        };
        SUCCESS
    })
}

#[cfg(feature = "coverage-c-consumer")]
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_test_client_batch_item_encoded_operation(
    inout_item: *mut *mut kmipkit_client_batch_item_t,
    out_operation: *mut u32,
) -> i32 {
    ffi_entrypoint(|| {
        if inout_item.is_null() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        // SAFETY: the caller supplies a writable in/out slot, as required by this test-only
        // consuming helper. The item is cleared as soon as ownership transfers.
        let item_slot = unsafe { &mut *inout_item };
        if item_slot.is_null() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let out = scalar_out!(out_operation);
        *out = 0;
        let item_handle = *item_slot;
        let handle = read_handle!(item_handle, BatchItem);
        if !matches!(&handle.value, HandleValue::BatchItem(_)) {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        drop(handle);
        let item = match owned_payload(item_handle, Kind::BatchItem) {
            Ok(HandleValue::BatchItem(item)) => *item,
            Ok(_) => return record_ffi_error(ERROR_INVALID_INPUT),
            Err(status) => return record_ffi_error(status),
        };
        *item_slot = ptr::null_mut();
        *out = try_ffi!(
            kmipkit_client::test_support::encoded_batch_item_operation(item)
                .map_err(|error| protocol_status(&error))
        );
        SUCCESS
    })
}

/// Creates an owned, read-only metadata view retaining the batch owner's lifetime.
///
/// The view does not copy request payload bytes or expose request payload access.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_at(
    batch: *mut kmipkit_client_batch_t,
    index: u64,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let Some(index) = usize::try_from(index).ok() else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let batch_handle = read_handle!(batch, ClientBatch);
        let HandleValue::ClientBatch(owner) = &batch_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(batch) = owner.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        let Some(batch) = batch.as_ref() else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = batch.items.get(index) else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };

        let unique_batch_item_id = if let Some(value) = item.unique_batch_item_id() {
            let mut copy = Vec::new();
            if copy.try_reserve_exact(value.len()).is_err() {
                return record_ffi_error(ERROR_RESOURCE_LIMIT);
            }
            copy.extend_from_slice(value);
            Some(copy)
        } else {
            None
        };
        let mut extensions = Vec::new();
        if extensions
            .try_reserve_exact(item.extension_count())
            .is_err()
        {
            return record_ffi_error(ERROR_RESOURCE_LIMIT);
        }
        for extension_index in 0..item.extension_count() {
            let Some(identity) = item.extension_identity_at(extension_index) else {
                return record_ffi_error(ERROR_INTERNAL);
            };
            let Some(criticality) = item.extension_criticality_indicator_at(extension_index) else {
                return record_ffi_error(ERROR_INTERNAL);
            };
            extensions.push((identity, criticality));
        }

        *out = try_ffi!(try_make_handle(
            Kind::BatchItem,
            HandleValue::BatchItemReadback(ClientBatchItemReadback {
                owner: Arc::clone(owner),
                index,
                metadata: ClientBatchItemMetadataSnapshot {
                    operation: item.operation(),
                    unique_batch_item_id,
                    extensions,
                },
            }),
        ));
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
        let Some(batch) = batch.as_mut() else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        batch.request_time_stamp = Some(time_stamp);
        SUCCESS
    })
}

/// Reads the caller-selected Request Header Time Stamp without synthesizing a value.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_request_time_stamp(
    batch: *mut kmipkit_client_batch_t,
    out_present: *mut u8,
    out_time_stamp: *mut i64,
) -> i32 {
    ffi_entrypoint(|| {
        if out_present.is_null() || out_time_stamp.is_null() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let present_start = out_present as usize;
        let time_stamp_start = out_time_stamp as usize;
        let Some(present_end) = present_start.checked_add(std::mem::size_of::<u8>()) else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(time_stamp_end) = time_stamp_start.checked_add(std::mem::size_of::<i64>()) else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        if out_time_stamp.align_offset(std::mem::align_of::<i64>()) != 0
            || (present_start < time_stamp_end && time_stamp_start < present_end)
        {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }

        let present = scalar_out!(out_present);
        let time_stamp = scalar_out!(out_time_stamp);
        *present = 0;
        *time_stamp = 0;

        let batch = read_handle!(batch, ClientBatch);
        let HandleValue::ClientBatch(batch) = &batch.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(batch) = batch.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        let Some(batch) = batch.as_ref() else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        if let Some(value) = batch.request_time_stamp {
            *present = 1;
            *time_stamp = value;
        }
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
        let Some(batch) = batch.as_mut() else {
            return record_ffi_error(ERROR_INVALID_INPUT);
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
        let Some(batch) = batch.as_mut() else {
            return record_ffi_error(ERROR_INVALID_INPUT);
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
        let Some(batch) = batch.as_mut() else {
            return record_ffi_error(ERROR_INVALID_INPUT);
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
        let value = try_ffi!(input_span(
            value,
            value_length,
            MAX_TRANSPORT_MATERIAL_BYTES
        ));
        let value = try_ffi!(std::str::from_utf8(value).map_err(|_| ERROR_INVALID_INPUT));
        let batch = read_handle!(batch, ClientBatch);
        let HandleValue::ClientBatch(batch) = &batch.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(mut batch) = batch.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        let Some(batch) = batch.as_mut() else {
            return record_ffi_error(ERROR_INVALID_INPUT);
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
        execute_client_batch(
            client,
            batch,
            limits,
            out_response,
            None,
            &RequestOptions::default(),
            None,
        )
    })
}

/// Executes one typed batch with an optional wire-encoding override.
///
/// A value of -1 uses the immutable client configuration default; 0, 1, and 2
/// select TTLV, JSON, and XML respectively.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_execute_with_options(
    client: *mut kmipkit_client_t,
    batch: *mut *mut kmipkit_client_batch_t,
    limits: *mut kmipkit_codec_limits_t,
    encoding_override: i32,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        let request_options = match encoding_override {
            -1 => RequestOptions::default(),
            0 => RequestOptions::default().with_wire_encoding(WireEncoding::Ttlv),
            1 => RequestOptions::default().with_wire_encoding(WireEncoding::Json),
            2 => RequestOptions::default().with_wire_encoding(WireEncoding::Xml),
            _ => return record_ffi_error(ERROR_INVALID_INPUT),
        };
        execute_client_batch(
            client,
            batch,
            limits,
            out_response,
            None,
            &request_options,
            None,
        )
    })
}

fn copy_async_correlation_spans(
    spans: &[crate::kmipkit_byte_span_v1_t],
) -> FfiResult<Vec<Vec<u8>>> {
    let mut values: Vec<Vec<u8>> = Vec::new();
    let mut total_bytes = 0_usize;
    let maximum = match usize::try_from(MAX_FFI_INPUT_BYTES) {
        Ok(maximum) => maximum,
        Err(_) => usize::MAX,
    };
    values
        .try_reserve_exact(spans.len())
        .map_err(|_| ERROR_RESOURCE_LIMIT)?;
    for span in spans {
        let bytes = match input_span(span.data, span.length, MAX_FFI_INPUT_BYTES) {
            Ok(bytes) => bytes,
            Err(status) => {
                for value in &mut values {
                    value.zeroize();
                }
                return Err(status);
            }
        };
        if bytes.len() > maximum - total_bytes {
            for value in &mut values {
                value.zeroize();
            }
            return Err(ERROR_RESOURCE_LIMIT);
        }
        let mut value = Vec::new();
        if value.try_reserve_exact(bytes.len()).is_err() {
            for value in &mut values {
                value.zeroize();
            }
            return Err(ERROR_RESOURCE_LIMIT);
        }
        value.extend_from_slice(bytes);
        total_bytes += bytes.len();
        values.push(value);
    }
    Ok(values)
}

/// Creates a standalone KMIP 2.1 §6.1.2 Add Attribute request.
///
/// The optional Unique Identifier record and New Attribute Item are borrowed
/// and copied; the returned model owns its fields and performs no I/O.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_add_attribute_request_create(
    unique_identifier: *const kmipkit_unique_identifier_v1_t,
    new_attribute: *mut kmipkit_ttlv_item_t,
    out_request: *mut *mut kmipkit_add_attribute_request_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_request);
        let unique_identifier = try_ffi!(copy_optional_text_identifier(unique_identifier));
        let new_attribute = try_ffi!(copy_required_item_handle(new_attribute));
        let request = AddAttributeRequest::new(unique_identifier, NewAttribute::new(new_attribute));
        *out = try_ffi!(try_make_handle(
            Kind::AddAttributeRequest,
            HandleValue::AddAttributeRequest(Box::new(request)),
        ));
        SUCCESS
    })
}

/// Returns the optional Unique Identifier as a borrowed view into the request.
/// For Text String identifiers, `text_data` remains valid only while the request
/// handle remains open; copy the bytes before releasing the request.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_add_attribute_request_unique_identifier(
    request: *mut kmipkit_add_attribute_request_t,
    out_unique_identifier: *mut kmipkit_unique_identifier_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        let request = read_handle!(request, AddAttributeRequest);
        let HandleValue::AddAttributeRequest(request) = &request.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        if let Err(status) =
            write_unique_identifier_text_v1(out_unique_identifier, request.unique_identifier())
        {
            return record_ffi_error(status);
        }
        SUCCESS
    })
}

/// Returns an owned copy of the direct New Attribute Item.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_add_attribute_request_new_attribute(
    request: *mut kmipkit_add_attribute_request_t,
    out_new_attribute: *mut *mut kmipkit_ttlv_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_new_attribute);
        let request = read_handle!(request, AddAttributeRequest);
        let HandleValue::AddAttributeRequest(request) = &request.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let item = try_ffi!(
            request
                .new_attribute()
                .item()
                .try_clone()
                .map_err(model_status)
        );
        *out = try_ffi!(try_make_handle(Kind::Item, HandleValue::Item(item),));
        SUCCESS
    })
}

/// Copies the Add Attribute Request Payload into a caller-owned TTLV Structure.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_add_attribute_request_to_ttlv_payload(
    request: *mut kmipkit_add_attribute_request_t,
    out_payload: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_payload);
        let request = read_handle!(request, AddAttributeRequest);
        let HandleValue::AddAttributeRequest(request) = &request.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let payload = try_ffi!(
            request
                .to_ttlv_payload()
                .map_err(|error| protocol_status(&error))
        );
        *out = try_ffi!(try_make_handle(
            Kind::Structure,
            HandleValue::Structure(payload),
        ));
        SUCCESS
    })
}

/// Creates an Add Attribute batch item from a standalone typed request.
///
/// The request remains valid and caller-owned after this call.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_add_attribute_from_request(
    request: *mut kmipkit_add_attribute_request_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let request = read_handle!(request, AddAttributeRequest);
        let HandleValue::AddAttributeRequest(request) = &request.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let copied = AddAttributeRequest::new(
            request.unique_identifier().map(str::to_owned),
            NewAttribute::new(try_ffi!(
                request
                    .new_attribute()
                    .item()
                    .try_clone()
                    .map_err(model_status)
            )),
        );
        *out = try_ffi!(batch_item_handle(ClientRequest::AddAttribute(copied)));
        SUCCESS
    })
}

/// Creates a standalone KMIP 2.1 §6.1.3 Adjust Attribute request.
///
/// The optional Text String Unique Identifier, versioned Attribute Reference,
/// raw Adjustment Type, and optional Value are copied and owned by the model.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_adjust_attribute_request_create(
    unique_identifier: *const kmipkit_unique_identifier_v1_t,
    attribute_reference: *const crate::kmipkit_attribute_reference_v1_t,
    adjustment_type: u32,
    adjustment_value: *mut kmipkit_ttlv_value_t,
    out_request: *mut *mut kmipkit_adjust_attribute_request_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_request);
        let unique_identifier = try_ffi!(copy_optional_text_identifier(unique_identifier));
        let attribute_reference = try_ffi!(copy_attribute_reference(attribute_reference));
        let adjustment_value = try_ffi!(copy_optional_value_handle(adjustment_value));
        let request = AdjustAttributeRequest::new(
            unique_identifier,
            attribute_reference,
            AdjustmentType::from_raw(adjustment_type),
            adjustment_value,
        );
        *out = try_ffi!(try_make_handle(
            Kind::AdjustAttributeRequest,
            HandleValue::AdjustAttributeRequest(Box::new(request)),
        ));
        SUCCESS
    })
}

/// Returns the optional Unique Identifier as a borrowed view into the request.
/// Its Text String span remains valid only while the request handle remains open;
/// copy the bytes before releasing the request.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_adjust_attribute_request_unique_identifier(
    request: *mut kmipkit_adjust_attribute_request_t,
    out_unique_identifier: *mut kmipkit_unique_identifier_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        let request = read_handle!(request, AdjustAttributeRequest);
        let HandleValue::AdjustAttributeRequest(request) = &request.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        if let Err(status) =
            write_unique_identifier_text_v1(out_unique_identifier, request.unique_identifier())
        {
            return record_ffi_error(status);
        }
        SUCCESS
    })
}

/// Returns the Attribute Reference as a borrowed view into the request.
/// Name-form text spans remain valid only while the request handle remains open;
/// copy the bytes before releasing the request.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_adjust_attribute_request_attribute_reference(
    request: *mut kmipkit_adjust_attribute_request_t,
    out_attribute_reference: *mut crate::kmipkit_attribute_reference_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        let request = read_handle!(request, AdjustAttributeRequest);
        let HandleValue::AdjustAttributeRequest(request) = &request.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        if let Err(status) = write_attribute_reference_v1(
            out_attribute_reference,
            Some(request.attribute_reference()),
        ) {
            return record_ffi_error(status);
        }
        SUCCESS
    })
}

/// Returns the raw-preserving Adjustment Type value.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_adjust_attribute_request_adjustment_type(
    request: *mut kmipkit_adjust_attribute_request_t,
    out_adjustment_type: *mut u32,
) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_adjustment_type);
        let request = read_handle!(request, AdjustAttributeRequest);
        let HandleValue::AdjustAttributeRequest(request) = &request.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = request.adjustment_type().raw();
        SUCCESS
    })
}

/// Copies the optional generic Adjustment Value into a caller-owned handle.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_adjust_attribute_request_adjustment_value(
    request: *mut kmipkit_adjust_attribute_request_t,
    out_present: *mut u8,
    out_adjustment_value: *mut *mut kmipkit_ttlv_value_t,
) -> i32 {
    ffi_entrypoint(|| {
        if out_present.is_null()
            || out_adjustment_value.is_null()
            || ffi_output_ranges_overlap(out_present, out_adjustment_value)
        {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let present = scalar_out!(out_present);
        let out = out_slot!(out_adjustment_value);
        *present = 0;
        let request = read_handle!(request, AdjustAttributeRequest);
        let HandleValue::AdjustAttributeRequest(request) = &request.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(value) = request.adjustment_value() else {
            return SUCCESS;
        };
        let value = try_ffi!(kmipkit_ttlv::try_clone_value(value).map_err(model_status));
        *out = try_ffi!(try_make_handle(Kind::Value, HandleValue::Value(value)));
        *present = 1;
        SUCCESS
    })
}

/// Copies the Adjust Attribute Request Payload into a caller-owned TTLV Structure.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_adjust_attribute_request_to_ttlv_payload(
    request: *mut kmipkit_adjust_attribute_request_t,
    out_payload: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_payload);
        let request = read_handle!(request, AdjustAttributeRequest);
        let HandleValue::AdjustAttributeRequest(request) = &request.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let payload = try_ffi!(
            request
                .to_ttlv_payload()
                .map_err(|error| protocol_status(&error))
        );
        *out = try_ffi!(try_make_handle(
            Kind::Structure,
            HandleValue::Structure(payload)
        ));
        SUCCESS
    })
}

/// Creates an Adjust Attribute batch item while leaving its typed request open.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_adjust_attribute_from_request(
    request: *mut kmipkit_adjust_attribute_request_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let request = read_handle!(request, AdjustAttributeRequest);
        let HandleValue::AdjustAttributeRequest(request) = &request.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let adjustment_value = try_ffi!(
            request
                .adjustment_value()
                .map(kmipkit_ttlv::try_clone_value)
                .transpose()
                .map_err(model_status)
        );
        let copied = AdjustAttributeRequest::new(
            request.unique_identifier().map(str::to_owned),
            request.attribute_reference().clone(),
            request.adjustment_type(),
            adjustment_value,
        );
        *out = try_ffi!(batch_item_handle(ClientRequest::AdjustAttribute(copied)));
        SUCCESS
    })
}

/// Creates a standalone KMIP 2.1 §6.1.13 Delete Attribute request.
///
/// The Unique Identifier, Current Attribute Item, and Attribute Reference are
/// independently optional; every supplied input is copied.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_delete_attribute_request_create(
    unique_identifier: *const kmipkit_unique_identifier_v1_t,
    current_attribute: *mut kmipkit_ttlv_item_t,
    attribute_reference: *const crate::kmipkit_attribute_reference_v1_t,
    out_request: *mut *mut kmipkit_delete_attribute_request_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_request);
        let unique_identifier = try_ffi!(copy_optional_text_identifier(unique_identifier));
        let current_attribute = if current_attribute.is_null() {
            None
        } else {
            Some(try_ffi!(copy_required_item_handle(current_attribute)))
        };
        let attribute_reference = if attribute_reference.is_null() {
            None
        } else {
            Some(try_ffi!(copy_attribute_reference(attribute_reference)))
        };
        let request = DeleteAttributeRequest::new(
            unique_identifier,
            current_attribute.map(CurrentAttribute::new),
            attribute_reference,
        );
        *out = try_ffi!(try_make_handle(
            Kind::DeleteAttributeRequest,
            HandleValue::DeleteAttributeRequest(Box::new(request)),
        ));
        SUCCESS
    })
}

/// Returns the optional Unique Identifier as a borrowed view into the request.
/// Its Text String span remains valid only while the request handle remains open;
/// copy the bytes before releasing the request.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_delete_attribute_request_unique_identifier(
    request: *mut kmipkit_delete_attribute_request_t,
    out_unique_identifier: *mut kmipkit_unique_identifier_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        let request = read_handle!(request, DeleteAttributeRequest);
        let HandleValue::DeleteAttributeRequest(request) = &request.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        if let Err(status) =
            write_unique_identifier_text_v1(out_unique_identifier, request.unique_identifier())
        {
            return record_ffi_error(status);
        }
        SUCCESS
    })
}

/// Copies the optional Current Attribute Item from a Delete request.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_delete_attribute_request_current_attribute(
    request: *mut kmipkit_delete_attribute_request_t,
    out_present: *mut u8,
    out_current_attribute: *mut *mut kmipkit_ttlv_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        if out_present.is_null()
            || out_current_attribute.is_null()
            || ffi_output_ranges_overlap(out_present, out_current_attribute)
        {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let present = scalar_out!(out_present);
        let out = out_slot!(out_current_attribute);
        *present = 0;
        let request = read_handle!(request, DeleteAttributeRequest);
        let HandleValue::DeleteAttributeRequest(request) = &request.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(attribute) = request.current_attribute() else {
            return SUCCESS;
        };
        let item = try_ffi!(attribute.item().try_clone().map_err(model_status));
        *out = try_ffi!(try_make_handle(Kind::Item, HandleValue::Item(item)));
        *present = 1;
        SUCCESS
    })
}

/// Returns the optional Attribute Reference as a borrowed view into the request.
/// Name-form text spans remain valid only while the request handle remains open;
/// copy the bytes before releasing the request.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_delete_attribute_request_attribute_reference(
    request: *mut kmipkit_delete_attribute_request_t,
    out_present: *mut u8,
    out_attribute_reference: *mut crate::kmipkit_attribute_reference_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        if out_present.is_null()
            || out_attribute_reference.is_null()
            || ffi_output_ranges_overlap(out_present, out_attribute_reference)
        {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let present = scalar_out!(out_present);
        *present = 0;
        if let Err(status) = write_attribute_reference_v1(out_attribute_reference, None) {
            return record_ffi_error(status);
        }
        let request = read_handle!(request, DeleteAttributeRequest);
        let HandleValue::DeleteAttributeRequest(request) = &request.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        if let Some(reference) = request.attribute_reference() {
            if let Err(status) =
                write_attribute_reference_v1(out_attribute_reference, Some(reference))
            {
                return record_ffi_error(status);
            }
            *present = 1;
        }
        SUCCESS
    })
}

/// Copies the Delete Attribute Request Payload into a caller-owned TTLV Structure.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_delete_attribute_request_to_ttlv_payload(
    request: *mut kmipkit_delete_attribute_request_t,
    out_payload: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_payload);
        let request = read_handle!(request, DeleteAttributeRequest);
        let HandleValue::DeleteAttributeRequest(request) = &request.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let payload = try_ffi!(
            request
                .to_ttlv_payload()
                .map_err(|error| protocol_status(&error))
        );
        *out = try_ffi!(try_make_handle(
            Kind::Structure,
            HandleValue::Structure(payload)
        ));
        SUCCESS
    })
}

/// Creates a Delete Attribute batch item while leaving its typed request open.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_delete_attribute_from_request(
    request: *mut kmipkit_delete_attribute_request_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let request = read_handle!(request, DeleteAttributeRequest);
        let HandleValue::DeleteAttributeRequest(request) = &request.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let current_attribute = try_ffi!(
            request
                .current_attribute()
                .map(|attribute| attribute.item().try_clone().map(CurrentAttribute::new))
                .transpose()
                .map_err(model_status)
        );
        let copied = DeleteAttributeRequest::new(
            request.unique_identifier().map(str::to_owned),
            current_attribute,
            request.attribute_reference().cloned(),
        );
        *out = try_ffi!(batch_item_handle(ClientRequest::DeleteAttribute(copied)));
        SUCCESS
    })
}

/// Creates a standalone KMIP 2.1 §6.1.34 Modify Attribute request.
///
/// Current Attribute is optional and New Attribute is required. All supplied
/// identifiers and Items are copied into the owned request model.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_modify_attribute_request_create(
    unique_identifier: *const kmipkit_unique_identifier_v1_t,
    current_attribute: *mut kmipkit_ttlv_item_t,
    new_attribute: *mut kmipkit_ttlv_item_t,
    out_request: *mut *mut kmipkit_modify_attribute_request_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_request);
        let unique_identifier = try_ffi!(copy_optional_text_identifier(unique_identifier));
        let current_attribute = if current_attribute.is_null() {
            None
        } else {
            Some(try_ffi!(copy_required_item_handle(current_attribute)))
        };
        let new_attribute = try_ffi!(copy_required_item_handle(new_attribute));
        let request = ModifyAttributeRequest::new(
            unique_identifier,
            current_attribute.map(CurrentAttribute::new),
            NewAttribute::new(new_attribute),
        );
        *out = try_ffi!(try_make_handle(
            Kind::ModifyAttributeRequest,
            HandleValue::ModifyAttributeRequest(Box::new(request)),
        ));
        SUCCESS
    })
}

/// Returns the optional Unique Identifier as a borrowed view into the request.
/// Its Text String span remains valid only while the request handle remains open;
/// copy the bytes before releasing the request.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_modify_attribute_request_unique_identifier(
    request: *mut kmipkit_modify_attribute_request_t,
    out_unique_identifier: *mut kmipkit_unique_identifier_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        let request = read_handle!(request, ModifyAttributeRequest);
        let HandleValue::ModifyAttributeRequest(request) = &request.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        if let Err(status) =
            write_unique_identifier_text_v1(out_unique_identifier, request.unique_identifier())
        {
            return record_ffi_error(status);
        }
        SUCCESS
    })
}

/// Copies the optional Current Attribute Item from a Modify request.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_modify_attribute_request_current_attribute(
    request: *mut kmipkit_modify_attribute_request_t,
    out_present: *mut u8,
    out_current_attribute: *mut *mut kmipkit_ttlv_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        if out_present.is_null()
            || out_current_attribute.is_null()
            || ffi_output_ranges_overlap(out_present, out_current_attribute)
        {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let present = scalar_out!(out_present);
        let out = out_slot!(out_current_attribute);
        *present = 0;
        let request = read_handle!(request, ModifyAttributeRequest);
        let HandleValue::ModifyAttributeRequest(request) = &request.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(attribute) = request.current_attribute() else {
            return SUCCESS;
        };
        let item = try_ffi!(attribute.item().try_clone().map_err(model_status));
        *out = try_ffi!(try_make_handle(Kind::Item, HandleValue::Item(item)));
        *present = 1;
        SUCCESS
    })
}

/// Returns an owned copy of the required direct New Attribute Item.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_modify_attribute_request_new_attribute(
    request: *mut kmipkit_modify_attribute_request_t,
    out_new_attribute: *mut *mut kmipkit_ttlv_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_new_attribute);
        let request = read_handle!(request, ModifyAttributeRequest);
        let HandleValue::ModifyAttributeRequest(request) = &request.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let item = try_ffi!(
            request
                .new_attribute()
                .item()
                .try_clone()
                .map_err(model_status)
        );
        *out = try_ffi!(try_make_handle(Kind::Item, HandleValue::Item(item)));
        SUCCESS
    })
}

/// Copies the Modify Attribute Request Payload into a caller-owned TTLV Structure.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_modify_attribute_request_to_ttlv_payload(
    request: *mut kmipkit_modify_attribute_request_t,
    out_payload: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_payload);
        let request = read_handle!(request, ModifyAttributeRequest);
        let HandleValue::ModifyAttributeRequest(request) = &request.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let payload = try_ffi!(
            request
                .to_ttlv_payload()
                .map_err(|error| protocol_status(&error))
        );
        *out = try_ffi!(try_make_handle(
            Kind::Structure,
            HandleValue::Structure(payload)
        ));
        SUCCESS
    })
}

/// Creates a Modify Attribute batch item while leaving its typed request open.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_modify_attribute_from_request(
    request: *mut kmipkit_modify_attribute_request_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let request = read_handle!(request, ModifyAttributeRequest);
        let HandleValue::ModifyAttributeRequest(request) = &request.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let current_attribute = try_ffi!(
            request
                .current_attribute()
                .map(|attribute| attribute.item().try_clone().map(CurrentAttribute::new))
                .transpose()
                .map_err(model_status)
        );
        let new_attribute = try_ffi!(
            request
                .new_attribute()
                .item()
                .try_clone()
                .map_err(model_status)
        );
        let copied = ModifyAttributeRequest::new(
            request.unique_identifier().map(str::to_owned),
            current_attribute,
            NewAttribute::new(new_attribute),
        );
        *out = try_ffi!(batch_item_handle(ClientRequest::ModifyAttribute(copied)));
        SUCCESS
    })
}

/// Creates a standalone KMIP 2.1 §6.1.51 Set Attribute request.
///
/// The optional Text String Unique Identifier and required New Attribute Item
/// are borrowed and copied into the owned model.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_set_attribute_request_create(
    unique_identifier: *const kmipkit_unique_identifier_v1_t,
    new_attribute: *mut kmipkit_ttlv_item_t,
    out_request: *mut *mut kmipkit_set_attribute_request_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_request);
        let unique_identifier = try_ffi!(copy_optional_text_identifier(unique_identifier));
        let new_attribute = try_ffi!(copy_required_item_handle(new_attribute));
        let request = SetAttributeRequest::new(unique_identifier, NewAttribute::new(new_attribute));
        *out = try_ffi!(try_make_handle(
            Kind::SetAttributeRequest,
            HandleValue::SetAttributeRequest(Box::new(request)),
        ));
        SUCCESS
    })
}

/// Returns the optional Unique Identifier as a borrowed view into the request.
/// Its Text String span remains valid only while the request handle remains open;
/// copy the bytes before releasing the request.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_set_attribute_request_unique_identifier(
    request: *mut kmipkit_set_attribute_request_t,
    out_unique_identifier: *mut kmipkit_unique_identifier_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        let request = read_handle!(request, SetAttributeRequest);
        let HandleValue::SetAttributeRequest(request) = &request.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        if let Err(status) =
            write_unique_identifier_text_v1(out_unique_identifier, request.unique_identifier())
        {
            return record_ffi_error(status);
        }
        SUCCESS
    })
}

/// Returns an owned copy of the required direct New Attribute Item.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_set_attribute_request_new_attribute(
    request: *mut kmipkit_set_attribute_request_t,
    out_new_attribute: *mut *mut kmipkit_ttlv_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_new_attribute);
        let request = read_handle!(request, SetAttributeRequest);
        let HandleValue::SetAttributeRequest(request) = &request.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let item = try_ffi!(
            request
                .new_attribute()
                .item()
                .try_clone()
                .map_err(model_status)
        );
        *out = try_ffi!(try_make_handle(Kind::Item, HandleValue::Item(item)));
        SUCCESS
    })
}

/// Copies the Set Attribute Request Payload into a caller-owned TTLV Structure.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_set_attribute_request_to_ttlv_payload(
    request: *mut kmipkit_set_attribute_request_t,
    out_payload: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_payload);
        let request = read_handle!(request, SetAttributeRequest);
        let HandleValue::SetAttributeRequest(request) = &request.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let payload = try_ffi!(
            request
                .to_ttlv_payload()
                .map_err(|error| protocol_status(&error))
        );
        *out = try_ffi!(try_make_handle(
            Kind::Structure,
            HandleValue::Structure(payload)
        ));
        SUCCESS
    })
}

/// Creates a Set Attribute batch item while leaving its typed request open.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_set_attribute_from_request(
    request: *mut kmipkit_set_attribute_request_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let request = read_handle!(request, SetAttributeRequest);
        let HandleValue::SetAttributeRequest(request) = &request.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let new_attribute = try_ffi!(
            request
                .new_attribute()
                .item()
                .try_clone()
                .map_err(model_status)
        );
        let copied = SetAttributeRequest::new(
            request.unique_identifier().map(str::to_owned),
            NewAttribute::new(new_attribute),
        );
        *out = try_ffi!(batch_item_handle(ClientRequest::SetAttribute(copied)));
        SUCCESS
    })
}

/// Creates a standalone KMIP 2.1 §6.1.21 Get Attribute List request.
/// The optional Unique Identifier input is borrowed and copied.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_get_attribute_list_request_create(
    unique_identifier: *const kmipkit_unique_identifier_v1_t,
    out_request: *mut *mut kmipkit_get_attribute_list_request_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_request);
        let identifier = try_ffi!(copy_optional_text_identifier(unique_identifier));
        let request = GetAttributeListRequest::new(identifier);
        *out = try_ffi!(try_make_handle(
            Kind::GetAttributeListRequest,
            HandleValue::GetAttributeListRequest(Box::new(request)),
        ));
        SUCCESS
    })
}

/// Returns the optional Unique Identifier as a borrowed view into the request.
/// Its Text String span remains valid only while the request handle remains open;
/// copy the bytes before releasing the request.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_get_attribute_list_request_unique_identifier(
    request: *mut kmipkit_get_attribute_list_request_t,
    out_unique_identifier: *mut kmipkit_unique_identifier_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        let handle = read_handle!(request, GetAttributeListRequest);
        let HandleValue::GetAttributeListRequest(request) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        if let Err(status) =
            write_unique_identifier_text_v1(out_unique_identifier, request.unique_identifier())
        {
            return record_ffi_error(status);
        }
        SUCCESS
    })
}

/// Copies the Get Attribute List Request Payload into a caller-owned structure.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_get_attribute_list_request_to_ttlv_payload(
    request: *mut kmipkit_get_attribute_list_request_t,
    out_payload: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_payload);
        let handle = read_handle!(request, GetAttributeListRequest);
        let HandleValue::GetAttributeListRequest(request) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let payload = try_ffi!(
            request
                .to_ttlv_payload()
                .map_err(|error| protocol_status(&error))
        );
        *out = try_ffi!(try_make_handle(
            Kind::Structure,
            HandleValue::Structure(payload),
        ));
        SUCCESS
    })
}

/// Creates an independent `ClientBatchItem` from a standalone Get Attribute List request.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_get_attribute_list_from_request(
    request: *mut kmipkit_get_attribute_list_request_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let handle = read_handle!(request, GetAttributeListRequest);
        let HandleValue::GetAttributeListRequest(request) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = try_ffi!(batch_item_handle(ClientRequest::get_attribute_list(
            request.as_ref().clone(),
        )));
        SUCCESS
    })
}

/// Creates a standalone KMIP 2.1 §6.1.20 Get Attributes request.
/// Inputs are borrowed and copied; duplicate Attribute References are rejected.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_get_attributes_request_create(
    unique_identifier: *const kmipkit_unique_identifier_v1_t,
    attribute_references: *const crate::kmipkit_attribute_reference_v1_t,
    attribute_reference_count: u64,
    out_request: *mut *mut crate::kmipkit_get_attributes_request_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_request);
        let identifier = try_ffi!(copy_optional_text_identifier(unique_identifier));
        let source_references = try_ffi!(input_array(
            attribute_references,
            attribute_reference_count,
            MAX_GET_ATTRIBUTES_REFERENCES,
        ));
        let mut references = Vec::new();
        try_ffi!(
            references
                .try_reserve_exact(source_references.len())
                .map_err(|_| ERROR_RESOURCE_LIMIT)
        );
        for reference in source_references {
            references.push(try_ffi!(copy_attribute_reference(ptr::from_ref(reference))));
        }
        let request = try_ffi!(
            GetAttributesRequest::try_new(identifier, references).map_err(|_| ERROR_INVALID_INPUT)
        );
        *out = try_ffi!(try_make_handle(
            Kind::GetAttributesRequest,
            HandleValue::GetAttributesRequest(Box::new(request)),
        ));
        SUCCESS
    })
}

/// Returns the optional Unique Identifier as a borrowed view into the request.
/// Its Text String span remains valid only while the request handle remains open;
/// copy the bytes before releasing the request.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_get_attributes_request_unique_identifier(
    request: *mut crate::kmipkit_get_attributes_request_t,
    out_unique_identifier: *mut kmipkit_unique_identifier_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        let handle = read_handle!(request, GetAttributesRequest);
        let HandleValue::GetAttributesRequest(request) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        if let Err(status) =
            write_unique_identifier_text_v1(out_unique_identifier, request.unique_identifier())
        {
            return record_ffi_error(status);
        }
        SUCCESS
    })
}

/// Returns the number of ordered Attribute References in the request.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_get_attributes_request_attribute_reference_count(
    request: *mut crate::kmipkit_get_attributes_request_t,
    out_count: *mut u64,
) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_count);
        let handle = read_handle!(request, GetAttributesRequest);
        let HandleValue::GetAttributesRequest(request) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = try_ffi!(
            u64::try_from(request.attribute_references().len()).map_err(|_| ERROR_RESOURCE_LIMIT)
        );
        SUCCESS
    })
}

/// Returns one Attribute Reference as a borrowed view into the request.
/// Name-form text spans remain valid only while the request handle remains open;
/// copy the bytes before releasing the request.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_get_attributes_request_attribute_reference_at(
    request: *mut crate::kmipkit_get_attributes_request_t,
    index: u64,
    out_reference: *mut crate::kmipkit_attribute_reference_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        let handle = read_handle!(request, GetAttributesRequest);
        let HandleValue::GetAttributesRequest(request) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(index) = usize::try_from(index) else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(reference) = request.attribute_references().get(index) else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        if let Err(status) = write_attribute_reference_v1(out_reference, Some(reference)) {
            return record_ffi_error(status);
        }
        SUCCESS
    })
}

/// Copies the Get Attributes Request Payload into a caller-owned structure.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_get_attributes_request_to_ttlv_payload(
    request: *mut crate::kmipkit_get_attributes_request_t,
    out_payload: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_payload);
        let handle = read_handle!(request, GetAttributesRequest);
        let HandleValue::GetAttributesRequest(request) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let payload = try_ffi!(
            request
                .to_ttlv_payload()
                .map_err(|error| protocol_status(&error))
        );
        *out = try_ffi!(try_make_handle(
            Kind::Structure,
            HandleValue::Structure(payload),
        ));
        SUCCESS
    })
}

/// Creates an independent `ClientBatchItem` from a standalone Get Attributes request.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_get_attributes_from_request(
    request: *mut crate::kmipkit_get_attributes_request_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let handle = read_handle!(request, GetAttributesRequest);
        let HandleValue::GetAttributesRequest(request) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = try_ffi!(batch_item_handle(ClientRequest::get_attributes(
            request.as_ref().clone(),
        )));
        SUCCESS
    })
}

/// Creates a local KMIP 2.1 Cancel request model without performing I/O.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_cancel_request_create(
    correlation_value: *const u8,
    correlation_value_length: u64,
    out_request: *mut *mut kmipkit_cancel_request_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_request);
        let correlation = try_ffi!(input_span(
            correlation_value,
            correlation_value_length,
            MAX_FFI_INPUT_BYTES
        ));
        *out = try_ffi!(try_make_handle(
            Kind::CancelRequest,
            HandleValue::CancelRequest(Box::new(CancelRequest::new(correlation))),
        ));
        SUCCESS
    })
}

/// Borrows the exact Cancel correlation bytes for the lifetime of its request handle.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_cancel_request_asynchronous_correlation_value(
    request: *mut kmipkit_cancel_request_t,
    out_value: *mut crate::kmipkit_byte_span_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_value);
        let request = read_handle!(request, CancelRequest);
        let HandleValue::CancelRequest(request) = &request.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let bytes = request.asynchronous_correlation_value();
        *out = crate::kmipkit_byte_span_v1_t {
            data: if bytes.is_empty() {
                ptr::null()
            } else {
                bytes.as_ptr()
            },
            length: bytes.len() as u64,
        };
        SUCCESS
    })
}

/// Copies the Cancel request payload into an owned generic TTLV structure.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_cancel_request_to_ttlv_payload(
    request: *mut kmipkit_cancel_request_t,
    out_payload: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_payload);
        let request = read_handle!(request, CancelRequest);
        let HandleValue::CancelRequest(request) = &request.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let payload = try_ffi!(
            request
                .to_ttlv_payload()
                .map_err(|error| protocol_status(&error))
        );
        *out = try_ffi!(try_make_handle(
            Kind::Structure,
            HandleValue::Structure(payload),
        ));
        SUCCESS
    })
}

/// Creates a local KMIP 2.1 Poll request model without performing I/O.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_poll_request_create(
    correlation_value: *const u8,
    correlation_value_length: u64,
    out_request: *mut *mut kmipkit_poll_request_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_request);
        let correlation = try_ffi!(input_span(
            correlation_value,
            correlation_value_length,
            MAX_FFI_INPUT_BYTES
        ));
        *out = try_ffi!(try_make_handle(
            Kind::PollRequest,
            HandleValue::PollRequest(Box::new(PollRequest::new(correlation))),
        ));
        SUCCESS
    })
}

/// Borrows the exact Poll correlation bytes for the lifetime of its request handle.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_poll_request_asynchronous_correlation_value(
    request: *mut kmipkit_poll_request_t,
    out_value: *mut crate::kmipkit_byte_span_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_value);
        let request = read_handle!(request, PollRequest);
        let HandleValue::PollRequest(request) = &request.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let bytes = request.asynchronous_correlation_value();
        *out = crate::kmipkit_byte_span_v1_t {
            data: if bytes.is_empty() {
                ptr::null()
            } else {
                bytes.as_ptr()
            },
            length: bytes.len() as u64,
        };
        SUCCESS
    })
}

/// Copies the Poll request payload into an owned generic TTLV structure.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_poll_request_to_ttlv_payload(
    request: *mut kmipkit_poll_request_t,
    out_payload: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_payload);
        let request = read_handle!(request, PollRequest);
        let HandleValue::PollRequest(request) = &request.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let payload = try_ffi!(
            request
                .to_ttlv_payload()
                .map_err(|error| protocol_status(&error))
        );
        *out = try_ffi!(try_make_handle(
            Kind::Structure,
            HandleValue::Structure(payload),
        ));
        SUCCESS
    })
}

/// Creates a local KMIP 2.1 Process request model without performing I/O.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_process_request_create(
    correlation_value: *const u8,
    correlation_value_length: u64,
    out_request: *mut *mut kmipkit_process_request_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_request);
        let correlation = try_ffi!(input_span(
            correlation_value,
            correlation_value_length,
            MAX_FFI_INPUT_BYTES
        ));
        *out = try_ffi!(try_make_handle(
            Kind::ProcessRequest,
            HandleValue::ProcessRequest(Box::new(ProcessRequest::new(correlation))),
        ));
        SUCCESS
    })
}

/// Copies the Process request payload into an owned generic TTLV structure.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_process_request_to_ttlv_payload(
    request: *mut kmipkit_process_request_t,
    out_payload: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_payload);
        let request = read_handle!(request, ProcessRequest);
        let HandleValue::ProcessRequest(request) = &request.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let payload = try_ffi!(
            request
                .to_ttlv_payload()
                .map_err(|error| protocol_status(&error))
        );
        *out = try_ffi!(try_make_handle(
            Kind::Structure,
            HandleValue::Structure(payload),
        ));
        SUCCESS
    })
}

/// Creates an unfiltered local Query Asynchronous Requests model.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_query_async_requests_request_create(
    out_request: *mut *mut kmipkit_query_async_requests_request_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_request);
        *out = try_ffi!(try_make_handle(
            Kind::QueryAsyncRequestsRequest,
            HandleValue::QueryAsyncRequestsRequest(Box::new(QueryAsyncRequestsRequest::new())),
        ));
        SUCCESS
    })
}

/// Returns a query model with the supplied correlation filter and preserves the input handle.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_query_async_requests_request_with_correlation_values(
    request: *mut kmipkit_query_async_requests_request_t,
    values: *const crate::kmipkit_byte_span_v1_t,
    value_count: u64,
    out_request: *mut *mut kmipkit_query_async_requests_request_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_request);
        let request = read_handle!(request, QueryAsyncRequestsRequest);
        let spans = try_ffi!(input_array(values, value_count, 100_000));
        let values = try_ffi!(copy_async_correlation_spans(spans));
        let HandleValue::QueryAsyncRequestsRequest(request) = &request.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let request = request.as_ref().clone().with_correlation_values(values);
        *out = try_ffi!(try_make_handle(
            Kind::QueryAsyncRequestsRequest,
            HandleValue::QueryAsyncRequestsRequest(Box::new(request)),
        ));
        SUCCESS
    })
}

/// Returns a query model with the supplied operation filter and preserves the input handle.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_query_async_requests_request_with_operations(
    request: *mut kmipkit_query_async_requests_request_t,
    operations: *const u32,
    operation_count: u64,
    out_request: *mut *mut kmipkit_query_async_requests_request_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_request);
        let request = read_handle!(request, QueryAsyncRequestsRequest);
        let operations = try_ffi!(input_array(operations, operation_count, 100_000));
        let mut operation_values = Vec::new();
        if operation_values
            .try_reserve_exact(operations.len())
            .is_err()
        {
            return record_ffi_error(ERROR_RESOURCE_LIMIT);
        }
        operation_values.extend_from_slice(operations);
        let HandleValue::QueryAsyncRequestsRequest(request) = &request.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let request = request.as_ref().clone().with_operations(operation_values);
        *out = try_ffi!(try_make_handle(
            Kind::QueryAsyncRequestsRequest,
            HandleValue::QueryAsyncRequestsRequest(Box::new(request)),
        ));
        SUCCESS
    })
}

/// Copies the query filters into an owned generic TTLV request payload.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_query_async_requests_request_to_ttlv_payload(
    request: *mut kmipkit_query_async_requests_request_t,
    out_payload: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_payload);
        let request = read_handle!(request, QueryAsyncRequestsRequest);
        let HandleValue::QueryAsyncRequestsRequest(request) = &request.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let payload = try_ffi!(
            request
                .to_ttlv_payload()
                .map_err(|error| protocol_status(&error))
        );
        *out = try_ffi!(try_make_handle(
            Kind::Structure,
            HandleValue::Structure(payload),
        ));
        SUCCESS
    })
}

/// Creates an independent typed Cancel batch item from a borrowed request model.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_cancel_from_request(
    request: *mut crate::kmipkit_cancel_request_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let request = read_handle!(request, CancelRequest);
        let HandleValue::CancelRequest(request) = &request.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = try_ffi!(batch_item_handle(ClientRequest::cancel(
            request.as_ref().clone(),
        )));
        SUCCESS
    })
}

/// Creates an independent typed Poll batch item from a borrowed request model.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_poll_from_request(
    request: *mut crate::kmipkit_poll_request_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let request = read_handle!(request, PollRequest);
        let HandleValue::PollRequest(request) = &request.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = try_ffi!(batch_item_handle(ClientRequest::poll(
            request.as_ref().clone(),
        )));
        SUCCESS
    })
}

/// Creates an independent typed Process batch item from a borrowed request model.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_process_from_request(
    request: *mut crate::kmipkit_process_request_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let request = read_handle!(request, ProcessRequest);
        let HandleValue::ProcessRequest(request) = &request.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = try_ffi!(batch_item_handle(ClientRequest::process(
            request.as_ref().clone(),
        )));
        SUCCESS
    })
}

/// Creates an independent typed Query Asynchronous Requests item from a borrowed model.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_query_async_requests_from_request(
    request: *mut crate::kmipkit_query_async_requests_request_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let request = read_handle!(request, QueryAsyncRequestsRequest);
        let HandleValue::QueryAsyncRequestsRequest(request) = &request.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = try_ffi!(batch_item_handle(ClientRequest::query_async_requests(
            request.as_ref().clone(),
        )));
        SUCCESS
    })
}

/// Creates a typed KMIP 3.0 Create Group batch item from a borrowed `AttributeSet`.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_create_group_from_attributes(
    attributes: *mut crate::kmipkit_attribute_set_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let attributes = try_ffi!(copy_attribute_set_model(attributes));
        let request = CreateGroupRequest::new(attributes);
        *out = try_ffi!(batch_item_handle(ClientRequest::create_group(request)));
        SUCCESS
    })
}

/// Creates a typed KMIP 3.0 Create User batch item from a borrowed `AttributeSet`.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_create_user_from_attributes(
    attributes: *mut crate::kmipkit_attribute_set_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let attributes = try_ffi!(copy_attribute_set_model(attributes));
        let request = CreateUserRequest::new(attributes);
        *out = try_ffi!(batch_item_handle(ClientRequest::create_user(request)));
        SUCCESS
    })
}

/// Creates a typed KMIP 3.0 Obliterate item from a copied Unique Identifier.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_obliterate_from_identifier(
    unique_identifier: *const kmipkit_unique_identifier_v1_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let identifier =
            try_ffi!(copy_unique_identifier(unique_identifier)).ok_or(ERROR_INVALID_INPUT);
        let identifier = try_ffi!(identifier);
        let request = ObliterateRequest::new(identifier);
        *out = try_ffi!(batch_item_handle(ClientRequest::obliterate(request)));
        SUCCESS
    })
}

/// Creates a typed KMIP 3.0 Deactivate item from semantic fields and copied inputs.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_deactivate_from_fields(
    unique_identifier: *const kmipkit_unique_identifier_v1_t,
    deactivation_reason: *mut kmipkit_ttlv_structure_t,
    has_deactivation_date: u8,
    deactivation_date: i64,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        if has_deactivation_date > 1 || (has_deactivation_date == 0 && deactivation_date != 0) {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let identifier =
            try_ffi!(copy_unique_identifier(unique_identifier)).ok_or(ERROR_INVALID_INPUT);
        let identifier = try_ffi!(identifier);
        let reason = try_ffi!(copy_structure_handle(deactivation_reason, false));
        let mut request = DeactivateRequest::new(identifier);
        if let Some(reason) = reason {
            request = request.with_deactivation_reason(reason);
        }
        if has_deactivation_date == 1 {
            request = request.with_deactivation_date(deactivation_date);
        }
        *out = try_ffi!(batch_item_handle(ClientRequest::deactivate(request)));
        SUCCESS
    })
}

/// Creates a typed KMIP 3.0 Create Credential item from its explicit fields.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_create_credential_from_fields(
    credential_type_raw: u32,
    object_type_raw: u32,
    attributes: *mut crate::kmipkit_attribute_set_t,
    object: *mut crate::kmipkit_ttlv_item_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let attributes = try_ffi!(copy_attribute_set_model(attributes));
        let object = try_ffi!(copy_required_item_handle(object));
        let object = try_ffi!(create_credential_object_from_item(object_type_raw, object));
        let request = CreateCredentialRequest::new(
            CredentialType::from_raw(credential_type_raw),
            attributes,
            object,
        );
        *out = try_ffi!(batch_item_handle(ClientRequest::create_credential(request)));
        SUCCESS
    })
}

/// Creates a typed KMIP 3.0 Encapsulate item from its semantic fields.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_encapsulate_from_fields(
    unique_identifier: *const kmipkit_unique_identifier_v1_t,
    cryptographic_parameters: *mut crate::kmipkit_cryptographic_parameters_t,
    attributes: *mut crate::kmipkit_attribute_set_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let identifier =
            try_ffi!(copy_unique_identifier(unique_identifier)).ok_or(ERROR_INVALID_INPUT);
        let identifier = try_ffi!(identifier);
        let parameters = try_ffi!(copy_optional_cryptographic_parameters_structure(
            cryptographic_parameters,
        ));
        let attributes = try_ffi!(copy_optional_attribute_set_model(attributes));
        let mut request = EncapsulateRequest::new(identifier);
        if let Some(parameters) = parameters {
            request = request.with_cryptographic_parameters(parameters);
        }
        if let Some(attributes) = attributes {
            request = request.with_attributes(attributes);
        }
        *out = try_ffi!(batch_item_handle(ClientRequest::encapsulate(request)));
        SUCCESS
    })
}

/// Creates a typed KMIP 3.0 Decapsulate item from its semantic fields.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_decapsulate_from_fields(
    unique_identifier: *const kmipkit_unique_identifier_v1_t,
    ciphertext: *const u8,
    ciphertext_length: u64,
    cryptographic_parameters: *mut crate::kmipkit_cryptographic_parameters_t,
    attributes: *mut crate::kmipkit_attribute_set_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let identifier =
            try_ffi!(copy_unique_identifier(unique_identifier)).ok_or(ERROR_INVALID_INPUT);
        let identifier = try_ffi!(identifier);
        let ciphertext = try_ffi!(copy_transport_span(
            ciphertext,
            ciphertext_length,
            MAX_FFI_INPUT_BYTES,
        ));
        let parameters = try_ffi!(copy_optional_cryptographic_parameters_structure(
            cryptographic_parameters,
        ));
        let attributes = try_ffi!(copy_optional_attribute_set_model(attributes));
        let mut request = DecapsulateRequest::new(identifier, ciphertext);
        if let Some(parameters) = parameters {
            request = request.with_cryptographic_parameters(parameters);
        }
        if let Some(attributes) = attributes {
            request = request.with_attributes(attributes);
        }
        *out = try_ffi!(batch_item_handle(ClientRequest::decapsulate(request)));
        SUCCESS
    })
}

/// Creates a typed KMIP 2.1 Set Endpoint Role batch item.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_set_endpoint_role(
    endpoint_role: u32,
    vendor_identification: *const u8,
    vendor_identification_length: u64,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let role = kmipkit_protocol::EndpointRole::from_raw(endpoint_role);
        let mut request = kmipkit_protocol::SetEndpointRoleRequest::new(role);
        if !vendor_identification.is_null() && vendor_identification_length > 0 {
            let span = try_ffi!(input_span(
                vendor_identification,
                vendor_identification_length,
                MAX_FFI_INPUT_BYTES,
            ));
            let text = match std::str::from_utf8(span) {
                Ok(t) => t,
                Err(_) => return record_ffi_error(ERROR_INVALID_INPUT),
            };
            request = request.with_vendor_identification(text);
        } else if !vendor_identification.is_null() || vendor_identification_length != 0 {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        *out = try_ffi!(batch_item_handle(ClientRequest::set_endpoint_role(request)));
        SUCCESS
    })
}

/// Executes exactly one KMIP 2.1 Poll request with the caller's opaque correlation bytes.
///
/// The outcome is caller-owned and must be released with
/// `kmipkit_client_async_operation_outcome_release`. This function performs one exchange and
/// never issues another Poll or retries the request.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_execute_poll(
    client: *mut kmipkit_client_t,
    correlation_value: *const u8,
    correlation_value_length: u64,
    limits: *mut kmipkit_codec_limits_t,
    out_outcome: *mut *mut kmipkit_client_async_operation_outcome_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_outcome);
        let correlation_value = try_ffi!(input_span(
            correlation_value,
            correlation_value_length,
            MAX_FFI_INPUT_BYTES
        ));
        execute_async_client_operation(client, limits, out, |client, limits| {
            client.execute_poll(
                kmipkit_protocol::PollRequest::new(correlation_value),
                limits,
            )
        })
    })
}

/// Executes exactly one KMIP 2.1 Cancel request with the caller's opaque correlation bytes.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_execute_cancel(
    client: *mut kmipkit_client_t,
    correlation_value: *const u8,
    correlation_value_length: u64,
    limits: *mut kmipkit_codec_limits_t,
    out_outcome: *mut *mut kmipkit_client_async_operation_outcome_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_outcome);
        let correlation_value = try_ffi!(input_span(
            correlation_value,
            correlation_value_length,
            MAX_FFI_INPUT_BYTES
        ));
        execute_async_client_operation(client, limits, out, |client, limits| {
            client.execute_cancel(
                kmipkit_protocol::CancelRequest::new(correlation_value),
                limits,
            )
        })
    })
}

/// Executes exactly one KMIP 2.1 Process request with caller-selected asynchronous behavior.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_execute_process(
    client: *mut kmipkit_client_t,
    correlation_value: *const u8,
    correlation_value_length: u64,
    has_asynchronous_indicator: u8,
    asynchronous_indicator: u32,
    limits: *mut kmipkit_codec_limits_t,
    out_outcome: *mut *mut kmipkit_client_async_operation_outcome_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_outcome);
        let correlation_value = try_ffi!(input_span(
            correlation_value,
            correlation_value_length,
            MAX_FFI_INPUT_BYTES
        ));
        if has_asynchronous_indicator > 1 {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let indicator = (has_asynchronous_indicator == 1).then_some(asynchronous_indicator);
        execute_async_client_operation(client, limits, out, |client, limits| {
            client.execute_process(
                kmipkit_protocol::ProcessRequest::new(correlation_value),
                indicator,
                limits,
            )
        })
    })
}

/// Executes exactly one KMIP 2.1 Query Asynchronous Requests operation.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_execute_query_async_requests(
    client: *mut kmipkit_client_t,
    has_correlation_values: u8,
    correlation_values: *const crate::kmipkit_byte_span_v1_t,
    correlation_value_count: u64,
    has_operations: u8,
    operations: *const u32,
    operation_count: u64,
    has_asynchronous_indicator: u8,
    asynchronous_indicator: u32,
    limits: *mut kmipkit_codec_limits_t,
    out_outcome: *mut *mut kmipkit_client_async_operation_outcome_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_outcome);
        if has_correlation_values > 1 || has_operations > 1 || has_asynchronous_indicator > 1 {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let correlation_spans = try_ffi!(input_array(
            correlation_values,
            correlation_value_count,
            100_000,
        ));
        let operation_values = try_ffi!(input_array(operations, operation_count, 100_000));
        let mut request = kmipkit_protocol::QueryAsyncRequestsRequest::new();
        if has_correlation_values == 0 && correlation_value_count != 0 {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        if has_operations == 0 && operation_count != 0 {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        if has_correlation_values == 1 {
            let values = correlation_spans
                .iter()
                .map(|span| input_span(span.data, span.length, MAX_FFI_INPUT_BYTES))
                .collect::<FfiResult<Vec<_>>>();
            let values = try_ffi!(values);
            request = request.with_correlation_values(values.into_iter().map(<[u8]>::to_vec));
        }
        if has_operations == 1 {
            request = request.with_operations(operation_values.iter().copied());
        }
        let indicator = (has_asynchronous_indicator == 1).then_some(asynchronous_indicator);
        execute_async_client_operation(client, limits, out, |client, limits| {
            client.execute_query_async_requests(request, indicator, limits)
        })
    })
}

/// Executes exactly one Poll request with per-exchange timeout overrides.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_execute_poll_with_options_v1(
    client: *mut kmipkit_client_t,
    correlation_value: *const u8,
    correlation_value_length: u64,
    limits: *mut kmipkit_codec_limits_t,
    options: *const kmipkit_request_options_v1_t,
    out_outcome: *mut *mut kmipkit_client_async_operation_outcome_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_outcome);
        let correlation_value = try_ffi!(input_span(
            correlation_value,
            correlation_value_length,
            MAX_FFI_INPUT_BYTES
        ));
        let request_options = try_ffi!(request_options_from_pointer_v1(options));
        execute_async_client_operation(client, limits, out, |client, limits| {
            client.execute_poll_with_options(
                kmipkit_protocol::PollRequest::new(correlation_value),
                limits,
                &request_options,
            )
        })
    })
}

/// Executes exactly one Cancel request with per-exchange timeout overrides.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_execute_cancel_with_options_v1(
    client: *mut kmipkit_client_t,
    correlation_value: *const u8,
    correlation_value_length: u64,
    limits: *mut kmipkit_codec_limits_t,
    options: *const kmipkit_request_options_v1_t,
    out_outcome: *mut *mut kmipkit_client_async_operation_outcome_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_outcome);
        let correlation_value = try_ffi!(input_span(
            correlation_value,
            correlation_value_length,
            MAX_FFI_INPUT_BYTES
        ));
        let request_options = try_ffi!(request_options_from_pointer_v1(options));
        execute_async_client_operation(client, limits, out, |client, limits| {
            client.execute_cancel_with_options(
                kmipkit_protocol::CancelRequest::new(correlation_value),
                limits,
                &request_options,
            )
        })
    })
}

/// Executes exactly one Process request with per-exchange timeout overrides.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_execute_process_with_options_v1(
    client: *mut kmipkit_client_t,
    correlation_value: *const u8,
    correlation_value_length: u64,
    has_asynchronous_indicator: u8,
    asynchronous_indicator: u32,
    limits: *mut kmipkit_codec_limits_t,
    options: *const kmipkit_request_options_v1_t,
    out_outcome: *mut *mut kmipkit_client_async_operation_outcome_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_outcome);
        let correlation_value = try_ffi!(input_span(
            correlation_value,
            correlation_value_length,
            MAX_FFI_INPUT_BYTES
        ));
        if has_asynchronous_indicator > 1 {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let indicator = (has_asynchronous_indicator == 1).then_some(asynchronous_indicator);
        let request_options = try_ffi!(request_options_from_pointer_v1(options));
        execute_async_client_operation(client, limits, out, |client, limits| {
            client.execute_process_with_options(
                kmipkit_protocol::ProcessRequest::new(correlation_value),
                indicator,
                limits,
                &request_options,
            )
        })
    })
}

/// Executes exactly one Query Asynchronous Requests operation with timeout overrides.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_execute_query_async_requests_with_options_v1(
    client: *mut kmipkit_client_t,
    has_correlation_values: u8,
    correlation_values: *const crate::kmipkit_byte_span_v1_t,
    correlation_value_count: u64,
    has_operations: u8,
    operations: *const u32,
    operation_count: u64,
    has_asynchronous_indicator: u8,
    asynchronous_indicator: u32,
    limits: *mut kmipkit_codec_limits_t,
    options: *const kmipkit_request_options_v1_t,
    out_outcome: *mut *mut kmipkit_client_async_operation_outcome_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_outcome);
        if has_correlation_values > 1 || has_operations > 1 || has_asynchronous_indicator > 1 {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let correlation_spans = try_ffi!(input_array(
            correlation_values,
            correlation_value_count,
            100_000,
        ));
        let operation_values = try_ffi!(input_array(operations, operation_count, 100_000));
        let mut request = kmipkit_protocol::QueryAsyncRequestsRequest::new();
        if has_correlation_values == 0 && correlation_value_count != 0 {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        if has_operations == 0 && operation_count != 0 {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        if has_correlation_values == 1 {
            let values = correlation_spans
                .iter()
                .map(|span| input_span(span.data, span.length, MAX_FFI_INPUT_BYTES))
                .collect::<FfiResult<Vec<_>>>();
            let values = try_ffi!(values);
            request = request.with_correlation_values(values.into_iter().map(<[u8]>::to_vec));
        }
        if has_operations == 1 {
            request = request.with_operations(operation_values.iter().copied());
        }
        let indicator = (has_asynchronous_indicator == 1).then_some(asynchronous_indicator);
        let request_options = try_ffi!(request_options_from_pointer_v1(options));
        execute_async_client_operation(client, limits, out, |client, limits| {
            client.execute_query_async_requests_with_options(
                request,
                indicator,
                limits,
                &request_options,
            )
        })
    })
}

fn execute_async_client_operation(
    client_pointer: *mut kmipkit_client_t,
    limits_pointer: *mut kmipkit_codec_limits_t,
    out: &mut *mut kmipkit_client_async_operation_outcome_t,
    execute: impl FnOnce(
        &mut Client,
        &CodecLimits,
    ) -> Result<kmipkit_client::ClientOperationOutcome, ClientError>,
) -> i32 {
    let client_handle = read_handle!(client_pointer, Client);
    let HandleValue::Client(client) = &client_handle.value else {
        return record_ffi_error(ERROR_INVALID_INPUT);
    };
    let limits_handle = read_handle!(limits_pointer, CodecLimits);
    let HandleValue::CodecLimits(limits) = &limits_handle.value else {
        return record_ffi_error(ERROR_INVALID_INPUT);
    };
    let Ok(mut client) = client.lock() else {
        return record_ffi_error(ERROR_INTERNAL);
    };
    match execute(&mut client, limits) {
        Ok(outcome) => {
            *out = try_ffi!(try_make_handle(
                Kind::ClientAsyncOperationOutcome,
                HandleValue::ClientAsyncOperationOutcome(Mutex::new(outcome)),
            ));
            SUCCESS
        }
        Err(error) => record_client_error(&error),
    }
}

/// Releases one caller-owned asynchronous operation outcome.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_async_operation_outcome_release(
    outcome: *mut kmipkit_client_async_operation_outcome_t,
) {
    ffi_entrypoint(|| release(outcome, Kind::ClientAsyncOperationOutcome));
}

/// Reads common KMIP result metadata; result-message bytes borrow from `outcome`.
///
/// # Safety
///
/// `outcome` must be null or a live asynchronous-outcome handle. `out_metadata`
/// must be aligned and writable for a complete v1 metadata record and must not
/// overlap the handle storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kmipkit_client_async_operation_outcome_metadata_v1(
    outcome: *mut kmipkit_client_async_operation_outcome_t,
    out_metadata: *mut kmipkit_client_async_operation_metadata_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        if out_metadata.is_null() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let handle = read_handle!(outcome, ClientAsyncOperationOutcome);
        let HandleValue::ClientAsyncOperationOutcome(outcome) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(outcome) = outcome.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        let result = outcome.result();
        let message = result
            .message()
            .map(kmipkit_protocol::ResultMessage::as_bytes);
        let cancellation_result = outcome.cancellation_result();
        let Ok(struct_size) = u32::try_from(std::mem::size_of::<
            kmipkit_client_async_operation_metadata_v1_t,
        >()) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        // SAFETY: the caller provides a writable pointer to the complete v1 record.
        unsafe {
            *out_metadata = kmipkit_client_async_operation_metadata_v1_t {
                struct_size,
                abi_version: 1,
                operation: outcome.operation().raw(),
                outcome_kind: if outcome.is_pending() {
                    BATCH_OUTCOME_PENDING
                } else {
                    BATCH_OUTCOME_COMPLETED
                },
                result_status: result.status().raw(),
                has_result_reason: u32::from(result.reason().is_some()),
                result_reason: result
                    .reason()
                    .map_or(0, kmipkit_protocol::ResultReason::raw),
                has_cancellation_result: u32::from(cancellation_result.is_some()),
                cancellation_result: cancellation_result
                    .map_or(0, kmipkit_protocol::CancellationResult::raw),
                has_result_message: u32::from(message.is_some()),
                reserved: 0,
                result_message_data: message.map_or(ptr::null(), <[u8]>::as_ptr),
                result_message_length: message
                    .map_or(0, |value| u64::try_from(value.len()).unwrap_or(u64::MAX)),
            };
        }
        SUCCESS
    })
}

fn async_response_v1_struct_size() -> FfiResult<u32> {
    u32::try_from(std::mem::size_of::<
        kmipkit_client_async_operation_response_v1_t,
    >())
    .map_err(|_| ERROR_INTERNAL)
}

fn async_response_correlation_span(
    value: Option<(*const u8, usize)>,
) -> FfiResult<(u32, kmipkit_byte_span_v1_t)> {
    value.map_or(
        Ok((0, kmipkit_byte_span_v1_t::default())),
        |(data, length)| {
            Ok((
                1,
                kmipkit_byte_span_v1_t {
                    data: if length == 0 { ptr::null() } else { data },
                    length: u64::try_from(length).map_err(|_| ERROR_RESOURCE_LIMIT)?,
                },
            ))
        },
    )
}

fn cancel_response_echo_is_exposed(result_status: u32, has_cancellation_result: bool) -> bool {
    result_status == 0 && has_cancellation_result
}

/// Initializes the two output slots used by the typed asynchronous response exports.
///
/// # Safety
///
/// Both output pointers must be aligned and writable for their declared types. They must not
/// overlap each other or the asynchronous outcome handle storage.
unsafe fn async_response_v1_output(
    out_present: *mut u8,
    out_response: *mut kmipkit_client_async_operation_response_v1_t,
) -> FfiResult<()> {
    if out_present.is_null()
        || out_response.is_null()
        || ffi_output_ranges_overlap(out_present, out_response)
    {
        return Err(ERROR_INVALID_INPUT);
    }
    // SAFETY: the caller guarantees aligned writable storage; the checks above reject null and
    // overlapping output pointers before either pointer is dereferenced.
    unsafe {
        *out_present = 0;
        *out_response = kmipkit_client_async_operation_response_v1_t::default();
    }
    Ok(())
}

/// Reads a typed Cancel response view with a correlation span borrowed from the outcome.
///
/// # Safety
///
/// `outcome_pointer` must be null or a live asynchronous-outcome handle. `out_present` and
/// `out_response` must point to aligned, writable storage of their respective types and must not
/// overlap each other or the outcome handle storage. Any returned correlation span borrows from
/// the outcome and must be copied or discarded before the handle is released.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kmipkit_client_async_operation_outcome_cancel_response_v1(
    outcome_pointer: *mut kmipkit_client_async_operation_outcome_t,
    out_present: *mut u8,
    out_response: *mut kmipkit_client_async_operation_response_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        // SAFETY: the export's caller must satisfy the documented pointer preconditions.
        let output_initialization = unsafe { async_response_v1_output(out_present, out_response) };
        try_ffi!(output_initialization);
        let handle = read_handle!(outcome_pointer, ClientAsyncOperationOutcome);
        let HandleValue::ClientAsyncOperationOutcome(outcome) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(outcome) = outcome.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        let Some(response) = outcome.cancel_response() else {
            return SUCCESS;
        };
        let expose_echo = cancel_response_echo_is_exposed(
            response.result().status().raw(),
            response.cancellation_result().is_some(),
        );
        let (has_correlation, correlation) = if expose_echo {
            try_ffi!(async_response_correlation_span(
                response.with_asynchronous_correlation_value(|bytes| (bytes.as_ptr(), bytes.len())),
            ))
        } else {
            (0, kmipkit_byte_span_v1_t::default())
        };
        let output = kmipkit_client_async_operation_response_v1_t {
            struct_size: try_ffi!(async_response_v1_struct_size()),
            abi_version: 1,
            operation: outcome.operation().raw(),
            result_status: response.result().status().raw(),
            is_pending: 0,
            has_cancellation_result: u32::from(response.cancellation_result().is_some()),
            cancellation_result: response
                .cancellation_result()
                .map_or(0, kmipkit_protocol::CancellationResult::raw),
            has_asynchronous_correlation_value: has_correlation,
            asynchronous_correlation_value: correlation,
            has_response_payload: u32::from(outcome.raw().is_some()),
            reserved: 0,
            response_payload_member_count: 0,
        };
        // SAFETY: the caller guarantees valid output storage; the helper checked null and overlap.
        unsafe {
            *out_response = output;
            *out_present = 1;
        }
        SUCCESS
    })
}

/// Reads a typed Poll response view with a correlation span borrowed from the outcome.
///
/// # Safety
///
/// `outcome_pointer` must be null or a live asynchronous-outcome handle. `out_present` and
/// `out_response` must point to aligned, writable storage of their respective types and must not
/// overlap each other or the outcome handle storage. Any returned correlation span borrows from
/// the outcome and must be copied or discarded before the handle is released.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kmipkit_client_async_operation_outcome_poll_response_v1(
    outcome_pointer: *mut kmipkit_client_async_operation_outcome_t,
    out_present: *mut u8,
    out_response: *mut kmipkit_client_async_operation_response_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        // SAFETY: the export's caller must satisfy the documented pointer preconditions.
        let output_initialization = unsafe { async_response_v1_output(out_present, out_response) };
        try_ffi!(output_initialization);
        let handle = read_handle!(outcome_pointer, ClientAsyncOperationOutcome);
        let HandleValue::ClientAsyncOperationOutcome(outcome) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(outcome) = outcome.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        let Some(response) = outcome.poll_response() else {
            return SUCCESS;
        };
        let (has_correlation, correlation) = try_ffi!(async_response_correlation_span(
            response.with_asynchronous_correlation_value(|bytes| (bytes.as_ptr(), bytes.len())),
        ));
        let output = kmipkit_client_async_operation_response_v1_t {
            struct_size: try_ffi!(async_response_v1_struct_size()),
            abi_version: 1,
            operation: outcome.operation().raw(),
            result_status: response.result().status().raw(),
            is_pending: u32::from(response.is_pending()),
            has_cancellation_result: 0,
            cancellation_result: 0,
            has_asynchronous_correlation_value: has_correlation,
            asynchronous_correlation_value: correlation,
            has_response_payload: u32::from(outcome.raw().is_some()),
            reserved: 0,
            response_payload_member_count: 0,
        };
        // SAFETY: the caller guarantees valid output storage; the helper checked null and overlap.
        unsafe {
            *out_response = output;
            *out_present = 1;
        }
        SUCCESS
    })
}

/// Reads a typed Process response view with borrowed correlation and payload-shape metadata.
///
/// # Safety
///
/// `outcome_pointer` must be null or a live asynchronous-outcome handle. `out_present` and
/// `out_response` must point to aligned, writable storage of their respective types and must not
/// overlap each other or the outcome handle storage. Any returned correlation span borrows from
/// the outcome and must be copied or discarded before the handle is released.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kmipkit_client_async_operation_outcome_process_response_v1(
    outcome_pointer: *mut kmipkit_client_async_operation_outcome_t,
    out_present: *mut u8,
    out_response: *mut kmipkit_client_async_operation_response_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        // SAFETY: the export's caller must satisfy the documented pointer preconditions.
        let output_initialization = unsafe { async_response_v1_output(out_present, out_response) };
        try_ffi!(output_initialization);
        let handle = read_handle!(outcome_pointer, ClientAsyncOperationOutcome);
        let HandleValue::ClientAsyncOperationOutcome(outcome) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(outcome) = outcome.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        let Some(response) = outcome.process_response() else {
            return SUCCESS;
        };
        let (has_correlation, correlation) = try_ffi!(async_response_correlation_span(
            response.with_asynchronous_correlation_value(|bytes| (bytes.as_ptr(), bytes.len())),
        ));
        let payload_member_count = response.payload_member_count();
        let Ok(response_payload_member_count) = u64::try_from(payload_member_count.unwrap_or(0))
        else {
            return record_ffi_error(ERROR_RESOURCE_LIMIT);
        };
        let output = kmipkit_client_async_operation_response_v1_t {
            struct_size: try_ffi!(async_response_v1_struct_size()),
            abi_version: 1,
            operation: outcome.operation().raw(),
            result_status: response.result().status().raw(),
            is_pending: u32::from(response.is_pending()),
            has_cancellation_result: 0,
            cancellation_result: 0,
            has_asynchronous_correlation_value: has_correlation,
            asynchronous_correlation_value: correlation,
            has_response_payload: u32::from(payload_member_count.is_some()),
            reserved: 0,
            response_payload_member_count,
        };
        // SAFETY: the caller guarantees valid output storage; the helper checked null and overlap.
        unsafe {
            *out_response = output;
            *out_present = 1;
        }
        SUCCESS
    })
}

/// Reads Query Asynchronous Requests status; Table 286 payload content remains generic TTLV.
///
/// # Safety
///
/// `outcome_pointer` must be null or a live asynchronous-outcome handle. `out_present` and
/// `out_response` must point to aligned, writable storage of their respective types and must not
/// overlap each other or the outcome handle storage. Any returned correlation span borrows from
/// the outcome and must be copied or discarded before the handle is released.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kmipkit_client_async_operation_outcome_query_async_requests_response_v1(
    outcome_pointer: *mut kmipkit_client_async_operation_outcome_t,
    out_present: *mut u8,
    out_response: *mut kmipkit_client_async_operation_response_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        // SAFETY: the export's caller must satisfy the documented pointer preconditions.
        let output_initialization = unsafe { async_response_v1_output(out_present, out_response) };
        try_ffi!(output_initialization);
        let handle = read_handle!(outcome_pointer, ClientAsyncOperationOutcome);
        let HandleValue::ClientAsyncOperationOutcome(outcome) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(outcome) = outcome.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        let Some(response) = outcome.query_async_requests_response() else {
            return SUCCESS;
        };
        let (has_correlation, correlation) = try_ffi!(async_response_correlation_span(
            outcome.with_asynchronous_correlation_value(|bytes| (bytes.as_ptr(), bytes.len())),
        ));
        let output = kmipkit_client_async_operation_response_v1_t {
            struct_size: try_ffi!(async_response_v1_struct_size()),
            abi_version: 1,
            operation: outcome.operation().raw(),
            result_status: response.result().status().raw(),
            is_pending: u32::from(response.result().status().raw() == 2),
            has_cancellation_result: 0,
            cancellation_result: 0,
            has_asynchronous_correlation_value: has_correlation,
            asynchronous_correlation_value: correlation,
            has_response_payload: u32::from(response.with_response_payload(|_| ()).is_some()),
            reserved: 0,
            response_payload_member_count: 0,
        };
        // SAFETY: the caller guarantees valid output storage; the helper checked null and overlap.
        unsafe {
            *out_response = output;
            *out_present = 1;
        }
        SUCCESS
    })
}

/// Borrows the exact asynchronous correlation bytes while the outcome remains open.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_async_operation_outcome_correlation_value(
    outcome: *mut kmipkit_client_async_operation_outcome_t,
    out_present: *mut u8,
    out_value: *mut crate::kmipkit_byte_span_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        let value = scalar_out!(out_value);
        *present = 0;
        *value = crate::kmipkit_byte_span_v1_t {
            data: ptr::null(),
            length: 0,
        };
        let handle = read_handle!(outcome, ClientAsyncOperationOutcome);
        let HandleValue::ClientAsyncOperationOutcome(outcome) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(outcome) = outcome.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        if let Some((data, length)) =
            outcome.with_asynchronous_correlation_value(|bytes| (bytes.as_ptr(), bytes.len()))
        {
            *present = 1;
            *value = crate::kmipkit_byte_span_v1_t {
                data: if length == 0 { ptr::null() } else { data },
                length: u64::try_from(length).unwrap_or(u64::MAX),
            };
        }
        SUCCESS
    })
}

/// Copies a present generic Response Payload into a caller-owned TTLV handle.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_async_operation_outcome_response_payload_ttlv(
    outcome: *mut kmipkit_client_async_operation_outcome_t,
    out_present: *mut u8,
    out_payload: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        let payload = out_slot!(out_payload);
        *present = 0;
        let handle = read_handle!(outcome, ClientAsyncOperationOutcome);
        let HandleValue::ClientAsyncOperationOutcome(outcome) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(outcome) = outcome.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        let Some(structure) =
            outcome.with_response_payload(|view| super::clone_structure_view(&view))
        else {
            return SUCCESS;
        };
        let structure = try_ffi!(structure);
        *payload = try_ffi!(try_make_handle(
            Kind::Structure,
            HandleValue::Structure(structure)
        ));
        *present = 1;
        SUCCESS
    })
}

/// Returns the number of accepted response Message Extensions in wire order.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_async_operation_outcome_extension_count(
    outcome: *mut kmipkit_client_async_operation_outcome_t,
    out_count: *mut u64,
) -> i32 {
    ffi_entrypoint(|| {
        let count = scalar_out!(out_count);
        let handle = read_handle!(outcome, ClientAsyncOperationOutcome);
        let HandleValue::ClientAsyncOperationOutcome(outcome) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(outcome) = outcome.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        *count =
            try_ffi!(u64::try_from(outcome.extensions().len()).map_err(|_| ERROR_RESOURCE_LIMIT));
        SUCCESS
    })
}

/// Copies one accepted response Message Extension as generic TTLV.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_async_operation_outcome_extension_ttlv_at(
    outcome: *mut kmipkit_client_async_operation_outcome_t,
    index: u64,
    out_present: *mut u8,
    out_extension: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        let extension_out = out_slot!(out_extension);
        *present = 0;

        let handle = read_handle!(outcome, ClientAsyncOperationOutcome);
        let HandleValue::ClientAsyncOperationOutcome(outcome) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(outcome) = outcome.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        let Ok(index) = usize::try_from(index) else {
            return SUCCESS;
        };
        let Some(extension) = outcome.extensions().get(index) else {
            return SUCCESS;
        };
        let structure = extension.with_ttlv(|view| super::clone_structure_view(&view));
        let structure = try_ffi!(structure);
        *extension_out = try_ffi!(try_make_handle(
            Kind::Structure,
            HandleValue::Structure(structure)
        ));
        *present = 1;
        SUCCESS
    })
}

/// Creates local accounting from a live successful Get Usage Allocation item.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_usage_allocation_context_create(
    item: *mut kmipkit_client_batch_item_response_t,
    requested_units: i64,
    usage_limits_unit: u32,
    out_context: *mut *mut kmipkit_usage_allocation_context_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_context);
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        if item.outcome().result().status().raw() != 0 {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let kmipkit_client::ClientBatchOutcome::GetUsageAllocation(allocation) = item.outcome()
        else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(context) = UsageAllocationContext::from_successful_allocation(
            allocation,
            requested_units,
            usage_limits_unit,
        ) else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = try_ffi!(try_make_handle(
            Kind::UsageAllocationContext,
            HandleValue::UsageAllocationContext(Mutex::new(context)),
        ));
        SUCCESS
    })
}

/// Releases a caller-owned local Usage Allocation context.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_usage_allocation_context_release(
    handle: *mut kmipkit_usage_allocation_context_t,
) {
    ffi_entrypoint(|| release(handle, Kind::UsageAllocationContext));
}

/// Returns the caller-supplied open Usage Limits Unit enumeration.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_usage_allocation_context_usage_limits_unit(
    context: *mut kmipkit_usage_allocation_context_t,
    out_unit: *mut u32,
) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_unit);
        *out = 0;
        let context = read_handle!(context, UsageAllocationContext);
        let HandleValue::UsageAllocationContext(context) = &context.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(context) = context.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        *out = context.usage_limits_unit();
        SUCCESS
    })
}

/// Returns the exact positive allocation count supplied by the caller.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_usage_allocation_context_allocated_units(
    context: *mut kmipkit_usage_allocation_context_t,
    out_units: *mut i64,
) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_units);
        *out = 0;
        let context = read_handle!(context, UsageAllocationContext);
        let HandleValue::UsageAllocationContext(context) = &context.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(context) = context.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        *out = context.allocated_units();
        SUCCESS
    })
}

/// Returns units that have not been reserved through this context.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_usage_allocation_context_available_units(
    context: *mut kmipkit_usage_allocation_context_t,
    out_units: *mut i64,
) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_units);
        *out = 0;
        let context = read_handle!(context, UsageAllocationContext);
        let HandleValue::UsageAllocationContext(context) = &context.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(context) = context.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        *out = context.available_units();
        SUCCESS
    })
}

/// Returns units reserved by guarded calls through this context.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_usage_allocation_context_reserved_units(
    context: *mut kmipkit_usage_allocation_context_t,
    out_units: *mut i64,
) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_units);
        *out = 0;
        let context = read_handle!(context, UsageAllocationContext);
        let HandleValue::UsageAllocationContext(context) = &context.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(context) = context.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        *out = context.reserved_units();
        SUCCESS
    })
}

/// Borrows the immutable server-resolved identifier until the context is released.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_usage_allocation_context_unique_identifier(
    context: *mut kmipkit_usage_allocation_context_t,
    out_identifier: *mut kmipkit_unique_identifier_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        if let Err(status) = write_unique_identifier_v1(out_identifier, None) {
            return record_ffi_error(status);
        }
        let context = read_handle!(context, UsageAllocationContext);
        let HandleValue::UsageAllocationContext(context) = &context.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(context) = context.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        if let Err(status) =
            write_unique_identifier_v1(out_identifier, Some(context.unique_identifier()))
        {
            return record_ffi_error(status);
        }
        SUCCESS
    })
}

/// Executes one typed guarded protection item and returns the standard batch response.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_execute_guarded_protection_item(
    client: *mut kmipkit_client_t,
    context: *mut kmipkit_usage_allocation_context_t,
    item: *mut *mut kmipkit_client_batch_item_t,
    cost: i64,
    limits: *mut kmipkit_codec_limits_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        execute_guarded_client_item(client, context, item, cost, limits, None, out_response)
    })
}

/// Executes one guarded item with an explicit Asynchronous Indicator.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_execute_guarded_protection_item_with_asynchronous_indicator(
    client: *mut kmipkit_client_t,
    context: *mut kmipkit_usage_allocation_context_t,
    item: *mut *mut kmipkit_client_batch_item_t,
    cost: i64,
    asynchronous_indicator: u32,
    limits: *mut kmipkit_codec_limits_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        execute_guarded_client_item(
            client,
            context,
            item,
            cost,
            limits,
            Some(asynchronous_indicator),
            out_response,
        )
    })
}

fn execute_guarded_client_item(
    client: *mut kmipkit_client_t,
    context: *mut kmipkit_usage_allocation_context_t,
    item: *mut *mut kmipkit_client_batch_item_t,
    cost: i64,
    limits: *mut kmipkit_codec_limits_t,
    asynchronous_indicator: Option<u32>,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    let out = out_slot!(out_response);
    if item.is_null() {
        return record_ffi_error(ERROR_INVALID_INPUT);
    }
    // SAFETY: the caller provides a writable in/out slot holding the one-shot item handle.
    let item_slot = unsafe { &mut *item };
    if item_slot.is_null() {
        return record_ffi_error(ERROR_INVALID_INPUT);
    }
    let client = read_handle!(client, Client);
    let HandleValue::Client(client) = &client.value else {
        return record_ffi_error(ERROR_INVALID_INPUT);
    };
    let context = read_handle!(context, UsageAllocationContext);
    let HandleValue::UsageAllocationContext(context) = &context.value else {
        return record_ffi_error(ERROR_INVALID_INPUT);
    };
    let limits_handle = read_handle!(limits, CodecLimits);
    let HandleValue::CodecLimits(limits) = &limits_handle.value else {
        return record_ffi_error(ERROR_INVALID_INPUT);
    };
    if let Err(status) = reference_handle(*item_slot, Kind::BatchItem) {
        return record_ffi_error(status);
    }
    let Ok(mut client) = client.lock() else {
        return record_ffi_error(ERROR_INTERNAL);
    };
    let Ok(mut context) = context.lock() else {
        return record_ffi_error(ERROR_INTERNAL);
    };
    let item_pointer = *item_slot;
    *item_slot = ptr::null_mut();
    let item = match owned_payload(item_pointer, Kind::BatchItem) {
        Ok(HandleValue::BatchItem(item)) => *item,
        Ok(_) => return record_ffi_error(ERROR_INVALID_INPUT),
        Err(status) => return record_ffi_error(status),
    };
    let result = match asynchronous_indicator {
        Some(indicator) => client.execute_guarded_protection_item_with_asynchronous_indicator(
            &mut context,
            item,
            cost,
            indicator,
            limits,
        ),
        None => client.execute_guarded_protection_item(&mut context, item, cost, limits),
    };
    match result {
        Ok(response) => {
            *out = try_ffi!(try_make_handle(
                Kind::ClientBatchResponse,
                HandleValue::ClientBatchResponse(Arc::new(response)),
            ));
            SUCCESS
        }
        Err(error) => record_client_error(&error),
    }
}

/// Executes one typed batch with per-exchange timeout overrides.
///
/// # Safety
///
/// Every non-null `client` and `limits` handle must be live and of its declared
/// type. `batch` must point to an aligned, writable batch-handle slot;
/// `options` must point to a readable, aligned, initialized v1 options record;
/// and `out_response` must point to an aligned, writable output slot. These
/// regions must remain valid for the call. The batch slot, options record, and
/// output slot must not overlap.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kmipkit_client_execute_with_options_v1(
    client: *mut kmipkit_client_t,
    batch: *mut *mut kmipkit_client_batch_t,
    limits: *mut kmipkit_codec_limits_t,
    options: *const kmipkit_request_options_v1_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        // Validate and clear the caller's output before any request-options validation can fail.
        let _ = out_slot!(out_response);
        if options.is_null() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        // SAFETY: non-null options must point to a readable v1 prefix for this call.
        let options = unsafe { &*options };
        let options = match request_options_from_v1(options) {
            Ok(options) => options,
            Err(status) => return record_ffi_error(status),
        };
        execute_client_batch(client, batch, limits, out_response, None, &options, None)
    })
}

/// Executes a typed batch and optionally returns an owned generic response on shape errors.
fn execute_client_batch(
    client: *mut kmipkit_client_t,
    batch: *mut *mut kmipkit_client_batch_t,
    limits: *mut kmipkit_codec_limits_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
    out_error_response_ttlv: Option<*mut *mut kmipkit_ttlv_structure_t>,
    request_options: &RequestOptions,
    profile_validation: Option<(&ProfileValidationSelection, &ProfileValidationContext)>,
) -> i32 {
    let out = out_slot!(out_response);
    let out_error = match out_error_response_ttlv {
        Some(output) => Some(out_slot!(output)),
        None => None,
    };
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
    if let Err(status) = reference_handle(batch_handle, Kind::ClientBatch) {
        return record_ffi_error(status);
    }
    *batch_slot = ptr::null_mut();
    let batch_payload = match owned_payload(batch_handle, Kind::ClientBatch) {
        Ok(HandleValue::ClientBatch(batch)) => batch,
        Ok(_) => return record_ffi_error(ERROR_INVALID_INPUT),
        Err(status) => return record_ffi_error(status),
    };
    let batch = match batch_payload.lock() {
        Ok(mut batch) => match batch.take() {
            Some(batch) => batch.into_batch(),
            None => return record_ffi_error(ERROR_INVALID_INPUT),
        },
        Err(_) => return record_ffi_error(ERROR_INTERNAL),
    };
    let Ok(mut client) = client.lock() else {
        return record_ffi_error(ERROR_INTERNAL);
    };
    let result = match profile_validation {
        Some((selection, context)) => client.execute_with_profiles_and_context_and_options(
            batch,
            selection,
            context,
            limits,
            request_options,
        ),
        None => client.execute_with_options(batch, limits, request_options),
    };
    match result {
        Ok(response) => {
            *out = try_ffi!(try_make_handle(
                Kind::ClientBatchResponse,
                HandleValue::ClientBatchResponse(Arc::new(response)),
            ));
            SUCCESS
        }
        Err(error) => {
            let status = record_client_error(&error);
            if let Some(output) = out_error {
                match clone_error_response_ttlv(&error) {
                    Ok(Some(response)) => {
                        *output = try_ffi!(try_make_handle(
                            Kind::Structure,
                            HandleValue::Structure(response)
                        ));
                    }
                    Ok(None) => {}
                    Err(clone_status) => return record_ffi_error(clone_status), // COVERAGE-EXCLUDE: the retained response is validated; cloning can fail only on allocation or invariant failure, with no fault-injection hook.
                }
            }
            status
        }
    }
}

const PROFILE_SELECTION_MAX_IDS: u64 = 4_096;
const PROFILE_CONTEXT_MAX_STATES: u64 = 100_000;
const PROFILE_STATE_ORIGINAL_CREATION_DATE_PRESENT: u32 = 1;
const PROFILE_STATE_USAGE_LIMITS_PRESENT: u32 = 2;
const PROFILE_STATE_PROTECTION_USE_STARTED: u32 = 4;
const PROFILE_STATE_GET_USAGE_ALLOCATION_PERFORMED: u32 = 8;
const PROFILE_STATE_GET_USAGE_ALLOCATION_CHANGED_COUNT: u32 = 16;
const PROFILE_STATE_USAGE_LIMITS_TOTAL: u32 = 32;
const PROFILE_STATE_USAGE_LIMITS_COUNT: u32 = 64;
const PROFILE_STATE_KNOWN_FIELDS: u32 = 127;
const PROFILE_STATE_BOOLEAN_FIELDS: u32 = 31;

fn profile_selection_from_v1(
    selection: *const kmipkit_profile_validation_selection_v1_t,
) -> FfiResult<ProfileValidationSelection> {
    if selection.is_null() {
        return Err(ERROR_INVALID_INPUT);
    }
    // SAFETY: a non-null selection must reference a readable v1 record for this call.
    let selection = unsafe { &*selection };
    if selection.struct_size as usize != size_of::<kmipkit_profile_validation_selection_v1_t>()
        || selection.abi_version != 1
    {
        return Err(ERROR_INVALID_INPUT);
    }
    if selection.profile_id_count > PROFILE_SELECTION_MAX_IDS {
        return Err(ERROR_RESOURCE_LIMIT);
    }
    let (offsets, lengths) = if selection.profile_id_count == 0 {
        (&[][..], &[][..])
    } else {
        if selection.profile_id_offsets.is_null() || selection.profile_id_lengths.is_null() {
            return Err(ERROR_INVALID_INPUT);
        }
        let count =
            usize::try_from(selection.profile_id_count).map_err(|_| ERROR_RESOURCE_LIMIT)?;
        // SAFETY: the caller supplies readable offset and length arrays of `count` elements.
        unsafe {
            (
                std::slice::from_raw_parts(selection.profile_id_offsets, count),
                std::slice::from_raw_parts(selection.profile_id_lengths, count),
            )
        }
    };
    let profile_id_data = input_span(
        selection.profile_id_data,
        selection.profile_id_data_length,
        MAX_FFI_INPUT_BYTES,
    )?;
    let mut profile_ids = Vec::new();
    profile_ids
        .try_reserve_exact(offsets.len())
        .map_err(|_| ERROR_RESOURCE_LIMIT)?;
    for (offset, length) in offsets.iter().zip(lengths) {
        if *length > MAX_TEXT_BYTES {
            return Err(ERROR_RESOURCE_LIMIT);
        }
        let start = usize::try_from(*offset).map_err(|_| ERROR_INVALID_INPUT)?;
        let end = start
            .checked_add(usize::try_from(*length).map_err(|_| ERROR_INVALID_INPUT)?)
            .filter(|end| *end <= profile_id_data.len())
            .ok_or(ERROR_INVALID_INPUT)?;
        let bytes = &profile_id_data[start..end];
        let text = std::str::from_utf8(bytes).map_err(|_| ERROR_INVALID_INPUT)?;
        profile_ids.push(text.to_owned());
    }
    Ok(ProfileValidationSelection::new(profile_ids))
}

fn profile_context_from_v1(
    context: *const kmipkit_profile_validation_context_v1_t,
) -> FfiResult<ProfileValidationContext> {
    if context.is_null() {
        return Ok(ProfileValidationContext::new());
    }
    // SAFETY: a non-null context must reference a readable v1 record for this call.
    let context = unsafe { &*context };
    if context.struct_size as usize != size_of::<kmipkit_profile_validation_context_v1_t>()
        || context.abi_version != 1
    {
        return Err(ERROR_INVALID_INPUT);
    }
    if context.object_state_count > PROFILE_CONTEXT_MAX_STATES {
        return Err(ERROR_RESOURCE_LIMIT);
    }
    let states = if context.object_state_count == 0 {
        &[][..]
    } else {
        if context.object_states.is_null() {
            return Err(ERROR_INVALID_INPUT);
        }
        let count =
            usize::try_from(context.object_state_count).map_err(|_| ERROR_RESOURCE_LIMIT)?;
        // SAFETY: the caller supplies `object_state_count` readable v1 object-state records.
        unsafe { std::slice::from_raw_parts(context.object_states, count) }
    };
    let mut result = ProfileValidationContext::new();
    let mut context_text_bytes = 0_usize;
    for entry in states {
        if entry.struct_size as usize != size_of::<kmipkit_profile_object_state_v1_t>()
            || entry.abi_version != 1
            || entry.known_fields & !PROFILE_STATE_KNOWN_FIELDS != 0
            || entry.boolean_values & !PROFILE_STATE_BOOLEAN_FIELDS != 0
            || entry.boolean_values & !entry.known_fields != 0
        {
            return Err(ERROR_INVALID_INPUT);
        }
        let index = usize::try_from(entry.batch_item_index).map_err(|_| ERROR_RESOURCE_LIMIT)?;
        let bytes = input_span(
            entry.unique_identifier_data,
            entry.unique_identifier_length,
            MAX_TEXT_BYTES,
        )?;
        context_text_bytes = context_text_bytes
            .checked_add(bytes.len())
            .filter(|total| usize::try_from(MAX_FFI_INPUT_BYTES).is_ok_and(|limit| *total <= limit))
            .ok_or(ERROR_RESOURCE_LIMIT)?;
        let identifier = std::str::from_utf8(bytes).map_err(|_| ERROR_INVALID_INPUT)?;
        let boolean =
            |field| (entry.known_fields & field != 0).then_some(entry.boolean_values & field != 0);
        let state = ProfileObjectState::new(identifier)
            .with_original_creation_date_present(boolean(
                PROFILE_STATE_ORIGINAL_CREATION_DATE_PRESENT,
            ))
            .with_usage_limits_present(boolean(PROFILE_STATE_USAGE_LIMITS_PRESENT))
            .with_protection_use_started(boolean(PROFILE_STATE_PROTECTION_USE_STARTED))
            .with_get_usage_allocation_performed(boolean(
                PROFILE_STATE_GET_USAGE_ALLOCATION_PERFORMED,
            ))
            .with_get_usage_allocation_changed_count(boolean(
                PROFILE_STATE_GET_USAGE_ALLOCATION_CHANGED_COUNT,
            ))
            .with_usage_limits_total(
                (entry.known_fields & PROFILE_STATE_USAGE_LIMITS_TOTAL != 0)
                    .then_some(entry.usage_limits_total),
            )
            .with_usage_limits_count(
                (entry.known_fields & PROFILE_STATE_USAGE_LIMITS_COUNT != 0)
                    .then_some(entry.usage_limits_count),
            );
        result = result.with_object_state(index, state);
    }
    Ok(result)
}

/// Resolves one versioned caller selection with the canonical Rust profile validator.
///
/// A profile-rule failure returns `ERROR_INVALID_INPUT`, clears `out_result`, and records only
/// the stable profile validation detail view. The selection bytes are never copied into an error
/// message. The returned result owns its ordered profile and requirement IDs.
///
/// # Safety
///
/// `selection` must reference an aligned readable v1 selection record and all nested spans and
/// arrays must be readable for this call. `out_result` must reference an aligned writable result
/// handle slot that does not overlap the selection record or its input storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kmipkit_profile_validation_selection_validate_v1(
    selection: *const kmipkit_profile_validation_selection_v1_t,
    out_result: *mut *mut kmipkit_profile_validation_result_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_result);
        let selection = match profile_selection_from_v1(selection) {
            Ok(selection) => selection,
            Err(status) => return record_ffi_error(status),
        };
        let result = match selection.validate() {
            Ok(result) => result,
            Err(error) => {
                let status = record_ffi_error(ERROR_INVALID_INPUT);
                LAST_PROFILE_VALIDATION_ERROR.with(|last_error| {
                    *last_error.borrow_mut() = Some(error);
                });
                return status;
            }
        };
        LAST_PROFILE_VALIDATION_ERROR.with(|last_error| *last_error.borrow_mut() = None);
        *out = try_ffi!(try_make_handle(
            Kind::ProfileValidationResult,
            HandleValue::ProfileValidationResult(result),
        ));
        SUCCESS
    })
}

/// Returns the number of profile IDs in a resolved selection.
///
/// # Safety
///
/// `result` must be a live result handle. `out_count` must reference an aligned writable `u64`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kmipkit_profile_validation_result_profile_id_count_v1(
    result: *mut kmipkit_profile_validation_result_t,
    out_count: *mut u64,
) -> i32 {
    ffi_entrypoint(|| {
        let count = scalar_out!(out_count);
        *count = 0;
        let handle = read_handle!(result, ProfileValidationResult);
        let HandleValue::ProfileValidationResult(result) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *count =
            try_ffi!(u64::try_from(result.profile_ids().len()).map_err(|_| ERROR_RESOURCE_LIMIT));
        SUCCESS
    })
}

/// Returns one profile ID span borrowed from a resolved result handle.
///
/// The returned UTF-8 span remains valid until `kmipkit_profile_validation_result_release`.
///
/// # Safety
///
/// `result` must be live without concurrent release. `out_span` must reference an aligned writable
/// byte-span record and must not overlap the result handle's owned storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kmipkit_profile_validation_result_profile_id_at_v1(
    result: *mut kmipkit_profile_validation_result_t,
    index: u64,
    out_span: *mut crate::kmipkit_byte_span_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        if out_span.is_null() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        // SAFETY: the caller promises a writable aligned span record for this ABI call.
        let span = unsafe { &mut *out_span };
        *span = crate::kmipkit_byte_span_v1_t {
            data: ptr::null(),
            length: 0,
        };
        let handle = read_handle!(result, ProfileValidationResult);
        let HandleValue::ProfileValidationResult(result) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let index = try_ffi!(usize::try_from(index).map_err(|_| ERROR_INVALID_INPUT));
        let Some(profile_id) = result.profile_ids().get(index) else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        span.data = profile_id.as_ptr();
        span.length = try_ffi!(u64::try_from(profile_id.len()).map_err(|_| ERROR_RESOURCE_LIMIT));
        SUCCESS
    })
}

/// Returns the number of requirement IDs in a resolved selection.
///
/// # Safety
///
/// `result` must be a live result handle. `out_count` must reference an aligned writable `u64`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kmipkit_profile_validation_result_requirement_id_count_v1(
    result: *mut kmipkit_profile_validation_result_t,
    out_count: *mut u64,
) -> i32 {
    ffi_entrypoint(|| {
        let count = scalar_out!(out_count);
        *count = 0;
        let handle = read_handle!(result, ProfileValidationResult);
        let HandleValue::ProfileValidationResult(result) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *count = try_ffi!(
            u64::try_from(result.requirement_ids().len()).map_err(|_| ERROR_RESOURCE_LIMIT)
        );
        SUCCESS
    })
}

/// Returns one requirement ID span borrowed from a resolved result handle.
///
/// The returned UTF-8 span remains valid until `kmipkit_profile_validation_result_release`.
///
/// # Safety
///
/// `result` must be live without concurrent release. `out_span` must reference an aligned writable
/// byte-span record and must not overlap the result handle's owned storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kmipkit_profile_validation_result_requirement_id_at_v1(
    result: *mut kmipkit_profile_validation_result_t,
    index: u64,
    out_span: *mut crate::kmipkit_byte_span_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        if out_span.is_null() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        // SAFETY: the caller promises a writable aligned span record for this ABI call.
        let span = unsafe { &mut *out_span };
        *span = crate::kmipkit_byte_span_v1_t {
            data: ptr::null(),
            length: 0,
        };
        let handle = read_handle!(result, ProfileValidationResult);
        let HandleValue::ProfileValidationResult(result) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let index = try_ffi!(usize::try_from(index).map_err(|_| ERROR_INVALID_INPUT));
        let Some(requirement_id) = result.requirement_ids().get(index) else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        span.data = requirement_id.as_ptr();
        span.length =
            try_ffi!(u64::try_from(requirement_id.len()).map_err(|_| ERROR_RESOURCE_LIMIT));
        SUCCESS
    })
}

fn request_message_from_borrowed_ttlv(
    structure: *mut kmipkit_ttlv_structure_t,
) -> FfiResult<RequestMessage> {
    let handle = reference_handle(structure, Kind::Structure)?;
    let HandleValue::Structure(structure) = &handle.value else {
        return Err(ERROR_INVALID_INPUT);
    };
    let structure = clone_structure_view(&structure.view())?;
    RequestMessage::try_from_ttlv(structure).map_err(|_| ERROR_PROTOCOL)
}

fn response_message_from_borrowed_ttlv(
    structure: *mut kmipkit_ttlv_structure_t,
) -> FfiResult<ResponseMessage> {
    let handle = reference_handle(structure, Kind::Structure)?;
    let HandleValue::Structure(structure) = &handle.value else {
        return Err(ERROR_INVALID_INPUT);
    };
    let structure = clone_structure_view(&structure.view())?;
    ResponseMessage::try_from_ttlv(structure).map_err(|_| ERROR_PROTOCOL)
}

fn record_profile_validation_outcome(result: Result<(), ProfileValidationError>) -> i32 {
    match result {
        Ok(()) => {
            LAST_PROFILE_VALIDATION_ERROR.with(|last_error| *last_error.borrow_mut() = None);
            SUCCESS
        }
        Err(error) => {
            let status = record_ffi_error(ERROR_INVALID_INPUT);
            LAST_PROFILE_VALIDATION_ERROR.with(|last_error| {
                *last_error.borrow_mut() = Some(error);
            });
            status
        }
    }
}

/// Checks the selected rules' transport constraints using the canonical Rust validator.
///
/// `transport` is 1 for raw TTLV over TLS and 2 for TTLV over HTTPS/HTTP 1.1.
/// A profile-rule failure is available through the thread-local profile-validation error view.
///
/// # Safety
///
/// `result` must be a live profile-validation-result handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kmipkit_profile_validation_result_validate_transport_v1(
    result: *mut kmipkit_profile_validation_result_t,
    transport: u32,
) -> i32 {
    ffi_entrypoint(|| {
        let handle = read_handle!(result, ProfileValidationResult);
        let HandleValue::ProfileValidationResult(result) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let transport = match transport {
            1 => ProfileTransport::RawTtlvTls,
            2 => ProfileTransport::Https,
            _ => return record_ffi_error(ERROR_INVALID_INPUT),
        };
        record_profile_validation_outcome(result.validate_transport(transport))
    })
}

/// Validates one borrowed generic Request Message TTLV tree against the selected profile rules.
///
/// The input structure remains owned by the caller. A profile-rule failure is available through
/// the thread-local profile-validation error view; malformed message TTLV returns `ERROR_PROTOCOL`.
///
/// # Safety
///
/// `result` and `request` must be live handles of their declared types for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kmipkit_profile_validation_result_validate_request_v1(
    result: *mut kmipkit_profile_validation_result_t,
    request: *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let handle = read_handle!(result, ProfileValidationResult);
        let HandleValue::ProfileValidationResult(result) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let request = match request_message_from_borrowed_ttlv(request) {
            Ok(request) => request,
            Err(status) => return record_ffi_error(status),
        };
        record_profile_validation_outcome(result.validate_request(&request))
    })
}

/// Validates a borrowed generic request TTLV tree with optional caller-supplied object state.
///
/// A null `context` means an empty context. The request structure remains owned by the caller.
/// Profile-rule failures are available through the thread-local profile-validation error view;
/// malformed message TTLV returns `ERROR_PROTOCOL`.
///
/// # Safety
///
/// `result` and `request` must be live handles of their declared types. A non-null `context` must
/// point to a readable v1 record with readable nested state arrays and spans for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kmipkit_profile_validation_result_validate_request_with_context_v1(
    result: *mut kmipkit_profile_validation_result_t,
    request: *mut kmipkit_ttlv_structure_t,
    context: *const kmipkit_profile_validation_context_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        let handle = read_handle!(result, ProfileValidationResult);
        let HandleValue::ProfileValidationResult(result) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let request = match request_message_from_borrowed_ttlv(request) {
            Ok(request) => request,
            Err(status) => return record_ffi_error(status),
        };
        let context = match profile_context_from_v1(context) {
            Ok(context) => context,
            Err(status) => return record_ffi_error(status),
        };
        record_profile_validation_outcome(result.validate_request_with_context(&request, &context))
    })
}

/// Validates a borrowed request and response TTLV tree pair against selected profile rules.
///
/// Both structures remain owned by the caller. A profile-rule failure is available through the
/// thread-local profile-validation error view; malformed message TTLV returns `ERROR_PROTOCOL`.
///
/// # Safety
///
/// `result`, `request`, and `response` must be live handles of their declared types for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kmipkit_profile_validation_result_validate_response_v1(
    result: *mut kmipkit_profile_validation_result_t,
    request: *mut kmipkit_ttlv_structure_t,
    response: *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let handle = read_handle!(result, ProfileValidationResult);
        let HandleValue::ProfileValidationResult(result) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let request = match request_message_from_borrowed_ttlv(request) {
            Ok(request) => request,
            Err(status) => return record_ffi_error(status),
        };
        let response = match response_message_from_borrowed_ttlv(response) {
            Ok(response) => response,
            Err(status) => return record_ffi_error(status),
        };
        record_profile_validation_outcome(result.validate_response(&request, &response))
    })
}

/// Validates a borrowed request and response TTLV tree pair with optional caller-supplied state.
///
/// A null `context` means an empty context. Both structures remain owned by the caller. Profile-
/// rule failures are available through the thread-local profile-validation error view; malformed
/// message TTLV returns `ERROR_PROTOCOL`.
///
/// # Safety
///
/// `result`, `request`, and `response` must be live handles of their declared types. A non-null
/// `context` must point to a readable v1 record with readable nested state arrays and spans.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kmipkit_profile_validation_result_validate_response_with_context_v1(
    result: *mut kmipkit_profile_validation_result_t,
    request: *mut kmipkit_ttlv_structure_t,
    response: *mut kmipkit_ttlv_structure_t,
    context: *const kmipkit_profile_validation_context_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        let handle = read_handle!(result, ProfileValidationResult);
        let HandleValue::ProfileValidationResult(result) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let request = match request_message_from_borrowed_ttlv(request) {
            Ok(request) => request,
            Err(status) => return record_ffi_error(status),
        };
        let response = match response_message_from_borrowed_ttlv(response) {
            Ok(response) => response,
            Err(status) => return record_ffi_error(status),
        };
        let context = match profile_context_from_v1(context) {
            Ok(context) => context,
            Err(status) => return record_ffi_error(status),
        };
        record_profile_validation_outcome(
            result.validate_response_with_context(&request, &response, &context),
        )
    })
}

fn execute_profile_client_batch(
    client: *mut kmipkit_client_t,
    batch: *mut *mut kmipkit_client_batch_t,
    limits: *mut kmipkit_codec_limits_t,
    selection: *const kmipkit_profile_validation_selection_v1_t,
    context: *const kmipkit_profile_validation_context_v1_t,
    options: Option<*const kmipkit_request_options_v1_t>,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    let _ = out_slot!(out_response);
    let selection = match profile_selection_from_v1(selection) {
        Ok(selection) => selection,
        Err(status) => return record_ffi_error(status),
    };
    let context = match profile_context_from_v1(context) {
        Ok(context) => context,
        Err(status) => return record_ffi_error(status),
    };
    let request_options = match options {
        Some(options) => {
            if options.is_null() {
                return record_ffi_error(ERROR_INVALID_INPUT);
            }
            // SAFETY: a non-null options pointer must reference a readable v1 record.
            match request_options_from_v1(unsafe { &*options }) {
                Ok(options) => options,
                Err(status) => return record_ffi_error(status),
            }
        }
        None => RequestOptions::default(),
    };
    execute_client_batch(
        client,
        batch,
        limits,
        out_response,
        None,
        &request_options,
        Some((&selection, &context)),
    )
}

/// Executes one explicitly profile-validated batch with optional caller-owned context.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_execute_with_profiles(
    client: *mut kmipkit_client_t,
    batch: *mut *mut kmipkit_client_batch_t,
    limits: *mut kmipkit_codec_limits_t,
    selection: *const kmipkit_profile_validation_selection_v1_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        execute_profile_client_batch(
            client,
            batch,
            limits,
            selection,
            ptr::null(),
            None,
            out_response,
        )
    })
}

/// Executes one profile-validated batch with a one-exchange timeout override.
///
/// # Safety
///
/// `options` must point to a readable initialized v1 record. All other handle,
/// input, and output pointer contracts match `kmipkit_client_execute_with_profiles`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kmipkit_client_execute_with_profiles_and_options_v1(
    client: *mut kmipkit_client_t,
    batch: *mut *mut kmipkit_client_batch_t,
    limits: *mut kmipkit_codec_limits_t,
    selection: *const kmipkit_profile_validation_selection_v1_t,
    options: *const kmipkit_request_options_v1_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        execute_profile_client_batch(
            client,
            batch,
            limits,
            selection,
            ptr::null(),
            Some(options),
            out_response,
        )
    })
}

/// Executes one profile-validated batch with caller-attested, exchange-local object state.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_execute_with_profiles_and_context(
    client: *mut kmipkit_client_t,
    batch: *mut *mut kmipkit_client_batch_t,
    limits: *mut kmipkit_codec_limits_t,
    selection: *const kmipkit_profile_validation_selection_v1_t,
    context: *const kmipkit_profile_validation_context_v1_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        execute_profile_client_batch(
            client,
            batch,
            limits,
            selection,
            context,
            None,
            out_response,
        )
    })
}

/// Executes one profile-validated batch with caller state and a timeout override.
///
/// # Safety
///
/// `options` must point to a readable initialized v1 record. All other handle,
/// input, and output pointer contracts match `kmipkit_client_execute_with_profiles_and_context`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kmipkit_client_execute_with_profiles_and_context_and_options_v1(
    client: *mut kmipkit_client_t,
    batch: *mut *mut kmipkit_client_batch_t,
    limits: *mut kmipkit_codec_limits_t,
    selection: *const kmipkit_profile_validation_selection_v1_t,
    context: *const kmipkit_profile_validation_context_v1_t,
    options: *const kmipkit_request_options_v1_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        execute_profile_client_batch(
            client,
            batch,
            limits,
            selection,
            context,
            Some(options),
            out_response,
        )
    })
}

/// Executes one batch with optional access to the complete generic response on shape errors.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_execute_with_error_ttlv_v1(
    client: *mut kmipkit_client_t,
    batch: *mut *mut kmipkit_client_batch_t,
    limits: *mut kmipkit_codec_limits_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
    out_error_response_ttlv: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        execute_client_batch(
            client,
            batch,
            limits,
            out_response,
            Some(out_error_response_ttlv),
            &RequestOptions::default(),
            None,
        )
    })
}

/// Executes a batch with request options and returns its generic TTLV on a typed response error.
///
/// # Safety
///
/// Every non-null `client` and `limits` handle must be live and of its declared
/// type. `batch` must point to an aligned, writable batch-handle slot;
/// `options` must point to a readable, aligned, initialized v1 options record;
/// and both output pointers must point to aligned, writable slots. These
/// regions must remain valid for the call. The batch slot, options record, and
/// output slots must not overlap one another.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kmipkit_client_execute_with_options_and_error_ttlv_v1(
    client: *mut kmipkit_client_t,
    batch: *mut *mut kmipkit_client_batch_t,
    limits: *mut kmipkit_codec_limits_t,
    options: *const kmipkit_request_options_v1_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
    out_error_response_ttlv: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        // Clear both output slots before validating any caller-provided options.
        let _ = out_slot!(out_response);
        let _ = out_slot!(out_error_response_ttlv);
        if options.is_null() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        // SAFETY: the non-null options pointer must reference a readable v1 record.
        let options = unsafe { &*options };
        let request_options = match request_options_from_v1(options) {
            Ok(options) => options,
            Err(status) => return record_ffi_error(status),
        };
        execute_client_batch(
            client,
            batch,
            limits,
            out_response,
            Some(out_error_response_ttlv),
            &request_options,
            None,
        )
    })
}

/// Makes the optional protocol-error response independent of the temporary client error.
fn clone_error_response_ttlv(error: &ClientError) -> Result<Option<Structure>, i32> {
    error
        .response_ttlv()
        .map(|response_ttlv| response_ttlv.with_ttlv(|view| super::clone_structure_view(&view)))
        .transpose()
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
            *out = try_ffi!(try_make_handle(
                Kind::ClientBatchItemResponse,
                HandleValue::ClientBatchItemResponse {
                    response: Arc::clone(response),
                    index,
                },
            ));
        }
        SUCCESS
    })
}

/// Returns the number of accepted response Message Extensions on one result item.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_extension_count(
    item: *mut kmipkit_client_batch_item_response_t,
    out_count: *mut u64,
) -> i32 {
    ffi_entrypoint(|| {
        let count = scalar_out!(out_count);
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        *count = try_ffi!(u64::try_from(item.extensions().len()).map_err(|_| ERROR_RESOURCE_LIMIT));
        SUCCESS
    })
}

/// Copies one accepted response Message Extension as generic TTLV.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_extension_ttlv_at(
    item: *mut kmipkit_client_batch_item_response_t,
    index: u64,
    out_present: *mut u8,
    out_extension: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        let extension_out = out_slot!(out_extension);
        *present = 0;

        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        let Ok(index) = usize::try_from(index) else {
            return SUCCESS;
        };
        let Some(extension) = item.extensions().get(index) else {
            return SUCCESS;
        };
        let structure = extension.with_ttlv(|view| super::clone_structure_view(&view));
        let structure = try_ffi!(structure);
        *extension_out = try_ffi!(try_make_handle(
            Kind::Structure,
            HandleValue::Structure(structure)
        ));
        *present = 1;
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
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
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

/// Reads result metadata into a caller-initialized v2 record.
///
/// # Safety
///
/// `item` must be null or a live response-item handle. `out_metadata` must
/// point to an initialized, aligned, writable v2 metadata record and must not
/// overlap the handle storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kmipkit_client_batch_item_response_metadata_v2(
    item: *mut kmipkit_client_batch_item_response_t,
    out_metadata: *mut kmipkit_client_batch_item_metadata_v2_t,
) -> i32 {
    ffi_entrypoint(|| {
        if out_metadata.is_null() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        // SAFETY: the caller supplies a readable initialized header for the v2 output record.
        let (struct_size, abi_version) = unsafe {
            (
                std::ptr::addr_of!((*out_metadata).struct_size).read(),
                std::ptr::addr_of!((*out_metadata).abi_version).read(),
            )
        };
        if struct_size as usize != std::mem::size_of::<kmipkit_client_batch_item_metadata_v2_t>()
            || abi_version != 2
        {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        let outcome = item.outcome();
        let result = outcome.result();
        let message_data = result
            .message()
            .map_or(ptr::null(), |message| message.as_bytes().as_ptr());
        let message_length = result.message().map_or(0, |message| {
            u64::try_from(message.as_bytes().len()).unwrap_or(u64::MAX)
        });
        let metadata = kmipkit_client_batch_item_metadata_v2_t {
            struct_size,
            abi_version: 2,
            operation: outcome.operation().raw(),
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
            has_result_message: u32::from(result.message().is_some()),
            reserved: 0,
            result_message_data: message_data,
            result_message_length: message_length,
        };
        // SAFETY: the caller supplied a writable record with the checked v2 size.
        unsafe { out_metadata.write(metadata) };
        SUCCESS
    })
}

fn empty_unique_identifier_v1(
    out_identifier: *mut kmipkit_unique_identifier_v1_t,
) -> FfiResult<kmipkit_unique_identifier_v1_t> {
    if out_identifier.is_null() {
        return Err(ERROR_INVALID_INPUT);
    }
    // SAFETY: the caller supplies readable version and size fields in the output record.
    let (struct_size, abi_version) = unsafe {
        (
            std::ptr::addr_of!((*out_identifier).struct_size).read(),
            std::ptr::addr_of!((*out_identifier).abi_version).read(),
        )
    };
    if struct_size as usize != std::mem::size_of::<kmipkit_unique_identifier_v1_t>()
        || abi_version != 1
    {
        return Err(ERROR_INVALID_INPUT);
    }
    Ok(kmipkit_unique_identifier_v1_t {
        struct_size,
        abi_version: 1,
        kind: 0,
        reserved: 0,
        text_data: ptr::null(),
        text_length: 0,
        enumeration_value: 0,
        integer_value: 0,
    })
}

pub(super) fn write_unique_identifier_v1(
    out_identifier: *mut kmipkit_unique_identifier_v1_t,
    identifier: Option<&kmipkit_protocol::UniqueIdentifier>,
) -> FfiResult<()> {
    let mut output = empty_unique_identifier_v1(out_identifier)?;
    if let Some(identifier) = identifier {
        match identifier {
            kmipkit_protocol::UniqueIdentifier::TextString(text) => {
                output.kind = crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
                output.text_data = text.as_ptr();
                output.text_length = u64::try_from(text.len()).unwrap_or(u64::MAX);
            }
            kmipkit_protocol::UniqueIdentifier::Enumeration(value) => {
                output.kind = crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_ENUMERATION;
                output.enumeration_value = *value;
            }
            kmipkit_protocol::UniqueIdentifier::Integer(value) => {
                output.kind = crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_INTEGER;
                output.integer_value = *value;
            }
            kmipkit_protocol::UniqueIdentifier::Identifier(text) => {
                output.kind = crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_IDENTIFIER;
                output.text_data = text.as_ptr();
                output.text_length = u64::try_from(text.len()).unwrap_or(u64::MAX);
            }
            _ => return Err(ERROR_INVALID_INPUT),
        }
    }
    // SAFETY: the caller supplied a writable record with the checked v1 size.
    unsafe { out_identifier.write(output) };
    Ok(())
}

fn write_unique_identifier_text_v1(
    out_identifier: *mut kmipkit_unique_identifier_v1_t,
    text: Option<&str>,
) -> FfiResult<()> {
    let mut output = empty_unique_identifier_v1(out_identifier)?;
    if let Some(text) = text {
        output.kind = crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
        output.text_data = text.as_ptr();
        output.text_length = u64::try_from(text.len()).unwrap_or(u64::MAX);
    }
    // SAFETY: the caller supplied a writable record with the checked v1 size.
    unsafe { out_identifier.write(output) };
    Ok(())
}

macro_rules! lifecycle_response_identifier {
    ($name:ident, $accessor:ident, $operation:literal) => {
        #[doc = concat!("Copies the successful ", $operation, " response Unique Identifier.")]
        #[unsafe(no_mangle)]
        pub extern "C" fn $name(
            item: *mut kmipkit_client_batch_item_response_t,
            out_present: *mut u8,
            out_unique_identifier: *mut kmipkit_unique_identifier_v1_t,
        ) -> i32 {
            ffi_entrypoint(|| {
                if out_present.is_null()
                    || out_unique_identifier.is_null()
                    || ffi_output_ranges_overlap(out_present, out_unique_identifier)
                {
                    return record_ffi_error(ERROR_INVALID_INPUT);
                }
                let present = scalar_out!(out_present);
                if let Err(status) = write_unique_identifier_v1(out_unique_identifier, None) {
                    return record_ffi_error(status);
                }
                *present = 0;
                let response_handle = read_handle!(item, ClientBatchItemResponse);
                let item = try_ffi!(batch_response_item_from_handle(&response_handle));
                let outcome = item.outcome();
                if outcome.result().status().raw() != 0 {
                    return SUCCESS;
                }
                if let Some(response) = outcome.$accessor() {
                    let Some(identifier) = response.unique_identifier() else {
                        return record_ffi_error(ERROR_INTERNAL);
                    };
                    if let Err(status) =
                        write_unique_identifier_v1(out_unique_identifier, Some(identifier))
                    {
                        return record_ffi_error(status);
                    }
                    *present = 1;
                }
                SUCCESS
            })
        }
    };
}

lifecycle_response_identifier!(
    kmipkit_client_batch_item_response_activate,
    activate_response,
    "Activate"
);
lifecycle_response_identifier!(
    kmipkit_client_batch_item_response_archive,
    archive_response,
    "Archive"
);
lifecycle_response_identifier!(
    kmipkit_client_batch_item_response_destroy,
    destroy_response,
    "Destroy"
);
lifecycle_response_identifier!(
    kmipkit_client_batch_item_response_recover,
    recover_response,
    "Recover"
);
lifecycle_response_identifier!(
    kmipkit_client_batch_item_response_create_group,
    create_group_response,
    "Create Group"
);
lifecycle_response_identifier!(
    kmipkit_client_batch_item_response_create_user,
    create_user_response,
    "Create User"
);
lifecycle_response_identifier!(
    kmipkit_client_batch_item_response_create_credential,
    create_credential_response,
    "Create Credential"
);
lifecycle_response_identifier!(
    kmipkit_client_batch_item_response_deactivate,
    deactivate_response,
    "Deactivate"
);
lifecycle_response_identifier!(
    kmipkit_client_batch_item_response_decapsulate,
    decapsulate_response,
    "Decapsulate"
);

/// Copies the successful Obliterate response result.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_obliterate(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
) -> i32 {
    ffi_entrypoint(|| {
        if out_present.is_null() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let present = scalar_out!(out_present);
        *present = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        let outcome = item.outcome();
        if outcome.result().status().raw() != 0 {
            return SUCCESS;
        }
        if outcome.obliterate_response().is_some() {
            *present = 1;
        }
        SUCCESS
    })
}

/// Copies the successful Set Endpoint Role response fields.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_set_endpoint_role(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_role: *mut u32,
    out_vendor_identification: *mut crate::kmipkit_byte_span_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        if out_present.is_null() || out_role.is_null() || out_vendor_identification.is_null() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let present = scalar_out!(out_present);
        let role = scalar_out!(out_role);
        let vendor_id = scalar_out!(out_vendor_identification);
        *present = 0;
        *role = 0;
        *vendor_id = crate::kmipkit_byte_span_v1_t::default();
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        let outcome = item.outcome();
        if outcome.result().status().raw() != 0 {
            return SUCCESS;
        }
        if let Some(resp) = outcome.set_endpoint_role_response() {
            *present = 1;
            if let Some(r) = resp.endpoint_role() {
                *role = r.raw();
            }
            if let Some(vid) = resp.vendor_identification() {
                vendor_id.data = vid.as_ptr();
                vendor_id.length = u64::try_from(vid.len()).unwrap_or(u64::MAX);
            }
        }
        SUCCESS
    })
}

/// Validates whether an originating operation is permitted under NIST SP 800-57 Part 1 Rev 5.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_nist_key_lifecycle_can_originate(
    state_enum: u32,
    out_can_originate: *mut u8,
) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_can_originate);
        let Some(state) = kmipkit_protocol::KeyLifecycleState::from_u32(state_enum) else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = u8::from(kmipkit_protocol::NistKeyLifecycleGuard::can_originate(
            state,
        ));
        SUCCESS
    })
}

/// Validates whether a recipient or processing operation is permitted under NIST SP 800-57 Part 1 Rev 5.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_nist_key_lifecycle_can_process(
    state_enum: u32,
    out_can_process: *mut u8,
) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_can_process);
        let Some(state) = kmipkit_protocol::KeyLifecycleState::from_u32(state_enum) else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = u8::from(kmipkit_protocol::NistKeyLifecycleGuard::can_process(state));
        SUCCESS
    })
}

/// Copies the successful Encapsulate response Unique Identifier and borrows the ciphertext data.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_encapsulate(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_unique_identifier: *mut kmipkit_unique_identifier_v1_t,
    out_data_present: *mut u8,
    out_data: *mut crate::kmipkit_byte_span_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        if out_present.is_null()
            || out_unique_identifier.is_null()
            || out_data_present.is_null()
            || out_data.is_null()
            || ffi_output_ranges_overlap(out_present, out_unique_identifier)
        {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let present = scalar_out!(out_present);
        let data_present = scalar_out!(out_data_present);
        let data = scalar_out!(out_data);
        *present = 0;
        try_ffi!(set_borrowed_span(data_present, data, None));
        if let Err(status) = write_unique_identifier_v1(out_unique_identifier, None) {
            return record_ffi_error(status);
        }
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        let outcome = item.outcome();
        if outcome.result().status().raw() != 0 {
            return SUCCESS;
        }
        if let Some(encapsulate) = outcome.encapsulate_response() {
            let Some(identifier) = encapsulate.unique_identifier() else {
                return record_ffi_error(ERROR_INTERNAL);
            };
            if let Err(status) = write_unique_identifier_v1(out_unique_identifier, Some(identifier))
            {
                return record_ffi_error(status);
            }
            try_ffi!(set_borrowed_span(data_present, data, encapsulate.data()));
            *present = 1;
        }
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_create(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_object_type: *mut u32,
    out_unique_identifier: *mut kmipkit_unique_identifier_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        let object_type = scalar_out!(out_object_type);
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        if let Err(status) = write_unique_identifier_v1(out_unique_identifier, None) {
            return record_ffi_error(status);
        }
        *present = 0;
        *object_type = 0;
        if let Some(create) = item.outcome().create_response()
            && let (Some(object), Some(identifier)) =
                (create.object_type(), create.unique_identifier())
        {
            if let Err(status) = write_unique_identifier_v1(out_unique_identifier, Some(identifier))
            {
                return record_ffi_error(status); // COVERAGE-EXCLUDE: the same stable output record passed the preflight immediately above.
            }
            *present = 1;
            *object_type = object.raw();
        }
        SUCCESS
    })
}

/// Copies the generated certificate identifier only from a successful Certify result.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_certify(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_unique_identifier: *mut kmipkit_unique_identifier_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        if let Err(status) = write_unique_identifier_v1(out_unique_identifier, None) {
            return record_ffi_error(status);
        }
        *present = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        if item.outcome().result().status().raw() != 0 {
            return SUCCESS;
        }
        if let Some(certify) = item.outcome().certify_response() {
            if let Err(status) = // COVERAGE-EXCLUDE: output record passed preflight.
                write_unique_identifier_v1(
                    out_unique_identifier,
                    certify.unique_identifier(),
                )
            {
                return record_ffi_error(status); // COVERAGE-EXCLUDE: the same stable output record passed the preflight immediately above.
            }
            *present = 1;
        }
        SUCCESS
    })
}

/// Copies the replacement certificate identifier only from a successful Re-certify result.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_recertify(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_unique_identifier: *mut kmipkit_unique_identifier_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        if let Err(status) = write_unique_identifier_v1(out_unique_identifier, None) {
            return record_ffi_error(status);
        }
        *present = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        let outcome = item.outcome();
        if outcome.result().status().raw() != 0 {
            return SUCCESS;
        }
        if let Some(recertify) = outcome.recertify_response() {
            if let Err(status) = // COVERAGE-EXCLUDE: output record passed preflight.
                write_unique_identifier_v1(
                    out_unique_identifier,
                    recertify.unique_identifier(),
                )
            {
                return record_ffi_error(status); // COVERAGE-EXCLUDE: the same stable output record passed the preflight immediately above.
            }
            *present = 1;
        }
        SUCCESS
    })
}

/// Copies the replacement Symmetric Key identifier only from a successful Re-key result.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_rekey(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_unique_identifier: *mut kmipkit_unique_identifier_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        if let Err(status) = write_unique_identifier_v1(out_unique_identifier, None) {
            return record_ffi_error(status);
        }
        *present = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        let outcome = item.outcome();
        if outcome.result().status().raw() != 0 {
            return SUCCESS;
        }
        if let Some(rekey) = outcome.rekey_response() {
            if let Err(status) =
                write_unique_identifier_v1(out_unique_identifier, rekey.unique_identifier())
            {
                return record_ffi_error(status);
            }
            *present = 1;
        }
        SUCCESS
    })
}

/// Copies both replacement identifiers only from a successful Re-key Key Pair result.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_rekey_key_pair(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_private_key_unique_identifier: *mut kmipkit_unique_identifier_v1_t,
    out_public_key_unique_identifier: *mut kmipkit_unique_identifier_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        if let Err(status) = write_unique_identifier_v1(out_private_key_unique_identifier, None) {
            return record_ffi_error(status);
        }
        if let Err(status) = write_unique_identifier_v1(out_public_key_unique_identifier, None) {
            return record_ffi_error(status);
        }
        *present = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        let outcome = item.outcome();
        if outcome.result().status().raw() != 0 {
            return SUCCESS;
        }
        if let Some(result) = outcome.rekey_key_pair_response() {
            if let Err(status) = write_unique_identifier_v1(
                out_private_key_unique_identifier,
                result.private_key_unique_identifier(),
            ) {
                return record_ffi_error(status);
            }
            if let Err(status) = write_unique_identifier_v1(
                out_public_key_unique_identifier,
                result.public_key_unique_identifier(),
            ) {
                return record_ffi_error(status);
            }
            *present = 1;
        }
        SUCCESS
    })
}

/// Copies the optional Re-Provision identifier only from a successful Table 314 result.
///
/// A successful typed response is present even when its optional Unique Identifier is absent.
/// Output regions must be writable and must not overlap.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_reprovision(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_identifier_present: *mut u8,
    out_unique_identifier: *mut kmipkit_unique_identifier_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        if out_present.is_null()
            || out_identifier_present.is_null()
            || out_unique_identifier.is_null()
            || ffi_output_ranges_overlap(out_present, out_identifier_present)
            || ffi_output_ranges_overlap(out_present, out_unique_identifier)
            || ffi_output_ranges_overlap(out_identifier_present, out_unique_identifier)
        {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let present = scalar_out!(out_present);
        let identifier_present = scalar_out!(out_identifier_present);
        *present = 0;
        *identifier_present = 0;
        if let Err(status) = write_unique_identifier_v1(out_unique_identifier, None) {
            return record_ffi_error(status);
        }
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        if item.outcome().result().status().raw() != 0 {
            return SUCCESS;
        }
        if let Some(reprovision) = item.outcome().reprovision_response() {
            if let Some(identifier) = reprovision.unique_identifier() {
                if let Err(status) =
                    write_unique_identifier_v1(out_unique_identifier, Some(identifier))
                {
                    return record_ffi_error(status);
                }
                *identifier_present = 1;
            }
            *present = 1;
        }
        SUCCESS
    })
}

fn ffi_output_ranges_overlap<T, U>(first: *mut T, second: *mut U) -> bool {
    let first_start = first.cast::<u8>() as usize;
    let second_start = second.cast::<u8>() as usize;
    let Some(first_end) = first_start.checked_add(std::mem::size_of::<T>()) else {
        return true;
    };
    let Some(second_end) = second_start.checked_add(std::mem::size_of::<U>()) else {
        return true;
    };
    first_start < second_end && second_start < first_end
}

/// Copies typed Check fields, preserving policy values on failure and hiding any failure identifier.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_check(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_unique_identifier_present: *mut u8,
    out_unique_identifier: *mut kmipkit_unique_identifier_v1_t,
    out_usage_limits_count_present: *mut u8,
    out_usage_limits_count: *mut i64,
    out_cryptographic_usage_mask_present: *mut u8,
    out_cryptographic_usage_mask: *mut i32,
    out_lease_time_present: *mut u8,
    out_lease_time: *mut u32,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        let identifier_present = scalar_out!(out_unique_identifier_present);
        let count_present = scalar_out!(out_usage_limits_count_present);
        let count = scalar_out!(out_usage_limits_count);
        let mask_present = scalar_out!(out_cryptographic_usage_mask_present);
        let mask = scalar_out!(out_cryptographic_usage_mask);
        let lease_present = scalar_out!(out_lease_time_present);
        let lease = scalar_out!(out_lease_time);
        *present = 0;
        *identifier_present = 0;
        *count_present = 0;
        *count = 0;
        *mask_present = 0;
        *mask = 0;
        *lease_present = 0;
        *lease = 0;
        if let Err(status) = write_unique_identifier_v1(out_unique_identifier, None) {
            return record_ffi_error(status);
        }
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        if let Some(check) = item.outcome().check_response() {
            if let Some(identifier) = check.unique_identifier() {
                if let Err(status) = // COVERAGE-EXCLUDE: output record passed preflight.
                    write_unique_identifier_v1(out_unique_identifier, Some(identifier))
                {
                    return record_ffi_error(status); // COVERAGE-EXCLUDE: the same stable output record passed the preflight immediately above.
                }
                *identifier_present = 1;
            }
            if let Some(value) = check.usage_limits_count() {
                *count = value;
                *count_present = 1;
            }
            if let Some(value) = check.cryptographic_usage_mask() {
                *mask = value;
                *mask_present = 1;
            }
            if let Some(value) = check.lease_time() {
                *lease = value;
                *lease_present = 1;
            }
            *present = 1;
        }
        SUCCESS
    })
}

/// Returns a caller-owned copy of the complete ordered Check response TTLV, including failures.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_check_ttlv(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_response_ttlv: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        let response_ttlv = out_slot!(out_response_ttlv);
        *present = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        if let Some(check) = item.outcome().check_response() {
            let structure = try_ffi!(check.with_ttlv(|view| super::clone_structure_view(&view)));
            *response_ttlv = try_ffi!(try_make_handle(
                Kind::Structure,
                HandleValue::Structure(structure),
            ));
            *present = 1;
        }
        SUCCESS
    })
}

/// Reports whether the batch item has a typed Ping response, including Pending.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_ping(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        *present = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        if item.outcome().response().ping().is_some() {
            *present = 1;
        }
        SUCCESS
    })
}

/// Reports whether the batch item has a typed Logout response, including Pending.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_logout(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        *present = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        if item.outcome().logout_response().is_some() {
            *present = 1;
        }
        SUCCESS
    })
}

/// Copies the typed Query response payload, retaining ordered and unknown Items.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_query_ttlv(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_response_ttlv: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        let response_ttlv = out_slot!(out_response_ttlv);
        *present = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        let outcome = item.outcome();
        let response = outcome.response();
        let Some(query) = response.query() else {
            return SUCCESS;
        };
        let structure = try_ffi!(query.to_ttlv_payload().map_err(|_| ERROR_INTERNAL));
        *response_ttlv = try_ffi!(try_make_handle(
            Kind::Structure,
            HandleValue::Structure(structure),
        ));
        *present = 1;
        SUCCESS
    })
}

/// Returns whether this result is a typed Query response and its field count.
///
/// `out_present` is zero when the result belongs to another operation. A
/// failed Query response is still present but can have zero payload fields.
/// Output regions must be writable and must not overlap.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_query_field_count(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_count: *mut u64,
) -> i32 {
    ffi_entrypoint(|| {
        if out_present.is_null()
            || out_count.is_null()
            || ffi_output_ranges_overlap(out_present, out_count)
        {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let present = scalar_out!(out_present);
        let count = scalar_out!(out_count);
        *present = 0;
        *count = 0;

        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        let outcome = item.outcome();
        let response = outcome.response();
        let Some(query) = response.query() else {
            return SUCCESS;
        };
        *count =
            try_ffi!(u64::try_from(query.response_field_count()).map_err(|_| ERROR_RESOURCE_LIMIT));
        *present = 1;
        SUCCESS
    })
}

/// Returns one Query field in wire order using the Rust protocol classifier.
///
/// Enumeration values are written to `out_enumeration` and have no Item
/// handle. Text, structure, and unknown members return a caller-owned `Item`
/// handle, released with `kmipkit_ttlv_item_release`. `out_present` is zero
/// when the result is not a Query response; an out-of-range index is invalid.
/// Output regions must be writable and must not overlap.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_query_field_at(
    item: *mut kmipkit_client_batch_item_response_t,
    index: u64,
    out_present: *mut u8,
    out_tag: *mut u32,
    out_kind: *mut u8,
    out_enumeration: *mut u32,
    out_item: *mut *mut kmipkit_ttlv_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let outputs = [
            out_present.cast::<u8>(),
            out_tag.cast::<u8>(),
            out_kind.cast::<u8>(),
            out_enumeration.cast::<u8>(),
            out_item.cast::<u8>(),
        ];
        if outputs.iter().any(|output| output.is_null())
            || ffi_output_ranges_overlap(out_present, out_tag)
            || ffi_output_ranges_overlap(out_present, out_kind)
            || ffi_output_ranges_overlap(out_present, out_enumeration)
            || ffi_output_ranges_overlap(out_present, out_item)
            || ffi_output_ranges_overlap(out_tag, out_kind)
            || ffi_output_ranges_overlap(out_tag, out_enumeration)
            || ffi_output_ranges_overlap(out_tag, out_item)
            || ffi_output_ranges_overlap(out_kind, out_enumeration)
            || ffi_output_ranges_overlap(out_kind, out_item)
            || ffi_output_ranges_overlap(out_enumeration, out_item)
        {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let present = scalar_out!(out_present);
        let tag = scalar_out!(out_tag);
        let kind = scalar_out!(out_kind);
        let enumeration = scalar_out!(out_enumeration);
        let output_item = out_slot!(out_item);
        *present = 0;
        *tag = 0;
        *kind = 0;
        *enumeration = 0;

        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let response_item = try_ffi!(batch_response_item_from_handle(&response_handle));
        let outcome = response_item.outcome();
        let response = outcome.response();
        let Some(query) = response.query() else {
            return SUCCESS;
        };
        let index = try_ffi!(usize::try_from(index).map_err(|_| ERROR_INVALID_INPUT));
        let Some(field) = query.response_field(index) else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *tag = field.tag();
        let field_kind = field.kind();
        *kind = field_kind as u8;
        match field_kind {
            QueryResponseFieldKind::Enumeration => {
                *enumeration = try_ffi!(field.raw_enumeration().ok_or(ERROR_INTERNAL));
            }
            QueryResponseFieldKind::TextString
            | QueryResponseFieldKind::Structure
            | QueryResponseFieldKind::Unknown => {}
        }
        if field_kind != QueryResponseFieldKind::Enumeration {
            let owned_item = try_ffi!(
                clone_query_response_field_item(field).and_then(|item| item.ok_or(ERROR_INTERNAL))
            );
            *output_item = try_ffi!(try_make_handle(Kind::Item, HandleValue::Item(owned_item)));
        }
        *present = 1;
        SUCCESS
    })
}

/// Copies the ordered Response Payload for any valid batch result, when present.
///
/// The returned structure is caller-owned and remains valid after the result
/// handle is released. Release it with `kmipkit_ttlv_structure_release`.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_raw_ttlv(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_payload: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        let payload = out_slot!(out_payload);
        *present = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        let Some(view) = item.raw() else {
            return SUCCESS;
        };
        let structure = try_ffi!(super::clone_structure_view(&view));
        *payload = try_ffi!(try_make_handle(
            Kind::Structure,
            HandleValue::Structure(structure),
        ));
        *present = 1;
        SUCCESS
    })
}

/// Copies the ordered Response Header for any valid batch result.
///
/// The returned structure is caller-owned and remains valid after the result
/// handle is released. Release it with `kmipkit_ttlv_structure_release`.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_header_ttlv(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_header: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        let header = out_slot!(out_header);
        *present = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        let view = try_ffi!(item.response_header_raw().ok_or(ERROR_INTERNAL));
        let structure = try_ffi!(super::clone_structure_view(&view));
        *header = try_ffi!(try_make_handle(
            Kind::Structure,
            HandleValue::Structure(structure),
        ));
        *present = 1;
        SUCCESS
    })
}

/// Reads the raw-preserving Ticket fields for §6.1.12 Delegated Login.
///
/// The returned Ticket Value span is borrowed from `item` and remains valid
/// until that response item is released. The span must not be logged.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_delegated_login(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_ticket_present: *mut u8,
    out_ticket_type: *mut u32,
    out_ticket_value: *mut crate::kmipkit_byte_span_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        let ticket_present = scalar_out!(out_ticket_present);
        let ticket_type = scalar_out!(out_ticket_type);
        let ticket_value = scalar_out!(out_ticket_value);
        *present = 0;
        *ticket_type = 0;
        try_ffi!(set_borrowed_span(ticket_present, ticket_value, None));
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        if let Some(delegated_login) = item.outcome().delegated_login_response() {
            *present = 1;
            if let Some(ticket) = delegated_login.ticket() {
                *ticket_type = ticket.ticket_type().raw();
                if let Err(status) = ticket.with_ticket_value(|bytes| {
                    set_borrowed_span(ticket_present, ticket_value, Some(bytes))
                }) {
                    return record_ffi_error(status); // COVERAGE-EXCLUDE: usize lengths fit u64 on all supported 64-bit targets.
                }
            }
        }
        SUCCESS
    })
}

/// Returns an owned generic copy of the complete Ticket TTLV Structure.
///
/// Child order, unknown tags, and extension fields are retained exactly.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_delegated_login_ttlv(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_ticket_ttlv: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        let ticket_ttlv = out_slot!(out_ticket_ttlv);
        *present = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        if let Some(ticket) = item
            .outcome()
            .delegated_login_response()
            .and_then(kmipkit_protocol::DelegatedLoginResponse::ticket)
        {
            let structure = try_ffi!(ticket.with_ttlv(|view| super::clone_structure_view(&view)));
            *ticket_ttlv = try_ffi!(try_make_handle(
                Kind::Structure,
                HandleValue::Structure(structure),
            ));
            *present = 1;
        }
        SUCCESS
    })
}

/// Reads the raw-preserving Ticket fields for §6.1.30 Login.
///
/// The returned Ticket Value span is borrowed from `item` and remains valid
/// until that response item is released. The span must not be logged.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_login(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_ticket_present: *mut u8,
    out_ticket_type: *mut u32,
    out_ticket_value: *mut crate::kmipkit_byte_span_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        let ticket_present = scalar_out!(out_ticket_present);
        let ticket_type = scalar_out!(out_ticket_type);
        let ticket_value = scalar_out!(out_ticket_value);
        *present = 0;
        *ticket_type = 0;
        try_ffi!(set_borrowed_span(ticket_present, ticket_value, None));
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        if let Some(login) = item.outcome().login_response() {
            *present = 1;
            if let Some(ticket) = login.ticket() {
                *ticket_type = ticket.ticket_type().raw();
                if let Err(status) = ticket.with_ticket_value(|bytes| {
                    set_borrowed_span(ticket_present, ticket_value, Some(bytes))
                }) {
                    return record_ffi_error(status); // COVERAGE-EXCLUDE: usize lengths fit u64 on all supported 64-bit targets.
                }
            }
        }
        SUCCESS
    })
}

/// Returns an owned generic copy of the complete Login Ticket TTLV Structure.
///
/// Child order, unknown tags, and extension fields are retained exactly.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_login_ttlv(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_ticket_ttlv: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        let ticket_ttlv = out_slot!(out_ticket_ttlv);
        *present = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        if let Some(ticket) = item
            .outcome()
            .login_response()
            .and_then(kmipkit_protocol::LoginResponse::ticket)
        {
            let structure = try_ffi!(ticket.with_ttlv(|view| super::clone_structure_view(&view)));
            *ticket_ttlv = try_ffi!(try_make_handle(
                Kind::Structure,
                HandleValue::Structure(structure),
            ));
            *present = 1;
        }
        SUCCESS
    })
}

/// Copies the complete generic response only from a successful Certify result.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_certify_ttlv(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_response_ttlv: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        let response_ttlv = out_slot!(out_response_ttlv);
        *present = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        let outcome = item.outcome();
        if outcome.result().status().raw() != 0 {
            return SUCCESS;
        }
        if let Some(certify) = outcome.certify_response() {
            let Some(structure) = certify.with_ttlv(|view| super::clone_structure_view(&view))
            else {
                return record_ffi_error(ERROR_INTERNAL); // COVERAGE-EXCLUDE: a successful parsed Certify result always retains its validated TTLV.
            };
            let structure = try_ffi!(structure);
            *response_ttlv = try_ffi!(try_make_handle(
                Kind::Structure,
                HandleValue::Structure(structure),
            ));
            *present = 1;
        }
        SUCCESS
    })
}

/// Copies the complete generic response only from a successful Re-certify result.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_recertify_ttlv(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_response_ttlv: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        let response_ttlv = out_slot!(out_response_ttlv);
        *present = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        let outcome = item.outcome();
        if outcome.result().status().raw() != 0 {
            return SUCCESS;
        }
        if let Some(recertify) = outcome.recertify_response() {
            let structure = try_ffi!(require_success_response_ttlv(
                recertify.with_ttlv(|view| super::clone_structure_view(&view))
            ));
            *response_ttlv = try_ffi!(try_make_handle(
                Kind::Structure,
                HandleValue::Structure(structure),
            ));
            *present = 1;
        }
        SUCCESS
    })
}

/// Copies the complete generic response only from a successful Re-key result.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_rekey_ttlv(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_response_ttlv: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        let response_ttlv = out_slot!(out_response_ttlv);
        *present = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        let outcome = item.outcome();
        if outcome.result().status().raw() != 0 {
            return SUCCESS;
        }
        if let Some(rekey) = outcome.rekey_response() {
            let structure = try_ffi!(require_success_response_ttlv(
                rekey.with_ttlv(|view| super::clone_structure_view(&view))
            ));
            *response_ttlv = try_ffi!(try_make_handle(
                Kind::Structure,
                HandleValue::Structure(structure),
            ));
            *present = 1;
        }
        SUCCESS
    })
}

/// Copies complete generic response TTLV only from a successful Re-key Key Pair result.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_rekey_key_pair_ttlv(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_response_ttlv: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        let response_ttlv = out_slot!(out_response_ttlv);
        *present = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        if item.outcome().result().status().raw() != 0 {
            return SUCCESS;
        }
        if let Some(result) = item.outcome().rekey_key_pair_response() {
            let structure = try_ffi!(require_success_response_ttlv(
                result.with_ttlv(|view| super::clone_structure_view(&view))
            ));
            *response_ttlv = try_ffi!(try_make_handle(
                Kind::Structure,
                HandleValue::Structure(structure)
            ));
            *present = 1;
        }
        SUCCESS
    })
}

/// Copies the complete generic TTLV only from a successful Re-Provision result.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_reprovision_ttlv(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_response_ttlv: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        let response_ttlv = out_slot!(out_response_ttlv);
        *present = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        if item.outcome().result().status().raw() != 0 {
            return SUCCESS;
        }
        if let Some(reprovision) = item.outcome().reprovision_response() {
            let structure = try_ffi!(require_success_response_ttlv(
                reprovision.with_ttlv(|view| super::clone_structure_view(&view))
            ));
            *response_ttlv = try_ffi!(try_make_handle(
                Kind::Structure,
                HandleValue::Structure(structure),
            ));
            *present = 1;
        }
        SUCCESS
    })
}

#[derive(Clone, Copy)]
enum RegisterRevokeResponseKind {
    Register,
    Revoke,
}

fn copy_register_revoke_identifier_impl(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_unique_identifier: *mut kmipkit_unique_identifier_v1_t,
    kind: RegisterRevokeResponseKind,
) -> i32 {
    if out_present.is_null()
        || out_unique_identifier.is_null()
        || ffi_output_ranges_overlap(out_present, out_unique_identifier)
    {
        return record_ffi_error(ERROR_INVALID_INPUT);
    }
    let present = scalar_out!(out_present);
    if let Err(status) = write_unique_identifier_v1(out_unique_identifier, None) {
        return record_ffi_error(status);
    }
    *present = 0;
    let response_handle = read_handle!(item, ClientBatchItemResponse);
    let item = try_ffi!(batch_response_item_from_handle(&response_handle));
    if item.outcome().result().status().raw() != 0 {
        return SUCCESS;
    }
    let identifier = match (kind, item.outcome()) {
        (
            RegisterRevokeResponseKind::Register,
            kmipkit_client::ClientBatchOutcome::Register(value),
        ) => value.unique_identifier(),
        (RegisterRevokeResponseKind::Revoke, kmipkit_client::ClientBatchOutcome::Revoke(value)) => {
            value.unique_identifier()
        }
        _ => return SUCCESS,
    };
    let Some(identifier) = identifier else {
        return record_ffi_error(ERROR_INTERNAL);
    };
    if let Err(status) = write_unique_identifier_v1(out_unique_identifier, Some(identifier)) {
        return record_ffi_error(status);
    }
    *present = 1;
    SUCCESS
}

/// Copies the required returned Unique Identifier only from a successful Register result.
/// Output regions must be writable and must not overlap.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_register(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_unique_identifier: *mut kmipkit_unique_identifier_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        copy_register_revoke_identifier_impl(
            item,
            out_present,
            out_unique_identifier,
            RegisterRevokeResponseKind::Register,
        )
    })
}

fn copy_register_revoke_ttlv_impl(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_response_ttlv: *mut *mut kmipkit_ttlv_structure_t,
    kind: RegisterRevokeResponseKind,
) -> i32 {
    if out_present.is_null()
        || out_response_ttlv.is_null()
        || ffi_output_ranges_overlap(out_present, out_response_ttlv)
    {
        return record_ffi_error(ERROR_INVALID_INPUT);
    }
    let present = scalar_out!(out_present);
    let response_ttlv = out_slot!(out_response_ttlv);
    *present = 0;
    let response_handle = read_handle!(item, ClientBatchItemResponse);
    let item = try_ffi!(batch_response_item_from_handle(&response_handle));
    if item.outcome().result().status().raw() != 0 {
        return SUCCESS;
    }
    let response_ttlv_view = match (kind, item.outcome()) {
        (
            RegisterRevokeResponseKind::Register,
            kmipkit_client::ClientBatchOutcome::Register(value),
        ) => value.with_ttlv(|view| super::clone_structure_view(&view)),
        (RegisterRevokeResponseKind::Revoke, kmipkit_client::ClientBatchOutcome::Revoke(value)) => {
            value.with_ttlv(|view| super::clone_structure_view(&view))
        }
        _ => return SUCCESS,
    };
    let structure = try_ffi!(require_success_response_ttlv(response_ttlv_view));
    *response_ttlv = try_ffi!(try_make_handle(
        Kind::Structure,
        HandleValue::Structure(structure),
    ));
    *present = 1;
    SUCCESS
}

/// Copies the complete ordered response TTLV only from successful Register results.
/// Output regions must be writable and must not overlap.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_register_ttlv(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_response_ttlv: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        copy_register_revoke_ttlv_impl(
            item,
            out_present,
            out_response_ttlv,
            RegisterRevokeResponseKind::Register,
        )
    })
}

/// Copies the required returned Unique Identifier only from a successful Revoke result.
/// Output regions must be writable and must not overlap.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_revoke(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_unique_identifier: *mut kmipkit_unique_identifier_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        copy_register_revoke_identifier_impl(
            item,
            out_present,
            out_unique_identifier,
            RegisterRevokeResponseKind::Revoke,
        )
    })
}

/// Copies the complete ordered response TTLV only from successful Revoke results.
/// Output regions must be writable and must not overlap.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_revoke_ttlv(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_response_ttlv: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        copy_register_revoke_ttlv_impl(
            item,
            out_present,
            out_response_ttlv,
            RegisterRevokeResponseKind::Revoke,
        )
    })
}

/// Borrows the resolved identifier from a successful Get Usage Allocation response item.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_get_usage_allocation(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_unique_identifier: *mut kmipkit_unique_identifier_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        if let Err(status) = write_unique_identifier_v1(out_unique_identifier, None) {
            return record_ffi_error(status);
        }
        *present = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        if item.outcome().result().status().raw() != 0 {
            return SUCCESS;
        }
        if let kmipkit_client::ClientBatchOutcome::GetUsageAllocation(allocation) = item.outcome()
            && let Some(identifier) = allocation.unique_identifier()
        {
            if let Err(status) = write_unique_identifier_v1(out_unique_identifier, Some(identifier))
            {
                return record_ffi_error(status); // COVERAGE-EXCLUDE: the same stable output record passed the preflight immediately above.
            }
            *present = 1;
        }
        SUCCESS
    })
}

/// Copies the complete generic response only from a successful Get Usage Allocation result.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_get_usage_allocation_ttlv(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_response_ttlv: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        let response_ttlv = out_slot!(out_response_ttlv);
        *present = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        let outcome = item.outcome();
        if outcome.result().status().raw() != 0 {
            return SUCCESS;
        }
        if let kmipkit_client::ClientBatchOutcome::GetUsageAllocation(allocation) = outcome {
            let Some(structure) = allocation.with_ttlv(|view| super::clone_structure_view(&view))
            else {
                return record_ffi_error(ERROR_INTERNAL); // COVERAGE-EXCLUDE: a successful parsed Get Usage Allocation result always retains its validated TTLV.
            };
            let structure = try_ffi!(structure);
            *response_ttlv = try_ffi!(try_make_handle(
                Kind::Structure,
                HandleValue::Structure(structure),
            ));
            *present = 1;
        }
        SUCCESS
    })
}

/// Reads the successful Obtain Lease result from KMIP 2.1 §6.1.35, Table 269.
/// The identifier storage is borrowed from the response item and remains valid
/// while that response handle is alive. Outputs are cleared for non-success items.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_obtain_lease(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_unique_identifier: *mut kmipkit_unique_identifier_v1_t,
    out_lease_time: *mut u32,
    out_last_change_date: *mut i64,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        let lease_time = scalar_out!(out_lease_time);
        let last_change_date = scalar_out!(out_last_change_date);
        if let Err(status) = write_unique_identifier_v1(out_unique_identifier, None) {
            return record_ffi_error(status);
        }
        *present = 0;
        *lease_time = 0;
        *last_change_date = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        let outcome = item.outcome();
        if outcome.result().status().raw() != 0 {
            return SUCCESS;
        }
        if let kmipkit_client::ClientBatchOutcome::ObtainLease(lease) = outcome {
            let (Some(identifier), Some(interval), Some(last_change)) = (
                lease.unique_identifier(),
                lease.lease_time(),
                lease.last_change_date(),
            ) else {
                return record_ffi_error(ERROR_INTERNAL);
            };
            if let Err(status) = write_unique_identifier_v1(out_unique_identifier, Some(identifier))
            {
                return record_ffi_error(status);
            }
            *lease_time = interval;
            *last_change_date = last_change;
            *present = 1;
        }
        SUCCESS
    })
}

/// Copies the complete generic response only from a successful Obtain Lease result.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_obtain_lease_ttlv(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_response_ttlv: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        let response_ttlv = out_slot!(out_response_ttlv);
        *present = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        let outcome = item.outcome();
        if outcome.result().status().raw() != 0 {
            return SUCCESS;
        }
        if let kmipkit_client::ClientBatchOutcome::ObtainLease(lease) = outcome {
            let structure = try_ffi!(require_success_response_ttlv(
                lease.with_ttlv(|view| super::clone_structure_view(&view))
            ));
            *response_ttlv = try_ffi!(try_make_handle(
                Kind::Structure,
                HandleValue::Structure(structure),
            ));
            *present = 1;
        }
        SUCCESS
    })
}

/// Borrows the resulting identifier from a successful Import response item.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_import(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_unique_identifier: *mut kmipkit_unique_identifier_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        if let Err(status) = write_unique_identifier_v1(out_unique_identifier, None) {
            return record_ffi_error(status);
        }
        *present = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        let outcome = item.outcome();
        if outcome.result().status().raw() != 0 {
            return SUCCESS;
        }
        if let kmipkit_client::ClientBatchOutcome::Import(import) = outcome {
            let Some(identifier) = import.unique_identifier() else {
                return record_ffi_error(ERROR_INTERNAL); // COVERAGE-EXCLUDE: successful Import responses require a Unique Identifier during protocol parsing.
            };
            if let Err(status) = write_unique_identifier_v1(out_unique_identifier, Some(identifier))
            {
                return record_ffi_error(status); // COVERAGE-EXCLUDE: the same stable output record passed the preflight immediately above.
            }
            *present = 1;
        }
        SUCCESS
    })
}

/// Borrows the joined-object identifier from a successful Join Split Key response.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_join_split_key(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_unique_identifier: *mut kmipkit_unique_identifier_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        if let Err(status) = write_unique_identifier_v1(out_unique_identifier, None) {
            return record_ffi_error(status);
        }
        *present = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        let outcome = item.outcome();
        if outcome.result().status().raw() != 0 {
            return SUCCESS;
        }
        if let Some(joined) = outcome.join_split_key_response() {
            let Some(identifier) = joined.unique_identifier() else {
                return record_ffi_error(ERROR_INTERNAL);
            };
            if let Err(status) = write_unique_identifier_v1(out_unique_identifier, Some(identifier))
            {
                return record_ffi_error(status);
            }
            *present = 1;
        }
        SUCCESS
    })
}

/// Copies the complete ordered response only from a successful Join Split Key result.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_join_split_key_ttlv(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_response_ttlv: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        let response_ttlv = out_slot!(out_response_ttlv);
        *present = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        let outcome = item.outcome();
        if outcome.result().status().raw() != 0 {
            return SUCCESS;
        }
        if let Some(joined) = outcome.join_split_key_response() {
            let structure = try_ffi!(joined.with_ttlv(|view| super::clone_structure_view(&view)));
            *response_ttlv = try_ffi!(try_make_handle(
                Kind::Structure,
                HandleValue::Structure(structure),
            ));
            *present = 1;
        }
        SUCCESS
    })
}

/// Copies the complete ordered response only from a successful Import result.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_import_ttlv(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_response_ttlv: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        let response_ttlv = out_slot!(out_response_ttlv);
        *present = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        let outcome = item.outcome();
        if outcome.result().status().raw() != 0 {
            return SUCCESS;
        }
        if let kmipkit_client::ClientBatchOutcome::Import(import) = outcome {
            let structure = try_ffi!(import.with_ttlv(|view| super::clone_structure_view(&view)));
            *response_ttlv = try_ffi!(try_make_handle(
                Kind::Structure,
                HandleValue::Structure(structure),
            ));
            *present = 1;
        }
        SUCCESS
    })
}

/// Copies the complete generic response only from a successful Get Constraints result.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_get_constraints_ttlv(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_response_ttlv: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        let response_ttlv = out_slot!(out_response_ttlv);
        *present = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        if item.outcome().result().status().raw() != 0 {
            return SUCCESS;
        }
        if let kmipkit_client::ClientBatchOutcome::GetConstraints(constraints) = item.outcome() {
            let structure =
                try_ffi!(constraints.with_ttlv(|view| super::clone_structure_view(&view)));
            *response_ttlv = try_ffi!(try_make_handle(
                Kind::Structure,
                HandleValue::Structure(structure),
            ));
            *present = 1;
        }
        SUCCESS
    })
}

/// Copies the successful Get Attribute List response payload as generic TTLV.
/// Output regions must be writable and must not overlap.
///
/// # Safety
///
/// `item` must be null or a live response-item handle. Both output pointers
/// must be aligned, writable, and non-overlapping for their declared types.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kmipkit_client_batch_item_response_get_attribute_list_payload_ttlv(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_response_ttlv: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        if out_present.is_null()
            || out_response_ttlv.is_null()
            || ffi_output_ranges_overlap(out_present, out_response_ttlv)
        {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let present = scalar_out!(out_present);
        let response_ttlv = out_slot!(out_response_ttlv);
        *present = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        if item.outcome().result().status().raw() != 0 {
            return SUCCESS;
        }
        if let kmipkit_client::ClientBatchOutcome::GetAttributeList(result) = item.outcome() {
            let Some(structure) = try_ffi!(result.to_ttlv_payload().map_err(|_| ERROR_INTERNAL))
            else {
                return record_ffi_error(ERROR_INTERNAL);
            };
            *response_ttlv = try_ffi!(try_make_handle(
                Kind::Structure,
                HandleValue::Structure(structure),
            ));
            *present = 1;
        }
        SUCCESS
    })
}

/// Copies the successful Get Attributes identifier and direct Attributes as
/// caller-owned generic TTLV. Non-success and other-operation results expose
/// neither operation field. Repeated items retain their original order.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_get_attributes(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_unique_identifier: *mut kmipkit_unique_identifier_v1_t,
    out_attributes: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        if out_present.is_null()
            || out_unique_identifier.is_null()
            || out_attributes.is_null()
            || !out_present.is_aligned()
            || !out_unique_identifier.is_aligned()
            || !out_attributes.is_aligned()
            || ffi_output_ranges_overlap(out_present, out_unique_identifier)
            || ffi_output_ranges_overlap(out_present, out_attributes)
            || ffi_output_ranges_overlap(out_unique_identifier, out_attributes)
        {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let present = scalar_out!(out_present);
        let unique_identifier = scalar_out!(out_unique_identifier);
        let attributes_output = out_slot!(out_attributes);
        *present = 0;
        *attributes_output = ptr::null_mut();
        if let Err(status) = write_unique_identifier_v1(unique_identifier, None) {
            return record_ffi_error(status);
        }

        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        let kmipkit_client::ClientBatchOutcome::GetAttributes(result) = item.outcome() else {
            return SUCCESS;
        };
        let Some(value) = result.unique_identifier() else {
            return SUCCESS;
        };

        let attributes_handle: *mut kmipkit_ttlv_structure_t =
            if let Some(attribute_set) = result.attributes() {
                let mut attributes = Structure::new();
                for attribute in attribute_set.as_items() {
                    let copy = match attribute.try_clone() {
                        Ok(copy) => copy,
                        Err(error) => return record_ffi_error(model_status(error)),
                    };
                    if let Err(error) = attributes.try_push(copy) {
                        return record_ffi_error(model_status(error));
                    }
                }
                match try_make_handle(Kind::Structure, HandleValue::Structure(attributes)) {
                    Ok(handle) => handle,
                    Err(status) => return record_ffi_error(status),
                }
            } else {
                ptr::null_mut()
            };
        if let Err(status) = write_unique_identifier_text_v1(unique_identifier, Some(value)) {
            if !attributes_handle.is_null() {
                release(attributes_handle, Kind::Structure);
            }
            return record_ffi_error(status);
        }
        *attributes_output = attributes_handle;
        *present = 1;
        SUCCESS
    })
}

#[derive(Clone, Copy)]
enum AttributeMutationResponseKind {
    Add,
    Adjust,
    Delete,
    Modify,
    Set,
}

fn attribute_mutation_response_identifier(
    outcome: &kmipkit_client::ClientBatchOutcome,
    expected: AttributeMutationResponseKind,
) -> Option<&str> {
    match (expected, outcome) {
        (
            AttributeMutationResponseKind::Add,
            kmipkit_client::ClientBatchOutcome::AddAttribute(response),
        ) => response.unique_identifier(),
        (
            AttributeMutationResponseKind::Adjust,
            kmipkit_client::ClientBatchOutcome::AdjustAttribute(response),
        ) => response.unique_identifier(),
        (
            AttributeMutationResponseKind::Delete,
            kmipkit_client::ClientBatchOutcome::DeleteAttribute(response),
        ) => response.unique_identifier(),
        (
            AttributeMutationResponseKind::Modify,
            kmipkit_client::ClientBatchOutcome::ModifyAttribute(response),
        ) => response.unique_identifier(),
        (
            AttributeMutationResponseKind::Set,
            kmipkit_client::ClientBatchOutcome::SetAttribute(response),
        ) => response.unique_identifier(),
        _ => None,
    }
}

fn attribute_mutation_response_identifier_impl(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_unique_identifier: *mut kmipkit_unique_identifier_v1_t,
    kind: AttributeMutationResponseKind,
) -> i32 {
    if out_present.is_null()
        || out_unique_identifier.is_null()
        || !out_present.is_aligned()
        || !out_unique_identifier.is_aligned()
        || ffi_output_ranges_overlap(out_present, out_unique_identifier)
    {
        return record_ffi_error(ERROR_INVALID_INPUT);
    }
    let present = scalar_out!(out_present);
    let unique_identifier = scalar_out!(out_unique_identifier);
    *present = 0;
    if let Err(status) = write_unique_identifier_v1(unique_identifier, None) {
        return record_ffi_error(status);
    }

    let response_handle = read_handle!(item, ClientBatchItemResponse);
    let item = try_ffi!(batch_response_item_from_handle(&response_handle));
    let Some(value) = attribute_mutation_response_identifier(item.outcome(), kind) else {
        return SUCCESS;
    };
    if let Err(status) = write_unique_identifier_text_v1(unique_identifier, Some(value)) {
        return record_ffi_error(status);
    }
    *present = 1;
    SUCCESS
}

/// Copies the successful Add Attribute response's required Text String Unique
/// Identifier. The borrowed identifier span remains valid while `item` is open.
/// Non-success and other-operation results set `out_present` to zero.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_add_attribute(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_unique_identifier: *mut kmipkit_unique_identifier_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        attribute_mutation_response_identifier_impl(
            item,
            out_present,
            out_unique_identifier,
            AttributeMutationResponseKind::Add,
        )
    })
}

/// Copies the successful Modify Attribute response's required Text String
/// Unique Identifier. The borrowed text remains valid while `item` is open;
/// non-success and other-operation results set `out_present` to zero.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_modify_attribute(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_unique_identifier: *mut kmipkit_unique_identifier_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        attribute_mutation_response_identifier_impl(
            item,
            out_present,
            out_unique_identifier,
            AttributeMutationResponseKind::Modify,
        )
    })
}

/// Copies the successful Set Attribute response's required Text String Unique
/// Identifier. The span is borrowed while `item` remains open; non-success and
/// other-operation results set `out_present` to zero.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_set_attribute(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_unique_identifier: *mut kmipkit_unique_identifier_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        attribute_mutation_response_identifier_impl(
            item,
            out_present,
            out_unique_identifier,
            AttributeMutationResponseKind::Set,
        )
    })
}

/// Copies the successful Adjust Attribute response's required Text String Unique
/// Identifier. The borrowed identifier span remains valid while `item` is open.
/// Non-success and other-operation results set `out_present` to zero.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_adjust_attribute(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_unique_identifier: *mut kmipkit_unique_identifier_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        attribute_mutation_response_identifier_impl(
            item,
            out_present,
            out_unique_identifier,
            AttributeMutationResponseKind::Adjust,
        )
    })
}

/// Copies the successful Delete Attribute response's required Text String
/// Unique Identifier. The borrowed span remains valid while `item` is open;
/// non-success and other-operation results set `out_present` to zero.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_delete_attribute(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_unique_identifier: *mut kmipkit_unique_identifier_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        attribute_mutation_response_identifier_impl(
            item,
            out_present,
            out_unique_identifier,
            AttributeMutationResponseKind::Delete,
        )
    })
}

/// Copies the successful Get Attribute List UID and ordered typed references.
///
/// Pass a null reference pointer with zero capacity to query the count. Text
/// spans in the returned records are borrowed while `item` remains open.
/// Output records must be initialized with their exact size and ABI version 1.
///
/// # Safety
///
/// `item` must be null or a live response-item handle. The scalar and
/// identifier outputs must be aligned and writable. When `capacity` is
/// nonzero, `out_references` must point to an aligned, writable array of that
/// many initialized v1 records. A null reference pointer is allowed only with
/// zero capacity. The function detects and rejects overlapping output ranges
/// before mutating any output.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kmipkit_client_batch_item_response_get_attribute_list(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_unique_identifier: *mut kmipkit_unique_identifier_v1_t,
    out_references: *mut crate::kmipkit_attribute_reference_v1_t,
    capacity: u64,
    out_count: *mut u64,
) -> i32 {
    ffi_entrypoint(|| {
        if out_present.is_null()
            || out_unique_identifier.is_null()
            || out_count.is_null()
            || !out_present.is_aligned()
            || !out_unique_identifier.is_aligned()
            || !out_count.is_aligned()
            || (out_references.is_null() && capacity != 0)
        {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let Ok(capacity) = usize::try_from(capacity) else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(reference_bytes) =
            capacity.checked_mul(std::mem::size_of::<crate::kmipkit_attribute_reference_v1_t>())
        else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        if !out_references.is_null() && !out_references.is_aligned() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        if ffi_output_ranges_overlap(out_present, out_unique_identifier)
            || ffi_output_ranges_overlap(out_present, out_count)
            || ffi_output_ranges_overlap(out_unique_identifier, out_count)
            || (!out_references.is_null()
                && (raw_output_ranges_overlap(
                    out_references.cast(),
                    reference_bytes,
                    out_present.cast(),
                    std::mem::size_of::<u8>(),
                ) || raw_output_ranges_overlap(
                    out_references.cast(),
                    reference_bytes,
                    out_unique_identifier.cast(),
                    std::mem::size_of::<kmipkit_unique_identifier_v1_t>(),
                ) || raw_output_ranges_overlap(
                    out_references.cast(),
                    reference_bytes,
                    out_count.cast(),
                    std::mem::size_of::<u64>(),
                )))
        {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let present = scalar_out!(out_present);
        let count = scalar_out!(out_count);
        *present = 0;
        *count = 0;
        if let Err(status) = write_unique_identifier_v1(out_unique_identifier, None) {
            return record_ffi_error(status);
        }

        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        let kmipkit_client::ClientBatchOutcome::GetAttributeList(result) = item.outcome() else {
            return SUCCESS;
        };
        let Some(unique_identifier) = result.unique_identifier() else {
            return SUCCESS;
        };
        let Some(references) = result.attribute_references() else {
            return SUCCESS;
        };
        let Ok(reference_count) = u64::try_from(references.len()) else {
            return record_ffi_error(ERROR_RESOURCE_LIMIT);
        };
        *present = 1;
        *count = reference_count;
        if let Err(status) =
            write_unique_identifier_text_v1(out_unique_identifier, Some(unique_identifier))
        {
            return record_ffi_error(status);
        }
        if out_references.is_null() || capacity == 0 {
            return SUCCESS;
        }
        if capacity < references.len() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        for offset in 0..references.len() {
            let output =
                // SAFETY: `out_references` is aligned and caller-owned; capacity was checked
                // against the exact number of initialized response references.
                unsafe { out_references.add(offset) };
            if let Err(status) = validate_attribute_reference_output(output) {
                return record_ffi_error(status);
            }
        }

        for (offset, reference) in references.iter().enumerate() {
            let output =
                // SAFETY: `out_references` is aligned and caller-owned; capacity was checked
                // against the exact number of initialized response references.
                unsafe { out_references.add(offset) };
            if let Err(status) = write_attribute_reference_v1(output, Some(reference)) {
                return record_ffi_error(status); // COVERAGE-EXCLUDE: each output header passed preflight above.
            }
        }
        SUCCESS
    })
}

fn raw_output_ranges_overlap(
    first: *mut u8,
    first_length: usize,
    second: *mut u8,
    second_length: usize,
) -> bool {
    let first_start = first as usize;
    let second_start = second as usize;
    let Some(first_end) = first_start.checked_add(first_length) else {
        return true;
    };
    let Some(second_end) = second_start.checked_add(second_length) else {
        return true;
    };
    first_start < second_end && second_start < first_end
}

fn validate_attribute_reference_output(
    output: *mut crate::kmipkit_attribute_reference_v1_t,
) -> FfiResult<()> {
    if output.is_null() || !output.is_aligned() {
        return Err(ERROR_INVALID_INPUT);
    }
    // SAFETY: `output` is non-null and aligned; the caller initializes the fixed header before
    // calling, and both fields are within the public v1 record.
    let (struct_size, abi_version) = unsafe {
        (
            std::ptr::addr_of!((*output).struct_size).read(),
            std::ptr::addr_of!((*output).abi_version).read(),
        )
    };
    if struct_size as usize != std::mem::size_of::<crate::kmipkit_attribute_reference_v1_t>()
        || abi_version != 1
    {
        return Err(ERROR_INVALID_INPUT);
    }
    Ok(())
}

fn write_attribute_reference_v1(
    output: *mut crate::kmipkit_attribute_reference_v1_t,
    reference: Option<&kmipkit_protocol::AttributeReference>,
) -> FfiResult<()> {
    validate_attribute_reference_output(output)?;
    // SAFETY: `validate_attribute_reference_output` verified the writable record's size/version.
    let struct_size = unsafe { std::ptr::addr_of!((*output).struct_size).read() };
    let mut value = crate::kmipkit_attribute_reference_v1_t {
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
    if let Some(reference) = reference {
        if let Some(tag_value) = reference.tag_value() {
            value.kind = crate::KMIPKIT_ATTRIBUTE_REFERENCE_KIND_TAG;
            value.tag_value = tag_value;
        } else if let Some((vendor_identification, attribute_name)) = reference.name_parts() {
            value.kind = crate::KMIPKIT_ATTRIBUTE_REFERENCE_KIND_NAME;
            value.vendor_identification_data = vendor_identification.as_ptr();
            value.vendor_identification_length =
                u64::try_from(vendor_identification.len()).map_err(|_| ERROR_RESOURCE_LIMIT)?;
            value.attribute_name_data = attribute_name.as_ptr();
            value.attribute_name_length =
                u64::try_from(attribute_name.len()).map_err(|_| ERROR_RESOURCE_LIMIT)?;
        } else {
            return Err(ERROR_INTERNAL);
        }
    }
    // SAFETY: `output` is caller-owned writable memory with the validated v1 layout.
    unsafe { output.write(value) };
    Ok(())
}

/// Copies the complete generic response only from a successful Set Constraints result.
/// Output regions must be writable and must not overlap.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_set_constraints_ttlv(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_response_ttlv: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        if out_present.is_null()
            || out_response_ttlv.is_null()
            || ffi_output_ranges_overlap(out_present, out_response_ttlv)
        {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let present = scalar_out!(out_present);
        let response_ttlv = out_slot!(out_response_ttlv);
        *present = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        if item.outcome().result().status().raw() != 0 {
            return SUCCESS;
        }
        if let kmipkit_client::ClientBatchOutcome::SetConstraints(constraints) = item.outcome() {
            let Some(structure) = constraints.with_ttlv(|view| super::clone_structure_view(&view))
            else {
                return record_ffi_error(ERROR_INTERNAL);
            };
            let structure = try_ffi!(structure);
            *response_ttlv = try_ffi!(try_make_handle(
                Kind::Structure,
                HandleValue::Structure(structure),
            ));
            *present = 1;
        }
        SUCCESS
    })
}

/// Copies the complete generic response only from a successful Set Defaults result.
/// Output regions must be writable and must not overlap.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_set_defaults_ttlv(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_response_ttlv: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        if out_present.is_null()
            || out_response_ttlv.is_null()
            || ffi_output_ranges_overlap(out_present, out_response_ttlv)
        {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let present = scalar_out!(out_present);
        let response_ttlv = out_slot!(out_response_ttlv);
        *present = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        if item.outcome().result().status().raw() != 0 {
            return SUCCESS;
        }
        if let kmipkit_client::ClientBatchOutcome::SetDefaults(defaults) = item.outcome() {
            let Some(structure) = defaults.with_ttlv(|view| super::clone_structure_view(&view))
            else {
                return record_ffi_error(ERROR_INTERNAL); // COVERAGE-EXCLUDE: KMIP validation requires the successful Set Defaults payload and retains its TTLV in the typed outcome.
            };
            let structure = try_ffi!(structure);
            *response_ttlv = try_ffi!(try_make_handle(
                Kind::Structure,
                HandleValue::Structure(structure),
            ));
            *present = 1;
        }
        SUCCESS
    })
}

/// Returns the open §6.1.57 Validity Indicator only for a successful Validate result.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_validate(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_validity_indicator: *mut u32,
) -> i32 {
    ffi_entrypoint(|| {
        if out_present.is_null()
            || out_validity_indicator.is_null()
            || ffi_output_ranges_overlap(out_present, out_validity_indicator)
        {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let present = scalar_out!(out_present);
        let validity_indicator = scalar_out!(out_validity_indicator);
        *present = 0;
        *validity_indicator = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        if item.outcome().result().status().raw() != 0 {
            return SUCCESS;
        }
        if let kmipkit_client::ClientBatchOutcome::Validate(validate) = item.outcome()
            && let Some(indicator) = validate.validity_indicator()
        {
            *validity_indicator = indicator.raw();
            *present = 1;
        }
        SUCCESS
    })
}

/// Copies the complete ordered response TTLV only for successful Validate results.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_validate_ttlv(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_response_ttlv: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        if out_present.is_null()
            || out_response_ttlv.is_null()
            || ffi_output_ranges_overlap(out_present, out_response_ttlv)
        {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let present = scalar_out!(out_present);
        let response_ttlv = out_slot!(out_response_ttlv);
        *present = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        if item.outcome().result().status().raw() != 0 {
            return SUCCESS;
        }
        if let kmipkit_client::ClientBatchOutcome::Validate(validate) = item.outcome() {
            let structure = try_ffi!(validate.with_ttlv(|view| super::clone_structure_view(&view)));
            *response_ttlv = try_ffi!(try_make_handle(
                Kind::Structure,
                HandleValue::Structure(structure),
            ));
            *present = 1;
        }
        SUCCESS
    })
}

/// Borrows the derived object identifier from a successful Derive Key response.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_derive_key(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_unique_identifier: *mut kmipkit_unique_identifier_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        if let Err(status) = write_unique_identifier_v1(out_unique_identifier, None) {
            return record_ffi_error(status);
        }
        *present = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        let outcome = item.outcome();
        if outcome.result().status().raw() != 0 {
            return SUCCESS;
        }
        if let Some(derive_key) = outcome.derive_key_response()
            && let Some(identifier) = derive_key.unique_identifier()
        {
            if let Err(status) = write_unique_identifier_v1(out_unique_identifier, Some(identifier))
            {
                return record_ffi_error(status); // COVERAGE-EXCLUDE: the same stable output record passed the preflight immediately above.
            }
            *present = 1;
        }
        SUCCESS
    })
}

/// Copies the complete generic response only from a successful Derive Key result.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_derive_key_ttlv(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_response_ttlv: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        let response_ttlv = out_slot!(out_response_ttlv);
        *present = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        let outcome = item.outcome();
        if outcome.result().status().raw() != 0 {
            return SUCCESS;
        }
        if let Some(derive_key) = outcome.derive_key_response() {
            let structure =
                try_ffi!(derive_key.with_ttlv(|view| super::clone_structure_view(&view)));
            *response_ttlv = try_ffi!(try_make_handle(
                Kind::Structure,
                HandleValue::Structure(structure),
            ));
            *present = 1;
        }
        SUCCESS
    })
}

/// Copies the typed Export result, retaining unknown Object Type values and exposing opaque TTLV.
///
/// `out_present` is set for every typed Export result, including KMIP Failure. Payload outputs are
/// populated only when the KMIP Result Status is Success. The returned Attributes and Any Object
/// structures are independent, zeroizing copies; Any Object contains its original outer item.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_export(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_payload_present: *mut u8,
    out_object_type: *mut u32,
    out_unique_identifier: *mut kmipkit_unique_identifier_v1_t,
    out_attributes: *mut *mut kmipkit_ttlv_structure_t,
    out_any_object: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        let payload_present = scalar_out!(out_payload_present);
        let object_type = scalar_out!(out_object_type);
        let attributes_out = out_slot!(out_attributes);
        let any_object_out = out_slot!(out_any_object);
        *present = 0;
        *payload_present = 0;
        *object_type = 0;
        if let Err(status) = write_unique_identifier_v1(out_unique_identifier, None) {
            return record_ffi_error(status);
        }

        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        let Some(export) = item.outcome().export_response() else {
            return SUCCESS;
        };
        *present = 1;
        if export.result().status().raw() != 0 {
            return SUCCESS;
        }

        let (Some(raw_object_type), Some(identifier), Some(attributes), Some(any_object)) = (
            export.object_type(),
            export.unique_identifier(),
            export.attributes(),
            export.object(),
        ) else {
            return record_ffi_error(ERROR_INTERNAL); // COVERAGE-EXCLUDE: successful Export responses contain all required fields after protocol parsing.
        };
        let attributes = try_ffi!(attributes.with_value(|value| match value {
            ValueView::Structure(structure) => super::clone_structure_view(&structure),
            _ => Err(ERROR_INTERNAL), // COVERAGE-EXCLUDE: Export Attributes are represented as a Structure by the typed response model.
        }));
        let mut any_object_structure = Structure::new();
        let any_object_item = try_ffi!(any_object.try_clone().map_err(model_status));
        if let Err(error) = any_object_structure.try_push(any_object_item) {
            return record_ffi_error(model_status(error)); // COVERAGE-EXCLUDE: response depth keeps Any Object below the 64-level model cap; allocation failure has no deterministic injection hook.
        }
        let attributes_handle =
            match try_make_handle(Kind::Structure, HandleValue::Structure(attributes)) {
                Ok(handle) => handle,
                Err(status) => return record_ffi_error(status), // COVERAGE-EXCLUDE: handle allocation failure and registry exhaustion have no deterministic injection path.
            };
        let any_object_handle = match try_make_handle(
            Kind::Structure,
            HandleValue::Structure(any_object_structure),
        ) {
            Ok(handle) => handle,
            Err(status) => {
                release(attributes_handle, Kind::Structure); // COVERAGE-EXCLUDE: the second handle allocation failure is not injectible after the first succeeds.
                return record_ffi_error(status); // COVERAGE-EXCLUDE: handle allocation failure and registry exhaustion have no deterministic injection path.
            }
        };
        if let Err(status) = write_unique_identifier_v1(out_unique_identifier, Some(identifier)) {
            release(attributes_handle, Kind::Structure); // COVERAGE-EXCLUDE: the same stable output record passed the preflight above.
            release(any_object_handle, Kind::Structure); // COVERAGE-EXCLUDE: the same stable output record passed the preflight above.
            return record_ffi_error(status); // COVERAGE-EXCLUDE: the same stable output record passed the preflight above.
        }
        *object_type = raw_object_type.raw();
        *attributes_out = attributes_handle;
        *any_object_out = any_object_handle;
        *payload_present = 1;
        SUCCESS
    })
}

/// Returns an independent copy of the complete ordered Export response TTLV, including failures.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_export_ttlv(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_response_ttlv: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        let response_ttlv = out_slot!(out_response_ttlv);
        *present = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        if let Some(export) = item.outcome().export_response() {
            let structure = try_ffi!(
                export
                    .with_ttlv(|view| view.try_clone())
                    .map_err(model_status)
            );
            *response_ttlv = try_ffi!(try_make_handle(
                Kind::Structure,
                HandleValue::Structure(structure),
            ));
            *present = 1;
        }
        SUCCESS
    })
}

/// Copies the typed Get response fields, including a zeroizing Any Object copy on Success.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_get(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_payload_present: *mut u8,
    out_object_type: *mut u32,
    out_unique_identifier: *mut kmipkit_unique_identifier_v1_t,
    out_any_object: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        let payload_present = scalar_out!(out_payload_present);
        let object_type = scalar_out!(out_object_type);
        let any_object_out = out_slot!(out_any_object);
        *present = 0;
        *payload_present = 0;
        *object_type = 0;
        if let Err(status) = write_unique_identifier_v1(out_unique_identifier, None) {
            return record_ffi_error(status);
        }

        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        let Some(get) = item.outcome().get_response() else {
            return SUCCESS;
        };
        *present = 1;
        if get.result().status().raw() != 0 {
            return SUCCESS;
        }

        let (Some(raw_object_type), Some(identifier), Some(any_object)) =
            (get.object_type(), get.unique_identifier(), get.object())
        else {
            return record_ffi_error(ERROR_INTERNAL); // COVERAGE-EXCLUDE: successful Get responses contain all required fields after protocol parsing.
        };
        let any_object = try_ffi!(any_object.try_clone().map_err(model_status));
        let mut any_object_structure = Structure::new();
        if let Err(error) = any_object_structure.try_push(any_object) {
            return record_ffi_error(model_status(error)); // COVERAGE-EXCLUDE: response depth keeps Any Object below the 64-level model cap; allocation failure has no deterministic injection hook.
        }
        let any_object_handle = match try_make_handle(
            Kind::Structure,
            HandleValue::Structure(any_object_structure),
        ) {
            Ok(handle) => handle,
            Err(status) => return record_ffi_error(status), // COVERAGE-EXCLUDE: handle allocation failure and registry exhaustion have no deterministic injection path.
        };
        if let Err(status) = write_unique_identifier_v1(out_unique_identifier, Some(identifier)) {
            release(any_object_handle, Kind::Structure); // COVERAGE-EXCLUDE: the same stable output record passed the preflight above.
            return record_ffi_error(status); // COVERAGE-EXCLUDE: the same stable output record passed the preflight above.
        }
        *object_type = raw_object_type.raw();
        *any_object_out = any_object_handle;
        *payload_present = 1;
        SUCCESS
    })
}

/// Copies the completed Locate response's optional count and ordered identifier count.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_locate(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_payload_present: *mut u8,
    out_located_items_present: *mut u8,
    out_located_items: *mut i32,
    out_unique_identifier_count: *mut u64,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        let payload_present = scalar_out!(out_payload_present);
        let located_items_present = scalar_out!(out_located_items_present);
        let located_items = scalar_out!(out_located_items);
        let identifier_count = scalar_out!(out_unique_identifier_count);
        *present = 0;
        *payload_present = 0;
        *located_items_present = 0;
        *located_items = 0;
        *identifier_count = 0;

        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        let Some(locate) = item.outcome().locate_response() else {
            return SUCCESS;
        };
        *present = 1;
        if locate.result().status().raw() != 0 {
            return SUCCESS;
        }
        *payload_present = 1;
        if let Some(value) = locate.located_items() {
            *located_items_present = 1;
            *located_items = value;
        }
        *identifier_count = // COVERAGE-EXCLUDE: LLVM records the macro invocation's hit on its evaluated expression.
            try_ffi!(u64::try_from(locate.unique_identifiers().len()).map_err(|_| ERROR_RESOURCE_LIMIT));
        SUCCESS
    })
}

/// Borrows one Locate response Unique Identifier in its original server order.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_locate_unique_identifier_at(
    item: *mut kmipkit_client_batch_item_response_t,
    index: u64,
    out_unique_identifier: *mut kmipkit_unique_identifier_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        if let Err(status) = write_unique_identifier_v1(out_unique_identifier, None) {
            return record_ffi_error(status);
        }
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse {
            response,
            index: item_index,
        } = &item.value
        else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*item_index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        let Some(locate) = item.outcome().locate_response() else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(identifier) = usize::try_from(index)
            .ok()
            .and_then(|index| locate.unique_identifiers().get(index))
        else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        match write_unique_identifier_v1(out_unique_identifier, Some(identifier)) {
            Ok(()) => SUCCESS,
            Err(status) => record_ffi_error(status), // COVERAGE-EXCLUDE: the same stable output record passed the preflight immediately above.
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_create_key_pair(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_private_key_unique_identifier: *mut kmipkit_unique_identifier_v1_t,
    out_public_key_unique_identifier: *mut kmipkit_unique_identifier_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        if let Err(status) = write_unique_identifier_v1(out_private_key_unique_identifier, None) {
            return record_ffi_error(status);
        }
        if let Err(status) = write_unique_identifier_v1(out_public_key_unique_identifier, None) {
            return record_ffi_error(status);
        }
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        *present = 0;
        if let Some(create) = item.outcome().create_key_pair_response()
            && let (Some(private_identifier), Some(public_identifier)) = (
                create.private_key_unique_identifier(),
                create.public_key_unique_identifier(),
            )
        {
            if let Err(status) = write_unique_identifier_v1(
                out_private_key_unique_identifier,
                Some(private_identifier),
            ) {
                return record_ffi_error(status); // COVERAGE-EXCLUDE: the same stable output record passed the preflight immediately above.
            }
            if let Err(status) = write_unique_identifier_v1(
                out_public_key_unique_identifier,
                Some(public_identifier),
            ) {
                return record_ffi_error(status); // COVERAGE-EXCLUDE: the same stable output record passed the preflight immediately above.
            }
            *present = 1;
        }
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_create_split_key_count(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_count: *mut u64,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        let count = scalar_out!(out_count);
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        let response = item.outcome().response();
        *present = 0;
        *count = 0;
        if let Some(create) = response.create_split_key() {
            let identifiers = create.unique_identifiers();
            if !identifiers.is_empty() {
                *present = 1;
                *count =
                    try_ffi!(u64::try_from(identifiers.len()).map_err(|_| ERROR_RESOURCE_LIMIT));
            }
        }
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_create_split_key_unique_identifier_at(
    item: *mut kmipkit_client_batch_item_response_t,
    index: u64,
    out_identifier: *mut kmipkit_unique_identifier_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        if let Err(status) = write_unique_identifier_v1(out_identifier, None) {
            return record_ffi_error(status);
        }
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse {
            response,
            index: item_index,
        } = &item.value
        else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*item_index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        let response = item.outcome().response();
        let Some(create) = response.create_split_key() else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(identifier) = usize::try_from(index)
            .ok()
            .and_then(|index| create.unique_identifiers().get(index))
        else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        match write_unique_identifier_v1(out_identifier, Some(identifier)) {
            Ok(()) => SUCCESS,
            Err(status) => record_ffi_error(status), // COVERAGE-EXCLUDE: the same stable output record passed the preflight immediately above.
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_unique_batch_item_id(
    item: *mut kmipkit_client_batch_item_t,
    out_present: *mut u8,
    out_value: *mut crate::kmipkit_byte_span_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        let value = scalar_out!(out_value);
        *present = 0;
        *value = crate::kmipkit_byte_span_v1_t {
            data: ptr::null(),
            length: 0,
        };
        let item_handle = read_handle!(item, BatchItem);
        let unique_batch_item_id = match &item_handle.value {
            HandleValue::BatchItem(item) => item.unique_batch_item_id(),
            HandleValue::BatchItemReadback(item) if item.owner_index_is_valid() => {
                item.metadata.unique_batch_item_id.as_deref()
            }
            HandleValue::BatchItemReadback(_) => return record_ffi_error(ERROR_INTERNAL),
            _ => return record_ffi_error(ERROR_INVALID_INPUT),
        };
        if let Some(bytes) = unique_batch_item_id {
            *present = 1;
            *value = crate::kmipkit_byte_span_v1_t {
                data: bytes.as_ptr(),
                length: u64::try_from(bytes.len()).unwrap_or(u64::MAX),
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
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
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
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
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

fn set_borrowed_span(
    present: &mut u8,
    output: &mut crate::kmipkit_byte_span_v1_t,
    bytes: Option<&[u8]>,
) -> FfiResult<()> {
    if let Some(bytes) = bytes {
        *present = 1;
        *output = crate::kmipkit_byte_span_v1_t {
            data: if bytes.is_empty() {
                ptr::null()
            } else {
                bytes.as_ptr()
            },
            length: u64::try_from(bytes.len()).map_err(|_| ERROR_INTERNAL)?,
        };
    } else {
        *present = 0;
        *output = crate::kmipkit_byte_span_v1_t {
            data: ptr::null(),
            length: 0,
        };
    }
    Ok(())
}

/// Reads the typed Hash result fields. Returned byte spans are borrowed from `item`.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_hash(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_data_present: *mut u8,
    out_data: *mut crate::kmipkit_byte_span_v1_t,
    out_correlation_present: *mut u8,
    out_correlation_value: *mut crate::kmipkit_byte_span_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        let data_present = scalar_out!(out_data_present);
        let data = scalar_out!(out_data);
        let correlation_present = scalar_out!(out_correlation_present);
        let correlation = scalar_out!(out_correlation_value);
        *present = 0;
        try_ffi!(set_borrowed_span(data_present, data, None));
        try_ffi!(set_borrowed_span(correlation_present, correlation, None));
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        if item.outcome().result().status().raw() != 0 {
            return SUCCESS;
        }
        if let Some(value) = item.outcome().hash_response() {
            *present = 1;
            if let Some(bytes) = value.data() {
                let status =
                    bytes.with_bytes(|bytes| set_borrowed_span(data_present, data, Some(bytes)));
                if let Err(status) = status {
                    return record_ffi_error(status); // COVERAGE-EXCLUDE: usize lengths fit u64 on all supported 64-bit targets.
                }
            }
            if let Some(bytes) = value.correlation_value() {
                let status = bytes.with_bytes(|bytes| {
                    set_borrowed_span(correlation_present, correlation, Some(bytes))
                });
                if let Err(status) = status {
                    return record_ffi_error(status); // COVERAGE-EXCLUDE: usize lengths fit u64 on supported targets.
                }
            }
        }
        SUCCESS
    })
}

/// Reads typed Encrypt result fields only for a successful result.
/// Returned spans are borrowed from `item` and remain valid while it is alive.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_encrypt(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_unique_identifier: *mut kmipkit_unique_identifier_v1_t,
    out_data_present: *mut u8,
    out_data: *mut crate::kmipkit_byte_span_v1_t,
    out_iv_counter_nonce_present: *mut u8,
    out_iv_counter_nonce: *mut crate::kmipkit_byte_span_v1_t,
    out_correlation_present: *mut u8,
    out_correlation_value: *mut crate::kmipkit_byte_span_v1_t,
    out_authenticated_encryption_tag_present: *mut u8,
    out_authenticated_encryption_tag: *mut crate::kmipkit_byte_span_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        let data_present = scalar_out!(out_data_present);
        let data = scalar_out!(out_data);
        let iv_present = scalar_out!(out_iv_counter_nonce_present);
        let iv = scalar_out!(out_iv_counter_nonce);
        let correlation_present = scalar_out!(out_correlation_present);
        let correlation = scalar_out!(out_correlation_value);
        let tag_present = scalar_out!(out_authenticated_encryption_tag_present);
        let tag = scalar_out!(out_authenticated_encryption_tag);
        *present = 0;
        try_ffi!(set_borrowed_span(data_present, data, None));
        try_ffi!(set_borrowed_span(iv_present, iv, None));
        try_ffi!(set_borrowed_span(correlation_present, correlation, None));
        try_ffi!(set_borrowed_span(tag_present, tag, None));
        if let Err(status) = write_unique_identifier_v1(out_unique_identifier, None) {
            return record_ffi_error(status);
        }
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        if item.outcome().result().status().raw() != 0 {
            return SUCCESS;
        }
        if let Some(value) = item.outcome().encrypt_response() {
            *present = 1;
            if let Err(status) =
                write_unique_identifier_v1(out_unique_identifier, value.unique_identifier())
            {
                return record_ffi_error(status);
            }
            if let Some(bytes) = value.data()
                && let Err(status) =
                    bytes.with_bytes(|bytes| set_borrowed_span(data_present, data, Some(bytes)))
            {
                return record_ffi_error(status); // COVERAGE-EXCLUDE: usize lengths fit u64 on supported targets.
            }
            if let Some(bytes) = value.iv_counter_nonce()
                && let Err(status) =
                    bytes.with_bytes(|bytes| set_borrowed_span(iv_present, iv, Some(bytes)))
            {
                return record_ffi_error(status); // COVERAGE-EXCLUDE: usize lengths fit u64 on supported targets.
            }
            if let Some(bytes) = value.correlation_value()
                && let Err(status) = bytes.with_bytes(|bytes| {
                    set_borrowed_span(correlation_present, correlation, Some(bytes))
                })
            {
                return record_ffi_error(status); // COVERAGE-EXCLUDE: usize lengths fit u64 on supported targets.
            }
            if let Some(bytes) = value.authenticated_encryption_tag()
                && let Err(status) =
                    bytes.with_bytes(|bytes| set_borrowed_span(tag_present, tag, Some(bytes)))
            {
                return record_ffi_error(status); // COVERAGE-EXCLUDE: usize lengths fit u64 on supported targets.
            }
        }
        SUCCESS
    })
}

/// Reads typed Decrypt result fields only for a successful result.
/// Returned spans are borrowed from `item` and remain valid while it is alive.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_decrypt(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_unique_identifier: *mut kmipkit_unique_identifier_v1_t,
    out_data_present: *mut u8,
    out_data: *mut crate::kmipkit_byte_span_v1_t,
    out_correlation_present: *mut u8,
    out_correlation_value: *mut crate::kmipkit_byte_span_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        let data_present = scalar_out!(out_data_present);
        let data = scalar_out!(out_data);
        let correlation_present = scalar_out!(out_correlation_present);
        let correlation = scalar_out!(out_correlation_value);
        *present = 0;
        try_ffi!(set_borrowed_span(data_present, data, None));
        try_ffi!(set_borrowed_span(correlation_present, correlation, None));
        if let Err(status) = write_unique_identifier_v1(out_unique_identifier, None) {
            return record_ffi_error(status);
        }
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        if item.outcome().result().status().raw() != 0 {
            return SUCCESS;
        }
        if let Some(value) = item.outcome().decrypt_response() {
            *present = 1;
            if let Err(status) =
                write_unique_identifier_v1(out_unique_identifier, value.unique_identifier())
            {
                return record_ffi_error(status);
            }
            if let Some(bytes) = value.data()
                && let Err(status) =
                    bytes.with_bytes(|bytes| set_borrowed_span(data_present, data, Some(bytes)))
            {
                return record_ffi_error(status); // COVERAGE-EXCLUDE: usize lengths fit u64 on supported targets.
            }
            if let Some(bytes) = value.correlation_value()
                && let Err(status) = bytes.with_bytes(|bytes| {
                    set_borrowed_span(correlation_present, correlation, Some(bytes))
                })
            {
                return record_ffi_error(status); // COVERAGE-EXCLUDE: usize lengths fit u64 on supported targets.
            }
        }
        SUCCESS
    })
}

/// Reads the typed MAC result fields. Returned byte spans are borrowed from `item`.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_mac(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_unique_identifier: *mut kmipkit_unique_identifier_v1_t,
    out_mac_data_present: *mut u8,
    out_mac_data: *mut crate::kmipkit_byte_span_v1_t,
    out_correlation_present: *mut u8,
    out_correlation_value: *mut crate::kmipkit_byte_span_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        let mac_present = scalar_out!(out_mac_data_present);
        let mac_data = scalar_out!(out_mac_data);
        let correlation_present = scalar_out!(out_correlation_present);
        let correlation = scalar_out!(out_correlation_value);
        *present = 0;
        try_ffi!(set_borrowed_span(mac_present, mac_data, None));
        try_ffi!(set_borrowed_span(correlation_present, correlation, None));
        if let Err(status) = write_unique_identifier_v1(out_unique_identifier, None) {
            return record_ffi_error(status);
        }
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        if item.outcome().result().status().raw() != 0 {
            return SUCCESS;
        }
        if let Some(value) = item.outcome().mac_response() {
            *present = 1;
            if let Err(status) = // COVERAGE-EXCLUDE: output record passed preflight.
                write_unique_identifier_v1(out_unique_identifier, value.unique_identifier())
            {
                return record_ffi_error(status); // COVERAGE-EXCLUDE: the same stable output record passed the preflight above.
            }
            if let Some(bytes) = value.mac_data() {
                let status =
                    bytes.with_bytes(|bytes| set_borrowed_span(mac_present, mac_data, Some(bytes)));
                if let Err(status) = status {
                    return record_ffi_error(status); // COVERAGE-EXCLUDE: usize lengths fit u64 on supported targets.
                }
            }
            if let Some(bytes) = value.correlation_value() {
                let status = bytes.with_bytes(|bytes| {
                    set_borrowed_span(correlation_present, correlation, Some(bytes))
                });
                if let Err(status) = status {
                    return record_ffi_error(status); // COVERAGE-EXCLUDE: usize lengths fit u64 on supported targets.
                }
            }
        }
        SUCCESS
    })
}

/// Reads the typed MAC Verify result fields and preserves unknown Validity Indicator values.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_mac_verify(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_unique_identifier: *mut kmipkit_unique_identifier_v1_t,
    out_validity_present: *mut u8,
    out_validity_indicator: *mut u32,
    out_correlation_present: *mut u8,
    out_correlation_value: *mut crate::kmipkit_byte_span_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        let validity_present = scalar_out!(out_validity_present);
        let validity = scalar_out!(out_validity_indicator);
        let correlation_present = scalar_out!(out_correlation_present);
        let correlation = scalar_out!(out_correlation_value);
        *present = 0;
        *validity_present = 0;
        *validity = 0;
        try_ffi!(set_borrowed_span(correlation_present, correlation, None));
        if let Err(status) = write_unique_identifier_v1(out_unique_identifier, None) {
            return record_ffi_error(status);
        }
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        if item.outcome().result().status().raw() != 0 {
            return SUCCESS;
        }
        if let Some(value) = item.outcome().mac_verify_response() {
            *present = 1;
            if let Err(status) = // COVERAGE-EXCLUDE: output record passed preflight.
                write_unique_identifier_v1(out_unique_identifier, value.unique_identifier())
            {
                return record_ffi_error(status); // COVERAGE-EXCLUDE: the same stable output record passed the preflight above.
            }
            if let Some(indicator) = value.validity_indicator() {
                *validity_present = 1;
                *validity = indicator.raw();
            }
            if let Some(bytes) = value.correlation_value() {
                let status = bytes.with_bytes(|bytes| {
                    set_borrowed_span(correlation_present, correlation, Some(bytes))
                });
                if let Err(status) = status {
                    return record_ffi_error(status); // COVERAGE-EXCLUDE: usize lengths fit u64 on supported targets.
                }
            }
        }
        SUCCESS
    })
}

/// Reads the typed Sign result fields. Returned byte spans are borrowed from `item`.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_sign(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_unique_identifier: *mut kmipkit_unique_identifier_v1_t,
    out_signature_data_present: *mut u8,
    out_signature_data: *mut crate::kmipkit_byte_span_v1_t,
    out_correlation_present: *mut u8,
    out_correlation_value: *mut crate::kmipkit_byte_span_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        let signature_present = scalar_out!(out_signature_data_present);
        let signature_data = scalar_out!(out_signature_data);
        let correlation_present = scalar_out!(out_correlation_present);
        let correlation = scalar_out!(out_correlation_value);
        *present = 0;
        try_ffi!(set_borrowed_span(signature_present, signature_data, None));
        try_ffi!(set_borrowed_span(correlation_present, correlation, None));
        if let Err(status) = write_unique_identifier_v1(out_unique_identifier, None) {
            return record_ffi_error(status);
        }
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        if item.outcome().result().status().raw() != 0 {
            return SUCCESS;
        }
        if let Some(value) = item.outcome().sign_response() {
            *present = 1;
            if let Err(status) = // COVERAGE-EXCLUDE: output record passed preflight.
                write_unique_identifier_v1(out_unique_identifier, value.unique_identifier())
            {
                return record_ffi_error(status); // COVERAGE-EXCLUDE: the same stable output record passed the preflight above.
            }
            if let Some(bytes) = value.signature_data() {
                let status = bytes.with_bytes(|bytes| {
                    set_borrowed_span(signature_present, signature_data, Some(bytes))
                });
                if let Err(status) = status {
                    return record_ffi_error(status); // COVERAGE-EXCLUDE: usize lengths fit u64 on supported targets.
                }
            }
            if let Some(bytes) = value.correlation_value() {
                let status = bytes.with_bytes(|bytes| {
                    set_borrowed_span(correlation_present, correlation, Some(bytes))
                });
                if let Err(status) = status {
                    return record_ffi_error(status); // COVERAGE-EXCLUDE: usize lengths fit u64 on supported targets.
                }
            }
        }
        SUCCESS
    })
}

/// Reads RNG Retrieve output bytes for Success or the typed response for Pending.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_rng_retrieve(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_data_present: *mut u8,
    out_data: *mut crate::kmipkit_byte_span_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        if out_present.is_null()
            || out_data_present.is_null()
            || out_data.is_null()
            || ffi_output_ranges_overlap(out_present, out_data_present)
            || ffi_output_ranges_overlap(out_present, out_data)
            || ffi_output_ranges_overlap(out_data_present, out_data)
        {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let present = scalar_out!(out_present);
        let data_present = scalar_out!(out_data_present);
        let data = scalar_out!(out_data);
        *present = 0;
        try_ffi!(set_borrowed_span(data_present, data, None));
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        if !outcome_has_success_or_pending_response(item.outcome()) {
            return SUCCESS;
        }
        let response_view = item.outcome().response();
        let Some(rng_retrieve) = response_view.rng_retrieve() else {
            return SUCCESS;
        };
        *present = 1;
        if let Some(bytes) = rng_retrieve.data() {
            let status =
                bytes.with_bytes(|bytes| set_borrowed_span(data_present, data, Some(bytes)));
            if let Err(status) = status {
                return record_ffi_error(status); // COVERAGE-EXCLUDE: usize lengths fit u64 on supported targets.
            }
        }
        SUCCESS
    })
}

/// Copies the complete generic TTLV for a successful or Pending RNG Retrieve response.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_rng_retrieve_ttlv(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_response_ttlv: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        if out_present.is_null()
            || out_response_ttlv.is_null()
            || ffi_output_ranges_overlap(out_present, out_response_ttlv)
        {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let present = scalar_out!(out_present);
        let response_ttlv = out_slot!(out_response_ttlv);
        *present = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        if !outcome_has_success_or_pending_response(item.outcome()) {
            return SUCCESS;
        }
        let response_view = item.outcome().response();
        let Some(rng_retrieve) = response_view.rng_retrieve() else {
            return SUCCESS;
        };
        let structure = try_ffi!(rng_retrieve.with_ttlv(|view| super::clone_structure_view(&view)));
        *response_ttlv = try_ffi!(try_make_handle(
            Kind::Structure,
            HandleValue::Structure(structure)
        ));
        *present = 1;
        SUCCESS
    })
}

/// Reads RNG Seed's server-reported Data Length for Success or its typed Pending response.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_rng_seed(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_data_length_present: *mut u8,
    out_data_length: *mut i32,
) -> i32 {
    ffi_entrypoint(|| {
        if out_present.is_null()
            || out_data_length_present.is_null()
            || out_data_length.is_null()
            || ffi_output_ranges_overlap(out_present, out_data_length_present)
            || ffi_output_ranges_overlap(out_present, out_data_length)
            || ffi_output_ranges_overlap(out_data_length_present, out_data_length)
        {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let present = scalar_out!(out_present);
        let data_length_present = scalar_out!(out_data_length_present);
        let data_length = scalar_out!(out_data_length);
        *present = 0;
        *data_length_present = 0;
        *data_length = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        if !outcome_has_success_or_pending_response(item.outcome()) {
            return SUCCESS;
        }
        let response_view = item.outcome().response();
        let Some(rng_seed) = response_view.rng_seed() else {
            return SUCCESS;
        };
        *present = 1;
        if let Some(value) = rng_seed.data_length() {
            *data_length = value;
            *data_length_present = 1;
        }
        SUCCESS
    })
}

/// Copies the complete generic TTLV for a successful or Pending RNG Seed response.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_rng_seed_ttlv(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_response_ttlv: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        if out_present.is_null()
            || out_response_ttlv.is_null()
            || ffi_output_ranges_overlap(out_present, out_response_ttlv)
        {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let present = scalar_out!(out_present);
        let response_ttlv = out_slot!(out_response_ttlv);
        *present = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        if !outcome_has_success_or_pending_response(item.outcome()) {
            return SUCCESS;
        }
        let response_view = item.outcome().response();
        let Some(rng_seed) = response_view.rng_seed() else {
            return SUCCESS;
        };
        let structure = try_ffi!(rng_seed.with_ttlv(|view| super::clone_structure_view(&view)));
        *response_ttlv = try_ffi!(try_make_handle(
            Kind::Structure,
            HandleValue::Structure(structure)
        ));
        *present = 1;
        SUCCESS
    })
}

/// Reads the typed Signature Verify result and preserves unknown indicator values.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_signature_verify(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_unique_identifier: *mut kmipkit_unique_identifier_v1_t,
    out_validity_present: *mut u8,
    out_validity_indicator: *mut u32,
    out_recovered_data_present: *mut u8,
    out_recovered_data: *mut crate::kmipkit_byte_span_v1_t,
    out_correlation_present: *mut u8,
    out_correlation_value: *mut crate::kmipkit_byte_span_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        let validity_present = scalar_out!(out_validity_present);
        let validity = scalar_out!(out_validity_indicator);
        let recovered_present = scalar_out!(out_recovered_data_present);
        let recovered = scalar_out!(out_recovered_data);
        let correlation_present = scalar_out!(out_correlation_present);
        let correlation = scalar_out!(out_correlation_value);
        *present = 0;
        *validity_present = 0;
        *validity = 0;
        try_ffi!(set_borrowed_span(recovered_present, recovered, None));
        try_ffi!(set_borrowed_span(correlation_present, correlation, None));
        if let Err(status) = write_unique_identifier_v1(out_unique_identifier, None) {
            return record_ffi_error(status);
        }
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        if item.outcome().result().status().raw() != 0 {
            return SUCCESS;
        }
        if let Some(value) = item.outcome().signature_verify_response() {
            *present = 1;
            if let Err(status) = // COVERAGE-EXCLUDE: output record passed preflight.
                write_unique_identifier_v1(out_unique_identifier, value.unique_identifier())
            {
                return record_ffi_error(status); // COVERAGE-EXCLUDE: the same stable output record passed the preflight above.
            }
            if let Some(indicator) = value.validity_indicator() {
                *validity_present = 1;
                *validity = indicator.raw();
            }
            if let Some(bytes) = value.recovered_data() {
                let status = bytes.with_bytes(|bytes| {
                    set_borrowed_span(recovered_present, recovered, Some(bytes))
                });
                if let Err(status) = status {
                    return record_ffi_error(status); // COVERAGE-EXCLUDE: usize lengths fit u64 on supported targets.
                }
            }
            if let Some(bytes) = value.correlation_value() {
                let status = bytes.with_bytes(|bytes| {
                    set_borrowed_span(correlation_present, correlation, Some(bytes))
                });
                if let Err(status) = status {
                    return record_ffi_error(status); // COVERAGE-EXCLUDE: usize lengths fit u64 on supported targets.
                }
            }
        }
        SUCCESS
    })
}

/// Copies the complete ordered TTLV batch item from a successful Hash response.
///
/// Failed, pending, and other-operation results leave both outputs empty.
/// The caller owns the copied structure and must release it with
/// `kmipkit_ttlv_structure_release`.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_hash_ttlv(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_response_ttlv: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        if out_present.is_null()
            || out_response_ttlv.is_null()
            || ffi_output_ranges_overlap(out_present, out_response_ttlv)
        {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let present = scalar_out!(out_present);
        let response_ttlv = out_slot!(out_response_ttlv);
        *present = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        let outcome = item.outcome();
        if outcome.result().status().raw() != 0 {
            return SUCCESS;
        }
        if let Some(response) = outcome.hash_response() {
            let structure = try_ffi!(response.with_ttlv(|view| super::clone_structure_view(&view)));
            *response_ttlv = try_ffi!(try_make_handle(
                Kind::Structure,
                HandleValue::Structure(structure),
            ));
            *present = 1;
        }
        SUCCESS
    })
}

/// Copies the complete ordered TTLV batch item from a successful MAC response.
///
/// Failed, pending, and other-operation results leave both outputs empty.
/// The caller owns the copied structure and must release it with
/// `kmipkit_ttlv_structure_release`.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_mac_ttlv(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_response_ttlv: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        if out_present.is_null()
            || out_response_ttlv.is_null()
            || ffi_output_ranges_overlap(out_present, out_response_ttlv)
        {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let present = scalar_out!(out_present);
        let response_ttlv = out_slot!(out_response_ttlv);
        *present = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        let outcome = item.outcome();
        if outcome.result().status().raw() != 0 {
            return SUCCESS;
        }
        if let Some(response) = outcome.mac_response() {
            let structure = try_ffi!(response.with_ttlv(|view| super::clone_structure_view(&view)));
            *response_ttlv = try_ffi!(try_make_handle(
                Kind::Structure,
                HandleValue::Structure(structure),
            ));
            *present = 1;
        }
        SUCCESS
    })
}

/// Copies the complete ordered TTLV batch item from a successful MAC Verify response.
///
/// Failed, pending, and other-operation results leave both outputs empty.
/// The caller owns the copied structure and must release it with
/// `kmipkit_ttlv_structure_release`.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_mac_verify_ttlv(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_response_ttlv: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        if out_present.is_null()
            || out_response_ttlv.is_null()
            || ffi_output_ranges_overlap(out_present, out_response_ttlv)
        {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let present = scalar_out!(out_present);
        let response_ttlv = out_slot!(out_response_ttlv);
        *present = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        let outcome = item.outcome();
        if outcome.result().status().raw() != 0 {
            return SUCCESS;
        }
        if let Some(response) = outcome.mac_verify_response() {
            let structure = try_ffi!(response.with_ttlv(|view| super::clone_structure_view(&view)));
            *response_ttlv = try_ffi!(try_make_handle(
                Kind::Structure,
                HandleValue::Structure(structure),
            ));
            *present = 1;
        }
        SUCCESS
    })
}

/// Copies the complete ordered TTLV batch item from a successful Sign response.
///
/// Failed, pending, and other-operation results leave both outputs empty.
/// The caller owns the copied structure and must release it with
/// `kmipkit_ttlv_structure_release`.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_sign_ttlv(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_response_ttlv: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        if out_present.is_null()
            || out_response_ttlv.is_null()
            || ffi_output_ranges_overlap(out_present, out_response_ttlv)
        {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let present = scalar_out!(out_present);
        let response_ttlv = out_slot!(out_response_ttlv);
        *present = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        let outcome = item.outcome();
        if outcome.result().status().raw() != 0 {
            return SUCCESS;
        }
        if let Some(response) = outcome.sign_response() {
            let structure = try_ffi!(response.with_ttlv(|view| super::clone_structure_view(&view)));
            *response_ttlv = try_ffi!(try_make_handle(
                Kind::Structure,
                HandleValue::Structure(structure),
            ));
            *present = 1;
        }
        SUCCESS
    })
}

/// Copies the complete ordered TTLV batch item from a successful Signature Verify response.
///
/// Failed, pending, and other-operation results leave both outputs empty.
/// The caller owns the copied structure and must release it with
/// `kmipkit_ttlv_structure_release`.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_signature_verify_ttlv(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_response_ttlv: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        if out_present.is_null()
            || out_response_ttlv.is_null()
            || ffi_output_ranges_overlap(out_present, out_response_ttlv)
        {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let present = scalar_out!(out_present);
        let response_ttlv = out_slot!(out_response_ttlv);
        *present = 0;
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        let outcome = item.outcome();
        if outcome.result().status().raw() != 0 {
            return SUCCESS;
        }
        if let Some(response) = outcome.signature_verify_response() {
            let structure = try_ffi!(response.with_ttlv(|view| super::clone_structure_view(&view)));
            *response_ttlv = try_ffi!(try_make_handle(
                Kind::Structure,
                HandleValue::Structure(structure),
            ));
            *present = 1;
        }
        SUCCESS
    })
}

/// Copies the typed supported-version list from a successful Discover Versions result.
///
/// Call with `out_versions == NULL` and `capacity == 0` to query presence and
/// the required count. A present empty successful response sets `out_present`
/// to 1 and `out_count` to 0. Pending, failed, and other-operation results set
/// both outputs to 0. A second call with sufficient caller-owned capacity
/// copies values in source order, retaining repetitions.
///
/// # Safety
///
/// `item` must be null or a live response-item handle. `out_present` and
/// `out_count` must be aligned, writable, and non-overlapping. If
/// `out_versions` is non-null, it must point to an aligned, writable array of
/// `capacity` records, disjoint from both scalar outputs. A null versions
/// pointer is allowed only with zero capacity.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kmipkit_client_batch_item_response_discover_versions_supported_versions(
    item: *mut kmipkit_client_batch_item_response_t,
    out_present: *mut u8,
    out_versions: *mut crate::kmipkit_protocol_version_v1_t,
    capacity: u64,
    out_count: *mut u64,
) -> i32 {
    ffi_entrypoint(|| {
        let present = scalar_out!(out_present);
        let count = scalar_out!(out_count);
        if out_versions.is_null() && capacity != 0 {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let response_handle = read_handle!(item, ClientBatchItemResponse);
        let item = try_ffi!(batch_response_item_from_handle(&response_handle));
        let Some(versions) = item.discover_versions_supported_versions() else {
            *present = 0;
            *count = 0;
            return SUCCESS;
        };
        let Ok(version_count) = u64::try_from(versions.len()) else {
            return record_ffi_error(ERROR_INTERNAL); // COVERAGE-EXCLUDE: usize lengths fit u64 on all supported 64-bit targets.
        };
        *present = 1;
        *count = version_count;
        if out_versions.is_null() {
            return SUCCESS;
        }
        let Ok(capacity) = usize::try_from(capacity) else {
            return record_ffi_error(ERROR_INVALID_INPUT); // COVERAGE-EXCLUDE: the supported CI targets are 64-bit, so u64 fits usize.
        };
        if capacity < versions.len() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        for (offset, version) in versions.iter().enumerate() {
            // SAFETY: the caller supplied a non-null, aligned writable array;
            // capacity was checked against the slice length and `offset` is
            // an in-bounds element index.
            unsafe {
                out_versions
                    .add(offset)
                    .write(crate::kmipkit_protocol_version_v1_t {
                        major: version.major(),
                        minor: version.minor(),
                    });
            }
        }
        SUCCESS
    })
}

#[cfg(test)]
#[path = "../../tests/support/profile_validation_mapping_tests.rs"]
mod profile_validation_mapping_tests;

fn validate_attribute_set_item_limits<'a>(items: impl Iterator<Item = &'a Item>) -> FfiResult<()> {
    let limits = CodecLimits::defaults();
    let max_bytes = u64::try_from(limits.max_message_bytes()).map_err(|_| ERROR_RESOURCE_LIMIT)?;
    let max_elements = u64::try_from(limits.max_elements()).map_err(|_| ERROR_RESOURCE_LIMIT)?;
    let max_depth =
        u64::try_from(limits.max_structure_depth()).map_err(|_| ERROR_RESOURCE_LIMIT)?;
    let mut encoded_size = 8_u64;
    let mut element_count = 1_u64;
    for item in items {
        let (item_size, item_elements, item_depth) = check_item_limits_with_depth(item, &limits)?;
        let structure_depth = item_depth.saturating_add(u64::from(item_depth > 0)).max(1);
        if structure_depth > max_depth {
            return Err(ERROR_RESOURCE_LIMIT);
        }
        encoded_size = encoded_size
            .checked_add(item_size)
            .filter(|size| *size <= max_bytes)
            .ok_or(ERROR_RESOURCE_LIMIT)?;
        element_count = element_count
            .checked_add(item_elements)
            .filter(|count| *count <= max_elements)
            .ok_or(ERROR_RESOURCE_LIMIT)?;
    }
    Ok(())
}

fn validate_attribute_set_append_item(item: &Item) -> FfiResult<()> {
    let mut wrapper = Structure::new();
    wrapper
        .try_push(item.try_clone().map_err(model_status)?)
        .map_err(model_status)?;
    AttributeSet::try_new([item.try_clone().map_err(model_status)?])
        .map_err(|_| ERROR_INVALID_INPUT)?;
    Ok(())
}

/// Copies and validates the ordered direct Items from a generic Structure.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_attribute_set_create(
    items: *mut kmipkit_ttlv_structure_t,
    out_attribute_set: *mut *mut kmipkit_attribute_set_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_attribute_set);
        let source = try_ffi!(copy_required_structure_handle(items));
        try_ffi!(check_structure_limits(&source, &CodecLimits::defaults()));
        let source_view = source.view();
        let children = source_view.children();
        try_ffi!(validate_attribute_set_item_limits(children.iter()));
        let mut owned_items = Vec::new();
        try_ffi!(
            owned_items
                .try_reserve_exact(children.len())
                .map_err(|_| ERROR_RESOURCE_LIMIT)
        );
        for child in children {
            owned_items.push(try_ffi!(child.try_clone().map_err(model_status)));
        }
        let attributes =
            try_ffi!(AttributeSet::try_new(owned_items).map_err(|_| ERROR_INVALID_INPUT));
        *out = try_ffi!(try_make_handle(
            Kind::AttributeSet,
            HandleValue::AttributeSet(Mutex::new(attributes)),
        ));
        SUCCESS
    })
}

/// Copies and validates an Item before appending it; validation failures preserve the set.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_attribute_set_try_push(
    attributes: *mut kmipkit_attribute_set_t,
    item: *mut kmipkit_ttlv_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let handle = try_ffi!(reference_handle(attributes, Kind::AttributeSet));
        let HandleValue::AttributeSet(source) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let item = try_ffi!(copy_required_item_handle(item));
        try_ffi!(validate_attribute_set_append_item(&item));
        let Ok(mut set) = source.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        try_ffi!(validate_attribute_set_item_limits(
            set.as_items().iter().chain(std::iter::once(&item)),
        ));
        if let Err(_error) = set.try_push(item) {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        SUCCESS
    })
}

/// Validates a catalogued typed Item before appending it in source order.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_attribute_set_try_push_typed(
    attributes: *mut kmipkit_attribute_set_t,
    expected_tag: u32,
    item: *mut kmipkit_ttlv_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let handle = try_ffi!(reference_handle(attributes, Kind::AttributeSet));
        let HandleValue::AttributeSet(source) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let item = try_ffi!(copy_required_item_handle(item));
        let item = try_ffi!(typed_attributes::validate_typed_item(expected_tag, item));
        try_ffi!(validate_attribute_set_append_item(&item));
        let Ok(mut set) = source.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        try_ffi!(validate_attribute_set_item_limits(
            set.as_items().iter().chain(std::iter::once(&item)),
        ));
        if let Err(_error) = set.try_push(item) {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        SUCCESS
    })
}

/// Returns the number of occurrences of one catalogued typed attribute.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_attribute_set_typed_count(
    attributes: *mut kmipkit_attribute_set_t,
    expected_tag: u32,
    out_count: *mut u64,
) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_count);
        if !typed_attributes_catalog_generated::is_catalogued_attribute_tag(expected_tag) {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let handle = try_ffi!(reference_handle(attributes, Kind::AttributeSet));
        let HandleValue::AttributeSet(source) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(set) = source.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        *out = try_ffi!(
            u64::try_from(
                set.as_items()
                    .iter()
                    .filter(|item| item.tag().raw() == expected_tag)
                    .count(),
            )
            .map_err(|_| ERROR_RESOURCE_LIMIT)
        );
        SUCCESS
    })
}

/// Returns a caller-owned typed Item copy by occurrence index in insertion order.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_attribute_set_typed_at(
    attributes: *mut kmipkit_attribute_set_t,
    expected_tag: u32,
    occurrence: u64,
    out_item: *mut *mut kmipkit_ttlv_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        if !typed_attributes_catalog_generated::is_catalogued_attribute_tag(expected_tag) {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let handle = try_ffi!(reference_handle(attributes, Kind::AttributeSet));
        let HandleValue::AttributeSet(source) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(set) = source.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        let Ok(index) = usize::try_from(occurrence) else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = set
            .as_items()
            .iter()
            .filter(|item| item.tag().raw() == expected_tag)
            .nth(index)
        else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let item = try_ffi!(item.try_clone().map_err(model_status));
        let item = try_ffi!(typed_attributes::validate_typed_item(expected_tag, item));
        *out = try_ffi!(try_make_handle(Kind::Item, HandleValue::Item(item)));
        SUCCESS
    })
}

/// Returns an owned generic Structure copy without consuming the `AttributeSet`.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_attribute_set_as_items(
    attributes: *mut kmipkit_attribute_set_t,
    out_items: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_items);
        let handle = try_ffi!(reference_handle(attributes, Kind::AttributeSet));
        let HandleValue::AttributeSet(source) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(set) = source.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        try_ffi!(validate_attribute_set_item_limits(set.as_items().iter()));
        let mut structure = Structure::new();
        for item in set.as_items() {
            let copy = try_ffi!(item.try_clone().map_err(model_status));
            if let Err(error) = structure.try_push(copy) {
                return record_ffi_error(model_status(error));
            }
        }
        *out = try_ffi!(try_make_handle(
            Kind::Structure,
            HandleValue::Structure(structure)
        ));
        SUCCESS
    })
}

/// Returns the number of direct Items in the `AttributeSet`.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_attribute_set_len(
    attributes: *mut kmipkit_attribute_set_t,
    out_count: *mut u64,
) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_count);
        let handle = try_ffi!(reference_handle(attributes, Kind::AttributeSet));
        let HandleValue::AttributeSet(source) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(set) = source.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        *out = try_ffi!(u64::try_from(set.len()).map_err(|_| ERROR_RESOURCE_LIMIT));
        SUCCESS
    })
}

/// Returns whether the `AttributeSet` contains no direct Items.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_attribute_set_is_empty(
    attributes: *mut kmipkit_attribute_set_t,
    out_is_empty: *mut u8,
) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_is_empty);
        let handle = try_ffi!(reference_handle(attributes, Kind::AttributeSet));
        let HandleValue::AttributeSet(source) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(set) = source.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        *out = u8::from(set.is_empty());
        SUCCESS
    })
}

/// Consumes the `AttributeSet` and returns its direct Items in original order.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_attribute_set_into_items(
    attributes: *mut kmipkit_attribute_set_t,
    out_items: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_items);
        let owner = try_ffi!(consume_unshared_owner(attributes, Kind::AttributeSet));
        let payload = try_ffi!(unwrap_payload(owner));
        let HandleValue::AttributeSet(source) = payload else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(set) = source.into_inner() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        let mut structure = Structure::new();
        for item in set.into_items() {
            if let Err(error) = structure.try_push(item) {
                return record_ffi_error(model_status(error));
            }
        }
        *out = try_ffi!(try_make_handle(
            Kind::Structure,
            HandleValue::Structure(structure)
        ));
        SUCCESS
    })
}

fn response_view_error_status(error: kmipkit_client::ResponseEncodingError) -> i32 {
    match error {
        kmipkit_client::ResponseEncodingError::InvalidResponseTree => ERROR_INVALID_INPUT,
        kmipkit_client::ResponseEncodingError::LimitExceeded
        | kmipkit_client::ResponseEncodingError::AllocationFailed => ERROR_RESOURCE_LIMIT,
        _ => ERROR_INTERNAL,
    }
}

type ResponseViewRenderer = fn(
    &kmipkit_client::ClientBatchItemResponse,
    &CodecLimits,
) -> Result<Zeroizing<String>, kmipkit_client::ResponseEncodingError>;

fn render_response_view(
    item: *mut kmipkit_client_batch_item_response_t,
    limits: *mut kmipkit_codec_limits_t,
    output: *mut *mut kmipkit_secret_bytes_t,
    render: ResponseViewRenderer,
) -> i32 {
    let out = out_slot!(output);
    let item_handle = read_handle!(item, ClientBatchItemResponse);
    let item = try_ffi!(batch_response_item_from_handle(&item_handle));
    let limits_handle = read_handle!(limits, CodecLimits);
    let HandleValue::CodecLimits(limits) = &limits_handle.value else {
        return record_ffi_error(ERROR_INVALID_INPUT);
    };
    let mut rendered = try_ffi!(render(item, limits).map_err(response_view_error_status));
    // Move the allocation into the established zeroizing owner, without an unowned body copy.
    let bytes = std::mem::take(&mut *rendered).into_bytes();
    *out = try_ffi!(try_make_handle(
        Kind::SecretBytes,
        HandleValue::SecretBytes(SecretBytes::new(bytes)),
    ));
    SUCCESS
}

/// Renders the complete retained Response Message as canonical uppercase TTLV hex under caller limits.
///
/// This inspection-only call returns an independently owned zeroizing UTF-8 byte owner. Use
/// `kmipkit_secret_bytes_with_bytes` for scoped access and `kmipkit_secret_bytes_release` to release
/// it. The output is null on failure; status and diagnostics never include the response body.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_ttlv_hex(
    item: *mut kmipkit_client_batch_item_response_t,
    limits: *mut kmipkit_codec_limits_t,
    out_rendered: *mut *mut kmipkit_secret_bytes_t,
) -> i32 {
    ffi_entrypoint(|| {
        render_response_view(
            item,
            limits,
            out_rendered,
            kmipkit_client::ClientBatchItemResponse::ttlv_hex,
        )
    })
}

/// Renders the complete retained Response Message as KMIP JSON under caller limits.
///
/// This inspection-only call returns an independently owned zeroizing UTF-8 byte owner. Use
/// `kmipkit_secret_bytes_with_bytes` for scoped access and `kmipkit_secret_bytes_release` to release
/// it. The output is null on failure; status and diagnostics never include the response body.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_json(
    item: *mut kmipkit_client_batch_item_response_t,
    limits: *mut kmipkit_codec_limits_t,
    out_rendered: *mut *mut kmipkit_secret_bytes_t,
) -> i32 {
    ffi_entrypoint(|| {
        render_response_view(
            item,
            limits,
            out_rendered,
            kmipkit_client::ClientBatchItemResponse::json,
        )
    })
}

/// Renders the complete retained Response Message as KMIP XML under caller limits.
///
/// This inspection-only call returns an independently owned zeroizing UTF-8 byte owner. Use
/// `kmipkit_secret_bytes_with_bytes` for scoped access and `kmipkit_secret_bytes_release` to release
/// it. The output is null on failure; status and diagnostics never include the response body.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_xml(
    item: *mut kmipkit_client_batch_item_response_t,
    limits: *mut kmipkit_codec_limits_t,
    out_rendered: *mut *mut kmipkit_secret_bytes_t,
) -> i32 {
    ffi_entrypoint(|| {
        render_response_view(
            item,
            limits,
            out_rendered,
            kmipkit_client::ClientBatchItemResponse::xml,
        )
    })
}

fn listener_transport_config_from_v1(
    input: *const kmipkit_listener_transport_config_v1_t,
    options: *const kmipkit_listener_options_v1_t,
) -> FfiResult<kmipkit_transport::ListenerTransportConfig> {
    if input.is_null() {
        return Err(ERROR_INVALID_INPUT);
    }
    // SAFETY: caller guarantees non-null pointer references valid record.
    let input = unsafe { &*input };
    let required_size =
        u32::try_from(std::mem::size_of::<kmipkit_listener_transport_config_v1_t>())
            .map_err(|_| ERROR_INVALID_INPUT)?;
    if input.abi_version != 1
        || input.struct_size < required_size
        || input.reserved != 0
        || input.flags & !KMIPKIT_TRANSPORT_CONFIG_FLAG_TLS12_COMPAT != 0
    {
        return Err(ERROR_INVALID_INPUT);
    }

    let bind_host = if input.bind_host.length == 0 || input.bind_host.data.is_null() {
        "127.0.0.1".to_string()
    } else {
        transport_text(input.bind_host)?
    };

    let server_certificates =
        transport_input_slice(input.server_certificates, input.server_certificate_count)?;
    if server_certificates.is_empty() {
        return Err(ERROR_INVALID_INPUT);
    }
    let identity = ClientIdentity::new(
        client_certificate_chain(server_certificates)?,
        private_key_input(&input.server_private_key)?,
    );

    let mut builder =
        kmipkit_transport::ListenerTransportConfig::builder(bind_host, input.bind_port)
            .listener_identity(identity);
    if input.flags & KMIPKIT_TRANSPORT_CONFIG_FLAG_TLS12_COMPAT != 0 {
        builder = builder.tls_version_policy(kmipkit_transport::TlsVersionPolicy::Tls12And13Compat);
    }

    let trust_certificates =
        transport_input_slice(input.trust_certificates, input.trust_certificate_count)?;
    let trust = if !trust_certificates.is_empty() {
        let materials = certificate_input_materials(trust_certificates)?;
        let authorities = trust_certificates
            .iter()
            .zip(materials)
            .map(|(input, bytes)| match transport_input_encoding(input)? {
                TRANSPORT_ENCODING_PEM => Ok(CertificateInput::from_pem(bytes)),
                TRANSPORT_ENCODING_DER => Ok(CertificateInput::from_der(vec![bytes])),
                _ => Err(ERROR_INVALID_INPUT),
            })
            .collect::<FfiResult<Vec<_>>>()?;
        TrustSource::certificate_authorities(authorities)
    } else {
        TrustSource::platform()
    };
    builder = builder.trust_source(trust);

    let defaults = TimeoutPolicy::default();
    let timeouts = defaults
        .with_write(timeout_limit_v1(&input.write_timeout, defaults.write())?)
        .with_read(timeout_limit_v1(&input.read_timeout, defaults.read())?);
    builder = builder.timeout_policy(timeouts);

    if input.max_request_bytes > 0 {
        let max_bytes =
            usize::try_from(input.max_request_bytes).map_err(|_| ERROR_RESOURCE_LIMIT)?;
        builder = builder.max_request_bytes(max_bytes);
    }

    if !options.is_null() {
        // The caller supplies a readable options record.
        let options = unsafe { &*options };
        if options.struct_size != std::mem::size_of::<kmipkit_listener_options_v1_t>() as u32
            || options.abi_version != 1
            || options.reserved != 0
        {
            return Err(ERROR_INVALID_INPUT);
        }
        if !options.target_path.data.is_null() || options.target_path.length != 0 {
            builder = builder.target_path(transport_text(options.target_path)?);
        }
        let inputs =
            transport_input_slice(options.revocation_lists, options.revocation_list_count)?;
        validate_transport_input_total(inputs)?;
        builder = builder.revocation_lists(
            inputs
                .iter()
                .map(revocation_input)
                .collect::<FfiResult<_>>()?,
        );
        builder = match options.wire_encoding {
            0 => builder,
            1 => builder.wire_encoding(WireEncoding::Ttlv),
            2 => builder.wire_encoding(WireEncoding::Json),
            3 => builder.wire_encoding(WireEncoding::Xml),
            _ => return Err(ERROR_INVALID_INPUT),
        };
    }
    builder.build().map_err(|_| ERROR_INVALID_INPUT)
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_listener_bind_raw_tls(
    config: *const kmipkit_listener_transport_config_v1_t,
    out_listener: *mut *mut kmipkit_client_listener_t,
) -> i32 {
    ffi_entrypoint(|| {
        kmipkit_client_listener_bind_raw_tls_with_options_v1(config, ptr::null(), out_listener)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_listener_bind_raw_tls_with_options_v1(
    config: *const kmipkit_listener_transport_config_v1_t,
    options: *const kmipkit_listener_options_v1_t,
    out_listener: *mut *mut kmipkit_client_listener_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_listener);
        let transport_config = try_ffi!(listener_transport_config_from_v1(config, options));
        match kmipkit_client::ClientListener::bind_raw_tls(transport_config) {
            Ok(listener) => {
                *out = try_ffi!(try_make_handle(
                    Kind::ClientListener,
                    HandleValue::ClientListener(Arc::new(Mutex::new(listener))),
                ));
                SUCCESS
            }
            Err(err) => record_client_error(&err),
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_listener_bind_https(
    config: *const kmipkit_listener_transport_config_v1_t,
    out_listener: *mut *mut kmipkit_client_listener_t,
) -> i32 {
    ffi_entrypoint(|| {
        kmipkit_client_listener_bind_https_with_options_v1(config, ptr::null(), out_listener)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_listener_bind_https_with_options_v1(
    config: *const kmipkit_listener_transport_config_v1_t,
    options: *const kmipkit_listener_options_v1_t,
    out_listener: *mut *mut kmipkit_client_listener_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_listener);
        let transport_config = try_ffi!(listener_transport_config_from_v1(config, options));
        match kmipkit_client::ClientListener::bind_https(transport_config) {
            Ok(listener) => {
                *out = try_ffi!(try_make_handle(
                    Kind::ClientListener,
                    HandleValue::ClientListener(Arc::new(Mutex::new(listener))),
                ));
                SUCCESS
            }
            Err(err) => record_client_error(&err),
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_listener_get_port(
    listener: *mut kmipkit_client_listener_t,
    out_port: *mut u16,
) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_port);
        let listener_handle = read_handle!(listener, ClientListener);
        let HandleValue::ClientListener(listener) = &listener_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(listener) = listener.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        match listener.port() {
            Ok(port) => {
                *out = port;
                SUCCESS
            }
            Err(_) => record_ffi_error(ERROR_INTERNAL),
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_listener_accept(
    listener: *mut kmipkit_client_listener_t,
    timeout_ms: u64,
    out_connection: *mut *mut kmipkit_client_listener_connection_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_connection);
        let listener_handle = read_handle!(listener, ClientListener);
        let HandleValue::ClientListener(listener) = &listener_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(mut listener) = listener.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        let timeout = if timeout_ms == 0 {
            None
        } else {
            Some(std::time::Duration::from_millis(timeout_ms))
        };
        match listener.accept(timeout) {
            Ok(conn) => {
                *out = try_ffi!(try_make_handle(
                    Kind::ClientListenerConnection,
                    HandleValue::ClientListenerConnection(Arc::new(Mutex::new(Some(conn)))),
                ));
                SUCCESS
            }
            Err(err) => record_client_error(&err),
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_listener_release(listener: *mut kmipkit_client_listener_t) {
    ffi_entrypoint(|| release(listener, Kind::ClientListener));
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_listener_connection_recv_batch(
    connection: *mut kmipkit_client_listener_connection_t,
    limits: *mut kmipkit_codec_limits_t,
    timeout_ms: u64,
    out_batch: *mut *mut kmipkit_incoming_server_batch_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_batch);
        let conn_handle = read_handle!(connection, ClientListenerConnection);
        let HandleValue::ClientListenerConnection(conn_arc) = &conn_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(mut guard) = conn_arc.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        let conn = match guard.as_mut() {
            Some(c) => c,
            None => return record_ffi_error(ERROR_INVALID_INPUT),
        };
        let limits_handle = read_handle!(limits, CodecLimits);
        let HandleValue::CodecLimits(limits_val) = &limits_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let timeout = if timeout_ms == 0 {
            None
        } else {
            Some(std::time::Duration::from_millis(timeout_ms))
        };
        match conn.recv_batch(limits_val, timeout) {
            Ok(batch) => {
                *out = try_ffi!(try_make_handle(
                    Kind::IncomingServerBatch,
                    HandleValue::IncomingServerBatch(Arc::new(batch)),
                ));
                SUCCESS
            }
            Err(err) => record_client_error(&err),
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_listener_connection_send_response(
    connection: *mut kmipkit_client_listener_connection_t,
    response: *mut kmipkit_response_message_t,
    timeout_ms: u64,
) -> i32 {
    ffi_entrypoint(|| {
        let conn_handle = read_handle!(connection, ClientListenerConnection);
        let HandleValue::ClientListenerConnection(conn_arc) = &conn_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(mut guard) = conn_arc.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        let conn = match guard.as_mut() {
            Some(c) => c,
            None => return record_ffi_error(ERROR_INVALID_INPUT),
        };
        let resp_handle = read_handle!(response, ResponseMessage);
        let HandleValue::ResponseMessage(resp) = &resp_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let timeout = if timeout_ms == 0 {
            None
        } else {
            Some(std::time::Duration::from_millis(timeout_ms))
        };
        match conn.send_response(resp, timeout) {
            Ok(()) => SUCCESS,
            Err(err) => record_client_error(&err),
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_listener_connection_into_role_reversed(
    connection: *mut kmipkit_client_listener_connection_t,
    out_session: *mut *mut kmipkit_role_reversed_session_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_session);
        let conn_handle = read_handle!(connection, ClientListenerConnection);
        let HandleValue::ClientListenerConnection(conn_arc) = &conn_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(mut guard) = conn_arc.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        let conn = match guard.take() {
            Some(c) => c,
            None => return record_ffi_error(ERROR_INVALID_INPUT),
        };
        match conn.into_role_reversed() {
            Ok(session) => {
                *out = try_ffi!(try_make_handle(
                    Kind::RoleReversedSession,
                    HandleValue::RoleReversedSession(Arc::new(Mutex::new(session))),
                ));
                SUCCESS
            }
            Err(err) => record_client_error(&err),
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_listener_connection_close(
    connection: *mut kmipkit_client_listener_connection_t,
) -> i32 {
    ffi_entrypoint(|| {
        let conn_handle = read_handle!(connection, ClientListenerConnection);
        let HandleValue::ClientListenerConnection(conn_arc) = &conn_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(mut guard) = conn_arc.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        if let Some(conn) = guard.as_mut() {
            match conn.close() {
                Ok(()) => SUCCESS,
                Err(err) => record_client_error(&err),
            }
        } else {
            SUCCESS
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_listener_connection_release(
    connection: *mut kmipkit_client_listener_connection_t,
) {
    ffi_entrypoint(|| release(connection, Kind::ClientListenerConnection));
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_role_reversed_session_recv_batch(
    session: *mut kmipkit_role_reversed_session_t,
    limits: *mut kmipkit_codec_limits_t,
    timeout_ms: u64,
    out_batch: *mut *mut kmipkit_incoming_server_batch_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_batch);
        let session_handle = read_handle!(session, RoleReversedSession);
        let HandleValue::RoleReversedSession(session_arc) = &session_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(mut session) = session_arc.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        let limits_handle = read_handle!(limits, CodecLimits);
        let HandleValue::CodecLimits(limits_val) = &limits_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let timeout = if timeout_ms == 0 {
            None
        } else {
            Some(std::time::Duration::from_millis(timeout_ms))
        };
        match session.recv_batch(limits_val, timeout) {
            Ok(batch) => {
                *out = try_ffi!(try_make_handle(
                    Kind::IncomingServerBatch,
                    HandleValue::IncomingServerBatch(Arc::new(batch)),
                ));
                SUCCESS
            }
            Err(err) => record_client_error(&err),
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_role_reversed_session_send_response(
    session: *mut kmipkit_role_reversed_session_t,
    response: *mut kmipkit_response_message_t,
    timeout_ms: u64,
) -> i32 {
    ffi_entrypoint(|| {
        let session_handle = read_handle!(session, RoleReversedSession);
        let HandleValue::RoleReversedSession(session_arc) = &session_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(mut session) = session_arc.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        let resp_handle = read_handle!(response, ResponseMessage);
        let HandleValue::ResponseMessage(resp) = &resp_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let timeout = if timeout_ms == 0 {
            None
        } else {
            Some(std::time::Duration::from_millis(timeout_ms))
        };
        match session.send_response(resp, timeout) {
            Ok(()) => SUCCESS,
            Err(err) => record_client_error(&err),
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_role_reversed_session_close(
    session: *mut kmipkit_role_reversed_session_t,
) -> i32 {
    ffi_entrypoint(|| {
        let session_handle = read_handle!(session, RoleReversedSession);
        let HandleValue::RoleReversedSession(session_arc) = &session_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(mut session) = session_arc.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        match session.close() {
            Ok(()) => SUCCESS,
            Err(err) => record_client_error(&err),
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_role_reversed_session_release(
    session: *mut kmipkit_role_reversed_session_t,
) {
    ffi_entrypoint(|| release(session, Kind::RoleReversedSession));
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_incoming_server_batch_decode(
    data: *const u8,
    length: u64,
    encoding: u32,
    limits: *mut kmipkit_codec_limits_t,
    out_batch: *mut *mut kmipkit_incoming_server_batch_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_batch);
        let span = try_ffi!(input_span(data, length, MAX_FFI_INPUT_BYTES));
        let limits_handle = read_handle!(limits, CodecLimits);
        let HandleValue::CodecLimits(limits_val) = &limits_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let wire_enc = match encoding {
            0 => WireEncoding::Ttlv,
            1 => WireEncoding::Json,
            2 => WireEncoding::Xml,
            _ => return record_ffi_error(ERROR_INVALID_INPUT),
        };
        match kmipkit_client::IncomingServerBatch::from_encoded_bytes(span, wire_enc, limits_val) {
            Ok(batch) => {
                *out = try_ffi!(try_make_handle(
                    Kind::IncomingServerBatch,
                    HandleValue::IncomingServerBatch(Arc::new(batch)),
                ));
                SUCCESS
            }
            Err(err) => record_client_error(&err),
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_incoming_server_batch_decode_ttlv(
    data: *const u8,
    length: u64,
    limits: *mut kmipkit_codec_limits_t,
    out_batch: *mut *mut kmipkit_incoming_server_batch_t,
) -> i32 {
    ffi_entrypoint(|| kmipkit_incoming_server_batch_decode(data, length, 0, limits, out_batch))
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_incoming_server_batch_protocol_version(
    batch: *mut kmipkit_incoming_server_batch_t,
    out_major: *mut u32,
    out_minor: *mut u32,
) -> i32 {
    ffi_entrypoint(|| {
        let major = scalar_out!(out_major);
        let minor = scalar_out!(out_minor);
        let batch_handle = read_handle!(batch, IncomingServerBatch);
        let HandleValue::IncomingServerBatch(batch) = &batch_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let pv = batch.protocol_version();
        *major = pv.major() as u32;
        *minor = pv.minor() as u32;
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_incoming_server_batch_item_count(
    batch: *mut kmipkit_incoming_server_batch_t,
    out_count: *mut u32,
) -> i32 {
    ffi_entrypoint(|| {
        let count = scalar_out!(out_count);
        let batch_handle = read_handle!(batch, IncomingServerBatch);
        let HandleValue::IncomingServerBatch(batch) = &batch_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *count = batch.batch_count();
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_incoming_server_batch_item_operation(
    batch: *mut kmipkit_incoming_server_batch_t,
    item_index: u32,
    out_operation: *mut u32,
) -> i32 {
    ffi_entrypoint(|| {
        let operation = scalar_out!(out_operation);
        let batch_handle = read_handle!(batch, IncomingServerBatch);
        let HandleValue::IncomingServerBatch(batch) = &batch_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let items = batch.items();
        let idx = item_index as usize;
        if idx >= items.len() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        *operation = items[idx].operation();
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_incoming_server_batch_item_unique_batch_item_id(
    batch: *mut kmipkit_incoming_server_batch_t,
    item_index: u32,
    out_buffer: *mut u8,
    buffer_capacity: u64,
    out_written_len: *mut u64,
) -> i32 {
    ffi_entrypoint(|| {
        let written_len = scalar_out!(out_written_len);
        let batch_handle = read_handle!(batch, IncomingServerBatch);
        let HandleValue::IncomingServerBatch(batch) = &batch_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let items = batch.items();
        let idx = item_index as usize;
        if idx >= items.len() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        match items[idx].unique_batch_item_id() {
            Some(id) => {
                *written_len = id.len() as u64;
                if buffer_capacity == 0 {
                    return SUCCESS;
                }
                if buffer_capacity < *written_len || out_buffer.is_null() {
                    return record_ffi_error(ERROR_INVALID_INPUT);
                }
                unsafe {
                    ptr::copy_nonoverlapping(id.as_ptr(), out_buffer, id.len());
                }
                SUCCESS
            }
            None => {
                *written_len = 0;
                SUCCESS
            }
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_incoming_server_batch_item_discover_versions(
    batch: *mut kmipkit_incoming_server_batch_t,
    item_index: u32,
    out_versions: *mut u32,
    versions_capacity: u32,
    out_versions_count: *mut u32,
) -> i32 {
    ffi_entrypoint(|| {
        let out_count = scalar_out!(out_versions_count);
        let batch_handle = read_handle!(batch, IncomingServerBatch);
        let HandleValue::IncomingServerBatch(batch) = &batch_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let items = batch.items();
        let idx = item_index as usize;
        if idx >= items.len() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        if let Some(req) = items[idx].as_discover_versions() {
            let versions = req.protocol_versions();
            let count = versions.len() as u32;
            *out_count = count;
            if versions_capacity < count || out_versions.is_null() {
                return if versions_capacity == 0 {
                    SUCCESS
                } else {
                    record_ffi_error(ERROR_INVALID_INPUT)
                };
            }
            for (i, v) in versions.iter().enumerate() {
                let packed = ((v.major() as u32) << 16) | (v.minor() as u32);
                unsafe {
                    *out_versions.add(i) = packed;
                }
            }
            SUCCESS
        } else {
            record_ffi_error(ERROR_INVALID_INPUT)
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_incoming_server_batch_item_notify(
    batch: *mut kmipkit_incoming_server_batch_t,
    item_index: u32,
    out_uid_buffer: *mut u8,
    uid_capacity: u64,
    out_uid_len: *mut u64,
) -> i32 {
    ffi_entrypoint(|| {
        let out_len = scalar_out!(out_uid_len);
        let batch_handle = read_handle!(batch, IncomingServerBatch);
        let HandleValue::IncomingServerBatch(batch) = &batch_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let items = batch.items();
        let idx = item_index as usize;
        if idx >= items.len() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        if let Some(req) = items[idx].as_notify() {
            let uid = req.unique_identifier().as_bytes();
            let len = uid.len() as u64;
            *out_len = len;
            if uid_capacity < len || out_uid_buffer.is_null() {
                return if uid_capacity == 0 {
                    SUCCESS
                } else {
                    record_ffi_error(ERROR_INVALID_INPUT)
                };
            }
            unsafe {
                ptr::copy_nonoverlapping(uid.as_ptr(), out_uid_buffer, uid.len());
            }
            SUCCESS
        } else {
            record_ffi_error(ERROR_INVALID_INPUT)
        }
    })
}

fn write_ttlv_header(out: &mut Vec<u8>, tag_bytes: [u8; 3], item_type: u8, len: usize) {
    out.extend_from_slice(&tag_bytes);
    out.push(item_type);
    out.extend_from_slice(&u32::try_from(len).unwrap_or(0).to_be_bytes());
}

fn encode_structure_view_bytes(view: &kmipkit_ttlv::StructureView<'_>, out: &mut Vec<u8>) {
    for child in view.children() {
        kmipkit_ttlv::codec::encode_ttlv_item(child, out);
    }
}

/// Returns the Put request's identifiers, object, and attributes when present.
///
/// # Safety
///
/// `batch` must be a live incoming-batch handle. Each non-null output pointer
/// must reference aligned writable storage for its value; each non-null byte
/// buffer must be writable for its declared capacity. Output regions must not
/// overlap the batch handle or one another.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kmipkit_incoming_server_batch_item_put_v2(
    batch: *mut kmipkit_incoming_server_batch_t,
    item_index: u32,
    out_put_function: *mut u32,
    out_uid_buffer: *mut u8,
    uid_capacity: u64,
    out_uid_len: *mut u64,
    out_replaced_uid_buffer: *mut u8,
    replaced_uid_capacity: u64,
    out_replaced_uid_len: *mut u64,
    out_object_item: *mut *mut kmipkit_ttlv_item_t,
    out_attributes: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let batch_handle = read_handle!(batch, IncomingServerBatch);
        let HandleValue::IncomingServerBatch(batch) = &batch_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let items = batch.items();
        let idx = item_index as usize;
        if idx >= items.len() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        if let Some(req) = items[idx].as_put() {
            if !out_put_function.is_null() {
                unsafe {
                    *out_put_function = req.put_function().raw();
                }
            }
            let uid = req.unique_identifier().as_bytes();
            let uid_len = uid.len() as u64;
            if !out_uid_len.is_null() {
                unsafe {
                    *out_uid_len = uid_len;
                }
            }
            if !out_uid_buffer.is_null() && uid_capacity >= uid_len {
                unsafe {
                    ptr::copy_nonoverlapping(uid.as_ptr(), out_uid_buffer, uid.len());
                }
            } else if !out_uid_buffer.is_null() && uid_capacity < uid_len && uid_capacity > 0 {
                return record_ffi_error(ERROR_INVALID_INPUT);
            }
            if let Some(rep_uid) = req.replaced_unique_identifier() {
                let rep_bytes = rep_uid.as_bytes();
                let rep_len = rep_bytes.len() as u64;
                if !out_replaced_uid_len.is_null() {
                    unsafe {
                        *out_replaced_uid_len = rep_len;
                    }
                }
                if !out_replaced_uid_buffer.is_null() && replaced_uid_capacity >= rep_len {
                    unsafe {
                        ptr::copy_nonoverlapping(
                            rep_bytes.as_ptr(),
                            out_replaced_uid_buffer,
                            rep_bytes.len(),
                        );
                    }
                } else if !out_replaced_uid_buffer.is_null()
                    && replaced_uid_capacity < rep_len
                    && replaced_uid_capacity > 0
                {
                    return record_ffi_error(ERROR_INVALID_INPUT);
                }
            } else if !out_replaced_uid_len.is_null() {
                unsafe {
                    *out_replaced_uid_len = 0;
                }
            }
            if !out_object_item.is_null() {
                let cloned_item =
                    try_ffi!(req.object_item().try_clone().map_err(|_| ERROR_INTERNAL));
                unsafe {
                    *out_object_item =
                        try_ffi!(try_make_handle(Kind::Item, HandleValue::Item(cloned_item)));
                }
            }
            if !out_attributes.is_null() {
                if let Some(attrs) = req.attributes() {
                    let cloned_attrs =
                        try_ffi!(attrs.view().try_clone().map_err(|_| ERROR_INTERNAL));
                    unsafe {
                        *out_attributes = try_ffi!(try_make_handle(
                            Kind::Structure,
                            HandleValue::Structure(cloned_attrs)
                        ));
                    }
                } else {
                    unsafe {
                        *out_attributes = ptr::null_mut();
                    }
                }
            }
            SUCCESS
        } else {
            record_ffi_error(ERROR_INVALID_INPUT)
        }
    })
}

#[unsafe(no_mangle)]
/// Returns a Put request's unique identifier and operation when present.
///
/// # Safety
///
/// `batch` must be a live incoming-batch handle. Each non-null output pointer
/// must reference aligned writable storage for its value, and the optional
/// byte buffer must be writable for its declared capacity. Output regions must
/// not overlap the batch handle or one another.
pub unsafe extern "C" fn kmipkit_incoming_server_batch_item_put(
    batch: *mut kmipkit_incoming_server_batch_t,
    item_index: u32,
    out_uid_buffer: *mut u8,
    uid_capacity: u64,
    out_uid_len: *mut u64,
    out_put_function: *mut u32,
) -> i32 {
    ffi_entrypoint(|| {
        // SAFETY: the legacy ABI forwards the batch and output pointers under
        // the same caller preconditions and sets every added output to null.
        unsafe {
            kmipkit_incoming_server_batch_item_put_v2(
                batch,
                item_index,
                out_put_function,
                out_uid_buffer,
                uid_capacity,
                out_uid_len,
                ptr::null_mut(),
                0,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
            )
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_incoming_server_batch_item_query(
    batch: *mut kmipkit_incoming_server_batch_t,
    item_index: u32,
    out_query_function_flags: *mut u32,
) -> i32 {
    ffi_entrypoint(|| {
        let out_flags = scalar_out!(out_query_function_flags);
        let batch_handle = read_handle!(batch, IncomingServerBatch);
        let HandleValue::IncomingServerBatch(batch) = &batch_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let items = batch.items();
        let idx = item_index as usize;
        if idx >= items.len() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        if let Some(req) = items[idx].as_query() {
            let mut flags: u32 = 0;
            for qf in req.query_functions() {
                let raw = qf.raw();
                if (1..=32).contains(&raw) {
                    flags |= 1u32 << (raw - 1);
                }
            }
            *out_flags = flags;
            SUCCESS
        } else {
            record_ffi_error(ERROR_INVALID_INPUT)
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_incoming_server_batch_item_set_endpoint_role(
    batch: *mut kmipkit_incoming_server_batch_t,
    item_index: u32,
    out_endpoint_role: *mut u32,
) -> i32 {
    ffi_entrypoint(|| {
        let out_role = scalar_out!(out_endpoint_role);
        let batch_handle = read_handle!(batch, IncomingServerBatch);
        let HandleValue::IncomingServerBatch(batch) = &batch_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let items = batch.items();
        let idx = item_index as usize;
        if idx >= items.len() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        if let Some(req) = items[idx].as_set_endpoint_role() {
            *out_role = req.endpoint_role().raw();
            SUCCESS
        } else {
            record_ffi_error(ERROR_INVALID_INPUT)
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_incoming_server_batch_release(
    batch: *mut kmipkit_incoming_server_batch_t,
) {
    ffi_entrypoint(|| release(batch, Kind::IncomingServerBatch));
}

fn s2c_failure_result() -> kmipkit_protocol::KmipOperationResult {
    kmipkit_protocol::KmipOperationResult::new(
        kmipkit_protocol::ResultStatus::from_raw(1),
        Some(kmipkit_protocol::ResultReason::from_raw(0x05)),
        Some(kmipkit_protocol::ResultMessage::new(
            "operation not supported".to_string(),
        )),
    )
    .expect("valid failure result")
}

const MAX_S2C_CAPABILITY_ITEMS: u32 = 1024;

fn parse_s2c_capabilities_v1(
    capabilities: *const kmipkit_s2c_capabilities_v1_t,
) -> FfiResult<kmipkit_protocol::ServerCapabilities> {
    if capabilities.is_null() {
        return Err(ERROR_INVALID_INPUT);
    }
    let cap_ref = unsafe { &*capabilities };
    let expected_size = std::mem::size_of::<kmipkit_s2c_capabilities_v1_t>() as u32;
    if cap_ref.struct_size != expected_size || cap_ref.abi_version != 1 || cap_ref.reserved != 0 {
        return Err(ERROR_INVALID_INPUT);
    }
    if cap_ref.operations_count > MAX_S2C_CAPABILITY_ITEMS
        || cap_ref.object_types_count > MAX_S2C_CAPABILITY_ITEMS
        || cap_ref.extension_tags_count > MAX_S2C_CAPABILITY_ITEMS
    {
        return Err(ERROR_RESOURCE_LIMIT);
    }
    if (cap_ref.operations_count > 0 && cap_ref.operations.is_null())
        || (cap_ref.object_types_count > 0 && cap_ref.object_types.is_null())
        || (cap_ref.extension_tags_count > 0 && cap_ref.extension_tags.is_null())
    {
        return Err(ERROR_INVALID_INPUT);
    }
    let mut caps = kmipkit_protocol::ServerCapabilities::new();
    if cap_ref.operations_count > 0 {
        let slice = unsafe {
            std::slice::from_raw_parts(cap_ref.operations, cap_ref.operations_count as usize)
        };
        caps.set_operations(slice.to_vec());
    }
    if cap_ref.object_types_count > 0 {
        let slice = unsafe {
            std::slice::from_raw_parts(cap_ref.object_types, cap_ref.object_types_count as usize)
        };
        caps.set_object_types(slice.to_vec());
    }
    if cap_ref.vendor_identification_length > 0 || !cap_ref.vendor_identification_data.is_null() {
        let span = input_span(
            cap_ref.vendor_identification_data,
            cap_ref.vendor_identification_length,
            MAX_FFI_INPUT_BYTES,
        )?;
        let text = std::str::from_utf8(span).map_err(|_| ERROR_INVALID_INPUT)?;
        if !text.is_empty() {
            caps.set_vendor_identification(text);
        }
    }
    if cap_ref.extension_tags_count > 0 {
        let slice = unsafe {
            std::slice::from_raw_parts(
                cap_ref.extension_tags,
                cap_ref.extension_tags_count as usize,
            )
        };
        #[allow(deprecated)]
        caps.set_extension_tags(slice.to_vec());
    }
    Ok(caps)
}

fn parse_s2c_capabilities_v2(
    capabilities: *const kmipkit_s2c_capabilities_v2_t,
) -> FfiResult<kmipkit_protocol::ServerCapabilities> {
    if capabilities.is_null() {
        return Err(ERROR_INVALID_INPUT);
    }
    // The caller supplies a readable v2 record.
    let capabilities = unsafe { &*capabilities };
    if capabilities.struct_size != std::mem::size_of::<kmipkit_s2c_capabilities_v2_t>() as u32
        || capabilities.abi_version != 2
        || capabilities.reserved != 0
    {
        return Err(ERROR_INVALID_INPUT);
    }
    let mut base = parse_s2c_capabilities_v1(&capabilities.base)?;
    let count = capabilities.extension_information_count;
    if count == 0 {
        return Ok(base);
    }
    if count > MAX_S2C_CAPABILITY_ITEMS {
        return Err(ERROR_RESOURCE_LIMIT);
    }
    if capabilities.extension_information.is_null() || capabilities.base.extension_tags_count != 0 {
        return Err(ERROR_INVALID_INPUT);
    }
    // The caller keeps this array and its referenced handles live for the call.
    let handles =
        unsafe { std::slice::from_raw_parts(capabilities.extension_information, count as usize) };
    let information = handles
        .iter()
        .map(|&pointer| {
            let handle = reference_handle(pointer, Kind::Information)?;
            match &handle.value {
                HandleValue::Information(value) => Ok(value.clone()),
                _ => Err(ERROR_INVALID_INPUT),
            }
        })
        .collect::<FfiResult<Vec<_>>>()?;
    base.set_extension_information(information);
    Ok(base)
}

fn parse_s2c_operation_result_v1(
    raw: *const kmipkit_s2c_operation_result_v1_t,
) -> Result<kmipkit_protocol::KmipOperationResult, i32> {
    if raw.is_null() {
        return Ok(s2c_failure_result());
    }
    let res = unsafe { &*raw };
    let expected_size = std::mem::size_of::<kmipkit_s2c_operation_result_v1_t>() as u32;
    if res.struct_size != expected_size || res.abi_version != 1 || res.reserved != 0 {
        return Ok(s2c_failure_result());
    }
    let status = kmipkit_protocol::ResultStatus::from_raw(res.result_status);
    let reason = if res.has_result_reason != 0 {
        Some(kmipkit_protocol::ResultReason::from_raw(res.result_reason))
    } else {
        None
    };
    let message = if !res.result_message_data.is_null() {
        let span = input_span(
            res.result_message_data,
            res.result_message_length,
            MAX_FFI_INPUT_BYTES,
        )?;
        let text = std::str::from_utf8(span).map_err(|_| ERROR_INVALID_INPUT)?;
        Some(kmipkit_protocol::ResultMessage::new(text.to_owned()))
    } else {
        None
    };

    let reason = if status.raw() == 1 && reason.is_none() {
        Some(kmipkit_protocol::ResultReason::from_raw(0x0B))
    } else {
        reason
    };

    kmipkit_protocol::KmipOperationResult::new(status, reason, message)
        .map_err(|_| ERROR_INVALID_INPUT)
}

struct CServerRequestHandlerAdapterV1 {
    handler: kmipkit_server_request_handler_v1_t,
    current_batch_item_id: Option<Vec<u8>>,
}

impl kmipkit_client::ServerRequestHandler for CServerRequestHandlerAdapterV1 {
    fn set_current_batch_item(&mut self, item: &kmipkit_client::ServerRequestItemView<'_>) {
        self.current_batch_item_id = item.unique_batch_item_id().map(<[u8]>::to_vec);
    }

    fn handle_discover_versions(
        &mut self,
        request: &kmipkit_protocol::ServerDiscoverVersionsRequest,
    ) -> Result<
        kmipkit_protocol::ServerDiscoverVersionsResponse,
        kmipkit_protocol::KmipOperationResult,
    > {
        let callback = match self.handler.handle_discover_versions {
            Some(cb) => cb,
            None => return Err(s2c_failure_result()),
        };
        let view = kmipkit_server_request_item_view_v1_t {
            struct_size: std::mem::size_of::<kmipkit_server_request_item_view_v1_t>() as u32,
            abi_version: 1,
            operation: 0x0000_001E,
            unique_batch_item_id_data: self
                .current_batch_item_id
                .as_ref()
                .map_or(ptr::null(), |id| id.as_ptr()),
            unique_batch_item_id_length: self
                .current_batch_item_id
                .as_ref()
                .map_or(0, |id| id.len() as u64),
        };
        let mut supported: [u32; 16] = [0; 16];
        let mut supported_count: u32 = 0;
        let requested = request.protocol_versions();
        let (req_maj, req_min) = if let Some(pv) = requested.first() {
            (pv.major() as u32, pv.minor() as u32)
        } else {
            (2, 1)
        };
        let code = unsafe {
            callback(
                self.handler.user_data,
                &view,
                req_maj,
                req_min,
                supported.as_mut_ptr(),
                16,
                &mut supported_count,
            )
        };
        if code != 0 {
            return Err(s2c_failure_result());
        }
        let count = (supported_count as usize).min(16);
        let mut versions = Vec::new();
        for &packed in &supported[..count] {
            let maj = (packed >> 16) as i32;
            let min = (packed & 0xFFFF) as i32;
            versions.push(kmipkit_protocol::ProtocolVersion::from_raw(maj, min));
        }
        if versions.is_empty() {
            versions.push(kmipkit_protocol::ProtocolVersion::V2_1);
        }
        Ok(kmipkit_protocol::ServerDiscoverVersionsResponse::new(
            versions,
        ))
    }

    fn handle_notify(
        &mut self,
        request: &kmipkit_protocol::ServerNotifyRequest,
    ) -> Result<kmipkit_protocol::ServerNotifyResponse, kmipkit_protocol::KmipOperationResult> {
        let callback = match self.handler.handle_notify {
            Some(cb) => cb,
            None => return Err(s2c_failure_result()),
        };
        let uid = request.unique_identifier();
        let view = kmipkit_server_request_item_view_v1_t {
            struct_size: std::mem::size_of::<kmipkit_server_request_item_view_v1_t>() as u32,
            abi_version: 1,
            operation: 0x0000_001B,
            unique_batch_item_id_data: self
                .current_batch_item_id
                .as_ref()
                .map_or(ptr::null(), |id| id.as_ptr()),
            unique_batch_item_id_length: self
                .current_batch_item_id
                .as_ref()
                .map_or(0, |id| id.len() as u64),
        };
        let code = unsafe {
            callback(
                self.handler.user_data,
                &view,
                uid.as_ptr(),
                uid.len() as u64,
            )
        };
        if code != 0 {
            return Err(s2c_failure_result());
        }
        Ok(kmipkit_protocol::ServerNotifyResponse::new())
    }

    fn handle_put(
        &mut self,
        request: &kmipkit_protocol::ServerPutRequest,
    ) -> Result<kmipkit_protocol::ServerPutResponse, kmipkit_protocol::KmipOperationResult> {
        let callback = match self.handler.handle_put {
            Some(cb) => cb,
            None => return Err(s2c_failure_result()),
        };
        let uid = request.unique_identifier();
        let view = kmipkit_server_request_item_view_v1_t {
            struct_size: std::mem::size_of::<kmipkit_server_request_item_view_v1_t>() as u32,
            abi_version: 1,
            operation: 0x0000_001C,
            unique_batch_item_id_data: self
                .current_batch_item_id
                .as_ref()
                .map_or(ptr::null(), |id| id.as_ptr()),
            unique_batch_item_id_length: self
                .current_batch_item_id
                .as_ref()
                .map_or(0, |id| id.len() as u64),
        };
        let mut obj_bytes = Vec::new();
        kmipkit_ttlv::codec::encode_ttlv_item(request.object_item(), &mut obj_bytes);
        let code = unsafe {
            callback(
                self.handler.user_data,
                &view,
                uid.as_ptr(),
                uid.len() as u64,
                obj_bytes.as_ptr(),
                obj_bytes.len() as u64,
            )
        };
        if code != 0 {
            return Err(s2c_failure_result());
        }
        Ok(kmipkit_protocol::ServerPutResponse::new())
    }

    fn handle_query(
        &mut self,
        request: &kmipkit_protocol::ServerQueryRequest,
    ) -> Result<kmipkit_protocol::ServerQueryResponse, kmipkit_protocol::KmipOperationResult> {
        let callback = match self.handler.handle_query {
            Some(cb) => cb,
            None => return Err(s2c_failure_result()),
        };
        let view = kmipkit_server_request_item_view_v1_t {
            struct_size: std::mem::size_of::<kmipkit_server_request_item_view_v1_t>() as u32,
            abi_version: 1,
            operation: 0x0000_0018,
            unique_batch_item_id_data: self
                .current_batch_item_id
                .as_ref()
                .map_or(ptr::null(), |id| id.as_ptr()),
            unique_batch_item_id_length: self
                .current_batch_item_id
                .as_ref()
                .map_or(0, |id| id.len() as u64),
        };
        let mut flags: u32 = 0;
        for qf in request.query_functions() {
            let raw = qf.raw();
            if (1..=32).contains(&raw) {
                flags |= 1u32 << (raw - 1);
            }
        }
        let code = unsafe { callback(self.handler.user_data, &view, flags) };
        if code != 0 {
            return Err(s2c_failure_result());
        }
        let mut caps = kmipkit_protocol::ServerCapabilities::new();
        caps.set_operations(vec![0x0000_001E, 0x0000_001B, 0x0000_001C, 0x0000_0018]);
        caps.set_object_types(vec![0x01, 0x02, 0x07]);
        caps.set_vendor_identification("KMIPKit");
        Ok(kmipkit_protocol::ServerQueryResponse::respond(
            request, &caps,
        ))
    }

    fn handle_set_endpoint_role(
        &mut self,
        request: &kmipkit_protocol::SetEndpointRoleRequest,
    ) -> Result<kmipkit_protocol::SetEndpointRoleResponse, kmipkit_protocol::KmipOperationResult>
    {
        let callback = match self.handler.handle_set_endpoint_role {
            Some(cb) => cb,
            None => return Err(s2c_failure_result()),
        };
        let view = kmipkit_server_request_item_view_v1_t {
            struct_size: std::mem::size_of::<kmipkit_server_request_item_view_v1_t>() as u32,
            abi_version: 1,
            operation: 0x0000_0032,
            unique_batch_item_id_data: self
                .current_batch_item_id
                .as_ref()
                .map_or(ptr::null(), |id| id.as_ptr()),
            unique_batch_item_id_length: self
                .current_batch_item_id
                .as_ref()
                .map_or(0, |id| id.len() as u64),
        };
        let code =
            unsafe { callback(self.handler.user_data, &view, request.endpoint_role().raw()) };
        if code != 0 {
            return Err(s2c_failure_result());
        }
        Ok(kmipkit_protocol::SetEndpointRoleResponse::success(
            request.endpoint_role(),
            None,
        ))
    }
}

struct CServerRequestHandlerAdapterV2 {
    handler: kmipkit_server_request_handler_v2_t,
    current_batch_item_id: Option<Vec<u8>>,
}

impl kmipkit_client::ServerRequestHandler for CServerRequestHandlerAdapterV2 {
    fn set_current_batch_item(&mut self, item: &kmipkit_client::ServerRequestItemView<'_>) {
        self.current_batch_item_id = item.unique_batch_item_id().map(<[u8]>::to_vec);
    }

    fn handle_discover_versions(
        &mut self,
        request: &kmipkit_protocol::ServerDiscoverVersionsRequest,
    ) -> Result<
        kmipkit_protocol::ServerDiscoverVersionsResponse,
        kmipkit_protocol::KmipOperationResult,
    > {
        let callback = match self.handler.handle_discover_versions {
            Some(cb) => cb,
            None => return Err(s2c_failure_result()),
        };
        let view = kmipkit_server_request_item_view_v1_t {
            struct_size: std::mem::size_of::<kmipkit_server_request_item_view_v1_t>() as u32,
            abi_version: 1,
            operation: 0x0000_001E,
            unique_batch_item_id_data: self
                .current_batch_item_id
                .as_ref()
                .map_or(ptr::null(), |id| id.as_ptr()),
            unique_batch_item_id_length: self
                .current_batch_item_id
                .as_ref()
                .map_or(0, |id| id.len() as u64),
        };
        let mut supported: [u32; 16] = [0; 16];
        let mut supported_count: u32 = 0;
        let requested = request.protocol_versions();
        let (req_maj, req_min) = if let Some(pv) = requested.first() {
            (pv.major() as u32, pv.minor() as u32)
        } else {
            (2, 1)
        };
        let code = unsafe {
            callback(
                self.handler.user_data,
                &view,
                req_maj,
                req_min,
                supported.as_mut_ptr(),
                16,
                &mut supported_count,
            )
        };
        if code != 0 {
            return Err(s2c_failure_result());
        }
        let count = (supported_count as usize).min(16);
        let mut versions = Vec::new();
        for &packed in &supported[..count] {
            let maj = (packed >> 16) as i32;
            let min = (packed & 0xFFFF) as i32;
            versions.push(kmipkit_protocol::ProtocolVersion::from_raw(maj, min));
        }
        if versions.is_empty() {
            versions.push(kmipkit_protocol::ProtocolVersion::V2_1);
        }
        Ok(kmipkit_protocol::ServerDiscoverVersionsResponse::new(
            versions,
        ))
    }

    fn handle_notify(
        &mut self,
        request: &kmipkit_protocol::ServerNotifyRequest,
    ) -> Result<kmipkit_protocol::ServerNotifyResponse, kmipkit_protocol::KmipOperationResult> {
        let callback = match self.handler.handle_notify {
            Some(cb) => cb,
            None => return Err(s2c_failure_result()),
        };
        let uid = request.unique_identifier();
        let view = kmipkit_server_request_item_view_v1_t {
            struct_size: std::mem::size_of::<kmipkit_server_request_item_view_v1_t>() as u32,
            abi_version: 1,
            operation: 0x0000_001B,
            unique_batch_item_id_data: self
                .current_batch_item_id
                .as_ref()
                .map_or(ptr::null(), |id| id.as_ptr()),
            unique_batch_item_id_length: self
                .current_batch_item_id
                .as_ref()
                .map_or(0, |id| id.len() as u64),
        };
        let mut attr_bytes = Vec::new();
        let (attr_ptr, attr_len) = if let Some(attrs) = request.attributes() {
            let mut st_bytes = Vec::new();
            encode_structure_view_bytes(&attrs.view(), &mut st_bytes);
            let raw_tag = 0x0042_0125u32.to_be_bytes();
            write_ttlv_header(
                &mut attr_bytes,
                [raw_tag[1], raw_tag[2], raw_tag[3]],
                0x01,
                st_bytes.len(),
            );
            attr_bytes.extend_from_slice(&st_bytes);
            (attr_bytes.as_ptr(), attr_bytes.len() as u64)
        } else {
            (ptr::null(), 0)
        };
        let code = unsafe {
            callback(
                self.handler.user_data,
                &view,
                uid.as_ptr(),
                uid.len() as u64,
                attr_ptr,
                attr_len,
            )
        };
        if code != 0 {
            return Err(s2c_failure_result());
        }
        Ok(kmipkit_protocol::ServerNotifyResponse::new())
    }

    fn handle_put(
        &mut self,
        request: &kmipkit_protocol::ServerPutRequest,
    ) -> Result<kmipkit_protocol::ServerPutResponse, kmipkit_protocol::KmipOperationResult> {
        let callback = match self.handler.handle_put {
            Some(cb) => cb,
            None => return Err(s2c_failure_result()),
        };
        let uid = request.unique_identifier();
        let view = kmipkit_server_request_item_view_v1_t {
            struct_size: std::mem::size_of::<kmipkit_server_request_item_view_v1_t>() as u32,
            abi_version: 1,
            operation: 0x0000_001C,
            unique_batch_item_id_data: self
                .current_batch_item_id
                .as_ref()
                .map_or(ptr::null(), |id| id.as_ptr()),
            unique_batch_item_id_length: self
                .current_batch_item_id
                .as_ref()
                .map_or(0, |id| id.len() as u64),
        };
        let mut obj_bytes = Vec::new();
        kmipkit_ttlv::codec::encode_ttlv_item(request.object_item(), &mut obj_bytes);

        let mut attr_bytes = Vec::new();
        let (attr_ptr, attr_len) = if let Some(attrs) = request.attributes() {
            let mut st_bytes = Vec::new();
            encode_structure_view_bytes(&attrs.view(), &mut st_bytes);
            let raw_tag = 0x0042_0125u32.to_be_bytes();
            write_ttlv_header(
                &mut attr_bytes,
                [raw_tag[1], raw_tag[2], raw_tag[3]],
                0x01,
                st_bytes.len(),
            );
            attr_bytes.extend_from_slice(&st_bytes);
            (attr_bytes.as_ptr(), attr_bytes.len() as u64)
        } else {
            (ptr::null(), 0)
        };

        let (rep_ptr, rep_len) = if let Some(rep_uid) = request.replaced_unique_identifier() {
            (rep_uid.as_ptr(), rep_uid.len() as u64)
        } else {
            (ptr::null(), 0)
        };

        let code = unsafe {
            callback(
                self.handler.user_data,
                &view,
                request.put_function().raw(),
                uid.as_ptr(),
                uid.len() as u64,
                rep_ptr,
                rep_len,
                obj_bytes.as_ptr(),
                obj_bytes.len() as u64,
                attr_ptr,
                attr_len,
            )
        };
        if code != 0 {
            return Err(s2c_failure_result());
        }
        Ok(kmipkit_protocol::ServerPutResponse::new())
    }

    fn handle_query(
        &mut self,
        request: &kmipkit_protocol::ServerQueryRequest,
    ) -> Result<kmipkit_protocol::ServerQueryResponse, kmipkit_protocol::KmipOperationResult> {
        let callback = match self.handler.handle_query {
            Some(cb) => cb,
            None => return Err(s2c_failure_result()),
        };
        let view = kmipkit_server_request_item_view_v1_t {
            struct_size: std::mem::size_of::<kmipkit_server_request_item_view_v1_t>() as u32,
            abi_version: 1,
            operation: 0x0000_0018,
            unique_batch_item_id_data: self
                .current_batch_item_id
                .as_ref()
                .map_or(ptr::null(), |id| id.as_ptr()),
            unique_batch_item_id_length: self
                .current_batch_item_id
                .as_ref()
                .map_or(0, |id| id.len() as u64),
        };
        let mut flags: u32 = 0;
        for qf in request.query_functions() {
            let raw = qf.raw();
            if (1..=32).contains(&raw) {
                flags |= 1u32 << (raw - 1);
            }
        }
        let mut out_caps = kmipkit_s2c_capabilities_v1_t {
            struct_size: std::mem::size_of::<kmipkit_s2c_capabilities_v1_t>() as u32,
            abi_version: 1,
            operations: ptr::null(),
            operations_count: 0,
            object_types: ptr::null(),
            object_types_count: 0,
            vendor_identification_data: ptr::null(),
            vendor_identification_length: 0,
            extension_tags: ptr::null(),
            extension_tags_count: 0,
            reserved: 0,
        };
        let code = unsafe { callback(self.handler.user_data, &view, flags, &mut out_caps) };
        if code != 0 {
            return Err(s2c_failure_result());
        }
        let caps = match parse_s2c_capabilities_v1(&out_caps) {
            Ok(c) => c,
            Err(_) => return Err(s2c_failure_result()),
        };
        Ok(kmipkit_protocol::ServerQueryResponse::respond(
            request, &caps,
        ))
    }

    fn handle_set_endpoint_role(
        &mut self,
        request: &kmipkit_protocol::SetEndpointRoleRequest,
    ) -> Result<kmipkit_protocol::SetEndpointRoleResponse, kmipkit_protocol::KmipOperationResult>
    {
        let callback = match self.handler.handle_set_endpoint_role {
            Some(cb) => cb,
            None => return Err(s2c_failure_result()),
        };
        let view = kmipkit_server_request_item_view_v1_t {
            struct_size: std::mem::size_of::<kmipkit_server_request_item_view_v1_t>() as u32,
            abi_version: 1,
            operation: 0x0000_0032,
            unique_batch_item_id_data: self
                .current_batch_item_id
                .as_ref()
                .map_or(ptr::null(), |id| id.as_ptr()),
            unique_batch_item_id_length: self
                .current_batch_item_id
                .as_ref()
                .map_or(0, |id| id.len() as u64),
        };
        let code =
            unsafe { callback(self.handler.user_data, &view, request.endpoint_role().raw()) };
        if code != 0 {
            return Err(s2c_failure_result());
        }
        Ok(kmipkit_protocol::SetEndpointRoleResponse::success(
            request.endpoint_role(),
            None,
        ))
    }
}

struct CServerRequestHandlerAdapterV3 {
    handler: kmipkit_server_request_handler_v3_t,
    handler_v4: Option<kmipkit_server_request_handler_v4_t>,
    handler_v5: Option<kmipkit_server_request_handler_v5_t>,
    handler_v6: Option<kmipkit_server_request_handler_v6_t>,
    current_batch_item_id: Option<Vec<u8>>,
}

impl kmipkit_client::ServerRequestHandler for CServerRequestHandlerAdapterV3 {
    fn set_current_batch_item(&mut self, item: &kmipkit_client::ServerRequestItemView<'_>) {
        self.current_batch_item_id = item.unique_batch_item_id().map(<[u8]>::to_vec);
    }

    fn handle_discover_versions(
        &mut self,
        request: &kmipkit_protocol::ServerDiscoverVersionsRequest,
    ) -> Result<
        kmipkit_protocol::ServerDiscoverVersionsResponse,
        kmipkit_protocol::KmipOperationResult,
    > {
        let view = kmipkit_server_request_item_view_v1_t {
            struct_size: std::mem::size_of::<kmipkit_server_request_item_view_v1_t>() as u32,
            abi_version: 1,
            operation: 0x0000_001E,
            unique_batch_item_id_data: self
                .current_batch_item_id
                .as_ref()
                .map_or(ptr::null(), |id| id.as_ptr()),
            unique_batch_item_id_length: self
                .current_batch_item_id
                .as_ref()
                .map_or(0, |id| id.len() as u64),
        };
        let mut supported: [u32; 16] = [0; 16];
        let mut supported_count: u32 = 0;
        let requested: Vec<u32> = request
            .protocol_versions()
            .iter()
            .map(|pv| ((pv.major() as u32) << 16) | pv.minor() as u32)
            .collect();
        let mut out_result = kmipkit_s2c_operation_result_v1_t {
            struct_size: std::mem::size_of::<kmipkit_s2c_operation_result_v1_t>() as u32,
            abi_version: 1,
            result_status: 1,
            has_result_reason: 1,
            result_reason: 0x05,
            reserved: 0,
            result_message_data: ptr::null(),
            result_message_length: 0,
        };
        let mut borrowed_versions = ptr::null();
        let code = if let Some(handler) = self.handler_v4 {
            let Some(callback) = handler.handle_discover_versions else {
                return Err(s2c_failure_result());
            };
            unsafe {
                callback(
                    handler.user_data,
                    &view,
                    requested.as_ptr(),
                    requested.len() as u32,
                    &mut borrowed_versions,
                    &mut supported_count,
                    &mut out_result,
                )
            }
        } else {
            let Some(callback) = self.handler.handle_discover_versions else {
                return Err(s2c_failure_result());
            };
            let first = requested.first().copied().unwrap_or(0x0002_0001);
            unsafe {
                callback(
                    self.handler.user_data,
                    &view,
                    first >> 16,
                    first & 0xffff,
                    supported.as_mut_ptr(),
                    16,
                    &mut supported_count,
                    &mut out_result,
                )
            }
        };
        if code != 0 {
            let err_result = match parse_s2c_operation_result_v1(&out_result) {
                Ok(r) if r.status().raw() != 0 => r,
                _ => s2c_failure_result(),
            };
            return Err(err_result);
        }
        let packed_versions = if self.handler_v4.is_some() {
            if supported_count == 0 {
                &[][..]
            } else if borrowed_versions.is_null()
                || supported_count as u64 > MAX_FFI_INPUT_BYTES / 4
            {
                return Err(s2c_failure_result());
            } else {
                // The callback guarantees readable versions until its next invocation.
                unsafe { std::slice::from_raw_parts(borrowed_versions, supported_count as usize) }
            }
        } else {
            if supported_count > 16 {
                return Err(s2c_failure_result());
            }
            &supported[..supported_count as usize]
        };
        let mut versions = Vec::new();
        for &packed in packed_versions {
            let maj = (packed >> 16) as i32;
            let min = (packed & 0xFFFF) as i32;
            versions.push(kmipkit_protocol::ProtocolVersion::from_raw(maj, min));
        }
        Ok(kmipkit_protocol::ServerDiscoverVersionsResponse::new(
            versions,
        ))
    }

    fn handle_notify(
        &mut self,
        request: &kmipkit_protocol::ServerNotifyRequest,
    ) -> Result<kmipkit_protocol::ServerNotifyResponse, kmipkit_protocol::KmipOperationResult> {
        let uid = request.unique_identifier();
        let view = kmipkit_server_request_item_view_v1_t {
            struct_size: std::mem::size_of::<kmipkit_server_request_item_view_v1_t>() as u32,
            abi_version: 1,
            operation: 0x0000_001B,
            unique_batch_item_id_data: self
                .current_batch_item_id
                .as_ref()
                .map_or(ptr::null(), |id| id.as_ptr()),
            unique_batch_item_id_length: self
                .current_batch_item_id
                .as_ref()
                .map_or(0, |id| id.len() as u64),
        };
        let mut attr_bytes = Vec::new();
        let (attr_ptr, attr_len) = if let Some(attrs) = request.attributes() {
            let mut st_bytes = Vec::new();
            encode_structure_view_bytes(&attrs.view(), &mut st_bytes);
            let raw_tag = 0x0042_0125u32.to_be_bytes();
            write_ttlv_header(
                &mut attr_bytes,
                [raw_tag[1], raw_tag[2], raw_tag[3]],
                0x01,
                st_bytes.len(),
            );
            attr_bytes.extend_from_slice(&st_bytes);
            (attr_bytes.as_ptr(), attr_bytes.len() as u64)
        } else {
            (ptr::null(), 0)
        };
        let mut out_result = kmipkit_s2c_operation_result_v1_t {
            struct_size: std::mem::size_of::<kmipkit_s2c_operation_result_v1_t>() as u32,
            abi_version: 1,
            result_status: 1,
            has_result_reason: 1,
            result_reason: 0x05,
            reserved: 0,
            result_message_data: ptr::null(),
            result_message_length: 0,
        };
        let mut deleted_content = Vec::new();
        for reference in request.deleted_attribute_references() {
            let mut content = Vec::new();
            encode_structure_view_bytes(&reference.view(), &mut content);
            write_ttlv_header(
                &mut deleted_content,
                [0x42, 0x01, 0x3b],
                0x01,
                content.len(),
            );
            deleted_content.extend_from_slice(&content);
        }
        let mut deleted = Vec::new();
        write_ttlv_header(
            &mut deleted,
            [0x42, 0x00, 0x79],
            0x01,
            deleted_content.len(),
        );
        deleted.extend_from_slice(&deleted_content);
        let code = if let Some(handler) = self.handler_v5 {
            let Some(callback) = handler.handle_notify else {
                return Err(s2c_failure_result());
            };
            unsafe {
                callback(
                    handler.user_data,
                    &view,
                    uid.as_ptr(),
                    uid.len() as u64,
                    attr_ptr,
                    attr_len,
                    deleted.as_ptr(),
                    deleted.len() as u64,
                    &mut out_result,
                )
            }
        } else {
            let Some(callback) = self.handler.handle_notify else {
                return Err(s2c_failure_result());
            };
            unsafe {
                callback(
                    self.handler.user_data,
                    &view,
                    uid.as_ptr(),
                    uid.len() as u64,
                    attr_ptr,
                    attr_len,
                    &mut out_result,
                )
            }
        };
        if code != 0 {
            let err_result = match parse_s2c_operation_result_v1(&out_result) {
                Ok(r) if r.status().raw() != 0 => r,
                _ => s2c_failure_result(),
            };
            return Err(err_result);
        }
        Ok(kmipkit_protocol::ServerNotifyResponse::new())
    }

    fn handle_put(
        &mut self,
        request: &kmipkit_protocol::ServerPutRequest,
    ) -> Result<kmipkit_protocol::ServerPutResponse, kmipkit_protocol::KmipOperationResult> {
        let callback = match self.handler.handle_put {
            Some(cb) => cb,
            None => return Err(s2c_failure_result()),
        };
        let uid = request.unique_identifier();
        let view = kmipkit_server_request_item_view_v1_t {
            struct_size: std::mem::size_of::<kmipkit_server_request_item_view_v1_t>() as u32,
            abi_version: 1,
            operation: 0x0000_001C,
            unique_batch_item_id_data: self
                .current_batch_item_id
                .as_ref()
                .map_or(ptr::null(), |id| id.as_ptr()),
            unique_batch_item_id_length: self
                .current_batch_item_id
                .as_ref()
                .map_or(0, |id| id.len() as u64),
        };
        let mut obj_bytes = Vec::new();
        kmipkit_ttlv::codec::encode_ttlv_item(request.object_item(), &mut obj_bytes);

        let mut attr_bytes = Vec::new();
        let (attr_ptr, attr_len) = if let Some(attrs) = request.attributes() {
            let mut st_bytes = Vec::new();
            encode_structure_view_bytes(&attrs.view(), &mut st_bytes);
            let raw_tag = 0x0042_0125u32.to_be_bytes();
            write_ttlv_header(
                &mut attr_bytes,
                [raw_tag[1], raw_tag[2], raw_tag[3]],
                0x01,
                st_bytes.len(),
            );
            attr_bytes.extend_from_slice(&st_bytes);
            (attr_bytes.as_ptr(), attr_bytes.len() as u64)
        } else {
            (ptr::null(), 0)
        };

        let (rep_ptr, rep_len) = if let Some(rep_uid) = request.replaced_unique_identifier() {
            (rep_uid.as_ptr(), rep_uid.len() as u64)
        } else {
            (ptr::null(), 0)
        };

        let mut out_result = kmipkit_s2c_operation_result_v1_t {
            struct_size: std::mem::size_of::<kmipkit_s2c_operation_result_v1_t>() as u32,
            abi_version: 1,
            result_status: 1,
            has_result_reason: 1,
            result_reason: 0x05,
            reserved: 0,
            result_message_data: ptr::null(),
            result_message_length: 0,
        };
        let code = unsafe {
            callback(
                self.handler.user_data,
                &view,
                request.put_function().raw(),
                uid.as_ptr(),
                uid.len() as u64,
                rep_ptr,
                rep_len,
                obj_bytes.as_ptr(),
                obj_bytes.len() as u64,
                attr_ptr,
                attr_len,
                &mut out_result,
            )
        };
        if code != 0 {
            let err_result = match parse_s2c_operation_result_v1(&out_result) {
                Ok(r) if r.status().raw() != 0 => r,
                _ => s2c_failure_result(),
            };
            return Err(err_result);
        }
        Ok(kmipkit_protocol::ServerPutResponse::new())
    }

    fn handle_query(
        &mut self,
        request: &kmipkit_protocol::ServerQueryRequest,
    ) -> Result<kmipkit_protocol::ServerQueryResponse, kmipkit_protocol::KmipOperationResult> {
        let view = kmipkit_server_request_item_view_v1_t {
            struct_size: std::mem::size_of::<kmipkit_server_request_item_view_v1_t>() as u32,
            abi_version: 1,
            operation: 0x0000_0018,
            unique_batch_item_id_data: self
                .current_batch_item_id
                .as_ref()
                .map_or(ptr::null(), |id| id.as_ptr()),
            unique_batch_item_id_length: self
                .current_batch_item_id
                .as_ref()
                .map_or(0, |id| id.len() as u64),
        };
        let mut flags: u32 = 0;
        for qf in request.query_functions() {
            let raw = qf.raw();
            if (1..=32).contains(&raw) {
                flags |= 1u32 << (raw - 1);
            }
        }
        let base = kmipkit_s2c_capabilities_v1_t {
            struct_size: std::mem::size_of::<kmipkit_s2c_capabilities_v1_t>() as u32,
            abi_version: 1,
            operations: ptr::null(),
            operations_count: 0,
            object_types: ptr::null(),
            object_types_count: 0,
            vendor_identification_data: ptr::null(),
            vendor_identification_length: 0,
            extension_tags: ptr::null(),
            extension_tags_count: 0,
            reserved: 0,
        };
        let mut out_result = kmipkit_s2c_operation_result_v1_t {
            struct_size: std::mem::size_of::<kmipkit_s2c_operation_result_v1_t>() as u32,
            abi_version: 1,
            result_status: 1,
            has_result_reason: 1,
            result_reason: 0x05,
            reserved: 0,
            result_message_data: ptr::null(),
            result_message_length: 0,
        };
        let mut out_caps = kmipkit_s2c_capabilities_v2_t {
            struct_size: std::mem::size_of::<kmipkit_s2c_capabilities_v2_t>() as u32,
            abi_version: 2,
            base,
            extension_information: ptr::null(),
            extension_information_count: 0,
            reserved: 0,
        };
        let rich_query = self.handler_v6.and_then(|handler| handler.handle_query);
        let code = if let Some(callback) = rich_query {
            unsafe {
                callback(
                    self.handler.user_data,
                    &view,
                    flags,
                    &mut out_caps,
                    &mut out_result,
                )
            }
        } else if let Some(callback) = self.handler.handle_query {
            unsafe {
                callback(
                    self.handler.user_data,
                    &view,
                    flags,
                    &mut out_caps.base,
                    &mut out_result,
                )
            }
        } else {
            return Err(s2c_failure_result());
        };
        if code != 0 {
            let err_result = match parse_s2c_operation_result_v1(&out_result) {
                Ok(r) if r.status().raw() != 0 => r,
                _ => s2c_failure_result(),
            };
            return Err(err_result);
        }
        let caps = if rich_query.is_some() {
            parse_s2c_capabilities_v2(&out_caps)
        } else {
            parse_s2c_capabilities_v1(&out_caps.base)
        }
        .map_err(|_| s2c_failure_result())?;
        Ok(kmipkit_protocol::ServerQueryResponse::respond(
            request, &caps,
        ))
    }

    fn handle_set_endpoint_role(
        &mut self,
        request: &kmipkit_protocol::SetEndpointRoleRequest,
    ) -> Result<kmipkit_protocol::SetEndpointRoleResponse, kmipkit_protocol::KmipOperationResult>
    {
        let view = kmipkit_server_request_item_view_v1_t {
            struct_size: std::mem::size_of::<kmipkit_server_request_item_view_v1_t>() as u32,
            abi_version: 1,
            operation: 0x0000_0032,
            unique_batch_item_id_data: self
                .current_batch_item_id
                .as_ref()
                .map_or(ptr::null(), |id| id.as_ptr()),
            unique_batch_item_id_length: self
                .current_batch_item_id
                .as_ref()
                .map_or(0, |id| id.len() as u64),
        };
        let mut out_result = kmipkit_s2c_operation_result_v1_t {
            struct_size: std::mem::size_of::<kmipkit_s2c_operation_result_v1_t>() as u32,
            abi_version: 1,
            result_status: 1,
            has_result_reason: 1,
            result_reason: 0x05,
            reserved: 0,
            result_message_data: ptr::null(),
            result_message_length: 0,
        };
        let mut response_role = request.endpoint_role().raw();
        let mut response_vendor = kmipkit_byte_span_v1_t {
            data: ptr::null(),
            length: 0,
        };
        let code = if let Some(handler) = self.handler_v5 {
            let Some(callback) = handler.handle_set_endpoint_role else {
                return Err(s2c_failure_result());
            };
            let vendor = request.vendor_identification();
            unsafe {
                callback(
                    handler.user_data,
                    &view,
                    request.endpoint_role().raw(),
                    vendor.map_or(ptr::null(), |v| v.as_ptr()),
                    vendor.map_or(0, |v| v.len() as u64),
                    &mut response_role,
                    &mut response_vendor,
                    &mut out_result,
                )
            }
        } else {
            let Some(callback) = self.handler.handle_set_endpoint_role else {
                return Err(s2c_failure_result());
            };
            unsafe {
                callback(
                    self.handler.user_data,
                    &view,
                    request.endpoint_role().raw(),
                    &mut out_result,
                )
            }
        };
        if code != 0 {
            let err_result = match parse_s2c_operation_result_v1(&out_result) {
                Ok(r) if r.status().raw() != 0 => r,
                _ => s2c_failure_result(),
            };
            return Err(err_result);
        }
        let vendor = if response_vendor.data.is_null() {
            if response_vendor.length != 0 {
                return Err(s2c_failure_result());
            }
            None
        } else {
            let bytes = input_span(
                response_vendor.data,
                response_vendor.length,
                MAX_FFI_INPUT_BYTES,
            )
            .map_err(|_| s2c_failure_result())?;
            Some(
                std::str::from_utf8(bytes)
                    .map_err(|_| s2c_failure_result())?
                    .to_owned(),
            )
        };
        Ok(kmipkit_protocol::SetEndpointRoleResponse::success(
            kmipkit_protocol::EndpointRole::from_raw(response_role),
            vendor,
        ))
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_request_dispatcher_create(
    out_dispatcher: *mut *mut kmipkit_server_request_dispatcher_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_dispatcher);
        *out = try_ffi!(try_make_handle(
            Kind::ServerRequestDispatcher,
            HandleValue::ServerRequestDispatcher(Arc::new(Mutex::new(
                kmipkit_client::ServerRequestDispatcher::new(),
            ))),
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_request_dispatcher_dispatch_v1(
    dispatcher: *mut kmipkit_server_request_dispatcher_t,
    batch: *mut kmipkit_incoming_server_batch_t,
    handler: *const kmipkit_server_request_handler_v1_t,
    out_response: *mut *mut kmipkit_response_message_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_response);
        if handler.is_null() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let handler_ref = unsafe { &*handler };
        let req_size = std::mem::size_of::<kmipkit_server_request_handler_v1_t>() as u32;
        if handler_ref.abi_version != 1 || handler_ref.struct_size != req_size {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let disp_handle = read_handle!(dispatcher, ServerRequestDispatcher);
        let HandleValue::ServerRequestDispatcher(disp) = &disp_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let batch_handle = read_handle!(batch, IncomingServerBatch);
        let HandleValue::IncomingServerBatch(batch_val) = &batch_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(mut disp) = disp.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        let mut adapter = CServerRequestHandlerAdapterV1 {
            handler: *handler_ref,
            current_batch_item_id: None,
        };
        match disp.dispatch(batch_val, &mut adapter) {
            Ok(resp) => {
                *out = try_ffi!(try_make_handle(
                    Kind::ResponseMessage,
                    HandleValue::ResponseMessage(Arc::new(resp)),
                ));
                SUCCESS
            }
            Err(err) => record_client_error(&err),
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_request_dispatcher_dispatch_v2(
    dispatcher: *mut kmipkit_server_request_dispatcher_t,
    batch: *mut kmipkit_incoming_server_batch_t,
    handler: *const kmipkit_server_request_handler_v2_t,
    out_response: *mut *mut kmipkit_response_message_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_response);
        if handler.is_null() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let handler_ref = unsafe { &*handler };
        let req_size = std::mem::size_of::<kmipkit_server_request_handler_v2_t>() as u32;
        if handler_ref.abi_version != 2 || handler_ref.struct_size != req_size {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let disp_handle = read_handle!(dispatcher, ServerRequestDispatcher);
        let HandleValue::ServerRequestDispatcher(disp) = &disp_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let batch_handle = read_handle!(batch, IncomingServerBatch);
        let HandleValue::IncomingServerBatch(batch_val) = &batch_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(mut disp) = disp.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        let mut adapter = CServerRequestHandlerAdapterV2 {
            handler: *handler_ref,
            current_batch_item_id: None,
        };
        match disp.dispatch(batch_val, &mut adapter) {
            Ok(resp) => {
                *out = try_ffi!(try_make_handle(
                    Kind::ResponseMessage,
                    HandleValue::ResponseMessage(Arc::new(resp)),
                ));
                SUCCESS
            }
            Err(err) => record_client_error(&err),
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_request_dispatcher_dispatch_v3(
    dispatcher: *mut kmipkit_server_request_dispatcher_t,
    batch: *mut kmipkit_incoming_server_batch_t,
    handler: *const kmipkit_server_request_handler_v3_t,
    out_response: *mut *mut kmipkit_response_message_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_response);
        if handler.is_null() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let handler_ref = unsafe { &*handler };
        let req_size = std::mem::size_of::<kmipkit_server_request_handler_v3_t>() as u32;
        if handler_ref.abi_version != 3 || handler_ref.struct_size != req_size {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let disp_handle = read_handle!(dispatcher, ServerRequestDispatcher);
        let HandleValue::ServerRequestDispatcher(disp) = &disp_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let batch_handle = read_handle!(batch, IncomingServerBatch);
        let HandleValue::IncomingServerBatch(batch_val) = &batch_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(mut disp) = disp.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        let mut adapter = CServerRequestHandlerAdapterV3 {
            handler: *handler_ref,
            handler_v4: None,
            handler_v5: None,
            handler_v6: None,
            current_batch_item_id: None,
        };
        match disp.dispatch(batch_val, &mut adapter) {
            Ok(resp) => {
                *out = try_ffi!(try_make_handle(
                    Kind::ResponseMessage,
                    HandleValue::ResponseMessage(Arc::new(resp)),
                ));
                SUCCESS
            }
            Err(err) => record_client_error(&err),
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_request_dispatcher_dispatch_v4(
    dispatcher: *mut kmipkit_server_request_dispatcher_t,
    batch: *mut kmipkit_incoming_server_batch_t,
    handler: *const kmipkit_server_request_handler_v4_t,
    out_response: *mut *mut kmipkit_response_message_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_response);
        if handler.is_null() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let handler_ref = unsafe { &*handler };
        let req_size = std::mem::size_of::<kmipkit_server_request_handler_v4_t>() as u32;
        if handler_ref.abi_version != 4 || handler_ref.struct_size != req_size {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let disp_handle = read_handle!(dispatcher, ServerRequestDispatcher);
        let HandleValue::ServerRequestDispatcher(disp) = &disp_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let batch_handle = read_handle!(batch, IncomingServerBatch);
        let HandleValue::IncomingServerBatch(batch_val) = &batch_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(mut disp) = disp.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        let mut adapter = CServerRequestHandlerAdapterV3 {
            handler: kmipkit_server_request_handler_v3_t {
                struct_size: std::mem::size_of::<kmipkit_server_request_handler_v3_t>() as u32,
                abi_version: 3,
                user_data: handler_ref.user_data,
                handle_discover_versions: None,
                handle_notify: handler_ref.handle_notify,
                handle_put: handler_ref.handle_put,
                handle_query: handler_ref.handle_query,
                handle_set_endpoint_role: handler_ref.handle_set_endpoint_role,
            },
            handler_v4: Some(*handler_ref),
            handler_v5: None,
            handler_v6: None,
            current_batch_item_id: None,
        };
        match disp.dispatch(batch_val, &mut adapter) {
            Ok(resp) => {
                *out = try_ffi!(try_make_handle(
                    Kind::ResponseMessage,
                    HandleValue::ResponseMessage(Arc::new(resp)),
                ));
                SUCCESS
            }
            Err(err) => record_client_error(&err),
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_request_dispatcher_dispatch_v5(
    dispatcher: *mut kmipkit_server_request_dispatcher_t,
    batch: *mut kmipkit_incoming_server_batch_t,
    handler: *const kmipkit_server_request_handler_v5_t,
    out_response: *mut *mut kmipkit_response_message_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_response);
        if handler.is_null() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let handler_ref = unsafe { &*handler };
        let req_size = std::mem::size_of::<kmipkit_server_request_handler_v5_t>() as u32;
        if handler_ref.abi_version != 5 || handler_ref.struct_size != req_size {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        dispatch_server_request_v5(dispatcher, batch, handler_ref, None, out)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_request_dispatcher_dispatch_v6(
    dispatcher: *mut kmipkit_server_request_dispatcher_t,
    batch: *mut kmipkit_incoming_server_batch_t,
    handler: *const kmipkit_server_request_handler_v6_t,
    out_response: *mut *mut kmipkit_response_message_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_response);
        if handler.is_null() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let handler = unsafe { &*handler };
        if handler.abi_version != 6
            || handler.struct_size
                != std::mem::size_of::<kmipkit_server_request_handler_v6_t>() as u32
            || handler.base.abi_version != 5
            || handler.base.struct_size
                != std::mem::size_of::<kmipkit_server_request_handler_v5_t>() as u32
        {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        dispatch_server_request_v5(dispatcher, batch, &handler.base, Some(*handler), out)
    })
}

fn dispatch_server_request_v5(
    dispatcher: *mut kmipkit_server_request_dispatcher_t,
    batch: *mut kmipkit_incoming_server_batch_t,
    handler_ref: &kmipkit_server_request_handler_v5_t,
    handler_v6: Option<kmipkit_server_request_handler_v6_t>,
    out: &mut *mut kmipkit_response_message_t,
) -> i32 {
    let disp_handle = read_handle!(dispatcher, ServerRequestDispatcher);
    let HandleValue::ServerRequestDispatcher(disp) = &disp_handle.value else {
        return record_ffi_error(ERROR_INVALID_INPUT);
    };
    let batch_handle = read_handle!(batch, IncomingServerBatch);
    let HandleValue::IncomingServerBatch(batch_val) = &batch_handle.value else {
        return record_ffi_error(ERROR_INVALID_INPUT);
    };
    let Ok(mut disp) = disp.lock() else {
        return record_ffi_error(ERROR_INTERNAL);
    };
    let mut adapter = CServerRequestHandlerAdapterV3 {
        handler: kmipkit_server_request_handler_v3_t {
            struct_size: std::mem::size_of::<kmipkit_server_request_handler_v3_t>() as u32,
            abi_version: 3,
            user_data: handler_ref.user_data,
            handle_discover_versions: None,
            handle_notify: None,
            handle_put: handler_ref.handle_put,
            handle_query: handler_ref.handle_query,
            handle_set_endpoint_role: None,
        },
        handler_v4: Some(kmipkit_server_request_handler_v4_t {
            struct_size: std::mem::size_of::<kmipkit_server_request_handler_v4_t>() as u32,
            abi_version: 4,
            user_data: handler_ref.user_data,
            handle_discover_versions: handler_ref.handle_discover_versions,
            handle_notify: None,
            handle_put: handler_ref.handle_put,
            handle_query: handler_ref.handle_query,
            handle_set_endpoint_role: None,
        }),
        handler_v5: Some(*handler_ref),
        handler_v6,
        current_batch_item_id: None,
    };
    match disp.dispatch(batch_val, &mut adapter) {
        Ok(resp) => {
            *out = try_ffi!(try_make_handle(
                Kind::ResponseMessage,
                HandleValue::ResponseMessage(Arc::new(resp)),
            ));
            SUCCESS
        }
        Err(err) => record_client_error(&err),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_server_request_dispatcher_release(
    dispatcher: *mut kmipkit_server_request_dispatcher_t,
) {
    ffi_entrypoint(|| release(dispatcher, Kind::ServerRequestDispatcher));
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_s2c_discover_versions_negotiate(
    requested_major: u32,
    requested_minor: u32,
    supported_versions: *const u32,
    supported_count: u32,
    out_negotiated_major: *mut u32,
    out_negotiated_minor: *mut u32,
) -> i32 {
    ffi_entrypoint(|| {
        let neg_maj = scalar_out!(out_negotiated_major);
        let neg_min = scalar_out!(out_negotiated_minor);
        if supported_versions.is_null() && supported_count > 0 {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let mut supported = Vec::new();
        for i in 0..(supported_count as usize) {
            let packed = unsafe { *supported_versions.add(i) };
            let maj = (packed >> 16) as i32;
            let min = (packed & 0xFFFF) as i32;
            supported.push(kmipkit_protocol::ProtocolVersion::from_raw(maj, min));
        }
        let req_version = kmipkit_protocol::ProtocolVersion::from_raw(
            requested_major as i32,
            requested_minor as i32,
        );
        let req = kmipkit_protocol::ServerDiscoverVersionsRequest::new(vec![req_version]);
        let resp = kmipkit_protocol::ServerDiscoverVersionsResponse::negotiate(&req, &supported);
        if let Some(best) = resp.protocol_versions().first() {
            *neg_maj = best.major() as u32;
            *neg_min = best.minor() as u32;
            SUCCESS
        } else {
            record_ffi_error(ERROR_COMPATIBILITY_MISMATCH)
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_s2c_discover_versions_negotiate_v2(
    requested: *const kmipkit_protocol_version_v1_t,
    requested_count: u32,
    supported: *const kmipkit_protocol_version_v1_t,
    supported_count: u32,
    output: *mut kmipkit_protocol_version_v1_t,
    output_capacity: u32,
    out_count: *mut u32,
) -> i32 {
    ffi_entrypoint(|| {
        let count = scalar_out!(out_count);
        let read_versions =
            |ptr: *const kmipkit_protocol_version_v1_t, len: u32| -> FfiResult<Vec<_>> {
                if len == 0 {
                    return Ok(Vec::new());
                }
                if ptr.is_null() {
                    return Err(ERROR_INVALID_INPUT);
                }
                if u64::from(len) > MAX_FFI_INPUT_BYTES / 8 {
                    return Err(ERROR_RESOURCE_LIMIT);
                }
                // The ABI requires readable input records for the specified length.
                Ok(unsafe { std::slice::from_raw_parts(ptr, len as usize) }
                    .iter()
                    .map(|v| kmipkit_protocol::ProtocolVersion::from_raw(v.major, v.minor))
                    .collect())
            };
        let request = kmipkit_protocol::ServerDiscoverVersionsRequest::new(try_ffi!(
            read_versions(requested, requested_count)
        ));
        let supported = try_ffi!(read_versions(supported, supported_count));
        let response =
            kmipkit_protocol::ServerDiscoverVersionsResponse::negotiate(&request, &supported);
        let versions = response.protocol_versions();
        *count = versions.len() as u32;
        if output.is_null() && output_capacity == 0 {
            return SUCCESS;
        }
        if output.is_null() || output_capacity < *count {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        for (i, version) in versions.iter().enumerate() {
            // The caller supplies enough writable records.
            unsafe {
                output.add(i).write(kmipkit_protocol_version_v1_t {
                    major: version.major(),
                    minor: version.minor(),
                });
            }
        }
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_s2c_put_effective_function(
    requested_put_function: u32,
    replaced_object_exists_locally: u8,
    out_effective_put_function: *mut u32,
) -> i32 {
    ffi_entrypoint(|| {
        let eff = scalar_out!(out_effective_put_function);
        let pf = kmipkit_protocol::PutFunction::from_raw(requested_put_function);
        let exists = replaced_object_exists_locally != 0;
        let effective = if pf == kmipkit_protocol::PutFunction::REPLACE && !exists {
            kmipkit_protocol::PutFunction::NEW
        } else {
            pf
        };
        *eff = effective.raw();
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_s2c_query_respond_with_capabilities_v1(
    query_function_flags: u32,
    capabilities: *const kmipkit_s2c_capabilities_v1_t,
    out_structure: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_structure);
        let caps = try_ffi!(parse_s2c_capabilities_v1(capabilities));
        build_s2c_query_response(
            query_function_flags,
            kmipkit_protocol::ProtocolVersion::V2_1,
            &caps,
            out,
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_s2c_query_respond_with_capabilities_v2(
    query_function_flags: u32,
    version: kmipkit_protocol_version_v1_t,
    capabilities: *const kmipkit_s2c_capabilities_v2_t,
    out_structure: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_structure);
        let version = kmipkit_protocol::ProtocolVersion::from_raw(version.major, version.minor);
        let caps = try_ffi!(parse_s2c_capabilities_v2(capabilities));
        build_s2c_query_response(query_function_flags, version, &caps, out)
    })
}

fn build_s2c_query_response(
    query_function_flags: u32,
    version: kmipkit_protocol::ProtocolVersion,
    caps: &kmipkit_protocol::ServerCapabilities,
    out: &mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    let functions = (1..=32)
        .filter(|raw| query_function_flags & (1 << (raw - 1)) != 0)
        .map(kmipkit_protocol::QueryFunction::from_raw)
        .collect();
    let request = kmipkit_protocol::ServerQueryRequest::new(functions);
    let response = kmipkit_protocol::ServerQueryResponse::respond(&request, caps);
    let structure = try_ffi!(
        response
            .to_ttlv_payload_for_version(version)
            .map_err(|error| protocol_status(&error))
    );
    *out = try_ffi!(try_make_handle(
        Kind::Structure,
        HandleValue::Structure(structure)
    ));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_s2c_query_respond_ttlv(
    query_function_flags: u32,
    vendor_id_data: *const u8,
    vendor_id_len: u64,
    out_structure: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let caps = kmipkit_s2c_capabilities_v1_t {
            struct_size: std::mem::size_of::<kmipkit_s2c_capabilities_v1_t>() as u32,
            abi_version: 1,
            operations: [0x0000_001E, 0x0000_001B, 0x0000_001C, 0x0000_0018].as_ptr(),
            operations_count: 4,
            object_types: [0x01, 0x02, 0x07].as_ptr(),
            object_types_count: 3,
            vendor_identification_data: vendor_id_data,
            vendor_identification_length: vendor_id_len,
            extension_tags: ptr::null(),
            extension_tags_count: 0,
            reserved: 0,
        };
        kmipkit_s2c_query_respond_with_capabilities_v1(query_function_flags, &caps, out_structure)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_set_endpoint_role(
    client: *mut kmipkit_client_t,
    endpoint_role: u32,
    vendor_identification: *const u8,
    vendor_identification_length: u64,
    limits: *mut kmipkit_codec_limits_t,
    out_response: *mut *mut kmipkit_client_batch_item_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        kmipkit_client_set_endpoint_role_with_options_v1(
            client,
            endpoint_role,
            vendor_identification,
            vendor_identification_length,
            limits,
            ptr::null(),
            out_response,
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_set_endpoint_role_with_options_v1(
    client: *mut kmipkit_client_t,
    endpoint_role: u32,
    vendor_identification: *const u8,
    vendor_identification_length: u64,
    limits: *mut kmipkit_codec_limits_t,
    options: *const kmipkit_request_options_v1_t,
    out_response: *mut *mut kmipkit_client_batch_item_response_t,
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
        let request_options = if options.is_null() {
            RequestOptions::default()
        } else {
            match request_options_from_v1(unsafe { &*options }) {
                Ok(opt) => opt,
                Err(status) => return record_ffi_error(status),
            }
        };
        let role = kmipkit_protocol::EndpointRole::from_raw(endpoint_role);
        let mut request = kmipkit_protocol::SetEndpointRoleRequest::new(role);
        if !vendor_identification.is_null() && vendor_identification_length > 0 {
            let span = try_ffi!(input_span(
                vendor_identification,
                vendor_identification_length,
                MAX_FFI_INPUT_BYTES,
            ));
            let text = match std::str::from_utf8(span) {
                Ok(t) => t,
                Err(_) => return record_ffi_error(ERROR_INVALID_INPUT),
            };
            request = request.with_vendor_identification(text);
        } else if !vendor_identification.is_null() || vendor_identification_length != 0 {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let Ok(mut client) = client.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        let batch = ClientBatch::new(ClientBatchItem::set_endpoint_role(request));
        match client.execute_with_options(batch, limits, &request_options) {
            Ok(batch_resp) => {
                let arc_resp = Arc::new(batch_resp);
                *out = try_ffi!(try_make_handle(
                    Kind::ClientBatchItemResponse,
                    HandleValue::ClientBatchItemResponse {
                        response: arc_resp,
                        index: 0,
                    },
                ));
                SUCCESS
            }
            Err(error) => record_client_error(&error),
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_version_gate_operation_span(
    operation: u32,
    out_introduced_major: *mut u32,
    out_introduced_minor: *mut u32,
    out_deprecated_major: *mut u32,
    out_deprecated_minor: *mut u32,
    out_removed_major: *mut u32,
    out_removed_minor: *mut u32,
) -> i32 {
    ffi_entrypoint(|| {
        let intro_maj = scalar_out!(out_introduced_major);
        let intro_min = scalar_out!(out_introduced_minor);
        let depr_maj = scalar_out!(out_deprecated_major);
        let depr_min = scalar_out!(out_deprecated_minor);
        let rem_maj = scalar_out!(out_removed_major);
        let rem_min = scalar_out!(out_removed_minor);

        match kmipkit_protocol::VersionGate::operation_span(operation) {
            Some(span) => {
                *intro_maj = span.introduced.major() as u32;
                *intro_min = span.introduced.minor() as u32;
                *depr_maj = span.deprecated.map(|v| v.major() as u32).unwrap_or(0);
                *depr_min = span.deprecated.map(|v| v.minor() as u32).unwrap_or(0);
                *rem_maj = span.removed.map(|v| v.major() as u32).unwrap_or(0);
                *rem_min = span.removed.map(|v| v.minor() as u32).unwrap_or(0);
                SUCCESS
            }
            None => record_ffi_error(ERROR_INVALID_INPUT),
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_version_gate_tag_span(
    tag: u32,
    out_introduced_major: *mut u32,
    out_introduced_minor: *mut u32,
    out_deprecated_major: *mut u32,
    out_deprecated_minor: *mut u32,
    out_removed_major: *mut u32,
    out_removed_minor: *mut u32,
) -> i32 {
    ffi_entrypoint(|| {
        let intro_maj = scalar_out!(out_introduced_major);
        let intro_min = scalar_out!(out_introduced_minor);
        let depr_maj = scalar_out!(out_deprecated_major);
        let depr_min = scalar_out!(out_deprecated_minor);
        let rem_maj = scalar_out!(out_removed_major);
        let rem_min = scalar_out!(out_removed_minor);

        match kmipkit_protocol::VersionGate::tag_span(tag) {
            Some(span) => {
                *intro_maj = span.introduced.major() as u32;
                *intro_min = span.introduced.minor() as u32;
                *depr_maj = span.deprecated.map(|v| v.major() as u32).unwrap_or(0);
                *depr_min = span.deprecated.map(|v| v.minor() as u32).unwrap_or(0);
                *rem_maj = span.removed.map(|v| v.major() as u32).unwrap_or(0);
                *rem_min = span.removed.map(|v| v.minor() as u32).unwrap_or(0);
                SUCCESS
            }
            None => record_ffi_error(ERROR_INVALID_INPUT),
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_version_gate_validate_attribute_mutation(
    protocol_version_major: u32,
    protocol_version_minor: u32,
    attribute_tag: u32,
) -> i32 {
    ffi_entrypoint(|| {
        let pv = kmipkit_protocol::ProtocolVersion::from_raw(
            protocol_version_major as i32,
            protocol_version_minor as i32,
        );
        match kmipkit_protocol::VersionGate::validate_attribute_mutation(pv, attribute_tag) {
            Ok(()) => SUCCESS,
            Err(_) => record_ffi_error(ERROR_INVALID_INPUT),
        }
    })
}
