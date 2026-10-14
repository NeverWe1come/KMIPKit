use std::process::Command;

const CHILD_MODE: &str = "KMIPKIT_OPAQUE_HANDLE_TEST_CHILD";

#[test]
fn arbitrary_wrong_kind_and_stale_handles_are_rejected_in_child_process() {
    if std::env::var_os(CHILD_MODE).is_some() {
        assert_eq!(call_tag_value(forged_tag()), super::ERROR_INVALID_INPUT);

        let mut raw_tag = std::ptr::null_mut();
        assert_eq!(
            super::kmipkit_ttlv_raw_tag_create(0x4200_0000, &mut raw_tag),
            super::SUCCESS
        );

        let mut tag = std::ptr::null_mut();
        assert_eq!(
            super::kmipkit_ttlv_raw_tag_try_checked(raw_tag, &mut tag),
            super::SUCCESS
        );

        let wrong_kind = tag.cast::<super::kmipkit_raw_tag_t>();
        let mut raw = 0;
        assert_eq!(
            super::kmipkit_ttlv_raw_tag_value(wrong_kind, &mut raw),
            super::ERROR_INVALID_INPUT
        );

        super::kmipkit_tag_release(tag);
        assert_eq!(call_tag_value(tag), super::ERROR_INVALID_INPUT);
        super::kmipkit_raw_tag_release(raw_tag);
        return;
    }

    let output = Command::new(std::env::current_exe().expect("test executable path"))
        .arg("--exact")
        .arg(concat!(
            "extension_registry::opaque_handle_validation_tests::",
            "arbitrary_wrong_kind_and_stale_handles_are_rejected_in_child_process"
        ))
        .arg("--nocapture")
        .env(CHILD_MODE, "1")
        .output()
        .expect("spawn isolated handle validation test");

    assert!(
        output.status.success(),
        "isolated FFI handle validation failed (status: {:?})\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn forged_tag() -> *mut super::kmipkit_tag_t {
    std::ptr::without_provenance_mut(0x1000)
}

fn call_tag_value(tag: *mut super::kmipkit_tag_t) -> i32 {
    let mut value = 0;
    super::kmipkit_ttlv_tag_value(tag, &mut value)
}
