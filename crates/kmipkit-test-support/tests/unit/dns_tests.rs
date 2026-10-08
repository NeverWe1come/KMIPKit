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

fn query(question_name: &[u8], record_type: u16, record_class: u16) -> Vec<u8> {
    let mut query = vec![0_u8; 12];
    query[2..4].copy_from_slice(&0x0100_u16.to_be_bytes());
    query[4..6].copy_from_slice(&1_u16.to_be_bytes());
    query.extend_from_slice(question_name);
    query.extend_from_slice(&record_type.to_be_bytes());
    query.extend_from_slice(&record_class.to_be_bytes());
    query
}
