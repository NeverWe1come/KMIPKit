#![deny(unused_unsafe)]

use super::*;

// This helper is intentionally compiled but not invoked. Under `deny(unused_unsafe)`, every
// explicit unsafe block below proves that the corresponding raw-pointer entry point requires an
// unsafe Rust call. C callers retain the same ABI.
fn raw_pointer_ffi_call_sites() {
    let _ = unsafe {
        client_abi::kmipkit_client_batch_item_response_pkcs11_correlation_value(
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
        )
    };
    let _ = unsafe {
        client_abi::kmipkit_client_async_operation_outcome_metadata_v1(
            ptr::null_mut(),
            ptr::null_mut(),
        )
    };
    let _ = unsafe {
        client_abi::kmipkit_client_execute_with_options_v1(
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null(),
            ptr::null_mut(),
        )
    };
    let _ = unsafe {
        client_abi::kmipkit_client_execute_with_options_and_error_ttlv_v1(
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null(),
            ptr::null_mut(),
            ptr::null_mut(),
        )
    };
    let _ = unsafe {
        client_abi::kmipkit_client_batch_item_response_metadata_v2(
            ptr::null_mut(),
            ptr::null_mut(),
        )
    };
    let _ = unsafe {
        client_abi::kmipkit_client_batch_item_response_get_attribute_list_payload_ttlv(
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
        )
    };
    let _ = unsafe {
        client_abi::kmipkit_client_batch_item_response_get_attribute_list(
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            0,
            ptr::null_mut(),
        )
    };
    let _ = unsafe {
        client_abi::kmipkit_client_batch_item_response_discover_versions_supported_versions(
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            0,
            ptr::null_mut(),
        )
    };
    let _ = unsafe {
        kmipkit_api_levels_get_v1(ptr::null_mut(), ptr::null_mut(), ptr::null_mut())
    };
    let _ = unsafe {
        kmipkit_client_batch_item_validate(
            ptr::null_mut(),
            0,
            ptr::null(),
            0,
            ptr::null_mut(),
            0,
            0,
            0,
            ptr::null_mut(),
        )
    };
}

#[test]
fn pointer_dereferencing_c_exports_require_unsafe_rust_callers() {
    std::hint::black_box(raw_pointer_ffi_call_sites as fn());
}
