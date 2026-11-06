use crate::{
    kmipkit_byte_span_v1_t, kmipkit_profile_evidence_record_v1_t,
    kmipkit_profile_readiness_blocker_view_v1_t, kmipkit_profile_readiness_t,
    kmipkit_profile_readiness_view_v1_t,
};
use std::mem::size_of;
use std::ptr;

const BASELINE: &[u8] = b"KMIPKIT-PROFILE-BASELINE-CLIENT";
const UNKNOWN_KIND: &[u8] = b"unknown_requirement_record";
const UNKNOWN_MESSAGE: &[u8] = b"requirement evidence record is unknown";

fn span_equals(span: kmipkit_byte_span_v1_t, expected: &[u8]) -> bool {
    let Ok(length) = usize::try_from(span.length) else {
        return false;
    };
    if length != expected.len() || (length != 0 && span.data.is_null()) {
        return false;
    }
    if length == 0 {
        return true;
    }
    // SAFETY: The ABI view span is read while its returned handle remains live and the FFI
    // contract guarantees that the span points to `length` initialized bytes.
    let actual = unsafe { std::slice::from_raw_parts(span.data, length) };
    actual == expected
}

fn assert_empty_or_whitespace_id_is_redacted(requirement_id: &[u8]) {
    let profile_span = kmipkit_byte_span_v1_t {
        data: BASELINE.as_ptr(),
        length: u64::try_from(BASELINE.len()).expect("profile identifier length fits u64"),
    };
    let record = kmipkit_profile_evidence_record_v1_t {
        struct_size: u32::try_from(size_of::<kmipkit_profile_evidence_record_v1_t>())
            .expect("ABI record size fits u32"),
        abi_version: 1,
        requirement_id: kmipkit_byte_span_v1_t {
            data: if requirement_id.is_empty() {
                ptr::null()
            } else {
                requirement_id.as_ptr()
            },
            length: u64::try_from(requirement_id.len()).expect("identifier length fits u64"),
        },
        passed: 1,
        ..Default::default()
    };
    let mut handle = ptr::null_mut::<kmipkit_profile_readiness_t>();
    // SAFETY: Inputs remain live and disjoint for the call; the output handle slot is separate.
    let status = unsafe {
        crate::extension_registry::kmipkit_profile_readiness_evaluate_v1(
            &raw const profile_span,
            &raw const record,
            1,
            &raw mut handle,
        )
    };
    assert_eq!(status, 0);
    assert!(!handle.is_null());

    let mut view = kmipkit_profile_readiness_view_v1_t {
        struct_size: u32::try_from(size_of::<kmipkit_profile_readiness_view_v1_t>())
            .expect("ABI view size fits u32"),
        abi_version: 1,
        ..Default::default()
    };
    // SAFETY: `handle` is live and `view` is aligned writable storage that does not alias inputs.
    let status = unsafe {
        crate::extension_registry::kmipkit_profile_readiness_view_get_v1(handle, &raw mut view)
    };
    assert_eq!(status, 0);

    let mut found_unknown = false;
    for index in 0..view.blocker_count {
        let mut blocker = kmipkit_profile_readiness_blocker_view_v1_t {
            struct_size: u32::try_from(size_of::<kmipkit_profile_readiness_blocker_view_v1_t>())
                .expect("ABI blocker view size fits u32"),
            abi_version: 1,
            ..Default::default()
        };
        // SAFETY: The handle remains live and the output view is separate aligned storage.
        let status = unsafe {
            crate::extension_registry::kmipkit_profile_readiness_blocker_at_v1(
                handle,
                index,
                &raw mut blocker,
            )
        };
        assert_eq!(status, 0);
        if span_equals(blocker.kind, UNKNOWN_KIND) {
            assert!(span_equals(blocker.message, UNKNOWN_MESSAGE));
            assert_eq!(blocker.requirement_id_present, 0);
            assert!(blocker.requirement_id.data.is_null());
            found_unknown = true;
        }
    }

    crate::extension_registry::kmipkit_profile_readiness_release(handle);
    assert!(found_unknown);
}

#[test]
fn c_abi_treats_empty_and_whitespace_unknown_requirement_ids_as_redacted_blockers() {
    for requirement_id in [b"".as_slice(), b" \t\r\n".as_slice()] {
        assert_empty_or_whitespace_id_is_redacted(requirement_id);
    }
}
