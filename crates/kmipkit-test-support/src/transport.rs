//! Scripted low-level fake used by deterministic exchange tests.

use std::{fmt, io};

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
        let Some(script) = self.script.take() else {
            return Err(Self::error(RequestDeliveryState::not_sent()));
        };

        match script {
            ExchangeScript::Success {
                response,
                request_write_chunks,
            } => {
                let delivery_state = self.write_chunks(request, &request_write_chunks);
                self.response(&response, max_response_bytes, delivery_state)
            }
            ExchangeScript::FailBeforeWrite => Err(Self::error(RequestDeliveryState::not_sent())),
            ExchangeScript::FailAfterPartialWrite { written_bytes } => {
                let mut delivery_state = RequestDeliveryState::not_sent();
                self.write_bytes(request.len(), written_bytes, &mut delivery_state);
                Err(Self::error(delivery_state))
            }
            ExchangeScript::FailAfterPartialRead {
                written_bytes,
                response_bytes,
            } => {
                let mut delivery_state = RequestDeliveryState::not_sent();
                self.write_bytes(request.len(), written_bytes, &mut delivery_state);
                let response_len = response_bytes.len().min(max_response_bytes);
                self.maximum_response_bytes_retained =
                    self.maximum_response_bytes_retained.max(response_len);
                let response_state = delivery_state.response_bytes_received(response_len);
                drop(TransportResponse::new(
                    response_bytes[..response_len].to_vec(),
                ));
                Err(Self::error(response_state))
            }
        }
    }
}
