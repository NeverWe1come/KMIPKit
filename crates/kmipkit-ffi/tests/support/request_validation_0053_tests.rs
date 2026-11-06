use super::*;

unsafe extern "C" {
    fn kmipkit_request_validation_c_consumer() -> i32;
}

#[test]
fn compiled_c_consumer_reads_safe_validation_metadata() {
    // SAFETY: The consumer owns and releases all TTLV and validation handles.
    let status = unsafe { kmipkit_request_validation_c_consumer() };
    assert_eq!(status, SUCCESS);
}
