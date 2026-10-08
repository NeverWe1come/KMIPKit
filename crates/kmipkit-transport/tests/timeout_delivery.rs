//! Production transport timeout and HTTP delivery evidence tests.
//!
//! These tests include the real private timeout and worker state machines so
//! they can verify deadlines, cancellation, dispatch, response observation,
//! and finalization without exporting an internal test seam.

use std::convert::Infallible;
use std::error::Error as StdError;
use std::io::{self, IoSlice, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::pin::Pin;
use std::sync::{Arc, Mutex, mpsc};
use std::task::{Context, Poll, Waker};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use bytes::Bytes;
pub use config::{RequestOptions, TimeoutLimit, TimeoutPolicy, TransportConfig};
use hyper::body::{Body, Frame, SizeHint};
use hyper::client::conn::http1;
use hyper::http::Request;
use hyper::rt::{Read as HyperRead, ReadBuf as HyperReadBuf, Write as HyperWrite};
use kmipkit_test_support::{EphemeralPki, LoopbackTcpListener, fixtures};
pub use kmipkit_transport::{
    RequestDeliveryState, Transport, TransportCauseCategory, TransportError, TransportResponse,
};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::server::WebPkiClientVerifier;
use rustls::{RootCertStore, ServerConfig, ServerConnection, StreamOwned};
use std::future::poll_fn;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadBuf as TokioReadBuf};

const SHORT_IO_TIMEOUT: Duration = Duration::from_millis(250);

// The worker is compiled from its production source. Keep its implementation
// private to this integration test and suppress only dead-code noise from
// private helpers unrelated to the timeout contract.
#[allow(dead_code)]
#[path = "../src/worker.rs"]
mod worker;

// Include the production implementation directly so these tests exercise the
// private timeout seam rather than a test double.
#[path = "../src/timeout.rs"]
mod timeout;

// Include the production HTTPS adapter and its private dependencies so these
// tests exercise actual TLS, resolver, and HTTP integration boundaries.
#[path = "../src/config.rs"]
#[allow(dead_code)]
mod config;
#[path = "../src/https.rs"]
#[allow(dead_code)]
mod https;
#[path = "../src/resolver.rs"]
#[allow(dead_code)]
mod resolver;
#[path = "../src/secret.rs"]
#[allow(dead_code)]
mod secret;
#[path = "../src/tls.rs"]
#[allow(dead_code)]
mod tls;

use worker::ExchangeControl;

fn dispatched_control() -> ExchangeControl {
    let control = ExchangeControl::new();
    assert!(control.begin());
    assert!(control.commit_dispatch());
    control
}

fn deadline_io<I>(
    inner: I,
    read_timeout: Option<Duration>,
    write_timeout: Option<Duration>,
    total_deadline: Option<Instant>,
    control: ExchangeControl,
) -> timeout::DeadlineIo<I> {
    timeout::DeadlineIo::new(inner, read_timeout, write_timeout, total_deadline, control)
}

async fn write_once<W: HyperWrite + Unpin>(writer: &mut W, bytes: &[u8]) -> io::Result<usize> {
    poll_fn(|cx| Pin::new(&mut *writer).poll_write(cx, bytes)).await
}

async fn write_all<W: HyperWrite + Unpin>(writer: &mut W, mut bytes: &[u8]) -> io::Result<()> {
    while !bytes.is_empty() {
        let written = write_once(writer, bytes).await?;
        if written == 0 {
            return Err(io::Error::new(
                io::ErrorKind::WriteZero,
                "test writer closed",
            ));
        }
        bytes = &bytes[written..];
    }
    Ok(())
}

async fn flush<W: HyperWrite + Unpin>(writer: &mut W) -> io::Result<()> {
    poll_fn(|cx| Pin::new(&mut *writer).poll_flush(cx)).await
}

async fn read_exact<R: HyperRead + Unpin>(reader: &mut R, mut bytes: &mut [u8]) -> io::Result<()> {
    while !bytes.is_empty() {
        let mut read_buf = HyperReadBuf::new(bytes);
        poll_fn(|cx| Pin::new(&mut *reader).poll_read(cx, read_buf.unfilled())).await?;
        let read = read_buf.filled().len();
        if read == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "test reader closed",
            ));
        }
        bytes = &mut bytes[read..];
    }
    Ok(())
}

#[tokio::test]
async fn a_blocked_read_uses_the_read_inactivity_deadline() {
    let inner = ScriptIo::manual();
    let control = dispatched_control();
    let mut io = deadline_io(
        inner,
        Some(SHORT_IO_TIMEOUT),
        Some(Duration::from_secs(1)),
        None,
        control.clone(),
    );
    let mut response_byte = [0];

    let error = read_exact(&mut io, &mut response_byte)
        .await
        .expect_err("a read with no progress must expire");

    assert_eq!(error.kind(), io::ErrorKind::TimedOut);
    assert_eq!(control.cancel(), RequestDeliveryState::PossiblySent);
}

#[tokio::test]
async fn an_unrepresentable_finite_read_phase_is_rejected_as_invalid_input() {
    let control = ExchangeControl::new();
    let mut io = deadline_io(
        ScriptIo::manual(),
        Some(Duration::MAX),
        Some(Duration::from_secs(1)),
        None,
        control,
    );
    let mut response_byte = [0];

    let result = tokio::time::timeout(
        Duration::from_millis(100),
        read_exact(&mut io, &mut response_byte),
    )
    .await;

    assert!(
        matches!(result, Ok(Err(ref error)) if error.kind() == io::ErrorKind::InvalidInput),
        "a finite read phase outside the monotonic clock range must fail as invalid input"
    );
}

#[tokio::test]
async fn a_blocked_write_uses_the_write_inactivity_deadline() {
    let inner = ScriptIo::manual();
    let control = dispatched_control();
    let mut io = deadline_io(
        inner,
        Some(Duration::from_secs(1)),
        Some(SHORT_IO_TIMEOUT),
        None,
        control.clone(),
    );

    let error = write_all(&mut io, b"request")
        .await
        .expect_err("a write with no progress must expire");

    assert_eq!(error.kind(), io::ErrorKind::TimedOut);
    assert_eq!(control.cancel(), RequestDeliveryState::PossiblySent);
}

#[tokio::test]
async fn a_blocked_flush_uses_the_write_inactivity_deadline() {
    let inner = ScriptIo::manual();
    let control = dispatched_control();
    let mut io = deadline_io(
        inner,
        Some(Duration::from_secs(1)),
        Some(SHORT_IO_TIMEOUT),
        None,
        control.clone(),
    );

    let error = flush(&mut io)
        .await
        .expect_err("a flush with no progress must expire");

    assert_eq!(error.kind(), io::ErrorKind::TimedOut);
    assert_eq!(control.cancel(), RequestDeliveryState::PossiblySent);
}

#[tokio::test]
async fn finalized_exchange_rejects_later_flush_io() {
    let inner = ScriptIo::manual();
    let observed = inner.clone();
    let control = dispatched_control();
    let mut io = deadline_io(inner, None, None, None, control.clone());
    assert_eq!(control.cancel(), RequestDeliveryState::PossiblySent);
    observed.release_flush();

    let error = flush(&mut io)
        .await
        .expect_err("finalized exchange I/O is rejected before flushing");

    assert_eq!(error.kind(), io::ErrorKind::Interrupted);
}

#[tokio::test]
async fn positive_read_progress_restarts_only_the_read_deadline() {
    let inner = ScriptIo::manual();
    let progress = inner.clone();
    let control = dispatched_control();
    let mut io = deadline_io(
        inner,
        Some(Duration::from_millis(500)),
        Some(Duration::from_secs(1)),
        Some(Instant::now() + Duration::from_secs(1)),
        control.clone(),
    );
    let read = tokio::spawn(async move {
        let mut bytes = [0; 3];
        read_exact(&mut io, &mut bytes).await.map(|()| bytes)
    });

    for byte in [7, 8, 9] {
        tokio::time::sleep(Duration::from_millis(200)).await;
        progress.release_read(vec![byte]);
    }

    assert_eq!(
        read.await
            .expect("read task does not panic")
            .expect("each positive read resets the phase deadline"),
        [7, 8, 9]
    );
    assert_eq!(
        control.delivery_state(),
        RequestDeliveryState::ResponseStarted
    );
}

#[tokio::test]
async fn positive_write_progress_restarts_only_the_write_deadline() {
    let inner = ScriptIo::manual();
    let progress = inner.clone();
    let written = inner.clone();
    let control = dispatched_control();
    let mut io = deadline_io(
        inner,
        Some(Duration::from_secs(1)),
        Some(Duration::from_millis(500)),
        Some(Instant::now() + Duration::from_secs(1)),
        control.clone(),
    );
    let write = tokio::spawn(async move { write_all(&mut io, b"abc").await });

    for _ in 0..3 {
        tokio::time::sleep(Duration::from_millis(200)).await;
        progress.release_write(1);
    }

    write
        .await
        .expect("write task does not panic")
        .expect("each positive write resets the phase deadline");
    assert_eq!(written.written_bytes(), b"abc");
    assert_eq!(control.delivery_state(), RequestDeliveryState::PossiblySent);
}

#[tokio::test]
async fn successful_flush_progress_restarts_the_write_deadline() {
    let inner = ScriptIo::manual();
    let progress = inner.clone();
    let control = dispatched_control();
    let mut io = deadline_io(
        inner,
        Some(Duration::from_secs(1)),
        Some(Duration::from_millis(500)),
        Some(Instant::now() + Duration::from_secs(1)),
        control.clone(),
    );
    let releases = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(200)).await;
        progress.release_flush();
        tokio::time::sleep(Duration::from_millis(200)).await;
        progress.release_flush();
    });

    flush(&mut io)
        .await
        .expect("the first successful flush is progress");
    flush(&mut io)
        .await
        .expect("the write deadline restarts after the successful flush");
    releases.await.expect("flush release task does not panic");
    assert_eq!(control.delivery_state(), RequestDeliveryState::PossiblySent);
}

#[tokio::test]
async fn the_absolute_total_deadline_does_not_reset_after_write_progress() {
    let inner = ScriptIo::manual();
    let progress = inner.clone();
    let written = inner.clone();
    let control = dispatched_control();
    let deadline = Instant::now() + Duration::from_millis(1_500);
    let mut io = deadline_io(
        inner,
        Some(Duration::from_secs(2)),
        Some(Duration::from_secs(2)),
        Some(deadline),
        control.clone(),
    );
    let write = tokio::spawn(async move { write_all(&mut io, b"abc").await });

    for _ in 0..2 {
        tokio::time::sleep(Duration::from_millis(300)).await;
        progress.release_write(1);
    }

    let error = write
        .await
        .expect("write task does not panic")
        .expect_err("the total deadline remains absolute after progress");

    assert_eq!(error.kind(), io::ErrorKind::TimedOut);
    assert_eq!(written.written_bytes(), b"ab");
    assert_eq!(control.cancel(), RequestDeliveryState::PossiblySent);
}

#[tokio::test]
async fn each_wait_uses_the_earlier_phase_or_total_deadline() {
    for (phase, total, maximum_elapsed) in [
        (
            Duration::from_millis(500),
            Duration::from_millis(1_500),
            Duration::from_millis(1_000),
        ),
        (
            Duration::from_millis(1_500),
            Duration::from_millis(500),
            Duration::from_millis(1_000),
        ),
    ] {
        let control = dispatched_control();
        let mut io = deadline_io(
            ScriptIo::manual(),
            Some(Duration::from_secs(1)),
            Some(phase),
            Some(Instant::now() + total),
            control,
        );
        let started = Instant::now();

        let error = write_all(&mut io, b"request")
            .await
            .expect_err("one of the two deadlines must expire");

        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
        assert!(started.elapsed() < maximum_elapsed);
    }
}

#[tokio::test]
async fn hyper_sender_readiness_timeout_is_not_sent_before_dispatch_commit() {
    for (write_timeout, total_timeout) in [
        (SHORT_IO_TIMEOUT, Duration::from_secs(1)),
        (Duration::from_secs(1), SHORT_IO_TIMEOUT),
    ] {
        let (client_io, mut server_io) = tokio::io::duplex(8 * 1024);
        let primer_control = dispatched_control();
        let target_control = ExchangeControl::new();
        let wrapped = deadline_io(
            client_io,
            Some(Duration::from_secs(1)),
            Some(Duration::from_secs(5)),
            Some(Instant::now() + Duration::from_secs(10)),
            primer_control.clone(),
        );
        let (mut sender, driver) = http1::handshake::<_, TestBody>(wrapped)
            .await
            .expect("Hyper HTTP/1 handshake succeeds over the duplex stream");
        let connection = tokio::spawn(driver);
        let primer_response = sender.send_request(request(EmptyBody));
        tokio::pin!(primer_response);
        let request_headers = tokio::select! {
            result = &mut primer_response => {
                panic!("the primer response cannot arrive before the peer sends headers: {result:?}");
            }
            headers = read_http_headers(&mut server_io) => headers,
        };
        assert!(request_headers.starts_with(b"POST /kmip HTTP/1.1\r\n"));

        assert!(target_control.begin());
        let target_deadline = Instant::now() + total_timeout;
        let mut pending_request = Some(request(EmptyBody));
        // T010's helper holds this second request at readiness, then commits
        // the target control before calling Hyper's send_request.
        let first_attempt = timeout::send_request_when_ready(
            &mut sender,
            &mut pending_request,
            &target_control,
            Some(write_timeout),
            Some(target_deadline),
        )
        .await;
        let Err(first_error) = first_attempt else {
            panic!("the actual request remains blocked at sender readiness");
        };

        assert_eq!(first_error.kind(), io::ErrorKind::TimedOut);
        assert!(pending_request.is_some());
        assert_eq!(
            target_control.delivery_state(),
            RequestDeliveryState::NotSent
        );
        assert!(!target_control.commit_dispatch());

        // Release the first request only after the second exchange timed out.
        server_io
            .write_all(b"HTTP/1.1 200 OK\r\ncontent-length: 0\r\n\r\n")
            .await
            .expect("the first response releases Hyper's sender");
        let first_response = primer_response
            .await
            .expect("the first request future remains live while readiness waits");
        drop(first_response);
        assert_eq!(
            primer_control.delivery_state(),
            RequestDeliveryState::ResponseStarted
        );

        let retry_deadline = Instant::now() + Duration::from_secs(1);
        timeout::wait_for_sender_ready(
            &mut sender,
            Some(Duration::from_secs(1)),
            Some(retry_deadline),
        )
        .await
        .expect("the first response makes the same sender ready");
        let retry_attempt = timeout::send_request_when_ready(
            &mut sender,
            &mut pending_request,
            &target_control,
            Some(Duration::from_secs(1)),
            Some(retry_deadline),
        )
        .await;
        assert!(retry_attempt.is_err());
        assert!(pending_request.is_some());
        assert_eq!(
            target_control.delivery_state(),
            RequestDeliveryState::NotSent
        );

        let mut second_request_byte = [0_u8; 1];
        let second_request = tokio::time::timeout(
            Duration::from_secs(1),
            server_io.read(&mut second_request_byte),
        )
        .await;
        assert!(
            matches!(second_request, Err(_) | Ok(Ok(0))),
            "the timed-out request was not sent later; observed {second_request:?}"
        );
        connection.abort();
        let _ = connection.await;
    }
}

#[test]
fn delivery_states_advance_only_at_dispatch_and_first_response_byte() {
    let control = ExchangeControl::new();
    assert_eq!(control.delivery_state(), RequestDeliveryState::NotSent);
    assert!(control.begin());
    assert_eq!(control.delivery_state(), RequestDeliveryState::NotSent);
    assert!(control.commit_dispatch());
    assert_eq!(control.delivery_state(), RequestDeliveryState::PossiblySent);
    assert!(control.response_started());
    assert_eq!(
        control.delivery_state(),
        RequestDeliveryState::ResponseStarted
    );
    assert_eq!(control.cancel(), RequestDeliveryState::ResponseStarted);
}

#[test]
fn cancellation_before_dispatch_makes_a_later_commit_impossible() {
    let control = ExchangeControl::new();
    assert!(control.begin());

    assert_eq!(control.cancel(), RequestDeliveryState::NotSent);
    assert!(!control.commit_dispatch());
    assert_eq!(control.delivery_state(), RequestDeliveryState::NotSent);
}

#[test]
fn cancellation_and_dispatch_race_has_one_delivery_state_winner() {
    for _ in 0..64 {
        let control = ExchangeControl::new();
        assert!(control.begin());
        let start = Arc::new(std::sync::Barrier::new(3));
        let committing_control = control.clone();
        let committing_start = Arc::clone(&start);
        let committing = std::thread::spawn(move || {
            committing_start.wait();
            committing_control.commit_dispatch()
        });
        let canceling_control = control.clone();
        let canceling_start = Arc::clone(&start);
        let canceling = std::thread::spawn(move || {
            canceling_start.wait();
            canceling_control.cancel()
        });

        start.wait();
        let dispatch_won = committing.join().expect("dispatch racer does not panic");
        let delivery = canceling.join().expect("cancellation racer does not panic");
        assert_eq!(
            delivery,
            if dispatch_won {
                RequestDeliveryState::PossiblySent
            } else {
                RequestDeliveryState::NotSent
            }
        );
    }
}

#[tokio::test]
async fn writer_failure_immediately_after_dispatch_is_possibly_sent() {
    let inner = ScriptIo::failing_immediately();
    let control = ExchangeControl::new();
    assert!(control.begin());
    assert!(control.commit_dispatch());
    let mut io = deadline_io(inner, None, None, None, control.clone());

    let error = write_once(&mut io, b"request")
        .await
        .expect_err("the fake writer fails after dispatch commit");

    assert_eq!(error.kind(), io::ErrorKind::BrokenPipe);
    assert_eq!(control.cancel(), RequestDeliveryState::PossiblySent);
}

#[tokio::test]
async fn hyper_header_write_can_timeout_before_any_ttlv_body_byte() {
    let inner = ScriptIo::blocked_after_headers();
    let observed = inner.clone();
    let control = ExchangeControl::new();
    assert!(control.begin());
    let wrapped = deadline_io(
        inner,
        Some(Duration::from_secs(1)),
        Some(SHORT_IO_TIMEOUT),
        Some(Instant::now() + Duration::from_secs(1)),
        control.clone(),
    );
    let (mut sender, driver) = http1::handshake::<_, TestBody>(wrapped)
        .await
        .expect("Hyper HTTP/1 handshake succeeds over the scripted writer");
    let connection = tokio::spawn(driver);
    timeout::wait_for_sender_ready(
        &mut sender,
        Some(SHORT_IO_TIMEOUT),
        Some(Instant::now() + Duration::from_secs(1)),
    )
    .await
    .expect("sender is initially ready");
    assert!(control.commit_dispatch());

    let error = sender
        .send_request(request(OneChunkBody::new(b"TTLV")))
        .await
        .expect_err("the body write blocks after Hyper has emitted its headers");
    assert!(error_chain_has_io_kind(&error, io::ErrorKind::TimedOut));

    let bytes = observed.written_bytes();
    let body_start = bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|position| position + 4)
        .expect("Hyper emitted the request headers");
    assert!(bytes[..body_start].starts_with(b"POST /kmip HTTP/1.1\r\n"));
    assert_eq!(bytes[body_start..], []);
    assert_eq!(control.cancel(), RequestDeliveryState::PossiblySent);
    connection.abort();
}

#[tokio::test]
async fn hyper_partial_body_write_remains_possibly_sent() {
    let inner = ScriptIo::failing_after_body_bytes(2);
    let observed = inner.clone();
    let control = dispatched_control();
    let wrapped = deadline_io(
        inner,
        Some(Duration::from_secs(1)),
        Some(Duration::from_secs(1)),
        None,
        control.clone(),
    );
    let (mut sender, driver) = http1::handshake::<_, TestBody>(wrapped)
        .await
        .expect("Hyper HTTP/1 handshake succeeds over the scripted writer");
    let connection = tokio::spawn(driver);

    let _ = sender
        .send_request(request(OneChunkBody::new(b"TTLV")))
        .await
        .expect_err("the scripted writer fails after two TTLV body bytes");

    let bytes = observed.written_bytes();
    let body_start = bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|position| position + 4)
        .expect("Hyper emitted the request headers");
    assert_eq!(&bytes[body_start..], b"TT");
    assert_eq!(control.cancel(), RequestDeliveryState::PossiblySent);
    connection.abort();
}

#[tokio::test]
async fn a_positive_wrapped_read_wins_before_timeout_finalization() {
    let (client_io, mut server_io) = tokio::io::duplex(8 * 1024);
    let control = ExchangeControl::new();
    assert!(control.begin());
    let wrapped = deadline_io(
        client_io,
        Some(SHORT_IO_TIMEOUT),
        Some(Duration::from_secs(1)),
        Some(Instant::now() + Duration::from_secs(1)),
        control.clone(),
    );
    let (mut sender, driver) = http1::handshake::<_, TestBody>(wrapped)
        .await
        .expect("Hyper HTTP/1 handshake succeeds over the duplex stream");
    let connection = tokio::spawn(driver);
    timeout::wait_for_sender_ready(
        &mut sender,
        Some(Duration::from_secs(1)),
        Some(Instant::now() + Duration::from_secs(1)),
    )
    .await
    .expect("sender readiness succeeds before dispatch");
    assert!(control.commit_dispatch());
    let server = tokio::spawn(async move {
        let _ = read_http_headers(&mut server_io).await;
        server_io
            .write_all(b"HTTP/1.1 200 OK\r\ncontent-length: 4\r\n\r\n")
            .await
            .expect("server response headers are delivered");
        std::future::pending::<()>().await;
    });

    let mut response = sender
        .send_request(request(EmptyBody))
        .await
        .expect("Hyper parsed the response headers");
    assert_eq!(
        control.delivery_state(),
        RequestDeliveryState::ResponseStarted
    );

    let error = poll_fn(|cx| Pin::new(response.body_mut()).poll_frame(cx))
        .await
        .expect("response body does not end before bytes arrive")
        .expect_err("the response body read hits its inactivity deadline");

    assert!(error_chain_has_io_kind(&error, io::ErrorKind::TimedOut));
    let finalizing_control = control.clone();
    let (cleanup_started_tx, cleanup_started_rx) = tokio::sync::oneshot::channel();
    let (release_cleanup_tx, release_cleanup_rx) = tokio::sync::oneshot::channel();
    // The response headers already passed through DeadlineIo as a positive
    // read. T010's finalizer must preserve that winner and wait for real
    // request/driver cleanup before returning its delivery snapshot.
    let finalizer = tokio::spawn(async move {
        timeout::finalize_timeout(&finalizing_control, async move {
            let _ = cleanup_started_tx.send(());
            let _ = release_cleanup_rx.await;
            drop(response);
            server.abort();
            connection.abort();
            let _ = server.await;
            let _ = connection.await;
        })
        .await
    });
    cleanup_started_rx
        .await
        .expect("finalization closes the read gate before awaiting cleanup");
    assert!(!finalizer.is_finished());
    assert_eq!(
        control.delivery_state(),
        RequestDeliveryState::ResponseStarted
    );
    let _ = release_cleanup_tx.send(());
    let final_delivery = finalizer
        .await
        .expect("the timeout finalizer does not panic");
    assert_eq!(final_delivery, RequestDeliveryState::ResponseStarted);
}

#[tokio::test]
async fn vectored_writes_are_subject_to_the_same_write_deadline() {
    let inner = ScriptIo::manual();
    let observed = inner.clone();
    let control = dispatched_control();
    let mut io = deadline_io(
        inner,
        Some(Duration::from_secs(1)),
        Some(SHORT_IO_TIMEOUT),
        None,
        control.clone(),
    );
    let first = IoSlice::new(b"header");
    let second = IoSlice::new(b"body");

    let error = poll_fn(|cx| {
        hyper::rt::Write::poll_write_vectored(Pin::new(&mut io), cx, &[first, second])
    })
    .await
    .expect_err("a blocked vectored write is bounded by the write deadline");

    assert_eq!(error.kind(), io::ErrorKind::TimedOut);
    assert_eq!(observed.vectored_write_calls(), 1);
    assert_eq!(control.cancel(), RequestDeliveryState::PossiblySent);
}

#[tokio::test]
async fn a_late_positive_read_cannot_change_delivery_after_finalization_wins() {
    let (inner, first_read_poll, first_write, late_positive_read) =
        ScriptIo::automatic_with_exchange_notifications();
    let control = ExchangeControl::new();
    assert!(control.begin());
    let wrapped = deadline_io(
        inner.clone(),
        Some(Duration::from_secs(2)),
        Some(Duration::from_secs(2)),
        Some(Instant::now() + Duration::from_secs(3)),
        control.clone(),
    );
    let (mut sender, driver) = http1::handshake::<_, TestBody>(wrapped)
        .await
        .expect("Hyper HTTP/1 handshake succeeds over scripted I/O");
    let connection = tokio::spawn(driver);
    timeout::wait_for_sender_ready(
        &mut sender,
        Some(Duration::from_secs(1)),
        Some(Instant::now() + Duration::from_secs(2)),
    )
    .await
    .expect("sender readiness succeeds before dispatch");
    assert!(control.commit_dispatch());

    let (response_observed_tx, response_observed_rx) = tokio::sync::oneshot::channel();
    let response = tokio::spawn(async move {
        if let Ok(mut response) = sender.send_request(request(EmptyBody)).await {
            let _ = response_observed_tx.send(());
            let _ = poll_fn(|cx| Pin::new(response.body_mut()).poll_frame(cx)).await;
        }
    });
    first_write
        .await
        .expect("the request reached the wrapped writer");
    first_read_poll
        .await
        .expect("the real Hyper driver is waiting on the wrapped read side");
    assert_eq!(control.delivery_state(), RequestDeliveryState::PossiblySent);

    let finalizing_control = control.clone();
    let (cleanup_started_tx, cleanup_started_rx) = tokio::sync::oneshot::channel();
    let (release_cleanup_tx, release_cleanup_rx) = tokio::sync::oneshot::channel();
    let finalizer = tokio::spawn(async move {
        timeout::finalize_timeout(&finalizing_control, async move {
            // finalize_timeout must close response observation before it polls
            // this cancellation/driver-invalidation future.
            response.abort();
            connection.abort();
            let _ = cleanup_started_tx.send(());
            let _ = release_cleanup_rx.await;
            let _ = response.await;
            let _ = connection.await;
        })
        .await
    });
    cleanup_started_rx
        .await
        .expect("finalization closes response observation before cleanup");
    assert!(!finalizer.is_finished());

    // Make a complete positive HTTP response available only after the gate is
    // finalized. This deliberately exercises late-I/O suppression: it must
    // not revise delivery evidence while cleanup still holds the finalizer.
    let late_response = b"HTTP/1.1 200 OK\r\ncontent-length: 4\r\n\r\nTTLV";
    inner.release_read(late_response.to_vec());
    let late_read = tokio::time::timeout(Duration::from_millis(100), late_positive_read).await;
    assert!(
        late_read.is_err(),
        "the aborted driver must not perform a late positive read"
    );
    let response_observed =
        tokio::time::timeout(Duration::from_millis(100), response_observed_rx).await;
    assert!(
        !matches!(response_observed, Ok(Ok(()))),
        "the aborted request task must not receive the late response"
    );
    assert_eq!(inner.queued_read_bytes(), late_response.len());
    assert_eq!(control.delivery_state(), RequestDeliveryState::PossiblySent);
    assert!(!finalizer.is_finished());

    let _ = release_cleanup_tx.send(());
    let final_delivery = finalizer
        .await
        .expect("the timeout finalizer does not panic");
    assert_eq!(final_delivery, RequestDeliveryState::PossiblySent);
    assert_eq!(control.delivery_state(), RequestDeliveryState::PossiblySent);
}

#[test]
fn https_connect_deadline_ignores_a_late_blocked_resolver_result() {
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    let listener = LoopbackTcpListener::bind().expect("the late-result probe binds loopback");
    let address = listener.local_addr();
    let listener = listener.into_inner();
    let configuration = https_client_configuration(&pki, address.port());
    let (resolver, gate) = gated_https_resolver(vec![address]);
    let adapter = https::new_for_test_with_resolver(configuration, None, resolver);
    let options =
        RequestOptions::default().with_connect(TimeoutLimit::Bounded(Duration::from_millis(250)));
    let (result_tx, result_rx) = mpsc::sync_channel(1);

    thread::spawn(move || {
        let mut adapter = adapter;
        let result = adapter.exchange_with_options(fixtures::REQUEST_SENTINEL, 64, &options);
        let _ = result_tx.send((adapter, result));
    });

    gate.wait_until_started();
    let (_adapter, result) = result_rx
        .recv_timeout(Duration::from_secs(3))
        .expect("the connect deadline expires while the native resolver is held");
    let error = result.expect_err("the unresolved lookup reaches its connect deadline");
    assert_eq!(error.delivery_state(), RequestDeliveryState::NotSent);
    assert_eq!(error.cause_category(), TransportCauseCategory::Timeout);

    gate.release();
    gate.wait_until_returned();
    assert_no_tcp_connection_for(&listener, Duration::from_millis(500));
}

#[test]
fn https_unresolved_resolver_sends_no_early_bytes_then_succeeds_after_release() {
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    let listener = LoopbackTcpListener::bind().expect("the HTTPS peer binds loopback");
    let address = listener.local_addr();
    let configuration = https_client_configuration(&pki, address.port());
    let (resolver, gate) = gated_https_resolver(vec![address]);
    let (peer, connected) = spawn_https_peer(
        listener.into_inner(),
        https_server_config(&pki, &pki),
        Some(complete_http_response(b"done")),
        None,
    );
    let adapter = https::new_for_test_with_resolver(configuration, None, resolver);
    let (result_tx, result_rx) = mpsc::sync_channel(1);

    thread::spawn(move || {
        let mut adapter = adapter;
        let result = adapter.exchange(fixtures::REQUEST_SENTINEL, 64);
        let _ = result_tx.send(result);
    });

    gate.wait_until_started();
    assert!(
        matches!(
            connected.recv_timeout(Duration::from_millis(200)),
            Err(mpsc::RecvTimeoutError::Timeout)
        ),
        "the HTTPS peer sees no TCP connection while DNS resolution is unresolved"
    );
    gate.release();
    gate.wait_until_returned();

    connected
        .recv_timeout(Duration::from_secs(3))
        .expect("TCP starts only after the resolver returns its candidate");
    let response = result_rx
        .recv_timeout(Duration::from_secs(3))
        .expect("the released resolver allows the HTTPS exchange to finish")
        .expect("the verified HTTPS exchange succeeds after lookup release");
    assert_eq!(response.as_bytes(), b"done");
    let peer = peer.join().expect("the bounded HTTPS peer completes");
    assert!(peer.handshake_completed);
    assert_eq!(peer.requests.len(), 1);
    assert_eq!(
        http_request_body(&peer.requests[0]),
        fixtures::REQUEST_SENTINEL
    );
}

#[test]
fn https_new_uses_the_system_resolver_for_a_loopback_ip_endpoint() {
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    let listener = LoopbackTcpListener::bind().expect("the HTTPS peer binds loopback");
    let address = listener.local_addr();
    let configuration = https_client_configuration_for_endpoint(
        &pki,
        format!("https://127.0.0.1:{}", address.port()),
        Some(HTTPS_SERVER_NAME),
    );
    let (peer, connected) = spawn_https_peer(
        listener.into_inner(),
        https_server_config(&pki, &pki),
        Some(complete_http_response(b"done")),
        None,
    );
    let mut adapter = https::HttpsTransport::new(configuration)
        .expect("the numeric loopback endpoint builds a verified HTTPS transport");

    let response = adapter
        .exchange(fixtures::REQUEST_SENTINEL, 64)
        .expect("the production system resolver resolves the loopback IP endpoint");
    assert_eq!(response.as_bytes(), b"done");
    connected
        .recv_timeout(Duration::from_secs(1))
        .expect("the system-resolver-backed transport reaches the HTTPS peer");
    let peer = peer
        .join()
        .expect("the bounded system-resolver peer completes");
    assert!(peer.handshake_completed);
    assert_eq!(peer.requests.len(), 1);
    assert_eq!(
        http_request_body(&peer.requests[0]),
        fixtures::REQUEST_SENTINEL
    );
}

#[test]
fn https_connect_deadline_during_tls_handshake_is_not_sent() {
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    let listener = LoopbackTcpListener::bind().expect("the stalled TLS peer binds loopback");
    let address = listener.local_addr();
    let configuration = https_client_configuration(&pki, address.port());
    let resolver = https_resolver(vec![address]);
    let (peer, accepted, release) = spawn_stalled_tls_peer(listener.into_inner());
    let mut adapter = https::new_for_test_with_resolver(configuration, None, resolver);
    let options =
        RequestOptions::default().with_connect(TimeoutLimit::Bounded(Duration::from_secs(1)));

    let error = adapter
        .exchange_with_options(fixtures::REQUEST_SENTINEL, 64, &options)
        .expect_err("the stalled TLS handshake exceeds the connect deadline");
    assert_eq!(error.delivery_state(), RequestDeliveryState::NotSent);
    assert_eq!(error.cause_category(), TransportCauseCategory::Timeout);
    accepted
        .recv_timeout(Duration::from_secs(1))
        .expect("the stalled peer accepted the TCP connection");
    release
        .send(())
        .expect("the timed-out handshake peer is released for cleanup");
    let peer = peer.join().expect("the stalled TLS peer completes");
    assert_eq!(peer.first_tls_record_type, Some(22));
    assert!(
        !peer.handshake_completed,
        "the client deadline expires before TLS completes"
    );
    assert!(
        peer.http_request.is_none(),
        "HTTP and KMIP bytes are never dispatched before a completed TLS handshake"
    );
}

#[test]
fn https_tries_resolver_candidates_in_order_and_dispatches_only_after_trusted_tls() {
    let client_pki = EphemeralPki::generate().expect("the trusted client PKI is generated");
    let untrusted_server_pki =
        EphemeralPki::generate().expect("the first endpoint has a separate server PKI");
    let first_listener =
        LoopbackTcpListener::bind().expect("the untrusted TLS endpoint binds loopback");
    let first_address = first_listener.local_addr();
    let second_listener =
        LoopbackTcpListener::bind().expect("the trusted TLS endpoint binds loopback");
    let second_address = second_listener.local_addr();
    let configuration = https_client_configuration(&client_pki, first_address.port());
    let resolver = https_resolver(vec![first_address, second_address]);
    let (release_first_handshake, first_handshake_gate) = mpsc::sync_channel(1);
    let (first_peer, first_connected) = spawn_https_peer(
        first_listener.into_inner(),
        https_server_config(&untrusted_server_pki, &client_pki),
        None,
        Some(first_handshake_gate),
    );
    let (second_peer, second_connected) = spawn_https_peer(
        second_listener.into_inner(),
        https_server_config(&client_pki, &client_pki),
        Some(complete_http_response(b"done")),
        None,
    );
    let adapter = https::new_for_test_with_resolver(configuration, None, resolver);
    let (result_tx, result_rx) = mpsc::sync_channel(1);

    thread::spawn(move || {
        let mut adapter = adapter;
        let result = adapter.exchange(fixtures::REQUEST_SENTINEL, 64);
        let _ = result_tx.send(result);
    });
    first_connected
        .recv_timeout(Duration::from_secs(1))
        .expect("the first resolver candidate reaches TCP");
    assert!(
        matches!(
            second_connected.recv_timeout(Duration::from_millis(200)),
            Err(mpsc::RecvTimeoutError::Timeout)
        ),
        "the second TCP candidate cannot start while the first TLS handshake is held"
    );
    release_first_handshake
        .send(())
        .expect("the first untrusted TLS handshake is released");
    let response = result_rx
        .recv_timeout(Duration::from_secs(3))
        .expect("the candidate loop advances after the first TLS failure")
        .expect("the later trusted TLS candidate completes the HTTPS exchange");
    assert_eq!(response.as_bytes(), b"done");
    second_connected
        .recv_timeout(Duration::from_secs(1))
        .expect("the trusted candidate starts after the first TLS failure");

    let first_peer = first_peer
        .join()
        .expect("the untrusted TLS peer finishes its bounded handshake");
    let second_peer = second_peer
        .join()
        .expect("the trusted HTTPS peer completes");
    assert!(
        !first_peer.handshake_completed,
        "the first endpoint fails server certificate validation"
    );
    assert!(
        first_peer.requests.is_empty(),
        "no HTTP or KMIP request reaches an untrusted TLS endpoint"
    );
    assert!(second_peer.handshake_completed);
    assert_eq!(second_peer.requests.len(), 1);
    assert_eq!(
        http_request_body(&second_peer.requests[0]),
        fixtures::REQUEST_SENTINEL
    );
}

#[test]
fn https_total_deadline_after_partial_body_invalidates_and_explicitly_reconnects() {
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    let listener = LoopbackTcpListener::bind().expect("the reconnect peer binds loopback");
    let address = listener.local_addr();
    let configuration = https_client_configuration(&pki, address.port());
    let resolver = https_resolver(vec![address]);
    let (peer, partial_response_sent, release_peer) = spawn_partial_then_success_https_peer(
        listener.into_inner(),
        https_server_config(&pki, &pki),
    );
    let adapter = https::new_for_test_with_resolver(configuration, None, resolver);
    let first_options =
        RequestOptions::default().with_total(TimeoutLimit::Bounded(Duration::from_millis(1_500)));
    let (result_tx, result_rx) = mpsc::sync_channel(1);

    thread::spawn(move || {
        let mut adapter = adapter;
        let result = adapter.exchange_with_options(b"first-call", 64, &first_options);
        let _ = result_tx.send((adapter, result));
    });

    partial_response_sent
        .recv_timeout(Duration::from_secs(3))
        .expect("the first request receives HTTP headers and a partial body");
    let (mut adapter, first_result) = result_rx
        .recv_timeout(Duration::from_secs(3))
        .expect("the total deadline finalizes the partially received exchange");
    let first_error =
        first_result.expect_err("the incomplete first response reaches total timeout");
    assert_eq!(
        first_error.delivery_state(),
        RequestDeliveryState::ResponseStarted
    );
    assert_eq!(
        first_error.cause_category(),
        TransportCauseCategory::Timeout
    );
    assert!(
        !https::has_cached_connection_for_test(&adapter),
        "a connection with a timed-out partial response is invalidated"
    );

    release_peer
        .send(())
        .expect("the peer advances to an explicit second exchange");
    let second_response = adapter
        .exchange_with_options(b"second-call", 64, &RequestOptions::default())
        .expect("the later explicit exchange reconnects and succeeds");
    assert_eq!(second_response.as_bytes(), b"done");
    let peer = peer.join().expect("the two-connection peer completes");
    assert_eq!(peer.accepted_connections, 2);
    assert_eq!(peer.requests.len(), 2);
    assert_eq!(http_request_body(&peer.requests[0]), b"first-call");
    assert_eq!(http_request_body(&peer.requests[1]), b"second-call");
}

#[test]
fn https_body_read_timeout_preserves_timeout_cause_and_redacts_payloads() {
    const REQUEST_SENTINEL: &[u8] = b"REQUEST_SECRET_SENTINEL";
    const RESPONSE_SENTINEL: &[u8] = b"PARTIAL_RESPONSE_SECRET";

    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    let listener = LoopbackTcpListener::bind().expect("the stalled-body peer binds loopback");
    let address = listener.local_addr();
    let configuration = https_client_configuration(&pki, address.port());
    let resolver = https_resolver(vec![address]);
    let (peer, partial_sent, release_peer) = spawn_stalled_partial_body_https_peer(
        listener.into_inner(),
        https_server_config(&pki, &pki),
        RESPONSE_SENTINEL,
    );
    let adapter = https::new_for_test_with_resolver(configuration, None, resolver);
    let options = RequestOptions::default()
        .with_read(TimeoutLimit::Bounded(Duration::from_millis(500)))
        .with_total(TimeoutLimit::Bounded(Duration::from_secs(5)));
    let (result_tx, result_rx) = mpsc::sync_channel(1);

    thread::spawn(move || {
        let mut adapter = adapter;
        let result = adapter.exchange_with_options(REQUEST_SENTINEL, 256, &options);
        let _ = result_tx.send((adapter, result));
    });

    partial_sent
        .recv_timeout(Duration::from_secs(3))
        .expect("the peer sends response headers and part of the body");
    let (adapter, result) = result_rx
        .recv_timeout(Duration::from_secs(3))
        .expect("the read inactivity deadline ends the stalled body read");
    let error = result.expect_err("the response body remains incomplete");
    assert_eq!(
        error.delivery_state(),
        RequestDeliveryState::ResponseStarted
    );
    assert_eq!(
        error.cause_category(),
        TransportCauseCategory::Timeout,
        "a Hyper body-read timeout keeps its timeout cause"
    );
    let display = error.to_string();
    let debug = format!("{error:?}");
    for sentinel in [REQUEST_SENTINEL, RESPONSE_SENTINEL] {
        let sentinel = std::str::from_utf8(sentinel).expect("the test sentinels are UTF-8");
        assert!(!display.contains(sentinel), "Display redacts {sentinel}");
        assert!(!debug.contains(sentinel), "Debug redacts {sentinel}");
    }
    assert!(
        !https::has_cached_connection_for_test(&adapter),
        "a connection with a timed-out partial body is invalidated"
    );

    release_peer
        .send(())
        .expect("the stalled peer accepts its bounded cleanup release");
    let peer = peer.join().expect("the partial-body peer exits");
    assert!(peer.handshake_completed);
    assert_eq!(peer.requests.len(), 1);
    assert_eq!(http_request_body(&peer.requests[0]), REQUEST_SENTINEL);
}

const HTTPS_PEER_TIMEOUT: Duration = Duration::from_secs(6);
const HTTPS_SERVER_NAME: &str = "server.kmipkit.test";

struct HttpsResolverGate {
    started: mpsc::Receiver<()>,
    release: mpsc::SyncSender<()>,
    returned: mpsc::Receiver<()>,
}

impl HttpsResolverGate {
    fn wait_until_started(&self) {
        self.started
            .recv_timeout(Duration::from_secs(2))
            .expect("the injected native resolver starts");
    }

    fn release(&self) {
        self.release
            .send(())
            .expect("the held native resolver accepts its release");
    }

    fn wait_until_returned(&self) {
        self.returned
            .recv_timeout(Duration::from_secs(2))
            .expect("the released native resolver returns its late result");
    }
}

fn gated_https_resolver(addresses: Vec<SocketAddr>) -> (resolver::Resolver, HttpsResolverGate) {
    let (started_tx, started_rx) = mpsc::sync_channel(1);
    let (release_tx, release_rx) = mpsc::sync_channel(1);
    let (returned_tx, returned_rx) = mpsc::sync_channel(1);
    let release_rx = Arc::new(Mutex::new(release_rx));
    let release_waiter = Arc::clone(&release_rx);
    let resolver = resolver::Resolver::with_lookup_and_governor(
        move |_host, _port| {
            let _ = started_tx.send(());
            let released = match release_waiter.lock() {
                Ok(receiver) => receiver.recv_timeout(HTTPS_PEER_TIMEOUT).is_ok(),
                Err(_) => false,
            };
            let _ = returned_tx.send(());
            if released {
                Ok(addresses.clone())
            } else {
                Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "test resolver gate expired",
                ))
            }
        },
        Arc::new(tokio::sync::Semaphore::new(1)),
    );
    (
        resolver,
        HttpsResolverGate {
            started: started_rx,
            release: release_tx,
            returned: returned_rx,
        },
    )
}

fn https_resolver(addresses: Vec<SocketAddr>) -> resolver::Resolver {
    resolver::Resolver::with_lookup_and_governor(
        move |_host, _port| Ok(addresses.clone()),
        Arc::new(tokio::sync::Semaphore::new(1)),
    )
}

fn https_client_configuration(pki: &EphemeralPki, port: u16) -> config::TransportConfig {
    https_client_configuration_for_endpoint(
        pki,
        format!("https://{HTTPS_SERVER_NAME}:{port}"),
        None,
    )
}

fn https_client_configuration_for_endpoint(
    pki: &EphemeralPki,
    endpoint: String,
    tls_server_name: Option<&str>,
) -> config::TransportConfig {
    let client_chain = pki
        .client_identity()
        .certificate_chain_der()
        .into_iter()
        .map(<[u8]>::to_vec)
        .collect();
    let builder = config::TransportConfig::builder(config::Endpoint::https(endpoint));
    let builder = if let Some(tls_server_name) = tls_server_name {
        builder.tls_server_name(tls_server_name)
    } else {
        builder
    };
    builder
        .client_identity(config::ClientIdentity::new(
            config::CertificateInput::from_der(client_chain),
            config::PrivateKeyInput::from_der(pki.client_identity().private_key_der().to_vec()),
        ))
        .trust_source(config::TrustSource::certificate_authorities(vec![
            config::CertificateInput::from_der(vec![pki.authority_certificate_der().to_vec()]),
        ]))
        .build()
        .expect("the client identity and server trust inputs build an HTTPS config")
}

fn https_server_config(server_pki: &EphemeralPki, client_pki: &EphemeralPki) -> Arc<ServerConfig> {
    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    let builder = ServerConfig::builder_with_provider(provider)
        .with_protocol_versions(&[&rustls::version::TLS13])
        .expect("the local HTTPS server uses TLS 1.3 only");
    let mut client_roots = RootCertStore::empty();
    client_roots
        .add(CertificateDer::from(
            client_pki.authority_certificate_der().to_vec(),
        ))
        .expect("the client test CA is valid");
    let verifier = WebPkiClientVerifier::builder(Arc::new(client_roots))
        .build()
        .expect("the HTTPS peer requires a verified client certificate");
    let config = builder
        .with_client_cert_verifier(verifier)
        .with_single_cert(
            vec![CertificateDer::from(
                server_pki.server_identity().certificate_der().to_vec(),
            )],
            PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(
                server_pki.server_identity().private_key_der().to_vec(),
            )),
        )
        .expect("the local server identity is valid");
    Arc::new(config)
}

#[derive(Default)]
struct HttpsPeerObservation {
    handshake_completed: bool,
    requests: Vec<Vec<u8>>,
}

fn spawn_https_peer(
    listener: TcpListener,
    configuration: Arc<ServerConfig>,
    response: Option<Vec<u8>>,
    handshake_gate: Option<mpsc::Receiver<()>>,
) -> (JoinHandle<HttpsPeerObservation>, mpsc::Receiver<()>) {
    let (connected_tx, connected_rx) = mpsc::sync_channel(1);
    let peer = thread::spawn(move || {
        let Ok((stream, _)) = accept_https_connection(&listener) else {
            return HttpsPeerObservation::default();
        };
        let _ = connected_tx.send(());
        if let Some(handshake_gate) = handshake_gate
            && handshake_gate.recv_timeout(HTTPS_PEER_TIMEOUT).is_err()
        {
            return HttpsPeerObservation::default();
        }
        let Ok(mut tls) = finish_https_server_handshake(stream, configuration) else {
            return HttpsPeerObservation::default();
        };
        let mut observation = HttpsPeerObservation {
            handshake_completed: true,
            requests: Vec::new(),
        };
        let Ok(request) = read_https_request(&mut tls) else {
            return observation;
        };
        observation.requests.push(request);
        if let Some(response) = response {
            let _ = tls.write_all(&response);
            let _ = tls.flush();
        }
        observation
    });
    (peer, connected_rx)
}

fn accept_https_connection(listener: &TcpListener) -> io::Result<(TcpStream, SocketAddr)> {
    listener.set_nonblocking(true)?;
    let deadline = Instant::now() + HTTPS_PEER_TIMEOUT;
    loop {
        match listener.accept() {
            Ok(connection) => return Ok(connection),
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                if Instant::now() >= deadline {
                    return Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        "HTTPS test peer accept deadline expired",
                    ));
                }
                thread::sleep(Duration::from_millis(5));
            }
            Err(error) => return Err(error),
        }
    }
}

fn finish_https_server_handshake(
    stream: TcpStream,
    configuration: Arc<ServerConfig>,
) -> io::Result<StreamOwned<ServerConnection, TcpStream>> {
    stream.set_nonblocking(false)?;
    stream.set_read_timeout(Some(HTTPS_PEER_TIMEOUT))?;
    stream.set_write_timeout(Some(HTTPS_PEER_TIMEOUT))?;
    let connection = ServerConnection::new(configuration).map_err(io::Error::other)?;
    let mut tls = StreamOwned::new(connection, stream);
    let deadline = Instant::now() + HTTPS_PEER_TIMEOUT;
    while tls.conn.is_handshaking() {
        if Instant::now() >= deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "HTTPS test TLS handshake deadline expired",
            ));
        }
        tls.conn
            .complete_io(&mut tls.sock)
            .map_err(io::Error::other)?;
    }
    Ok(tls)
}

fn read_https_request(reader: &mut impl Read) -> io::Result<Vec<u8>> {
    const MAX_REQUEST: usize = 1024 * 1024;
    let mut bytes = Vec::new();
    let mut chunk = [0; 1024];
    loop {
        if let Some(expected_len) = https_request_wire_length(&bytes)?
            && bytes.len() >= expected_len
        {
            bytes.truncate(expected_len);
            return Ok(bytes);
        }
        if bytes.len() >= MAX_REQUEST {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "HTTPS test request exceeds its bound",
            ));
        }
        let count = reader.read(&mut chunk)?;
        if count == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "HTTPS test peer observed an incomplete request",
            ));
        }
        bytes.extend_from_slice(&chunk[..count]);
    }
}

fn https_request_wire_length(bytes: &[u8]) -> io::Result<Option<usize>> {
    let Some(header_end) = https_header_end(bytes) else {
        return Ok(None);
    };
    let headers = String::from_utf8_lossy(&bytes[..header_end]);
    let content_length = headers
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse::<usize>().ok())
                .flatten()
        })
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Content-Length is missing"))?;
    header_end
        .checked_add(content_length)
        .map(Some)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "request length overflow"))
}

fn https_header_end(bytes: &[u8]) -> Option<usize> {
    bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|i| i + 4)
}

fn http_request_body(request: &[u8]) -> &[u8] {
    let body_start = https_header_end(request).expect("the captured request has HTTP headers");
    &request[body_start..]
}

fn complete_http_response(body: &[u8]) -> Vec<u8> {
    let mut response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .into_bytes();
    response.extend_from_slice(body);
    response
}

fn spawn_stalled_tls_peer(
    listener: TcpListener,
) -> (
    JoinHandle<StalledTlsObservation>,
    mpsc::Receiver<()>,
    mpsc::SyncSender<()>,
) {
    let (accepted_tx, accepted_rx) = mpsc::sync_channel(1);
    let (release_tx, release_rx) = mpsc::sync_channel(1);
    let peer = thread::spawn(move || {
        let Ok((mut stream, _)) = accept_https_connection(&listener) else {
            return StalledTlsObservation::default();
        };
        let _ = accepted_tx.send(());
        let _ = stream.set_nonblocking(false);
        let _ = stream.set_read_timeout(Some(HTTPS_PEER_TIMEOUT));
        let mut record_prefix = [0; 5];
        let first_tls_record_type = match stream.read(&mut record_prefix) {
            Ok(count) if count > 0 => Some(record_prefix[0]),
            _ => None,
        };
        let _ = release_rx.recv_timeout(HTTPS_PEER_TIMEOUT);
        StalledTlsObservation {
            first_tls_record_type,
            handshake_completed: false,
            http_request: None,
        }
    });
    (peer, accepted_rx, release_tx)
}

#[derive(Default)]
struct StalledTlsObservation {
    first_tls_record_type: Option<u8>,
    handshake_completed: bool,
    http_request: Option<Vec<u8>>,
}

fn assert_no_tcp_connection_for(listener: &TcpListener, duration: Duration) {
    listener
        .set_nonblocking(true)
        .expect("the resolver late-result probe is nonblocking");
    let deadline = Instant::now() + duration;
    while Instant::now() < deadline {
        match listener.accept() {
            Ok((stream, _)) => {
                drop(stream);
                panic!("a TCP candidate started after the resolver deadline");
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(5));
            }
            Err(error) => panic!("the TCP probe failed: {error}"),
        }
    }
}

#[derive(Default)]
struct ReconnectPeerObservation {
    accepted_connections: usize,
    requests: Vec<Vec<u8>>,
}

fn spawn_partial_then_success_https_peer(
    listener: TcpListener,
    configuration: Arc<ServerConfig>,
) -> (
    JoinHandle<ReconnectPeerObservation>,
    mpsc::Receiver<()>,
    mpsc::SyncSender<()>,
) {
    let (partial_sent_tx, partial_sent_rx) = mpsc::sync_channel(1);
    let (release_peer_tx, release_peer_rx) = mpsc::sync_channel(1);
    let peer = thread::spawn(move || {
        let mut observation = ReconnectPeerObservation::default();
        let Ok((first_stream, _)) = accept_https_connection(&listener) else {
            return observation;
        };
        observation.accepted_connections += 1;
        {
            let Ok(mut tls) =
                finish_https_server_handshake(first_stream, Arc::clone(&configuration))
            else {
                return observation;
            };
            let Ok(request) = read_https_request(&mut tls) else {
                return observation;
            };
            observation.requests.push(request);
            let partial_response = b"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: 8\r\n\r\npart";
            if tls.write_all(partial_response).is_err() || tls.flush().is_err() {
                return observation;
            }
            let _ = partial_sent_tx.send(());
            let _ = release_peer_rx.recv_timeout(HTTPS_PEER_TIMEOUT);
        }

        let Ok((second_stream, _)) = accept_https_connection(&listener) else {
            return observation;
        };
        observation.accepted_connections += 1;
        let Ok(mut tls) = finish_https_server_handshake(second_stream, configuration) else {
            return observation;
        };
        let Ok(request) = read_https_request(&mut tls) else {
            return observation;
        };
        observation.requests.push(request);
        let _ = tls.write_all(&complete_http_response(b"done"));
        let _ = tls.flush();
        observation
    });
    (peer, partial_sent_rx, release_peer_tx)
}

fn spawn_stalled_partial_body_https_peer(
    listener: TcpListener,
    configuration: Arc<ServerConfig>,
    body_prefix: &'static [u8],
) -> (
    JoinHandle<HttpsPeerObservation>,
    mpsc::Receiver<()>,
    mpsc::SyncSender<()>,
) {
    let (partial_sent_tx, partial_sent_rx) = mpsc::sync_channel(1);
    let (release_peer_tx, release_peer_rx) = mpsc::sync_channel(1);
    let peer = thread::spawn(move || {
        let Ok((stream, _)) = accept_https_connection(&listener) else {
            return HttpsPeerObservation::default();
        };
        let Ok(mut tls) = finish_https_server_handshake(stream, configuration) else {
            return HttpsPeerObservation::default();
        };
        let mut observation = HttpsPeerObservation {
            handshake_completed: true,
            requests: Vec::new(),
        };
        let Ok(request) = read_https_request(&mut tls) else {
            return observation;
        };
        observation.requests.push(request);

        let declared_length = body_prefix.len() + 8;
        let mut partial_response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {declared_length}\r\n\r\n"
        )
        .into_bytes();
        partial_response.extend_from_slice(body_prefix);
        if tls.write_all(&partial_response).is_err() || tls.flush().is_err() {
            return observation;
        }
        let _ = partial_sent_tx.send(());
        let _ = release_peer_rx.recv_timeout(HTTPS_PEER_TIMEOUT);
        observation
    });
    (peer, partial_sent_rx, release_peer_tx)
}

async fn read_http_headers<R: AsyncRead + Unpin>(reader: &mut R) -> Vec<u8> {
    let mut bytes = Vec::new();
    loop {
        let byte = reader
            .read_u8()
            .await
            .expect("the HTTP/1 peer keeps the connection open for request headers");
        bytes.push(byte);
        if bytes.ends_with(b"\r\n\r\n") {
            return bytes;
        }
    }
}

fn error_chain_has_io_kind(error: &(dyn StdError + 'static), kind: io::ErrorKind) -> bool {
    let mut source = Some(error);
    while let Some(current) = source {
        if current
            .downcast_ref::<io::Error>()
            .is_some_and(|source| source.kind() == kind)
        {
            return true;
        }
        source = current.source();
    }
    false
}

fn request(body: impl Into<TestBody>) -> Request<TestBody> {
    Request::builder()
        .method("POST")
        .uri("/kmip")
        .header("host", "kmip.test")
        .body(body.into())
        .expect("the fixed test request is valid")
}

struct EmptyBody;

struct OneChunkBody(Option<Bytes>);

impl OneChunkBody {
    fn new(bytes: &'static [u8]) -> Self {
        Self(Some(Bytes::from_static(bytes)))
    }
}

enum TestBody {
    Empty(EmptyBody),
    OneChunk(OneChunkBody),
}

impl From<EmptyBody> for TestBody {
    fn from(body: EmptyBody) -> Self {
        Self::Empty(body)
    }
}

impl From<OneChunkBody> for TestBody {
    fn from(body: OneChunkBody) -> Self {
        Self::OneChunk(body)
    }
}

impl Body for TestBody {
    type Data = Bytes;
    type Error = Infallible;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        match &mut *self {
            Self::Empty(_) => Poll::Ready(None),
            Self::OneChunk(body) => Poll::Ready(body.0.take().map(|bytes| Ok(Frame::data(bytes)))),
        }
    }

    fn is_end_stream(&self) -> bool {
        matches!(self, Self::Empty(_) | Self::OneChunk(OneChunkBody(None)))
    }

    fn size_hint(&self) -> SizeHint {
        match self {
            Self::Empty(_) | Self::OneChunk(OneChunkBody(None)) => SizeHint::with_exact(0),
            Self::OneChunk(OneChunkBody(Some(bytes))) => SizeHint::with_exact(bytes.len() as u64),
        }
    }
}

#[derive(Clone)]
struct ScriptIo {
    state: Arc<Mutex<ScriptState>>,
}

struct ScriptState {
    read_chunks: std::collections::VecDeque<Vec<u8>>,
    write_permits: std::collections::VecDeque<usize>,
    flush_permits: usize,
    write_mode: WriteMode,
    written: Vec<u8>,
    vectored_write_calls: usize,
    waker: Option<Waker>,
    read_poll_notification: Option<tokio::sync::oneshot::Sender<()>>,
    positive_read_notification: Option<tokio::sync::oneshot::Sender<()>>,
    first_write_notification: Option<tokio::sync::oneshot::Sender<()>>,
}

#[derive(Clone, Copy)]
enum WriteMode {
    Manual,
    Automatic,
    FailImmediately,
    BlockAfterHeaders,
    FailAfterBodyBytes(usize),
}

impl ScriptIo {
    fn manual() -> Self {
        Self::new(WriteMode::Manual)
    }

    fn automatic_with_exchange_notifications() -> (
        Self,
        tokio::sync::oneshot::Receiver<()>,
        tokio::sync::oneshot::Receiver<()>,
        tokio::sync::oneshot::Receiver<()>,
    ) {
        let (read_poll_notification, first_read_poll) = tokio::sync::oneshot::channel();
        let (first_write_notification, first_write) = tokio::sync::oneshot::channel();
        let (positive_read_notification, positive_read) = tokio::sync::oneshot::channel();
        let io = Self::new(WriteMode::Automatic);
        {
            let mut state = io.state.lock().expect("script state mutex is not poisoned");
            state.read_poll_notification = Some(read_poll_notification);
            state.first_write_notification = Some(first_write_notification);
            state.positive_read_notification = Some(positive_read_notification);
        }
        (io, first_read_poll, first_write, positive_read)
    }

    fn failing_immediately() -> Self {
        Self::new(WriteMode::FailImmediately)
    }

    fn blocked_after_headers() -> Self {
        Self::new(WriteMode::BlockAfterHeaders)
    }

    fn failing_after_body_bytes(limit: usize) -> Self {
        Self::new(WriteMode::FailAfterBodyBytes(limit))
    }

    fn new(write_mode: WriteMode) -> Self {
        Self {
            state: Arc::new(Mutex::new(ScriptState {
                read_chunks: std::collections::VecDeque::new(),
                write_permits: std::collections::VecDeque::new(),
                flush_permits: 0,
                write_mode,
                written: Vec::new(),
                vectored_write_calls: 0,
                waker: None,
                read_poll_notification: None,
                positive_read_notification: None,
                first_write_notification: None,
            })),
        }
    }

    fn release_read(&self, bytes: Vec<u8>) {
        self.update(|state| state.read_chunks.push_back(bytes));
    }

    fn release_write(&self, bytes: usize) {
        self.update(|state| state.write_permits.push_back(bytes));
    }

    fn release_flush(&self) {
        self.update(|state| state.flush_permits += 1);
    }

    fn written_bytes(&self) -> Vec<u8> {
        self.state
            .lock()
            .expect("script state mutex is not poisoned")
            .written
            .clone()
    }

    fn vectored_write_calls(&self) -> usize {
        self.state
            .lock()
            .expect("script state mutex is not poisoned")
            .vectored_write_calls
    }

    fn queued_read_bytes(&self) -> usize {
        self.state
            .lock()
            .expect("script state mutex is not poisoned")
            .read_chunks
            .iter()
            .map(Vec::len)
            .sum()
    }

    fn update(&self, update: impl FnOnce(&mut ScriptState)) {
        let waker = {
            let mut state = self
                .state
                .lock()
                .expect("script state mutex is not poisoned");
            update(&mut state);
            state.waker.take()
        };
        if let Some(waker) = waker {
            waker.wake();
        }
    }
}

impl AsyncRead for ScriptIo {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut TokioReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let mut state = self
            .state
            .lock()
            .expect("script state mutex is not poisoned");
        if let Some(notification) = state.read_poll_notification.take() {
            let _ = notification.send(());
        }
        if let Some(mut chunk) = state.read_chunks.pop_front() {
            let read = chunk.len().min(buf.remaining());
            buf.put_slice(&chunk[..read]);
            let positive_read_notification = if read > 0 {
                state.positive_read_notification.take()
            } else {
                None
            };
            if read < chunk.len() {
                chunk.drain(..read);
                state.read_chunks.push_front(chunk);
            }
            drop(state);
            if let Some(notification) = positive_read_notification {
                let _ = notification.send(());
            }
            Poll::Ready(Ok(()))
        } else {
            state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl AsyncWrite for ScriptIo {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.write_bytes(cx, buf, false)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let mut state = self
            .state
            .lock()
            .expect("script state mutex is not poisoned");
        if state.flush_permits > 0 {
            state.flush_permits -= 1;
            Poll::Ready(Ok(()))
        } else if matches!(state.write_mode, WriteMode::Automatic) {
            Poll::Ready(Ok(()))
        } else {
            state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn is_write_vectored(&self) -> bool {
        true
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[IoSlice<'_>],
    ) -> Poll<io::Result<usize>> {
        let bytes = bufs
            .iter()
            .flat_map(|part| part.iter().copied())
            .collect::<Vec<_>>();
        self.write_bytes(cx, &bytes, true)
    }
}

impl ScriptIo {
    fn write_bytes(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bytes: &[u8],
        vectored: bool,
    ) -> Poll<io::Result<usize>> {
        if bytes.is_empty() {
            return Poll::Ready(Ok(0));
        }
        let mut state = self
            .state
            .lock()
            .expect("script state mutex is not poisoned");
        if vectored {
            state.vectored_write_calls += 1;
        }
        let writable = match state.write_mode {
            WriteMode::Manual => state
                .write_permits
                .pop_front()
                .map(|limit| limit.min(bytes.len())),
            WriteMode::Automatic => Some(bytes.len()),
            WriteMode::FailImmediately => {
                return Poll::Ready(Err(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "scripted immediate write failure",
                )));
            }
            WriteMode::BlockAfterHeaders => Some(
                bytes_before_body(&state.written, bytes)
                    .unwrap_or(bytes.len())
                    .min(bytes.len()),
            ),
            WriteMode::FailAfterBodyBytes(limit) => {
                let existing_header_end = header_end(&state.written);
                let current_len = state.written.len();
                let completed_header_end = existing_header_end.or_else(|| {
                    state
                        .written
                        .iter()
                        .chain(bytes.iter())
                        .copied()
                        .collect::<Vec<_>>()
                        .windows(4)
                        .position(|window| window == b"\r\n\r\n")
                        .map(|position| position + 4)
                });
                match completed_header_end {
                    Some(end) if current_len >= end => {
                        let sent_body = current_len.saturating_sub(end);
                        if sent_body >= limit {
                            return Poll::Ready(Err(io::Error::new(
                                io::ErrorKind::BrokenPipe,
                                "scripted partial body write failure",
                            )));
                        }
                        Some((limit - sent_body).min(bytes.len()))
                    }
                    Some(end) => {
                        let header_bytes = end.saturating_sub(current_len);
                        Some((header_bytes + limit).min(bytes.len()))
                    }
                    None => Some(bytes.len()),
                }
            }
        };

        match writable {
            Some(0) if matches!(state.write_mode, WriteMode::BlockAfterHeaders) => {
                state.waker = Some(cx.waker().clone());
                Poll::Pending
            }
            Some(0) => Poll::Ready(Err(io::Error::new(
                io::ErrorKind::WriteZero,
                "scripted writer made no progress",
            ))),
            Some(written) => {
                state.written.extend_from_slice(&bytes[..written]);
                let first_write_notification = state.first_write_notification.take();
                drop(state);
                if let Some(notification) = first_write_notification {
                    let _ = notification.send(());
                }
                Poll::Ready(Ok(written))
            }
            None => {
                state.waker = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}

fn header_end(bytes: &[u8]) -> Option<usize> {
    bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|position| position + 4)
}

fn bytes_before_body(written: &[u8], incoming: &[u8]) -> Option<usize> {
    if header_end(written).is_some() {
        return Some(0);
    }
    let mut combined = Vec::with_capacity(written.len() + incoming.len());
    combined.extend_from_slice(written);
    combined.extend_from_slice(incoming);
    header_end(&combined).map(|end| end.saturating_sub(written.len()))
}
