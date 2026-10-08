use kmipkit_test_support::{EphemeralPki, fixtures};
use std::time::Duration;

use crate::Transport;
use crate::config::{Endpoint, PrivateKeyInput, RequestOptions, TimeoutLimit};
use crate::https::{
    DriverCleanupGateForTest, ResponseBufferObserver, has_cached_connection_for_test,
    new_for_test_canceling_before_success_finish,
    new_for_test_canceling_before_success_finish_with_cleanup_gate, new_for_test_with_resolver,
    new_for_test_with_resolver_and_driver_abort_observer,
    new_for_test_with_resolver_and_driver_cleanup_gate, new_for_test_with_response_buffer_observer,
    new_for_test_with_worker_spawner,
};
use crate::resolver::Resolver;
use crate::secret::SecretBufferObserver;
use crate::transport_test_support::{
    client_config, fixed_resolver, loopback_listener, server_config, spawn_http_peer,
    spawn_http_peer_with_response,
};

#[test]
fn verified_https_exchange_reuses_one_connection_and_preserves_request_bytes() {
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    let (listener, address) = loopback_listener();
    let peer = spawn_http_peer(listener, server_config(&pki), 2);
    let config = client_config(
        &pki,
        Endpoint::https(format!(
            "https://server.kmipkit.test:{}/kmip",
            address.port()
        )),
    );
    let request_observer = SecretBufferObserver::new(fixtures::REQUEST_SENTINEL.len());
    let mut adapter = new_for_test_with_resolver(
        config,
        Some(request_observer.clone()),
        fixed_resolver(address),
    );

    for _ in 0..2 {
        let response = adapter
            .exchange(fixtures::REQUEST_SENTINEL, 64)
            .expect("the verified HTTP/1.1 exchange succeeds");
        assert_eq!(response.as_bytes(), b"response");
    }
    assert!(has_cached_connection_for_test(&adapter));
    drop(adapter);
    assert_eq!(
        request_observer.initialized_len(),
        fixtures::REQUEST_SENTINEL.len()
    );
    assert!(request_observer.initialized_range_was_zero());

    let requests = peer
        .join()
        .expect("the local HTTPS peer thread does not panic")
        .expect("the TLS peer exchanges two requests");
    assert_eq!(requests.len(), 2);
    for request in requests {
        assert!(request.starts_with(b"POST /kmip HTTP/1.1\r\n"));
        assert!(request.ends_with(fixtures::REQUEST_SENTINEL));
    }
}

#[test]
fn https_rejects_oversized_requests_and_unrepresentable_deadlines_before_connecting() {
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    let (listener, address) = loopback_listener();
    listener
        .set_nonblocking(true)
        .expect("the loopback listener accepts a nonblocking check");
    let config = client_config(
        &pki,
        Endpoint::https(format!(
            "https://server.kmipkit.test:{}/kmip",
            address.port()
        )),
    );
    let mut adapter = new_for_test_with_resolver(config, None, fixed_resolver(address));

    let oversized_request = vec![0; 16 * 1024 * 1024 + 1];
    let error = adapter
        .exchange_with_options(&oversized_request, 64, &RequestOptions::default())
        .expect_err("a request above the configured cap is rejected before dispatch");
    assert_eq!(error.delivery_state(), crate::RequestDeliveryState::NotSent);
    assert_eq!(error.cause_category(), crate::TransportCauseCategory::Other);

    let unrepresentable =
        RequestOptions::default().with_total(TimeoutLimit::Bounded(Duration::MAX));
    let error = adapter
        .exchange_with_options(fixtures::REQUEST_SENTINEL, 64, &unrepresentable)
        .expect_err("a total deadline outside the clock range is rejected");
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
fn https_rejects_server_error_status_without_exposing_response_bytes() {
    const SECRET_RESPONSE: &[u8] = b"HTTP/1.1 500 Server Error\r\nContent-Type: application/octet-stream\r\nContent-Length: 21\r\n\r\nKMIPKIT_SECRET_STATUS";

    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    let (listener, address) = loopback_listener();
    let peer = spawn_http_peer_with_response(listener, server_config(&pki), 1, SECRET_RESPONSE);
    let config = client_config(
        &pki,
        Endpoint::https(format!(
            "https://server.kmipkit.test:{}/kmip",
            address.port()
        )),
    );
    let mut adapter = new_for_test_with_resolver(config, None, fixed_resolver(address));
    let error = adapter
        .exchange_with_options(fixtures::REQUEST_SENTINEL, 64, &RequestOptions::default())
        .expect_err("non-success HTTP status is rejected");
    assert_eq!(error.cause_category(), crate::TransportCauseCategory::Http);
    assert_eq!(
        error.delivery_state(),
        crate::RequestDeliveryState::ResponseStarted
    );
    assert!(!format!("{error:?}").contains("KMIPKIT_SECRET_STATUS"));
    drop(adapter);
    let requests = peer
        .join()
        .expect("the local HTTP peer thread does not panic")
        .expect("the HTTP peer reads one request");
    assert_eq!(requests.len(), 1);
}

#[test]
fn https_rejects_response_header_policy_violations_before_returning_body_bytes() {
    const CASES: [&[u8]; 4] = [
        b"HTTP/1.1 200 OK\r\nContent-Length: 8\r\n\r\nresponse",
        b"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Type: application/octet-stream\r\nContent-Length: 8\r\n\r\nresponse",
        b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 8\r\n\r\nresponse",
        b"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Encoding: gzip\r\nContent-Length: 8\r\n\r\nresponse",
    ];

    for response in CASES {
        let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
        let (listener, address) = loopback_listener();
        let peer = spawn_http_peer_with_response(listener, server_config(&pki), 1, response);
        let config = client_config(
            &pki,
            Endpoint::https(format!(
                "https://server.kmipkit.test:{}/kmip",
                address.port()
            )),
        );
        let mut adapter = new_for_test_with_resolver(config, None, fixed_resolver(address));
        let error = adapter
            .exchange_with_options(fixtures::REQUEST_SENTINEL, 64, &RequestOptions::default())
            .expect_err("unsupported response metadata is rejected");
        assert_eq!(error.cause_category(), crate::TransportCauseCategory::Http);
        assert_eq!(
            error.delivery_state(),
            crate::RequestDeliveryState::ResponseStarted
        );
        drop(adapter);
        let requests = peer
            .join()
            .expect("the local HTTP peer thread does not panic")
            .expect("the HTTP peer reads one request");
        assert_eq!(requests.len(), 1);
    }
}

#[test]
fn https_truncated_body_is_reported_after_response_started() {
    const TRUNCATED: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: 16\r\n\r\npartial";
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    let (listener, address) = loopback_listener();
    let peer = spawn_http_peer_with_response(listener, server_config(&pki), 1, TRUNCATED);
    let config = client_config(
        &pki,
        Endpoint::https(format!(
            "https://server.kmipkit.test:{}/kmip",
            address.port()
        )),
    );
    let mut adapter = new_for_test_with_resolver(config, None, fixed_resolver(address));
    let error = adapter
        .exchange_with_options(fixtures::REQUEST_SENTINEL, 64, &RequestOptions::default())
        .expect_err("short HTTP body is rejected");
    assert_eq!(error.cause_category(), crate::TransportCauseCategory::Http);
    assert_eq!(
        error.delivery_state(),
        crate::RequestDeliveryState::ResponseStarted
    );
    let requests = peer
        .join()
        .expect("the local HTTP peer thread does not panic")
        .expect("the HTTP peer captures the request before truncating the body");
    assert_eq!(requests.len(), 1);
    assert!(requests[0].ends_with(fixtures::REQUEST_SENTINEL));
}

#[test]
fn observed_private_key_input_zeroizes_its_initialized_bytes() {
    let sentinel = b"private-key-input-observer-sentinel";
    let observer = SecretBufferObserver::new(sentinel.len());
    let key = PrivateKeyInput::from_pem_with_observer_for_test(sentinel.to_vec(), observer.clone());

    assert_eq!(observer.initialized_len(), 0);
    drop(key);
    assert_eq!(observer.initialized_len(), sentinel.len());
    assert!(observer.initialized_range_was_zero());
}

#[test]
fn https_test_constructors_keep_observers_and_cancellation_gates_adapter_local() {
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    let (_listener, address) = loopback_listener();
    let config = || {
        client_config(
            &pki,
            Endpoint::https(format!(
                "https://server.kmipkit.test:{}/kmip",
                address.port()
            )),
        )
    };
    let resolver = fixed_resolver(address);

    let basic = new_for_test_with_resolver(config(), None, resolver.clone());
    assert!(!has_cached_connection_for_test(&basic));

    let response_observer = ResponseBufferObserver::new();
    let observed_adapter =
        new_for_test_with_response_buffer_observer(config(), resolver.clone(), response_observer);
    assert!(!has_cached_connection_for_test(&observed_adapter));

    let (abort_sender, _abort_receiver) = std::sync::mpsc::sync_channel(1);
    let (_cancel_sender, cancel_receiver) = tokio::sync::oneshot::channel();
    let abortable = new_for_test_with_resolver_and_driver_abort_observer(
        config(),
        resolver.clone(),
        abort_sender,
        cancel_receiver,
    );
    assert!(!has_cached_connection_for_test(&abortable));

    let (cleanup_events, _cleanup_event_receiver) = std::sync::mpsc::channel();
    let (_cleanup_release_sender, cleanup_release_receiver) = tokio::sync::oneshot::channel();
    let (_cleanup_cancel_sender, cleanup_cancel_receiver) = tokio::sync::oneshot::channel();
    let gated = new_for_test_with_resolver_and_driver_cleanup_gate(
        config(),
        resolver.clone(),
        DriverCleanupGateForTest {
            release: cleanup_release_receiver,
            events: cleanup_events,
        },
        cleanup_cancel_receiver,
    );
    assert!(!has_cached_connection_for_test(&gated));

    let cancel_before_finish =
        new_for_test_canceling_before_success_finish(config(), resolver.clone());
    assert!(!has_cached_connection_for_test(&cancel_before_finish));

    let (_release_sender, release_receiver) = tokio::sync::oneshot::channel();
    let (events, _event_receiver) = std::sync::mpsc::channel();
    let cancel_before_finish_with_gate =
        new_for_test_canceling_before_success_finish_with_cleanup_gate(
            config(),
            resolver,
            DriverCleanupGateForTest {
                release: release_receiver,
                events,
            },
        );
    assert!(!has_cached_connection_for_test(
        &cancel_before_finish_with_gate
    ));

    drop((
        basic,
        observed_adapter,
        abortable,
        gated,
        cancel_before_finish,
        cancel_before_finish_with_gate,
    ));
}

#[test]
fn https_worker_start_failure_is_sanitized_and_precedes_network_dispatch() {
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    let (listener, address) = loopback_listener();
    listener
        .set_nonblocking(true)
        .expect("the loopback listener accepts a nonblocking check");
    let config = client_config(
        &pki,
        Endpoint::https(format!(
            "https://server.kmipkit.test:{}/kmip",
            address.port()
        )),
    );
    let mut adapter = new_for_test_with_worker_spawner(config, fixed_resolver(address), |_task| {
        Err(std::io::Error::other("injected worker startup failure"))
    });

    let error = adapter
        .exchange(fixtures::REQUEST_SENTINEL, 64)
        .expect_err("worker startup failure is returned before connection");
    assert_eq!(error.cause_category(), crate::TransportCauseCategory::Other);
    assert_eq!(error.delivery_state(), crate::RequestDeliveryState::NotSent);
    assert_eq!(
        listener
            .accept()
            .expect_err("failed worker startup opens no socket")
            .kind(),
        std::io::ErrorKind::WouldBlock
    );
}

#[test]
fn https_empty_resolver_result_fails_before_opening_a_tcp_connection() {
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    let (listener, address) = loopback_listener();
    listener
        .set_nonblocking(true)
        .expect("the loopback listener accepts a nonblocking check");
    let config = client_config(
        &pki,
        Endpoint::https(format!(
            "https://server.kmipkit.test:{}/kmip",
            address.port()
        )),
    );
    let empty_resolver = Resolver::with_lookup_and_governor(
        |_host, _port| Ok(Vec::new()),
        std::sync::Arc::new(tokio::sync::Semaphore::new(1)),
    );
    let mut adapter = new_for_test_with_resolver(config, None, empty_resolver);

    let error = adapter
        .exchange(fixtures::REQUEST_SENTINEL, 64)
        .expect_err("an empty resolver result cannot start a connection");
    assert_eq!(error.cause_category(), crate::TransportCauseCategory::Io);
    assert_eq!(error.delivery_state(), crate::RequestDeliveryState::NotSent);
    assert_eq!(
        listener
            .accept()
            .expect_err("an empty resolver result opens no socket")
            .kind(),
        std::io::ErrorKind::WouldBlock
    );
}

#[test]
fn https_zero_response_limit_and_unrepresentable_deadline_fail_before_dispatch() {
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    let (_listener, address) = loopback_listener();
    let config = client_config(
        &pki,
        Endpoint::https(format!(
            "https://server.kmipkit.test:{}/kmip",
            address.port()
        )),
    );
    let mut adapter = new_for_test_with_resolver(config, None, fixed_resolver(address));
    let error = adapter
        .exchange(fixtures::REQUEST_SENTINEL, 0)
        .expect_err("zero response capacity is rejected before network work");
    assert_eq!(error.delivery_state(), crate::RequestDeliveryState::NotSent);
    assert_eq!(error.cause_category(), crate::TransportCauseCategory::Other);
}
