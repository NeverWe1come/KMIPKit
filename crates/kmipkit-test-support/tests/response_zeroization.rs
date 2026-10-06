//! Lifecycle tests for response fixtures owned by the deterministic fake.
//! These are test-support project-policy tests, not OASIS conformance vectors.
//!
//! Traceability: `KMIPKIT-0007-FR-013`, `-SC-009`; ADR-0014; OASIS KMIP v2.1
//! §9.12, Table 417 for bounded fake exchange behavior.

use kmipkit_test_support::{ExchangeScript, ResponseDropObserver, ScriptedTransport};
use kmipkit_transport::{RequestDeliveryState, Transport};

const RESPONSE: &[u8] = b"SCRIPTED_RESPONSE_SECRET_SENTINEL";

#[test]
fn successful_exchange_zeroizes_the_owned_response_fixture() {
    let observer = ResponseDropObserver::new(RESPONSE.len());
    let mut transport = ScriptedTransport::new(ExchangeScript::Success {
        response: RESPONSE.to_vec(),
        request_write_chunks: vec![1],
    })
    .with_response_drop_observer(observer.clone());

    let response = transport
        .exchange(b"request", usize::MAX)
        .expect("scripted success returns its response");

    assert_eq!(response.as_bytes(), RESPONSE);
    assert!(observer.initialized_bytes_were_zeroized());
}

#[test]
fn partial_read_error_zeroizes_the_owned_response_fixture() {
    let observer = ResponseDropObserver::new(RESPONSE.len());
    let mut transport = ScriptedTransport::new(ExchangeScript::FailAfterPartialRead {
        written_bytes: 1,
        response_bytes: RESPONSE.to_vec(),
    })
    .with_response_drop_observer(observer.clone());

    let error = transport
        .exchange(b"request", usize::MAX)
        .expect_err("scripted partial read fails");

    assert_eq!(
        error.delivery_state(),
        RequestDeliveryState::ResponseStarted
    );
    assert!(observer.initialized_bytes_were_zeroized());
}

#[test]
fn dropping_an_unconsumed_script_zeroizes_its_response_fixture() {
    let observer = ResponseDropObserver::new(RESPONSE.len());
    let transport = ScriptedTransport::new(ExchangeScript::Success {
        response: RESPONSE.to_vec(),
        request_write_chunks: vec![1],
    })
    .with_response_drop_observer(observer.clone());

    drop(transport);

    assert!(observer.initialized_bytes_were_zeroized());
}

#[test]
fn exhausted_script_rejects_a_second_exchange_without_a_new_script() {
    let mut transport = ScriptedTransport::new(ExchangeScript::FailBeforeWrite);
    let first = transport
        .exchange(b"request", usize::MAX)
        .expect_err("the configured failure is returned");
    let second = transport
        .exchange(b"request", usize::MAX)
        .expect_err("an exhausted script does not synthesize a response");

    assert_eq!(first.delivery_state(), RequestDeliveryState::NotSent);
    assert_eq!(second.delivery_state(), RequestDeliveryState::NotSent);
}

#[test]
fn dropping_unconsumed_write_failure_scripts_is_safe() {
    drop(ScriptedTransport::new(ExchangeScript::FailBeforeWrite));
    drop(ScriptedTransport::new(
        ExchangeScript::FailAfterPartialWrite { written_bytes: 2 },
    ));
}
