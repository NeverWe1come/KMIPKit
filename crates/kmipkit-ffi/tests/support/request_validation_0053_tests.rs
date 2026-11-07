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

#[test]
fn validation_report_abi_rejects_invalid_and_released_handles() {
    let mut report = std::ptr::null_mut();
    // SAFETY: Null input is explicitly accepted as invalid, and the report output slot is live.
    let status = unsafe {
        super::request_validation::kmipkit_ttlv_structure_validation_report_v1(
            std::ptr::null_mut(),
            std::ptr::null(),
            0,
            &raw mut report,
        )
    };
    assert_eq!(status, ERROR_INVALID_INPUT);
    assert!(report.is_null());

    let mut structure = std::ptr::null_mut();
    assert_eq!(kmipkit_ttlv_structure_create(&raw mut structure), SUCCESS);
    let schema = b"RequestPayload/Create";
    assert_eq!(
        // SAFETY: The structure and output handles are live and the schema span is readable.
        unsafe {
            super::request_validation::kmipkit_ttlv_structure_validation_report_v1(
                structure,
                schema.as_ptr(),
                schema.len() as u64,
                &raw mut report,
            )
        },
        SUCCESS
    );
    assert!(!report.is_null());

    let mut view = kmipkit_validation_issue_view_v1_t {
        struct_size: u32::try_from(std::mem::size_of::<kmipkit_validation_issue_view_v1_t>())
            .expect("the versioned test record size fits u32"),
        abi_version: 2,
        ..kmipkit_validation_issue_view_v1_t::default()
    };
    assert_eq!(
        // SAFETY: The report handle is live, and `view` is a writable versioned output record.
        unsafe {
            super::request_validation::kmipkit_validation_report_issue_at_v1(
                report,
                0,
                &raw mut view,
            )
        },
        ERROR_INVALID_INPUT
    );
    view.abi_version = 1;
    assert_eq!(
        // SAFETY: The report handle is live, and `view` is a writable versioned output record.
        unsafe {
            super::request_validation::kmipkit_validation_report_issue_at_v1(
                report,
                99,
                &raw mut view,
            )
        },
        ERROR_INVALID_INPUT
    );

    super::request_validation::kmipkit_validation_report_release(report);
    let mut is_valid = 0;
    assert_eq!(
        super::request_validation::kmipkit_validation_report_is_valid(report, &raw mut is_valid),
        ERROR_INVALID_INPUT
    );
    kmipkit_ttlv_structure_release(structure);
}
