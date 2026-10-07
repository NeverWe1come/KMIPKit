use std::collections::BTreeMap;
use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket};
use std::time::Duration;

use kmipkit_test_support::{EphemeralPki, LocalDnsFixture, LoopbackTcpListener};

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

    let formatted = format!("{first:?}");
    assert!(!formatted.contains("private_key_der"));
    assert!(!formatted.contains(&hex(first.server_identity().private_key_der())));
    assert!(!formatted.contains(&hex(first.client_identity().private_key_der())));
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
fn local_transport_listener_never_binds_a_public_interface() {
    let fixture = LoopbackTcpListener::bind()
        .expect("the transport fixture binds to an ephemeral loopback port");

    assert!(fixture.local_addr().ip().is_loopback());
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
