#![allow(unsafe_code)]

use std::ffi::{CString, c_char};
use std::path::Path;

unsafe extern "C" {
    fn kmipkit_extension_registry_c_consumer_main(argc: i32, argv: *mut *mut c_char) -> i32;
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
    let mut arguments = [
        executable.as_ptr().cast_mut(),
        fixture.as_ptr().cast_mut(),
    ];

    // SAFETY: both C strings and the argument vector remain live for the duration of the call;
    // the existing consumer reads these arguments and calls only its documented KMIPKit handles.
    let result = unsafe {
        kmipkit_extension_registry_c_consumer_main(arguments.len() as i32, arguments.as_mut_ptr())
    };

    assert_eq!(result, 0, "the existing C ABI behavior suite must pass");
}
