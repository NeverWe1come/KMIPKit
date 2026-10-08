//! Deterministic lifecycle tests for the native system resolver adapter.

use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex as StdMutex, mpsc};
use std::time::{Duration, Instant};

use tokio::runtime::Builder;
use tokio::sync::{Semaphore, watch};

use super::{
    Lookup, ResolveFailure, Resolver, ResolverJobState, collect_candidates, finish_lookup,
    lock_job_state, run_lookup, wait_for_cancel, wait_until,
};

const TEST_DEADLINE: Duration = Duration::from_secs(2);
const PROMPT_RETURN_LIMIT: Duration = Duration::from_secs(1);

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

struct CountingSocketAddresses {
    next_octet: u8,
    end_octet: u8,
    next_calls: Arc<AtomicUsize>,
}

impl Iterator for CountingSocketAddresses {
    type Item = SocketAddr;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_calls.fetch_add(1, Ordering::Relaxed);
        if self.next_octet > self.end_octet {
            return None;
        }

        let address = address(self.next_octet, 5696);
        self.next_octet += 1;
        Some(address)
    }
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

    assert_ne!(candidates.len(), 0);
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
    let first = Resolver::system();
    let second = Resolver::system();

    assert!(Arc::ptr_eq(first.governor(), second.governor()));
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

#[test]
fn candidate_collection_consumes_only_the_first_sixteen_addresses() {
    let next_calls = Arc::new(AtomicUsize::new(0));
    let candidates = collect_candidates(CountingSocketAddresses {
        next_octet: 1,
        end_octet: 20,
        next_calls: Arc::clone(&next_calls),
    });
    let expected = (1..=16)
        .map(|octet| address(octet, 5696))
        .collect::<Vec<_>>();

    assert_eq!(
        candidates, expected,
        "the first sixteen addresses retain OS order"
    );
    assert_eq!(next_calls.load(Ordering::Relaxed), 16);
}

#[tokio::test]
async fn shared_governor_caps_active_resolver_jobs_at_thirty_two() {
    let (started_sender, started_receiver) = mpsc::sync_channel(32);
    let gate = Arc::new(LookupGate {
        started: started_sender,
        released: (StdMutex::new(false), Condvar::new()),
    });
    let governor = Arc::new(Semaphore::new(32));
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
                .expect("each admitted resolver job starts");
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
    let candidate_attempts = Arc::new(AtomicUsize::new(0));
    let observed_attempts = Arc::clone(&candidate_attempts);
    let (cancel_sender, cancel_receiver) = cancellation_channel();
    let first_resolver = resolver.clone();
    let mut first_lookup = tokio::spawn(async move {
        let outcome = first_resolver
            .lookup_candidates(
                "blocked.example.test",
                5696,
                Instant::now() + TEST_DEADLINE,
                cancel_receiver,
            )
            .await;
        if let Ok(candidates) = &outcome {
            observed_attempts.fetch_add(candidates.len(), Ordering::Relaxed);
        }
        outcome
    });

    tokio::task::spawn_blocking(move || {
        started_receiver
            .recv_timeout(TEST_DEADLINE)
            .expect("the native lookup starts");
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
    assert_eq!(candidate_attempts.load(Ordering::Relaxed), 0);
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
    let candidate_attempts = Arc::new(AtomicUsize::new(0));
    let observed_attempts = Arc::clone(&candidate_attempts);
    let (_cancel_sender, cancel_receiver) = cancellation_channel();
    let first_resolver = resolver.clone();
    let mut first_lookup = tokio::spawn(async move {
        let outcome = first_resolver
            .lookup_candidates(
                "slow.example.test",
                5696,
                Instant::now() + Duration::from_millis(250),
                cancel_receiver,
            )
            .await;
        if let Ok(candidates) = &outcome {
            observed_attempts.fetch_add(candidates.len(), Ordering::Relaxed);
        }
        outcome
    });

    tokio::task::spawn_blocking(move || {
        started_receiver
            .recv_timeout(TEST_DEADLINE)
            .expect("the native lookup starts");
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
    assert_eq!(candidate_attempts.load(Ordering::Relaxed), 0);
}

#[tokio::test]
async fn cancellation_and_expired_deadline_are_rejected_before_admission() {
    let calls = Arc::new(AtomicUsize::new(0));
    let lookup_calls = Arc::clone(&calls);
    let governor = Arc::new(Semaphore::new(1));
    let resolver = Resolver::with_lookup_and_governor(
        move |_host, _port| {
            lookup_calls.fetch_add(1, Ordering::Relaxed);
            Ok(vec![address(1, 5696)])
        },
        Arc::clone(&governor),
    );
    let (cancel_sender, cancel_receiver) = cancellation_channel();
    cancel_sender
        .send(true)
        .expect("the test cancellation receiver remains open");
    assert_eq!(
        resolver
            .lookup_candidates(
                "cancelled.example.test",
                5696,
                Instant::now(),
                cancel_receiver
            )
            .await,
        Err(ResolveFailure::Cancelled)
    );

    let (_deadline_sender, deadline_receiver) = cancellation_channel();
    assert_eq!(
        resolver
            .lookup_candidates(
                "expired.example.test",
                5696,
                Instant::now(),
                deadline_receiver
            )
            .await,
        Err(ResolveFailure::Deadline)
    );
    assert_eq!(calls.load(Ordering::Relaxed), 0);
    assert_eq!(governor.available_permits(), 1);
}

#[tokio::test]
async fn native_lookup_failures_and_empty_results_are_sanitized() {
    let failing = Resolver::with_lookup_and_governor(
        |_host, _port| Err(io::Error::other("sensitive resolver detail")),
        Arc::new(Semaphore::new(1)),
    );
    let (_cancel_sender, cancel_receiver) = cancellation_channel();
    assert_eq!(
        failing
            .lookup_candidates(
                "failure.example.test",
                5696,
                Instant::now() + TEST_DEADLINE,
                cancel_receiver,
            )
            .await,
        Err(ResolveFailure::Lookup)
    );

    let empty = Resolver::with_lookup_and_governor(
        |_host, _port| Ok(Vec::new()),
        Arc::new(Semaphore::new(1)),
    );
    let (_cancel_sender, cancel_receiver) = cancellation_channel();
    assert_eq!(
        empty
            .lookup_candidates(
                "empty.example.test",
                5696,
                Instant::now() + TEST_DEADLINE,
                cancel_receiver,
            )
            .await,
        Err(ResolveFailure::Lookup)
    );

    let panicking = Resolver::with_lookup_and_governor(
        |_host, _port| -> io::Result<Vec<SocketAddr>> { panic!("resolver test panic") },
        Arc::new(Semaphore::new(1)),
    );
    let (_cancel_sender, cancel_receiver) = cancellation_channel();
    assert_eq!(
        panicking
            .lookup_candidates(
                "panic.example.test",
                5696,
                Instant::now() + TEST_DEADLINE,
                cancel_receiver,
            )
            .await,
        Err(ResolveFailure::Lookup)
    );
}

#[test]
fn completed_results_recheck_deadline_cancellation_and_candidate_presence() {
    let address = address(4, 5696);
    let now = Instant::now();
    let later = now + Duration::from_secs(1);
    assert_eq!(
        finish_lookup(vec![address], later, false, now),
        Ok(vec![address])
    );
    assert_eq!(
        finish_lookup(vec![address], now, false, now),
        Err(ResolveFailure::Deadline)
    );
    assert_eq!(
        finish_lookup(vec![address], later, true, now),
        Err(ResolveFailure::Cancelled)
    );
    assert_eq!(
        finish_lookup(Vec::new(), later, false, now),
        Err(ResolveFailure::Lookup)
    );
}

#[tokio::test]
async fn cancellation_wait_ends_when_its_sender_is_dropped() {
    let (cancel_sender, mut cancel_receiver) = watch::channel(false);
    drop(cancel_sender);
    tokio::time::timeout(PROMPT_RETURN_LIMIT, wait_for_cancel(&mut cancel_receiver))
        .await
        .expect("a closed cancellation channel ends its wait");
}

#[test]
fn dropping_a_lookup_aborts_it_while_queued_on_the_only_blocking_thread() {
    let runtime = Builder::new_current_thread()
        .enable_all()
        .max_blocking_threads(1)
        .build()
        .expect("test runtime should build");
    let (blocking_gate, blocking_started) = LookupGate::new();
    let blocker_gate = Arc::clone(&blocking_gate);
    let lookup_starts = Arc::new(AtomicUsize::new(0));
    let observed_starts = Arc::clone(&lookup_starts);
    let governor = Arc::new(Semaphore::new(1));
    let resolver = Resolver::with_lookup_and_governor(
        move |_host, _port| {
            observed_starts.fetch_add(1, Ordering::Relaxed);
            Ok(vec![address(7, 5696)])
        },
        Arc::clone(&governor),
    );

    runtime.block_on(async move {
        let blocker = tokio::task::spawn_blocking(move || blocker_gate.block_until_released());
        blocking_started
            .recv_timeout(TEST_DEADLINE)
            .expect("the sole blocking thread is occupied");

        let (_cancel_sender, cancel_receiver) = cancellation_channel();
        let queued_resolver = resolver.clone();
        let queued_lookup = tokio::spawn(async move {
            queued_resolver
                .lookup_candidates(
                    "queued.example.test",
                    5696,
                    Instant::now() + TEST_DEADLINE,
                    cancel_receiver,
                )
                .await
        });
        tokio::time::timeout(TEST_DEADLINE, async {
            while governor.available_permits() != 0 {
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        })
        .await
        .expect("queued resolver owns its governor permit");

        queued_lookup.abort();
        assert!(
            queued_lookup
                .await
                .expect_err("lookup task was aborted")
                .is_cancelled()
        );
        let permit_returned_before_blocker_release =
            tokio::time::timeout(Duration::from_millis(250), wait_for_permit(&governor, 1))
                .await
                .is_ok();

        blocking_gate.release();
        blocker.await.expect("blocking gate exits after release");
        wait_for_permit(&governor, 1).await;

        assert!(permit_returned_before_blocker_release);
        assert_eq!(lookup_starts.load(Ordering::Relaxed), 0);
    });
}

#[test]
fn a_queued_job_released_by_its_guard_never_calls_the_resolver() {
    let calls = Arc::new(AtomicUsize::new(0));
    let observed_calls = Arc::clone(&calls);
    let lookup: Arc<Lookup> = Arc::new(move |_host, _port| {
        observed_calls.fetch_add(1, Ordering::Relaxed);
        Ok(vec![address(8, 5696)])
    });
    let state = std::sync::Mutex::new(ResolverJobState {
        started: false,
        permit: None,
    });

    let result = run_lookup(&state, lookup.as_ref(), "canceled.example.test", 5696);

    assert_eq!(
        result
            .expect_err("canceled queued lookup must not run")
            .kind(),
        io::ErrorKind::Interrupted
    );
    assert_eq!(calls.load(Ordering::Relaxed), 0);
    assert!(super::lock_job_state(&state).started);
}

#[test]
fn poisoned_resolver_state_lock_recovers_the_job_state() {
    let state = StdMutex::new(ResolverJobState {
        started: false,
        permit: None,
    });
    let poisoned = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _guard = state.lock().expect("the resolver state starts unlocked");
        panic!("inject mutex poisoning for the recovery test");
    }));
    assert!(poisoned.is_err());

    assert!(!lock_job_state(&state).started);
}

#[tokio::test]
async fn unbounded_resolver_deadline_wait_remains_pending() {
    let result = tokio::time::timeout(Duration::from_millis(1), wait_until(None)).await;

    assert!(result.is_err(), "an unbounded deadline must not fire");
}
