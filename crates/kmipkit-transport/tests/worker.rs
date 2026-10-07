use std::convert::Infallible;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use kmipkit_transport::RequestDeliveryState;

#[path = "../src/worker.rs"]
#[allow(dead_code)]
mod worker;

use worker::{ClientWorker, ExchangeControl, WorkerError, WorkerStartError};

#[test]
fn constructing_worker_does_not_start_network_activity() {
    let worker = ClientWorker::start().expect("worker thread should start");
    let network_starts = Arc::new(AtomicUsize::new(0));

    assert_eq!(network_starts.load(Ordering::SeqCst), 0);

    let observed_starts = Arc::clone(&network_starts);
    worker
        .exchange(None, move |control| async move {
            assert!(control.commit_dispatch());
            observed_starts.fetch_add(1, Ordering::SeqCst);
            Ok::<_, Infallible>(())
        })
        .expect("the first exchange may start the network operation");

    assert_eq!(network_starts.load(Ordering::SeqCst), 1);
}

#[test]
fn worker_serializes_one_in_flight_exchange_and_one_queued_exchange() {
    let worker = Arc::new(ClientWorker::start().expect("worker thread should start"));
    let (first_started_tx, first_started_rx) = mpsc::channel();
    let (first_release_tx, first_release_rx) = tokio::sync::oneshot::channel();
    let first_worker = Arc::clone(&worker);

    let first = thread::spawn(move || {
        first_worker.exchange(None, move |control| async move {
            assert!(control.commit_dispatch());
            first_started_tx
                .send(())
                .expect("test receiver remains open");
            let _ = first_release_rx.await;
            Ok::<_, Infallible>(1)
        })
    });

    first_started_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("first exchange should become active");

    let (second_started_tx, second_started_rx) = mpsc::channel();
    let second_worker = Arc::clone(&worker);
    let second = thread::spawn(move || {
        second_worker.exchange(None, move |control| async move {
            assert!(control.commit_dispatch());
            second_started_tx
                .send(())
                .expect("test receiver remains open");
            Ok::<_, Infallible>(2)
        })
    });

    assert!(
        second_started_rx
            .recv_timeout(Duration::from_millis(50))
            .is_err()
    );
    first_release_tx
        .send(())
        .expect("first exchange is still waiting");
    assert_eq!(first.join().expect("first caller does not panic"), Ok(1));
    second_started_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("queued exchange starts after the first completes");
    assert_eq!(second.join().expect("second caller does not panic"), Ok(2));
}

#[test]
fn synchronous_exchange_is_safe_inside_a_tokio_runtime() {
    let worker = ClientWorker::start().expect("worker thread should start");
    let caller_runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("caller runtime should build");

    let result = caller_runtime.block_on(async {
        worker.exchange(None, |control| async move {
            assert!(control.commit_dispatch());
            Ok::<_, Infallible>(())
        })
    });

    assert_eq!(result, Ok(()));
}

#[test]
fn worker_start_failure_is_returned_without_panicking() {
    let result = ClientWorker::start_with_spawner(|_task| {
        Err(std::io::Error::other("deterministic thread-start failure"))
    });

    assert!(matches!(result, Err(WorkerStartError::Thread)));
}

#[test]
fn an_expired_queued_exchange_returns_not_sent_and_never_dispatches_later() {
    let worker = Arc::new(ClientWorker::start().expect("worker thread should start"));
    let (active_started_tx, active_started_rx) = mpsc::channel();
    let (active_release_tx, active_release_rx) = tokio::sync::oneshot::channel();
    let active_worker = Arc::clone(&worker);
    let active = thread::spawn(move || {
        active_worker.exchange(None, move |control| async move {
            assert!(control.commit_dispatch());
            active_started_tx
                .send(())
                .expect("test receiver remains open");
            let _ = active_release_rx.await;
            Ok::<_, Infallible>(())
        })
    });
    active_started_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("active exchange should start");

    let late_dispatches = Arc::new(AtomicUsize::new(0));
    let queued_worker = Arc::clone(&worker);
    let observed_dispatches = Arc::clone(&late_dispatches);
    let queued = thread::spawn(move || {
        queued_worker.exchange(
            Some(Instant::now() + Duration::from_millis(100)),
            move |control| async move {
                assert!(control.commit_dispatch());
                observed_dispatches.fetch_add(1, Ordering::SeqCst);
                Ok::<_, Infallible>(())
            },
        )
    });

    let queued_result = queued.join().expect("queued caller does not panic");
    assert_eq!(
        queued_result,
        Err(WorkerError::Deadline(RequestDeliveryState::NotSent))
    );

    active_release_tx
        .send(())
        .expect("active exchange is still waiting");
    assert_eq!(active.join().expect("active caller does not panic"), Ok(()));

    worker
        .exchange(
            Some(Instant::now() + Duration::from_secs(1)),
            |control| async move {
                assert!(control.commit_dispatch());
                Ok::<_, Infallible>(())
            },
        )
        .expect("a later barrier exchange drains the expired queue entry");

    assert_eq!(late_dispatches.load(Ordering::SeqCst), 0);
}

#[test]
fn cancellation_and_dispatch_commit_have_one_atomic_winner() {
    for _ in 0..64 {
        let control = ExchangeControl::new();
        assert!(control.begin());
        let barrier = Arc::new(Barrier::new(3));

        let committing_control = control.clone();
        let committing_barrier = Arc::clone(&barrier);
        let commit = thread::spawn(move || {
            committing_barrier.wait();
            committing_control.commit_dispatch()
        });

        let cancelling_control = control.clone();
        let cancelling_barrier = Arc::clone(&barrier);
        let cancel = thread::spawn(move || {
            cancelling_barrier.wait();
            cancelling_control.cancel()
        });

        barrier.wait();
        let dispatch_committed = commit.join().expect("commit race does not panic");
        let delivery = cancel.join().expect("cancel race does not panic");

        if dispatch_committed {
            assert_ne!(delivery, RequestDeliveryState::NotSent);
        } else {
            assert_eq!(delivery, RequestDeliveryState::NotSent);
        }
    }
}

#[test]
fn close_cancels_active_and_queued_work_with_bounded_shutdown() {
    let worker = Arc::new(ClientWorker::start().expect("worker thread should start"));
    let (active_started_tx, active_started_rx) = mpsc::channel();
    let (active_worker, queued_worker) = (Arc::clone(&worker), Arc::clone(&worker));
    let active = thread::spawn(move || {
        active_worker.exchange(None, move |control| async move {
            assert!(control.commit_dispatch());
            active_started_tx
                .send(())
                .expect("test receiver remains open");
            std::future::pending::<Result<(), Infallible>>().await
        })
    });
    active_started_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("active exchange should start");

    let queued_dispatches = Arc::new(AtomicUsize::new(0));
    let observed_dispatches = Arc::clone(&queued_dispatches);
    let queued = thread::spawn(move || {
        queued_worker.exchange(None, move |control| async move {
            assert!(control.commit_dispatch());
            observed_dispatches.fetch_add(1, Ordering::SeqCst);
            Ok::<_, Infallible>(())
        })
    });

    let started = Instant::now();
    worker
        .close(Duration::from_secs(1))
        .expect("async work should be canceled during bounded shutdown");
    assert!(started.elapsed() < Duration::from_secs(1));
    assert_eq!(
        active.join().expect("active caller does not panic"),
        Err(WorkerError::Closed(RequestDeliveryState::PossiblySent))
    );
    assert_eq!(
        queued.join().expect("queued caller does not panic"),
        Err(WorkerError::Closed(RequestDeliveryState::NotSent))
    );
    assert_eq!(queued_dispatches.load(Ordering::SeqCst), 0);
}
