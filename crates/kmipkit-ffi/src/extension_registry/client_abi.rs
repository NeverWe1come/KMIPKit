//! Common validated client and transport C ABI implementation.
#![allow(clippy::wildcard_imports)] // Shares the parent module's private FFI handle invariants.

use super::*;
use zeroize::{Zeroize, Zeroizing};

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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
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
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_pkcs11_correlation_value(
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
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

#[cfg(test)]
#[derive(Clone, Default)]
struct SecretBufferObserver(std::sync::Arc<std::sync::Mutex<Vec<bool>>>);

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
            HandleValue::TransportConfig(configuration),
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
        );
        let transport_configuration = match owned_payload(transport_handle, Kind::TransportConfig) {
            Ok(HandleValue::TransportConfig(configuration)) => configuration,
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
            HandleValue::ClientBatch(std::sync::Mutex::new(ClientBatchBuilder::new())),
        ));
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
        )
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
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
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_execute_with_options_v1(
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
        execute_client_batch(client, batch, limits, out_response, None, &options)
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
    let batch = match batch_payload.into_inner() {
        Ok(batch) => batch.into_batch(),
        Err(_) => return record_ffi_error(ERROR_INTERNAL),
    };
    let Ok(mut client) = client.lock() else {
        return record_ffi_error(ERROR_INTERNAL);
    };
    match client.execute_with_options(batch, limits, request_options) {
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
        )
    })
}

/// Executes a batch with request options and returns its generic TTLV on a typed response error.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_execute_with_options_and_error_ttlv_v1(
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
pub extern "C" fn kmipkit_client_batch_item_response_metadata_v2(
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
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

fn write_unique_identifier_v1(
    out_identifier: *mut kmipkit_unique_identifier_v1_t,
    identifier: Option<&kmipkit_protocol::UniqueIdentifier>,
) -> FfiResult<()> {
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
    let mut output = kmipkit_unique_identifier_v1_t {
        struct_size,
        abi_version: 1,
        kind: 0,
        reserved: 0,
        text_data: ptr::null(),
        text_length: 0,
        enumeration_value: 0,
        integer_value: 0,
    };
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
            _ => return Err(ERROR_INVALID_INPUT), // COVERAGE-EXCLUDE: the current UniqueIdentifier model has only text, enumeration, and integer variants.
        }
    }
    // SAFETY: the caller supplied a writable record with the checked v1 size.
    unsafe { out_identifier.write(output) };
    Ok(())
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        let outcome = item.outcome();
        if outcome.result().status().raw() != 0 {
            return SUCCESS;
        }
        if let Some(recertify) = outcome.recertify_response() {
            let structure =
                try_ffi!(recertify.with_ttlv(|view| super::clone_structure_view(&view)));
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        let outcome = item.outcome();
        if outcome.result().status().raw() != 0 {
            return SUCCESS;
        }
        if let Some(rekey) = outcome.rekey_response() {
            let structure = try_ffi!(rekey.with_ttlv(|view| super::clone_structure_view(&view)));
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        if item.outcome().result().status().raw() != 0 {
            return SUCCESS;
        }
        if let Some(result) = item.outcome().rekey_key_pair_response() {
            let structure = try_ffi!(result.with_ttlv(|view| super::clone_structure_view(&view)));
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        if item.outcome().result().status().raw() != 0 {
            return SUCCESS;
        }
        if let Some(reprovision) = item.outcome().reprovision_response() {
            let structure =
                try_ffi!(reprovision.with_ttlv(|view| super::clone_structure_view(&view)));
            *response_ttlv = try_ffi!(try_make_handle(
                Kind::Structure,
                HandleValue::Structure(structure),
            ));
            *present = 1;
        }
        SUCCESS
    })
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        if item.outcome().result().status().raw() != 0 {
            return SUCCESS;
        }
        if let kmipkit_client::ClientBatchOutcome::Register(register) = item.outcome() {
            let Some(identifier) = register.unique_identifier() else {
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

/// Copies the complete ordered response TTLV only from successful Register results.
/// Output regions must be writable and must not overlap.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_register_ttlv(
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        if item.outcome().result().status().raw() != 0 {
            return SUCCESS;
        }
        if let kmipkit_client::ClientBatchOutcome::Register(register) = item.outcome() {
            let structure = try_ffi!(register.with_ttlv(|view| super::clone_structure_view(&view)));
            *response_ttlv = try_ffi!(try_make_handle(
                Kind::Structure,
                HandleValue::Structure(structure),
            ));
            *present = 1;
        }
        SUCCESS
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        let outcome = item.outcome();
        if outcome.result().status().raw() != 0 {
            return SUCCESS;
        }
        if let kmipkit_client::ClientBatchOutcome::ObtainLease(lease) = outcome {
            let structure = try_ffi!(lease.with_ttlv(|view| super::clone_structure_view(&view)));
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
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

        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
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

        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
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

        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        *present = 0;
        *count = 0;
        if let Some(create) = item.outcome().create_split_key_response() {
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
        let Some(create) = item.outcome().create_split_key_response() else {
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
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

/// Reads successful RNG Retrieve output bytes borrowed from `item`.
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        if item.outcome().result().status().raw() != 0 {
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

/// Copies the complete generic TTLV for a successful RNG Retrieve response.
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        if item.outcome().result().status().raw() != 0 {
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

/// Reads successful RNG Seed's server-reported Data Length, preserving zero.
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        if item.outcome().result().status().raw() != 0 {
            return SUCCESS;
        }
        let response_view = item.outcome().response();
        let Some(rng_seed) = response_view.rng_seed() else {
            return SUCCESS;
        };
        let Some(value) = rng_seed.data_length() else {
            return record_ffi_error(ERROR_INTERNAL); // COVERAGE-EXCLUDE: a successful parsed result has required Data Length.
        };
        *data_length = value;
        *data_length_present = 1;
        *present = 1;
        SUCCESS
    })
}

/// Copies the complete generic TTLV for a successful RNG Seed response.
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        if item.outcome().result().status().raw() != 0 {
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
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

/// Copies the typed supported-version list from a successful Discover Versions result.
///
/// Call with `out_versions == NULL` and `capacity == 0` to query presence and
/// the required count. A present empty successful response sets `out_present`
/// to 1 and `out_count` to 0. Pending, failed, and other-operation results set
/// both outputs to 0. A second call with sufficient caller-owned capacity
/// copies values in source order, retaining repetitions.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_response_discover_versions_supported_versions(
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
        let item = read_handle!(item, ClientBatchItemResponse);
        let HandleValue::ClientBatchItemResponse { response, index } = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(item) = response.get(*index) else {
            return record_ffi_error(ERROR_INTERNAL);
        };
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
