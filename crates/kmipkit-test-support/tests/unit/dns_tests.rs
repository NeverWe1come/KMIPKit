use super::{
    DnsQueryType, FixtureMetrics, FixtureState, LocalDnsFixture, MAX_QUERY_BYTES, TcpMetrics,
    TcpWorkerKind, TcpWorkerSpawner, build_response, prepare_accepted_tcp_stream, read_u16,
    serve_tcp_with_spawner_and_cloner, validate_records,
};
use std::collections::BTreeMap;
use std::io::{self, Read, Write};
use std::net::{IpAddr, Ipv4Addr, Shutdown, TcpListener, TcpStream, UdpSocket};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::thread;
use std::time::Duration;

static NETWORK_FIXTURE_LOCK: Mutex<()> = Mutex::new(());

fn lock_network_fixture() -> MutexGuard<'static, ()> {
    match NETWORK_FIXTURE_LOCK.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn is_close_result(result: &io::Result<usize>) -> bool {
    match result {
        Ok(0) => true,
        Err(error) => matches!(
            error.kind(),
            io::ErrorKind::ConnectionAborted | io::ErrorKind::ConnectionReset
        ),
        Ok(_) => false,
    }
}

#[test]
fn rejects_duplicate_names_after_case_and_trailing_dot_normalization() {
    let records = BTreeMap::from([
        (
            "fixture.kmipkit.test".to_owned(),
            vec![IpAddr::V4(Ipv4Addr::LOCALHOST)],
        ),
        (
            "FIXTURE.KMIPKIT.TEST.".to_owned(),
            vec![IpAddr::V4(Ipv4Addr::LOCALHOST)],
        ),
    ]);

    let error = validate_records(records).expect_err("normalized names must be unique");

    assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
}

#[test]
fn rejects_invalid_names_empty_address_sets_and_too_many_addresses() {
    let invalid_inputs = [
        BTreeMap::from([("bad name".to_owned(), vec![IpAddr::V4(Ipv4Addr::LOCALHOST)])]),
        BTreeMap::from([("empty.kmipkit.test".to_owned(), Vec::new())]),
        BTreeMap::from([(
            "too-many.kmipkit.test".to_owned(),
            vec![IpAddr::V4(Ipv4Addr::LOCALHOST); 33],
        )]),
    ];

    for records in invalid_inputs {
        let error = validate_records(records).expect_err("invalid fixture data is rejected");
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
    }
}

#[test]
fn malformed_dns_queries_are_ignored_without_panicking() {
    let records = BTreeMap::new();
    assert!(build_response(&[], &records, false).is_none());
    assert!(build_response(&[0; 513], &records, false).is_none());

    let mut response_as_query = query(&[1, b'a', 0], 1, 1);
    response_as_query[2] = 0x80;
    assert!(build_response(&response_as_query, &records, false).is_none());

    let mut multiple_questions = query(&[1, b'a', 0], 1, 1);
    multiple_questions[5] = 2;
    assert!(build_response(&multiple_questions, &records, false).is_none());

    assert!(build_response(&query(&[64; 1], 1, 1), &records, false).is_none());
    assert!(build_response(&query(&[1, 0xff, 0], 1, 1), &records, false).is_none());
    assert!(build_response(&query(&[1, b'a'], 1, 1), &records, false).is_none());
    assert!(build_response(&query(&[0], 1, 1), &records, false).is_none());
    assert_eq!(read_u16(&[], 0), None);
}

#[test]
fn unsupported_record_types_return_a_valid_empty_answer() {
    let records = BTreeMap::from([(
        "fixture.kmipkit.test".to_owned(),
        vec![IpAddr::V4(Ipv4Addr::LOCALHOST)],
    )]);
    let request = query(
        &[
            7, b'f', b'i', b'x', b't', b'u', b'r', b'e', 7, b'k', b'm', b'i', b'p', b'k', b'i',
            b't', 4, b't', b'e', b's', b't', 0,
        ],
        15,
        1,
    );

    let response = build_response(&request, &records, false).expect("valid query receives a reply");

    assert_eq!(read_u16(&response, 6), Some(0));
}

#[test]
fn parser_rejects_unsupported_types_and_truncated_labels() {
    let mut unsupported = fixture_query(15);
    assert!(super::parse_question(&unsupported).is_none());

    unsupported.truncate(15);
    assert!(super::parse_question_name(&unsupported).is_none());
}

#[test]
fn retries_dns_fixture_tcp_bind_when_ephemeral_port_is_reserved() {
    let mut attempts = 0;
    let (udp_socket, tcp_listener) = super::bind_local_dns_sockets_with(|address| {
        attempts += 1;
        if attempts == 1 {
            Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "simulated Windows excluded TCP port",
            ))
        } else {
            std::net::TcpListener::bind(address)
        }
    })
    .expect("a reserved ephemeral port should be retried");

    assert!((2..=super::MAX_SOCKET_PAIR_BIND_ATTEMPTS).contains(&attempts));
    assert_eq!(
        udp_socket
            .local_addr()
            .expect("UDP socket has a local address"),
        tcp_listener
            .local_addr()
            .expect("TCP listener has the paired local address")
    );
}

#[test]
fn stops_retrying_dns_fixture_tcp_bind_after_the_attempt_limit() {
    let mut attempts = 0;
    let error = super::bind_local_dns_sockets_with(|_address| {
        attempts += 1;
        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "simulated reserved TCP port",
        ))
    })
    .expect_err("socket-pair binding stops after its bounded retry limit");

    assert_eq!(attempts, super::MAX_SOCKET_PAIR_BIND_ATTEMPTS);
    assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
}

#[test]
fn does_not_retry_permanent_dns_fixture_tcp_bind_errors() {
    let mut attempts = 0;
    let error = super::bind_local_dns_sockets_with(|_address| {
        attempts += 1;
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "simulated invalid TCP bind request",
        ))
    })
    .expect_err("permanent socket-pair errors are returned immediately");

    assert_eq!(attempts, 1);
    assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
}

#[test]
fn loopback_udp_fixture_answers_tracks_nxdomain_and_drops_selected_questions() {
    let _guard = lock_network_fixture();
    let fixture = LocalDnsFixture::bind(BTreeMap::from([(
        "fixture.kmipkit.test".to_owned(),
        vec![IpAddr::V4(Ipv4Addr::LOCALHOST)],
    )]))
    .expect("loopback-only DNS fixture should bind");
    let client =
        UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).expect("loopback DNS client should bind");
    client
        .set_read_timeout(Some(Duration::from_millis(250)))
        .expect("DNS client timeout should be configured");
    let request = fixture_query(1);
    let mut response = [0_u8; MAX_QUERY_BYTES + 128];

    client
        .send_to(&request, fixture.local_addr())
        .expect("DNS A question should be sent");
    let (length, _) = client
        .recv_from(&mut response)
        .expect("fixture should answer a valid A question");
    assert_eq!(read_u16(&response[..length], 6), Some(1));

    fixture.set_nxdomain("FIXTURE.KMIPKIT.TEST.");
    client
        .send_to(&request, fixture.local_addr())
        .expect("second DNS question should be sent");
    let (length, _) = client
        .recv_from(&mut response)
        .expect("fixture should answer configured NXDOMAIN");
    assert_eq!(
        read_u16(&response[..length], 2).map(|flags| flags & 0x000f),
        Some(3)
    );

    fixture.drop_next_questions("Fixture.KmipKit.Test.", DnsQueryType::A, 1);
    client
        .send_to(&request, fixture.local_addr())
        .expect("dropped DNS question should be sent");
    assert!(matches!(
        client
            .recv_from(&mut response)
            .expect_err("selected query is dropped")
            .kind(),
        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
    ));
    assert_eq!(
        fixture.query_count("fixture.kmipkit.test", DnsQueryType::A),
        3
    );
}

#[test]
fn loopback_tcp_fixture_answers_a_framed_query_and_closes_on_short_frame() {
    let _guard = lock_network_fixture();
    let fixture = LocalDnsFixture::bind(BTreeMap::from([(
        "fixture.kmipkit.test".to_owned(),
        vec![IpAddr::V4(Ipv4Addr::LOCALHOST)],
    )]))
    .expect("loopback-only DNS fixture should bind");
    let mut client = TcpStream::connect(fixture.local_addr())
        .expect("DNS TCP connection should use the loopback listener");
    client
        .set_read_timeout(Some(Duration::from_secs(1)))
        .expect("DNS TCP read timeout should be configured");
    let request = fixture_query(1);
    write_tcp_dns_query(&mut client, &request).expect("framed DNS question should be sent");

    let response =
        read_tcp_dns_response(&mut client).expect("fixture should return a framed DNS response");
    assert_eq!(read_u16(&response, 6), Some(1));

    client
        .write_all(&11_u16.to_be_bytes())
        .expect("short DNS frame header should be sent");
    let mut trailing = [0_u8; 1];
    let read_result = client.read(&mut trailing);
    assert!(
        is_close_result(&read_result),
        "fixture closes an invalid short frame; client read returned {read_result:?}"
    );
}

#[test]
fn accepted_tcp_stream_is_blocking_before_a_delayed_read() {
    let listener =
        TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("loopback TCP listener should bind");
    let mut writer = TcpStream::connect(listener.local_addr().expect("listener address exists"))
        .expect("loopback TCP client should connect");
    let (mut accepted, _) = listener
        .accept()
        .expect("loopback TCP connection should be accepted");

    accepted
        .set_read_timeout(Some(Duration::from_secs(1)))
        .expect("accepted TCP stream should have a finite read timeout");
    accepted
        .set_nonblocking(true)
        .expect("accepted stream should model an inherited nonblocking mode");
    prepare_accepted_tcp_stream(&accepted)
        .expect("accepted stream should be normalized before its handler reads");

    let writer_thread = thread::spawn(move || {
        thread::sleep(Duration::from_millis(25));
        writer.write_all(&[0x5a])
    });
    let mut byte = [0_u8; 1];
    accepted
        .read_exact(&mut byte)
        .expect("blocking read should wait for the delayed writer");
    writer_thread
        .join()
        .expect("writer thread should not panic")
        .expect("delayed byte should be written");

    assert_eq!(byte, [0x5a]);
}

#[test]
fn accepted_tcp_connection_is_served_when_connection_worker_spawn_fails() {
    let _guard = lock_network_fixture();
    let records = BTreeMap::from([(
        "fixture.kmipkit.test".to_owned(),
        vec![IpAddr::V4(Ipv4Addr::LOCALHOST)],
    )]);
    let (spawner, spawn_attempts, _connection_worker_done, connection_worker_started, _) =
        tcp_worker_spawner_failing(TcpWorkerKind::Connection);
    let (local_addr, state, server) =
        start_tcp_server_for_spawn_test(records, spawner, TcpStream::try_clone);

    let mut client = TcpStream::connect(local_addr).expect("DNS TCP client should connect");
    client
        .set_read_timeout(Some(Duration::from_secs(1)))
        .expect("DNS TCP read timeout should be configured");
    let request = fixture_query(1);
    write_tcp_dns_query(&mut client, &request).expect("framed DNS question should be sent");
    let response = read_tcp_dns_response(&mut client);
    let _shutdown_result = client.shutdown(Shutdown::Both);
    state.stop.store(true, Ordering::Release);
    let server_result = server.join();

    let connection_attempts = spawn_attempts.connection.load(Ordering::Acquire);
    assert!(spawn_attempts.response.load(Ordering::Acquire) <= 1);
    assert!(connection_attempts <= 1);
    if connection_attempts == 1 {
        assert!(
            !connection_worker_started
                .recv_timeout(Duration::from_secs(1))
                .expect("injected connection worker failure should report its outcome")
        );
    }
    assert!(server_result.is_ok(), "TCP fixture accept loop should exit");
    let response = response.expect("accepted query should still receive a DNS response");
    assert_eq!(read_u16(&response, 0), Some(0));
    assert_eq!(read_u16(&response, 6), Some(1));
}

#[test]
fn accepted_tcp_connection_is_served_inline_when_accepted_stream_clone_fails() {
    let _guard = lock_network_fixture();
    let records = BTreeMap::from([(
        "fixture.kmipkit.test".to_owned(),
        vec![IpAddr::V4(Ipv4Addr::LOCALHOST)],
    )]);
    let (spawner, spawn_attempts, _connection_worker_done, _connection_worker_started, _) =
        tcp_worker_spawner_failing(TcpWorkerKind::Response);
    let clone_attempts = Arc::new(AtomicUsize::new(0));
    let observed_clone_attempts = Arc::clone(&clone_attempts);
    let (local_addr, state, server) =
        start_tcp_server_for_spawn_test(records, spawner, move |stream: &TcpStream| {
            if observed_clone_attempts.fetch_add(1, Ordering::AcqRel) == 0 {
                Err(io::Error::other("injected accepted-stream clone failure"))
            } else {
                stream.try_clone()
            }
        });

    let mut client = TcpStream::connect(local_addr).expect("DNS TCP client should connect");
    client
        .set_read_timeout(Some(Duration::from_secs(1)))
        .expect("DNS TCP read timeout should be configured");
    let request = fixture_query(1);
    write_tcp_dns_query(&mut client, &request).expect("framed DNS question should be sent");
    let response = read_tcp_dns_response(&mut client);
    let _shutdown_result = client.shutdown(Shutdown::Both);
    state.stop.store(true, Ordering::Release);
    let server_result = server.join();

    assert_eq!(clone_attempts.load(Ordering::Acquire), 2);
    assert_eq!(spawn_attempts.connection.load(Ordering::Acquire), 0);
    assert!(spawn_attempts.response.load(Ordering::Acquire) <= 1);
    assert!(server_result.is_ok(), "TCP fixture accept loop should exit");
    let response = response.expect("inline clone fallback should return a DNS response");
    assert_eq!(read_u16(&response, 0), Some(0));
    assert_eq!(read_u16(&response, 6), Some(1));
}

#[test]
fn accepted_tcp_connection_is_served_inline_when_both_stream_clones_fail() {
    let _guard = lock_network_fixture();
    let records = BTreeMap::from([(
        "fixture.kmipkit.test".to_owned(),
        vec![IpAddr::V4(Ipv4Addr::LOCALHOST)],
    )]);
    let (spawner, spawn_attempts, _connection_worker_done, _connection_worker_started, _) =
        tcp_worker_spawner_failing(TcpWorkerKind::Response);
    let clone_attempts = Arc::new(AtomicUsize::new(0));
    let observed_clone_attempts = Arc::clone(&clone_attempts);
    let (local_addr, state, server) =
        start_tcp_server_for_spawn_test(records, spawner, move |_: &TcpStream| {
            observed_clone_attempts.fetch_add(1, Ordering::AcqRel);
            Err(io::Error::other("injected TCP stream clone failure"))
        });

    let mut client = TcpStream::connect(local_addr).expect("DNS TCP client should connect");
    client
        .set_read_timeout(Some(Duration::from_secs(1)))
        .expect("DNS TCP read timeout should be configured");
    write_tcp_dns_query(&mut client, &fixture_query(1))
        .expect("framed DNS question should be sent");
    let response = read_tcp_dns_response(&mut client);
    let _shutdown_result = client.shutdown(Shutdown::Both);
    state.stop.store(true, Ordering::Release);
    let server_result = server.join();

    assert_eq!(clone_attempts.load(Ordering::Acquire), 2);
    assert_eq!(spawn_attempts.connection.load(Ordering::Acquire), 0);
    assert_eq!(spawn_attempts.response.load(Ordering::Acquire), 0);
    assert!(server_result.is_ok(), "TCP fixture accept loop should exit");
    let response = response.expect("accepted query should receive an inline DNS response");
    assert_eq!(read_u16(&response, 0), Some(0));
    assert_eq!(read_u16(&response, 6), Some(1));
    let metrics = state
        .tcp_metrics
        .lock()
        .expect("TCP metrics should remain available");
    assert_eq!(metrics.active_by_connection.get(&1), Some(&0));
    assert_eq!(metrics.peak_by_connection.get(&1), Some(&1));
}

#[test]
fn dns_response_is_written_inline_when_response_worker_spawn_fails() {
    let _guard = lock_network_fixture();
    let records = BTreeMap::from([(
        "fixture.kmipkit.test".to_owned(),
        vec![IpAddr::V4(Ipv4Addr::LOCALHOST)],
    )]);
    let (spawner, spawn_attempts, connection_worker_done, connection_worker_started, _) =
        tcp_worker_spawner_failing(TcpWorkerKind::Response);
    let (local_addr, state, server) =
        start_tcp_server_for_spawn_test(records, spawner, TcpStream::try_clone);

    let mut client = TcpStream::connect(local_addr).expect("DNS TCP client should connect");
    client
        .set_read_timeout(Some(Duration::from_secs(1)))
        .expect("DNS TCP read timeout should be configured");
    let request = fixture_query(1);
    write_tcp_dns_query(&mut client, &request).expect("framed DNS question should be sent");
    let response = read_tcp_dns_response(&mut client);
    let _shutdown_result = client.shutdown(Shutdown::Both);
    state.stop.store(true, Ordering::Release);
    let server_result = server.join();
    let connection_attempts = spawn_attempts.connection.load(Ordering::Acquire);
    if connection_attempts == 1
        && connection_worker_started
            .recv_timeout(Duration::from_secs(1))
            .expect("connection worker spawn should report its outcome")
    {
        connection_worker_done
            .recv_timeout(Duration::from_secs(1))
            .expect("connection worker should finish before metrics are inspected");
    }

    assert!(connection_attempts <= 1);
    assert!(spawn_attempts.response.load(Ordering::Acquire) <= 1);
    assert!(server_result.is_ok(), "TCP fixture accept loop should exit");
    let response = response.expect("DNS response should be sent if its worker cannot start");
    assert_eq!(read_u16(&response, 0), Some(0));
    assert_eq!(read_u16(&response, 6), Some(1));
    let metrics = state
        .tcp_metrics
        .lock()
        .expect("TCP metrics should remain available");
    assert_eq!(metrics.active_by_connection.get(&1), Some(&0));
    assert_eq!(metrics.peak_by_connection.get(&1), Some(&1));
}

#[test]
fn dns_response_is_written_inline_when_writer_clone_fails_in_connection_worker() {
    let _guard = lock_network_fixture();
    let records = BTreeMap::from([(
        "fixture.kmipkit.test".to_owned(),
        vec![IpAddr::V4(Ipv4Addr::LOCALHOST)],
    )]);
    let (spawner, spawn_attempts, connection_worker_done, connection_worker_started, _) =
        tcp_worker_spawner_with_failure(None);
    let clone_attempts = Arc::new(AtomicUsize::new(0));
    let accepted_clone_succeeded = Arc::new(AtomicUsize::new(0));
    let injected_writer_clone_failure = Arc::new(AtomicUsize::new(0));
    let observed_clone_attempts = Arc::clone(&clone_attempts);
    let observed_accepted_success = Arc::clone(&accepted_clone_succeeded);
    let observed_writer_failure = Arc::clone(&injected_writer_clone_failure);
    let (local_addr, state, server) =
        start_tcp_server_for_spawn_test(records, spawner, move |stream: &TcpStream| {
            match observed_clone_attempts.fetch_add(1, Ordering::AcqRel) {
                0 => {
                    let clone = retry_tcp_stream_clone(stream)?;
                    observed_accepted_success.fetch_add(1, Ordering::AcqRel);
                    Ok(clone)
                }
                1 => {
                    observed_writer_failure.fetch_add(1, Ordering::AcqRel);
                    Err(io::Error::other("injected writer-stream clone failure"))
                }
                _ => Err(io::Error::other("unexpected additional TCP stream clone")),
            }
        });

    let mut client = TcpStream::connect(local_addr).expect("DNS TCP client should connect");
    client
        .set_read_timeout(Some(Duration::from_secs(3)))
        .expect("DNS TCP read timeout should be configured");
    write_tcp_dns_query(&mut client, &fixture_query(1))
        .expect("framed DNS question should be sent");
    let response = read_tcp_dns_response(&mut client);
    let _shutdown_result = client.shutdown(Shutdown::Both);
    let connection_worker_started = connection_worker_started
        .recv_timeout(Duration::from_secs(1))
        .expect("connection worker spawn should report its outcome");
    if connection_worker_started {
        connection_worker_done
            .recv_timeout(Duration::from_secs(1))
            .expect("connection worker should finish before metrics are inspected");
    }
    state.stop.store(true, Ordering::Release);
    let server_result = server.join();

    assert!(
        connection_worker_started,
        "request must run in a spawned worker"
    );
    assert_eq!(clone_attempts.load(Ordering::Acquire), 2);
    assert_eq!(accepted_clone_succeeded.load(Ordering::Acquire), 1);
    assert_eq!(injected_writer_clone_failure.load(Ordering::Acquire), 1);
    assert_eq!(spawn_attempts.connection.load(Ordering::Acquire), 1);
    assert_eq!(spawn_attempts.response.load(Ordering::Acquire), 0);
    assert!(server_result.is_ok(), "TCP fixture accept loop should exit");
    let response = response.expect("writer clone fallback should return a DNS response");
    assert_eq!(read_u16(&response, 0), Some(0));
    assert_eq!(read_u16(&response, 6), Some(1));
    let metrics = state
        .tcp_metrics
        .lock()
        .expect("TCP metrics should remain available");
    assert_eq!(metrics.active_by_connection.get(&1), Some(&0));
    assert_eq!(metrics.peak_by_connection.get(&1), Some(&1));
}

#[test]
fn connection_worker_spawn_failure_is_reached_after_successful_clones() {
    let _guard = lock_network_fixture();
    let records = BTreeMap::from([(
        "fixture.kmipkit.test".to_owned(),
        vec![IpAddr::V4(Ipv4Addr::LOCALHOST)],
    )]);
    let (
        spawner,
        spawn_attempts,
        _connection_worker_done,
        connection_worker_started,
        response_worker_done,
    ) = tcp_worker_spawner_with_failure(Some(TcpWorkerKind::Connection));
    let clone_attempts = Arc::new(AtomicUsize::new(0));
    let observed_clone_attempts = Arc::clone(&clone_attempts);
    let (local_addr, state, server) =
        start_tcp_server_for_spawn_test(records, spawner, move |stream: &TcpStream| {
            observed_clone_attempts.fetch_add(1, Ordering::AcqRel);
            retry_tcp_stream_clone(stream)
        });

    let mut client = TcpStream::connect(local_addr).expect("DNS TCP client should connect");
    client
        .set_read_timeout(Some(Duration::from_secs(3)))
        .expect("DNS TCP read timeout should be configured");
    write_tcp_dns_query(&mut client, &fixture_query(1))
        .expect("framed DNS question should be sent");
    let response = read_tcp_dns_response(&mut client);
    let _shutdown_result = client.shutdown(Shutdown::Both);
    let connection_worker_started = connection_worker_started
        .recv_timeout(Duration::from_secs(1))
        .expect("injected connection worker failure should report its outcome");
    response_worker_done
        .recv_timeout(Duration::from_secs(1))
        .expect("fallback response worker should finish before metrics are inspected");
    state.stop.store(true, Ordering::Release);
    let server_result = server.join();

    assert!(
        !connection_worker_started,
        "injected connection spawn must fail"
    );
    assert_eq!(clone_attempts.load(Ordering::Acquire), 2);
    assert_eq!(spawn_attempts.connection.load(Ordering::Acquire), 1);
    assert_eq!(spawn_attempts.response.load(Ordering::Acquire), 1);
    assert!(server_result.is_ok(), "TCP fixture accept loop should exit");
    let response = response.expect("accepted query should receive a DNS response");
    assert_eq!(read_u16(&response, 0), Some(0));
    assert_eq!(read_u16(&response, 6), Some(1));
    let metrics = state
        .tcp_metrics
        .lock()
        .expect("TCP metrics should remain available");
    assert_eq!(metrics.active_by_connection.get(&1), Some(&0));
    assert_eq!(metrics.peak_by_connection.get(&1), Some(&1));
}

#[test]
fn response_worker_spawn_failure_is_reached_after_successful_clones() {
    let _guard = lock_network_fixture();
    let records = BTreeMap::from([(
        "fixture.kmipkit.test".to_owned(),
        vec![IpAddr::V4(Ipv4Addr::LOCALHOST)],
    )]);
    let (spawner, spawn_attempts, connection_worker_done, connection_worker_started, _) =
        tcp_worker_spawner_with_failure(Some(TcpWorkerKind::Response));
    let clone_attempts = Arc::new(AtomicUsize::new(0));
    let observed_clone_attempts = Arc::clone(&clone_attempts);
    let (local_addr, state, server) =
        start_tcp_server_for_spawn_test(records, spawner, move |stream: &TcpStream| {
            observed_clone_attempts.fetch_add(1, Ordering::AcqRel);
            retry_tcp_stream_clone(stream)
        });

    let mut client = TcpStream::connect(local_addr).expect("DNS TCP client should connect");
    client
        .set_read_timeout(Some(Duration::from_secs(3)))
        .expect("DNS TCP read timeout should be configured");
    write_tcp_dns_query(&mut client, &fixture_query(1))
        .expect("framed DNS question should be sent");
    let response = read_tcp_dns_response(&mut client);
    let _shutdown_result = client.shutdown(Shutdown::Both);
    let connection_worker_started = connection_worker_started
        .recv_timeout(Duration::from_secs(1))
        .expect("connection worker spawn should report its outcome");
    if connection_worker_started {
        connection_worker_done
            .recv_timeout(Duration::from_secs(1))
            .expect("connection worker should finish before metrics are inspected");
    }
    state.stop.store(true, Ordering::Release);
    let server_result = server.join();

    assert!(
        connection_worker_started,
        "request must run in a spawned worker"
    );
    assert_eq!(clone_attempts.load(Ordering::Acquire), 2);
    assert_eq!(spawn_attempts.connection.load(Ordering::Acquire), 1);
    assert_eq!(spawn_attempts.response.load(Ordering::Acquire), 1);
    assert!(server_result.is_ok(), "TCP fixture accept loop should exit");
    let response = response.expect("DNS response should be sent if its worker cannot start");
    assert_eq!(read_u16(&response, 0), Some(0));
    assert_eq!(read_u16(&response, 6), Some(1));
    let metrics = state
        .tcp_metrics
        .lock()
        .expect("TCP metrics should remain available");
    assert_eq!(metrics.active_by_connection.get(&1), Some(&0));
    assert_eq!(metrics.peak_by_connection.get(&1), Some(&1));
}

fn tcp_worker_spawner_failing(
    failed_kind: TcpWorkerKind,
) -> (
    TcpWorkerSpawner,
    Arc<TcpWorkerSpawnAttempts>,
    Receiver<()>,
    Receiver<bool>,
    Receiver<()>,
) {
    tcp_worker_spawner_with_failure(Some(failed_kind))
}

fn tcp_worker_spawner_with_failure(
    failed_kind: Option<TcpWorkerKind>,
) -> (
    TcpWorkerSpawner,
    Arc<TcpWorkerSpawnAttempts>,
    Receiver<()>,
    Receiver<bool>,
    Receiver<()>,
) {
    let attempts = Arc::new(TcpWorkerSpawnAttempts::default());
    let spawner_attempts = Arc::clone(&attempts);
    let (connection_worker_done_sender, connection_worker_done) = mpsc::channel();
    let (connection_worker_started_sender, connection_worker_started) = mpsc::channel();
    let (response_worker_done_sender, response_worker_done) = mpsc::channel();
    let spawner: TcpWorkerSpawner = Arc::new(move |kind, job| {
        spawner_attempts.record_attempt(kind);
        if Some(kind) == failed_kind {
            if kind == TcpWorkerKind::Connection {
                let _ = connection_worker_started_sender.send(false);
            }
            drop(job);
            Err(io::Error::other("injected DNS worker spawn failure"))
        } else if kind == TcpWorkerKind::Connection {
            let done_sender = connection_worker_done_sender.clone();
            let spawn_result = thread::Builder::new().spawn(move || {
                job();
                let _ = done_sender.send(());
            });
            let _ = connection_worker_started_sender.send(spawn_result.is_ok());
            spawn_result.map(drop)
        } else {
            let done_sender = response_worker_done_sender.clone();
            thread::Builder::new()
                .spawn(move || {
                    job();
                    let _ = done_sender.send(());
                })
                .map(drop)
        }
    });
    (
        spawner,
        attempts,
        connection_worker_done,
        connection_worker_started,
        response_worker_done,
    )
}

fn retry_tcp_stream_clone(stream: &TcpStream) -> io::Result<TcpStream> {
    const MAX_CLONE_ATTEMPTS: usize = 100;
    const RETRY_DELAY: Duration = Duration::from_millis(2);

    let mut last_error = io::Error::other("TCP stream clone attempts were not started");
    for attempt in 0..MAX_CLONE_ATTEMPTS {
        match stream.try_clone() {
            Ok(clone) => return Ok(clone),
            Err(error) => {
                last_error = error;
                if attempt + 1 < MAX_CLONE_ATTEMPTS {
                    thread::sleep(RETRY_DELAY);
                }
            }
        }
    }
    Err(last_error)
}

#[derive(Default)]
struct TcpWorkerSpawnAttempts {
    connection: AtomicUsize,
    response: AtomicUsize,
}

impl TcpWorkerSpawnAttempts {
    fn record_attempt(&self, kind: TcpWorkerKind) {
        let attempts = match kind {
            TcpWorkerKind::Connection => &self.connection,
            TcpWorkerKind::Response => &self.response,
        };
        attempts.fetch_add(1, Ordering::AcqRel);
    }
}

fn start_tcp_server_for_spawn_test<F>(
    records: BTreeMap<String, Vec<IpAddr>>,
    spawner: TcpWorkerSpawner,
    clone_stream: F,
) -> (
    std::net::SocketAddr,
    Arc<FixtureState>,
    thread::JoinHandle<()>,
)
where
    F: Fn(&TcpStream) -> io::Result<TcpStream> + Send + Sync + 'static,
{
    let listener =
        TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("loopback test listener should bind");
    listener
        .set_nonblocking(true)
        .expect("loopback listener should use nonblocking accept");
    let local_addr = listener
        .local_addr()
        .expect("loopback test listener should expose its address");
    let state = Arc::new(FixtureState {
        stop: AtomicBool::new(false),
        metrics: Mutex::new(FixtureMetrics::default()),
        response_released: (Mutex::new(true), Condvar::new()),
        active_responses: AtomicUsize::new(0),
        peak_active_responses: AtomicUsize::new(0),
        next_tcp_connection: AtomicUsize::new(1),
        tcp_metrics: Mutex::new(TcpMetrics::default()),
    });
    let server_state = Arc::clone(&state);
    let server = thread::spawn(move || {
        serve_tcp_with_spawner_and_cloner(
            &listener,
            &records,
            &server_state,
            &spawner,
            clone_stream,
        );
    });
    (local_addr, state, server)
}

#[test]
fn close_result_accepts_only_eof_and_peer_disconnects() {
    let cases = [
        ("EOF", Ok(0), true),
        (
            "ConnectionAborted",
            Err(io::Error::from(io::ErrorKind::ConnectionAborted)),
            true,
        ),
        (
            "ConnectionReset",
            Err(io::Error::from(io::ErrorKind::ConnectionReset)),
            true,
        ),
        ("received data", Ok(1), false),
        (
            "WouldBlock",
            Err(io::Error::from(io::ErrorKind::WouldBlock)),
            false,
        ),
        (
            "TimedOut",
            Err(io::Error::from(io::ErrorKind::TimedOut)),
            false,
        ),
        (
            "unrelated error",
            Err(io::Error::other("unrelated read error")),
            false,
        ),
    ];

    for (case_name, read_result, expected_closed) in cases {
        assert_eq!(
            is_close_result(&read_result),
            expected_closed,
            "unexpected close classification for {case_name}"
        );
    }
}

fn fixture_query(record_type: u16) -> Vec<u8> {
    query(
        &[
            7, b'f', b'i', b'x', b't', b'u', b'r', b'e', 7, b'k', b'm', b'i', b'p', b'k', b'i',
            b't', 4, b't', b'e', b's', b't', 0,
        ],
        record_type,
        1,
    )
}

fn write_tcp_dns_query(client: &mut TcpStream, request: &[u8]) -> io::Result<()> {
    let length = u16::try_from(request.len())
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    client.write_all(&length.to_be_bytes())?;
    client.write_all(request)
}

fn read_tcp_dns_response(client: &mut TcpStream) -> io::Result<Vec<u8>> {
    let mut response_length = [0_u8; 2];
    client.read_exact(&mut response_length)?;
    let length = usize::from(u16::from_be_bytes(response_length));
    let mut response = vec![0_u8; length];
    client.read_exact(&mut response)?;
    Ok(response)
}

fn query(question_name: &[u8], record_type: u16, record_class: u16) -> Vec<u8> {
    let mut query = vec![0_u8; 12];
    query[2..4].copy_from_slice(&0x0100_u16.to_be_bytes());
    query[4..6].copy_from_slice(&1_u16.to_be_bytes());
    query.extend_from_slice(question_name);
    query.extend_from_slice(&record_type.to_be_bytes());
    query.extend_from_slice(&record_class.to_be_bytes());
    query
}
