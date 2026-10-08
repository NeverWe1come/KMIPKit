//! Deterministic lifecycle tests for the native system resolver adapter.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex as StdMutex, mpsc};
use std::time::{Duration, Instant};

use tokio::sync::{Mutex as AsyncMutex, Semaphore, watch};

// Include production code so injected lookup tests exercise the real admission
// and cancellation path without depending on mutable host DNS configuration.
#[allow(dead_code)]
#[path = "../src/resolver.rs"]
mod resolver;

use resolver::{ResolveFailure, Resolver};

const TEST_DEADLINE: Duration = Duration::from_secs(2);
const PROMPT_RETURN_LIMIT: Duration = Duration::from_secs(1);
static SYSTEM_GOVERNOR_TEST_LOCK: AsyncMutex<()> = AsyncMutex::const_new(());

#[derive(Debug)]
struct LookupGate {
    started: mpsc::SyncSender<()>,
    released: (StdMutex<bool>, Condvar),
}

impl LookupGate {
    fn new() -> (Arc<Self>, mpsc::Receiver<()>) {
        let (started_sender, started_receiver) = mpsc::sync_channel(0);
        (
            Arc::new(Self {
                started: started_sender,
                released: (StdMutex::new(false), Condvar::new()),
            }),
            started_receiver,
        )
    }

    fn block_until_released(&self) {
        let _ = self.started.send(());
        let (released, changed) = &self.released;
        let mut released = released.lock().expect("gate mutex is not poisoned");
        while !*released {
            released = changed
                .wait(released)
                .expect("gate mutex is not poisoned while waiting");
        }
    }

    fn release(&self) {
        let (released, changed) = &self.released;
        *released.lock().expect("gate mutex is not poisoned") = true;
        changed.notify_all();
    }
}

fn address(last_octet: u8, port: u16) -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 0, 2, last_octet)), port)
}

fn cancellation_channel() -> (watch::Sender<bool>, watch::Receiver<bool>) {
    watch::channel(false)
}

async fn wait_for_permit(semaphore: &Semaphore, expected: usize) {
    tokio::time::timeout(TEST_DEADLINE, async {
        while semaphore.available_permits() != expected {
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
    })
    .await
    .expect("resolver job releases its admission permit");
}

#[tokio::test]
async fn system_resolver_uses_the_host_configuration_for_localhost() {
    let _governor_guard = SYSTEM_GOVERNOR_TEST_LOCK.lock().await;
    let resolver = Resolver::system();
    let (_cancel_sender, cancel_receiver) = cancellation_channel();

    let candidates = resolver
        .lookup_candidates(
            "localhost",
            5696,
            Instant::now() + TEST_DEADLINE,
            cancel_receiver,
        )
        .await
        .expect("the active operating-system resolver resolves localhost");

    assert!(!candidates.is_empty());
    assert!(candidates.len() <= 16);
    assert!(
        candidates
            .iter()
            .all(|candidate| candidate.ip().is_loopback())
    );
    assert!(candidates.iter().all(|candidate| candidate.port() == 5696));
}

#[tokio::test]
async fn system_resolver_instances_share_one_governor() {
    let _governor_guard = SYSTEM_GOVERNOR_TEST_LOCK.lock().await;
    let first = Resolver::system();
    let second = Resolver::system();

    assert!(Arc::ptr_eq(first.governor(), second.governor()));
    assert_eq!(first.governor().available_permits(), 32);
}

#[tokio::test]
async fn returned_addresses_keep_os_order_and_are_capped_at_sixteen() {
    let expected = (1..=20)
        .map(|octet| address(octet, 5696))
        .collect::<Vec<_>>();
    let supplied = expected.clone();
    let calls = Arc::new(AtomicUsize::new(0));
    let lookup_calls = Arc::clone(&calls);
    let resolver = Resolver::with_lookup_and_governor(
        move |host, port| {
            lookup_calls.fetch_add(1, Ordering::Relaxed);
            assert_eq!(host, "kmip.example.test");
            assert_eq!(port, 5696);
            Ok(supplied.clone())
        },
        Arc::new(Semaphore::new(32)),
    );
    let (_cancel_sender, cancel_receiver) = cancellation_channel();

    let candidates = resolver
        .lookup_candidates(
            "kmip.example.test",
            5696,
            Instant::now() + TEST_DEADLINE,
            cancel_receiver,
        )
        .await
        .expect("injected resolution succeeds");

    assert_eq!(candidates, expected[..16].to_vec());
    assert_eq!(calls.load(Ordering::Relaxed), 1);
}

#[tokio::test]
async fn shared_governor_caps_active_resolver_jobs_at_thirty_two() {
    let _governor_guard = SYSTEM_GOVERNOR_TEST_LOCK.lock().await;
    let (started_sender, started_receiver) = mpsc::sync_channel(32);
    let gate = Arc::new(LookupGate {
        started: started_sender,
        released: (StdMutex::new(false), Condvar::new()),
    });
    let system_resolver = Resolver::system();
    let governor = Arc::clone(system_resolver.governor());
    let first_gate = Arc::clone(&gate);
    let first_resolver = Resolver::with_lookup_and_governor(
        move |_host, _port| {
            first_gate.block_until_released();
            Ok(vec![address(3, 5696)])
        },
        Arc::clone(&governor),
    );
    let second_gate = Arc::clone(&gate);
    let second_resolver = Resolver::with_lookup_and_governor(
        move |_host, _port| {
            second_gate.block_until_released();
            Ok(vec![address(3, 5696)])
        },
        Arc::clone(&governor),
    );
    let mut lookups = tokio::task::JoinSet::new();
    let mut cancel_senders = Vec::with_capacity(32);

    for index in 0..32 {
        let resolver = if index % 2 == 0 {
            first_resolver.clone()
        } else {
            second_resolver.clone()
        };
        let (cancel_sender, cancel_receiver) = cancellation_channel();
        cancel_senders.push(cancel_sender);
        lookups.spawn(async move {
            resolver
                .lookup_candidates(
                    &format!("host-{index}.example.test"),
                    5696,
                    Instant::now() + TEST_DEADLINE,
                    cancel_receiver,
                )
                .await
        });
    }

    tokio::task::spawn_blocking(move || {
        for _ in 0..32 {
            started_receiver
                .recv_timeout(TEST_DEADLINE)
                .expect("each admitted resolver job starts")
        }
    })
    .await
    .expect("started receiver task completes");
    assert_eq!(governor.available_permits(), 0);

    let (_cancel_sender, cancel_receiver) = cancellation_channel();
    let resolver = second_resolver;
    let rejected = resolver
        .lookup_candidates(
            "thirty-third.example.test",
            5696,
            Instant::now() + TEST_DEADLINE,
            cancel_receiver,
        )
        .await;
    assert!(matches!(rejected, Err(ResolveFailure::Capacity)));

    gate.release();
    while let Some(result) = lookups.join_next().await {
        assert!(result.expect("resolver task does not panic").is_ok());
    }
    drop(cancel_senders);
    assert_eq!(governor.available_permits(), 32);
}

#[tokio::test]
async fn resolver_admission_fails_fast_and_permit_stays_held_after_cancellation() {
    let (gate, started_receiver) = LookupGate::new();
    let lookup_gate = Arc::clone(&gate);
    let semaphore = Arc::new(Semaphore::new(1));
    let resolver = Resolver::with_lookup_and_governor(
        move |_host, _port| {
            lookup_gate.block_until_released();
            Ok(vec![address(1, 5696)])
        },
        Arc::clone(&semaphore),
    );
    let (cancel_sender, cancel_receiver) = cancellation_channel();
    let first_resolver = resolver.clone();
    let mut first_lookup = tokio::spawn(async move {
        first_resolver
            .lookup_candidates(
                "blocked.example.test",
                5696,
                Instant::now() + TEST_DEADLINE,
                cancel_receiver,
            )
            .await
    });

    tokio::task::spawn_blocking(move || {
        started_receiver
            .recv_timeout(TEST_DEADLINE)
            .expect("the native lookup starts")
    })
    .await
    .expect("started receiver task completes");
    assert_eq!(semaphore.available_permits(), 0);

    let (_second_cancel, second_cancel_receiver) = cancellation_channel();
    let second = tokio::time::timeout(
        PROMPT_RETURN_LIMIT,
        resolver.lookup_candidates(
            "second.example.test",
            5696,
            Instant::now() + TEST_DEADLINE,
            second_cancel_receiver,
        ),
    )
    .await
    .expect("capacity rejection does not wait for the native lookup");
    assert!(matches!(second, Err(ResolveFailure::Capacity)));

    cancel_sender
        .send(true)
        .expect("the first lookup still observes its cancellation channel");
    let canceled = tokio::time::timeout(PROMPT_RETURN_LIMIT, &mut first_lookup)
        .await
        .expect("caller-visible cancellation returns promptly")
        .expect("lookup task does not panic");
    assert!(matches!(canceled, Err(ResolveFailure::Cancelled)));
    assert_eq!(semaphore.available_permits(), 0);

    gate.release();
    wait_for_permit(&semaphore, 1).await;
}

#[tokio::test]
async fn deadline_returns_without_waiting_for_started_native_resolution() {
    let (gate, started_receiver) = LookupGate::new();
    let lookup_gate = Arc::clone(&gate);
    let semaphore = Arc::new(Semaphore::new(1));
    let resolver = Resolver::with_lookup_and_governor(
        move |_host, _port| {
            lookup_gate.block_until_released();
            Ok(vec![address(2, 5696)])
        },
        Arc::clone(&semaphore),
    );
    let (_cancel_sender, cancel_receiver) = cancellation_channel();
    let first_resolver = resolver.clone();
    let mut first_lookup = tokio::spawn(async move {
        first_resolver
            .lookup_candidates(
                "slow.example.test",
                5696,
                Instant::now() + Duration::from_millis(250),
                cancel_receiver,
            )
            .await
    });

    tokio::task::spawn_blocking(move || {
        started_receiver
            .recv_timeout(TEST_DEADLINE)
            .expect("the native lookup starts")
    })
    .await
    .expect("started receiver task completes");

    let expired = tokio::time::timeout(PROMPT_RETURN_LIMIT, &mut first_lookup)
        .await
        .expect("deadline is enforced while native resolution remains blocked")
        .expect("lookup task does not panic");
    assert!(matches!(expired, Err(ResolveFailure::Deadline)));
    assert_eq!(semaphore.available_permits(), 0);

    gate.release();
    wait_for_permit(&semaphore, 1).await;
}
