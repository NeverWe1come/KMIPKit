//! Raw TLS JSON/XML response framing under OASIS KMIP Profiles v2.1 §§5.4
//! (XML) and 5.5 (JSON), with the connection lifecycle defined by ADR-0018.

use super::{ExchangeControl, minimum_response_bytes, read_response, read_text_response};
use crate::{RequestDeliveryState, TransportCauseCategory, config::WireEncoding};
use std::{io, pin::Pin, task};

fn read_document(
    bytes: &[u8],
    encoding: WireEncoding,
    max_response_bytes: usize,
) -> Result<Vec<u8>, crate::TransportError> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("the text response test runtime is created");
    runtime.block_on(async {
        let (mut writer, mut reader) = tokio::io::duplex(bytes.len().max(1));
        tokio::io::AsyncWriteExt::write_all(&mut writer, bytes)
            .await
            .expect("the response fixture is staged");
        tokio::io::AsyncWriteExt::shutdown(&mut writer)
            .await
            .expect("the response fixture reaches EOF");
        let control = response_started_control();
        read_response(&mut reader, encoding, max_response_bytes, &control, None)
            .await
            .map(|response| response.as_bytes().to_vec())
    })
}

fn response_started_control() -> ExchangeControl {
    let control = ExchangeControl::new();
    assert!(control.begin());
    assert!(control.commit_dispatch());
    assert!(control.response_started());
    control
}

struct FailingReader;

impl tokio::io::AsyncRead for FailingReader {
    fn poll_read(
        self: Pin<&mut Self>,
        _context: &mut task::Context<'_>,
        _buffer: &mut tokio::io::ReadBuf<'_>,
    ) -> task::Poll<io::Result<()>> {
        task::Poll::Ready(Err(io::Error::other("test read failure")))
    }
}

#[test]
fn text_response_reader_returns_the_first_complete_json_document() {
    let response = read_document(
        br#"{"tag":"ResponseMessage"} false"#,
        WireEncoding::Json,
        64,
    )
    .expect("the first complete JSON object ends the response");
    assert_eq!(response, br#"{"tag":"ResponseMessage"}"#);
}

#[test]
fn text_response_reader_returns_a_complete_xml_document_at_its_exact_byte_limit() {
    let document = b"<ResponseMessage/>";
    let response = read_document(document, WireEncoding::Xml, document.len())
        .expect("a complete XML response fits the exact local limit");
    assert_eq!(response, document);
}

#[test]
fn text_response_reader_rejects_oversized_malformed_and_incomplete_responses_safely() {
    let oversized = read_document(b"{}", WireEncoding::Json, 1)
        .expect_err("the local response byte limit applies to JSON");
    assert_eq!(
        oversized.delivery_state(),
        RequestDeliveryState::ResponseStarted
    );
    assert_eq!(oversized.cause_category(), TransportCauseCategory::Other);

    let malformed = read_document(b"[1]", WireEncoding::Json, 16)
        .expect_err("a JSON array is not a KMIP message root");
    assert_eq!(
        malformed.delivery_state(),
        RequestDeliveryState::ResponseStarted
    );
    assert_eq!(malformed.cause_category(), TransportCauseCategory::Other);

    let incomplete = read_document(b"<ResponseMessage>", WireEncoding::Xml, 32)
        .expect_err("EOF before the XML root closes is rejected");
    assert_eq!(
        incomplete.delivery_state(),
        RequestDeliveryState::ResponseStarted
    );
    assert_eq!(incomplete.cause_category(), TransportCauseCategory::Other);
}

#[test]
fn text_response_reader_classifies_another_wire_encoding_without_payload_details() {
    let error = read_document(b"<ResponseMessage/>", WireEncoding::Json, 64)
        .expect_err("an XML response does not match a JSON exchange");
    assert_eq!(
        error.delivery_state(),
        RequestDeliveryState::ResponseStarted
    );
    assert_eq!(
        error.cause_category(),
        TransportCauseCategory::EncodingMismatch
    );
}

#[test]
fn response_limit_minimum_matches_the_selected_framing() {
    assert_eq!(minimum_response_bytes(WireEncoding::Ttlv), 8);
    assert_eq!(minimum_response_bytes(WireEncoding::Json), 1);
    assert_eq!(minimum_response_bytes(WireEncoding::Xml), 1);
}

#[test]
fn text_response_helper_rejects_ttlv_mode_with_a_safe_error() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("the text response test runtime is created");
    let control = response_started_control();
    let error = runtime
        .block_on(read_text_response(
            &mut FailingReader,
            WireEncoding::Ttlv,
            64,
            &control,
        ))
        .expect_err("TTLV is handled by its length-framed reader");
    assert_eq!(
        error.delivery_state(),
        RequestDeliveryState::ResponseStarted
    );
    assert_eq!(error.cause_category(), TransportCauseCategory::Other);
}

#[test]
fn response_reader_selects_length_framed_ttlv_and_preserves_its_body() {
    const RESPONSE: [u8; 16] = [
        0x42, 0x00, 0x7b, 0x01, 0x00, 0x00, 0x00, 0x08, 0x42, 0x00, 0x01, 0x01, 0x00, 0x00, 0x00,
        0x00,
    ];
    let response = read_document(&RESPONSE, WireEncoding::Ttlv, RESPONSE.len())
        .expect("the TTLV length header frames its complete response body");
    assert_eq!(response, RESPONSE);
}

#[test]
fn response_reader_rejects_ttlv_mismatches_invalid_headers_and_limits() {
    let mismatch = read_document(b"{}", WireEncoding::Ttlv, 64)
        .expect_err("a text response is not decoded as TTLV");
    assert_eq!(
        mismatch.delivery_state(),
        RequestDeliveryState::ResponseStarted
    );
    assert_eq!(
        mismatch.cause_category(),
        TransportCauseCategory::EncodingMismatch
    );

    let invalid_header = read_document(
        &[0x42, 0x00, 0x7a, 0x01, 0x00, 0x00, 0x00, 0x00],
        WireEncoding::Ttlv,
        64,
    )
    .expect_err("an invalid TTLV root header is rejected");
    assert_eq!(
        invalid_header.cause_category(),
        TransportCauseCategory::Other
    );

    let oversized = read_document(
        &[0x42, 0x00, 0x7b, 0x01, 0x00, 0x00, 0x00, 0x08],
        WireEncoding::Ttlv,
        8,
    )
    .expect_err("the TTLV body length is checked before allocation");
    assert_eq!(oversized.cause_category(), TransportCauseCategory::Other);
}

#[test]
fn response_reader_rejects_a_truncated_ttlv_body() {
    let error = read_document(
        &[
            0x42, 0x00, 0x7b, 0x01, 0x00, 0x00, 0x00, 0x08, 0x42, 0x00, 0x01,
        ],
        WireEncoding::Ttlv,
        64,
    )
    .expect_err("a truncated TTLV body is rejected");
    assert_eq!(
        error.delivery_state(),
        RequestDeliveryState::ResponseStarted
    );
    assert_eq!(error.cause_category(), TransportCauseCategory::Io);
}

#[test]
fn text_response_reader_maps_stream_read_errors_to_a_safe_io_cause() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("the text response test runtime is created");
    let control = response_started_control();
    let error = runtime
        .block_on(read_response(
            &mut FailingReader,
            WireEncoding::Json,
            64,
            &control,
            None,
        ))
        .expect_err("the failed stream read returns a sanitized error");
    assert_eq!(
        error.delivery_state(),
        RequestDeliveryState::ResponseStarted
    );
    assert_eq!(error.cause_category(), TransportCauseCategory::Io);
}
