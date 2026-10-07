#![allow(unsafe_code)]
#![cfg(all(feature = "coverage-c-consumer", target_os = "linux"))]

use std::ffi::{CString, c_char};
use std::path::Path;

use kmipkit_ffi::kmipkit_codec_limits_t;

#[link(name = "kmipkit_extension_registry_c_consumer", kind = "static")]
unsafe extern "C" {
    fn kmipkit_extension_registry_c_consumer_main(argc: i32, argv: *mut *mut c_char) -> i32;
}

#[link(name = "kmipkit_ffi")]
unsafe extern "C" {
    fn kmipkit_codec_limits_defaults(out_limits: *mut *mut kmipkit_codec_limits_t) -> i32;
    fn kmipkit_codec_limits_release(limits: *mut kmipkit_codec_limits_t);
}

#[test]
fn existing_c_consumer_exercises_the_exported_abi_in_the_coverage_process() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/extensions/cases.json")
        .canonicalize()
        .expect("shared C consumer fixture exists");
    let executable = CString::new("kmipkit_extension_registry_consumer")
        .expect("test executable name has no NUL byte");
    let fixture = CString::new(fixture.to_str().expect("fixture path is UTF-8"))
        .expect("fixture path has no NUL byte");
    let mut arguments = [executable.as_ptr().cast_mut(), fixture.as_ptr().cast_mut()];
    let mut limits = std::ptr::null_mut();

    // SAFETY: `limits` is a valid writable output slot and every C string and argument pointer
    // remains live for each call; the C consumer reads these arguments and uses valid handles.
    let result = unsafe {
        let status = kmipkit_codec_limits_defaults(&mut limits);
        if status != 0 || limits.is_null() {
            return assert_eq!(status, 0, "codec limits defaults must succeed");
        }
        kmipkit_codec_limits_release(limits);
        kmipkit_extension_registry_c_consumer_main(arguments.len() as i32, arguments.as_mut_ptr())
    };

    assert_eq!(result, 0, "the existing C ABI behavior suite must pass");
}
