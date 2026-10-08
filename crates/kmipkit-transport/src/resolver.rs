//! Bounded adapter around the operating system's synchronous name resolver.

use std::io;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::{Arc, OnceLock};
use std::time::Instant;

use tokio::sync::{Semaphore, watch};
use tokio::task::JoinError;

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
        mut canceled: watch::Receiver<bool>,
    ) -> Result<Vec<SocketAddr>, ResolveFailure> {
        if *canceled.borrow() {
            return Err(ResolveFailure::Cancelled);
        }
        if Instant::now() >= deadline {
            return Err(ResolveFailure::Deadline);
        }

        let permit = Arc::clone(&self.governor)
            .try_acquire_owned()
            .map_err(|_| ResolveFailure::Capacity)?;
        let lookup = Arc::clone(&self.lookup);
        let host = host.to_owned();
        let mut task = tokio::task::spawn_blocking(move || {
            // Keep admission charged until the native resolver call actually exits, even when
            // the async caller has already timed out or canceled.
            let _permit = permit;
            lookup(&host, port)
        });

        let result = tokio::select! {
            biased;
            () = wait_for_cancel(&mut canceled) => {
                task.abort();
                return Err(ResolveFailure::Cancelled);
            }
            () = tokio::time::sleep_until(deadline.into()) => {
                task.abort();
                return Err(ResolveFailure::Deadline);
            }
            result = &mut task => result,
        };

        let addresses = result
            .map_err(map_join_error)?
            .map_err(|_| ResolveFailure::Lookup)?;
        finish_lookup(addresses, deadline, *canceled.borrow(), Instant::now())
    }
}

fn system_lookup(host: &str, port: u16) -> io::Result<Vec<SocketAddr>> {
    (host, port).to_socket_addrs().map(Iterator::collect)
}

async fn wait_for_cancel(canceled: &mut watch::Receiver<bool>) {
    while !*canceled.borrow() {
        if canceled.changed().await.is_err() {
            return;
        }
    }
}

fn map_join_error(_error: JoinError) -> ResolveFailure {
    ResolveFailure::Lookup
}

fn finish_lookup(
    addresses: Vec<SocketAddr>,
    deadline: Instant,
    canceled: bool,
    now: Instant,
) -> Result<Vec<SocketAddr>, ResolveFailure> {
    if now >= deadline {
        return Err(ResolveFailure::Deadline);
    }
    if canceled {
        return Err(ResolveFailure::Cancelled);
    }

    let candidates = addresses
        .into_iter()
        .take(MAX_CANDIDATES)
        .collect::<Vec<_>>();
    if candidates.is_empty() {
        return Err(ResolveFailure::Lookup);
    }
    Ok(candidates)
}

#[cfg(test)]
#[path = "resolver_tests.rs"]
mod tests;
