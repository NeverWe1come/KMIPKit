//! A small authoritative DNS fixture bound only to IPv4 loopback.

use std::collections::BTreeMap;
use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

const READ_POLL_INTERVAL: Duration = Duration::from_millis(25);
const MAX_QUERY_BYTES: usize = 512;
const MAX_RECORDS_PER_NAME: usize = 16;

/// A local DNS responder for explicitly configured, loopback-only names.
pub struct LocalDnsFixture {
    local_addr: SocketAddr,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl LocalDnsFixture {
    /// Binds an ephemeral UDP port and serves the supplied loopback addresses.
    ///
    /// Names are matched case-insensitively. Any address that is not loopback,
    /// or any name with too many addresses, is rejected before the socket is
    /// bound.
    ///
    /// # Errors
    ///
    /// Returns an error if a record is invalid or the loopback socket or its
    /// service thread cannot be created.
    pub fn bind(records: BTreeMap<String, Vec<IpAddr>>) -> io::Result<Self> {
        let records = validate_records(records)?;
        let socket = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0))?;
        socket.set_read_timeout(Some(READ_POLL_INTERVAL))?;
        let local_addr = socket.local_addr()?;
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);
        let worker = thread::Builder::new()
            .name("kmipkit-local-dns-fixture".to_owned())
            .spawn(move || serve(&socket, &records, &thread_stop))?;

        Ok(Self {
            local_addr,
            stop,
            thread: Some(worker),
        })
    }

    /// Returns the fixture's ephemeral loopback UDP address.
    #[must_use]
    pub const fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }
}

impl Drop for LocalDnsFixture {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.thread.take() {
            let _join_result = worker.join();
        }
    }
}

fn validate_records(
    records: BTreeMap<String, Vec<IpAddr>>,
) -> io::Result<BTreeMap<String, Vec<IpAddr>>> {
    let mut normalized = BTreeMap::new();
    for (name, addresses) in records {
        let canonical_name = name.trim_end_matches('.').to_ascii_lowercase();
        if !valid_name(&canonical_name)
            || addresses.is_empty()
            || addresses.len() > MAX_RECORDS_PER_NAME
            || addresses.iter().any(|address| !address.is_loopback())
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "local DNS fixture accepts only valid names and loopback addresses",
            ));
        }
        if normalized.insert(canonical_name, addresses).is_some() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "local DNS fixture names must be unique ignoring case",
            ));
        }
    }
    Ok(normalized)
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 253
        && name.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && label
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
                && label.as_bytes()[0].is_ascii_alphanumeric()
                && label.as_bytes()[label.len() - 1].is_ascii_alphanumeric()
        })
}

fn serve(socket: &UdpSocket, records: &BTreeMap<String, Vec<IpAddr>>, stop: &AtomicBool) {
    let mut query = [0_u8; MAX_QUERY_BYTES];
    while !stop.load(Ordering::Acquire) {
        let Ok((length, peer)) = socket.recv_from(&mut query) else {
            continue;
        };
        if let Some(response) = build_response(&query[..length], records) {
            let _ = socket.send_to(&response, peer);
        }
    }
}

fn build_response(query: &[u8], records: &BTreeMap<String, Vec<IpAddr>>) -> Option<Vec<u8>> {
    if query.len() < 12 || query.len() > MAX_QUERY_BYTES {
        return None;
    }
    let query_flags = read_u16(query, 2)?;
    if query_flags & 0x8000 != 0 || read_u16(query, 4)? != 1 {
        return None;
    }

    let (name, question_end) = parse_question_name(query)?;
    let record_type = read_u16(query, question_end.checked_sub(4)?)?;
    let record_class = read_u16(query, question_end.checked_sub(2)?)?;
    let mut response = Vec::with_capacity(query.len() + 128);
    response.extend_from_slice(&query[..2]);
    // QR + AA, preserve RD, return NOERROR, and mark the response non-truncated.
    response.extend_from_slice(&(0x8400_u16 | (query_flags & 0x0100)).to_be_bytes());
    response.extend_from_slice(&1_u16.to_be_bytes());
    let answer_start = response.len();
    response.extend_from_slice(&0_u16.to_be_bytes());
    response.extend_from_slice(&[0; 4]);
    response.extend_from_slice(&query[12..question_end]);

    let values = records.get(&name);
    let matching: Vec<IpAddr> = match (record_class, record_type, values) {
        (1, 1, Some(values)) => values
            .iter()
            .filter(|address| address.is_ipv4())
            .copied()
            .collect(),
        (1, 28, Some(values)) => values
            .iter()
            .filter(|address| address.is_ipv6())
            .copied()
            .collect(),
        _ => Vec::new(),
    };

    let mut answer_count = 0_u16;
    for address in matching {
        response.extend_from_slice(&[0xc0, 0x0c]);
        response.extend_from_slice(&record_type.to_be_bytes());
        response.extend_from_slice(&1_u16.to_be_bytes());
        response.extend_from_slice(&0_u32.to_be_bytes());
        match address {
            IpAddr::V4(address) => {
                response.extend_from_slice(&4_u16.to_be_bytes());
                response.extend_from_slice(&address.octets());
            }
            IpAddr::V6(address) => {
                response.extend_from_slice(&16_u16.to_be_bytes());
                response.extend_from_slice(&address.octets());
            }
        }
        answer_count = answer_count.saturating_add(1);
    }
    response[answer_start..answer_start + 2].copy_from_slice(&answer_count.to_be_bytes());
    Some(response)
}

fn parse_question_name(query: &[u8]) -> Option<(String, usize)> {
    let mut offset = 12;
    let mut labels = Vec::new();
    loop {
        let label_length = usize::from(*query.get(offset)?);
        offset = offset.checked_add(1)?;
        if label_length == 0 {
            break;
        }
        if label_length > 63 {
            return None;
        }
        let label = query.get(offset..offset.checked_add(label_length)?)?;
        if !label.is_ascii()
            || label
                .iter()
                .any(|byte| !byte.is_ascii_alphanumeric() && *byte != b'-')
        {
            return None;
        }
        labels.push(std::str::from_utf8(label).ok()?.to_ascii_lowercase());
        offset = offset.checked_add(label_length)?;
        if offset >= query.len() {
            return None;
        }
    }
    let question_end = offset.checked_add(4)?;
    if labels.is_empty() || question_end > query.len() {
        return None;
    }
    Some((labels.join("."), question_end))
}

fn read_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_be_bytes([
        *bytes.get(offset)?,
        *bytes.get(offset.checked_add(1)?)?,
    ]))
}

#[cfg(test)]
mod tests {
    use super::{build_response, read_u16, validate_records};
    use std::collections::BTreeMap;
    use std::io;
    use std::net::{IpAddr, Ipv4Addr};

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
                vec![IpAddr::V4(Ipv4Addr::LOCALHOST); 17],
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
        assert!(build_response(&[], &records).is_none());
        assert!(build_response(&[0; 513], &records).is_none());

        let mut response_as_query = query(&[1, b'a', 0], 1, 1);
        response_as_query[2] = 0x80;
        assert!(build_response(&response_as_query, &records).is_none());

        let mut multiple_questions = query(&[1, b'a', 0], 1, 1);
        multiple_questions[5] = 2;
        assert!(build_response(&multiple_questions, &records).is_none());

        assert!(build_response(&query(&[64; 1], 1, 1), &records).is_none());
        assert!(build_response(&query(&[1, 0xff, 0], 1, 1), &records).is_none());
        assert!(build_response(&query(&[1, b'a'], 1, 1), &records).is_none());
        assert!(build_response(&query(&[0], 1, 1), &records).is_none());
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

        let response = build_response(&request, &records).expect("valid query receives a reply");

        assert_eq!(read_u16(&response, 6), Some(0));
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
}
