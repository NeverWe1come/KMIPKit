//! Derived client-execution tests for KMIP 2.1.
//!
//! OASIS KMIP Specification v2.1 §§8.3, 8.6, 9.8–9.10, 9.16, and 9.21;
//! Tables 396, 399, 407–409, 421, and 426. Unique Batch Item IDs associate
//! requests and responses; Client Correlation Value is optional and need not
//! be unique; Server Correlation Value is response metadata for client-to-
//! server operations. Batch Order Option controls execution order, while
//! `KMIPKit` associates response items by their IDs regardless of response
//! order. These derived tests are not official OASIS vectors.
//!
//! Traceability: `KMIPKIT-0007-FR-004`, `-FR-006`, and `-FR-016`;
//! `KMIPKIT-REQ-SPEC-8.3-003`, `KMIPKIT-REQ-SPEC-9.21-001-001`,
//! `KMIPKIT-REQ-SPEC-9.21-001-002`, `KMIPKIT-DISC-022`,
//! `KMIPKIT-0006-FR-008`, and `KMIPKIT-0006-FR-022`.

use kmipkit_protocol::ProtocolVersion;
use kmipkit_ttlv::codec::CodecLimits;
use quickcheck::{Arbitrary, Gen, QuickCheck};

use crate::execute::{
    BatchIdentity, ClientBatch, ClientBatchItem, ClientRequest, associate_batch_items,
    request_message_for_test, version_is_supported,
};

const PROPERTY_GEN_SIZE: usize = 32;
const HEADER_PROPERTY_SEED: u64 = 0x4B4D_4950_4B49_5430;
const ORDERED_BATCH_PROPERTY_SEED: u64 = 0x4B4D_4950_4B49_5431;
const UNORDERED_BATCH_PROPERTY_SEED: u64 = 0x4B4D_4950_4B49_5432;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct CandidateProtocolVersion {
    major: i32,
    minor: i32,
}

impl CandidateProtocolVersion {
    const fn new(major: i32, minor: i32) -> Self {
        Self { major, minor }
    }
}

impl Arbitrary for CandidateProtocolVersion {
    fn arbitrary(generator: &mut Gen) -> Self {
        Self::new(i32::arbitrary(generator), i32::arbitrary(generator))
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let major_value = self.major;
        let minor_value = self.minor;
        Box::new(
            major_value
                .shrink()
                .map(move |major| Self::new(major, minor_value))
                .chain(
                    minor_value
                        .shrink()
                        .map(move |minor| Self::new(major_value, minor)),
                ),
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CandidateBatchItem {
    operation: u32,
    unique_batch_item_id: Option<Vec<u8>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CandidateRequest {
    protocol_version: CandidateProtocolVersion,
    batch_order_option: Option<bool>,
    client_correlation_value: Option<String>,
    server_correlation_value: Option<String>,
    batch_items: Vec<CandidateBatchItem>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CandidateResponse {
    protocol_version: CandidateProtocolVersion,
    batch_items: Vec<CandidateBatchItem>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CandidateValidationError {
    Rejected,
}

#[derive(Clone, Copy, Debug)]
struct ReorderedBatchCase(u8);

impl Arbitrary for ReorderedBatchCase {
    fn arbitrary(generator: &mut Gen) -> Self {
        Self(u8::arbitrary(generator))
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(self.0.shrink().map(Self))
    }
}

// Deliberately incomplete test-only seams. T009 supplies the typed execution
// path and replaces these candidate functions with production behavior.
fn candidate_accepts_header_version(version: CandidateProtocolVersion) -> bool {
    version_is_supported(ProtocolVersion::from_raw(version.major, version.minor))
}

fn candidate_build_outgoing_request(
    batch_items: Vec<CandidateBatchItem>,
    batch_order_option: Option<bool>,
    client_correlation_value: Option<String>,
) -> CandidateRequest {
    // The candidate's arbitrary operation numbers exercise association logic;
    // the typed wire request itself is intentionally limited to Discover Versions.
    let mut wire_ids = std::collections::HashSet::new();
    let typed_items = batch_items.iter().enumerate().map(|(index, candidate)| {
        let item = ClientBatchItem::new(ClientRequest::discover_versions());
        match &candidate.unique_batch_item_id {
            Some(id) if wire_ids.insert(id.clone()) => item.with_unique_batch_item_id(id.clone()),
            Some(_) => item.with_unique_batch_item_id(format!("fixture-{index}").into_bytes()),
            None if batch_items.len() > 1 => {
                item.with_unique_batch_item_id(format!("fixture-{index}").into_bytes())
            }
            None => item,
        }
    });
    let mut batch = ClientBatch::from_items(typed_items);
    if let Some(value) = batch_order_option {
        batch = batch.with_batch_order_option(value);
    }
    if let Some(value) = client_correlation_value {
        batch = batch.with_client_correlation_value(value);
    }
    let limits = CodecLimits::defaults();
    let message = request_message_for_test(batch, &limits)
        .expect("the typed Discover Versions request is valid");
    let header = message.header();
    CandidateRequest {
        protocol_version: CandidateProtocolVersion::new(
            header.protocol_version().major(),
            header.protocol_version().minor(),
        ),
        batch_order_option: header.batch_order_option(),
        client_correlation_value: header.with_client_correlation_value(str::to_owned),
        // RequestHeaderView intentionally exposes no server-correlation
        // accessor: that field belongs to server messages.
        server_correlation_value: None,
        batch_items,
    }
}

fn candidate_associate_response(
    request: &CandidateRequest,
    response: &CandidateResponse,
) -> Result<Vec<CandidateBatchItem>, CandidateValidationError> {
    if !candidate_accepts_header_version(response.protocol_version) {
        return Err(CandidateValidationError::Rejected);
    }

    let request_items = request
        .batch_items
        .iter()
        .map(|item| BatchIdentity {
            operation: item.operation,
            unique_batch_item_id: item.unique_batch_item_id.clone(),
        })
        .collect::<Vec<_>>();
    let response_items = response
        .batch_items
        .iter()
        .map(|item| BatchIdentity {
            operation: item.operation,
            unique_batch_item_id: item.unique_batch_item_id.clone(),
        })
        .collect::<Vec<_>>();
    associate_batch_items(&request_items, &response_items)
        .map(|indices| {
            indices
                .into_iter()
                .map(|index| response.batch_items[index].clone())
                .collect()
        })
        .map_err(|_| CandidateValidationError::Rejected)
}

fn item(operation: u32, id: Option<&[u8]>) -> CandidateBatchItem {
    CandidateBatchItem {
        operation,
        unique_batch_item_id: id.map(<[u8]>::to_vec),
    }
}

fn request(
    batch_items: Vec<CandidateBatchItem>,
    batch_order_option: Option<bool>,
    client_correlation_value: Option<&str>,
) -> CandidateRequest {
    candidate_build_outgoing_request(
        batch_items,
        batch_order_option,
        client_correlation_value.map(str::to_owned),
    )
}

fn response(batch_items: Vec<CandidateBatchItem>) -> CandidateResponse {
    CandidateResponse {
        protocol_version: CandidateProtocolVersion::new(2, 1),
        batch_items,
    }
}

fn version_header_acceptance_is_exact(case: CandidateProtocolVersion) -> bool {
    candidate_accepts_header_version(case) == (case == CandidateProtocolVersion::new(2, 1))
}

fn reordered_pair_associates_by_id(case: ReorderedBatchCase, batch_order_option: bool) -> bool {
    let first = item(0x0000_0001, Some(&[case.0]));
    let second = item(0x0000_0002, Some(&[case.0.wrapping_add(1)]));
    let request = request(
        vec![first.clone(), second.clone()],
        Some(batch_order_option),
        None,
    );
    let response = response(vec![second, first.clone()]);

    request.batch_order_option == Some(batch_order_option)
        && candidate_associate_response(&request, &response)
            == Ok(vec![
                first.clone(),
                item(0x0000_0002, Some(&[case.0.wrapping_add(1)])),
            ])
}

fn reordered_pair_associates_by_id_with_ordered_batch(case: ReorderedBatchCase) -> bool {
    reordered_pair_associates_by_id(case, true)
}

fn reordered_pair_associates_by_id_with_unordered_batch(case: ReorderedBatchCase) -> bool {
    reordered_pair_associates_by_id(case, false)
}

#[test]
fn response_header_acceptance_is_exactly_kmip_2_1_for_signed_integer_pairs() {
    QuickCheck::new()
        .rng(Gen::from_size_and_seed(
            PROPERTY_GEN_SIZE,
            HEADER_PROPERTY_SEED,
        ))
        .tests(256)
        .quickcheck(version_header_acceptance_is_exact as fn(CandidateProtocolVersion) -> bool);
}

#[test]
fn response_header_accepts_kmip_2_1() {
    assert!(candidate_accepts_header_version(
        CandidateProtocolVersion::new(2, 1)
    ));
}

#[test]
fn response_header_rejects_signed_integer_boundaries_outside_kmip_2_1() {
    for version in [
        CandidateProtocolVersion::new(i32::MIN, 1),
        CandidateProtocolVersion::new(i32::MAX, 1),
        CandidateProtocolVersion::new(2, i32::MIN),
        CandidateProtocolVersion::new(2, i32::MAX),
    ] {
        assert!(!candidate_accepts_header_version(version));
    }
}

#[test]
fn outgoing_request_header_uses_exactly_kmip_2_1() {
    let outgoing = request(vec![item(1, None)], None, None);

    assert_eq!(
        outgoing.protocol_version,
        CandidateProtocolVersion::new(2, 1)
    );
}

#[test]
fn outgoing_client_request_omits_server_correlation_value() {
    let outgoing = request(vec![item(1, None)], None, None);

    assert_eq!(outgoing.server_correlation_value, None);
}

#[test]
fn matching_response_item_operation_and_id_are_accepted() {
    let request_items = vec![item(7, Some(b"item-7"))];
    let request = request(request_items.clone(), None, None);
    let response_items = request_items;
    let response = response(response_items.clone());

    assert_eq!(
        candidate_associate_response(&request, &response),
        Ok(response_items)
    );
}

#[test]
fn response_item_operation_must_match_its_request_item() {
    let request = request(vec![item(7, Some(b"id-7"))], None, None);
    let response = response(vec![item(8, Some(b"id-7"))]);

    assert_eq!(
        candidate_associate_response(&request, &response),
        Err(CandidateValidationError::Rejected)
    );
}

#[test]
fn response_item_id_must_echo_its_request_item_id() {
    let request = request(vec![item(7, Some(b"request-id"))], None, None);
    let response = response(vec![item(7, Some(b"different-id"))]);

    assert_eq!(
        candidate_associate_response(&request, &response),
        Err(CandidateValidationError::Rejected)
    );
}

#[test]
fn response_item_count_must_match_request_item_count() {
    let request = request(vec![item(7, Some(b"request-id"))], None, None);
    let response = response(Vec::new());

    assert_eq!(
        candidate_associate_response(&request, &response),
        Err(CandidateValidationError::Rejected)
    );
}

#[test]
fn response_header_with_a_non_2_1_version_is_rejected() {
    let request = request(vec![item(7, Some(b"request-id"))], None, None);
    let response = CandidateResponse {
        protocol_version: CandidateProtocolVersion::new(2, 2),
        batch_items: vec![item(7, Some(b"request-id"))],
    };

    assert_eq!(
        candidate_associate_response(&request, &response),
        Err(CandidateValidationError::Rejected)
    );
}

#[test]
fn reordered_response_items_are_associated_by_id_with_both_order_options() {
    let first = item(7, Some(b"first"));
    let second = item(8, Some(b"second"));
    let expected = vec![first.clone(), second.clone()];
    let server_order = vec![second, first];

    let results = [Some(true), Some(false)].map(|batch_order_option| {
        let request = request(expected.clone(), batch_order_option, None);
        assert_eq!(request.batch_order_option, batch_order_option);
        let response = response(server_order.clone());
        candidate_associate_response(&request, &response)
    });

    assert_eq!(results, [Ok(expected.clone()), Ok(expected)]);
}

#[test]
fn reordered_response_property_is_independent_of_true_batch_order_option() {
    QuickCheck::new()
        .rng(Gen::from_size_and_seed(
            PROPERTY_GEN_SIZE,
            ORDERED_BATCH_PROPERTY_SEED,
        ))
        .tests(256)
        .quickcheck(
            reordered_pair_associates_by_id_with_ordered_batch as fn(ReorderedBatchCase) -> bool,
        );
}

#[test]
fn reordered_response_property_is_independent_of_false_batch_order_option() {
    QuickCheck::new()
        .rng(Gen::from_size_and_seed(
            PROPERTY_GEN_SIZE,
            UNORDERED_BATCH_PROPERTY_SEED,
        ))
        .tests(256)
        .quickcheck(
            reordered_pair_associates_by_id_with_unordered_batch as fn(ReorderedBatchCase) -> bool,
        );
}

#[test]
fn multi_item_request_requires_each_unique_batch_item_id() {
    let request = request(
        vec![item(7, Some(b"first")), item(8, None)],
        Some(true),
        None,
    );
    let response = response(vec![item(7, Some(b"first")), item(8, None)]);

    assert!(candidate_associate_response(&request, &response).is_err());
}

#[test]
fn multi_item_request_rejects_duplicate_unique_batch_item_ids() {
    let request = request(
        vec![item(7, Some(b"same")), item(8, Some(b"same"))],
        Some(true),
        None,
    );
    let response = response(vec![item(7, Some(b"same")), item(8, Some(b"same"))]);

    assert!(candidate_associate_response(&request, &response).is_err());
}

#[test]
fn response_rejects_a_missing_unique_batch_item_id_echo() {
    let request = request(
        vec![item(7, Some(b"first")), item(8, Some(b"second"))],
        Some(true),
        None,
    );
    let response = response(vec![item(7, Some(b"first")), item(8, None)]);

    assert!(candidate_associate_response(&request, &response).is_err());
}

#[test]
fn response_rejects_duplicate_unique_batch_item_ids() {
    let request = request(
        vec![item(7, Some(b"first")), item(8, Some(b"second"))],
        Some(true),
        None,
    );
    let response = response(vec![item(7, Some(b"first")), item(8, Some(b"first"))]);

    assert!(candidate_associate_response(&request, &response).is_err());
}

#[test]
fn response_rejects_an_unexpected_unique_batch_item_id() {
    let request = request(
        vec![item(7, Some(b"first")), item(8, Some(b"second"))],
        Some(true),
        None,
    );
    let response = response(vec![item(7, Some(b"first")), item(8, Some(b"unexpected"))]);

    assert!(candidate_associate_response(&request, &response).is_err());
}

#[test]
fn one_item_request_and_response_may_omit_unique_batch_item_id() {
    let request = request(vec![item(7, None)], None, None);
    let response_items = vec![item(7, None)];
    let response = response(response_items.clone());

    assert_eq!(
        candidate_associate_response(&request, &response),
        Ok(response_items)
    );
}

#[test]
fn client_correlation_value_does_not_replace_a_unique_item_id() {
    let request = request(
        vec![item(7, Some(b"unique-id"))],
        None,
        Some("client-label"),
    );
    assert_eq!(
        request.client_correlation_value.as_deref(),
        Some("client-label")
    );
    let response = response(vec![item(7, Some(b"client-label"))]);

    assert!(candidate_associate_response(&request, &response).is_err());
}

#[test]
fn client_correlation_value_does_not_prevent_matching_the_unique_item_id() {
    let request = request(
        vec![item(7, Some(b"unique-id"))],
        None,
        Some("client-label"),
    );
    assert_eq!(
        request.client_correlation_value.as_deref(),
        Some("client-label")
    );
    let response_items = vec![item(7, Some(b"unique-id"))];
    let response = response(response_items.clone());

    assert_eq!(
        candidate_associate_response(&request, &response),
        Ok(response_items)
    );
}
