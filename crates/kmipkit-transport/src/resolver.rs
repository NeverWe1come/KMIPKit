//! Bounded adapter around the operating system's synchronous name resolver.

use std::io;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::time::Instant;

use tokio::sync::{OwnedSemaphorePermit, Semaphore, watch};
use tokio::task::{JoinError, JoinHandle};

const SHARED_RESOLVER_CAPACITY: usize = 32;
const MAX_CANDIDATES: usize = 16;

type Lookup = dyn Fn(&str, u16) -> io::Result<Vec<SocketAddr>> + Send + Sync + 'static;

static SYSTEM_GOVERNOR: OnceLock<Arc<Semaphore>> = OnceLock::new();

/// A sanitized failure while resolving a configured endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ResolveFailure {
    /// The library-wide bound on active and queued native calls is full.
    Capacity,
    /// The exchange was canceled while waiting for name resolution.
    Cancelled,
    /// The connect or total deadline elapsed during name resolution.
    Deadline,
    /// The operating-system resolver returned no usable result.
    Lookup,
}

/// One client's view of the system resolver and the library-wide admission bound.
#[derive(Clone)]
pub(crate) struct Resolver {
    lookup: Arc<Lookup>,
    governor: Arc<Semaphore>,
}

impl Resolver {
    /// Uses the host's native resolver and the shared per-library admission bound.
    pub(crate) fn system() -> Self {
        Self {
            lookup: Arc::new(system_lookup),
            governor: Arc::clone(
                SYSTEM_GOVERNOR.get_or_init(|| Arc::new(Semaphore::new(SHARED_RESOLVER_CAPACITY))),
            ),
        }
    }

    /// Builds a resolver around a deterministic lookup for lifecycle tests.
    #[cfg(test)]
    pub(crate) fn with_lookup_and_governor(
        lookup: impl Fn(&str, u16) -> io::Result<Vec<SocketAddr>> + Send + Sync + 'static,
        governor: Arc<Semaphore>,
    ) -> Self {
        Self {
            lookup: Arc::new(lookup),
            governor,
        }
    }

    /// Returns the governor shared by resolvers for the active library instance.
    pub(crate) fn governor(&self) -> &Arc<Semaphore> {
        &self.governor
    }

    /// Resolves one host/port pair without waiting for a blocking native call on cancellation.
    pub(crate) async fn lookup_candidates(
        &self,
        host: &str,
        port: u16,
        deadline: Instant,
        canceled: watch::Receiver<bool>,
    ) -> Result<Vec<SocketAddr>, ResolveFailure> {
        self.lookup_candidates_until(host, port, Some(deadline), canceled)
            .await
    }

    /// Resolves a host while honoring cancellation and an optional deadline.
    pub(crate) async fn lookup_candidates_until(
        &self,
        host: &str,
        port: u16,
        deadline: Option<Instant>,
        mut canceled: watch::Receiver<bool>,
    ) -> Result<Vec<SocketAddr>, ResolveFailure> {
        if *canceled.borrow() {
            return Err(ResolveFailure::Cancelled);
        }
        if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
            return Err(ResolveFailure::Deadline);
        }

        let permit = Arc::clone(&self.governor)
            .try_acquire_owned()
            .map_err(|_| ResolveFailure::Capacity)?;
        let lookup = Arc::clone(&self.lookup);
        let host = host.to_owned();
        let state = Arc::new(Mutex::new(ResolverJobState {
            started: false,
            permit: Some(permit),
        }));
        let task_state = Arc::clone(&state);
        let task = tokio::task::spawn_blocking(move || {
            run_lookup(&task_state, lookup.as_ref(), &host, port)
        });
        let mut task = AbortOnDrop::new(task, state);

        let result = tokio::select! {
            biased;
            () = wait_for_cancel(&mut canceled) => {
                return Err(ResolveFailure::Cancelled);
            }
            () = wait_until(deadline) => {
                return Err(ResolveFailure::Deadline);
            }
            result = task.join() => result,
        };

        let addresses = result
            .map_err(map_join_error)?
            .map_err(|_| ResolveFailure::Lookup)?;
        finish_lookup_until(addresses, deadline, *canceled.borrow(), Instant::now())
    }
}

fn system_lookup(host: &str, port: u16) -> io::Result<Vec<SocketAddr>> {
    (host, port).to_socket_addrs().map(collect_candidates)
}

fn collect_candidates(addresses: impl Iterator<Item = SocketAddr>) -> Vec<SocketAddr> {
    addresses.take(MAX_CANDIDATES).collect()
}

async fn wait_for_cancel(canceled: &mut watch::Receiver<bool>) {
    while !*canceled.borrow() {
        if canceled.changed().await.is_err() {
            return;
        }
    }
}

async fn wait_until(deadline: Option<Instant>) {
    if let Some(deadline) = deadline {
        tokio::time::sleep_until(deadline.into()).await;
    } else {
        std::future::pending::<()>().await;
    }
}

fn map_join_error(_error: JoinError) -> ResolveFailure {
    ResolveFailure::Lookup
}

/// Requests cancellation of an unstarted blocking job when its async owner exits.
struct AbortOnDrop<T> {
    task: JoinHandle<T>,
    state: Arc<Mutex<ResolverJobState>>,
}

impl<T> AbortOnDrop<T> {
    fn new(task: JoinHandle<T>, state: Arc<Mutex<ResolverJobState>>) -> Self {
        Self { task, state }
    }

    async fn join(&mut self) -> Result<T, JoinError> {
        (&mut self.task).await
    }
}

impl<T> Drop for AbortOnDrop<T> {
    fn drop(&mut self) {
        self.task.abort();
        let mut state = lock_job_state(&self.state);
        if !state.started {
            state.permit.take();
        }
    }
}

struct ResolverJobState {
    started: bool,
    permit: Option<OwnedSemaphorePermit>,
}

fn begin_resolver_job(state: &Mutex<ResolverJobState>) -> Option<OwnedSemaphorePermit> {
    let mut state = lock_job_state(state);
    state.started = true;
    state.permit.take()
}

fn run_lookup(
    state: &Mutex<ResolverJobState>,
    lookup: &Lookup,
    host: &str,
    port: u16,
) -> io::Result<Vec<SocketAddr>> {
    let Some(_permit) = begin_resolver_job(state) else {
        return Err(io::Error::new(
            io::ErrorKind::Interrupted,
            "resolver job canceled before start",
        ));
    };
    lookup(host, port)
}

fn lock_job_state(state: &Mutex<ResolverJobState>) -> MutexGuard<'_, ResolverJobState> {
    match state.lock() {
        Ok(state) => state,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn finish_lookup(
    addresses: Vec<SocketAddr>,
    deadline: Instant,
    canceled: bool,
    now: Instant,
) -> Result<Vec<SocketAddr>, ResolveFailure> {
    finish_lookup_until(addresses, Some(deadline), canceled, now)
}

fn finish_lookup_until(
    addresses: Vec<SocketAddr>,
    deadline: Option<Instant>,
    canceled: bool,
    now: Instant,
) -> Result<Vec<SocketAddr>, ResolveFailure> {
    if deadline.is_some_and(|deadline| now >= deadline) {
        return Err(ResolveFailure::Deadline);
    }
    if canceled {
        return Err(ResolveFailure::Cancelled);
    }

    let mut candidates = addresses;
    candidates.truncate(MAX_CANDIDATES);
    if candidates.is_empty() {
        return Err(ResolveFailure::Lookup);
    }
    Ok(candidates)
}

#[cfg(test)]
#[path = "resolver_tests.rs"]
mod tests;
