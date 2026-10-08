use std::collections::BTreeMap;
use std::io;
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpStream, UdpSocket};
use std::time::Duration;

use kmipkit_test_support::{
    DnsQueryType, EphemeralPki, LocalDnsFixture, LoopbackTcpListener, PkiFixtureError,
};

#[test]
fn ephemeral_pki_generates_distinct_client_and_server_credentials_without_debugging_keys() {
    let first = EphemeralPki::generate().expect("the first ephemeral PKI is generated");
    let second = EphemeralPki::generate().expect("the second ephemeral PKI is generated");

    assert_ne!(
        first.authority_certificate_der(),
        second.authority_certificate_der(),
        "each generated PKI must have a distinct trust root"
    );
    assert_ne!(
        first.server_identity().private_key_der(),
        second.server_identity().private_key_der(),
        "each generated server identity must use fresh key material"
    );
    assert_ne!(
        first.client_identity().private_key_der(),
        first.server_identity().private_key_der(),
        "client and server identities must not share key material"
    );
    assert_ne!(
        first.client_identity().certificate_der(),
        first.server_identity().certificate_der(),
        "client and server identities must have distinct certificates"
    );
    assert_eq!(
        first.client_identity().certificate_chain_der()[1],
        first.authority_certificate_der()
    );
    assert_eq!(
        first.server_identity().certificate_chain_der()[1],
        first.authority_certificate_der()
    );

    let formatted = format!("{first:?}");
    assert!(!formatted.contains("private_key_der"));
    assert!(formatted.contains("[REDACTED]"));
    assert!(!formatted.contains(&hex(first.server_identity().private_key_der())));
    assert!(!formatted.contains(&hex(first.client_identity().private_key_der())));
}

#[test]
fn ephemeral_pki_generation_errors_are_redacted() {
    assert_eq!(
        PkiFixtureError.to_string(),
        "ephemeral test PKI generation failed"
    );
}

#[test]
fn local_dns_fixture_answers_only_for_configured_loopback_names() {
    let fixture = LocalDnsFixture::bind(BTreeMap::from([(
        "transport.kmipkit.test".to_owned(),
        vec![
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            IpAddr::V6(Ipv6Addr::LOCALHOST),
        ],
    )]))
    .expect("the local DNS fixture binds to an ephemeral loopback port");
    assert!(fixture.local_addr().ip().is_loopback());

    let ipv4 = query_dns(fixture.local_addr(), "transport.kmipkit.test", 1)
        .expect("the configured local A record is answered");
    assert_eq!(ipv4, vec![127, 0, 0, 1]);

    let ipv6 = query_dns(fixture.local_addr(), "transport.kmipkit.test", 28)
        .expect("the configured local AAAA record is answered");
    assert_eq!(ipv6, vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]);

    let unknown = query_dns(fixture.local_addr(), "outside.kmipkit.test", 1)
        .expect("the unknown local name receives a negative DNS answer");
    assert_eq!(unknown, Vec::<u8>::new());
}

#[test]
fn local_dns_fixture_exposes_deterministic_retry_gate_and_nxdomain_controls() {
    let name = "scripted.kmipkit.test";
    let fixture = LocalDnsFixture::bind(BTreeMap::from([(
        name.to_owned(),
        vec![IpAddr::V4(Ipv4Addr::LOCALHOST)],
    )]))
    .expect("the local fixture binds on loopback");

    fixture.drop_next_questions(name, DnsQueryType::A, 1);
    send_dns_question(fixture.local_addr(), name, 1).expect("the first A question is sent");
    wait_for_question_count(&fixture, name, DnsQueryType::A, 1);
    let response = query_dns_response(fixture.local_addr(), name, 1)
        .expect("the second A question receives the scripted answer");
    assert_eq!(fixture.query_count(name, DnsQueryType::A), 2);
    assert_eq!(u16::from_be_bytes([response[6], response[7]]), 1);

    fixture.hold_responses();
    let held_server = fixture.local_addr();
    let held_name = name.to_owned();
    let held_query = std::thread::spawn(move || query_dns_response(held_server, &held_name, 1));
    wait_for_question_count(&fixture, name, DnsQueryType::A, 3);
    assert!(fixture.peak_active_responses() > 0);
    fixture.release_responses();
    assert!(
        held_query
            .join()
            .expect("the gated query thread completes")
            .is_ok()
    );

    let negative_name = "missing.kmipkit.test";
    fixture.set_nxdomain(negative_name);
    let negative = query_dns_response(fixture.local_addr(), negative_name, 1)
        .expect("the fixture returns its explicit negative answer");
    assert_eq!(u16::from_be_bytes([negative[2], negative[3]]) & 0x000f, 3);
}

#[test]
fn local_dns_fixture_tracks_multiplexed_tcp_requests_per_connection() {
    let name = "multiplexed.kmipkit.test";
    let fixture = LocalDnsFixture::bind(BTreeMap::from([(
        name.to_owned(),
        vec![IpAddr::V4(Ipv4Addr::LOCALHOST)],
    )]))
    .expect("the local DNS fixture binds UDP and TCP loopback sockets");
    fixture.hold_responses();

    let mut stream = TcpStream::connect(fixture.local_addr())
        .expect("the local TCP DNS endpoint accepts a connection");
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("the fixture read has a bounded wait");
    let query = dns_question(name, 1);
    for _ in 0..2 {
        let length =
            u16::try_from(query.len()).expect("the test DNS question length fits in the TCP frame");
        stream
            .write_all(&length.to_be_bytes())
            .expect("the TCP DNS frame length is written");
        stream
            .write_all(&query)
            .expect("the TCP DNS question is written");
    }

    wait_for_question_count(&fixture, name, DnsQueryType::A, 2);
    assert!(fixture.peak_active_tcp_requests_per_connection() >= 2);
    assert_eq!(fixture.active_tcp_requests(), 2);
    fixture.release_responses();

    for _ in 0..2 {
        let mut response_length = [0_u8; 2];
        stream
            .read_exact(&mut response_length)
            .expect("the TCP DNS response length is read");
        let mut response = vec![0_u8; usize::from(u16::from_be_bytes(response_length))];
        stream
            .read_exact(&mut response)
            .expect("the TCP DNS response is read");
        assert_eq!(u16::from_be_bytes([response[6], response[7]]), 1);
    }
}

#[test]
fn local_transport_listener_never_binds_a_public_interface() {
    let fixture = LoopbackTcpListener::bind()
        .expect("the transport fixture binds to an ephemeral loopback port");

    assert!(fixture.local_addr().ip().is_loopback());
}

#[test]
fn local_dns_fixture_rejects_non_loopback_records_before_binding() {
    let result = LocalDnsFixture::bind(BTreeMap::from([(
        "transport.kmipkit.test".to_owned(),
        vec![IpAddr::V4(Ipv4Addr::new(203, 0, 113, 1))],
    )]));

    let error = result
        .err()
        .expect("non-loopback records are rejected before binding");
    assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
}

fn query_dns_response(server: SocketAddr, hostname: &str, record_type: u16) -> io::Result<Vec<u8>> {
    let socket = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0))?;
    socket.set_read_timeout(Some(Duration::from_secs(2)))?;
    socket.send_to(&dns_question(hostname, record_type), server)?;
    receive_dns_response(&socket)
}

fn send_dns_question(server: SocketAddr, hostname: &str, record_type: u16) -> io::Result<()> {
    let socket = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0))?;
    socket.send_to(&dns_question(hostname, record_type), server)?;
    Ok(())
}

fn dns_question(hostname: &str, record_type: u16) -> Vec<u8> {
    let mut query = vec![
        0x4b, 0x4d, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    for label in hostname.split('.') {
        let label_bytes = label.as_bytes();
        query.push(u8::try_from(label_bytes.len()).expect("DNS labels fit in one length byte"));
        query.extend_from_slice(label_bytes);
    }
    query.push(0);
    query.extend_from_slice(&record_type.to_be_bytes());
    query.extend_from_slice(&1_u16.to_be_bytes());
    query
}

fn receive_dns_response(socket: &UdpSocket) -> io::Result<Vec<u8>> {
    let mut response = vec![0_u8; 512];
    let (response_len, _) = socket.recv_from(&mut response)?;
    response.truncate(response_len);
    if response.get(..2) != Some(&[0x4b, 0x4d]) || response.len() < 12 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "local DNS fixture returned an invalid response",
        ));
    }
    Ok(response)
}

fn wait_for_question_count(
    fixture: &LocalDnsFixture,
    name: &str,
    query_type: DnsQueryType,
    expected: usize,
) {
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while fixture.query_count(name, query_type) < expected && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(2));
    }
    assert_eq!(fixture.query_count(name, query_type), expected);
}

fn query_dns(server: SocketAddr, hostname: &str, record_type: u16) -> io::Result<Vec<u8>> {
    let socket = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0))?;
    socket.set_read_timeout(Some(Duration::from_secs(2)))?;

    let mut query = vec![
        0x4b, 0x4d, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    for label in hostname.split('.') {
        let label_bytes = label.as_bytes();
        query.push(u8::try_from(label_bytes.len()).expect("DNS labels fit in one length byte"));
        query.extend_from_slice(label_bytes);
    }
    query.push(0);
    query.extend_from_slice(&record_type.to_be_bytes());
    query.extend_from_slice(&1_u16.to_be_bytes());

    socket.send_to(&query, server)?;
    let mut response = vec![0_u8; 512];
    let (response_len, _) = socket.recv_from(&mut response)?;
    response.truncate(response_len);
    if response.get(..2) != Some(&[0x4b, 0x4d]) || response.len() < 12 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "local DNS fixture returned an invalid response",
        ));
    }

    let question_count = u16::from_be_bytes([response[4], response[5]]);
    let answer_count = u16::from_be_bytes([response[6], response[7]]);
    if question_count != 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "local DNS fixture returned an invalid question count",
        ));
    }

    let mut offset = 12;
    while response.get(offset).copied().unwrap_or_default() != 0 {
        let label_len = usize::from(response[offset]);
        offset = offset
            .checked_add(1 + label_len)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid DNS question"))?;
        if offset >= response.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "truncated DNS question",
            ));
        }
    }
    offset = offset
        .checked_add(5)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid DNS question"))?;

    if answer_count == 0 {
        return Ok(Vec::new());
    }

    if offset + 12 > response.len() || response[offset..offset + 2] != [0xc0, 0x0c] {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "local DNS fixture returned an invalid answer",
        ));
    }
    let data_len = usize::from(u16::from_be_bytes([
        response[offset + 10],
        response[offset + 11],
    ]));
    let data_start = offset + 12;
    let data_end = data_start
        .checked_add(data_len)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid DNS answer"))?;
    response
        .get(data_start..data_end)
        .map(<[u8]>::to_vec)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "truncated DNS answer"))
}

fn hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(output, "{byte:02x}");
    }
    output
}
