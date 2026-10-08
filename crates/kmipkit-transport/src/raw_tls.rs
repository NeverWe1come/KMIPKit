//! Synchronous raw TTLV over TLS 1.3.

use std::io;
use std::sync::Arc;
use std::time::{Duration, Instant};

use rustls::pki_types::ServerName;
use tokio::net::TcpStream;
use tokio::sync::watch;
use tokio_rustls::TlsConnector;
use tokio_rustls::client::TlsStream;
use zeroize::{Zeroize, Zeroizing};

use crate::config::TransportConfigError;
use crate::resolver::{ResolveFailure, Resolver};
use crate::secret::SecretBuffer;
#[cfg(test)]
use crate::secret::SecretBufferObserver;
use crate::timeout::DeadlineIo;
use crate::tls::{self, TlsClientConfig};
use crate::worker::{ClientWorker, ExchangeControl, WorkerError};
use crate::{
    RequestDeliveryState, RequestOptions, TimeoutLimit, TimeoutPolicy, Transport,
    TransportCauseCategory, TransportConfig, TransportError, TransportResponse,
};

const RESPONSE_HEADER_LEN: usize = 8;
const RESPONSE_STRUCTURE_TAG: [u8; 3] = [0x42, 0x00, 0x78];
const TTLV_STRUCTURE_TYPE: u8 = 0x01;

/// A serialized synchronous adapter for raw TTLV over TLS 1.3 with mutual TLS.
///
/// Construct this adapter from a validated raw-TLS [`TransportConfig`]. The
/// adapter uses the selected trust roots, certificate validity and hostname
/// verification, and caller-supplied CRLs before it dispatches request bytes.
/// Each call sends the caller's bytes unchanged, performs no KMIP encoding or
/// automatic retry, reads at most one bounded response frame, and closes the
/// TLS connection after that frame. The configured timeout policy applies to
/// [`Transport::exchange`]; [`Self::exchange_with_options`] accepts per-call
/// overrides. The per-client worker is started lazily on the first exchange.
///
/// `KMIPKit` zeroizes initialized bytes in its temporary request owner and its
/// response allocation. This does not cover caller-owned bytes or copies made
/// by rustls, AWS-LC, the operating system, or other dependencies.
pub struct RawTlsTransport {
    configuration: TransportConfig,
    host: String,
    port: u16,
    tls: TlsClientConfig,
    resolver: Resolver,
    worker: Option<ClientWorker>,
    #[cfg(test)]
    observer: Option<SecretBufferObserver>,
    #[cfg(test)]
    response_allocation_observer: Option<ResponseAllocationObserver>,
}

impl RawTlsTransport {
    /// Creates a raw-TLS adapter from a validated raw-TLS configuration.
    ///
    /// TLS policy construction is completed before this method returns. No
    /// DNS lookup or socket is opened until the first exchange.
    ///
    /// # Errors
    ///
    /// Returns the fixed configuration error category if the configuration
    /// selects HTTPS or its TLS policy cannot be built.
    pub fn new(configuration: TransportConfig) -> Result<Self, TransportConfigError> {
        let (host, port) = configuration.endpoint().raw_tls_address()?;
        let tls = tls::build_client_config(&configuration)?;
        Ok(Self {
            configuration,
            host,
            port,
            tls,
            resolver: Resolver::system(),
            worker: None,
            #[cfg(test)]
            observer: None,
            #[cfg(test)]
            response_allocation_observer: None,
        })
    }

    /// Exchanges caller-supplied request bytes with per-call timeout overrides.
    ///
    /// Unspecified timeout phases inherit the validated configuration's
    /// [`TimeoutPolicy`]. The request-size limit comes from that configuration;
    /// `max_response_bytes` caps the complete TTLV response frame including its
    /// eight-byte header.
    ///
    /// # Errors
    ///
    /// Returns a redacted error with request-delivery state. A too-large
    /// request or response limit smaller than a TTLV header is rejected before
    /// request dispatch. No dependency or endpoint text is retained.
    pub fn exchange_with_options(
        &mut self,
        request: &[u8],
        max_response_bytes: usize,
        options: &RequestOptions,
    ) -> Result<TransportResponse, TransportError> {
        let exchange_started = Instant::now();
        let policy = self.configuration.timeouts().with_overrides(options);
        if request.len() > self.configuration.max_request_bytes() {
            return Err(safe_error(
                RequestDeliveryState::NotSent,
                TransportCauseCategory::Other,
                io::Error::other("request exceeds configured limit"),
            ));
        }
        if max_response_bytes < RESPONSE_HEADER_LEN {
            return Err(safe_error(
                RequestDeliveryState::NotSent,
                TransportCauseCategory::Other,
                io::Error::other("response limit is smaller than a TTLV header"),
            ));
        }

        if self.worker.is_none() {
            self.worker = Some(ClientWorker::start().map_err(|error| {
                safe_error(
                    RequestDeliveryState::NotSent,
                    TransportCauseCategory::Other,
                    error,
                )
            })?);
        }

        #[cfg(test)]
        let request_owner = if let Some(observer) = self.observer.clone() {
            SecretBuffer::try_copy_from_slice_with_observer_for_test(request, observer)
        } else {
            SecretBuffer::try_copy_from_slice(request)
        };
        #[cfg(not(test))]
        let request_owner = SecretBuffer::try_copy_from_slice(request);
        let request_owner = request_owner.map_err(|error| {
            safe_error(
                RequestDeliveryState::NotSent,
                TransportCauseCategory::Other,
                error,
            )
        })?;

        let total_deadline = deadline_for(policy.total(), exchange_started).map_err(|error| {
            safe_error(
                RequestDeliveryState::NotSent,
                TransportCauseCategory::Other,
                error,
            )
        })?;
        let host = self.host.clone();
        let port = self.port;
        let resolver = self.resolver.clone();
        let client_config = Arc::clone(self.tls.rustls_config_arc());
        let server_name = self.tls.server_name().clone();
        #[cfg(test)]
        let response_allocation_observer = self.response_allocation_observer.clone();
        let result = self.worker.as_ref().map(|worker| {
            worker.exchange(total_deadline, move |control| async move {
                exchange_on_worker(
                    request_owner,
                    host,
                    port,
                    resolver,
                    client_config,
                    server_name,
                    policy,
                    total_deadline,
                    max_response_bytes,
                    control,
                    #[cfg(test)]
                    response_allocation_observer,
                )
                .await
            })
        });

        match result {
            Some(Ok(response)) => Ok(response),
            Some(Err(error)) => Err(worker_error(error)),
            None => Err(safe_error(
                RequestDeliveryState::NotSent,
                TransportCauseCategory::Other,
                io::Error::other("transport worker is unavailable"),
            )),
        }
    }
}

#[cfg(test)]
#[allow(dead_code)] // The source-including T025 target is the only caller.
pub(crate) fn new_for_test(
    configuration: TransportConfig,
    observer: Option<SecretBufferObserver>,
) -> RawTlsTransport {
    let mut adapter = RawTlsTransport::new(configuration)
        .expect("the raw TLS contract supplies validated configuration");
    adapter.observer = observer;
    adapter
}

#[cfg(test)]
#[allow(dead_code)] // The T028 integration target is the only caller.
pub(crate) fn new_for_test_with_response_allocation_observer(
    configuration: TransportConfig,
    observer: ResponseAllocationObserver,
) -> RawTlsTransport {
    let mut adapter = RawTlsTransport::new(configuration)
        .expect("the raw TLS contract supplies validated configuration");
    adapter.response_allocation_observer = Some(observer);
    adapter
}

impl Transport for RawTlsTransport {
    fn exchange(
        &mut self,
        request: &[u8],
        max_response_bytes: usize,
    ) -> Result<TransportResponse, TransportError> {
        self.exchange_with_options(request, max_response_bytes, &RequestOptions::default())
    }
}

#[allow(clippy::too_many_arguments)]
async fn exchange_on_worker(
    request: SecretBuffer,
    host: String,
    port: u16,
    resolver: Resolver,
    client_config: Arc<rustls::ClientConfig>,
    server_name: ServerName<'static>,
    policy: TimeoutPolicy,
    total_deadline: Option<Instant>,
    max_response_bytes: usize,
    control: ExchangeControl,
    #[cfg(test)] response_allocation_observer: Option<ResponseAllocationObserver>,
) -> Result<TransportResponse, TransportError> {
    let connect_deadline = earlier_deadline(
        deadline_for(policy.connect(), Instant::now()).map_err(|error| {
            safe_error(
                control.delivery_state(),
                TransportCauseCategory::Other,
                error,
            )
        })?,
        total_deadline,
    );
    let tls_stream = connect_and_handshake(
        &host,
        port,
        resolver,
        client_config,
        server_name,
        connect_deadline,
        control.clone(),
    )
    .await?;

    RawTlsConnection::new(tls_stream, policy, total_deadline, control)
        .exchange(
            request.as_slice(),
            max_response_bytes,
            #[cfg(test)]
            response_allocation_observer,
        )
        .await
}

/// Owns one raw TLS stream for one request/response exchange.
struct RawTlsConnection {
    io: DeadlineIo<TlsStream<TcpStream>>,
    control: ExchangeControl,
}

impl RawTlsConnection {
    fn new(
        stream: TlsStream<TcpStream>,
        policy: TimeoutPolicy,
        total_deadline: Option<Instant>,
        control: ExchangeControl,
    ) -> Self {
        Self {
            io: DeadlineIo::new(
                stream,
                timeout_duration(policy.read()),
                timeout_duration(policy.write()),
                total_deadline,
                control.clone(),
            ),
            control,
        }
    }

    /// Consumes this connection so every return path drops the raw TLS stream.
    async fn exchange(
        mut self,
        request: &[u8],
        max_response_bytes: usize,
        #[cfg(test)] response_allocation_observer: Option<ResponseAllocationObserver>,
    ) -> Result<TransportResponse, TransportError> {
        if !self.control.commit_dispatch() {
            return Err(safe_error(
                self.control.delivery_state(),
                TransportCauseCategory::Timeout,
                io::Error::new(io::ErrorKind::Interrupted, "request dispatch was canceled"),
            ));
        }

        tokio::io::AsyncWriteExt::write_all(&mut self.io, request)
            .await
            .map_err(|error| io_error(error, self.control.delivery_state()))?;
        tokio::io::AsyncWriteExt::flush(&mut self.io)
            .await
            .map_err(|error| io_error(error, self.control.delivery_state()))?;

        let mut header = Zeroizing::new([0_u8; RESPONSE_HEADER_LEN]);
        tokio::io::AsyncReadExt::read_exact(&mut self.io, header.as_mut())
            .await
            .map_err(|error| io_error(error, self.control.delivery_state()))?;

        let response_value_len = response_value_length(&header[..]).ok_or_else(|| {
            safe_error(
                self.control.delivery_state(),
                TransportCauseCategory::Other,
                io::Error::new(io::ErrorKind::InvalidData, "invalid TTLV response header"),
            )
        })?;
        let response_len = RESPONSE_HEADER_LEN
            .checked_add(response_value_len)
            .filter(|length| *length <= max_response_bytes)
            .ok_or_else(|| {
                safe_error(
                    self.control.delivery_state(),
                    TransportCauseCategory::Other,
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        "response exceeds configured limit",
                    ),
                )
            })?;

        #[cfg(test)]
        let allocation = if let Some(observer) = response_allocation_observer {
            ResponseBuffer::with_header_and_allocation_observer_for_test(
                response_len,
                &header[..],
                &observer,
            )
        } else {
            ResponseBuffer::with_header(response_len, &header[..])
        };
        #[cfg(not(test))]
        let allocation = ResponseBuffer::with_header(response_len, &header[..]);
        let mut response = allocation.map_err(|error| {
            safe_error(
                self.control.delivery_state(),
                TransportCauseCategory::Other,
                error,
            )
        })?;
        if response_len > RESPONSE_HEADER_LEN {
            tokio::io::AsyncReadExt::read_exact(
                &mut self.io,
                &mut response.bytes[RESPONSE_HEADER_LEN..],
            )
            .await
            .map_err(|error| io_error(error, self.control.delivery_state()))?;
        }
        Ok(response.into_transport_response())
    }
}

async fn connect_and_handshake(
    host: &str,
    port: u16,
    resolver: Resolver,
    client_config: Arc<rustls::ClientConfig>,
    server_name: ServerName<'static>,
    deadline: Option<Instant>,
    control: ExchangeControl,
) -> Result<TlsStream<TcpStream>, TransportError> {
    let mut canceled = control.subscribe_cancel();
    let addresses = resolver
        .lookup_candidates_until(host, port, deadline, canceled.clone())
        .await
        .map_err(|failure| resolve_error(failure, control.delivery_state()))?;

    let mut last_connect_error = None;
    let mut connected_stream = None;
    for address in addresses.iter().copied() {
        let result = tokio::select! {
            biased;
            () = wait_for_cancel(&mut canceled) => {
                return Err(canceled_error(control.delivery_state()));
            }
            () = wait_until(deadline) => {
                return Err(timeout_error(control.delivery_state()));
            }
            result = TcpStream::connect(address) => result,
        };
        match result {
            Ok(stream) => {
                connected_stream = Some(stream);
                break;
            }
            Err(error) => last_connect_error = Some(error),
        }
    }
    let stream = connected_stream.ok_or_else(|| {
        io_error(
            last_connect_error.unwrap_or_else(|| {
                io::Error::new(io::ErrorKind::AddrNotAvailable, "no endpoint address")
            }),
            control.delivery_state(),
        )
    })?;

    let connector = TlsConnector::from(client_config);
    tokio::select! {
        biased;
        () = wait_for_cancel(&mut canceled) => Err(canceled_error(control.delivery_state())),
        () = wait_until(deadline) => Err(timeout_error(control.delivery_state())),
        result = connector.connect(server_name, stream) => result.map_err(|_| {
            safe_error(
                control.delivery_state(),
                TransportCauseCategory::Tls,
                io::Error::other("TLS handshake failed"),
            )
        }),
    }
}

fn response_value_length(header: &[u8]) -> Option<usize> {
    if header.len() != RESPONSE_HEADER_LEN
        || header[..3] != RESPONSE_STRUCTURE_TAG
        || header[3] != TTLV_STRUCTURE_TYPE
    {
        return None;
    }
    let value_len = u32::from_be_bytes([header[4], header[5], header[6], header[7]]) as usize;
    if !value_len.is_multiple_of(8) {
        return None;
    }
    Some(value_len)
}

struct ResponseBuffer {
    bytes: Vec<u8>,
    #[cfg(test)]
    drop_observer: Option<ResponseBufferDropObserver>,
}

impl ResponseBuffer {
    fn with_header(length: usize, header: &[u8]) -> io::Result<Self> {
        let header = header
            .get(..RESPONSE_HEADER_LEN)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid TTLV header"))?;
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(length)
            .map_err(|_| io::Error::other("response allocation failed"))?;
        bytes.resize(length, 0);
        bytes[..RESPONSE_HEADER_LEN].copy_from_slice(header);
        Ok(Self {
            bytes,
            #[cfg(test)]
            drop_observer: None,
        })
    }

    #[cfg(test)]
    fn with_header_and_allocation_observer_for_test(
        _length: usize,
        _header: &[u8],
        observer: &ResponseAllocationObserver,
    ) -> io::Result<Self> {
        // Count the constructor attempt and stop before a regression can reserve an untrusted size.
        observer.record_allocation_attempt();
        Err(io::Error::other(
            "response allocation intercepted by test observer",
        ))
    }

    #[cfg(test)]
    fn with_header_and_observer_for_test(
        length: usize,
        header: &[u8],
        observer: ResponseBufferDropObserver,
    ) -> io::Result<Self> {
        let mut response = Self::with_header(length, header)?;
        response.drop_observer = Some(observer);
        Ok(response)
    }

    fn into_transport_response(mut self) -> TransportResponse {
        TransportResponse::new(std::mem::take(&mut self.bytes))
    }
}

#[cfg(test)]
#[derive(Clone, Default)]
/// Counts response-buffer allocation attempts for one test adapter.
pub(crate) struct ResponseAllocationObserver {
    allocation_count: Arc<std::sync::atomic::AtomicUsize>,
}

#[cfg(test)]
#[allow(dead_code)] // Constructed only by the source-including raw TLS integration target.
impl ResponseAllocationObserver {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    fn record_allocation_attempt(&self) {
        self.allocation_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }

    pub(crate) fn allocation_count(&self) -> usize {
        self.allocation_count
            .load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl Drop for ResponseBuffer {
    fn drop(&mut self) {
        #[cfg(test)]
        let contained_nonzero_bytes = self
            .bytes
            .get(RESPONSE_HEADER_LEN..)
            .is_some_and(|body| body.iter().any(|byte| *byte != 0));
        self.bytes.as_mut_slice().zeroize();
        #[cfg(test)]
        if let Some(observer) = &self.drop_observer {
            observer.record(self.bytes.as_slice(), contained_nonzero_bytes);
        }
    }
}

#[cfg(test)]
#[derive(Clone)]
struct ResponseBufferDropObserver {
    expected_len: usize,
    initialized_len: Arc<std::sync::atomic::AtomicUsize>,
    contained_nonzero_bytes: Arc<std::sync::atomic::AtomicBool>,
    initialized_range_was_zero: Arc<std::sync::atomic::AtomicBool>,
}

#[cfg(test)]
impl ResponseBufferDropObserver {
    fn new(expected_len: usize) -> Self {
        Self {
            expected_len,
            initialized_len: Arc::default(),
            contained_nonzero_bytes: Arc::default(),
            initialized_range_was_zero: Arc::default(),
        }
    }

    fn record(&self, initialized_bytes: &[u8], contained_nonzero_bytes: bool) {
        self.initialized_len
            .store(initialized_bytes.len(), std::sync::atomic::Ordering::SeqCst);
        self.contained_nonzero_bytes
            .store(contained_nonzero_bytes, std::sync::atomic::Ordering::SeqCst);
        let all_zero = initialized_bytes.len() == self.expected_len
            && initialized_bytes.iter().all(|byte| *byte == 0);
        self.initialized_range_was_zero
            .store(all_zero, std::sync::atomic::Ordering::SeqCst);
    }

    fn initialized_len(&self) -> usize {
        self.initialized_len
            .load(std::sync::atomic::Ordering::SeqCst)
    }

    fn contained_nonzero_bytes(&self) -> bool {
        self.contained_nonzero_bytes
            .load(std::sync::atomic::Ordering::SeqCst)
    }

    fn initialized_range_was_zero(&self) -> bool {
        self.initialized_range_was_zero
            .load(std::sync::atomic::Ordering::SeqCst)
    }
}

#[cfg(test)]
#[test]
fn response_buffer_zeroizes_partially_read_body_before_release() {
    const HEADER: [u8; RESPONSE_HEADER_LEN] = [0x42, 0x00, 0x78, 0x01, 0, 0, 0, 8];
    const INITIALIZED_BODY: [u8; 4] = [0xA5, 0x5A, 0xC3, 0x3C];
    const RESPONSE_LEN: usize = RESPONSE_HEADER_LEN + 8;

    let observer = ResponseBufferDropObserver::new(RESPONSE_LEN);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("the test runtime is created");
    runtime.block_on(async {
        let (mut source, mut reader) = tokio::io::duplex(INITIALIZED_BODY.len());
        tokio::io::AsyncWriteExt::write_all(&mut source, &INITIALIZED_BODY)
            .await
            .expect("the reader receives a partial response value");
        tokio::io::AsyncWriteExt::shutdown(&mut source)
            .await
            .expect("the partial response source closes");
        let mut response = ResponseBuffer::with_header_and_observer_for_test(
            RESPONSE_LEN,
            &HEADER,
            observer.clone(),
        )
        .expect("a bounded response buffer is reserved");
        let read_result = tokio::io::AsyncReadExt::read_exact(
            &mut reader,
            &mut response.bytes[RESPONSE_HEADER_LEN..],
        )
        .await;
        assert!(
            read_result.is_err(),
            "a truncated response value is rejected"
        );
        drop(response);
    });

    assert_eq!(observer.initialized_len(), RESPONSE_LEN);
    assert!(observer.contained_nonzero_bytes());
    assert!(observer.initialized_range_was_zero());
}

fn timeout_duration(limit: TimeoutLimit) -> Option<Duration> {
    match limit {
        TimeoutLimit::Bounded(duration) => Some(duration),
        TimeoutLimit::Unbounded => None,
    }
}

fn deadline_for(limit: TimeoutLimit, start: Instant) -> io::Result<Option<Instant>> {
    match limit {
        TimeoutLimit::Bounded(duration) => start
            .checked_add(duration)
            .map(Some)
            .ok_or_else(|| io::Error::other("transport deadline exceeds clock range")),
        TimeoutLimit::Unbounded => Ok(None),
    }
}

fn earlier_deadline(first: Option<Instant>, second: Option<Instant>) -> Option<Instant> {
    match (first, second) {
        (Some(first), Some(second)) => Some(first.min(second)),
        (Some(deadline), None) | (None, Some(deadline)) => Some(deadline),
        (None, None) => None,
    }
}

async fn wait_until(deadline: Option<Instant>) {
    if let Some(deadline) = deadline {
        tokio::time::sleep_until(deadline.into()).await;
    } else {
        std::future::pending::<()>().await;
    }
}

async fn wait_for_cancel(canceled: &mut watch::Receiver<bool>) {
    while !*canceled.borrow() {
        if canceled.changed().await.is_err() {
            return;
        }
    }
}

fn resolve_error(failure: ResolveFailure, state: RequestDeliveryState) -> TransportError {
    let (cause, message) = match failure {
        ResolveFailure::Deadline => (TransportCauseCategory::Timeout, "name resolution timed out"),
        ResolveFailure::Cancelled => (TransportCauseCategory::Timeout, "name resolution canceled"),
        ResolveFailure::Capacity => (
            TransportCauseCategory::Other,
            "resolver capacity unavailable",
        ),
        ResolveFailure::Lookup => (TransportCauseCategory::Io, "name resolution failed"),
    };
    safe_error(state, cause, io::Error::other(message))
}

fn worker_error(error: WorkerError) -> TransportError {
    let delivery_state = error.delivery_state();
    match error.into_operation_source() {
        Some(source) => safe_error(delivery_state, source.cause_category(), source),
        None => safe_error(
            delivery_state,
            match error {
                WorkerError::Deadline(_) => TransportCauseCategory::Timeout,
                WorkerError::Closed(_)
                | WorkerError::Stopped(_)
                | WorkerError::Operation { .. } => TransportCauseCategory::Other,
            },
            io::Error::other("transport worker exchange failed"),
        ),
    }
}

fn io_error(error: io::Error, state: RequestDeliveryState) -> TransportError {
    let cause = if error.kind() == io::ErrorKind::TimedOut {
        TransportCauseCategory::Timeout
    } else {
        TransportCauseCategory::Io
    };
    safe_error(state, cause, error)
}

fn timeout_error(state: RequestDeliveryState) -> TransportError {
    safe_error(
        state,
        TransportCauseCategory::Timeout,
        io::Error::new(io::ErrorKind::TimedOut, "transport deadline elapsed"),
    )
}

fn canceled_error(state: RequestDeliveryState) -> TransportError {
    safe_error(
        state,
        TransportCauseCategory::Timeout,
        io::Error::new(io::ErrorKind::Interrupted, "transport exchange canceled"),
    )
}

fn safe_error<E: std::error::Error + 'static>(
    delivery_state: RequestDeliveryState,
    cause: TransportCauseCategory,
    source: E,
) -> TransportError {
    TransportError::new(delivery_state, cause, source)
}
