use kmipkit_test_support::{EphemeralPki, fixtures};
use std::time::Duration;

use crate::Transport;
use crate::config::{Endpoint, RequestOptions, TimeoutLimit};
use crate::raw_tls::{
    CandidateEvent, CandidateEventObserver, RawTlsTransport, ResponseAllocationObserver,
    new_for_test, new_for_test_with_candidate_observer, new_for_test_with_lifecycle_controls,
    new_for_test_with_resolver, new_for_test_with_response_allocation_observer,
};
use crate::transport_test_support::{
    client_config, client_config_with_server_name, fixed_resolver, loopback_listener,
    server_config, spawn_raw_tls_peer,
};

const RESPONSE_FRAME: &[u8] = &[0x42, 0x00, 0x7b, 0x01, 0, 0, 0, 0];

#[test]
fn verified_raw_tls_exchange_sends_one_request_and_reads_one_frame() {
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    let (listener, address) = loopback_listener();
    let peer = spawn_raw_tls_peer(
        listener,
        server_config(&pki),
        fixtures::REQUEST_SENTINEL.len(),
        RESPONSE_FRAME,
    );
    let config = client_config(
        &pki,
        Endpoint::raw_tls("server.kmipkit.test", address.port()),
    );
    let candidate_observer = CandidateEventObserver::new();
    let mut adapter = new_for_test_with_candidate_observer(
        config,
        fixed_resolver(address),
        candidate_observer.clone(),
    );
    let response = adapter
        .exchange(fixtures::REQUEST_SENTINEL, RESPONSE_FRAME.len())
        .expect("the verified raw-TLS exchange succeeds");

    assert_eq!(response.as_bytes(), RESPONSE_FRAME);
    assert_eq!(
        candidate_observer.events(),
        vec![
            CandidateEvent::HandshakeSucceeded(address),
            CandidateEvent::RequestDispatch(address),
        ]
    );
    let request = peer
        .join()
        .expect("the local raw-TLS peer thread does not panic")
        .expect("the TLS peer reads and answers one request");
    assert_eq!(request, fixtures::REQUEST_SENTINEL);
}

#[test]
fn raw_tls_test_hooks_record_candidate_and_allocation_events() {
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    let (_listener, address) = loopback_listener();
    let config = || {
        client_config(
            &pki,
            Endpoint::raw_tls("server.kmipkit.test", address.port()),
        )
    };
    let plain = new_for_test(config(), None);
    let with_resolver = new_for_test_with_resolver(config(), None, fixed_resolver(address));
    let candidate_observer = CandidateEventObserver::new();
    let observed = new_for_test_with_candidate_observer(
        config(),
        fixed_resolver(address),
        candidate_observer.clone(),
    );
    let allocation_observer = ResponseAllocationObserver::new();
    let allocation_adapter =
        new_for_test_with_response_allocation_observer(config(), allocation_observer.clone());
    let (spawner_sender, spawner_receiver) = std::sync::mpsc::sync_channel(1);
    let lifecycle = new_for_test_with_lifecycle_controls(
        config(),
        None,
        fixed_resolver(address),
        move |task| {
            spawner_sender.send(task).map_err(|_| {
                std::io::Error::other("the unit test keeps its worker task receiver open")
            })?;
            Err(std::io::Error::other("the test worker is not started"))
        },
    );
    drop(spawner_receiver);

    candidate_observer.record_handshake_result(address, false);
    candidate_observer.record_handshake_result(address, true);
    candidate_observer.record_request_dispatch_for_selected_candidate();
    assert_eq!(
        candidate_observer.events(),
        vec![
            CandidateEvent::HandshakeFailed(address),
            CandidateEvent::HandshakeSucceeded(address),
            CandidateEvent::RequestDispatch(address),
        ]
    );

    allocation_observer.record_allocation_attempt();
    assert_eq!(allocation_observer.allocation_count(), 1);
    drop((
        plain,
        with_resolver,
        observed,
        allocation_adapter,
        lifecycle,
    ));
}

#[test]
fn raw_tls_worker_start_failure_and_response_allocation_observer_are_reported_safely() {
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    let (listener, address) = loopback_listener();
    listener
        .set_nonblocking(true)
        .expect("the loopback listener accepts a nonblocking check");
    let config = client_config(
        &pki,
        Endpoint::raw_tls("server.kmipkit.test", address.port()),
    );
    let mut failing_worker =
        new_for_test_with_lifecycle_controls(config, None, fixed_resolver(address), |_task| {
            Err(std::io::Error::other("injected worker startup failure"))
        });
    let error = failing_worker
        .exchange(fixtures::REQUEST_SENTINEL, RESPONSE_FRAME.len())
        .expect_err("the injected worker startup error is returned");
    assert_eq!(error.delivery_state(), crate::RequestDeliveryState::NotSent);
    assert_eq!(error.cause_category(), crate::TransportCauseCategory::Other);
    drop(failing_worker);
    let accept_error = listener
        .accept()
        .expect_err("a worker startup failure must not open a network connection");
    assert_eq!(accept_error.kind(), std::io::ErrorKind::WouldBlock);

    let pki = EphemeralPki::generate().expect("a fresh ephemeral test PKI is generated");
    let (listener, address) = loopback_listener();
    let peer = spawn_raw_tls_peer(
        listener,
        server_config(&pki),
        fixtures::REQUEST_SENTINEL.len(),
        RESPONSE_FRAME,
    );
    let config = client_config_with_server_name(
        &pki,
        Endpoint::raw_tls("127.0.0.1", address.port()),
        Some("server.kmipkit.test"),
    );
    let allocation_observer = ResponseAllocationObserver::new();
    let mut observed =
        new_for_test_with_response_allocation_observer(config, allocation_observer.clone());
    let error = observed
        .exchange(fixtures::REQUEST_SENTINEL, RESPONSE_FRAME.len())
        .expect_err("the test observer stops before response allocation");
    assert_eq!(allocation_observer.allocation_count(), 1);
    assert_eq!(error.cause_category(), crate::TransportCauseCategory::Other);
    assert_eq!(
        error.delivery_state(),
        crate::RequestDeliveryState::ResponseStarted
    );
    let request = peer
        .join()
        .expect("the local TLS peer thread does not panic")
        .expect("the peer reads the dispatched request");
    assert_eq!(request, fixtures::REQUEST_SENTINEL);
}

#[test]
fn raw_tls_rejects_zero_response_limit_before_dispatch() {
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    let (_listener, address) = loopback_listener();
    let config = client_config(
        &pki,
        Endpoint::raw_tls("server.kmipkit.test", address.port()),
    );
    let mut adapter = RawTlsTransport::new(config).expect("the explicit raw-TLS config is valid");
    let error = adapter
        .exchange(fixtures::REQUEST_SENTINEL, 0)
        .expect_err("a response cap smaller than the TTLV header is rejected");
    assert_eq!(error.delivery_state(), crate::RequestDeliveryState::NotSent);
}

#[test]
fn raw_tls_rejects_oversized_requests_and_unrepresentable_deadlines_before_connecting() {
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    let (listener, address) = loopback_listener();
    listener
        .set_nonblocking(true)
        .expect("the loopback listener accepts a nonblocking check");
    let config = client_config(
        &pki,
        Endpoint::raw_tls("server.kmipkit.test", address.port()),
    );
    let mut adapter = RawTlsTransport::new(config).expect("the explicit raw-TLS config is valid");

    let oversized_request = vec![0; 16 * 1024 * 1024 + 1];
    let error = adapter
        .exchange_with_options(
            &oversized_request,
            RESPONSE_FRAME.len(),
            &RequestOptions::default(),
        )
        .expect_err("a request above the configured cap is rejected before dispatch");
    assert_eq!(error.delivery_state(), crate::RequestDeliveryState::NotSent);
    assert_eq!(error.cause_category(), crate::TransportCauseCategory::Other);

    let unrepresentable = RequestOptions::default().with_read(TimeoutLimit::Bounded(Duration::MAX));
    let error = adapter
        .exchange_with_options(
            fixtures::REQUEST_SENTINEL,
            RESPONSE_FRAME.len(),
            &unrepresentable,
        )
        .expect_err("a read deadline outside the clock range is rejected");
    assert_eq!(error.delivery_state(), crate::RequestDeliveryState::NotSent);
    assert_eq!(error.cause_category(), crate::TransportCauseCategory::Other);
    assert_eq!(
        listener
            .accept()
            .expect_err("preflight failures open no connection")
            .kind(),
        std::io::ErrorKind::WouldBlock
    );
}

#[test]
fn raw_tls_rejects_malformed_ttlv_headers_after_one_request() {
    const BAD_ROOT_TAG: &[u8] = &[0x42, 0, 0x79, 0x01, 0, 0, 0, 0];
    const BAD_ROOT_TYPE: &[u8] = &[0x42, 0, 0x78, 0x02, 0, 0, 0, 0];
    const UNALIGNED_LENGTH: &[u8] = &[0x42, 0, 0x78, 0x01, 0, 0, 0, 1];
    const OVERSIZED_LENGTH: &[u8] = &[0x42, 0, 0x78, 0x01, 0xff, 0xff, 0xff, 0xf8];

    for response in [
        BAD_ROOT_TAG,
        BAD_ROOT_TYPE,
        UNALIGNED_LENGTH,
        OVERSIZED_LENGTH,
    ] {
        let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
        let (listener, address) = loopback_listener();
        let peer = spawn_raw_tls_peer(
            listener,
            server_config(&pki),
            fixtures::REQUEST_SENTINEL.len(),
            response,
        );
        let config = client_config(
            &pki,
            Endpoint::raw_tls("server.kmipkit.test", address.port()),
        );
        let mut adapter = new_for_test_with_resolver(config, None, fixed_resolver(address));
        let error = adapter
            .exchange(fixtures::REQUEST_SENTINEL, RESPONSE_FRAME.len())
            .expect_err("a malformed TTLV response header is rejected");
        assert_eq!(
            error.delivery_state(),
            crate::RequestDeliveryState::ResponseStarted
        );
        assert_eq!(error.cause_category(), crate::TransportCauseCategory::Other);
        drop(adapter);
        let request = peer
            .join()
            .expect("the local TLS peer thread does not panic")
            .expect("the peer reads one request");
        assert_eq!(request, fixtures::REQUEST_SENTINEL);
    }
}

#[test]
fn raw_tls_hostname_verification_failure_is_not_sent_and_records_failed_candidate() {
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    let (listener, address) = loopback_listener();
    let peer = spawn_raw_tls_peer(
        listener,
        server_config(&pki),
        fixtures::REQUEST_SENTINEL.len(),
        RESPONSE_FRAME,
    );
    let config = client_config_with_server_name(
        &pki,
        Endpoint::raw_tls("127.0.0.1", address.port()),
        Some("mismatched.server.kmipkit.test"),
    );
    let observer = CandidateEventObserver::new();
    let mut adapter =
        new_for_test_with_candidate_observer(config, fixed_resolver(address), observer.clone());

    let error = adapter
        .exchange(fixtures::REQUEST_SENTINEL, RESPONSE_FRAME.len())
        .expect_err("hostname verification failure aborts before request dispatch");
    assert_eq!(error.cause_category(), crate::TransportCauseCategory::Tls);
    assert_eq!(error.delivery_state(), crate::RequestDeliveryState::NotSent);
    assert_eq!(
        observer.events(),
        vec![CandidateEvent::HandshakeFailed(address)]
    );
    assert!(
        peer.join()
            .expect("the local TLS peer thread does not panic")
            .is_err()
    );
}
