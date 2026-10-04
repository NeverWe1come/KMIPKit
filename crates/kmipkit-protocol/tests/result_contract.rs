use std::error::Error;
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use kmipkit_protocol::{
    KmipOperationResult, ProtocolCauseCategory, ProtocolError, ProtocolErrorKind, ResultMessage,
    ResultReason, ResultStatus,
};

#[test]
fn known_status_values_match_catalog() {
    let values = ResultStatus::known_values();
    assert_ne!(values, []);

    for (status, expected_name) in values {
        assert_eq!(status.known_name(), Some(*expected_name));
        assert_eq!(
            ResultStatus::from_raw(status.raw()).known_name(),
            Some(*expected_name)
        );
    }
}

#[test]
fn known_reason_values_match_catalog() {
    let values = ResultReason::known_values();
    assert_ne!(values, []);

    for (reason, expected_name) in values {
        assert_eq!(reason.known_name(), Some(*expected_name));
        assert_eq!(
            ResultReason::from_raw(reason.raw()).known_name(),
            Some(*expected_name)
        );
    }
}

#[test]
fn unknown_status_preserves_raw_value() {
    let status = ResultStatus::from_raw(u32::MAX);

    assert_eq!(status.raw(), u32::MAX);
    assert_eq!(status.known_name(), None);
}

#[test]
fn unknown_reason_preserves_raw_value() {
    let reason = ResultReason::from_raw(u32::MAX);

    assert_eq!(reason.raw(), u32::MAX);
    assert_eq!(reason.known_name(), None);
}

#[test]
fn result_message_preserves_presence_and_text() {
    let success = known_status("Success");
    let absent = KmipOperationResult::new(success, None, None).expect("valid success result");
    let empty = KmipOperationResult::new(success, None, Some(ResultMessage::new(String::new())))
        .expect("valid success result with empty message");
    let text = "arbitrary server text: \u{1f512} \nnot trusted";
    let present =
        KmipOperationResult::new(success, None, Some(ResultMessage::new(text.to_owned())))
            .expect("valid success result with message");

    assert!(absent.message().is_none());
    assert_eq!(empty.message().map(ResultMessage::as_bytes), Some(&[][..]));
    assert_eq!(present.message().map(ResultMessage::as_str), Some(text));
    assert_eq!(
        present.message().map(ResultMessage::as_bytes),
        Some(text.as_bytes())
    );
}

#[test]
fn failure_requires_a_reason() {
    let failure = known_status("Operation Failed");

    assert!(KmipOperationResult::new(failure, None, None).is_err());
}

#[test]
fn success_forbids_a_reason() {
    let success = known_status("Success");
    let reason = ResultReason::from_raw(0);

    assert!(KmipOperationResult::new(success, Some(reason), None).is_err());
}

#[test]
fn other_statuses_do_not_gain_reason_presence_rules() {
    let pending = known_status("Operation Pending");
    let undone = known_status("Operation Undone");
    let unknown = ResultStatus::from_raw(u32::MAX);

    for status in [pending, undone, unknown] {
        assert!(KmipOperationResult::new(status, None, None).is_ok());
        assert!(
            KmipOperationResult::new(status, Some(ResultReason::from_raw(u32::MAX)), None).is_ok()
        );
    }
}

#[test]
fn result_message_debug_is_redacted() {
    let sentinel = "RESULT_MESSAGE_SECRET_SENTINEL";
    let message = ResultMessage::new(sentinel.to_owned());

    assert!(!format!("{message:?}").contains(sentinel));
}

#[test]
fn operation_result_display_and_debug_redact_message() {
    let sentinel = "RESULT_MESSAGE_SECRET_SENTINEL";
    let result = KmipOperationResult::new(
        known_status("Success"),
        None,
        Some(ResultMessage::new(sentinel.to_owned())),
    )
    .expect("valid success result");

    assert!(!format!("{result}").contains(sentinel));
    assert!(!format!("{result:?}").contains(sentinel));
}

#[test]
fn protocol_error_drops_untrusted_source_and_redacts_its_chain() {
    let dropped = Arc::new(AtomicBool::new(false));
    let sentinel = "PRIVATE_KEY_SENTINEL";
    let error = ProtocolError::new(
        ProtocolErrorKind::MalformedMessage,
        ProtocolCauseCategory::InvalidValue,
        DropProbe {
            dropped: Arc::clone(&dropped),
            text: sentinel,
        },
    );

    assert!(dropped.load(Ordering::Acquire));
    assert!(!format!("{error}").contains(sentinel));
    assert!(!format!("{error:?}").contains(sentinel));

    let mut source = error.source();
    while let Some(current) = source {
        assert!(!format!("{current}").contains(sentinel));
        assert!(!format!("{current:?}").contains(sentinel));
        source = current.source();
    }
}

fn known_status(name: &str) -> ResultStatus {
    ResultStatus::known_values()
        .iter()
        .find_map(|(status, known_name)| (*known_name == name).then_some(*status))
        .unwrap_or_else(|| panic!("missing catalog status {name}"))
}

struct DropProbe<'a> {
    dropped: Arc<AtomicBool>,
    text: &'a str,
}

impl fmt::Display for DropProbe<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.text)
    }
}

impl fmt::Debug for DropProbe<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.text)
    }
}

impl Error for DropProbe<'_> {}

impl Drop for DropProbe<'_> {
    fn drop(&mut self) {
        self.dropped.store(true, Ordering::Release);
    }
}
