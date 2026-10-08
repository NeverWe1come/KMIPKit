//! Synchronous HTTPS with TTLV over verified TLS 1.3 and HTTP/1.1.

use std::convert::Infallible;
use std::future::poll_fn;
use std::io;
use std::pin::Pin;
#[cfg(test)]
use std::sync::mpsc;
use std::sync::{Arc, Mutex as StdMutex, MutexGuard};
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

use bytes::Bytes;
use hyper::body::{Body, Frame, Incoming, SizeHint};
use hyper::client::conn::http1;
use hyper::header::{
    CACHE_CONTROL, CONTENT_ENCODING, CONTENT_LENGTH, CONTENT_TYPE, HOST, TRANSFER_ENCODING,
};
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
use crate::timeout::DeadlineIoControl;
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
/// verified TLS connection and Hyper sender are ready. It maintains at most
/// one direct connection per client and reuses it across healthy exchanges;
/// redirects and server-supplied endpoints are not followed, proxy settings
/// are not consulted, cookies are not retained, and compression is not
/// negotiated. TLS ALPN is empty and the adapter uses Hyper's HTTP/1 driver.
/// The default timeout policy comes from the validated configuration, and
/// [`Self::exchange_with_options`] accepts per-call overrides.
///
/// `KMIPKit` zeroizes initialized bytes in its staged request owner and partial
/// response allocation. This does not cover caller-owned input or copies made
/// by Hyper, rustls, AWS-LC, the operating system, or other dependencies.
pub struct HttpsTransport {
    configuration: TransportConfig,
    route: HttpsRoute,
    tls: crate::tls::TlsClientConfig,
    http1_tls_config: Arc<rustls::ClientConfig>,
    resolver: Resolver,
    connection: Arc<StdMutex<Option<HttpsConnection>>>,
    worker: Option<ClientWorker>,
    #[cfg(test)]
    observer: Option<SecretBufferObserver>,
    #[cfg(test)]
    response_buffer_observer: Option<ResponseBufferObserver>,
    #[cfg(test)]
    worker_spawner: Option<WorkerSpawnerForTest>,
    #[cfg(test)]
    driver_abort_observer: Option<mpsc::SyncSender<()>>,
    #[cfg(test)]
    cancel_exchange: Option<tokio::sync::oneshot::Receiver<()>>,
    #[cfg(test)]
    driver_cleanup_gate: Option<DriverCleanupGateForTest>,
    #[cfg(test)]
    cancel_before_finish_for_test: bool,
}

#[cfg(test)]
pub(crate) struct DriverCleanupGateForTest {
    pub(crate) release: tokio::sync::oneshot::Receiver<()>,
    pub(crate) events: mpsc::Sender<DriverCleanupEventForTest>,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)] // The source-included HTTPS integration target records exchange return.
pub(crate) enum DriverCleanupEventForTest {
    Started,
    Acknowledged,
    ExchangeReturned,
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

struct HttpsConnectionSettings {
    route: HttpsRoute,
    resolver: Resolver,
    client_config: Arc<rustls::ClientConfig>,
    server_name: ServerName<'static>,
    policy: TimeoutPolicy,
    total_deadline: Option<Instant>,
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

fn build_http1_tls_config(tls_config: &Arc<rustls::ClientConfig>) -> Arc<rustls::ClientConfig> {
    let mut http1_config = tls_config.as_ref().clone();
    // The HTTPS adapter only has a Hyper HTTP/1 driver; do not advertise a
    // different application protocol during TLS negotiation.
    http1_config.alpn_protocols.clear();
    Arc::new(http1_config)
}

#[cfg(test)]
#[path = "../tests/unit/https_alpn_tests.rs"]
mod alpn_tests;
#[cfg(test)]
#[path = "../tests/unit/https_error_tests.rs"]
mod error_tests;
#[cfg(test)]
#[path = "../tests/unit/https_request_tests.rs"]
mod request_tests;
#[cfg(test)]
#[path = "../tests/unit/https_response_tests.rs"]
mod response_tests;
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
        let http1_tls_config = build_http1_tls_config(tls.rustls_config_arc());
        Ok(Self {
            configuration,
            route,
            tls,
            http1_tls_config,
            resolver: Resolver::system(),
            connection: Arc::new(StdMutex::new(None)),
            worker: None,
            #[cfg(test)]
            observer: None,
            #[cfg(test)]
            response_buffer_observer: None,
            #[cfg(test)]
            worker_spawner: None,
            #[cfg(test)]
            driver_abort_observer: None,
            #[cfg(test)]
            cancel_exchange: None,
            #[cfg(test)]
            driver_cleanup_gate: None,
            #[cfg(test)]
            cancel_before_finish_for_test: false,
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

        let request_owner = Self::stage_request_owner(
            request,
            #[cfg(test)]
            self.observer.clone(),
        )?;

        let route = self.route.clone();
        let resolver = self.resolver.clone();
        let connection = Arc::clone(&self.connection);
        let client_config = Arc::clone(&self.http1_tls_config);
        let server_name = self.tls.server_name().clone();
        #[cfg(test)]
        let response_buffer_observer = self.response_buffer_observer.take();
        #[cfg(test)]
        let driver_abort_observer = self.driver_abort_observer.take();
        #[cfg(test)]
        let cancel_exchange = self.cancel_exchange.take();
        #[cfg(test)]
        let driver_cleanup_gate = self.driver_cleanup_gate.take();
        #[cfg(test)]
        let cancel_before_finish_for_test = std::mem::take(&mut self.cancel_before_finish_for_test);
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
                    connection,
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
                    #[cfg(test)]
                    response_buffer_observer,
                    #[cfg(test)]
                    cancel_before_finish_for_test,
                )
                .await
            })
            .map_err(worker_error)
    }

    fn stage_request_owner(
        request: &[u8],
        #[cfg(test)] observer: Option<SecretBufferObserver>,
    ) -> Result<SecretBuffer, TransportError> {
        #[cfg(test)]
        let owner = if let Some(observer) = observer {
            SecretBuffer::try_copy_from_slice_with_observer_for_test(request, observer)
        } else {
            SecretBuffer::try_copy_from_slice(request)
        };
        #[cfg(not(test))]
        let owner = SecretBuffer::try_copy_from_slice(request);
        owner.map_err(|error| {
            safe_error(
                RequestDeliveryState::NotSent,
                TransportCauseCategory::Other,
                error,
            )
        })
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
pub(crate) fn new_for_test_with_worker_spawner(
    configuration: TransportConfig,
    resolver: Resolver,
    spawner: impl FnOnce(WorkerTask) -> io::Result<JoinHandle<()>> + Send + 'static,
) -> HttpsTransport {
    let mut adapter = new_for_test_with_resolver(configuration, None, resolver);
    adapter.worker_spawner = Some(Box::new(spawner));
    adapter
}

#[cfg(test)]
#[allow(dead_code)] // The source-included HTTPS response contract uses this observer.
pub(crate) fn new_for_test_with_response_buffer_observer(
    configuration: TransportConfig,
    resolver: Resolver,
    observer: ResponseBufferObserver,
) -> HttpsTransport {
    let mut adapter = new_for_test_with_resolver(configuration, None, resolver);
    adapter.response_buffer_observer = Some(observer);
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
pub(crate) fn new_for_test_canceling_before_success_finish(
    configuration: TransportConfig,
    resolver: Resolver,
) -> HttpsTransport {
    let mut adapter = new_for_test_with_resolver(configuration, None, resolver);
    adapter.cancel_before_finish_for_test = true;
    adapter
}

#[cfg(test)]
#[allow(dead_code)]
pub(crate) fn new_for_test_canceling_before_success_finish_with_cleanup_gate(
    configuration: TransportConfig,
    resolver: Resolver,
    cleanup_gate: DriverCleanupGateForTest,
) -> HttpsTransport {
    let mut adapter = new_for_test_canceling_before_success_finish(configuration, resolver);
    adapter.driver_cleanup_gate = Some(cleanup_gate);
    adapter
}

#[cfg(test)]
#[allow(dead_code)]
pub(crate) fn has_cached_connection_for_test(adapter: &HttpsTransport) -> bool {
    lock_https_connection(&adapter.connection).is_some()
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
    connection_state: Arc<StdMutex<Option<HttpsConnection>>>,
    client_config: Arc<rustls::ClientConfig>,
    server_name: ServerName<'static>,
    policy: TimeoutPolicy,
    total_deadline: Option<Instant>,
    max_response_bytes: usize,
    control: ExchangeControl,
    #[cfg(test)] driver_abort_observer: Option<mpsc::SyncSender<()>>,
    #[cfg(test)] cancel_exchange: Option<tokio::sync::oneshot::Receiver<()>>,
    #[cfg(test)] driver_cleanup_gate: Option<DriverCleanupGateForTest>,
    #[cfg(test)] response_buffer_observer: Option<ResponseBufferObserver>,
    #[cfg(test)] cancel_before_finish_for_test: bool,
) -> Result<TransportResponse, TransportError> {
    #[cfg(test)]
    if let Some(cancel_exchange) = cancel_exchange {
        let cancellation_control = control.clone();
        tokio::spawn(async move {
            if cancel_exchange.await.is_ok() {
                cancellation_control.cancel();
            }
        });
    }

    let settings = HttpsConnectionSettings {
        route,
        resolver,
        client_config,
        server_name,
        policy,
        total_deadline,
    };
    let mut session = acquire_https_connection(
        &connection_state,
        &settings,
        &control,
        #[cfg(test)]
        driver_abort_observer,
        #[cfg(test)]
        driver_cleanup_gate,
    )
    .await?;
    let result = execute_https_exchange(
        &mut session,
        request_owner,
        &settings,
        max_response_bytes,
        &control,
        #[cfg(test)]
        response_buffer_observer,
    )
    .await;
    if result.is_ok() {
        session.io_control.finish_exchange();
        #[cfg(test)]
        if cancel_before_finish_for_test {
            control.cancel();
        }
        if session.is_reusable() {
            // Completion and cancellation share one atomic gate; publish only
            // when completion wins it.
            let (completed, _) = control.finish();
            if completed {
                *lock_https_connection(&connection_state) = Some(session);
            }
        }
    }
    result
}

async fn acquire_https_connection(
    connection_state: &StdMutex<Option<HttpsConnection>>,
    settings: &HttpsConnectionSettings,
    control: &ExchangeControl,
    #[cfg(test)] mut abort_observer: Option<mpsc::SyncSender<()>>,
    #[cfg(test)] mut cleanup_gate: Option<DriverCleanupGateForTest>,
) -> Result<HttpsConnection, TransportError> {
    // Give the idle HTTP/1 driver a chance to reject bytes queued after the
    // previous response before this exchange can dispatch its request.
    tokio::task::yield_now().await;
    if let Some(mut existing) = lock_https_connection(connection_state).take() {
        if existing.is_reusable() {
            existing.set_cleanup_context(
                control.clone(),
                #[cfg(test)]
                abort_observer.take(),
                #[cfg(test)]
                cleanup_gate.take(),
            );
            existing
                .begin_exchange(
                    timeout_duration(settings.policy.read()),
                    timeout_duration(settings.policy.write()),
                    settings.total_deadline,
                    control.clone(),
                )
                .map_err(|error| {
                    safe_error(
                        RequestDeliveryState::NotSent,
                        TransportCauseCategory::Other,
                        error,
                    )
                })?;
            return Ok(existing);
        }
        existing.set_cleanup_context(
            control.clone(),
            #[cfg(test)]
            None,
            #[cfg(test)]
            None,
        );
    }
    connect_https_connection(
        settings,
        control.clone(),
        #[cfg(test)]
        abort_observer,
        #[cfg(test)]
        cleanup_gate,
    )
    .await
}

async fn connect_https_connection(
    settings: &HttpsConnectionSettings,
    control: ExchangeControl,
    #[cfg(test)] abort_observer: Option<mpsc::SyncSender<()>>,
    #[cfg(test)] cleanup_gate: Option<DriverCleanupGateForTest>,
) -> Result<HttpsConnection, TransportError> {
    let connect_deadline = earlier_deadline(
        deadline_for(settings.policy.connect(), Instant::now()).map_err(|error| {
            safe_error(
                control.delivery_state(),
                TransportCauseCategory::Other,
                error,
            )
        })?,
        settings.total_deadline,
    );
    let tls_stream = connect_and_handshake(
        &settings.route.resolver_host,
        settings.route.port,
        settings.resolver.clone(),
        Arc::clone(&settings.client_config),
        settings.server_name.clone(),
        connect_deadline,
        control.clone(),
    )
    .await?;
    let (io, io_control) = DeadlineIo::new_reusable(
        tls_stream,
        timeout_duration(settings.policy.read()),
        timeout_duration(settings.policy.write()),
        settings.total_deadline,
        control.clone(),
    );
    io.validate_phase_deadlines().map_err(|error| {
        safe_error(
            RequestDeliveryState::NotSent,
            TransportCauseCategory::Other,
            error,
        )
    })?;
    let mut builder = http1::Builder::new();
    builder.max_headers(64);
    builder.max_header_size(64 * 1024);
    builder.max_buf_size(64 * 1024);
    let (sender, connection) = builder.handshake::<_, RequestBody>(io).await.map_err(|_| {
        safe_error(
            control.delivery_state(),
            TransportCauseCategory::Http,
            io::Error::other("HTTP/1 connection could not be initialized"),
        )
    })?;
    Ok(HttpsConnection::new(
        sender,
        tokio::spawn(connection),
        io_control,
        control,
        #[cfg(test)]
        abort_observer,
        #[cfg(test)]
        cleanup_gate,
    ))
}

async fn execute_https_exchange(
    session: &mut HttpsConnection,
    request_owner: SecretBuffer,
    settings: &HttpsConnectionSettings,
    max_response_bytes: usize,
    control: &ExchangeControl,
    #[cfg(test)] response_buffer_observer: Option<ResponseBufferObserver>,
) -> Result<TransportResponse, TransportError> {
    let request = build_http_request(&settings.route, request_owner).map_err(|_| {
        safe_error(
            RequestDeliveryState::NotSent,
            TransportCauseCategory::Other,
            io::Error::other("HTTP request construction failed"),
        )
    })?;
    let mut request = Some(request);
    let response = send_request_when_ready(
        &mut session.sender,
        &mut request,
        control,
        timeout_duration(settings.policy.write()),
        settings.total_deadline,
    )
    .await
    .map_err(|error| io_error(error, control.delivery_state()))?;
    read_response_body(
        response,
        max_response_bytes,
        control,
        #[cfg(test)]
        response_buffer_observer,
    )
    .await
}

struct HttpsConnection {
    sender: http1::SendRequest<RequestBody>,
    task: Option<tokio::task::JoinHandle<Result<(), hyper::Error>>>,
    io_control: DeadlineIoControl,
    cleanup_control: ExchangeControl,
    #[cfg(test)]
    abort_observer: Option<mpsc::SyncSender<()>>,
    #[cfg(test)]
    cleanup_gate: Option<DriverCleanupGateForTest>,
}

impl HttpsConnection {
    fn new(
        sender: http1::SendRequest<RequestBody>,
        task: tokio::task::JoinHandle<Result<(), hyper::Error>>,
        io_control: DeadlineIoControl,
        cleanup_control: ExchangeControl,
        #[cfg(test)] abort_observer: Option<mpsc::SyncSender<()>>,
        #[cfg(test)] cleanup_gate: Option<DriverCleanupGateForTest>,
    ) -> Self {
        Self {
            sender,
            task: Some(task),
            io_control,
            cleanup_control,
            #[cfg(test)]
            abort_observer,
            #[cfg(test)]
            cleanup_gate,
        }
    }

    fn set_cleanup_context(
        &mut self,
        control: ExchangeControl,
        #[cfg(test)] abort_observer: Option<mpsc::SyncSender<()>>,
        #[cfg(test)] cleanup_gate: Option<DriverCleanupGateForTest>,
    ) {
        self.cleanup_control = control;
        #[cfg(test)]
        {
            self.abort_observer = abort_observer;
            self.cleanup_gate = cleanup_gate;
        }
    }

    fn begin_exchange(
        &mut self,
        read_timeout: Option<Duration>,
        write_timeout: Option<Duration>,
        total_deadline: Option<Instant>,
        control: ExchangeControl,
    ) -> io::Result<()> {
        self.io_control
            .begin_exchange(read_timeout, write_timeout, total_deadline, control)
    }

    fn is_reusable(&self) -> bool {
        !self.io_control.is_invalid()
            && !self.sender.is_closed()
            && self.task.as_ref().is_some_and(|task| !task.is_finished())
    }
}

impl Drop for HttpsConnection {
    fn drop(&mut self) {
        let Some(task) = self.task.take() else {
            return;
        };
        self.io_control.invalidate();
        task.abort();
        #[cfg(test)]
        if let Some(observer) = self.abort_observer.take() {
            let _ = observer.send(());
        }
        let control = self.cleanup_control.clone();
        #[cfg(test)]
        let cleanup_gate = self.cleanup_gate.take();
        let cleanup = tokio::runtime::Handle::try_current().ok().map(|runtime| {
            runtime.spawn(async move {
                #[cfg(test)]
                let joined = if let Some(gate) = cleanup_gate {
                    let _ = gate.events.send(DriverCleanupEventForTest::Started);
                    let _ = gate.release.await;
                    let joined = task.await;
                    let _ = gate.events.send(DriverCleanupEventForTest::Acknowledged);
                    joined
                } else {
                    task.await
                };
                #[cfg(not(test))]
                let joined = task.await;
                let _ = joined;
            })
        });
        if let Some(cleanup) = cleanup {
            control.register_cleanup(cleanup);
        }
    }
}

fn lock_https_connection(
    connection: &StdMutex<Option<HttpsConnection>>,
) -> MutexGuard<'_, Option<HttpsConnection>> {
    connection
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
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
    response: hyper::Response<Incoming>,
    max_response_bytes: usize,
    control: &ExchangeControl,
    #[cfg(test)] observer: Option<ResponseBufferObserver>,
) -> Result<TransportResponse, TransportError> {
    let declared_length = validate_response_headers(&response, max_response_bytes, control)?;
    let mut body = response.into_body();
    #[cfg(test)]
    let response_buffer = ResponseBuffer::with_capacity(declared_length, observer);
    #[cfg(not(test))]
    let response_buffer = ResponseBuffer::with_capacity(declared_length);
    let mut response = response_buffer.map_err(|error| {
        safe_error(
            control.delivery_state(),
            TransportCauseCategory::Other,
            error,
        )
    })?;
    loop {
        let frame = poll_fn(|cx| Pin::new(&mut body).poll_frame(cx)).await;
        let Some(frame) = frame else {
            if response.len() != declared_length {
                return Err(safe_error(
                    control.delivery_state(),
                    TransportCauseCategory::Http,
                    io::Error::other("HTTP response length did not match Content-Length"),
                ));
            }
            return Ok(TransportResponse::new(response.into_bytes()));
        };
        let frame = frame.map_err(|error| response_body_error(error, control.delivery_state()))?;
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

fn validate_response_headers(
    response: &hyper::Response<Incoming>,
    max_response_bytes: usize,
    control: &ExchangeControl,
) -> Result<usize, TransportError> {
    // Hyper owns HTTP syntax parsing and produces the framed Incoming body.
    // KMIPKit owns the narrower response profile and the exact TTLV byte cap.
    if response.status() != hyper::StatusCode::OK {
        return Err(http_response_error(
            control,
            "HTTP response status was not 200",
        ));
    }

    validate_response_content_type(response.headers(), control)?;
    let declared_length = parse_response_content_length(response.headers(), control)?;

    if response.headers().contains_key(TRANSFER_ENCODING) {
        return Err(http_response_error(
            control,
            "HTTP Transfer-Encoding is unsupported",
        ));
    }
    if response.headers().contains_key(CONTENT_ENCODING) {
        return Err(http_response_error(
            control,
            "HTTP Content-Encoding is unsupported",
        ));
    }
    if declared_length > max_response_bytes {
        return Err(safe_error(
            control.delivery_state(),
            TransportCauseCategory::Other,
            io::Error::other("HTTP response exceeds configured limit"),
        ));
    }
    Ok(declared_length)
}

fn validate_response_content_type(
    headers: &hyper::http::HeaderMap,
    control: &ExchangeControl,
) -> Result<(), TransportError> {
    let mut content_types = headers.get_all(CONTENT_TYPE).iter();
    let Some(content_type) = content_types.next() else {
        return Err(http_response_error(
            control,
            "HTTP response Content-Type was missing",
        ));
    };
    if content_types.next().is_some() {
        return Err(http_response_error(
            control,
            "HTTP response Content-Type was duplicated",
        ));
    }
    let media_type = content_type
        .to_str()
        .map_err(|_| http_response_error(control, "HTTP response Content-Type was invalid"))?
        .split(';')
        .next()
        .map(str::trim);
    if !media_type.is_some_and(|value| value.eq_ignore_ascii_case("application/octet-stream")) {
        return Err(http_response_error(
            control,
            "HTTP response media type was not octet-stream",
        ));
    }
    Ok(())
}

fn parse_response_content_length(
    headers: &hyper::http::HeaderMap,
    control: &ExchangeControl,
) -> Result<usize, TransportError> {
    let mut content_lengths = headers.get_all(CONTENT_LENGTH).iter();
    let Some(content_length) = content_lengths.next() else {
        return Err(http_response_error(
            control,
            "HTTP response Content-Length was missing",
        ));
    };
    if content_lengths.next().is_some() {
        return Err(http_response_error(
            control,
            "HTTP response Content-Length was duplicated",
        ));
    }
    let length_bytes = content_length.as_bytes();
    if length_bytes.is_empty() || !length_bytes.iter().all(u8::is_ascii_digit) {
        return Err(http_response_error(
            control,
            "HTTP response Content-Length was invalid",
        ));
    }
    std::str::from_utf8(length_bytes)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .ok_or_else(|| http_response_error(control, "HTTP response Content-Length was invalid"))
}

fn http_response_error(control: &ExchangeControl, message: &'static str) -> TransportError {
    safe_error(
        control.delivery_state(),
        TransportCauseCategory::Http,
        io::Error::other(message),
    )
}

struct ResponseBuffer(Vec<u8>, #[cfg(test)] Option<ResponseBufferObserver>);

impl ResponseBuffer {
    #[cfg(not(test))]
    fn with_capacity(capacity: usize) -> io::Result<Self> {
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(capacity)
            .map_err(|_| io::Error::other("HTTP response allocation failed"))?;
        Ok(Self(bytes))
    }

    #[cfg(test)]
    fn with_capacity(
        capacity: usize,
        observer: Option<ResponseBufferObserver>,
    ) -> io::Result<Self> {
        let mut bytes = Vec::new();
        if capacity > 0 {
            if let Some(observer) = &observer {
                observer.record_allocation_attempt(capacity);
            }
            bytes
                .try_reserve_exact(capacity)
                .map_err(|_| io::Error::other("HTTP response allocation failed"))?;
        }
        Ok(Self(bytes, observer))
    }

    fn len(&self) -> usize {
        self.0.len()
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
        #[cfg(test)]
        if let Some(observer) = &self.1 {
            observer.record_allocation_attempt(capacity);
        }
        let mut replacement = Vec::new();
        replacement
            .try_reserve_exact(capacity)
            .map_err(|_| io::Error::other("HTTP response allocation failed"))?;
        replacement.extend_from_slice(&self.0);
        replacement.extend_from_slice(bytes);
        self.0.as_mut_slice().zeroize();
        #[cfg(test)]
        if let Some(observer) = &self.1 {
            observer.record_replaced_allocation(self.0.as_slice());
        }
        self.0 = replacement;
        Ok(())
    }

    fn into_bytes(mut self) -> Vec<u8> {
        std::mem::take(&mut self.0)
    }
}

impl Drop for ResponseBuffer {
    fn drop(&mut self) {
        #[cfg(test)]
        let initialized_len = self.0.len();
        self.0.as_mut_slice().zeroize();
        #[cfg(test)]
        if let Some(observer) = &self.1 {
            observer.record_drop(initialized_len, self.0.as_slice());
        }
    }
}

#[cfg(test)]
#[derive(Clone)]
pub(crate) struct ResponseBufferObserver {
    state: Arc<ResponseBufferObserverState>,
}

#[cfg(test)]
struct ResponseBufferObserverState {
    allocation_attempts: std::sync::atomic::AtomicUsize,
    requested_capacity: std::sync::atomic::AtomicUsize,
    initialized_len: std::sync::atomic::AtomicUsize,
    initialized_range_was_zero: std::sync::atomic::AtomicBool,
    replaced_allocations_were_zero: std::sync::atomic::AtomicBool,
}

#[cfg(test)]
impl ResponseBufferObserver {
    #[allow(dead_code)] // Integration tests construct this observer per adapter.
    pub(crate) fn new() -> Self {
        Self {
            state: Arc::new(ResponseBufferObserverState {
                allocation_attempts: std::sync::atomic::AtomicUsize::new(0),
                requested_capacity: std::sync::atomic::AtomicUsize::new(0),
                initialized_len: std::sync::atomic::AtomicUsize::new(0),
                initialized_range_was_zero: std::sync::atomic::AtomicBool::new(false),
                replaced_allocations_were_zero: std::sync::atomic::AtomicBool::new(true),
            }),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn allocation_attempts(&self) -> usize {
        self.state
            .allocation_attempts
            .load(std::sync::atomic::Ordering::Acquire)
    }

    #[allow(dead_code)]
    pub(crate) fn requested_capacity(&self) -> usize {
        self.state
            .requested_capacity
            .load(std::sync::atomic::Ordering::Acquire)
    }

    #[allow(dead_code)]
    pub(crate) fn initialized_len(&self) -> usize {
        self.state
            .initialized_len
            .load(std::sync::atomic::Ordering::Acquire)
    }

    #[allow(dead_code)]
    pub(crate) fn initialized_range_was_zero(&self) -> bool {
        self.state
            .initialized_range_was_zero
            .load(std::sync::atomic::Ordering::Acquire)
    }

    #[allow(dead_code)]
    pub(crate) fn replaced_allocations_were_zero(&self) -> bool {
        self.state
            .replaced_allocations_were_zero
            .load(std::sync::atomic::Ordering::Acquire)
    }

    fn record_allocation_attempt(&self, capacity: usize) {
        self.state
            .allocation_attempts
            .fetch_add(1, std::sync::atomic::Ordering::AcqRel);
        self.state
            .requested_capacity
            .fetch_max(capacity, std::sync::atomic::Ordering::AcqRel);
    }

    fn record_replaced_allocation(&self, bytes: &[u8]) {
        self.state.replaced_allocations_were_zero.fetch_and(
            bytes.iter().all(|byte| *byte == 0),
            std::sync::atomic::Ordering::AcqRel,
        );
    }

    fn record_drop(&self, initialized_len: usize, bytes: &[u8]) {
        self.state
            .initialized_len
            .store(initialized_len, std::sync::atomic::Ordering::Release);
        self.state.initialized_range_was_zero.store(
            bytes.iter().all(|byte| *byte == 0),
            std::sync::atomic::Ordering::Release,
        );
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
    let cause = error
        .get_ref()
        .and_then(|source| source.downcast_ref::<hyper::Error>())
        .map_or_else(|| io_error_cause(error.kind()), hyper_error_cause);
    safe_error(state, cause, error)
}

fn response_body_error(error: hyper::Error, state: RequestDeliveryState) -> TransportError {
    let cause = hyper_error_cause(&error);
    safe_error(state, cause, io::Error::other(error))
}

fn hyper_error_cause(error: &hyper::Error) -> TransportCauseCategory {
    if error_chain_contains_tls_failure(Some(error)) {
        TransportCauseCategory::Tls
    } else if error.is_parse()
        || error.is_incomplete_message()
        || hyper_error_has_io_kind(error, io::ErrorKind::UnexpectedEof)
    {
        TransportCauseCategory::Http
    } else if error.is_timeout() || hyper_error_has_io_kind(error, io::ErrorKind::TimedOut) {
        TransportCauseCategory::Timeout
    } else {
        TransportCauseCategory::Io
    }
}

fn error_chain_contains_tls_failure(
    mut source: Option<&(dyn std::error::Error + 'static)>,
) -> bool {
    while let Some(error) = source {
        if error.downcast_ref::<rustls::Error>().is_some()
            || error
                .downcast_ref::<io::Error>()
                .is_some_and(crate::timeout::is_tls_failure)
        {
            return true;
        }
        source = error.source();
    }
    false
}

fn hyper_error_has_io_kind(error: &hyper::Error, expected: io::ErrorKind) -> bool {
    let mut source = std::error::Error::source(error);
    while let Some(current) = source {
        if current
            .downcast_ref::<io::Error>()
            .is_some_and(|error| error.kind() == expected)
        {
            return true;
        }
        source = std::error::Error::source(current);
    }
    false
}

fn io_error_cause(kind: io::ErrorKind) -> TransportCauseCategory {
    match kind {
        io::ErrorKind::TimedOut => TransportCauseCategory::Timeout,
        io::ErrorKind::InvalidInput => TransportCauseCategory::Other,
        _ => TransportCauseCategory::Io,
    }
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
