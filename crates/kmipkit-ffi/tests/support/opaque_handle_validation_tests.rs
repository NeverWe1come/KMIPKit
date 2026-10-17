use std::process::Command;

const CHILD_MODE: &str = "KMIPKIT_OPAQUE_HANDLE_TEST_CHILD";

#[test]
fn arbitrary_wrong_kind_and_stale_handles_are_rejected_in_child_process() {
    if std::env::var_os(CHILD_MODE).is_some() {
        assert_eq!(call_tag_value(forged_tag()), super::ERROR_INVALID_INPUT);
        super::kmipkit_tag_release(forged_tag());

        let mut raw_tag = std::ptr::null_mut();
        assert_eq!(
            super::kmipkit_ttlv_raw_tag_create(0x0042_0001, &raw mut raw_tag),
            super::SUCCESS
        );
        let mut raw = 0;

        for guessed_id in 1..=64 {
            let guessed = std::ptr::without_provenance_mut(guessed_id);
            assert_eq!(
                super::kmipkit_ttlv_raw_tag_value(guessed, &raw mut raw),
                super::ERROR_INVALID_INPUT,
                "predictable handle token {guessed_id} reached a live handle"
            );
        }

        super::kmipkit_tag_release(raw_tag.cast());
        assert_eq!(
            super::kmipkit_ttlv_raw_tag_value(raw_tag, &raw mut raw),
            super::SUCCESS
        );
        assert_eq!(raw, 0x0042_0001);

        let mut tag = std::ptr::null_mut();
        assert_eq!(
            super::kmipkit_ttlv_raw_tag_try_checked(raw_tag, &raw mut tag),
            super::SUCCESS
        );

        let wrong_kind = tag.cast::<super::kmipkit_raw_tag_t>();
        assert_eq!(
            super::kmipkit_ttlv_raw_tag_value(wrong_kind, &raw mut raw),
            super::ERROR_INVALID_INPUT
        );
        super::kmipkit_raw_tag_release(wrong_kind);
        let mut tag_value = 0;
        assert_eq!(call_tag_value(tag), super::SUCCESS);
        assert_eq!(
            super::kmipkit_ttlv_tag_value(tag, &raw mut tag_value),
            super::SUCCESS
        );
        assert_eq!(tag_value, 0x0042_0001);

        super::kmipkit_tag_release(tag);
        super::kmipkit_tag_release(tag);
        assert_eq!(call_tag_value(tag), super::ERROR_INVALID_INPUT);

        let mut replacement_raw = std::ptr::null_mut();
        assert_eq!(
            super::kmipkit_ttlv_raw_tag_create(0x0042_0001, &raw mut replacement_raw),
            super::SUCCESS
        );
        let mut replacement_tag = std::ptr::null_mut();
        assert_eq!(
            super::kmipkit_ttlv_raw_tag_try_checked(replacement_raw, &raw mut replacement_tag),
            super::SUCCESS
        );
        assert_ne!(tag, replacement_tag);
        assert_eq!(call_tag_value(tag), super::ERROR_INVALID_INPUT);
        assert_eq!(call_tag_value(replacement_tag), super::SUCCESS);
        super::kmipkit_tag_release(replacement_tag);
        super::kmipkit_raw_tag_release(replacement_raw);
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

#[test]
fn tokens_are_keyed_nonzero_and_unique_without_reusing_sequences() {
    let registry = super::HandleRegistry::with_test_key(&[0x5a; 32]);
    let mut tokens = std::collections::HashSet::new();

    for sequence in 1..=4096 {
        let token = registry
            .token_for_sequence(sequence)
            .expect("the keyed token should be found within the configured bound");
        assert_ne!(token, 0);
        assert!(token <= super::TOKEN_MAX);
        assert_ne!(token, sequence);
        assert!(tokens.insert(token));
    }
}

#[test]
fn token_sequence_exhaustion_fails_closed() {
    let registry = super::HandleRegistry::with_test_key(&[0xa5; 32]);
    registry
        .next_sequence
        .store(super::TOKEN_MAX, std::sync::atomic::Ordering::Relaxed);

    assert!(registry.allocate_id().is_ok());
    assert_eq!(registry.allocate_id(), Err(super::ERROR_RESOURCE_LIMIT));
    assert_eq!(registry.allocate_id(), Err(super::ERROR_RESOURCE_LIMIT));
}

#[test]
fn token_cycle_walking_stops_at_its_configured_limit() {
    let mut calls = 0;
    let invalid_token = super::TOKEN_MAX + 1;

    assert_eq!(
        super::cycle_walk_token(1, |_| {
            calls += 1;
            invalid_token
        }),
        Err(super::ERROR_RESOURCE_LIMIT)
    );
    assert_eq!(calls, super::TOKEN_CYCLE_WALK_LIMIT);
}

fn forged_tag() -> *mut super::kmipkit_tag_t {
    std::ptr::without_provenance_mut(0x1000)
}

fn call_tag_value(tag: *mut super::kmipkit_tag_t) -> i32 {
    let mut value = 0;
    super::kmipkit_ttlv_tag_value(tag, &raw mut value)
}
