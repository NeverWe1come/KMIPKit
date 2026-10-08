use super::*;
use crate::worker::ExchangeControl;
use bytes::Bytes;
use hyper::body::{Body, Frame, SizeHint};
use hyper::client::conn::http1;
use hyper::http::Request;
use hyper::rt::{Read as HyperRead, ReadBuf as HyperReadBuf, Write as HyperWrite};
use std::collections::VecDeque;
use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use std::task::Waker;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadBuf};

#[derive(Clone, Copy)]
enum Mode {
    Ready,
    Pending,
    Error(io::ErrorKind),
}

#[derive(Clone)]
struct ScriptIo {
    state: Arc<Mutex<State>>,
}

struct State {
    read_mode: Mode,
    write_mode: Mode,
    flush_mode: Mode,
    shutdown_mode: Mode,
    reads: VecDeque<Vec<u8>>,
    written: Vec<u8>,
    vectored_calls: usize,
    waker: Option<Waker>,
}

impl ScriptIo {
    fn new(read_mode: Mode, write_mode: Mode, flush_mode: Mode, shutdown_mode: Mode) -> Self {
        Self {
            state: Arc::new(Mutex::new(State {
                read_mode,
                write_mode,
                flush_mode,
                shutdown_mode,
                reads: VecDeque::new(),
                written: Vec::new(),
                vectored_calls: 0,
                waker: None,
            })),
        }
    }

    fn ready() -> Self {
        Self::new(Mode::Ready, Mode::Ready, Mode::Ready, Mode::Ready)
    }

    fn push_read(&self, bytes: &[u8]) {
        self.state
            .lock()
            .expect("script state is available")
            .reads
            .push_back(bytes.to_vec());
    }

    fn written(&self) -> Vec<u8> {
        self.state
            .lock()
            .expect("script state is available")
            .written
            .clone()
    }

    fn vectored_calls(&self) -> usize {
        self.state
            .lock()
            .expect("script state is available")
            .vectored_calls
    }
}

impl AsyncRead for ScriptIo {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buffer: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let mut state = self.state.lock().expect("script state is available");
        if let Some(mut bytes) = state.reads.pop_front() {
            let count = bytes.len().min(buffer.remaining());
            buffer.put_slice(&bytes[..count]);
            if count < bytes.len() {
                bytes.drain(..count);
                state.reads.push_front(bytes);
            }
            return Poll::Ready(Ok(()));
        }
        match state.read_mode {
            Mode::Ready => Poll::Ready(Ok(())),
            Mode::Pending => {
                state.waker = Some(cx.waker().clone());
                Poll::Pending
            }
            Mode::Error(kind) => Poll::Ready(Err(io::Error::new(kind, "private read detail"))),
        }
    }
}

impl AsyncWrite for ScriptIo {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bytes: &[u8],
    ) -> Poll<io::Result<usize>> {
        let mut state = self.state.lock().expect("script state is available");
        match state.write_mode {
            Mode::Ready => {
                state.written.extend_from_slice(bytes);
                Poll::Ready(Ok(bytes.len()))
            }
            Mode::Pending => {
                state.waker = Some(cx.waker().clone());
                Poll::Pending
            }
            Mode::Error(kind) => Poll::Ready(Err(io::Error::new(kind, "private write detail"))),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let mut state = self.state.lock().expect("script state is available");
        match state.flush_mode {
            Mode::Ready => Poll::Ready(Ok(())),
            Mode::Pending => {
                state.waker = Some(cx.waker().clone());
                Poll::Pending
            }
            Mode::Error(kind) => Poll::Ready(Err(io::Error::new(kind, "private flush detail"))),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let mut state = self.state.lock().expect("script state is available");
        match state.shutdown_mode {
            Mode::Ready => Poll::Ready(Ok(())),
            Mode::Pending => {
                state.waker = Some(cx.waker().clone());
                Poll::Pending
            }
            Mode::Error(kind) => Poll::Ready(Err(io::Error::new(kind, "private shutdown detail"))),
        }
    }

    fn is_write_vectored(&self) -> bool {
        true
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buffers: &[IoSlice<'_>],
    ) -> Poll<io::Result<usize>> {
        self.state
            .lock()
            .expect("script state is available")
            .vectored_calls += 1;
        let bytes = buffers
            .iter()
            .flat_map(|buffer| buffer.iter().copied())
            .collect::<Vec<_>>();
        self.poll_write(cx, &bytes)
    }
}

#[derive(Debug)]
struct EmptyBody;

impl Body for EmptyBody {
    type Data = Bytes;
    type Error = Infallible;

    fn poll_frame(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        Poll::Ready(None)
    }

    fn is_end_stream(&self) -> bool {
        true
    }

    fn size_hint(&self) -> SizeHint {
        SizeHint::with_exact(0)
    }
}

fn active_control() -> ExchangeControl {
    let control = ExchangeControl::new();
    assert!(control.begin());
    control
}

fn dispatched_control() -> ExchangeControl {
    let control = active_control();
    assert!(control.commit_dispatch());
    control
}

async fn read_once<R: HyperRead + Unpin>(reader: &mut R, target: &mut [u8]) -> io::Result<usize> {
    let mut buffer = HyperReadBuf::new(target);
    poll_fn(|cx| Pin::new(&mut *reader).poll_read(cx, buffer.unfilled())).await?;
    Ok(buffer.filled().len())
}

async fn write_once<W: HyperWrite + Unpin>(writer: &mut W, bytes: &[u8]) -> io::Result<usize> {
    poll_fn(|cx| Pin::new(&mut *writer).poll_write(cx, bytes)).await
}

async fn write_vectored<W: HyperWrite + Unpin>(
    writer: &mut W,
    buffers: &[IoSlice<'_>],
) -> io::Result<usize> {
    poll_fn(|cx| Pin::new(&mut *writer).poll_write_vectored(cx, buffers)).await
}

async fn flush<W: HyperWrite + Unpin>(writer: &mut W) -> io::Result<()> {
    poll_fn(|cx| Pin::new(&mut *writer).poll_flush(cx)).await
}

async fn shutdown<W: HyperWrite + Unpin>(writer: &mut W) -> io::Result<()> {
    poll_fn(|cx| Pin::new(&mut *writer).poll_shutdown(cx)).await
}

fn request() -> Request<EmptyBody> {
    Request::builder()
        .method("POST")
        .uri("/kmip")
        .body(EmptyBody)
        .expect("unit-test request is valid")
}

async fn read_headers<R: AsyncRead + Unpin>(reader: &mut R) {
    let mut bytes = Vec::new();
    while !bytes.ends_with(b"\r\n\r\n") {
        let mut byte = [0_u8; 1];
        let count = reader
            .read(&mut byte)
            .await
            .expect("the local HTTP peer can read request headers");
        assert_eq!(count, 1, "Hyper emits complete request headers");
        bytes.push(byte[0]);
    }
}

#[tokio::test]
async fn deadline_io_records_positive_and_zero_response_reads() {
    let inner = ScriptIo::ready();
    inner.push_read(b"R");
    let control = dispatched_control();
    let mut io = DeadlineIo::new(inner, None, None, None, control.clone());
    let mut byte = [0_u8; 1];

    assert_eq!(
        read_once(&mut io, &mut byte).await.expect("read succeeds"),
        1
    );
    assert_eq!(byte, *b"R");
    assert_eq!(
        control.delivery_state(),
        RequestDeliveryState::ResponseStarted
    );
    assert_eq!(
        read_once(&mut io, &mut byte).await.expect("EOF succeeds"),
        0
    );
    assert_eq!(
        control.delivery_state(),
        RequestDeliveryState::ResponseStarted
    );

    let empty_control = dispatched_control();
    let mut empty_io = DeadlineIo::new(ScriptIo::ready(), None, None, None, empty_control.clone());
    assert_eq!(
        read_once(&mut empty_io, &mut byte)
            .await
            .expect("EOF succeeds"),
        0
    );
    assert_eq!(
        empty_control.delivery_state(),
        RequestDeliveryState::PossiblySent
    );
}

#[tokio::test]
async fn deadline_io_stops_reads_that_cannot_be_observed() {
    let active = active_control();
    let active_inner = ScriptIo::ready();
    active_inner.push_read(b"R");
    let mut io = DeadlineIo::new(active_inner, None, None, None, active.clone());
    let mut byte = [0_u8; 1];
    let error = read_once(&mut io, &mut byte)
        .await
        .expect_err("a response before dispatch is rejected");
    assert_eq!(error.kind(), io::ErrorKind::Interrupted);

    let finalized = dispatched_control();
    finalized.cancel();
    let mut io = DeadlineIo::new(ScriptIo::ready(), None, None, None, finalized);
    let error = read_once(&mut io, &mut byte)
        .await
        .expect_err("finalized response reads are rejected");
    assert_eq!(error.kind(), io::ErrorKind::Interrupted);

    let mut empty_cursor = HyperReadBuf::new(&mut []);
    poll_fn(|cx| Pin::new(&mut io).poll_read(cx, empty_cursor.unfilled()))
        .await
        .expect("an empty Hyper cursor needs no read");
    assert_eq!(
        write_once(&mut io, b"x")
            .await
            .expect_err("finalized writes are rejected")
            .kind(),
        io::ErrorKind::Interrupted
    );
    assert_eq!(
        flush(&mut io)
            .await
            .expect_err("finalized flushes are rejected")
            .kind(),
        io::ErrorKind::Interrupted
    );
    let data = IoSlice::new(b"x");
    assert_eq!(
        write_vectored(&mut io, &[data])
            .await
            .expect_err("finalized vectored writes are rejected")
            .kind(),
        io::ErrorKind::Interrupted
    );
}

#[tokio::test]
async fn read_and_write_phase_and_total_deadlines_expire() {
    let mut read_io = DeadlineIo::new(
        ScriptIo::new(Mode::Pending, Mode::Ready, Mode::Ready, Mode::Ready),
        Some(Duration::from_millis(20)),
        None,
        Some(Instant::now() + Duration::from_secs(1)),
        dispatched_control(),
    );
    let mut buffer = [0_u8; 1];
    let error = tokio::time::timeout(Duration::from_secs(1), read_once(&mut read_io, &mut buffer))
        .await
        .expect("read deadline wakes the task")
        .expect_err("blocked read expires");
    assert_eq!(error.kind(), io::ErrorKind::TimedOut);

    let mut total_io = DeadlineIo::new(
        ScriptIo::new(Mode::Pending, Mode::Ready, Mode::Ready, Mode::Ready),
        Some(Duration::from_secs(1)),
        None,
        Some(Instant::now() + Duration::from_millis(20)),
        dispatched_control(),
    );
    let error = tokio::time::timeout(
        Duration::from_secs(1),
        read_once(&mut total_io, &mut buffer),
    )
    .await
    .expect("total deadline wakes the task")
    .expect_err("the earlier total deadline expires");
    assert_eq!(error.kind(), io::ErrorKind::TimedOut);

    let mut write_io = DeadlineIo::new(
        ScriptIo::new(Mode::Ready, Mode::Pending, Mode::Ready, Mode::Ready),
        None,
        Some(Duration::from_millis(20)),
        Some(Instant::now() + Duration::from_secs(1)),
        dispatched_control(),
    );
    let error = tokio::time::timeout(Duration::from_secs(1), write_once(&mut write_io, b"x"))
        .await
        .expect("write deadline wakes the task")
        .expect_err("blocked write expires");
    assert_eq!(error.kind(), io::ErrorKind::TimedOut);

    assert_eq!(
        write_once(&mut write_io, b"").await.expect("empty write"),
        0
    );
}

#[tokio::test]
async fn blocked_flush_and_vectored_write_expire() {
    let mut flush_io = DeadlineIo::new(
        ScriptIo::new(Mode::Ready, Mode::Ready, Mode::Pending, Mode::Ready),
        None,
        Some(Duration::from_millis(20)),
        None,
        dispatched_control(),
    );
    let error = tokio::time::timeout(Duration::from_secs(1), flush(&mut flush_io))
        .await
        .expect("flush deadline wakes the task")
        .expect_err("blocked flush expires");
    assert_eq!(error.kind(), io::ErrorKind::TimedOut);

    let mut vectored_io = DeadlineIo::new(
        ScriptIo::new(Mode::Ready, Mode::Pending, Mode::Ready, Mode::Ready),
        None,
        Some(Duration::from_millis(20)),
        None,
        dispatched_control(),
    );
    let part = IoSlice::new(b"x");
    let error = tokio::time::timeout(
        Duration::from_secs(1),
        write_vectored(&mut vectored_io, &[part]),
    )
    .await
    .expect("vectored-write deadline wakes the task")
    .expect_err("blocked vectored write expires");
    assert_eq!(error.kind(), io::ErrorKind::TimedOut);

    let mut shutdown_io = DeadlineIo::new(
        ScriptIo::new(Mode::Ready, Mode::Ready, Mode::Ready, Mode::Pending),
        None,
        Some(Duration::from_millis(20)),
        None,
        dispatched_control(),
    );
    let error = tokio::time::timeout(Duration::from_secs(1), shutdown(&mut shutdown_io))
        .await
        .expect("shutdown deadline wakes the task")
        .expect_err("blocked shutdown expires");
    assert_eq!(error.kind(), io::ErrorKind::TimedOut);
}

#[tokio::test]
async fn phase_deadline_helpers_cover_absent_overflow_and_expired_deadlines() {
    assert_eq!(earlier_deadline(None, None), None);
    let only_phase = Instant::now();
    let only_total = Instant::now();
    assert_eq!(earlier_deadline(Some(only_phase), None), Some(only_phase));
    assert_eq!(earlier_deadline(None, Some(only_total)), Some(only_total));
    let first = Instant::now() + Duration::from_secs(1);
    let second = first + Duration::from_secs(1);
    assert_eq!(earlier_deadline(Some(first), Some(second)), Some(first));

    let mut disabled = PhaseDeadline::new(None);
    disabled.begin_pending();
    disabled.begin_pending();
    assert_eq!(disabled.deadline, None);
    let mut already_started = PhaseDeadline::new(Some(Duration::from_secs(1)));
    already_started.begin_pending();
    let first_deadline = already_started.deadline;
    already_started.begin_pending();
    assert_eq!(already_started.deadline, first_deadline);
    let mut huge = PhaseDeadline::new(Some(Duration::MAX));
    huge.begin_pending();
    assert_eq!(huge.deadline, None);

    let mut expired = PhaseDeadline::new(None);
    expired.deadline = Some(
        Instant::now()
            .checked_sub(Duration::from_secs(1))
            .expect("one second before now is representable"),
    );
    let was_expired = poll_fn(|cx| Poll::Ready(expired.poll_expiration(None, cx))).await;
    assert!(was_expired);
    let result =
        poll_fn(|cx| poll_with_deadline(&mut expired, None, cx, |_| Poll::Ready(Ok(())))).await;
    assert_eq!(
        result
            .expect_err("expired phase deadlines take precedence")
            .kind(),
        io::ErrorKind::TimedOut
    );
}

#[tokio::test]
async fn deadline_crossed_during_io_poll_returns_immediately() {
    let mut phase = PhaseDeadline::new(Some(Duration::from_millis(5)));
    phase.begin_pending();
    let result = poll_fn(|cx| {
        poll_with_deadline::<()>(&mut phase, None, cx, |_| {
            std::thread::sleep(Duration::from_millis(25));
            Poll::Pending
        })
    })
    .await;
    assert_eq!(
        result
            .expect_err("deadline crossing the inner poll expires")
            .kind(),
        io::ErrorKind::TimedOut
    );
}

#[tokio::test]
async fn progress_resets_phase_deadlines_and_vectored_write_uses_the_inner_path() {
    let inner = ScriptIo::ready();
    inner.push_read(b"A");
    inner.push_read(b"B");
    let observed = inner.clone();
    let mut io = DeadlineIo::new(
        inner,
        Some(Duration::from_millis(20)),
        Some(Duration::from_millis(20)),
        None,
        dispatched_control(),
    );
    let mut byte = [0_u8; 1];
    assert_eq!(read_once(&mut io, &mut byte).await.expect("read one"), 1);
    assert_eq!(read_once(&mut io, &mut byte).await.expect("read two"), 1);
    assert_eq!(write_once(&mut io, b"W").await.expect("write"), 1);
    assert!(io.is_write_vectored());
    let first = IoSlice::new(b"V");
    let second = IoSlice::new(b"X");
    assert_eq!(
        write_vectored(&mut io, &[first, second])
            .await
            .expect("vectored write"),
        2
    );
    write_vectored(&mut io, &[])
        .await
        .expect("empty vectored write is a no-op");
    flush(&mut io).await.expect("successful flush is progress");
    shutdown(&mut io)
        .await
        .expect("successful shutdown is forwarded");
    assert_eq!(observed.written(), b"WVX");
    assert_eq!(observed.vectored_calls(), 1);
}

#[tokio::test]
async fn write_flush_vectored_and_shutdown_errors_are_sanitized() {
    let control = dispatched_control();
    let mut write_io = DeadlineIo::new(
        ScriptIo::new(
            Mode::Ready,
            Mode::Error(io::ErrorKind::BrokenPipe),
            Mode::Ready,
            Mode::Ready,
        ),
        None,
        None,
        None,
        control.clone(),
    );
    let error = write_once(&mut write_io, b"secret")
        .await
        .expect_err("the test writer fails");
    assert_eq!(error.kind(), io::ErrorKind::BrokenPipe);
    assert_eq!(error.to_string(), "transport I/O failed");

    let mut flush_io = DeadlineIo::new(
        ScriptIo::new(
            Mode::Ready,
            Mode::Ready,
            Mode::Error(io::ErrorKind::Other),
            Mode::Ready,
        ),
        None,
        None,
        None,
        control.clone(),
    );
    let error = flush(&mut flush_io)
        .await
        .expect_err("the test flusher fails");
    assert_eq!(error.kind(), io::ErrorKind::Other);
    assert_eq!(error.to_string(), "transport I/O failed");

    let mut vectored_io = DeadlineIo::new(
        ScriptIo::new(
            Mode::Ready,
            Mode::Error(io::ErrorKind::TimedOut),
            Mode::Ready,
            Mode::Ready,
        ),
        None,
        None,
        None,
        control.clone(),
    );
    let first = IoSlice::new(b"secret");
    let error = write_vectored(&mut vectored_io, &[first])
        .await
        .expect_err("the test vectored writer fails");
    assert_eq!(error.kind(), io::ErrorKind::TimedOut);
    assert_eq!(error.to_string(), "I/O deadline elapsed");

    let mut shutdown_io = DeadlineIo::new(
        ScriptIo::new(
            Mode::Ready,
            Mode::Ready,
            Mode::Ready,
            Mode::Error(io::ErrorKind::Other),
        ),
        None,
        None,
        None,
        control,
    );
    let error = shutdown(&mut shutdown_io)
        .await
        .expect_err("the test shutdown fails");
    assert_eq!(error.kind(), io::ErrorKind::Other);
    assert_eq!(error.to_string(), "transport I/O failed");
}

#[tokio::test]
async fn readiness_and_request_helpers_cover_success_and_retained_failures() {
    let (client_io, mut peer_io) = tokio::io::duplex(4096);
    let control = active_control();
    let wrapped = DeadlineIo::new(client_io, None, None, None, control.clone());
    let (mut sender, driver) = http1::handshake::<_, EmptyBody>(wrapped)
        .await
        .expect("HTTP handshake succeeds");
    let connection = tokio::spawn(driver);
    wait_for_sender_ready(
        &mut sender,
        Some(Duration::from_secs(1)),
        Some(Instant::now() + Duration::from_secs(1)),
    )
    .await
    .expect("idle sender is ready");

    let peer = tokio::spawn(async move {
        read_headers(&mut peer_io).await;
        peer_io
            .write_all(b"HTTP/1.1 200 OK\r\ncontent-length: 0\r\n\r\n")
            .await
            .expect("the response is delivered");
    });
    let mut pending = Some(request());
    let response = send_request_when_ready(
        &mut sender,
        &mut pending,
        &control,
        Some(Duration::from_secs(1)),
        Some(Instant::now() + Duration::from_secs(1)),
    )
    .await
    .expect("ready request is dispatched");
    assert!(pending.is_none());
    assert_eq!(
        control.delivery_state(),
        RequestDeliveryState::ResponseStarted
    );
    drop(response);
    peer.await.expect("local peer does not panic");
    connection.abort();
    let _ = connection.await;

    let (client_io, _peer_io) = tokio::io::duplex(4096);
    let wrapped = DeadlineIo::new(client_io, None, None, None, active_control());
    let (mut sender, driver) = http1::handshake::<_, EmptyBody>(wrapped)
        .await
        .expect("second HTTP handshake succeeds");
    let mut missing = None;
    let error = send_request_when_ready(&mut sender, &mut missing, &active_control(), None, None)
        .await
        .expect_err("missing request is rejected before readiness");
    assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
    drop(driver);

    let (client_io, _peer_io) = tokio::io::duplex(4096);
    let wrapped = DeadlineIo::new(client_io, None, None, None, active_control());
    let (mut sender, driver) = http1::handshake::<_, EmptyBody>(wrapped)
        .await
        .expect("third HTTP handshake succeeds");
    let finalized = active_control();
    finalized.cancel();
    let mut retained = Some(request());
    let error = send_request_when_ready(&mut sender, &mut retained, &finalized, None, None)
        .await
        .expect_err("finalized request is rejected");
    assert_eq!(error.kind(), io::ErrorKind::Interrupted);
    assert!(retained.is_some());
    drop(driver);

    let (client_io, _peer_io) = tokio::io::duplex(4096);
    let wrapped = DeadlineIo::new(client_io, None, None, None, ExchangeControl::new());
    let (mut sender, driver) = http1::handshake::<_, EmptyBody>(wrapped)
        .await
        .expect("fourth HTTP handshake succeeds");
    let connection = tokio::spawn(driver);
    let mut unstarted = Some(request());
    let error = send_request_when_ready(
        &mut sender,
        &mut unstarted,
        &ExchangeControl::new(),
        None,
        None,
    )
    .await
    .expect_err("dispatch requires the worker's begin transition");
    assert_eq!(error.kind(), io::ErrorKind::Interrupted);
    assert!(unstarted.is_some());
    connection.abort();
    let _ = connection.await;
}

#[tokio::test]
async fn readiness_timeout_cancels_control_and_keeps_request_for_no_later_dispatch() {
    let (client_io, mut peer_io) = tokio::io::duplex(4096);
    let primer_control = dispatched_control();
    let wrapped = DeadlineIo::new(client_io, None, None, None, primer_control.clone());
    let (mut sender, driver) = http1::handshake::<_, EmptyBody>(wrapped)
        .await
        .expect("HTTP handshake succeeds");
    let connection = tokio::spawn(driver);
    let primer = sender.send_request(request());
    tokio::pin!(primer);
    read_headers(&mut peer_io).await;

    let target_control = active_control();
    let mut pending = Some(request());
    let error = send_request_when_ready(
        &mut sender,
        &mut pending,
        &target_control,
        Some(Duration::from_millis(20)),
        Some(Instant::now() + Duration::from_secs(1)),
    )
    .await
    .expect_err("the in-flight primer keeps the sender unready");
    assert_eq!(error.kind(), io::ErrorKind::TimedOut);
    assert!(pending.is_some());
    assert_eq!(
        target_control.delivery_state(),
        RequestDeliveryState::NotSent
    );
    assert!(!target_control.commit_dispatch());

    peer_io
        .write_all(b"HTTP/1.1 200 OK\r\ncontent-length: 0\r\n\r\n")
        .await
        .expect("the primer response releases Hyper");
    drop(primer.await.expect("the primer response is parsed"));
    connection.abort();
    let _ = connection.await;
}

#[tokio::test]
async fn closed_sender_readiness_returns_a_safe_error() {
    let (client_io, peer_io) = tokio::io::duplex(1024);
    let wrapped = DeadlineIo::new(client_io, None, None, None, dispatched_control());
    let (mut sender, driver) = http1::handshake::<_, EmptyBody>(wrapped)
        .await
        .expect("HTTP handshake succeeds");
    let connection = tokio::spawn(driver);
    drop(peer_io);
    let _ = tokio::time::timeout(Duration::from_secs(1), connection)
        .await
        .expect("driver exits after peer closure");

    let error = tokio::time::timeout(
        Duration::from_secs(1),
        wait_for_sender_ready(&mut sender, None, None),
    )
    .await
    .expect("sender readiness returns after driver closure")
    .expect_err("closed sender is reported");
    assert_eq!(error.kind(), io::ErrorKind::BrokenPipe);
    assert_eq!(error.to_string(), "HTTP sender failed");
}

#[tokio::test]
async fn timeout_finalizer_waits_for_cleanup_before_snapshot() {
    let control = dispatched_control();
    let finalizing_control = control.clone();
    let (started_tx, started_rx) = tokio::sync::oneshot::channel();
    let (release_tx, release_rx) = tokio::sync::oneshot::channel();
    let finalizer = tokio::spawn(async move {
        finalize_timeout(&finalizing_control, async move {
            let _ = started_tx.send(());
            let _ = release_rx.await;
        })
        .await
    });
    started_rx.await.expect("cleanup begins");
    assert!(!finalizer.is_finished());
    assert_eq!(control.delivery_state(), RequestDeliveryState::PossiblySent);
    let _ = release_tx.send(());
    assert_eq!(
        finalizer.await.expect("finalizer does not panic"),
        RequestDeliveryState::PossiblySent
    );
}
