//! Rust declarations for C ABI functions exercised by linked C consumers.
//!
//! These symbols are resolved from the `kmipkit-ffi` Rust crate dependency, so
//! the tests do not depend on Cargo placing the `cdylib` in a native search path.
#![allow(dead_code)] // Each integration-test binary exercises a different subset.

use kmipkit_ffi::{kmipkit_client_batch_t, kmipkit_codec_limits_t};

unsafe extern "C" {
    pub(crate) fn kmipkit_client_batch_create(out_batch: *mut *mut kmipkit_client_batch_t) -> i32;
    pub(crate) fn kmipkit_client_batch_release(batch: *mut kmipkit_client_batch_t);
    pub(crate) fn kmipkit_codec_limits_defaults(
        out_limits: *mut *mut kmipkit_codec_limits_t,
    ) -> i32;
    pub(crate) fn kmipkit_codec_limits_create(
        max_message_bytes: u64,
        max_structure_depth: u64,
        max_elements: u64,
        out_limits: *mut *mut kmipkit_codec_limits_t,
    ) -> i32;
    pub(crate) fn kmipkit_codec_limits_release(limits: *mut kmipkit_codec_limits_t);
    pub(crate) fn kmipkit_last_error_get(
        out_status: *mut i32,
        out_category: *mut u32,
        out_cause: *mut u32,
        out_delivery: *mut u32,
    ) -> i32;
}
