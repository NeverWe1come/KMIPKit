use super::*;

#[test]
fn poll_execution_clears_output_before_rejecting_an_invalid_correlation_span() {
    let mut output = 1_usize as *mut crate::kmipkit_client_async_operation_outcome_t;

    let status = kmipkit_client_execute_poll(
        ptr::null_mut(),
        ptr::null(),
        1,
        ptr::null_mut(),
        &mut output,
    );

    assert_eq!(status, ERROR_INVALID_INPUT);
    assert!(output.is_null());
}
