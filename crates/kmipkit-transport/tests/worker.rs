use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier, Condvar, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use super::{
    ClientWorker, Completion, ExchangeCommand, ExchangeControl, QueueSpace, WorkerCommand,
    WorkerError, WorkerShutdownError, WorkerStartError, WorkerThread, lock_unpoisoned,
    wait_timeout_unpoisoned, wait_unpoisoned,
};
use crate::{RequestDeliveryState, TransportCauseCategory, TransportError, TransportResponse};

fn response(marker: u8) -> TransportResponse {
    TransportResponse::new(vec![marker])
}

fn assert_response(result: Result<TransportResponse, WorkerError>, expected: u8) {
    assert_eq!(
        result
            .expect("worker operation should return a response")
            .as_bytes(),
        &[expected]
    );
}

fn assert_worker_error(result: Result<TransportResponse, WorkerError>, expected: WorkerError) {
    assert_eq!(
        result.expect_err("worker operation should return the expected error"),
        expected
    );
}

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
            Ok(response(0))
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
            Ok(response(1))
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
            Ok(response(2))
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
    assert_response(first.join().expect("first caller does not panic"), 1);
    second_started_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("queued exchange starts after the first completes");
    assert_response(second.join().expect("second caller does not panic"), 2);
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
            Ok(response(0))
        })
    });

    assert_response(result, 0);
}

#[test]
fn worker_start_failure_is_returned_without_panicking() {
    let result = ClientWorker::start_with_spawner(|_task| {
        Err(std::io::Error::other("deterministic thread-start failure"))
    });

    assert!(matches!(result, Err(WorkerStartError::Thread)));
}

#[test]
fn runtime_start_failure_and_startup_panic_are_sanitized() {
    let runtime_failure = ClientWorker::start_with_factories(
        |task| thread::Builder::new().spawn(task),
        || Err(std::io::Error::other("runtime startup detail")),
    );
    assert_eq!(runtime_failure.err(), Some(WorkerStartError::Runtime));

    let startup_panic = ClientWorker::start_with_factories(
        |task| thread::Builder::new().spawn(task),
        || -> std::io::Result<_> { panic!("startup panic payload") },
    );
    assert_eq!(startup_panic.err(), Some(WorkerStartError::Runtime));
}

#[test]
fn worker_error_formatting_redacts_operation_source_and_preserves_delivery() {
    let error = WorkerError::Operation {
        delivery_state: RequestDeliveryState::PossiblySent,
        source: TransportError::new(
            RequestDeliveryState::PossiblySent,
            TransportCauseCategory::Other,
            std::io::Error::other("private operation detail"),
        ),
    };

    assert_eq!(error.delivery_state(), RequestDeliveryState::PossiblySent);
    assert!(!format!("{error:?}").contains("private operation detail"));
    assert!(!error.to_string().contains("private operation detail"));
    assert_eq!(
        std::error::Error::source(&error)
            .and_then(|source| source.downcast_ref::<TransportError>())
            .map(TransportError::cause_category),
        Some(TransportCauseCategory::Other)
    );
    let source = error
        .into_operation_source()
        .expect("operation source category is preserved");
    assert_eq!(source.delivery_state(), RequestDeliveryState::PossiblySent);
    assert_eq!(source.cause_category(), TransportCauseCategory::Other);

    let deadline = WorkerError::Deadline(RequestDeliveryState::NotSent);
    let closed = WorkerError::Closed(RequestDeliveryState::NotSent);
    let stopped = WorkerError::Stopped(RequestDeliveryState::NotSent);
    for (error, phrase) in [
        (deadline, "deadline elapsed"),
        (closed, "worker closed"),
        (stopped, "worker stopped"),
    ] {
        assert_ne!(format!("{error:?}"), "");
        assert!(error.to_string().contains(phrase));
        assert_eq!(error.delivery_state(), RequestDeliveryState::NotSent);
        assert!(std::error::Error::source(&error).is_none());
        assert!(error.into_operation_source().is_none());
    }
}

#[test]
fn operation_errors_preserve_the_strongest_delivery_evidence() {
    assert_eq!(
        super::strongest_delivery(
            RequestDeliveryState::ResponseStarted,
            RequestDeliveryState::NotSent,
        ),
        RequestDeliveryState::ResponseStarted
    );
    assert_eq!(
        super::strongest_delivery(
            RequestDeliveryState::NotSent,
            RequestDeliveryState::ResponseStarted,
        ),
        RequestDeliveryState::ResponseStarted
    );
    assert_eq!(
        super::strongest_delivery(
            RequestDeliveryState::PossiblySent,
            RequestDeliveryState::NotSent,
        ),
        RequestDeliveryState::PossiblySent
    );
    assert_eq!(
        super::strongest_delivery(
            RequestDeliveryState::NotSent,
            RequestDeliveryState::PossiblySent,
        ),
        RequestDeliveryState::PossiblySent
    );
    assert_eq!(
        super::strongest_delivery(RequestDeliveryState::NotSent, RequestDeliveryState::NotSent,),
        RequestDeliveryState::NotSent
    );
}

#[test]
fn exchange_control_preserves_response_evidence_when_canceled() {
    let control = ExchangeControl::new();
    assert!(control.begin());
    assert!(control.commit_dispatch());
    assert!(control.response_started());
    assert_eq!(
        control.delivery_state(),
        RequestDeliveryState::ResponseStarted
    );
    assert_eq!(control.cancel(), RequestDeliveryState::ResponseStarted);
    assert_eq!(control.cancel(), RequestDeliveryState::ResponseStarted);
    assert!(!control.commit_dispatch());
    assert!(!control.response_started());

    let finalized = ExchangeControl::new();
    assert_eq!(finalized.cancel(), RequestDeliveryState::NotSent);
    assert!(!finalized.begin());
    assert_eq!(finalized.finish(), (false, RequestDeliveryState::NotSent));

    let preparing = ExchangeControl::new();
    assert!(preparing.begin());
    assert_eq!(preparing.cancel(), RequestDeliveryState::NotSent);

    let possibly_sent = ExchangeControl::new();
    assert!(possibly_sent.begin());
    assert!(possibly_sent.commit_dispatch());
    assert_eq!(possibly_sent.cancel(), RequestDeliveryState::PossiblySent);
    assert_eq!(possibly_sent.cancel(), RequestDeliveryState::PossiblySent);

    let before_finish = ExchangeControl::new();
    assert_eq!(
        before_finish.finish(),
        (true, RequestDeliveryState::NotSent)
    );
    let unknown = ExchangeControl::new();
    unknown.state.store(u8::MAX, Ordering::Release);
    assert_eq!(unknown.delivery_state(), RequestDeliveryState::NotSent);
    assert_eq!(unknown.cancel(), RequestDeliveryState::NotSent);
}

#[test]
fn concurrent_completion_and_cancellation_retry_the_atomic_gate() {
    for _ in 0..16 {
        let control = ExchangeControl::new();
        assert!(control.begin());
        let barrier = Arc::new(Barrier::new(9));
        let racers = (0..8)
            .map(|index| {
                let control = control.clone();
                let barrier = Arc::clone(&barrier);
                thread::spawn(move || {
                    barrier.wait();
                    if index % 2 == 0 {
                        control.finish().1
                    } else {
                        control.cancel()
                    }
                })
            })
            .collect::<Vec<_>>();

        barrier.wait();
        for racer in racers {
            assert_eq!(
                racer.join().expect("terminal-state race does not panic"),
                RequestDeliveryState::NotSent
            );
        }
    }

    let control = ExchangeControl::new();
    assert!(control.begin());
    let loaded = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let completing_control = control.clone();
    let completing_loaded = Arc::clone(&loaded);
    let completing_release = Arc::clone(&release);
    let completing = thread::spawn(move || {
        let mut first_load = true;
        completing_control.finish_after_load(|| {
            if first_load {
                first_load = false;
                completing_loaded.wait();
                completing_release.wait();
            }
        })
    });

    loaded.wait();
    assert_eq!(control.cancel(), RequestDeliveryState::NotSent);
    release.wait();
    assert_eq!(
        completing.join().expect("finish retry does not panic"),
        (false, RequestDeliveryState::NotSent)
    );
}

#[test]
fn a_worker_command_observes_expiry_cancellation_and_shutdown() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("test runtime should build");

    let (expired, expired_result, _) = make_command(
        Some(
            Instant::now()
                .checked_sub(Duration::from_millis(1))
                .expect("the process has been running for at least one millisecond"),
        ),
        ExchangeControl::new(),
        |_| async { Ok(response(0)) },
    );
    let (_shutdown_sender, shutdown_receiver) = tokio::sync::watch::channel(false);
    runtime.block_on(expired.run(shutdown_receiver));
    assert_worker_error(
        expired_result.recv().expect("command returns a result"),
        WorkerError::Deadline(RequestDeliveryState::NotSent),
    );

    let canceled_control = ExchangeControl::new();
    assert_eq!(canceled_control.cancel(), RequestDeliveryState::NotSent);
    let (canceled, canceled_result, _) =
        make_command(None, canceled_control, |_| async { Ok(response(0)) });
    let (_shutdown_sender, shutdown_receiver) = tokio::sync::watch::channel(false);
    runtime.block_on(canceled.run(shutdown_receiver));
    assert_worker_error(
        canceled_result.recv().expect("command returns a result"),
        WorkerError::Deadline(RequestDeliveryState::NotSent),
    );

    let cancel_during_operation = ExchangeControl::new();
    let (completed_after_cancel, completed_result, _) =
        make_command(None, cancel_during_operation, |control| async move {
            control.cancel();
            Ok(response(0))
        });
    let (_shutdown_sender, shutdown_receiver) = tokio::sync::watch::channel(false);
    runtime.block_on(completed_after_cancel.run(shutdown_receiver));
    assert_worker_error(
        completed_result
            .recv()
            .expect("command returns a result after racing cancellation"),
        WorkerError::Deadline(RequestDeliveryState::NotSent),
    );

    let (_shutdown_sender, shutdown_receiver) = tokio::sync::watch::channel(true);
    let (closed, closed_result, _) =
        make_command(None, ExchangeControl::new(), |_| async { Ok(response(0)) });
    runtime.block_on(closed.run(shutdown_receiver));
    assert_worker_error(
        closed_result.recv().expect("command returns a result"),
        WorkerError::Closed(RequestDeliveryState::NotSent),
    );

    let (timed, timed_result, _) = make_command(
        Some(Instant::now() + Duration::from_millis(10)),
        ExchangeControl::new(),
        |_| async { std::future::pending::<Result<TransportResponse, TransportError>>().await },
    );
    let (_shutdown_sender, shutdown_receiver) = tokio::sync::watch::channel(false);
    runtime.block_on(timed.run(shutdown_receiver));
    assert_worker_error(
        timed_result.recv().expect("command returns a result"),
        WorkerError::Deadline(RequestDeliveryState::NotSent),
    );

    let cancel_control = ExchangeControl::new();
    let caller_control = cancel_control.clone();
    let (canceled, canceled_result, _) = make_command(None, cancel_control, |_| async {
        std::future::pending::<Result<TransportResponse, TransportError>>().await
    });
    let (_shutdown_sender, shutdown_receiver) = tokio::sync::watch::channel(false);
    runtime.block_on(async move {
        tokio::spawn(async move {
            tokio::task::yield_now().await;
            caller_control.cancel();
        });
        canceled.run(shutdown_receiver).await;
    });
    assert_worker_error(
        canceled_result.recv().expect("command returns a result"),
        WorkerError::Deadline(RequestDeliveryState::NotSent),
    );

    let (failed, failed_result, _) =
        make_command(None, ExchangeControl::new(), |control| async move {
            assert!(control.commit_dispatch());
            Err::<TransportResponse, _>(TransportError::new(
                RequestDeliveryState::PossiblySent,
                TransportCauseCategory::Other,
                std::io::Error::other("private operation detail"),
            ))
        });
    let (_shutdown_sender, shutdown_receiver) = tokio::sync::watch::channel(false);
    runtime.block_on(failed.run(shutdown_receiver));
    let operation_error = failed_result
        .recv()
        .expect("command returns its operation error")
        .expect_err("operation error is propagated safely");
    assert!(
        !operation_error
            .to_string()
            .contains("private operation detail")
    );
}

#[test]
fn completion_that_wins_before_shutdown_keeps_its_result() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("test runtime should build");
    let (shutdown_sender, shutdown_receiver) = tokio::sync::watch::channel(false);
    let (completion_committed_sender, completion_committed_receiver) =
        tokio::sync::oneshot::channel();
    let (release_operation_sender, release_operation_receiver) = tokio::sync::oneshot::channel();
    let (command, result, control) =
        make_command(None, ExchangeControl::new(), move |control| async move {
            assert!(control.commit_dispatch());
            assert!(control.response_started());
            assert!(
                control.finish().0,
                "operation completion wins the atomic gate"
            );
            completion_committed_sender
                .send(())
                .expect("the test observes completion");
            let _ = release_operation_receiver.await;
            Ok(response(7))
        });
    let cancel_events = control.subscribe_cancel();

    runtime.block_on(async move {
        tokio::spawn(async move {
            completion_committed_receiver
                .await
                .expect("the operation commits completion");
            shutdown_sender.send_replace(true);
            let _ = release_operation_sender.send(());
        });
        command.run(shutdown_receiver).await;
    });

    assert_response(
        result
            .recv()
            .expect("the command returns its completed result"),
        7,
    );
    assert_eq!(control.cancel(), RequestDeliveryState::ResponseStarted);
    assert_eq!(
        control.finish(),
        (true, RequestDeliveryState::ResponseStarted)
    );
    assert_eq!(
        control.delivery_state(),
        RequestDeliveryState::ResponseStarted
    );
    assert!(
        !cancel_events
            .has_changed()
            .expect("completed control remains connected")
    );
}

#[test]
fn shutdown_wait_is_bounded_when_an_operation_blocks_its_worker_thread() {
    let worker = Arc::new(ClientWorker::start().expect("worker thread should start"));
    let (started_tx, started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let caller_worker = Arc::clone(&worker);
    let caller = thread::spawn(move || {
        caller_worker.exchange(None, move |control| async move {
            assert!(control.commit_dispatch());
            started_tx.send(()).expect("test receiver remains open");
            release_rx.recv().expect("test releases blocked operation");
            Ok(response(0))
        })
    });
    started_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("operation should be active");

    assert_eq!(
        worker.close(Duration::from_millis(20)),
        Err(WorkerShutdownError::TimedOut)
    );
    release_tx.send(()).expect("worker operation is blocked");
    assert_worker_error(
        caller.join().expect("exchange caller does not panic"),
        WorkerError::Closed(RequestDeliveryState::PossiblySent),
    );
    worker
        .close(Duration::from_secs(1))
        .expect("worker exits after the operation is released");
}

#[test]
fn shutdown_does_not_wait_for_a_started_blocking_resolver_job() {
    let worker = Arc::new(ClientWorker::start().expect("worker thread should start"));
    let (started_tx, started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let caller_worker = Arc::clone(&worker);
    let caller = thread::spawn(move || {
        caller_worker.exchange(None, move |_control| async move {
            let blocking = tokio::task::spawn_blocking(move || {
                started_tx.send(()).expect("test receiver remains open");
                release_rx
                    .recv()
                    .expect("test releases the blocking lookup");
            });
            blocking.await.expect("blocking test job completes");
            Ok(response(0))
        })
    });
    started_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("native resolver job should be active");

    let started = Instant::now();
    let shutdown = worker.close(Duration::from_millis(100));
    let elapsed = started.elapsed();

    release_tx
        .send(())
        .expect("blocking test job remains active until released");
    assert_worker_error(
        caller.join().expect("exchange caller does not panic"),
        WorkerError::Closed(RequestDeliveryState::NotSent),
    );
    worker
        .close(Duration::from_secs(1))
        .expect("worker thread finishes after the detached lookup exits");

    assert!(elapsed < Duration::from_secs(1));
    assert_eq!(shutdown, Ok(()));
}

#[test]
fn worker_operation_panic_is_contained_and_returns_delivery_evidence() {
    let worker = ClientWorker::start().expect("worker thread should start");
    let result = worker.exchange(None, |control| async move {
        assert!(control.commit_dispatch());
        panic!("test-only operation panic");
        #[allow(unreachable_code)]
        Ok(response(0))
    });

    assert!(matches!(
        result,
        Err(WorkerError::Stopped(RequestDeliveryState::PossiblySent))
    ));
    assert_worker_error(
        worker.exchange(None, |_| async { Ok(response(0)) }),
        WorkerError::Stopped(RequestDeliveryState::NotSent),
    );
    worker
        .close(Duration::from_secs(1))
        .expect("panic-contained worker has terminated");
}

#[test]
fn queue_space_and_completion_waiters_observe_notifications_and_deadlines() {
    let queue_space = Arc::new(QueueSpace::default());
    assert!(!queue_space.wait_for_change(0, Some(Instant::now())));
    assert!(!queue_space.wait_for_change(0, Some(Instant::now() + Duration::from_millis(5))));

    let waiting_space = Arc::clone(&queue_space);
    let notifier = thread::spawn(move || {
        thread::sleep(Duration::from_millis(5));
        waiting_space.notify_slot_available();
    });
    assert!(queue_space.wait_for_change(0, None));
    notifier.join().expect("queue notifier does not panic");

    let timed_space = Arc::new(QueueSpace::default());
    let timed_notifier_space = Arc::clone(&timed_space);
    let timed_notifier = thread::spawn(move || {
        thread::sleep(Duration::from_millis(5));
        timed_notifier_space.notify_slot_available();
    });
    assert!(timed_space.wait_for_change(0, Some(Instant::now() + Duration::from_secs(1))));
    timed_notifier
        .join()
        .expect("timed queue notifier does not panic");

    let completion = Completion::default();
    assert!(!completion.wait(Duration::ZERO));
    completion.mark_finished();
    assert!(completion.wait(Duration::ZERO));

    let signaled_completion = Arc::new(Completion::default());
    let waiter_completion = Arc::clone(&signaled_completion);
    let completion_notifier = thread::spawn(move || {
        thread::sleep(Duration::from_millis(5));
        waiter_completion.changed.notify_all();
        thread::sleep(Duration::from_millis(5));
        waiter_completion.mark_finished();
    });
    assert!(signaled_completion.wait(Duration::from_secs(1)));
    completion_notifier
        .join()
        .expect("completion notifier does not panic");

    let unexpected_panic = thread::spawn(|| panic!("test-only join failure"));
    let join = WorkerThread::new(unexpected_panic, Arc::new(Completion::default()));
    join.completion.mark_finished();
    assert_eq!(
        join.wait_and_join(Duration::from_millis(1)),
        Err(WorkerShutdownError::Panicked)
    );
}

#[test]
fn a_full_queue_waiter_expires_without_being_admitted() {
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
            Ok(response(0))
        })
    });
    active_started_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("first exchange should be active");

    let (queued_sender, queued_receiver) = mpsc::sync_channel(1);
    let queued_control = ExchangeControl::new();
    let queued_command = Box::new(ExchangeCommand {
        deadline: None,
        control: queued_control.clone(),
        operation: super::boxed_operation(|control: ExchangeControl| async move {
            assert!(control.commit_dispatch());
            Ok(response(2))
        }),
        result: queued_sender,
    });
    worker
        .enqueue(queued_command, &queued_control, None)
        .expect("the one waiting slot accepts one command");

    let late_dispatches = Arc::new(AtomicUsize::new(0));
    let observed_dispatches = Arc::clone(&late_dispatches);
    assert_worker_error(
        worker.exchange(
            Some(Instant::now() + Duration::from_millis(20)),
            move |control| async move {
                assert!(control.commit_dispatch());
                observed_dispatches.fetch_add(1, Ordering::SeqCst);
                Ok(response(0))
            },
        ),
        WorkerError::Deadline(RequestDeliveryState::NotSent),
    );

    active_release_tx
        .send(())
        .expect("active exchange is still waiting");
    assert_response(active.join().expect("active caller does not panic"), 0);
    assert_response(
        queued_receiver
            .recv_timeout(Duration::from_secs(1))
            .expect("queued operation should complete"),
        2,
    );
    assert_eq!(late_dispatches.load(Ordering::SeqCst), 0);
    worker
        .close(Duration::from_secs(1))
        .expect("worker shuts down cleanly");
    assert_worker_error(
        worker.exchange(None, |_| async { Ok(response(0)) }),
        WorkerError::Closed(RequestDeliveryState::NotSent),
    );
}

#[test]
fn a_full_queue_waiter_is_admitted_after_the_active_command_releases_capacity() {
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
            Ok(response(0))
        })
    });
    active_started_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("first exchange should be active");

    let (queued_sender, queued_receiver) = mpsc::sync_channel(1);
    let queued_control = ExchangeControl::new();
    let queued_command = Box::new(ExchangeCommand {
        deadline: None,
        control: queued_control.clone(),
        operation: super::boxed_operation(|control: ExchangeControl| async move {
            assert!(control.commit_dispatch());
            Ok(response(2))
        }),
        result: queued_sender,
    });
    worker
        .enqueue(queued_command, &queued_control, None)
        .expect("the one waiting slot accepts one command");

    let queued_worker = Arc::clone(&worker);
    let (third_called_tx, third_called_rx) = mpsc::channel();
    let third = thread::spawn(move || {
        third_called_tx
            .send(())
            .expect("test receiver remains open");
        queued_worker.exchange(
            Some(Instant::now() + Duration::from_secs(1)),
            |control| async move {
                assert!(control.commit_dispatch());
                Ok(response(3))
            },
        )
    });
    third_called_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("third caller is ready to wait for a slot");
    thread::sleep(Duration::from_millis(10));

    active_release_tx
        .send(())
        .expect("active exchange is still waiting");
    assert_response(active.join().expect("active caller does not panic"), 0);
    assert_response(
        queued_receiver
            .recv_timeout(Duration::from_secs(1))
            .expect("second exchange should complete"),
        2,
    );
    assert_response(third.join().expect("third caller does not panic"), 3);
    worker
        .close(Duration::from_secs(1))
        .expect("worker shuts down cleanly");
}

#[test]
fn runtime_loop_exits_when_shutdown_or_channel_close_wins() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("test runtime should build");
    let (command, command_result, _) =
        make_command(None, ExchangeControl::new(), |_| async { Ok(response(0)) });
    let (sender, receiver) = tokio::sync::mpsc::channel(1);
    sender
        .try_send(command)
        .expect("pending command fits the bounded queue");
    let queue_space = Arc::new(QueueSpace::default());
    let (shutdown_sender, shutdown_receiver) = tokio::sync::watch::channel(true);
    super::run_runtime(
        runtime,
        receiver,
        shutdown_receiver,
        Arc::clone(&queue_space),
    );
    assert_worker_error(
        command_result
            .recv_timeout(Duration::from_millis(1))
            .expect("shutdown rejects the pending command"),
        WorkerError::Closed(RequestDeliveryState::NotSent),
    );
    assert_eq!(queue_space.generation(), 1);
    drop(sender);
    drop(shutdown_sender);

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("test runtime should build");
    let (sender, receiver) = tokio::sync::mpsc::channel(1);
    drop(sender);
    let (_shutdown_sender, shutdown_receiver) = tokio::sync::watch::channel(false);
    super::run_runtime(
        runtime,
        receiver,
        shutdown_receiver,
        Arc::new(QueueSpace::default()),
    );
}

#[test]
fn runtime_loop_observes_shutdown_while_waiting_for_a_command() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("test runtime should build");
    let (sender, receiver) = tokio::sync::mpsc::channel(1);
    let (shutdown_sender, shutdown_receiver) = tokio::sync::watch::channel(false);

    runtime.block_on(async move {
        tokio::spawn(async move {
            tokio::task::yield_now().await;
            shutdown_sender.send_replace(true);
        });
        super::runtime_loop(receiver, shutdown_receiver, Arc::new(QueueSpace::default())).await;
    });
    drop(sender);
}

#[test]
fn result_wait_reports_disconnection_with_and_without_a_deadline() {
    let (sender, receiver) = mpsc::sync_channel::<Result<TransportResponse, WorkerError>>(1);
    drop(sender);
    assert!(matches!(
        super::receive_result(&receiver, Some(Instant::now() + Duration::from_secs(1))),
        Err(super::ReceiveFailure::Disconnected)
    ));

    let (sender, receiver) = mpsc::sync_channel::<Result<TransportResponse, WorkerError>>(1);
    drop(sender);
    assert!(matches!(
        super::receive_result(&receiver, None),
        Err(super::ReceiveFailure::Disconnected)
    ));
}

#[test]
fn worker_lifecycle_errors_have_safe_display() {
    for (error, phrase) in [
        (WorkerStartError::Thread, "thread could not start"),
        (WorkerStartError::Runtime, "runtime could not start"),
    ] {
        assert!(error.to_string().contains(phrase));
    }
    assert!(
        WorkerShutdownError::TimedOut
            .to_string()
            .contains("shutdown timed out")
    );
    assert!(
        WorkerShutdownError::Panicked
            .to_string()
            .contains("stopped unexpectedly")
    );
}

#[test]
fn poisoned_synchronization_helpers_recover_without_panicking() {
    let mutex = Arc::new(std::sync::Mutex::new(()));
    let poisoned_mutex = Arc::clone(&mutex);
    let _ = thread::spawn(move || {
        let _guard = poisoned_mutex.lock().expect("mutex begins unpoisoned");
        panic!("test-only mutex poison");
    })
    .join();
    drop(lock_unpoisoned(&mutex));

    let condition = Arc::new(Condvar::new());
    let guard = lock_unpoisoned(&mutex);
    let notify = thread::spawn({
        let condition = Arc::clone(&condition);
        move || {
            thread::sleep(Duration::from_millis(5));
            condition.notify_one();
        }
    });
    drop(wait_unpoisoned(&condition, guard));
    notify.join().expect("condition notifier does not panic");

    let guard = lock_unpoisoned(&mutex);
    let notify = thread::spawn({
        let condition = Arc::clone(&condition);
        move || {
            thread::sleep(Duration::from_millis(5));
            condition.notify_one();
        }
    });
    let (guard, timed_out) = wait_timeout_unpoisoned(&condition, guard, Duration::from_secs(1));
    drop(guard);
    assert!(!timed_out);
    notify.join().expect("condition notifier does not panic");
}

fn make_command<F, Fut>(
    deadline: Option<Instant>,
    control: ExchangeControl,
    operation: F,
) -> (
    Box<dyn WorkerCommand>,
    mpsc::Receiver<Result<TransportResponse, WorkerError>>,
    ExchangeControl,
)
where
    F: FnOnce(ExchangeControl) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Result<TransportResponse, TransportError>> + Send + 'static,
{
    let (result_sender, result_receiver) = mpsc::sync_channel(1);
    let command = Box::new(ExchangeCommand {
        deadline,
        control: control.clone(),
        operation: super::boxed_operation(operation),
        result: result_sender,
    });
    (command, result_receiver, control)
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
            Ok(response(0))
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
                Ok(response(0))
            },
        )
    });

    let queued_result = queued.join().expect("queued caller does not panic");
    assert_worker_error(
        queued_result,
        WorkerError::Deadline(RequestDeliveryState::NotSent),
    );

    active_release_tx
        .send(())
        .expect("active exchange is still waiting");
    assert_response(active.join().expect("active caller does not panic"), 0);

    worker
        .exchange(
            Some(Instant::now() + Duration::from_secs(1)),
            |control| async move {
                assert!(control.commit_dispatch());
                Ok(response(0))
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
            std::future::pending::<Result<TransportResponse, TransportError>>().await
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
            Ok(response(0))
        })
    });

    let started = Instant::now();
    worker
        .close(Duration::from_secs(1))
        .expect("async work should be canceled during bounded shutdown");
    assert!(started.elapsed() < Duration::from_secs(1));
    assert_worker_error(
        active.join().expect("active caller does not panic"),
        WorkerError::Closed(RequestDeliveryState::PossiblySent),
    );
    assert_worker_error(
        queued.join().expect("queued caller does not panic"),
        WorkerError::Closed(RequestDeliveryState::NotSent),
    );
    assert_eq!(queued_dispatches.load(Ordering::SeqCst), 0);
}
