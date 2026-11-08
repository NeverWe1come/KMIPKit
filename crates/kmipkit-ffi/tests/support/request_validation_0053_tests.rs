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

#[test]
fn validation_report_abi_covers_generic_known_unsupported_and_invalid_schemas() {
    let mut structure = std::ptr::null_mut();
    assert_eq!(kmipkit_ttlv_structure_create(&raw mut structure), SUCCESS);

    let mut report = std::ptr::null_mut();
    // SAFETY: Empty schema requests generic validation; the live structure and output slot do not overlap.
    assert_eq!(
        unsafe {
            super::request_validation::kmipkit_ttlv_structure_validation_report_v1(
                structure,
                std::ptr::null(),
                0,
                &raw mut report,
            )
        },
        SUCCESS
    );
    let mut valid = u8::MAX;
    assert_eq!(
        super::request_validation::kmipkit_validation_report_is_valid(report, &raw mut valid),
        SUCCESS
    );
    assert_eq!(valid, 1);
    let mut issue_count = u64::MAX;
    assert_eq!(
        super::request_validation::kmipkit_validation_report_issue_count(
            report,
            &raw mut issue_count,
        ),
        SUCCESS
    );
    assert_eq!(issue_count, 0);
    super::request_validation::kmipkit_validation_report_release(report);

    let unsupported = b"Unsupported";
    report = std::ptr::null_mut();
    // SAFETY: The schema slice and output handle slot are valid for the call.
    assert_eq!(
        unsafe {
            super::request_validation::kmipkit_ttlv_structure_validation_report_v1(
                structure,
                unsupported.as_ptr(),
                unsupported.len() as u64,
                &raw mut report,
            )
        },
        SUCCESS
    );
    issue_count = 0;
    assert_eq!(
        super::request_validation::kmipkit_validation_report_issue_count(
            report,
            &raw mut issue_count,
        ),
        SUCCESS
    );
    assert_eq!(issue_count, 1);
    super::request_validation::kmipkit_validation_report_release(report);

    for schema in [b"RequestPayload/Unknown".as_slice(), b"Nested/Unknown"] {
        report = std::ptr::null_mut();
        // SAFETY: The invalid but readable schema slice and writable output slot are live.
        assert_eq!(
            unsafe {
                super::request_validation::kmipkit_ttlv_structure_validation_report_v1(
                    structure,
                    schema.as_ptr(),
                    schema.len() as u64,
                    &raw mut report,
                )
            },
            ERROR_INVALID_INPUT
        );
        assert!(report.is_null());
    }

    let mut item = std::ptr::null_mut();
    assert_eq!(
        super::kmipkit_client_batch_item_ping(&raw mut item),
        SUCCESS
    );
    report = std::ptr::null_mut();
    // SAFETY: The BatchItem is live and the output slot is writable and distinct.
    assert_eq!(
        unsafe {
            super::request_validation::kmipkit_client_batch_item_validation_report(
                item,
                &raw mut report,
            )
        },
        SUCCESS
    );
    valid = u8::MAX;
    assert_eq!(
        super::request_validation::kmipkit_validation_report_is_valid(report, &raw mut valid),
        SUCCESS
    );
    assert_eq!(valid, 1);
    super::request_validation::kmipkit_validation_report_release(report);
    super::kmipkit_client_batch_item_release(item);
    kmipkit_ttlv_structure_release(structure);
}

#[test]
fn validation_report_issue_reader_copies_safe_metadata_and_checks_abi_bounds() {
    let mut structure = std::ptr::null_mut();
    assert_eq!(kmipkit_ttlv_structure_create(&raw mut structure), SUCCESS);
    let schema = b"RequestPayload/Create";
    let mut report = std::ptr::null_mut();
    // SAFETY: The live empty Structure is validated under a known Request Payload schema.
    assert_eq!(
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

    let record_size = u32::try_from(std::mem::size_of::<kmipkit_validation_issue_view_v1_t>())
        .expect("versioned ABI record size fits u32");
    let mut issue = kmipkit_validation_issue_view_v1_t {
        struct_size: record_size,
        abi_version: 1,
        ..kmipkit_validation_issue_view_v1_t::default()
    };
    // SAFETY: `issue` is an initialized, writable version 1 output record and report is live.
    assert_eq!(
        unsafe {
            super::request_validation::kmipkit_validation_report_issue_at_v1(
                report,
                0,
                &raw mut issue,
            )
        },
        SUCCESS
    );
    assert_eq!(issue.abi_version, 1);
    assert_eq!(issue.severity, 1);
    assert!(issue.code.length > 0);
    assert!(issue.path.length > 0);
    assert!(issue.source_identifier.length > 0);
    assert!(issue.source_reference.length > 0);

    assert_eq!(
        // SAFETY: Null output is rejected before dereferencing.
        unsafe {
            super::request_validation::kmipkit_validation_report_issue_at_v1(
                report,
                0,
                std::ptr::null_mut(),
            )
        },
        ERROR_INVALID_INPUT
    );
    issue.abi_version = 2;
    assert_eq!(
        // SAFETY: The initialized record is readable and writable but has an unsupported version.
        unsafe {
            super::request_validation::kmipkit_validation_report_issue_at_v1(
                report,
                0,
                &raw mut issue,
            )
        },
        ERROR_INVALID_INPUT
    );
    issue.abi_version = 1;
    assert_eq!(
        // SAFETY: The live report is queried with an out-of-range index and a valid output record.
        unsafe {
            super::request_validation::kmipkit_validation_report_issue_at_v1(
                report,
                u64::MAX,
                &raw mut issue,
            )
        },
        ERROR_INVALID_INPUT
    );

    super::request_validation::kmipkit_validation_report_release(report);
    kmipkit_ttlv_structure_release(structure);
}
