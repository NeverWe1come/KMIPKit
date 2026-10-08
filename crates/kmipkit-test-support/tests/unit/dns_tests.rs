use super::{
    DnsQueryType, LocalDnsFixture, MAX_QUERY_BYTES, build_response, read_u16, validate_records,
};
use std::collections::BTreeMap;
use std::io::{self, Read, Write};
use std::net::{IpAddr, Ipv4Addr, TcpStream, UdpSocket};
use std::sync::Mutex;
use std::time::Duration;

static NETWORK_FIXTURE_LOCK: Mutex<()> = Mutex::new(());

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

    assert_eq!(attempts, 2);
    assert_eq!(
        udp_socket.local_addr().expect("UDP socket has a local address"),
        tcp_listener
            .local_addr()
            .expect("TCP listener has the paired local address")
    );
}

#[test]
fn loopback_udp_fixture_answers_tracks_nxdomain_and_drops_selected_questions() {
    let _guard = NETWORK_FIXTURE_LOCK
        .lock()
        .expect("DNS fixture tests share one port-pair allocator");
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
    let _guard = NETWORK_FIXTURE_LOCK
        .lock()
        .expect("DNS fixture tests share one port-pair allocator");
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
    let length = u16::try_from(request.len()).expect("DNS request length fits its frame");
    client
        .write_all(&length.to_be_bytes())
        .and_then(|()| client.write_all(&request))
        .expect("framed DNS question should be sent");

    let mut response_length = [0_u8; 2];
    client
        .read_exact(&mut response_length)
        .expect("fixture should return a framed DNS response");
    let response_length = usize::from(u16::from_be_bytes(response_length));
    let mut response = vec![0_u8; response_length];
    client
        .read_exact(&mut response)
        .expect("complete DNS response should be returned");
    assert_eq!(read_u16(&response, 6), Some(1));

    client
        .write_all(&11_u16.to_be_bytes())
        .expect("short DNS frame header should be sent");
    let mut trailing = [0_u8; 1];
    let closed = match client.read(&mut trailing) {
        Ok(0) => true,
        Err(error) if error.kind() == io::ErrorKind::ConnectionAborted => true,
        Ok(_) | Err(_) => false,
    };
    assert!(closed, "fixture closes an invalid short frame");
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

fn query(question_name: &[u8], record_type: u16, record_class: u16) -> Vec<u8> {
    let mut query = vec![0_u8; 12];
    query[2..4].copy_from_slice(&0x0100_u16.to_be_bytes());
    query[4..6].copy_from_slice(&1_u16.to_be_bytes());
    query.extend_from_slice(question_name);
    query.extend_from_slice(&record_type.to_be_bytes());
    query.extend_from_slice(&record_class.to_be_bytes());
    query
}
