use hyper::header::{CONTENT_LENGTH, CONTENT_TYPE};
use hyper::http::{HeaderMap, HeaderValue};

use crate::worker::ExchangeControl;

use super::{
    HttpsTransport, ResponseBuffer, ResponseBufferObserver, parse_response_content_length,
    validate_response_content_type,
};

fn response_started_control() -> ExchangeControl {
    let control = ExchangeControl::new();
    assert!(control.begin());
    assert!(control.commit_dispatch());
    assert!(control.observe_response_byte());
    control
}

fn assert_http_response_error(error: crate::TransportError) {
    assert_eq!(error.cause_category(), crate::TransportCauseCategory::Http);
    assert_eq!(
        error.delivery_state(),
        crate::RequestDeliveryState::ResponseStarted
    );
}

#[test]
fn response_content_type_validation_accepts_octet_stream_with_parameters() {
    let mut headers = HeaderMap::new();
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("Application/Octet-Stream; version=1"),
    );

    assert!(validate_response_content_type(&headers, &response_started_control()).is_ok());
}

#[test]
fn response_content_type_validation_rejects_missing_duplicate_invalid_and_unsupported_values() {
    let control = response_started_control();

    let missing = HeaderMap::new();
    assert_http_response_error(
        validate_response_content_type(&missing, &control).expect_err("Content-Type is required"),
    );

    let mut duplicate = HeaderMap::new();
    duplicate.append(
        CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );
    duplicate.append(
        CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );
    assert_http_response_error(
        validate_response_content_type(&duplicate, &control)
            .expect_err("duplicate Content-Type fields are rejected"),
    );

    let mut invalid = HeaderMap::new();
    invalid.insert(
        CONTENT_TYPE,
        HeaderValue::from_bytes(b"application/octet-stream\xff")
            .expect("HTTP header values preserve obs-text octets"),
    );
    assert_http_response_error(
        validate_response_content_type(&invalid, &control)
            .expect_err("non-ASCII Content-Type is rejected"),
    );

    let mut unsupported = HeaderMap::new();
    unsupported.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    assert_http_response_error(
        validate_response_content_type(&unsupported, &control)
            .expect_err("unsupported media types are rejected"),
    );
}

#[test]
fn response_content_length_validation_covers_required_unique_decimal_and_bounded_values() {
    let control = response_started_control();
    let mut valid = HeaderMap::new();
    valid.insert(CONTENT_LENGTH, HeaderValue::from_static("42"));
    assert_eq!(parse_response_content_length(&valid, &control), Ok(42));

    let missing = HeaderMap::new();
    assert_http_response_error(
        parse_response_content_length(&missing, &control).expect_err("Content-Length is required"),
    );

    let mut duplicate = HeaderMap::new();
    duplicate.append(CONTENT_LENGTH, HeaderValue::from_static("1"));
    duplicate.append(CONTENT_LENGTH, HeaderValue::from_static("1"));
    assert_http_response_error(
        parse_response_content_length(&duplicate, &control)
            .expect_err("duplicate Content-Length fields are rejected"),
    );

    for value in ["", "1x", "999999999999999999999999999999999999999"] {
        let mut invalid = HeaderMap::new();
        invalid.insert(
            CONTENT_LENGTH,
            HeaderValue::from_str(value).expect("the test header value is valid"),
        );
        assert_http_response_error(
            parse_response_content_length(&invalid, &control)
                .expect_err("empty, non-decimal, and overflowing values are rejected"),
        );
    }
}

#[test]
fn response_buffer_growth_and_drop_zeroize_initialized_allocations() {
    let observer = ResponseBufferObserver::new();
    {
        let mut buffer = ResponseBuffer::with_capacity(2, Some(observer.clone()))
            .expect("the small response allocation succeeds");
        buffer
            .append(b"se", 64)
            .expect("the initial response bytes fit");
        buffer
            .append(b"cret", 64)
            .expect("the response buffer grows within its limit");
        assert_eq!(buffer.0.as_slice(), b"secret");
    }

    assert_eq!(observer.allocation_attempts(), 2);
    assert_eq!(observer.requested_capacity(), 6);
    assert_eq!(observer.initialized_len(), 6);
    assert!(observer.initialized_range_was_zero());
    assert!(observer.replaced_allocations_were_zero());
}

#[test]
fn response_buffer_rejects_limit_overflow_and_can_transfer_ownership() {
    let mut buffer =
        ResponseBuffer::with_capacity(2, None).expect("the response allocation succeeds");
    buffer.append(b"ab", 4).expect("the initial bytes fit");
    assert!(buffer.append(b"cde", 4).is_err());
    assert_eq!(buffer.0.as_slice(), b"ab");

    assert_eq!(buffer.into_bytes(), b"ab");
}

#[test]
fn staged_request_owner_zeroizes_observed_bytes_before_release() {
    let request = b"private request bytes";
    let observer = crate::secret::SecretBufferObserver::new(request.len());
    let owner = HttpsTransport::stage_request_owner(request, Some(observer.clone()))
        .expect("the bounded test request owner allocates");

    drop(owner);
    assert_eq!(observer.initialized_len(), request.len());
    assert!(observer.initialized_range_was_zero());
}
