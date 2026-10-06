//! Scripted low-level fake used by deterministic exchange tests.

use std::{fmt, io};

#[cfg(test)]
use std::{cell::Cell, rc::Rc};

use kmipkit_transport::{
    RequestDeliveryState, Transport, TransportCauseCategory, TransportError, TransportResponse,
};

/// One deterministic action for [`ScriptedTransport`].
#[derive(Clone, Eq, PartialEq)]
pub enum ExchangeScript {
    /// Writes the request in the listed chunks and returns `response`.
    Success {
        /// Response stream supplied by the fixture.
        response: Vec<u8>,
        /// Byte counts for successive partial writes; remaining bytes are written afterward.
        request_write_chunks: Vec<usize>,
    },
    /// Fails before writing any request bytes.
    FailBeforeWrite,
    /// Writes at most `written_bytes` request bytes, then fails.
    FailAfterPartialWrite {
        /// Number of request bytes accepted before failure.
        written_bytes: usize,
    },
    /// Writes request bytes, receives a response prefix, then fails.
    FailAfterPartialRead {
        /// Number of request bytes accepted before reading the response.
        written_bytes: usize,
        /// Initialized response bytes received before failure.
        response_bytes: Vec<u8>,
    },
}

struct ZeroizingFixtureBytes {
    bytes: Vec<u8>,
    #[cfg(test)]
    observer: Option<ResponseDropObserver>,
}

impl ZeroizingFixtureBytes {
    fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            #[cfg(test)]
            observer: None,
        }
    }

    #[cfg(test)]
    fn with_observer(bytes: Vec<u8>, observer: ResponseDropObserver) -> Self {
        Self {
            bytes,
            observer: Some(observer),
        }
    }

    fn as_slice(&self) -> &[u8] {
        &self.bytes
    }

    fn len(&self) -> usize {
        self.bytes.len()
    }
}

impl Drop for ZeroizingFixtureBytes {
    fn drop(&mut self) {
        #[cfg(test)]
        if let Some(observer) = &self.observer {
            observer.record(&self.bytes);
        }
    }
}

#[cfg(test)]
#[derive(Clone)]
struct ResponseDropObserver {
    initialized_len: usize,
    zeroized: Rc<Cell<bool>>,
}

#[cfg(test)]
impl ResponseDropObserver {
    fn new(initialized_len: usize) -> Self {
        Self {
            initialized_len,
            zeroized: Rc::new(Cell::new(false)),
        }
    }

    fn record(&self, bytes: &[u8]) {
        self.zeroized
            .set(bytes.len() == self.initialized_len && bytes.iter().all(|byte| *byte == 0));
    }

    fn initialized_bytes_were_zeroized(&self) -> bool {
        self.zeroized.get()
    }
}

impl fmt::Debug for ExchangeScript {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Success {
                response,
                request_write_chunks,
            } => formatter
                .debug_struct("Success")
                .field("response_len", &response.len())
                .field("request_write_chunks", request_write_chunks)
                .finish(),
            Self::FailBeforeWrite => formatter.write_str("FailBeforeWrite"),
            Self::FailAfterPartialWrite { written_bytes } => formatter
                .debug_struct("FailAfterPartialWrite")
                .field("written_bytes", written_bytes)
                .finish(),
            Self::FailAfterPartialRead {
                written_bytes,
                response_bytes,
            } => formatter
                .debug_struct("FailAfterPartialRead")
                .field("written_bytes", written_bytes)
                .field("response_bytes_len", &response_bytes.len())
                .finish(),
        }
    }
}

/// A one-exchange fake that records bounds and delivery behavior without
/// retaining or formatting request contents.
pub struct ScriptedTransport {
    script: Option<ExchangeScript>,
    exchange_count: usize,
    write_call_count: usize,
    written_byte_count: usize,
    last_response_limit: Option<usize>,
    maximum_response_bytes_retained: usize,
    retained_request: Option<Vec<u8>>,
    captured_logs: Vec<String>,
    #[cfg(test)]
    response_drop_observer: Option<ResponseDropObserver>,
}

impl fmt::Debug for ScriptedTransport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ScriptedTransport")
            .field("script", &self.script)
            .field("exchange_count", &self.exchange_count)
            .field("write_call_count", &self.write_call_count)
            .field("written_byte_count", &self.written_byte_count)
            .field("last_response_limit", &self.last_response_limit)
            .field(
                "maximum_response_bytes_retained",
                &self.maximum_response_bytes_retained,
            )
            .field(
                "retained_request_len",
                &self.retained_request.as_ref().map(Vec::len),
            )
            .field("captured_log_count", &self.captured_logs.len())
            .finish()
    }
}

impl ScriptedTransport {
    /// Creates a fake with one scripted exchange action.
    #[must_use]
    pub fn new(script: ExchangeScript) -> Self {
        Self {
            script: Some(script),
            exchange_count: 0,
            write_call_count: 0,
            written_byte_count: 0,
            last_response_limit: None,
            maximum_response_bytes_retained: 0,
            retained_request: None,
            captured_logs: Vec::new(),
            #[cfg(test)]
            response_drop_observer: None,
        }
    }

    /// Returns how many exchange calls the fake has received.
    #[must_use]
    pub const fn exchange_count(&self) -> usize {
        self.exchange_count
    }

    /// Returns how many non-empty request write chunks the fake accepted.
    #[must_use]
    pub const fn write_call_count(&self) -> usize {
        self.write_call_count
    }

    /// Returns the number of request bytes accepted by the fake.
    #[must_use]
    pub const fn written_byte_count(&self) -> usize {
        self.written_byte_count
    }

    /// Returns the response limit supplied by the most recent exchange.
    #[must_use]
    pub const fn last_response_limit(&self) -> Option<usize> {
        self.last_response_limit
    }

    /// Returns the largest initialized response buffer held during the exchange.
    #[must_use]
    pub const fn maximum_response_bytes_retained(&self) -> usize {
        self.maximum_response_bytes_retained
    }

    /// Returns request bytes retained by the fake, if any.
    #[must_use]
    pub fn retained_request_bytes(&self) -> Option<&[u8]> {
        self.retained_request.as_deref()
    }

    /// Returns captured fake log messages.
    #[must_use]
    pub fn captured_logs(&self) -> &[String] {
        &self.captured_logs
    }

    fn write_bytes(
        &mut self,
        remaining_bytes: usize,
        requested_bytes: usize,
        delivery_state: &mut RequestDeliveryState,
    ) -> usize {
        let accepted_bytes = remaining_bytes.min(requested_bytes);
        if accepted_bytes > 0 {
            self.write_call_count += 1;
            self.written_byte_count += accepted_bytes;
            *delivery_state = delivery_state.write_started();
        }
        accepted_bytes
    }

    fn write_chunks(&mut self, request: &[u8], chunks: &[usize]) -> RequestDeliveryState {
        let mut delivery_state = RequestDeliveryState::not_sent();
        let mut written = 0usize;
        for chunk in chunks {
            written += self.write_bytes(
                request.len().saturating_sub(written),
                *chunk,
                &mut delivery_state,
            );
        }
        if written < request.len() {
            let remaining = request.len() - written;
            self.write_bytes(remaining, remaining, &mut delivery_state);
        }
        delivery_state
    }

    fn response(
        &mut self,
        response_stream: &[u8],
        max_response_bytes: usize,
        delivery_state: RequestDeliveryState,
    ) -> Result<TransportResponse, TransportError> {
        let retained = response_stream.len().min(max_response_bytes);
        let response_bytes = response_stream[..retained].to_vec();
        self.maximum_response_bytes_retained = self.maximum_response_bytes_retained.max(retained);
        let response_state = delivery_state.response_bytes_received(retained);

        if response_stream.len() > max_response_bytes {
            drop(TransportResponse::new(response_bytes));
            return Err(Self::error(response_state));
        }

        Ok(TransportResponse::new(response_bytes))
    }

    fn owned_response_fixture(&self, bytes: Vec<u8>) -> ZeroizingFixtureBytes {
        #[cfg(test)]
        if let Some(observer) = &self.response_drop_observer {
            return ZeroizingFixtureBytes::with_observer(bytes, observer.clone());
        }
        ZeroizingFixtureBytes::new(bytes)
    }

    fn error(delivery_state: RequestDeliveryState) -> TransportError {
        TransportError::new(
            delivery_state,
            TransportCauseCategory::Other,
            io::Error::other("scripted transport failure"),
        )
    }
}

impl Transport for ScriptedTransport {
    fn exchange(
        &mut self,
        request: &[u8],
        max_response_bytes: usize,
    ) -> Result<TransportResponse, TransportError> {
        self.exchange_count += 1;
        self.last_response_limit = Some(max_response_bytes);
        let Some(mut script) = self.script.take() else {
            return Err(Self::error(RequestDeliveryState::not_sent()));
        };

        match &mut script {
            ExchangeScript::Success {
                response,
                request_write_chunks,
            } => {
                let response = self.owned_response_fixture(std::mem::take(response));
                let delivery_state = self.write_chunks(request, request_write_chunks);
                self.response(response.as_slice(), max_response_bytes, delivery_state)
            }
            ExchangeScript::FailBeforeWrite => Err(Self::error(RequestDeliveryState::not_sent())),
            ExchangeScript::FailAfterPartialWrite { written_bytes } => {
                let mut delivery_state = RequestDeliveryState::not_sent();
                self.write_bytes(request.len(), *written_bytes, &mut delivery_state);
                Err(Self::error(delivery_state))
            }
            ExchangeScript::FailAfterPartialRead {
                written_bytes,
                response_bytes,
            } => {
                let response_bytes = self.owned_response_fixture(std::mem::take(response_bytes));
                let mut delivery_state = RequestDeliveryState::not_sent();
                self.write_bytes(request.len(), *written_bytes, &mut delivery_state);
                let response_len = response_bytes.len().min(max_response_bytes);
                self.maximum_response_bytes_retained =
                    self.maximum_response_bytes_retained.max(response_len);
                let response_state = delivery_state.response_bytes_received(response_len);
                drop(TransportResponse::new(
                    response_bytes.as_slice()[..response_len].to_vec(),
                ));
                Err(Self::error(response_state))
            }
        }
    }
}

impl Drop for ScriptedTransport {
    fn drop(&mut self) {
        if let Some(script) = &mut self.script {
            let response = match script {
                ExchangeScript::Success { response, .. } => Some(response),
                ExchangeScript::FailAfterPartialRead { response_bytes, .. } => Some(response_bytes),
                ExchangeScript::FailBeforeWrite | ExchangeScript::FailAfterPartialWrite { .. } => {
                    None
                }
            };
            if let Some(response) = response {
                #[cfg(test)]
                if let Some(observer) = &self.response_drop_observer {
                    observer.record(response);
                }
                #[cfg(not(test))]
                let _ = response;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ExchangeScript, ResponseDropObserver, ScriptedTransport};
    use kmipkit_transport::{RequestDeliveryState, Transport};

    const RESPONSE: &[u8] = b"SCRIPTED_RESPONSE_SECRET_SENTINEL";

    #[test]
    fn successful_exchange_zeroizes_the_owned_response_fixture() {
        let observer = ResponseDropObserver::new(RESPONSE.len());
        let mut transport = ScriptedTransport::new(ExchangeScript::Success {
            response: RESPONSE.to_vec(),
            request_write_chunks: vec![1],
        });
        transport.response_drop_observer = Some(observer.clone());

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
        });
        transport.response_drop_observer = Some(observer.clone());

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
        let mut transport = ScriptedTransport::new(ExchangeScript::Success {
            response: RESPONSE.to_vec(),
            request_write_chunks: vec![1],
        });
        transport.response_drop_observer = Some(observer.clone());

        drop(transport);

        assert!(observer.initialized_bytes_were_zeroized());
    }
}
