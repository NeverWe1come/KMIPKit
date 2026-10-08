//! Private monotonic I/O deadlines and request-delivery coordination.

use std::future::{Future, poll_fn};
use std::io::{self, IoSlice};
use std::pin::Pin;
use std::sync::{Arc, Mutex, MutexGuard};
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

use hyper::body::{Body, Incoming};
use hyper::client::conn::http1;
use hyper::http::Request;
use hyper::rt::{Read as HyperRead, ReadBufCursor as HyperReadBufCursor, Write as HyperWrite};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf as TokioReadBuf};
use tokio::time::{Instant as TokioInstant, Sleep};
use zeroize::{Zeroize, Zeroizing};

use crate::RequestDeliveryState;
use crate::worker::ExchangeControl;

const READ_BUFFER_SIZE: usize = 16 * 1024;

/// TLS I/O with independent inactivity limits and an absolute exchange limit.
pub(crate) struct DeadlineIo<I> {
    inner: Pin<Box<I>>,
    state: Arc<Mutex<DeadlineIoState>>,
}

/// Mutable per-exchange deadline and delivery state for a persistent HTTP/1 connection.
#[allow(dead_code)] // timeout_delivery source-includes this module without the HTTPS adapter.
#[derive(Clone)]
pub(crate) struct DeadlineIoControl {
    state: Arc<Mutex<DeadlineIoState>>,
}

enum DeadlineIoState {
    Active(ActiveDeadlineIo),
    #[allow(dead_code)] // Constructed by HTTPS after a successful persistent exchange.
    Idle,
    Invalid,
}

struct ActiveDeadlineIo {
    read_deadline: PhaseDeadline,
    write_deadline: PhaseDeadline,
    total_deadline: Option<Instant>,
    control: ExchangeControl,
}

impl<I> DeadlineIo<I> {
    pub(crate) fn new(
        inner: I,
        read_timeout: Option<Duration>,
        write_timeout: Option<Duration>,
        total_deadline: Option<Instant>,
        control: ExchangeControl,
    ) -> Self {
        Self {
            inner: Box::pin(inner),
            state: Arc::new(Mutex::new(DeadlineIoState::Active(ActiveDeadlineIo::new(
                read_timeout,
                write_timeout,
                total_deadline,
                control,
            )))),
        }
    }

    /// Creates a deadline-aware I/O stream whose exchange context can be renewed between requests.
    #[allow(dead_code)] // Used by HTTPS; timeout_delivery source-includes this module alone.
    pub(crate) fn new_reusable(
        inner: I,
        read_timeout: Option<Duration>,
        write_timeout: Option<Duration>,
        total_deadline: Option<Instant>,
        control: ExchangeControl,
    ) -> (Self, DeadlineIoControl) {
        let state = Arc::new(Mutex::new(DeadlineIoState::Active(ActiveDeadlineIo::new(
            read_timeout,
            write_timeout,
            total_deadline,
            control,
        ))));
        (
            Self {
                inner: Box::pin(inner),
                state: Arc::clone(&state),
            },
            DeadlineIoControl { state },
        )
    }

    #[allow(dead_code)] // The raw TLS source-including target validates before dispatch.
    pub(crate) fn validate_phase_deadlines(&self) -> io::Result<()> {
        match &*lock_deadline_state(&self.state) {
            DeadlineIoState::Active(active)
                if active.read_deadline.invalid || active.write_deadline.invalid =>
            {
                Err(invalid_deadline_io_error())
            }
            DeadlineIoState::Active(_) => Ok(()),
            DeadlineIoState::Idle | DeadlineIoState::Invalid => Err(invalid_deadline_io_error()),
        }
    }
}

impl ActiveDeadlineIo {
    fn new(
        read_timeout: Option<Duration>,
        write_timeout: Option<Duration>,
        total_deadline: Option<Instant>,
        control: ExchangeControl,
    ) -> Self {
        Self {
            read_deadline: PhaseDeadline::new(read_timeout),
            write_deadline: PhaseDeadline::new(write_timeout),
            total_deadline,
            control,
        }
    }
}

#[allow(dead_code)] // Used by HTTPS; timeout_delivery source-includes this module alone.
impl DeadlineIoControl {
    /// Starts a new serialized exchange on an idle persistent connection.
    pub(crate) fn begin_exchange(
        &self,
        read_timeout: Option<Duration>,
        write_timeout: Option<Duration>,
        total_deadline: Option<Instant>,
        control: ExchangeControl,
    ) -> io::Result<()> {
        let mut state = lock_deadline_state(&self.state);
        match &*state {
            DeadlineIoState::Idle => {
                let active =
                    ActiveDeadlineIo::new(read_timeout, write_timeout, total_deadline, control);
                if active.read_deadline.invalid || active.write_deadline.invalid {
                    return Err(invalid_deadline_io_error());
                }
                *state = DeadlineIoState::Active(active);
                Ok(())
            }
            DeadlineIoState::Invalid => Err(unsolicited_response_io_error()),
            DeadlineIoState::Active(_) => Err(io::Error::other(
                "HTTP connection already has an exchange in flight",
            )),
        }
    }

    /// Stops applying an exchange's delivery state and deadlines after its full response arrives.
    pub(crate) fn finish_exchange(&self) {
        let mut state = lock_deadline_state(&self.state);
        if matches!(*state, DeadlineIoState::Active(_)) {
            *state = DeadlineIoState::Idle;
        }
    }

    /// Prevents a connection with malformed or unsolicited input from being reused.
    pub(crate) fn invalidate(&self) {
        *lock_deadline_state(&self.state) = DeadlineIoState::Invalid;
    }

    /// Reports whether the persistent I/O observed bytes outside an active exchange.
    pub(crate) fn is_invalid(&self) -> bool {
        matches!(*lock_deadline_state(&self.state), DeadlineIoState::Invalid)
    }
}

fn lock_deadline_state(state: &Mutex<DeadlineIoState>) -> MutexGuard<'_, DeadlineIoState> {
    state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

impl<I> HyperRead for DeadlineIo<I>
where
    I: AsyncRead + AsyncWrite,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut buffer: HyperReadBufCursor<'_>,
    ) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        let capacity = buffer.remaining().min(READ_BUFFER_SIZE);
        if capacity == 0 {
            return Poll::Ready(Ok(()));
        }

        let mut scratch = [0_u8; READ_BUFFER_SIZE];
        let mut inner_buffer = TokioReadBuf::new(&mut scratch[..capacity]);
        let read_result = {
            let (inner, state) = (&mut this.inner, &this.state);
            let mut state = lock_deadline_state(state);
            match &mut *state {
                DeadlineIoState::Active(active) => {
                    if active.control.is_finalized() {
                        Poll::Ready(Err(finalized_io_error()))
                    } else {
                        let result = poll_with_deadline(
                            &mut active.read_deadline,
                            active.total_deadline,
                            cx,
                            |cx| inner.as_mut().poll_read(cx, &mut inner_buffer),
                        );
                        if let Poll::Ready(Ok(())) = result {
                            if inner_buffer.filled().is_empty() {
                                result
                            } else if active.control.observe_response_byte() {
                                active.read_deadline.reset_after_progress();
                                result
                            } else {
                                *state = DeadlineIoState::Invalid;
                                Poll::Ready(Err(finalized_io_error()))
                            }
                        } else {
                            result
                        }
                    }
                }
                DeadlineIoState::Idle => match inner.as_mut().poll_read(cx, &mut inner_buffer) {
                    Poll::Ready(Ok(())) if !inner_buffer.filled().is_empty() => {
                        *state = DeadlineIoState::Invalid;
                        Poll::Ready(Err(unsolicited_response_io_error()))
                    }
                    Poll::Ready(Err(error)) => {
                        *state = DeadlineIoState::Invalid;
                        Poll::Ready(Err(sanitize_io_error(error)))
                    }
                    result => result,
                },
                DeadlineIoState::Invalid => Poll::Ready(Err(unsolicited_response_io_error())),
            }
        };

        match read_result {
            Poll::Ready(Ok(())) => {
                let bytes_read = inner_buffer.filled().len();
                if bytes_read == 0 {
                    scratch.zeroize();
                    return Poll::Ready(Ok(()));
                }

                buffer.put_slice(&scratch[..bytes_read]);
                scratch.zeroize();
                Poll::Ready(Ok(()))
            }
            Poll::Ready(Err(error)) => {
                scratch.zeroize();
                Poll::Ready(Err(sanitize_io_error(error)))
            }
            Poll::Pending => {
                scratch.zeroize();
                Poll::Pending
            }
        }
    }
}

// Tokio's extension traits are convenient for the raw-TLS adapter. Forward
// their polls through the Hyper traits so both adapters share one deadline
// state and the same response-delivery observation path.
impl<I> AsyncRead for DeadlineIo<I>
where
    I: AsyncRead + AsyncWrite,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buffer: &mut TokioReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        let capacity = buffer.remaining().min(READ_BUFFER_SIZE);
        if capacity == 0 {
            return Poll::Ready(Ok(()));
        }

        let mut scratch = Zeroizing::new([0_u8; READ_BUFFER_SIZE]);
        let mut hyper_buffer = hyper::rt::ReadBuf::new(&mut scratch[..capacity]);
        match HyperRead::poll_read(Pin::new(this), cx, hyper_buffer.unfilled()) {
            Poll::Ready(Ok(())) => {
                let bytes_read = hyper_buffer.filled().len();
                if bytes_read > 0 {
                    buffer.put_slice(&scratch[..bytes_read]);
                }
                Poll::Ready(Ok(()))
            }
            Poll::Ready(Err(error)) => Poll::Ready(Err(error)),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<I> HyperWrite for DeadlineIo<I>
where
    I: AsyncRead + AsyncWrite,
{
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bytes: &[u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();
        if bytes.is_empty() {
            return Poll::Ready(Ok(0));
        }
        let result = {
            let (inner, state) = (&mut this.inner, &this.state);
            let mut state = lock_deadline_state(state);
            match &mut *state {
                DeadlineIoState::Active(active) if !active.control.is_finalized() => {
                    let result = poll_with_deadline(
                        &mut active.write_deadline,
                        active.total_deadline,
                        cx,
                        |cx| inner.as_mut().poll_write(cx, bytes),
                    );
                    if matches!(result, Poll::Ready(Ok(written)) if written > 0) {
                        active.write_deadline.reset_after_progress();
                    }
                    result
                }
                DeadlineIoState::Active(_) => Poll::Ready(Err(finalized_io_error())),
                DeadlineIoState::Idle => {
                    *state = DeadlineIoState::Invalid;
                    Poll::Ready(Err(unsolicited_response_io_error()))
                }
                DeadlineIoState::Invalid => Poll::Ready(Err(unsolicited_response_io_error())),
            }
        };
        match result {
            Poll::Ready(Ok(written)) => Poll::Ready(Ok(written)),
            Poll::Ready(Err(error)) => Poll::Ready(Err(sanitize_io_error(error))),
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        let result = {
            let (inner, state) = (&mut this.inner, &this.state);
            let mut state = lock_deadline_state(state);
            match &mut *state {
                DeadlineIoState::Active(active) if !active.control.is_finalized() => {
                    let result = poll_with_deadline(
                        &mut active.write_deadline,
                        active.total_deadline,
                        cx,
                        |cx| inner.as_mut().poll_flush(cx),
                    );
                    if matches!(result, Poll::Ready(Ok(()))) {
                        active.write_deadline.reset_after_progress();
                    }
                    result
                }
                DeadlineIoState::Active(_) => Poll::Ready(Err(finalized_io_error())),
                DeadlineIoState::Idle => match inner.as_mut().poll_flush(cx) {
                    Poll::Ready(Err(error)) => {
                        *state = DeadlineIoState::Invalid;
                        Poll::Ready(Err(sanitize_io_error(error)))
                    }
                    result => result,
                },
                DeadlineIoState::Invalid => Poll::Ready(Err(unsolicited_response_io_error())),
            }
        };
        match result {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(error)) => Poll::Ready(Err(sanitize_io_error(error))),
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        let result = {
            let (inner, state) = (&mut this.inner, &this.state);
            let mut state = lock_deadline_state(state);
            match &mut *state {
                DeadlineIoState::Active(active) => poll_with_deadline(
                    &mut active.write_deadline,
                    active.total_deadline,
                    cx,
                    |cx| inner.as_mut().poll_shutdown(cx),
                ),
                DeadlineIoState::Idle => {
                    *state = DeadlineIoState::Invalid;
                    inner.as_mut().poll_shutdown(cx)
                }
                DeadlineIoState::Invalid => Poll::Ready(Err(unsolicited_response_io_error())),
            }
        };
        match result {
            Poll::Ready(Err(error)) => Poll::Ready(Err(sanitize_io_error(error))),
            other => other,
        }
    }

    fn is_write_vectored(&self) -> bool {
        self.inner.is_write_vectored()
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buffers: &[IoSlice<'_>],
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();
        if buffers.iter().all(|buffer| buffer.is_empty()) {
            return Poll::Ready(Ok(0));
        }
        let result = {
            let (inner, state) = (&mut this.inner, &this.state);
            let mut state = lock_deadline_state(state);
            match &mut *state {
                DeadlineIoState::Active(active) if !active.control.is_finalized() => {
                    let result = poll_with_deadline(
                        &mut active.write_deadline,
                        active.total_deadline,
                        cx,
                        |cx| inner.as_mut().poll_write_vectored(cx, buffers),
                    );
                    if matches!(result, Poll::Ready(Ok(written)) if written > 0) {
                        active.write_deadline.reset_after_progress();
                    }
                    result
                }
                DeadlineIoState::Active(_) => Poll::Ready(Err(finalized_io_error())),
                DeadlineIoState::Idle => {
                    *state = DeadlineIoState::Invalid;
                    Poll::Ready(Err(unsolicited_response_io_error()))
                }
                DeadlineIoState::Invalid => Poll::Ready(Err(unsolicited_response_io_error())),
            }
        };
        match result {
            Poll::Ready(Ok(written)) => Poll::Ready(Ok(written)),
            Poll::Ready(Err(error)) => Poll::Ready(Err(sanitize_io_error(error))),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<I> AsyncWrite for DeadlineIo<I>
where
    I: AsyncRead + AsyncWrite,
{
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buffer: &[u8],
    ) -> Poll<io::Result<usize>> {
        HyperWrite::poll_write(self, cx, buffer)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        HyperWrite::poll_flush(self, cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        HyperWrite::poll_shutdown(self, cx)
    }
}

/// Bounds a sender-readiness wait by both write inactivity and total time.
pub(crate) async fn wait_for_sender_ready<B>(
    sender: &mut http1::SendRequest<B>,
    write_timeout: Option<Duration>,
    total_deadline: Option<Instant>,
) -> io::Result<()> {
    let mut phase_deadline = PhaseDeadline::new(write_timeout);
    let mut readiness = Box::pin(sender.ready());
    poll_fn(|cx| {
        poll_with_deadline(
            &mut phase_deadline,
            total_deadline,
            cx,
            |cx| match readiness.as_mut().poll(cx) {
                Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
                Poll::Ready(Err(_)) => Poll::Ready(Err(sender_closed_error())),
                Poll::Pending => Poll::Pending,
            },
        )
    })
    .await
}

/// Waits for sender readiness, atomically commits dispatch, then hands the
/// retained request to Hyper. A pre-commit failure leaves the request intact.
pub(crate) async fn send_request_when_ready<B>(
    sender: &mut http1::SendRequest<B>,
    request: &mut Option<Request<B>>,
    control: &ExchangeControl,
    write_timeout: Option<Duration>,
    total_deadline: Option<Instant>,
) -> io::Result<hyper::Response<Incoming>>
where
    B: Body + 'static,
    B::Data: Send,
    B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    if request.is_none() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "HTTP request is unavailable",
        ));
    }
    if control.is_finalized() {
        return Err(finalized_io_error());
    }

    if let Err(error) = wait_for_sender_ready(sender, write_timeout, total_deadline).await {
        control.cancel();
        return Err(error);
    }
    if !control.commit_dispatch() {
        return Err(finalized_io_error());
    }

    // The exclusive borrow across readiness keeps callers from clearing this slot;
    // retain a safe fallback if that invariant changes with a future signature.
    let Some(request) = request.take() else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "HTTP request is unavailable",
        ));
    };
    sender.send_request(request).await.map_err(io::Error::other)
}

/// Closes response observation, awaits exchange cleanup, then snapshots state.
pub(crate) async fn finalize_timeout<F>(
    control: &ExchangeControl,
    cancel_and_invalidate: F,
) -> RequestDeliveryState
where
    F: Future<Output = ()>,
{
    control.cancel();
    cancel_and_invalidate.await;
    control.delivery_state()
}

struct PhaseDeadline {
    inactivity: Option<Duration>,
    deadline: Option<Instant>,
    timer: Option<DeadlineTimer>,
    invalid: bool,
}

// Keep the cached sleep paired with the deadline it was created for.
struct DeadlineTimer {
    deadline: Instant,
    sleep: Pin<Box<Sleep>>,
}

impl DeadlineTimer {
    fn new(deadline: Instant) -> Self {
        Self {
            deadline,
            sleep: Box::pin(tokio::time::sleep_until(TokioInstant::from_std(deadline))),
        }
    }

    fn poll(&mut self, cx: &mut Context<'_>) -> bool {
        self.sleep.as_mut().poll(cx).is_ready()
    }
}

impl PhaseDeadline {
    fn new(inactivity: Option<Duration>) -> Self {
        let invalid =
            inactivity.is_some_and(|duration| Instant::now().checked_add(duration).is_none());
        Self {
            inactivity,
            deadline: None,
            timer: None,
            invalid,
        }
    }

    fn begin_pending(&mut self) {
        if self.deadline.is_none() {
            self.deadline = self.inactivity.and_then(|duration| {
                let deadline = Instant::now().checked_add(duration);
                if deadline.is_none() {
                    self.invalid = true;
                }
                deadline
            });
        }
    }

    fn reset_after_progress(&mut self) {
        self.deadline = None;
        self.timer = None;
    }

    fn is_expired(&self, total_deadline: Option<Instant>) -> bool {
        let now = Instant::now();
        self.deadline.is_some_and(|deadline| deadline <= now)
            || total_deadline.is_some_and(|deadline| deadline <= now)
    }

    fn poll_expiration(&mut self, total_deadline: Option<Instant>, cx: &mut Context<'_>) -> bool {
        let deadline = earlier_deadline(self.deadline, total_deadline);
        let Some(deadline) = deadline else {
            self.timer = None;
            return false;
        };
        let timer_matches = self
            .timer
            .as_ref()
            .is_some_and(|timer| timer.deadline == deadline);
        if !timer_matches {
            self.timer = Some(DeadlineTimer::new(deadline));
        }
        self.timer.as_mut().is_some_and(|timer| timer.poll(cx))
    }
}

fn poll_with_deadline<T>(
    phase_deadline: &mut PhaseDeadline,
    total_deadline: Option<Instant>,
    cx: &mut Context<'_>,
    poll_io: impl FnOnce(&mut Context<'_>) -> Poll<io::Result<T>>,
) -> Poll<io::Result<T>> {
    if phase_deadline.invalid {
        return Poll::Ready(Err(invalid_deadline_io_error()));
    }
    if phase_deadline.is_expired(total_deadline) {
        return Poll::Ready(Err(timeout_io_error()));
    }

    match poll_io(cx) {
        Poll::Ready(result) => Poll::Ready(result),
        Poll::Pending => {
            phase_deadline.begin_pending();
            if phase_deadline.invalid {
                return Poll::Ready(Err(invalid_deadline_io_error()));
            }
            if phase_deadline.poll_expiration(total_deadline, cx) {
                Poll::Ready(Err(timeout_io_error()))
            } else {
                Poll::Pending
            }
        }
    }
}

fn earlier_deadline(first: Option<Instant>, second: Option<Instant>) -> Option<Instant> {
    match (first, second) {
        (Some(first), Some(second)) => Some(first.min(second)),
        (Some(deadline), None) | (None, Some(deadline)) => Some(deadline),
        (None, None) => None,
    }
}

fn timeout_io_error() -> io::Error {
    SafeIoFailure::DeadlineElapsed.into_io_error()
}

fn invalid_deadline_io_error() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        "finite timeout exceeds clock range",
    )
}

fn finalized_io_error() -> io::Error {
    SafeIoFailure::ExchangeFinalized.into_io_error()
}

fn unsolicited_response_io_error() -> io::Error {
    io::Error::other("HTTP connection received unsolicited data while idle")
}

fn sender_closed_error() -> io::Error {
    SafeIoFailure::SenderUnavailable.into_io_error()
}

fn sanitize_io_error(error: io::Error) -> io::Error {
    SafeIoFailure::from_io_error(error).into_io_error()
}

pub(crate) fn is_tls_failure(error: &io::Error) -> bool {
    error
        .get_ref()
        .and_then(|source| source.downcast_ref::<SafeIoFailure>())
        .is_some_and(|failure| matches!(failure, SafeIoFailure::TlsFailure))
}

// Retain only a fixed safe cause; never carry the source error text.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SafeIoFailure {
    DeadlineElapsed,
    ExchangeFinalized,
    SenderUnavailable,
    TlsFailure,
    Inner(io::ErrorKind),
}

impl SafeIoFailure {
    fn from_io_error(error: io::Error) -> Self {
        let kind = error.kind();
        let tls_record_rejected = error
            .get_ref()
            .is_some_and(|source| source.downcast_ref::<rustls::Error>().is_some());
        drop(error);
        if tls_record_rejected {
            Self::TlsFailure
        } else if kind == io::ErrorKind::TimedOut {
            Self::DeadlineElapsed
        } else {
            Self::Inner(kind)
        }
    }

    fn into_io_error(self) -> io::Error {
        match self {
            Self::TlsFailure => io::Error::new(io::ErrorKind::InvalidData, self),
            Self::DeadlineElapsed => {
                io::Error::new(io::ErrorKind::TimedOut, "I/O deadline elapsed")
            }
            Self::ExchangeFinalized => {
                io::Error::new(io::ErrorKind::Interrupted, "exchange I/O finalized")
            }
            Self::SenderUnavailable => {
                io::Error::new(io::ErrorKind::BrokenPipe, "HTTP sender failed")
            }
            Self::Inner(kind) => io::Error::new(kind, "transport I/O failed"),
        }
    }
}

impl std::fmt::Display for SafeIoFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("TLS failure")
    }
}

impl std::error::Error for SafeIoFailure {}

#[cfg(test)]
#[path = "timeout_tests.rs"]
mod tests;
