//! Private synchronous bridge to one Tokio current-thread runtime per client.
//!
//! The worker owns no network policy itself. Callers submit one operation at a
//! time through a bounded command queue; the operation owns its async I/O and
//! must pass [`ExchangeControl::commit_dispatch`] immediately before handing
//! request bytes to its writer.

use std::error::Error;
use std::fmt;
use std::future::Future;
use std::io;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use tokio::runtime::{Builder, Runtime};
use tokio::sync::{mpsc as async_mpsc, watch};

use crate::RequestDeliveryState;

const QUEUED: u8 = 0;
const PREPARING: u8 = 1;
const DISPATCH_COMMITTED: u8 = 2;
const RESPONSE_STARTED: u8 = 3;
const FINALIZED_NOT_SENT: u8 = 4;
const FINALIZED_POSSIBLY_SENT: u8 = 5;
const FINALIZED_RESPONSE_STARTED: u8 = 6;
const COMMAND_CAPACITY: usize = 1;
const DROP_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(1);

type BoxedCommand = Box<dyn WorkerCommand>;
pub(crate) type WorkerTask = Box<dyn FnOnce() + Send + 'static>;
type CommandFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

/// Failure to start the private runtime thread, without exposing OS error text.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorkerStartError {
    /// The worker thread could not be created.
    Thread,
    /// The worker's current-thread Tokio runtime could not be created.
    Runtime,
}

impl fmt::Display for WorkerStartError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Thread => formatter.write_str("transport worker thread could not start"),
            Self::Runtime => formatter.write_str("transport worker runtime could not start"),
        }
    }
}

impl Error for WorkerStartError {}

/// A lifecycle error with delivery evidence and no dependency-provided text.
#[derive(PartialEq, Eq)]
pub(crate) enum WorkerError<E> {
    /// The exchange expired before it could be accepted or dispatched.
    Deadline(RequestDeliveryState),
    /// Worker shutdown canceled the exchange.
    Closed(RequestDeliveryState),
    /// The worker exited before it returned an exchange result.
    Stopped(RequestDeliveryState),
    /// The exchange operation failed after retaining the current delivery state.
    Operation {
        delivery_state: RequestDeliveryState,
        source: E,
    },
}

impl<E> WorkerError<E> {
    /// Returns the strongest delivery evidence represented by this error.
    pub(crate) const fn delivery_state(&self) -> RequestDeliveryState {
        match self {
            Self::Deadline(state) | Self::Closed(state) | Self::Stopped(state) => *state,
            Self::Operation { delivery_state, .. } => *delivery_state,
        }
    }

    /// Extracts the operation error without formatting it.
    pub(crate) fn into_operation_source(self) -> Option<E> {
        match self {
            Self::Operation { source, .. } => Some(source),
            Self::Deadline(_) | Self::Closed(_) | Self::Stopped(_) => None,
        }
    }
}

impl<E> fmt::Debug for WorkerError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Deadline(state) => formatter.debug_tuple("Deadline").field(state).finish(),
            Self::Closed(state) => formatter.debug_tuple("Closed").field(state).finish(),
            Self::Stopped(state) => formatter.debug_tuple("Stopped").field(state).finish(),
            Self::Operation { delivery_state, .. } => formatter
                .debug_struct("Operation")
                .field("delivery_state", delivery_state)
                .field("source", &"[REDACTED]")
                .finish(),
        }
    }
}

impl<E> fmt::Display for WorkerError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Deadline(state) => write!(formatter, "exchange deadline elapsed ({state:?})"),
            Self::Closed(state) => write!(formatter, "transport worker closed ({state:?})"),
            Self::Stopped(state) => write!(formatter, "transport worker stopped ({state:?})"),
            Self::Operation { delivery_state, .. } => {
                write!(formatter, "transport operation failed ({delivery_state:?})")
            }
        }
    }
}

impl<E: 'static> Error for WorkerError<E> {}

/// Atomic cancellation/dispatch gate shared by a caller and its worker command.
#[derive(Clone)]
pub(crate) struct ExchangeControl {
    state: Arc<AtomicU8>,
    canceled: watch::Sender<bool>,
}

impl ExchangeControl {
    /// Creates a control gate in the queued state.
    pub(crate) fn new() -> Self {
        let (canceled, _receiver) = watch::channel(false);
        Self {
            state: Arc::new(AtomicU8::new(QUEUED)),
            canceled,
        }
    }

    /// Moves a command from the queue into connection preparation.
    pub(crate) fn begin(&self) -> bool {
        self.state
            .compare_exchange(QUEUED, PREPARING, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }

    /// Atomically commits request dispatch or reports that cancellation won.
    pub(crate) fn commit_dispatch(&self) -> bool {
        self.state
            .compare_exchange(
                PREPARING,
                DISPATCH_COMMITTED,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_ok()
    }

    /// Records the first observed response byte before it reaches a parser.
    pub(crate) fn response_started(&self) -> bool {
        self.state
            .compare_exchange(
                DISPATCH_COMMITTED,
                RESPONSE_STARTED,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_ok()
    }

    /// Finalizes cancellation without moving delivery evidence backwards.
    pub(crate) fn cancel(&self) -> RequestDeliveryState {
        loop {
            let current = self.state.load(Ordering::Acquire);
            let Some(finalized) = final_state(current) else {
                return delivery_for_state(current);
            };
            if self
                .state
                .compare_exchange(current, finalized, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                self.canceled.send_replace(true);
                return delivery_for_state(finalized);
            }
        }
    }

    /// Finalizes a completed operation; returns false if cancellation already won.
    fn finish(&self) -> (bool, RequestDeliveryState) {
        loop {
            let current = self.state.load(Ordering::Acquire);
            if is_final(current) {
                return (false, delivery_for_state(current));
            }
            let finalized = final_state(current).unwrap_or(FINALIZED_NOT_SENT);
            if self
                .state
                .compare_exchange(current, finalized, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return (true, delivery_for_state(finalized));
            }
        }
    }

    /// Returns a stable view of the current delivery state.
    pub(crate) fn delivery_state(&self) -> RequestDeliveryState {
        delivery_for_state(self.state.load(Ordering::Acquire))
    }

    fn subscribe_cancel(&self) -> watch::Receiver<bool> {
        self.canceled.subscribe()
    }
}

fn final_state(state: u8) -> Option<u8> {
    match state {
        DISPATCH_COMMITTED => Some(FINALIZED_POSSIBLY_SENT),
        RESPONSE_STARTED => Some(FINALIZED_RESPONSE_STARTED),
        FINALIZED_NOT_SENT | FINALIZED_POSSIBLY_SENT | FINALIZED_RESPONSE_STARTED => None,
        _ => Some(FINALIZED_NOT_SENT),
    }
}

fn is_final(state: u8) -> bool {
    matches!(
        state,
        FINALIZED_NOT_SENT | FINALIZED_POSSIBLY_SENT | FINALIZED_RESPONSE_STARTED
    )
}

fn delivery_for_state(state: u8) -> RequestDeliveryState {
    match state {
        DISPATCH_COMMITTED | FINALIZED_POSSIBLY_SENT => RequestDeliveryState::PossiblySent,
        RESPONSE_STARTED | FINALIZED_RESPONSE_STARTED => RequestDeliveryState::ResponseStarted,
        _ => RequestDeliveryState::NotSent,
    }
}

/// One per-client owner of the current-thread runtime and bounded exchange queue.
pub(crate) struct ClientWorker {
    commands: async_mpsc::Sender<BoxedCommand>,
    shutdown: watch::Sender<bool>,
    queue_space: Arc<QueueSpace>,
    closed: AtomicBool,
    completion: Arc<Completion>,
    join: Mutex<Option<JoinHandle<()>>>,
}

impl ClientWorker {
    /// Starts a private current-thread runtime without opening a network connection.
    pub(crate) fn start() -> Result<Self, WorkerStartError> {
        Self::start_with_spawner(|task| {
            thread::Builder::new()
                .name("kmipkit-transport".to_owned())
                .spawn(task)
        })
    }

    /// Starts the worker through an injectable thread spawner for deterministic failure tests.
    pub(crate) fn start_with_spawner(
        spawner: impl FnOnce(WorkerTask) -> io::Result<JoinHandle<()>>,
    ) -> Result<Self, WorkerStartError> {
        let (command_sender, command_receiver) = async_mpsc::channel(COMMAND_CAPACITY);
        let (shutdown_sender, shutdown_receiver) = watch::channel(false);
        let (ready_sender, ready_receiver) = mpsc::sync_channel(1);
        let queue_space = Arc::new(QueueSpace::default());
        let completion = Arc::new(Completion::default());
        let thread_completion = Arc::clone(&completion);
        let runtime_queue_space = Arc::clone(&queue_space);
        let thread_queue_space = Arc::clone(&queue_space);

        let task = Box::new(move || {
            let thread_result = catch_unwind(AssertUnwindSafe(|| {
                let Ok(runtime) = Builder::new_current_thread().enable_all().build() else {
                    let _ = ready_sender.send(Err(WorkerStartError::Runtime));
                    return;
                };

                if ready_sender.send(Ok(())).is_err() {
                    return;
                }

                run_runtime(
                    runtime,
                    command_receiver,
                    shutdown_receiver,
                    runtime_queue_space,
                );
            }));
            drop(thread_result);
            thread_queue_space.notify_slot_available();
            thread_completion.mark_finished();
        });

        let join = spawner(task).map_err(|_| WorkerStartError::Thread)?;
        match ready_receiver.recv() {
            Ok(Ok(())) => Ok(Self {
                commands: command_sender,
                shutdown: shutdown_sender,
                queue_space,
                closed: AtomicBool::new(false),
                completion,
                join: Mutex::new(Some(join)),
            }),
            Ok(Err(error)) => {
                let _ = join.join();
                Err(error)
            }
            Err(_) => {
                let _ = join.join();
                Err(WorkerStartError::Runtime)
            }
        }
    }

    /// Submits one synchronous operation and waits until its absolute deadline.
    pub(crate) fn exchange<T, E, F, Fut>(
        &self,
        deadline: Option<Instant>,
        operation: F,
    ) -> Result<T, WorkerError<E>>
    where
        T: Send + 'static,
        E: Send + 'static,
        F: FnOnce(ExchangeControl) -> Fut + Send + 'static,
        Fut: Future<Output = Result<T, E>> + Send + 'static,
    {
        let control = ExchangeControl::new();
        let (result_sender, result_receiver) = mpsc::sync_channel(1);
        let command: BoxedCommand = Box::new(ExchangeCommand {
            deadline,
            control: control.clone(),
            operation: Some(operation),
            result: result_sender,
        });

        self.enqueue(command, &control, deadline)?;
        match receive_result(result_receiver, deadline) {
            Ok(result) => result,
            Err(ReceiveFailure::Deadline) => Err(WorkerError::Deadline(control.cancel())),
            Err(ReceiveFailure::Disconnected) => Err(WorkerError::Stopped(control.cancel())),
        }
    }

    /// Signals cancellation and waits no longer than `timeout` for worker exit.
    pub(crate) fn close(&self, timeout: Duration) -> Result<(), WorkerShutdownError> {
        self.closed.store(true, Ordering::Release);
        self.shutdown.send_replace(true);
        self.queue_space.notify_slot_available();

        if !self.completion.wait(timeout) {
            return Err(WorkerShutdownError::TimedOut);
        }

        let mut join = lock_unpoisoned(&self.join);
        if let Some(handle) = join.take()
            && handle.join().is_err()
        {
            return Err(WorkerShutdownError::Panicked);
        }
        Ok(())
    }

    fn enqueue<E>(
        &self,
        mut command: BoxedCommand,
        control: &ExchangeControl,
        deadline: Option<Instant>,
    ) -> Result<(), WorkerError<E>> {
        loop {
            if self.closed.load(Ordering::Acquire) {
                return Err(WorkerError::Closed(control.cancel()));
            }

            let observed_generation = self.queue_space.generation();
            match self.commands.try_send(command) {
                Ok(()) => return Ok(()),
                Err(async_mpsc::error::TrySendError::Closed(returned)) => {
                    drop(returned);
                    return Err(WorkerError::Stopped(control.cancel()));
                }
                Err(async_mpsc::error::TrySendError::Full(returned)) => {
                    command = returned;
                    if !self
                        .queue_space
                        .wait_for_change(observed_generation, deadline)
                    {
                        return Err(WorkerError::Deadline(control.cancel()));
                    }
                }
            }
        }
    }
}

impl Drop for ClientWorker {
    fn drop(&mut self) {
        let _ = self.close(DROP_SHUTDOWN_TIMEOUT);
    }
}

/// A bounded command queue item whose result type remains local to its caller.
trait WorkerCommand: Send {
    fn run(self: Box<Self>, shutdown: watch::Receiver<bool>) -> CommandFuture;

    fn reject(self: Box<Self>, error: CommandRejection);
}

enum CommandRejection {
    Closed,
}

struct ExchangeCommand<F, T, E> {
    deadline: Option<Instant>,
    control: ExchangeControl,
    operation: Option<F>,
    result: SyncSender<Result<T, WorkerError<E>>>,
}

impl<F, Fut, T, E> WorkerCommand for ExchangeCommand<F, T, E>
where
    F: FnOnce(ExchangeControl) -> Fut + Send + 'static,
    Fut: Future<Output = Result<T, E>> + Send + 'static,
    T: Send + 'static,
    E: Send + 'static,
{
    fn run(mut self: Box<Self>, mut shutdown: watch::Receiver<bool>) -> CommandFuture {
        Box::pin(async move {
            if *shutdown.borrow() {
                self.reject(CommandRejection::Closed);
                return;
            }
            if self
                .deadline
                .is_some_and(|deadline| Instant::now() >= deadline)
            {
                let state = self.control.cancel();
                let _ = self.result.send(Err(WorkerError::Deadline(state)));
                return;
            }
            if !self.control.begin() {
                let state = self.control.delivery_state();
                let _ = self.result.send(Err(WorkerError::Deadline(state)));
                return;
            }

            let Some(operation) = self.operation.take() else {
                let state = self.control.cancel();
                let _ = self.result.send(Err(WorkerError::Stopped(state)));
                return;
            };
            let control = self.control.clone();
            let mut canceled = self.control.subscribe_cancel();
            let result_sender = self.result;
            let operation_future = operation(control.clone());
            tokio::pin!(operation_future);

            tokio::select! {
                biased;
                changed = shutdown.changed() => {
                    let _ = changed;
                    let state = control.cancel();
                    let _ = result_sender.send(Err(WorkerError::Closed(state)));
                }
                changed = canceled.changed() => {
                    let _ = changed;
                    let state = control.delivery_state();
                    let _ = result_sender.send(Err(WorkerError::Deadline(state)));
                }
                () = wait_until(self.deadline) => {
                    let state = control.cancel();
                    let _ = result_sender.send(Err(WorkerError::Deadline(state)));
                }
                result = &mut operation_future => {
                    let (completed, delivery_state) = control.finish();
                    if completed {
                        let result = result.map_err(|source| WorkerError::Operation {
                            delivery_state,
                            source,
                        });
                        let _ = result_sender.send(result);
                    } else {
                        let _ = result_sender.send(Err(WorkerError::Deadline(delivery_state)));
                    }
                }
            }
        })
    }

    fn reject(self: Box<Self>, rejection: CommandRejection) {
        let state = self.control.cancel();
        let error = match rejection {
            CommandRejection::Closed => WorkerError::Closed(state),
        };
        let _ = self.result.send(Err(error));
    }
}

fn run_runtime(
    runtime: Runtime,
    mut commands: async_mpsc::Receiver<BoxedCommand>,
    mut shutdown: watch::Receiver<bool>,
    queue_space: Arc<QueueSpace>,
) {
    runtime.block_on(async move {
        loop {
            let command = tokio::select! {
                biased;
                changed = shutdown.changed() => {
                    let _ = changed;
                    break;
                }
                command = commands.recv() => match command {
                    Some(command) => command,
                    None => break,
                }
            };
            queue_space.notify_slot_available();
            command.run(shutdown.clone()).await;
            if *shutdown.borrow() {
                break;
            }
        }

        while let Ok(command) = commands.try_recv() {
            command.reject(CommandRejection::Closed);
            queue_space.notify_slot_available();
        }
    });
    drop(runtime);
}

async fn wait_until(deadline: Option<Instant>) {
    match deadline {
        Some(deadline) => tokio::time::sleep_until(deadline.into()).await,
        None => std::future::pending::<()>().await,
    }
}

enum ReceiveFailure {
    Deadline,
    Disconnected,
}

fn receive_result<T, E>(
    receiver: Receiver<Result<T, WorkerError<E>>>,
    deadline: Option<Instant>,
) -> Result<Result<T, WorkerError<E>>, ReceiveFailure> {
    let result = match deadline {
        Some(deadline) => {
            let remaining = deadline.saturating_duration_since(Instant::now());
            match receiver.recv_timeout(remaining) {
                Ok(result) => Ok(result),
                Err(mpsc::RecvTimeoutError::Timeout) => Err(ReceiveFailure::Deadline),
                Err(mpsc::RecvTimeoutError::Disconnected) => Err(ReceiveFailure::Disconnected),
            }
        }
        None => receiver.recv().map_err(|_| ReceiveFailure::Disconnected),
    };
    drop(receiver);
    result
}

#[derive(Default)]
struct QueueSpace {
    generation: Mutex<u64>,
    changed: Condvar,
}

impl QueueSpace {
    fn generation(&self) -> u64 {
        *lock_unpoisoned(&self.generation)
    }

    fn notify_slot_available(&self) {
        let mut generation = lock_unpoisoned(&self.generation);
        *generation = generation.wrapping_add(1);
        self.changed.notify_all();
    }

    fn wait_for_change(&self, observed: u64, deadline: Option<Instant>) -> bool {
        let mut generation = lock_unpoisoned(&self.generation);
        while *generation == observed {
            match deadline {
                Some(deadline) => {
                    let remaining = deadline.saturating_duration_since(Instant::now());
                    if remaining.is_zero() {
                        return false;
                    }
                    let (updated, timeout) =
                        wait_timeout_unpoisoned(&self.changed, generation, remaining);
                    generation = updated;
                    if timeout && *generation == observed {
                        return false;
                    }
                }
                None => {
                    generation = wait_unpoisoned(&self.changed, generation);
                }
            }
        }
        true
    }
}

#[derive(Default)]
struct Completion {
    finished: Mutex<bool>,
    changed: Condvar,
}

impl Completion {
    fn mark_finished(&self) {
        let mut finished = lock_unpoisoned(&self.finished);
        *finished = true;
        self.changed.notify_all();
    }

    fn wait(&self, timeout: Duration) -> bool {
        let mut finished = lock_unpoisoned(&self.finished);
        if *finished {
            return true;
        }
        let started = Instant::now();
        loop {
            let remaining = timeout.saturating_sub(started.elapsed());
            if remaining.is_zero() {
                return false;
            }
            let (updated, timed_out) = wait_timeout_unpoisoned(&self.changed, finished, remaining);
            finished = updated;
            if *finished {
                return true;
            }
            if timed_out {
                return false;
            }
        }
    }
}

fn lock_unpoisoned<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn wait_unpoisoned<'a, T>(condition: &Condvar, guard: MutexGuard<'a, T>) -> MutexGuard<'a, T> {
    match condition.wait(guard) {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn wait_timeout_unpoisoned<'a, T>(
    condition: &Condvar,
    guard: MutexGuard<'a, T>,
    timeout: Duration,
) -> (MutexGuard<'a, T>, bool) {
    match condition.wait_timeout(guard, timeout) {
        Ok((guard, result)) => (guard, result.timed_out()),
        Err(poisoned) => {
            let (guard, result) = poisoned.into_inner();
            (guard, result.timed_out())
        }
    }
}

/// Failure to stop or join the worker within the requested bound.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorkerShutdownError {
    /// The worker did not exit before the caller's shutdown bound.
    TimedOut,
    /// The worker thread panicked outside its contained runtime boundary.
    Panicked,
}

impl fmt::Display for WorkerShutdownError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TimedOut => formatter.write_str("transport worker shutdown timed out"),
            Self::Panicked => formatter.write_str("transport worker stopped unexpectedly"),
        }
    }
}

impl Error for WorkerShutdownError {}
