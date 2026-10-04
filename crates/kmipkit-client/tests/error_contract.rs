use std::error::Error;
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use kmipkit_client::{ClientCauseCategory, ClientError, ClientErrorCategory};
use kmipkit_protocol::{
    KmipOperationResult, ProtocolCauseCategory, ProtocolError, ProtocolErrorKind, ResultMessage,
    ResultReason, ResultStatus,
};
use kmipkit_transport::{RequestDeliveryState, TransportCauseCategory, TransportError};

#[test]
fn local_error_constructors_require_delivery_evidence() {
    let _: fn(
        ClientCauseCategory,
        RequestDeliveryState,
        std::io::Error,
    ) -> ClientError = ClientError::validation::<std::io::Error>;
    let _: fn(ProtocolError, RequestDeliveryState) -> ClientError = ClientError::protocol;
}

#[test]
fn local_failure_categories_are_distinct() {
    let validation = ClientError::validation(
        ClientCauseCategory::InvalidInput,
        Some(RequestDeliveryState::NotSent),
        synthetic_error("validation source"),
    );
    let protocol = ClientError::protocol(
        ProtocolError::new(
            ProtocolErrorKind::MalformedMessage,
            ProtocolCauseCategory::InvalidEncoding,
            synthetic_error("protocol source"),
        ),
        Some(RequestDeliveryState::ResponseStarted),
    );
    let transport = ClientError::transport(TransportError::new(
        RequestDeliveryState::PossiblySent,
        TransportCauseCategory::Tls,
        synthetic_error("transport source"),
    ));

    assert_eq!(validation.category(), ClientErrorCategory::Validation);
    assert_eq!(protocol.category(), ClientErrorCategory::Protocol);
    assert_eq!(transport.category(), ClientErrorCategory::Transport);
    for error in [&validation, &protocol, &transport] {
        assert_eq!(error.server_operation_result(), None);
        assert_eq!(error.server_status(), None);
    }
}

#[test]
fn safe_cause_categories_and_delivery_evidence_remain_inspectable() {
    let validation = ClientError::validation(
        ClientCauseCategory::InvalidInput,
        Some(RequestDeliveryState::NotSent),
        synthetic_error("unsafe validation text"),
    );
    let protocol = ClientError::protocol(
        ProtocolError::new(
            ProtocolErrorKind::InvalidValue,
            ProtocolCauseCategory::InvalidValue,
            synthetic_error("unsafe protocol text"),
        ),
        Some(RequestDeliveryState::ResponseStarted),
    );
    let transport = ClientError::transport(TransportError::new(
        RequestDeliveryState::PossiblySent,
        TransportCauseCategory::Timeout,
        synthetic_error("unsafe transport text"),
    ));

    assert_eq!(
        validation.cause_category(),
        Some(ClientCauseCategory::InvalidInput)
    );
    assert_eq!(
        validation.delivery_state(),
        Some(RequestDeliveryState::NotSent)
    );
    assert_eq!(
        protocol.cause_category(),
        Some(ClientCauseCategory::Protocol(
            ProtocolCauseCategory::InvalidValue
        ))
    );
    assert_eq!(
        protocol.delivery_state(),
        Some(RequestDeliveryState::ResponseStarted)
    );
    assert_eq!(
        transport.cause_category(),
        Some(ClientCauseCategory::Transport(
            TransportCauseCategory::Timeout
        ))
    );
    assert_eq!(
        transport.delivery_state(),
        Some(RequestDeliveryState::PossiblySent)
    );
    assert!(!format!("{protocol}").contains("unsafe protocol text"));
    assert!(!format!("{protocol:?}").contains("unsafe protocol text"));
    assert!(!format!("{transport}").contains("unsafe transport text"));
    assert!(!format!("{transport:?}").contains("unsafe transport text"));
    assert_eq!(
        protocol
            .cause_category()
            .map(|category| category.to_string()),
        Some("protocol: invalid value".to_owned())
    );
    assert_eq!(
        transport
            .cause_category()
            .map(|category| category.to_string()),
        Some("transport: timeout".to_owned())
    );
    assert!(protocol.source().is_some());
    assert!(transport.source().is_some());
}

#[test]
fn other_validation_cause_has_safe_formatting_and_source() {
    let error = ClientError::validation(
        ClientCauseCategory::Other,
        None,
        synthetic_error("sensitive validation source"),
    );

    assert_eq!(error.cause_category(), Some(ClientCauseCategory::Other));
    assert!(error.to_string().contains("other validation cause"));
    assert!(!format!("{error:?}").contains("sensitive validation source"));
    assert!(error.source().is_some());
}

#[test]
fn unsafe_local_sources_are_dropped_and_unreachable_from_every_source_chain() {
    let sentinels = [
        "CREDENTIAL_SENTINEL",
        "PRIVATE_KEY_SENTINEL",
        "SECRET_KEY_MATERIAL_SENTINEL",
        "ONE_TIME_PASSWORD_SENTINEL",
        "TICKET_SENTINEL",
        "RAW_KMIP_BODY_SENTINEL",
    ];

    for sentinel in sentinels {
        let dropped = Arc::new(AtomicBool::new(false));
        let error = ClientError::validation(
            ClientCauseCategory::InvalidInput,
            Some(RequestDeliveryState::NotSent),
            DropProbe {
                dropped: Arc::clone(&dropped),
                text: sentinel.to_owned(),
            },
        );

        assert!(dropped.load(Ordering::Acquire));
        assert_redacted(&error, sentinel);
    }
}

#[test]
fn complete_server_result_is_distinct_redacted_and_has_no_delivery_state() {
    let sentinel = "SERVER_RESULT_MESSAGE_SENTINEL";
    let result = KmipOperationResult::new(
        known_status("Operation Failed"),
        Some(known_reason("Item Not Found")),
        Some(ResultMessage::new(sentinel.to_owned())),
    )
    .expect("failed result includes a reason");
    let error = ClientError::server_result(result);

    assert_eq!(error.category(), ClientErrorCategory::ServerResult);
    assert_eq!(error.cause_category(), None);
    assert_eq!(error.delivery_state(), None);
    assert_redacted(&error, sentinel);
}

fn assert_redacted(error: &ClientError, sentinel: &str) {
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

fn known_reason(name: &str) -> ResultReason {
    ResultReason::known_values()
        .iter()
        .find_map(|(reason, known_name)| (*known_name == name).then_some(*reason))
        .unwrap_or_else(|| panic!("missing catalog reason {name}"))
}

struct DropProbe {
    dropped: Arc<AtomicBool>,
    text: String,
}

impl fmt::Display for DropProbe {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.text)
    }
}

impl fmt::Debug for DropProbe {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.text)
    }
}

impl Error for DropProbe {}

impl Drop for DropProbe {
    fn drop(&mut self) {
        self.dropped.store(true, Ordering::Release);
    }
}

fn synthetic_error(text: &str) -> DropProbe {
    DropProbe {
        dropped: Arc::new(AtomicBool::new(false)),
        text: text.to_owned(),
    }
}
