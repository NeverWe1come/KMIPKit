//! OASIS KMIP Profiles v2.1 §§5.3.1–5.3.2, interpreted by ADR-0018.
//! Transport fixtures use empty KMIP containers to distinguish wire formats;
//! typed message validity and codec decoding are tested by the client.

use std::error::Error;

use super::*;
use config::WireEncoding;

const FORMATS: [(WireEncoding, &str, &[u8]); 3] = [
    (
        WireEncoding::Ttlv,
        "application/octet-stream",
        b"\x42\x00\x7b\x01\0\0\0\0",
    ),
    (
        WireEncoding::Json,
        "application/json",
        br#"{"tag":"ResponseMessage","type":"Structure","value":[]}"#,
    ),
    (
        WireEncoding::Xml,
        "text/xml",
        b"<ResponseMessage type=\"Structure\"></ResponseMessage>",
    ),
];

fn response_wire(media_type: &str, body: &[u8], declared_length: usize) -> Vec<u8> {
    let mut response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {media_type}\r\nContent-Length: {declared_length}\r\n\r\n"
    )
    .into_bytes();
    response.extend_from_slice(body);
    response
}

#[test]
fn public_https_adapter_uses_selected_content_type_for_each_wire_format() {
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    for (encoding, media_type, body) in FORMATS {
        let public_encoding = match encoding {
            WireEncoding::Ttlv => kmipkit_transport::WireEncoding::Ttlv,
            WireEncoding::Json => kmipkit_transport::WireEncoding::Json,
            WireEncoding::Xml => kmipkit_transport::WireEncoding::Xml,
        };
        let listener = LoopbackTcpListener::bind().expect("the HTTPS peer binds loopback");
        let address = listener.local_addr();
        let peer = spawn_raw_response_peer(
            listener.into_inner(),
            server_config(&pki),
            response_wire(media_type, body, body.len()),
        );
        let mut adapter = public_local_https_adapter(&pki, address);
        let result = adapter.exchange_with_options(
            body,
            body.len(),
            &kmipkit_transport::RequestOptions::default().with_wire_encoding(public_encoding),
        );
        let peer = peer.join().expect("the bounded HTTPS peer completes");
        let request = peer
            .request
            .expect("the peer captured the selected request");
        assert_eq!(header_values(&request, "content-type"), [media_type]);
        assert!(
            request.body == body,
            "the public adapter preserves wire bytes"
        );
        assert!(matches!(result, Ok(response) if response.as_bytes() == body));
    }
}

#[test]
fn selected_encoding_posts_exact_body_and_headers_and_accepts_the_same_format_at_the_byte_cap() {
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    for (encoding, media_type, body) in FORMATS {
        let listener = LoopbackTcpListener::bind().expect("the HTTPS peer binds loopback");
        let address = listener.local_addr();
        let peer = spawn_raw_response_peer(
            listener.into_inner(),
            server_config(&pki),
            response_wire(&media_type.to_ascii_uppercase(), body, body.len()),
        );
        let config = client_config(
            &pki,
            format!("https://{SERVER_NAME}:{}", address.port()),
            None,
            None,
        );
        let observer = secret::SecretBufferObserver::new(body.len());
        let mut adapter = https::new_for_test_with_resolver(
            config,
            Some(observer.clone()),
            fixed_resolver(address),
        );
        let result = adapter.exchange_with_options(
            body,
            body.len(),
            &RequestOptions::default().with_wire_encoding(encoding),
        );
        let peer = peer.join().expect("the bounded HTTPS peer completes");
        let request = peer.request.expect("the peer captured the encoded request");
        assert_eq!(header_values(&request, "content-type"), [media_type]);
        assert_eq!(
            header_values(&request, "content-length"),
            [body.len().to_string()]
        );
        assert_eq!(header_values(&request, "cache-control"), ["no-cache"]);
        assert_eq!(request.method, "POST");
        assert_eq!(request.version, "HTTP/1.1");
        assert!(
            request.body == body,
            "the selected wire bytes are unchanged"
        );
        assert_eq!(header_values(&request, "accept-encoding").len(), 0);
        assert_eq!(header_values(&request, "content-encoding").len(), 0);
        assert_eq!(header_values(&request, "transfer-encoding").len(), 0);
        assert_eq!(
            peer.protocol_version,
            Some(rustls::ProtocolVersion::TLSv1_3)
        );
        assert!(peer.client_identity_present);
        assert!(matches!(result, Ok(response) if response.as_bytes() == body));
        assert_request_owner_zeroized(&observer, body.len());
    }
}

#[test]
fn selected_encoding_changes_on_a_reused_https_connection() {
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    let listener = LoopbackTcpListener::bind().expect("the HTTPS peer binds loopback");
    let address = listener.local_addr();
    let responses = FORMATS
        .iter()
        .chain(std::iter::once(&FORMATS[0]))
        .map(|(_, media_type, body)| {
            response_wire(&format!("{media_type}; version=1"), body, body.len())
        })
        .collect();
    let peer = spawn_response_sequence_peer(listener.into_inner(), server_config(&pki), responses);
    let config = client_config(
        &pki,
        format!("https://{SERVER_NAME}:{}", address.port()),
        None,
        None,
    );
    let mut adapter = https::new_for_test_with_resolver(config, None, fixed_resolver(address));
    for (encoding, _, body) in FORMATS {
        let response = adapter
            .exchange_with_options(
                body,
                body.len(),
                &RequestOptions::default().with_wire_encoding(encoding),
            )
            .expect("each exchange uses its own encoding on the healthy connection");
        assert!(response.as_bytes() == body, "response bytes are unchanged");
    }
    let body = FORMATS[0].2;
    let response = adapter
        .exchange(body, body.len())
        .expect("a direct call after XML restores the default TTLV encoding");
    assert!(
        response.as_bytes() == body,
        "the direct response remains TTLV"
    );
    let peer = peer.join().expect("the bounded sequence peer completes");
    assert_eq!(peer.requests.len(), 4);
    for (request, (_, media_type, body)) in peer
        .requests
        .iter()
        .zip(FORMATS.into_iter().chain([FORMATS[0]]))
    {
        assert_eq!(header_values(request, "content-type"), [media_type]);
        assert!(request.body == body, "request bytes are unchanged");
    }
}

#[test]
fn selected_encoding_rejects_mismatched_duplicate_missing_and_invalid_response_types_without_allocation()
 {
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    for (encoding, selected_media_type, body) in FORMATS {
        let mut response_types: Vec<String> = FORMATS
            .iter()
            .filter(|(_, media_type, _)| *media_type != selected_media_type)
            .map(|(_, media_type, _)| (*media_type).to_owned())
            .collect();
        response_types.extend([
            format!("{selected_media_type}\r\nContent-Type: {selected_media_type}"),
            format!("{selected_media_type}, {selected_media_type}"),
            format!("{selected_media_type}; marker=private-header-sentinel\r\nContent-Type: application/x-private-header-sentinel"),
            "application/x-private-header-sentinel".to_owned(),
            format!("{selected_media_type}ÿ"),
            String::new(),
        ]);
        for response_type in response_types {
            let listener = LoopbackTcpListener::bind().expect("the HTTPS peer binds loopback");
            let address = listener.local_addr();
            let mut wire = response_wire(&response_type, body, body.len());
            if response_type.is_empty() {
                wire = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n", body.len())
                    .into_bytes();
                wire.extend_from_slice(body);
            }
            let peer = spawn_raw_response_peer(listener.into_inner(), server_config(&pki), wire);
            let config = client_config(
                &pki,
                format!("https://{SERVER_NAME}:{}", address.port()),
                None,
                None,
            );
            let observer = https::ResponseBufferObserver::new();
            let mut adapter = https::new_for_test_with_response_buffer_observer(
                config,
                fixed_resolver(address),
                observer.clone(),
            );
            let result = adapter.exchange_with_options(
                body,
                body.len(),
                &RequestOptions::default().with_wire_encoding(encoding),
            );
            let peer = peer.join().expect("the bounded HTTPS peer completes");
            let error =
                result.expect_err("a response type outside the selected format is rejected");
            assert_eq!(error.cause_category(), TransportCauseCategory::Http);
            assert_eq!(
                error.delivery_state(),
                RequestDeliveryState::ResponseStarted
            );
            assert_eq!(observer.allocation_attempts(), 0);
            assert!(!https::has_cached_connection_for_test(&adapter));
            assert!(peer.request.is_some());
            let source = error.source().expect("the safe cause is retained");
            for text in [
                error.to_string(),
                format!("{error:?}"),
                source.to_string(),
                format!("{source:?}"),
            ] {
                assert!(!text.contains("private-header-sentinel"));
                assert!(!text.contains("ResponseMessage"));
            }
        }
    }
}

#[test]
fn selected_encoding_mismatch_never_retries_or_falls_back() {
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    for (encoding, selected_media_type, body) in FORMATS {
        let listener = LoopbackTcpListener::bind().expect("the HTTPS peer binds loopback");
        let address = listener.local_addr();
        let wrong_type = if encoding == WireEncoding::Ttlv {
            "application/json"
        } else {
            "application/octet-stream"
        };
        let peer = spawn_retry_probe_peer_with_response(
            listener.into_inner(),
            server_config(&pki),
            response_wire(wrong_type, body, body.len()),
        );
        let config = client_config(
            &pki,
            format!("https://{SERVER_NAME}:{}", address.port()),
            None,
            None,
        );
        let mut adapter = https::new_for_test_with_resolver(config, None, fixed_resolver(address));
        let result = adapter.exchange_with_options(
            body,
            body.len(),
            &RequestOptions::default().with_wire_encoding(encoding),
        );
        let peer = peer.join().expect("the bounded retry probe completes");
        let error = result.expect_err("the mismatched response is rejected without fallback");
        assert_eq!(error.cause_category(), TransportCauseCategory::Http);
        assert_eq!(
            error.delivery_state(),
            RequestDeliveryState::ResponseStarted
        );
        assert_eq!(peer.accepted_connections, 1);
        assert_eq!(peer.requests.len(), 1);
        assert_eq!(
            header_values(&peer.requests[0], "content-type"),
            [selected_media_type]
        );
    }
}

#[test]
fn selected_encoding_preserves_response_byte_limits_and_partial_response_zeroization() {
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    for (encoding, media_type, body) in FORMATS {
        for oversized in [true, false] {
            let listener = LoopbackTcpListener::bind().expect("the HTTPS peer binds loopback");
            let address = listener.local_addr();
            let declared_length = body.len() + 1;
            let peer = spawn_raw_response_peer(
                listener.into_inner(),
                server_config(&pki),
                response_wire(media_type, body, declared_length),
            );
            let config = client_config(
                &pki,
                format!("https://{SERVER_NAME}:{}", address.port()),
                None,
                None,
            );
            let observer = https::ResponseBufferObserver::new();
            let mut adapter = https::new_for_test_with_response_buffer_observer(
                config,
                fixed_resolver(address),
                observer.clone(),
            );
            let limit = if oversized {
                body.len()
            } else {
                declared_length
            };
            let error = adapter
                .exchange_with_options(
                    body,
                    limit,
                    &RequestOptions::default().with_wire_encoding(encoding),
                )
                .expect_err("oversize and truncated responses are rejected");
            let peer = peer.join().expect("the bounded HTTPS peer completes");
            assert_eq!(
                error.delivery_state(),
                RequestDeliveryState::ResponseStarted
            );
            assert!(!https::has_cached_connection_for_test(&adapter));
            assert!(peer.request.is_some());
            if oversized {
                assert_eq!(error.cause_category(), TransportCauseCategory::Other);
                assert_eq!(observer.allocation_attempts(), 0);
            } else {
                assert_eq!(error.cause_category(), TransportCauseCategory::Http);
                assert_eq!(observer.initialized_len(), body.len());
                assert!(observer.initialized_range_was_zero());
            }
        }
    }
}

#[test]
fn selected_encoding_preserves_request_limits_before_connection() {
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    for (encoding, _, body) in FORMATS {
        let config = config_builder(&pki, "https://127.0.0.1:1".to_owned())
            .tls_server_name(SERVER_NAME)
            .max_request_bytes(body.len() - 1)
            .build()
            .expect("the bounded request configuration is valid");
        let mut adapter = https::HttpsTransport::new(config).expect("the adapter initializes");
        let error = adapter
            .exchange_with_options(
                body,
                64,
                &RequestOptions::default().with_wire_encoding(encoding),
            )
            .expect_err("an oversized request never reaches a socket");
        assert_eq!(error.delivery_state(), RequestDeliveryState::NotSent);
        assert_eq!(error.cause_category(), TransportCauseCategory::Other);
    }
}
