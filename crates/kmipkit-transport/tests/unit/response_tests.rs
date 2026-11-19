use std::error::Error;
use std::fmt;

use crate::{RequestDeliveryState, TransportCauseCategory, TransportError};

use super::{DropObserver, TransportResponse};

const RESPONSE_SENTINEL: &[u8] = b"KMIP_RESPONSE_BODY_SENTINEL";
const PARTIAL_RESPONSE_SENTINEL: &str = "KMIP_PARTIAL_RESPONSE_BODY_SENTINEL";

#[derive(Debug)]
struct PartialReadSource(&'static str);

impl fmt::Display for PartialReadSource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

impl Error for PartialReadSource {}

fn partial_read_failure(observer: DropObserver) -> Result<(), TransportError> {
    let _partial_response = TransportResponse::with_drop_observer(
        PARTIAL_RESPONSE_SENTINEL.as_bytes().to_vec(),
        observer,
    );

    Err(TransportError::new(
        RequestDeliveryState::ResponseStarted,
        TransportCauseCategory::Io,
        PartialReadSource(PARTIAL_RESPONSE_SENTINEL),
    ))
}

#[test]
fn successful_response_drop_zeroizes_initialized_bytes_before_release() {
    let observer = DropObserver::new(RESPONSE_SENTINEL.len());
    let response =
        TransportResponse::with_drop_observer(RESPONSE_SENTINEL.to_vec(), observer.clone());

    assert_eq!(response.as_bytes(), RESPONSE_SENTINEL);
    drop(response);

    assert!(
        observer.initialized_range_was_zero(),
        "observer must reject a zero-length slice after initialized response bytes were stored"
    );
}

#[test]
fn partial_read_error_drops_zeroized_response_and_redacts_source() {
    let observer = DropObserver::new(PARTIAL_RESPONSE_SENTINEL.len());
    let error = partial_read_failure(observer.clone())
        .expect_err("partial read returns a redacted transport error");
    let rendered = format!("{error} {error:?}");
    assert!(!rendered.contains(PARTIAL_RESPONSE_SENTINEL));
    let mut source = error.source();
    while let Some(current) = source {
        assert!(!current.to_string().contains(PARTIAL_RESPONSE_SENTINEL));
        source = current.source();
    }
    assert!(
        observer.initialized_range_was_zero(),
        "observer must reject a zero-length slice on the partial-read error path"
    );
}

#[test]
fn response_debug_redacts_initialized_response_bytes() {
    let response = TransportResponse::new(RESPONSE_SENTINEL.to_vec());

    assert!(!format!("{response:?}").contains("KMIP_RESPONSE_BODY_SENTINEL"));
}
