//! An authoritative DNS fixture bound only to IPv4 loopback over UDP and TCP.

use std::collections::BTreeMap;
use std::io::{self, Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

const READ_POLL_INTERVAL: Duration = Duration::from_millis(25);
const MAX_QUERY_BYTES: usize = 512;
const MAX_SOCKET_PAIR_BIND_ATTEMPTS: usize = 32;
// Resolver tests need answers larger than the production 16-candidate cap.
const MAX_RECORDS_PER_NAME: usize = 32;

/// A local DNS responder for explicitly configured, loopback-only names.
pub struct LocalDnsFixture {
    local_addr: SocketAddr,
    state: Arc<FixtureState>,
    udp_thread: Option<JoinHandle<()>>,
    tcp_thread: Option<JoinHandle<()>>,
}

/// DNS question type observed by the loopback fixture.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum DnsQueryType {
    /// IPv4 address query.
    A,
    /// IPv6 address query.
    Aaaa,
}

struct FixtureState {
    stop: AtomicBool,
    metrics: Mutex<FixtureMetrics>,
    response_released: (Mutex<bool>, Condvar),
    active_responses: AtomicUsize,
    peak_active_responses: AtomicUsize,
    next_tcp_connection: AtomicUsize,
    tcp_metrics: Mutex<TcpMetrics>,
}

#[derive(Default)]
struct FixtureMetrics {
    query_counts: BTreeMap<(String, DnsQueryType), usize>,
    nxdomain_names: std::collections::BTreeSet<String>,
    dropped_questions: BTreeMap<(String, DnsQueryType), usize>,
}

#[derive(Default)]
struct TcpMetrics {
    active_by_connection: BTreeMap<usize, usize>,
    peak_by_connection: BTreeMap<usize, usize>,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum TcpWorkerKind {
    Connection,
    Response,
}

type TcpConnectionJob = Box<dyn FnOnce() + Send + 'static>;
type TcpWorkerSpawner =
    Arc<dyn Fn(TcpWorkerKind, TcpConnectionJob) -> io::Result<()> + Send + Sync + 'static>;

impl LocalDnsFixture {
    /// Binds an ephemeral UDP/TCP port pair and serves the supplied loopback addresses.
    ///
    /// Names are matched case-insensitively. Any address that is not loopback,
    /// or any name with more than 32 addresses, is rejected before either
    /// socket is bound.
    ///
    /// # Errors
    ///
    /// Returns an error if a record is invalid or the loopback socket or its
    /// service thread cannot be created.
    pub fn bind(records: BTreeMap<String, Vec<IpAddr>>) -> io::Result<Self> {
        let records = validate_records(records)?;
        let (socket, tcp_listener) = bind_local_dns_sockets()?;
        socket.set_read_timeout(Some(READ_POLL_INTERVAL))?;
        let local_addr = socket.local_addr()?;
        tcp_listener.set_nonblocking(true)?;
        let state = Arc::new(FixtureState {
            stop: AtomicBool::new(false),
            metrics: Mutex::new(FixtureMetrics::default()),
            response_released: (Mutex::new(true), Condvar::new()),
            active_responses: AtomicUsize::new(0),
            peak_active_responses: AtomicUsize::new(0),
            next_tcp_connection: AtomicUsize::new(1),
            tcp_metrics: Mutex::new(TcpMetrics::default()),
        });
        let tcp_records = records.clone();
        let udp_state = Arc::clone(&state);
        let udp_worker = thread::Builder::new()
            .name("kmipkit-local-dns-fixture".to_owned())
            .spawn(move || serve(&socket, &records, &udp_state))?;
        let tcp_state = Arc::clone(&state);
        let tcp_worker = match thread::Builder::new()
            .name("kmipkit-local-dns-tcp-fixture".to_owned())
            .spawn(move || serve_tcp(&tcp_listener, &tcp_records, &tcp_state))
        {
            Ok(worker) => worker,
            Err(error) => {
                state.stop.store(true, Ordering::Release);
                if let Ok(mut released) = state.response_released.0.lock() {
                    *released = true;
                    state.response_released.1.notify_all();
                }
                let _join_result = udp_worker.join();
                return Err(error);
            }
        };

        Ok(Self {
            local_addr,
            state,
            udp_thread: Some(udp_worker),
            tcp_thread: Some(tcp_worker),
        })
    }

    /// Returns the fixture's ephemeral loopback address, shared by UDP and TCP.
    #[must_use]
    pub const fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// Returns how many questions for `name` and `query_type` reached the fixture.
    #[must_use]
    pub fn query_count(&self, name: &str, query_type: DnsQueryType) -> usize {
        let canonical = canonical_name(name);
        self.state
            .metrics
            .lock()
            .map(|metrics| {
                metrics
                    .query_counts
                    .get(&(canonical, query_type))
                    .copied()
                    .unwrap_or_default()
            })
            .unwrap_or_default()
    }

    /// Drops the next `count` matching DNS questions without sending a response.
    pub fn drop_next_questions(&self, name: &str, query_type: DnsQueryType, count: usize) {
        if let Ok(mut metrics) = self.state.metrics.lock() {
            metrics
                .dropped_questions
                .insert((canonical_name(name), query_type), count);
        }
    }

    /// Returns NXDOMAIN for the matching name instead of an empty NOERROR response.
    pub fn set_nxdomain(&self, name: &str) {
        if let Ok(mut metrics) = self.state.metrics.lock() {
            metrics.nxdomain_names.insert(canonical_name(name));
        }
    }

    /// Holds all generated responses until [`Self::release_responses`] is called.
    pub fn hold_responses(&self) {
        if let Ok(mut released) = self.state.response_released.0.lock() {
            *released = false;
        }
    }

    /// Releases responses held by [`Self::hold_responses`].
    pub fn release_responses(&self) {
        if let Ok(mut released) = self.state.response_released.0.lock() {
            *released = true;
            self.state.response_released.1.notify_all();
        }
    }

    /// Returns the greatest number of generated responses concurrently in flight.
    #[must_use]
    pub fn peak_active_responses(&self) -> usize {
        self.state.peak_active_responses.load(Ordering::Acquire)
    }

    /// Returns the highest in-flight request count observed on any TCP connection.
    #[must_use]
    pub fn peak_active_tcp_requests_per_connection(&self) -> usize {
        self.state
            .tcp_metrics
            .lock()
            .map(|metrics| {
                metrics
                    .peak_by_connection
                    .values()
                    .copied()
                    .max()
                    .unwrap_or_default()
            })
            .unwrap_or_default()
    }

    /// Returns the current in-flight request count across this fixture's TCP connections.
    #[must_use]
    pub fn active_tcp_requests(&self) -> usize {
        self.state
            .tcp_metrics
            .lock()
            .map(|metrics| metrics.active_by_connection.values().sum())
            .unwrap_or_default()
    }
}

fn bind_local_dns_sockets() -> io::Result<(UdpSocket, TcpListener)> {
    bind_local_dns_sockets_with(TcpListener::bind)
}

fn bind_local_dns_sockets_with(
    mut bind_tcp: impl FnMut(SocketAddr) -> io::Result<TcpListener>,
) -> io::Result<(UdpSocket, TcpListener)> {
    let mut attempts_remaining = MAX_SOCKET_PAIR_BIND_ATTEMPTS;
    loop {
        let socket = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0))?;
        let local_addr = socket.local_addr()?;
        match bind_tcp(local_addr) {
            Ok(tcp_listener) => return Ok((socket, tcp_listener)),
            Err(error) if is_retryable_socket_pair_bind_error(error.kind()) => {
                attempts_remaining -= 1;
                if attempts_remaining == 0 {
                    return Err(error);
                }
            }
            Err(error) => return Err(error),
        }
    }
}

fn is_retryable_socket_pair_bind_error(kind: io::ErrorKind) -> bool {
    matches!(
        kind,
        io::ErrorKind::AddrInUse
            | io::ErrorKind::AddrNotAvailable
            | io::ErrorKind::PermissionDenied
    )
}

impl Drop for LocalDnsFixture {
    fn drop(&mut self) {
        self.state.stop.store(true, Ordering::Release);
        self.release_responses();
        if let Some(worker) = self.udp_thread.take() {
            let _join_result = worker.join();
        }
        if let Some(worker) = self.tcp_thread.take() {
            let _join_result = worker.join();
        }
    }
}

fn validate_records(
    records: BTreeMap<String, Vec<IpAddr>>,
) -> io::Result<BTreeMap<String, Vec<IpAddr>>> {
    let mut normalized = BTreeMap::new();
    for (name, addresses) in records {
        let canonical_name = name.trim_end_matches('.').to_ascii_lowercase();
        if !valid_name(&canonical_name)
            || addresses.is_empty()
            || addresses.len() > MAX_RECORDS_PER_NAME
            || addresses.iter().any(|address| !address.is_loopback())
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "local DNS fixture accepts only valid names and loopback addresses",
            ));
        }
        if normalized.insert(canonical_name, addresses).is_some() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "local DNS fixture names must be unique ignoring case",
            ));
        }
    }
    Ok(normalized)
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 253
        && name.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && label
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
                && label.as_bytes()[0].is_ascii_alphanumeric()
                && label.as_bytes()[label.len() - 1].is_ascii_alphanumeric()
        })
}

fn serve(socket: &UdpSocket, records: &BTreeMap<String, Vec<IpAddr>>, state: &Arc<FixtureState>) {
    let mut query = [0_u8; MAX_QUERY_BYTES];
    while !state.stop.load(Ordering::Acquire) {
        let Ok((length, peer)) = socket.recv_from(&mut query) else {
            continue;
        };
        let Some((name, query_type)) = parse_question(&query[..length]) else {
            continue;
        };
        if record_question(state, &name, query_type) {
            continue;
        }
        let is_nxdomain = is_nxdomain(state, &name);
        let Some(response) = build_response(&query[..length], records, is_nxdomain) else {
            continue;
        };
        let Ok(response_socket) = socket.try_clone() else {
            continue;
        };
        let peer_response_state = Arc::clone(state);
        let active = state.active_responses.fetch_add(1, Ordering::AcqRel) + 1;
        state
            .peak_active_responses
            .fetch_max(active, Ordering::AcqRel);
        let spawn_result = thread::Builder::new()
            .name("kmipkit-local-dns-response".to_owned())
            .spawn(move || {
                let (released, wake) = &peer_response_state.response_released;
                let Ok(mut is_released) = released.lock() else {
                    peer_response_state
                        .active_responses
                        .fetch_sub(1, Ordering::AcqRel);
                    return;
                };
                while !*is_released && !peer_response_state.stop.load(Ordering::Acquire) {
                    let Ok(next) = wake.wait(is_released) else {
                        peer_response_state
                            .active_responses
                            .fetch_sub(1, Ordering::AcqRel);
                        return;
                    };
                    is_released = next;
                }
                drop(is_released);
                if !peer_response_state.stop.load(Ordering::Acquire) {
                    let _ = response_socket.send_to(&response, peer);
                }
                peer_response_state
                    .active_responses
                    .fetch_sub(1, Ordering::AcqRel);
            });
        if spawn_result.is_err() {
            state.active_responses.fetch_sub(1, Ordering::AcqRel);
        }
    }
}

fn serve_tcp(
    listener: &TcpListener,
    records: &BTreeMap<String, Vec<IpAddr>>,
    state: &Arc<FixtureState>,
) {
    let spawn_worker: TcpWorkerSpawner = Arc::new(spawn_tcp_worker);
    serve_tcp_with_spawner(listener, records, state, &spawn_worker);
}

fn serve_tcp_with_spawner(
    listener: &TcpListener,
    records: &BTreeMap<String, Vec<IpAddr>>,
    state: &Arc<FixtureState>,
    spawn_worker: &TcpWorkerSpawner,
) {
    serve_tcp_with_spawner_and_cloner(listener, records, state, spawn_worker, TcpStream::try_clone);
}

fn serve_tcp_with_spawner_and_cloner<F>(
    listener: &TcpListener,
    records: &BTreeMap<String, Vec<IpAddr>>,
    state: &Arc<FixtureState>,
    spawn_worker: &TcpWorkerSpawner,
    clone_stream: F,
) where
    F: Fn(&TcpStream) -> io::Result<TcpStream> + Send + Sync + 'static,
{
    let clone_stream = Arc::new(clone_stream);
    while !state.stop.load(Ordering::Acquire) {
        match listener.accept() {
            Ok((stream, _)) => {
                let connection_id = state.next_tcp_connection.fetch_add(1, Ordering::AcqRel);
                let Ok(worker_stream) = clone_stream(&stream) else {
                    serve_tcp_connection(
                        stream,
                        records,
                        state,
                        connection_id,
                        spawn_worker,
                        clone_stream.as_ref(),
                    );
                    continue;
                };
                let connection_state = Arc::clone(state);
                let connection_records = records.clone();
                let connection_spawner = Arc::clone(spawn_worker);
                let connection_cloner = Arc::clone(&clone_stream);
                let job: TcpConnectionJob = Box::new(move || {
                    serve_tcp_connection(
                        worker_stream,
                        &connection_records,
                        &connection_state,
                        connection_id,
                        &connection_spawner,
                        connection_cloner.as_ref(),
                    );
                });
                if spawn_worker(TcpWorkerKind::Connection, job).is_err() {
                    // Keep the accepted socket alive if resource pressure blocks worker creation.
                    // This rare fallback can delay new accepts until the connection closes.
                    serve_tcp_connection(
                        stream,
                        records,
                        state,
                        connection_id,
                        spawn_worker,
                        clone_stream.as_ref(),
                    );
                }
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(READ_POLL_INTERVAL);
            }
            Err(_) => break,
        }
    }
}

fn serve_tcp_connection<F>(
    mut reader: TcpStream,
    records: &BTreeMap<String, Vec<IpAddr>>,
    state: &Arc<FixtureState>,
    connection_id: usize,
    spawn_worker: &TcpWorkerSpawner,
    clone_stream: &F,
) where
    F: Fn(&TcpStream) -> io::Result<TcpStream>,
{
    let writer = clone_stream(&reader)
        .ok()
        .map(|writer| Arc::new(Mutex::new(writer)));
    while !state.stop.load(Ordering::Acquire) {
        let mut length = [0_u8; 2];
        if reader.read_exact(&mut length).is_err() {
            break;
        }
        let message_length = usize::from(u16::from_be_bytes(length));
        if !(12..=MAX_QUERY_BYTES).contains(&message_length) {
            break;
        }
        let mut query = vec![0_u8; message_length];
        if reader.read_exact(&mut query).is_err() {
            break;
        }
        let Some((name, query_type)) = parse_question(&query) else {
            continue;
        };
        if record_question(state, &name, query_type) {
            continue;
        }
        let response = build_response(&query, records, is_nxdomain(state, &name));
        let Some(response) = response else {
            continue;
        };
        begin_tcp_request(state, connection_id);
        let response = Arc::new(response);
        let Some(writer) = writer.as_ref() else {
            send_tcp_response_inline(state, &mut reader, connection_id, &response);
            continue;
        };
        let response_state = Arc::clone(state);
        let response_writer = Arc::clone(writer);
        let worker_response = Arc::clone(&response);
        let response_spawner = Arc::clone(spawn_worker);
        let response_job: TcpConnectionJob = Box::new(move || {
            send_tcp_response(
                &response_state,
                &response_writer,
                connection_id,
                &worker_response,
            );
        });
        if response_spawner(TcpWorkerKind::Response, response_job).is_err() {
            send_tcp_response(state, writer, connection_id, &response);
        }
    }
}

fn send_tcp_response(
    state: &FixtureState,
    writer: &Mutex<TcpStream>,
    connection_id: usize,
    response: &[u8],
) {
    if wait_for_response_release(state)
        && let Ok(mut stream) = writer.lock()
    {
        write_tcp_response_frame(&mut stream, response);
    }
    finish_tcp_request(state, connection_id);
}

fn send_tcp_response_inline(
    state: &FixtureState,
    writer: &mut TcpStream,
    connection_id: usize,
    response: &[u8],
) {
    if wait_for_response_release(state) {
        write_tcp_response_frame(writer, response);
    }
    finish_tcp_request(state, connection_id);
}

fn write_tcp_response_frame(writer: &mut TcpStream, response: &[u8]) {
    if let Ok(response_length) = u16::try_from(response.len()) {
        let _ = writer.write_all(&response_length.to_be_bytes());
        let _ = writer.write_all(response);
    }
}

fn spawn_tcp_worker(kind: TcpWorkerKind, job: TcpConnectionJob) -> io::Result<()> {
    let name = match kind {
        TcpWorkerKind::Connection => "kmipkit-local-dns-tcp-connection",
        TcpWorkerKind::Response => "kmipkit-local-dns-tcp-response",
    };
    thread::Builder::new()
        .name(name.to_owned())
        .spawn(job)
        .map(drop)
}

fn record_question(state: &FixtureState, name: &str, query_type: DnsQueryType) -> bool {
    let Ok(mut metrics) = state.metrics.lock() else {
        return true;
    };
    let key = (name.to_owned(), query_type);
    *metrics.query_counts.entry(key.clone()).or_default() += 1;
    match metrics.dropped_questions.get_mut(&key) {
        Some(remaining) if *remaining > 0 => {
            *remaining -= 1;
            true
        }
        _ => false,
    }
}

fn is_nxdomain(state: &FixtureState, name: &str) -> bool {
    state
        .metrics
        .lock()
        .is_ok_and(|metrics| metrics.nxdomain_names.contains(name))
}

fn begin_tcp_request(state: &FixtureState, connection_id: usize) {
    let Ok(mut metrics) = state.tcp_metrics.lock() else {
        return;
    };
    let active = metrics
        .active_by_connection
        .entry(connection_id)
        .or_default();
    *active += 1;
    let active_count = *active;
    metrics
        .peak_by_connection
        .entry(connection_id)
        .and_modify(|peak| *peak = (*peak).max(active_count))
        .or_insert(active_count);
}

fn finish_tcp_request(state: &FixtureState, connection_id: usize) {
    if let Ok(mut metrics) = state.tcp_metrics.lock()
        && let Some(active) = metrics.active_by_connection.get_mut(&connection_id)
    {
        *active = active.saturating_sub(1);
    }
}

fn wait_for_response_release(state: &FixtureState) -> bool {
    let (released, wake) = &state.response_released;
    let Ok(mut is_released) = released.lock() else {
        return false;
    };
    while !*is_released && !state.stop.load(Ordering::Acquire) {
        let Ok(next) = wake.wait(is_released) else {
            return false;
        };
        is_released = next;
    }
    *is_released && !state.stop.load(Ordering::Acquire)
}

fn build_response(
    query: &[u8],
    records: &BTreeMap<String, Vec<IpAddr>>,
    nxdomain: bool,
) -> Option<Vec<u8>> {
    if query.len() < 12 || query.len() > MAX_QUERY_BYTES {
        return None;
    }
    let query_flags = read_u16(query, 2)?;
    if query_flags & 0x8000 != 0 || read_u16(query, 4)? != 1 {
        return None;
    }

    let (name, question_end) = parse_question_name(query)?;
    let record_type = read_u16(query, question_end.checked_sub(4)?)?;
    let record_class = read_u16(query, question_end.checked_sub(2)?)?;
    let mut response = Vec::with_capacity(query.len() + 128);
    response.extend_from_slice(&query[..2]);
    // QR + AA, preserve RD, and mark the response non-truncated. NXDOMAIN
    // uses RCODE 3; the ordinary fixture response is authoritative NOERROR.
    let response_flags = 0x8400_u16 | (query_flags & 0x0100) | if nxdomain { 3 } else { 0 };
    response.extend_from_slice(&response_flags.to_be_bytes());
    response.extend_from_slice(&1_u16.to_be_bytes());
    let answer_start = response.len();
    response.extend_from_slice(&0_u16.to_be_bytes());
    response.extend_from_slice(&[0; 4]);
    response.extend_from_slice(&query[12..question_end]);

    let values = records.get(&name);
    let matching: Vec<IpAddr> = if nxdomain {
        Vec::new()
    } else {
        match (record_class, record_type, values) {
            (1, 1, Some(values)) => values
                .iter()
                .filter(|address| address.is_ipv4())
                .copied()
                .collect(),
            (1, 28, Some(values)) => values
                .iter()
                .filter(|address| address.is_ipv6())
                .copied()
                .collect(),
            _ => Vec::new(),
        }
    };

    let mut answer_count = 0_u16;
    for address in matching {
        response.extend_from_slice(&[0xc0, 0x0c]);
        response.extend_from_slice(&record_type.to_be_bytes());
        response.extend_from_slice(&1_u16.to_be_bytes());
        response.extend_from_slice(&60_u32.to_be_bytes());
        match address {
            IpAddr::V4(address) => {
                response.extend_from_slice(&4_u16.to_be_bytes());
                response.extend_from_slice(&address.octets());
            }
            IpAddr::V6(address) => {
                response.extend_from_slice(&16_u16.to_be_bytes());
                response.extend_from_slice(&address.octets());
            }
        }
        answer_count = answer_count.saturating_add(1);
    }
    response[answer_start..answer_start + 2]
        .copy_from_slice(&if nxdomain { 0_u16 } else { answer_count }.to_be_bytes());
    Some(response)
}

fn parse_question(query: &[u8]) -> Option<(String, DnsQueryType)> {
    let (name, question_end) = parse_question_name(query)?;
    let query_type = read_u16(query, question_end.checked_sub(4)?)?;
    let query_type = match query_type {
        1 => DnsQueryType::A,
        28 => DnsQueryType::Aaaa,
        _ => return None,
    };
    Some((name, query_type))
}

fn canonical_name(name: &str) -> String {
    name.trim_end_matches('.').to_ascii_lowercase()
}

fn parse_question_name(query: &[u8]) -> Option<(String, usize)> {
    let mut offset = 12;
    let mut labels = Vec::new();
    loop {
        let label_length = usize::from(*query.get(offset)?);
        offset = offset.checked_add(1)?;
        if label_length == 0 {
            break;
        }
        if label_length > 63 {
            return None;
        }
        let label = query.get(offset..offset.checked_add(label_length)?)?;
        if !label.is_ascii()
            || label
                .iter()
                .any(|byte| !byte.is_ascii_alphanumeric() && *byte != b'-')
        {
            return None;
        }
        labels.push(std::str::from_utf8(label).ok()?.to_ascii_lowercase());
        offset = offset.checked_add(label_length)?;
        if offset >= query.len() {
            return None;
        }
    }
    let question_end = offset.checked_add(4)?;
    if labels.is_empty() || question_end > query.len() {
        return None;
    }
    Some((labels.join("."), question_end))
}

fn read_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_be_bytes([
        *bytes.get(offset)?,
        *bytes.get(offset.checked_add(1)?)?,
    ]))
}

#[cfg(test)]
#[path = "../tests/unit/dns_tests.rs"]
mod tests;
