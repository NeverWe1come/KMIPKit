#![cfg(feature = "coverage-c-consumer")]

#[link(name = "kmipkit_operation_parity_0053_consumer", kind = "static")]
unsafe extern "C" {
    fn kmipkit_operation_parity_0053_all_56() -> i32;
}

#[link(name = "kmipkit_client_abi_c_consumer", kind = "static")]
unsafe extern "C" {}

#[link(name = "kmipkit_ffi")]
unsafe extern "C" {
    fn kmipkit_client_batch_create(out_batch: *mut *mut core::ffi::c_void) -> i32;
    fn kmipkit_client_batch_release(batch: *mut core::ffi::c_void);
}

#[test]
fn compiled_c_consumer_invokes_all_56_client_operation_routes() {
    let mut batch = core::ptr::null_mut();
    // SAFETY: the null output is writable, and the created batch is released below.
    assert_eq!(unsafe { kmipkit_client_batch_create(&mut batch) }, 0);
    assert!(!batch.is_null());
    // SAFETY: `batch` is the live handle returned by the ABI constructor.
    unsafe { kmipkit_client_batch_release(batch) };

    // SAFETY: the C consumer owns and releases every handle it creates.
    let result = unsafe { kmipkit_operation_parity_0053_all_56() };

    assert_eq!(
        result, 48,
        "the matrix must batch 48 operation items; the existing C consumer covers Interop, Join Split Key, Logout, and PKCS#11, and negative results identify the first failing route"
    );
}
