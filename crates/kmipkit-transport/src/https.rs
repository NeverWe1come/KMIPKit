//! Synchronous HTTPS with TTLV over verified TLS 1.3 and HTTP/1.1.

use std::convert::Infallible;
use std::future::poll_fn;
use std::io;
use std::pin::Pin;
use std::sync::Arc;
#[cfg(test)]
use std::sync::mpsc;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

use bytes::Bytes;
use hyper::body::{Body, Frame, Incoming, SizeHint};
use hyper::client::conn::http1;
use hyper::header::{CACHE_CONTROL, CONTENT_LENGTH, CONTENT_TYPE, HOST};
use hyper::http::{Method, Request};
use rustls::pki_types::ServerName;
use tokio::net::TcpStream;
use tokio::sync::watch;
use tokio_rustls::TlsConnector;
use tokio_rustls::client::TlsStream;
use zeroize::Zeroize;

use crate::config::TransportConfigError;
use crate::resolver::{ResolveFailure, Resolver};
use crate::secret::SecretBuffer;
#[cfg(test)]
use crate::secret::SecretBufferObserver;
use crate::timeout::{DeadlineIo, send_request_when_ready};
#[cfg(test)]
use crate::worker::WorkerTask;
use crate::worker::{ClientWorker, ExchangeControl, WorkerError, WorkerStartError};
use crate::{
    RequestDeliveryState, RequestOptions, TimeoutLimit, TimeoutPolicy, Transport,
    TransportCauseCategory, TransportConfig, TransportError, TransportResponse,
};
#[cfg(test)]
use std::thread::JoinHandle;

/// A synchronous HTTPS/HTTP 1.1 adapter using TLS 1.3 mutual authentication.
///
/// The adapter posts caller-supplied bytes to the validated origin-form target
/// and never retries automatically. It sends request bytes only after the
/// verified TLS connection and Hyper sender are ready. Each exchange uses one
/// direct connection; proxy settings are not consulted. The default timeout
/// policy comes from the validated configuration, and
/// [`Self::exchange_with_options`] accepts per-call overrides.
///
/// `KMIPKit` zeroizes initialized bytes in its staged request owner and partial
/// response allocation. This does not cover caller-owned input or copies made
/// by Hyper, rustls, AWS-LC, the operating system, or other dependencies.
pub struct HttpsTransport {
    configuration: TransportConfig,
    route: HttpsRoute,
    tls: crate::tls::TlsClientConfig,
    resolver: Resolver,
    worker: Option<ClientWorker>,
    #[cfg(test)]
    observer: Option<SecretBufferObserver>,
    #[cfg(test)]
    worker_spawner: Option<WorkerSpawnerForTest>,
    #[cfg(test)]
    driver_abort_observer: Option<mpsc::SyncSender<()>>,
    #[cfg(test)]
    cancel_exchange: Option<tokio::sync::oneshot::Receiver<()>>,
    #[cfg(test)]
    driver_cleanup_gate: Option<DriverCleanupGateForTest>,
}

#[cfg(test)]
pub(crate) struct DriverCleanupGateForTest {
    pub(crate) started: mpsc::SyncSender<()>,
    pub(crate) release: tokio::sync::oneshot::Receiver<()>,
    pub(crate) acknowledged: mpsc::SyncSender<()>,
}

// Keep endpoint routing and its origin-form target together. The target never
// supplies the resolver authority or HTTP Host; TLS server-name selection
// remains owned by the independently validated TLS configuration.
#[derive(Clone)]
struct HttpsRoute {
    resolver_host: String,
    port: u16,
    authority: String,
    target: String,
}

impl HttpsRoute {
    fn from_configuration(configuration: &TransportConfig) -> Result<Self, TransportConfigError> {
        let (resolver_host, port, authority) =
            configuration.endpoint().https_connection_details()?;
        Ok(Self {
            resolver_host,
            port,
            authority,
            target: configuration.target_uri().unwrap_or("/kmip").to_owned(),
        })
    }
}

#[cfg(test)]
type WorkerSpawnerForTest =
    Box<dyn FnOnce(WorkerTask) -> io::Result<JoinHandle<()>> + Send + 'static>;

impl HttpsTransport {
    /// Creates an HTTPS adapter from a validated HTTPS configuration.
    ///
    /// TLS policy construction finishes before return. No DNS lookup or socket
    /// is opened until the first exchange.
    ///
    /// # Errors
    ///
    /// Returns the fixed configuration error if the endpoint is not HTTPS or
    /// its TLS policy cannot be built.
    pub fn new(configuration: TransportConfig) -> Result<Self, TransportConfigError> {
        let route = HttpsRoute::from_configuration(&configuration)?;
        let tls = crate::tls::build_client_config(&configuration)?;
        Ok(Self {
            configuration,
            route,
            tls,
            resolver: Resolver::system(),
            worker: None,
            #[cfg(test)]
            observer: None,
            #[cfg(test)]
            worker_spawner: None,
            #[cfg(test)]
            driver_abort_observer: None,
            #[cfg(test)]
            cancel_exchange: None,
            #[cfg(test)]
            driver_cleanup_gate: None,
        })
    }

    /// Exchanges caller bytes using timeout overrides for this request.
    ///
    /// Unspecified timeout phases inherit the validated configuration's
    /// [`TimeoutPolicy`]. The response cap is enforced while streaming the
    /// HTTP message body.
    ///
    /// # Errors
    ///
    /// Returns a redacted error with the strongest available request-delivery
    /// state. Oversized requests and invalid response caps fail before
    /// dispatch. KMIPKit-owned staged request and partial response bytes are
    /// zeroized before this method returns.
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
        if max_response_bytes == 0 {
            return Err(safe_error(
                RequestDeliveryState::NotSent,
                TransportCauseCategory::Other,
                io::Error::other("response limit must be positive"),
            ));
        }
        let total_deadline =
            validated_total_deadline(policy, exchange_started).map_err(|error| {
                safe_error(
                    RequestDeliveryState::NotSent,
                    TransportCauseCategory::Other,
                    error,
                )
            })?;

        if self.worker.is_none() {
            #[cfg(test)]
            let worker = match self.worker_spawner.take() {
                Some(spawner) => ClientWorker::start_with_spawner_until(spawner, total_deadline),
                None => ClientWorker::start_until(total_deadline),
            };
            #[cfg(not(test))]
            let worker = ClientWorker::start_until(total_deadline);
            self.worker = Some(worker.map_err(|error| match error {
                WorkerStartError::Deadline => timeout_error(RequestDeliveryState::NotSent),
                error => safe_error(
                    RequestDeliveryState::NotSent,
                    TransportCauseCategory::Other,
                    error,
                ),
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

        let route = self.route.clone();
        let resolver = self.resolver.clone();
        let client_config = Arc::clone(self.tls.rustls_config_arc());
        let server_name = self.tls.server_name().clone();
        #[cfg(test)]
        let driver_abort_observer = self.driver_abort_observer.take();
        #[cfg(test)]
        let cancel_exchange = self.cancel_exchange.take();
        #[cfg(test)]
        let driver_cleanup_gate = self.driver_cleanup_gate.take();
        let worker = self.worker.as_ref().ok_or_else(|| {
            safe_error(
                RequestDeliveryState::NotSent,
                TransportCauseCategory::Other,
                io::Error::other("transport worker is unavailable"),
            )
        })?;

        worker
            .exchange(total_deadline, move |control| async move {
                exchange_on_worker(
                    request_owner,
                    route,
                    resolver,
                    client_config,
                    server_name,
                    policy,
                    total_deadline,
                    max_response_bytes,
                    control,
                    #[cfg(test)]
                    driver_abort_observer,
                    #[cfg(test)]
                    cancel_exchange,
                    #[cfg(test)]
                    driver_cleanup_gate,
                )
                .await
            })
            .map_err(worker_error)
    }
}

#[cfg(test)]
#[allow(dead_code)] // The source-included HTTPS integration target supplies these seams.
pub(crate) fn new_for_test_with_resolver(
    configuration: TransportConfig,
    observer: Option<SecretBufferObserver>,
    resolver: Resolver,
) -> HttpsTransport {
    let mut adapter = HttpsTransport::new(configuration)
        .expect("the HTTPS test contract supplies validated configuration");
    adapter.observer = observer;
    adapter.resolver = resolver;
    adapter
}

#[cfg(test)]
#[allow(dead_code)] // The source-included integration target uses this adapter seam.
pub(crate) fn new_for_test_with_resolver_and_driver_abort_observer(
    configuration: TransportConfig,
    resolver: Resolver,
    driver_abort_observer: mpsc::SyncSender<()>,
    cancel_exchange: tokio::sync::oneshot::Receiver<()>,
) -> HttpsTransport {
    let mut adapter = new_for_test_with_resolver(configuration, None, resolver);
    adapter.driver_abort_observer = Some(driver_abort_observer);
    adapter.cancel_exchange = Some(cancel_exchange);
    adapter
}

#[cfg(test)]
#[allow(dead_code)] // The source-included HTTPS integration target uses this cleanup gate.
pub(crate) fn new_for_test_with_resolver_and_driver_cleanup_gate(
    configuration: TransportConfig,
    resolver: Resolver,
    driver_cleanup_gate: DriverCleanupGateForTest,
    cancel_exchange: tokio::sync::oneshot::Receiver<()>,
) -> HttpsTransport {
    let mut adapter = new_for_test_with_resolver(configuration, None, resolver);
    adapter.driver_cleanup_gate = Some(driver_cleanup_gate);
    adapter.cancel_exchange = Some(cancel_exchange);
    adapter
}

#[cfg(test)]
#[allow(dead_code)]
pub(crate) fn build_request_for_test(
    configuration: &TransportConfig,
    request: &[u8],
) -> Result<Request<impl Body<Data = Bytes, Error = Infallible>>, hyper::http::Error> {
    let route = HttpsRoute::from_configuration(configuration)
        .expect("the request-builder test supplies a validated HTTPS endpoint");
    let owner = SecretBuffer::try_copy_from_slice(request)
        .expect("the bounded test request owner allocates");
    build_http_request(&route, owner)
}

impl Transport for HttpsTransport {
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
    request_owner: SecretBuffer,
    route: HttpsRoute,
    resolver: Resolver,
    client_config: Arc<rustls::ClientConfig>,
    server_name: ServerName<'static>,
    policy: TimeoutPolicy,
    total_deadline: Option<Instant>,
    max_response_bytes: usize,
    control: ExchangeControl,
    #[cfg(test)] driver_abort_observer: Option<mpsc::SyncSender<()>>,
    #[cfg(test)] cancel_exchange: Option<tokio::sync::oneshot::Receiver<()>>,
    #[cfg(test)] driver_cleanup_gate: Option<DriverCleanupGateForTest>,
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
        &route.resolver_host,
        route.port,
        resolver,
        client_config,
        server_name,
        connect_deadline,
        control.clone(),
    )
    .await?;

    let io = DeadlineIo::new(
        tls_stream,
        timeout_duration(policy.read()),
        timeout_duration(policy.write()),
        total_deadline,
        control.clone(),
    );
    io.validate_phase_deadlines().map_err(|error| {
        safe_error(
            RequestDeliveryState::NotSent,
            TransportCauseCategory::Other,
            error,
        )
    })?;
    let (mut sender, connection) = http1::handshake::<_, RequestBody>(io).await.map_err(|_| {
        safe_error(
            control.delivery_state(),
            TransportCauseCategory::Http,
            io::Error::other("HTTP/1 connection could not be initialized"),
        )
    })?;
    let driver = HyperDriverGuard {
        task: Some(tokio::spawn(connection)),
        #[cfg(test)]
        abort_observer: driver_abort_observer,
        #[cfg(test)]
        cleanup_gate: driver_cleanup_gate,
    };
    #[cfg(test)]
    if let Some(cancel_exchange) = cancel_exchange {
        let cancellation_control = control.clone();
        tokio::spawn(async move {
            if cancel_exchange.await.is_ok() {
                cancellation_control.cancel();
            }
        });
    }

    let result = async {
        let request = build_http_request(&route, request_owner).map_err(|_| {
            safe_error(
                RequestDeliveryState::NotSent,
                TransportCauseCategory::Other,
                io::Error::other("HTTP request construction failed"),
            )
        })?;
        let mut request = Some(request);
        let response = send_request_when_ready(
            &mut sender,
            &mut request,
            &control,
            timeout_duration(policy.write()),
            total_deadline,
        )
        .await
        .map_err(|error| io_error(error, control.delivery_state()))?;
        read_response_body(response.into_body(), max_response_bytes, &control).await
    }
    .await;
    drop(driver);
    result
}

struct HyperDriverGuard {
    task: Option<tokio::task::JoinHandle<Result<(), hyper::Error>>>,
    #[cfg(test)]
    abort_observer: Option<mpsc::SyncSender<()>>,
    #[cfg(test)]
    cleanup_gate: Option<DriverCleanupGateForTest>,
}

impl Drop for HyperDriverGuard {
    fn drop(&mut self) {
        let Some(task) = self.task.take() else {
            return;
        };
        task.abort();
        #[cfg(test)]
        if let Some(observer) = self.abort_observer.take() {
            let _ = observer.send(());
        }
        #[cfg(test)]
        if let Some(gate) = self.cleanup_gate.take() {
            let _ = gate.started.send(());
            tokio::spawn(async move {
                let _ = gate.release.await;
                let _ = task.await;
                let _ = gate.acknowledged.send(());
            });
            return;
        }
        drop(task);
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
    let connector = TlsConnector::from(client_config);
    let mut last_connect_error = None;
    let mut handshake_failed = false;

    for address in addresses {
        let stream = tokio::select! {
            biased;
            () = wait_for_cancel(&mut canceled) => {
                return Err(canceled_error(control.delivery_state()));
            }
            () = wait_until(deadline) => {
                return Err(timeout_error(control.delivery_state()));
            }
            result = TcpStream::connect(address) => match result {
                Ok(stream) => stream,
                Err(error) => {
                    last_connect_error = Some(error);
                    continue;
                }
            },
        };
        let result = tokio::select! {
            biased;
            () = wait_for_cancel(&mut canceled) => {
                return Err(canceled_error(control.delivery_state()));
            }
            () = wait_until(deadline) => {
                return Err(timeout_error(control.delivery_state()));
            }
            result = connector.connect(server_name.clone(), stream) => result,
        };
        if let Ok(stream) = result {
            return Ok(stream);
        }
        handshake_failed = true;
    }

    if handshake_failed {
        Err(safe_error(
            control.delivery_state(),
            TransportCauseCategory::Tls,
            io::Error::other("TLS handshake failed"),
        ))
    } else {
        Err(io_error(
            last_connect_error.unwrap_or_else(|| {
                io::Error::new(io::ErrorKind::AddrNotAvailable, "no endpoint address")
            }),
            control.delivery_state(),
        ))
    }
}

async fn read_response_body(
    mut body: Incoming,
    max_response_bytes: usize,
    control: &ExchangeControl,
) -> Result<TransportResponse, TransportError> {
    let mut response = ResponseBuffer::new();
    loop {
        let frame = poll_fn(|cx| Pin::new(&mut body).poll_frame(cx)).await;
        let Some(frame) = frame else {
            return Ok(TransportResponse::new(response.into_bytes()));
        };
        let frame = frame.map_err(|_| {
            safe_error(
                control.delivery_state(),
                TransportCauseCategory::Http,
                io::Error::other("HTTP response body failed"),
            )
        })?;
        if let Ok(data) = frame.into_data() {
            response
                .append(&data, max_response_bytes)
                .map_err(|error| {
                    safe_error(
                        control.delivery_state(),
                        TransportCauseCategory::Other,
                        error,
                    )
                })?;
        }
    }
}

struct ResponseBuffer(Vec<u8>);

impl ResponseBuffer {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn append(&mut self, bytes: &[u8], limit: usize) -> io::Result<()> {
        let new_len = self
            .0
            .len()
            .checked_add(bytes.len())
            .filter(|length| *length <= limit)
            .ok_or_else(|| io::Error::other("HTTP response exceeds configured limit"))?;
        if new_len <= self.0.capacity() {
            self.0.extend_from_slice(bytes);
            return Ok(());
        }

        let capacity = self.0.capacity().saturating_mul(2).max(new_len).min(limit);
        let mut replacement = Vec::new();
        replacement
            .try_reserve_exact(capacity)
            .map_err(|_| io::Error::other("HTTP response allocation failed"))?;
        replacement.extend_from_slice(&self.0);
        replacement.extend_from_slice(bytes);
        self.0.as_mut_slice().zeroize();
        self.0 = replacement;
        Ok(())
    }

    fn into_bytes(mut self) -> Vec<u8> {
        std::mem::take(&mut self.0)
    }
}

impl Drop for ResponseBuffer {
    fn drop(&mut self) {
        self.0.as_mut_slice().zeroize();
    }
}

struct RequestBodyOwner(SecretBuffer);

impl AsRef<[u8]> for RequestBodyOwner {
    fn as_ref(&self) -> &[u8] {
        self.0.as_slice()
    }
}

struct RequestBody {
    data: Option<Bytes>,
    length: u64,
}

impl RequestBody {
    fn from_owner(owner: SecretBuffer) -> Self {
        let length = owner.as_slice().len() as u64;
        Self {
            data: Some(Bytes::from_owner(RequestBodyOwner(owner))),
            length,
        }
    }
}

impl Body for RequestBody {
    type Data = Bytes;
    type Error = Infallible;

    fn poll_frame(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        Poll::Ready(self.get_mut().data.take().map(Frame::data).map(Ok))
    }

    fn is_end_stream(&self) -> bool {
        self.data.is_none()
    }

    fn size_hint(&self) -> SizeHint {
        SizeHint::with_exact(self.length)
    }
}

fn build_http_request(
    route: &HttpsRoute,
    request_owner: SecretBuffer,
) -> Result<Request<RequestBody>, hyper::http::Error> {
    let body = RequestBody::from_owner(request_owner);
    let body_length = body.length;
    Request::builder()
        .method(Method::POST)
        .uri(&route.target)
        .header(HOST, &route.authority)
        .header(CONTENT_TYPE, "application/octet-stream")
        .header(CONTENT_LENGTH, body_length.to_string())
        .header(CACHE_CONTROL, "no-cache")
        .body(body)
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

fn validated_total_deadline(
    policy: TimeoutPolicy,
    exchange_started: Instant,
) -> io::Result<Option<Instant>> {
    let total_deadline = deadline_for(policy.total(), exchange_started)?;
    let _ = deadline_for(policy.connect(), exchange_started)?;
    let _ = deadline_for(policy.read(), exchange_started)?;
    let _ = deadline_for(policy.write(), exchange_started)?;
    Ok(total_deadline)
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
    let cause = match error.kind() {
        io::ErrorKind::TimedOut => TransportCauseCategory::Timeout,
        io::ErrorKind::InvalidInput => TransportCauseCategory::Other,
        _ => TransportCauseCategory::Io,
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
