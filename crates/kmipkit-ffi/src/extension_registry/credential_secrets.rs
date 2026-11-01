//! Opaque, zeroizing owners for credential bytes and UTF-8 text.

use std::ffi::c_void;

use super::{
    ERROR_INVALID_INPUT, ERROR_RESOURCE_LIMIT, FfiResult, HandleValue, Kind, SecretBytes, SecretText,
    SUCCESS, consume_unshared_owner, ffi_entrypoint, input_span, kmipkit_secret_bytes_t,
    kmipkit_secret_text_t, kmipkit_ttlv_value_t, ptr, record_ffi_error, reference_handle, release,
    try_make_handle, unwrap_payload,
};
use zeroize::Zeroize;

type SecretBytesCallback = unsafe extern "C" fn(*const u8, u64, *mut c_void);
type SecretTextCallback = unsafe extern "C" fn(*const u8, u64, *mut c_void);

fn copy_secret_input(data: *const u8, length: u64) -> FfiResult<Vec<u8>> {
    let input = input_span(data, length, u64::MAX)?;
    let mut owned = Vec::new();
    owned
        .try_reserve_exact(input.len())
        .map_err(|_| ERROR_RESOURCE_LIMIT)?;
    owned.extend_from_slice(input);
    Ok(owned)
}

fn invalid_utf8_bytes(error: std::string::FromUtf8Error) {
    let mut bytes = error.into_bytes();
    bytes.zeroize();
}

/// Copies arbitrary bytes into a native zeroizing secret owner.
///
/// A null input pointer is accepted only when `length` is zero. The caller's input remains
/// caller-owned and is not modified. Allocation failure returns the stable resource-limit status.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_secret_bytes_create(
    data: *const u8,
    length: u64,
    out_secret: *mut *mut kmipkit_secret_bytes_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_secret);
        let bytes = try_ffi!(copy_secret_input(data, length));
        *out = try_ffi!(try_make_handle(
            Kind::SecretBytes,
            HandleValue::SecretBytes(SecretBytes::new(bytes))
        ));
        SUCCESS
    })
}

/// Calls a C callback with a borrowed byte span that expires when the callback returns.
///
/// The callback must not retain the pointer, unwind or jump nonlocally across the ABI, or use the
/// span asynchronously. It may copy the bytes, but that copy is outside `KMIPKit`'s zeroization
/// guarantee. No registry lock is held while the callback runs.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_secret_bytes_with_bytes(
    secret: *mut kmipkit_secret_bytes_t,
    callback: Option<SecretBytesCallback>,
    context: *mut c_void,
) -> i32 {
    ffi_entrypoint(|| {
        let Some(callback) = callback else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let owner = try_ffi!(reference_handle(secret, Kind::SecretBytes));
        let HandleValue::SecretBytes(secret) = &owner.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let callback_result = secret.with_bytes(|bytes| -> FfiResult<()> {
            let length = u64::try_from(bytes.len()).map_err(|_| ERROR_RESOURCE_LIMIT)?;
            // SAFETY: `owner` keeps the secret allocation alive through callback return, and the
            // registry lock was released by `reference_handle`. The callback's pointer is scoped
            // to this synchronous call by contract.
            unsafe { callback(bytes.as_ptr(), length, context) };
            Ok(())
        });
        try_ffi!(callback_result);
        SUCCESS
    })
}

/// Moves the native secret allocation into a zeroizing TTLV Byte String value.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_secret_bytes_into_ttlv_value(
    secret: *mut kmipkit_secret_bytes_t,
    out_value: *mut *mut kmipkit_ttlv_value_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_value);
        let owner = try_ffi!(consume_unshared_owner(secret, Kind::SecretBytes));
        let payload = try_ffi!(unwrap_payload(owner));
        let HandleValue::SecretBytes(secret) = payload else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = try_ffi!(try_make_handle(
            Kind::Value,
            HandleValue::Value(secret.into_ttlv_value())
        ));
        SUCCESS
    })
}

/// Releases a secret byte handle and zeroizes its currently owned allocation.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_secret_bytes_release(secret: *mut kmipkit_secret_bytes_t) {
    ffi_entrypoint(|| release(secret, Kind::SecretBytes));
}

/// Copies strict UTF-8 text into a native zeroizing secret owner.
///
/// Text is preserved exactly: no normalization or C-string termination is applied. A null input
/// pointer is accepted only when `length` is zero. Invalid UTF-8 is rejected without including
/// the input bytes in diagnostics, and the rejected native copy is zeroized.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_secret_text_create(
    utf8: *const u8,
    length: u64,
    out_secret: *mut *mut kmipkit_secret_text_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_secret);
        let bytes = try_ffi!(copy_secret_input(utf8, length));
        let text = match String::from_utf8(bytes) {
            Ok(text) => text,
            Err(error) => {
                invalid_utf8_bytes(error);
                return record_ffi_error(ERROR_INVALID_INPUT);
            }
        };
        *out = try_ffi!(try_make_handle(
            Kind::SecretText,
            HandleValue::SecretText(SecretText::new(text))
        ));
        SUCCESS
    })
}

/// Calls a C callback with borrowed UTF-8 bytes that expire when the callback returns.
///
/// The callback must not retain the pointer, unwind or jump nonlocally across the ABI, or use the
/// span asynchronously. It may copy the bytes, but that copy is outside `KMIPKit`'s zeroization
/// guarantee. No registry lock is held while the callback runs.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_secret_text_with_str(
    secret: *mut kmipkit_secret_text_t,
    callback: Option<SecretTextCallback>,
    context: *mut c_void,
) -> i32 {
    ffi_entrypoint(|| {
        let Some(callback) = callback else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let owner = try_ffi!(reference_handle(secret, Kind::SecretText));
        let HandleValue::SecretText(secret) = &owner.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let callback_result = secret.with_str(|text| -> FfiResult<()> {
            let length = u64::try_from(text.len()).map_err(|_| ERROR_RESOURCE_LIMIT)?;
            // SAFETY: `owner` keeps the secret allocation alive through callback return, and the
            // registry lock was released by `reference_handle`. The callback's pointer is scoped
            // to this synchronous call by contract.
            unsafe { callback(text.as_ptr(), length, context) };
            Ok(())
        });
        try_ffi!(callback_result);
        SUCCESS
    })
}

/// Moves the native secret allocation into a zeroizing TTLV Text String value.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_secret_text_into_ttlv_value(
    secret: *mut kmipkit_secret_text_t,
    out_value: *mut *mut kmipkit_ttlv_value_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_value);
        let owner = try_ffi!(consume_unshared_owner(secret, Kind::SecretText));
        let payload = try_ffi!(unwrap_payload(owner));
        let HandleValue::SecretText(secret) = payload else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = try_ffi!(try_make_handle(
            Kind::Value,
            HandleValue::Value(secret.into_ttlv_value())
        ));
        SUCCESS
    })
}

/// Releases a secret text handle and zeroizes its currently owned allocation.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_secret_text_release(secret: *mut kmipkit_secret_text_t) {
    ffi_entrypoint(|| release(secret, Kind::SecretText));
}
