//! Minimal host-native library used only by the offline package smoke tests.

use core::ffi::c_void;
use core::ptr;

/// Returns the host-native test fixture's deliberately incompatible ABI revision.
///
/// # Safety
///
/// Each output pointer must be aligned, writable for one `u32`, and disjoint
/// from the other output pointers. Null pointers are rejected before writes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kmipkit_version_get(
    major: *mut u32,
    minor: *mut u32,
    patch: *mut u32,
    abi_version: *mut u32,
) -> i32 {
    if major.is_null() || minor.is_null() || patch.is_null() || abi_version.is_null() {
        return 1;
    }

    // SAFETY: Each pointer was checked for null and the C ABI caller provides
    // one writable u32 output for each argument.
    unsafe {
        ptr::write(major, 1);
        ptr::write(minor, 0);
        ptr::write(patch, 0);
        ptr::write(abi_version, 2);
    }
    0
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_org_kmipkit_NativeExtensionRegistry_nativeAbiVersion(
    _environment: *mut c_void,
    _class: *mut c_void,
) -> i32 {
    2
}
