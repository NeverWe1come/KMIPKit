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

use crate::{RequestDeliveryState, TransportError, TransportResponse};

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
pub(crate) type OperationFuture =
    Pin<Box<dyn Future<Output = Result<TransportResponse, TransportError>> + Send + 'static>>;
pub(crate) type WorkerOperation =
    Box<dyn FnOnce(ExchangeControl) -> OperationFuture + Send + 'static>;

/// Erases an operation future before it enters the worker command queue.
pub(crate) fn boxed_operation<F, Fut>(operation: F) -> WorkerOperation
where
    F: FnOnce(ExchangeControl) -> Fut + Send + 'static,
    Fut: Future<Output = Result<TransportResponse, TransportError>> + Send + 'static,
{
    Box::new(move |control| Box::pin(operation(control)))
}

/// Failure to start the private runtime thread, without exposing OS error text.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorkerStartError {
    /// The worker thread could not be created.
    Thread,
    /// The worker's current-thread Tokio runtime could not be created.
    Runtime,
    /// The worker did not become ready before the exchange deadline.
    Deadline,
}

impl fmt::Display for WorkerStartError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Thread => formatter.write_str("transport worker thread could not start"),
            Self::Runtime => formatter.write_str("transport worker runtime could not start"),
            Self::Deadline => formatter.write_str("transport worker missed the exchange deadline"),
        }
    }
}

impl Error for WorkerStartError {}

/// A lifecycle error with delivery evidence and no dependency-provided text.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum WorkerError {
    /// The exchange expired before it could be accepted or dispatched.
    Deadline(RequestDeliveryState),
    /// Worker shutdown canceled the exchange.
    Closed(RequestDeliveryState),
    /// The worker exited before it returned an exchange result.
    Stopped(RequestDeliveryState),
    /// The exchange operation failed after retaining the current delivery state.
    Operation {
        delivery_state: RequestDeliveryState,
        source: TransportError,
    },
}

impl WorkerError {
    /// Returns the strongest delivery evidence represented by this error.
    pub(crate) const fn delivery_state(self) -> RequestDeliveryState {
        match self {
            Self::Deadline(state) | Self::Closed(state) | Self::Stopped(state) => state,
            Self::Operation { delivery_state, .. } => delivery_state,
        }
    }

    /// Extracts the operation error without formatting it.
    pub(crate) fn into_operation_source(self) -> Option<TransportError> {
        match self {
            Self::Operation { source, .. } => Some(source),
            Self::Deadline(_) | Self::Closed(_) | Self::Stopped(_) => None,
        }
    }
}

impl fmt::Debug for WorkerError {
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

impl fmt::Display for WorkerError {
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

impl Error for WorkerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Operation { source, .. } => Some(source),
            Self::Deadline(_) | Self::Closed(_) | Self::Stopped(_) => None,
        }
    }
}

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

    /// Records or confirms a positive response read unless finalization won.
    pub(crate) fn observe_response_byte(&self) -> bool {
        loop {
            let current = self.state.load(Ordering::Acquire);
            match current {
                DISPATCH_COMMITTED => {
                    if self
                        .state
                        .compare_exchange(
                            DISPATCH_COMMITTED,
                            RESPONSE_STARTED,
                            Ordering::AcqRel,
                            Ordering::Acquire,
                        )
                        .is_ok()
                    {
                        return true;
                    }
                }
                RESPONSE_STARTED => return true,
                _ => return false,
            }
        }
    }

    /// Returns whether request I/O may still be observed by the active exchange.
    pub(crate) fn is_finalized(&self) -> bool {
        is_final(self.state.load(Ordering::Acquire))
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
        self.finish_after_load(|| {})
    }

    fn finish_after_load(&self, mut after_load: impl FnMut()) -> (bool, RequestDeliveryState) {
        loop {
            let current = self.state.load(Ordering::Acquire);
            if is_final(current) {
                return (false, delivery_for_state(current));
            }
            let finalized = final_state(current).unwrap_or(FINALIZED_NOT_SENT);
            after_load();
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

    pub(crate) fn subscribe_cancel(&self) -> watch::Receiver<bool> {
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

fn strongest_delivery(
    control: RequestDeliveryState,
    operation: RequestDeliveryState,
) -> RequestDeliveryState {
    match (control, operation) {
        (RequestDeliveryState::ResponseStarted, _) | (_, RequestDeliveryState::ResponseStarted) => {
            RequestDeliveryState::ResponseStarted
        }
        (RequestDeliveryState::PossiblySent, _) | (_, RequestDeliveryState::PossiblySent) => {
            RequestDeliveryState::PossiblySent
        }
        _ => RequestDeliveryState::NotSent,
    }
}

/// One per-client owner of the current-thread runtime and bounded exchange queue.
pub(crate) struct ClientWorker {
    commands: async_mpsc::Sender<BoxedCommand>,
    shutdown: watch::Sender<bool>,
    queue_space: Arc<QueueSpace>,
    closed: AtomicBool,
    thread: WorkerThread,
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

    /// Starts a private runtime, bounding lazy readiness by an absolute deadline.
    pub(crate) fn start_until(deadline: Option<Instant>) -> Result<Self, WorkerStartError> {
        if deadline.is_none() {
            return Self::start();
        }
        Self::start_with_factories_until(
            deadline,
            |task| {
                thread::Builder::new()
                    .name("kmipkit-transport".to_owned())
                    .spawn(task)
            },
            || {
                Builder::new_current_thread()
                    .enable_all()
                    .max_blocking_threads(1)
                    .build()
            },
        )
    }

    /// Starts the worker through an injectable thread spawner for deterministic failure tests.
    pub(crate) fn start_with_spawner(
        spawner: impl FnOnce(WorkerTask) -> io::Result<JoinHandle<()>>,
    ) -> Result<Self, WorkerStartError> {
        Self::start_with_factories(spawner, || {
            Builder::new_current_thread()
                .enable_all()
                .max_blocking_threads(1)
                .build()
        })
    }

    /// Test-only spawner variant for deterministic deadline-bound readiness tests.
    #[cfg(test)]
    pub(crate) fn start_with_spawner_until(
        spawner: impl FnOnce(WorkerTask) -> io::Result<JoinHandle<()>>,
        deadline: Option<Instant>,
    ) -> Result<Self, WorkerStartError> {
        Self::start_with_factories_until(deadline, spawner, || {
            Builder::new_current_thread()
                .enable_all()
                .max_blocking_threads(1)
                .build()
        })
    }

    fn start_with_factories(
        spawner: impl FnOnce(WorkerTask) -> io::Result<JoinHandle<()>>,
        runtime_factory: impl FnOnce() -> io::Result<Runtime> + Send + 'static,
    ) -> Result<Self, WorkerStartError> {
        Self::start_with_factories_until(None, spawner, runtime_factory)
    }

    fn start_with_factories_until(
        deadline: Option<Instant>,
        spawner: impl FnOnce(WorkerTask) -> io::Result<JoinHandle<()>>,
        runtime_factory: impl FnOnce() -> io::Result<Runtime> + Send + 'static,
    ) -> Result<Self, WorkerStartError> {
        if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
            return Err(WorkerStartError::Deadline);
        }
        let (command_sender, command_receiver) = async_mpsc::channel(COMMAND_CAPACITY);
        let (shutdown_sender, shutdown_receiver) = watch::channel(false);
        let (ready_sender, ready_receiver) = mpsc::sync_channel(1);
        let queue_space = Arc::new(QueueSpace::default());
        let completion = Arc::new(Completion::default());
        let thread_completion = Arc::clone(&completion);
        let runtime_queue_space = Arc::clone(&queue_space);
        let thread_queue_space = Arc::clone(&queue_space);

        let task = Box::new(move || {
            let runtime_result = catch_unwind(AssertUnwindSafe(|| {
                let Ok(runtime) = runtime_factory() else {
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
            if let Err(panic_payload) = runtime_result {
                // Panic payloads can contain operation state; discard without formatting.
                drop(panic_payload);
            }
            thread_queue_space.notify_slot_available();
            thread_completion.mark_finished();
        });

        let join = spawner(task).map_err(|_| WorkerStartError::Thread)?;
        let readiness = match deadline {
            Some(deadline) => {
                ready_receiver.recv_timeout(deadline.saturating_duration_since(Instant::now()))
            }
            None => ready_receiver
                .recv()
                .map_err(|_| mpsc::RecvTimeoutError::Disconnected),
        };
        match readiness {
            Ok(Ok(())) if deadline.is_some_and(|deadline| Instant::now() >= deadline) => {
                shutdown_sender.send_replace(true);
                drop(join);
                Err(WorkerStartError::Deadline)
            }
            Ok(Ok(())) => Ok(Self {
                commands: command_sender,
                shutdown: shutdown_sender,
                queue_space,
                closed: AtomicBool::new(false),
                thread: WorkerThread::new(join, completion),
            }),
            Ok(Err(error)) => {
                let _ = join.join();
                Err(error)
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                shutdown_sender.send_replace(true);
                drop(join);
                Err(WorkerStartError::Deadline)
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                let _ = join.join();
                Err(WorkerStartError::Runtime)
            }
        }
    }

    /// Submits one synchronous operation and waits until its absolute deadline.
    pub(crate) fn exchange<F, Fut>(
        &self,
        deadline: Option<Instant>,
        operation: F,
    ) -> Result<TransportResponse, WorkerError>
    where
        F: FnOnce(ExchangeControl) -> Fut + Send + 'static,
        Fut: Future<Output = Result<TransportResponse, TransportError>> + Send + 'static,
    {
        let control = ExchangeControl::new();
        let (result_sender, result_receiver) = mpsc::sync_channel(1);
        let command: BoxedCommand = Box::new(ExchangeCommand {
            deadline,
            control: control.clone(),
            operation: boxed_operation(operation),
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

        self.thread.wait_and_join(timeout)
    }

    fn enqueue(
        &self,
        mut command: BoxedCommand,
        control: &ExchangeControl,
        deadline: Option<Instant>,
    ) -> Result<(), WorkerError> {
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

    fn reject_closed(self: Box<Self>);
}

struct ExchangeCommand {
    deadline: Option<Instant>,
    control: ExchangeControl,
    operation: WorkerOperation,
    result: SyncSender<Result<TransportResponse, WorkerError>>,
}

impl WorkerCommand for ExchangeCommand {
    fn run(self: Box<Self>, mut shutdown: watch::Receiver<bool>) -> CommandFuture {
        let ExchangeCommand {
            deadline,
            control,
            operation,
            result,
        } = *self;
        Box::pin(async move {
            if *shutdown.borrow() {
                let state = control.cancel();
                let _ = result.send(Err(WorkerError::Closed(state)));
                return;
            }
            if deadline.is_some_and(|limit| Instant::now() >= limit) {
                let state = control.cancel();
                let _ = result.send(Err(WorkerError::Deadline(state)));
                return;
            }
            if !control.begin() {
                let state = control.delivery_state();
                let _ = result.send(Err(WorkerError::Deadline(state)));
                return;
            }

            let mut canceled = control.subscribe_cancel();
            let operation_future = operation(control.clone());
            tokio::pin!(operation_future);

            tokio::select! {
                biased;
                changed = shutdown.changed() => {
                    let _ = changed;
                    let state = control.cancel();
                    let _ = result.send(Err(WorkerError::Closed(state)));
                }
                changed = canceled.changed() => {
                    let _ = changed;
                    let state = control.delivery_state();
                    let _ = result.send(Err(WorkerError::Deadline(state)));
                }
                () = wait_until(deadline) => {
                    let state = control.cancel();
                    let _ = result.send(Err(WorkerError::Deadline(state)));
                }
                operation_result = &mut operation_future => {
                    if *shutdown.borrow() {
                        let state = control.cancel();
                        let _ = result.send(Err(WorkerError::Closed(state)));
                    } else {
                        let (completed, delivery_state) = control.finish();
                        if completed {
                            let outcome = operation_result.map_err(|source| {
                                WorkerError::Operation {
                                    delivery_state: strongest_delivery(
                                        delivery_state,
                                        source.delivery_state(),
                                    ),
                                    source,
                                }
                            });
                            let _ = result.send(outcome);
                        } else {
                            let _ = result.send(Err(WorkerError::Deadline(delivery_state)));
                        }
                    }
                }
            }
        })
    }

    fn reject_closed(self: Box<Self>) {
        let state = self.control.cancel();
        let _ = self.result.send(Err(WorkerError::Closed(state)));
    }
}

fn run_runtime(
    runtime: Runtime,
    commands: async_mpsc::Receiver<BoxedCommand>,
    shutdown: watch::Receiver<bool>,
    queue_space: Arc<QueueSpace>,
) {
    runtime.block_on(runtime_loop(commands, shutdown, queue_space));
    // A started `spawn_blocking` resolver call cannot be interrupted. Detach such
    // calls after the async worker has shut down instead of blocking client close.
    runtime.shutdown_timeout(Duration::ZERO);
}

async fn runtime_loop(
    mut commands: async_mpsc::Receiver<BoxedCommand>,
    mut shutdown: watch::Receiver<bool>,
    queue_space: Arc<QueueSpace>,
) {
    loop {
        if *shutdown.borrow() {
            break;
        }
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
        command.reject_closed();
        queue_space.notify_slot_available();
    }
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

fn receive_result(
    receiver: Receiver<Result<TransportResponse, WorkerError>>,
    deadline: Option<Instant>,
) -> Result<Result<TransportResponse, WorkerError>, ReceiveFailure> {
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

/// Owns the worker's completion signal and join handle as one lifecycle unit.
struct WorkerThread {
    completion: Arc<Completion>,
    join: Mutex<Option<JoinHandle<()>>>,
}

impl WorkerThread {
    fn new(join: JoinHandle<()>, completion: Arc<Completion>) -> Self {
        Self {
            completion,
            join: Mutex::new(Some(join)),
        }
    }

    fn wait_and_join(&self, timeout: Duration) -> Result<(), WorkerShutdownError> {
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

#[cfg(test)]
#[path = "../tests/worker.rs"]
mod tests;
