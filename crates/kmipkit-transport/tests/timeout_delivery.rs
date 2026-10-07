//! Production transport timeout and HTTP delivery evidence tests.
//!
//! These tests include the real private timeout and worker state machines so
//! they can verify deadlines, cancellation, dispatch, response observation,
//! and finalization without exporting an internal test seam.

use std::convert::Infallible;
use std::error::Error as StdError;
use std::io::{self, IoSlice};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant};

use bytes::Bytes;
use hyper::body::{Body, Frame, SizeHint};
use hyper::client::conn::http1;
use hyper::http::Request;
use hyper::rt::{Read as HyperRead, ReadBuf as HyperReadBuf, Write as HyperWrite};
use kmipkit_transport::{
    RequestDeliveryState, TransportCauseCategory, TransportError, TransportResponse,
};
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

    let error = poll_fn(|cx| Pin::new(&mut io).poll_write_vectored(cx, &[first, second]))
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
