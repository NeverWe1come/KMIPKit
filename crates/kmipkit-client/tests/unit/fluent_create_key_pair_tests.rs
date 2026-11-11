//! Fluent Create Key Pair tests derived from KMIP v2.1 §6.1.9,
//! Tables 189–191, and KMIPKIT-0097-FR-001 through FR-004.

use kmipkit_protocol::attribute::typed;
use kmipkit_protocol::{
    Attribute, AttributeDefinition, CreateKeyPairRequest, CryptographicAlgorithm,
};
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::RequestDeliveryState;
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::{Item, Structure, ValueView};

use crate::asynchronous_execution_test_support::client_for;
use crate::fluent::{CryptographicUsageMask, FluentBuildError};

const RSA: u32 = 4;
const DSA: u32 = 5;
const COMMON_ATTRIBUTES: u32 = 0x0042_0126;
const PRIVATE_KEY_ATTRIBUTES: u32 = 0x0042_0127;
const PUBLIC_KEY_ATTRIBUTES: u32 = 0x0042_0128;
const PRIVATE_NAME: &str = "signing-private-key";
const PUBLIC_NAME: &str = "signing-public-key";
const PRIVATE_SENTINEL: &str = "PRIVATE_KEY_PAIR_INPUT_SENTINEL";

fn attribute_tags(group: &Item) -> Vec<u32> {
    group
        .as_structure()
        .expect("attribute group is a Structure")
        .children()
        .iter()
        .map(|attribute| attribute.tag().raw())
        .collect()
}

fn read_integer(item: &Item) -> Option<i32> {
    item.with_value(|value| match value {
        ValueView::Integer(value) => Some(*value),
        _ => None,
    })
}

fn read_enumeration(item: &Item) -> Option<u32> {
    item.with_value(|value| match value {
        ValueView::Enumeration(value) => Some(*value),
        _ => None,
    })
}

fn assert_name(item: &Item, expected: &str) {
    let fields = item.as_structure().expect("Name is a Structure").children();
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].tag().raw(), 0x0042_0097);
    assert_eq!(read_enumeration(&fields[0]), Some(1));
    assert_eq!(fields[1].tag().raw(), 0x0042_0098);
    assert!(fields[1].with_value(|value| matches!(
        value,
        ValueView::TextString(value) if value == expected
    )));
}

fn payload_groups(payload: &Structure) -> Vec<&Item> {
    payload.view().children().iter().collect()
}

#[test]
fn key_pair_builder_requires_distinct_explicit_algorithm_and_length_values() {
    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    let algorithm_error = client
        .request()
        .create_key_pair()
        .length(2048)
        .private_name(PRIVATE_SENTINEL)
        .build()
        .err()
        .expect("algorithm is required even when length is explicit");
    assert_eq!(
        algorithm_error,
        FluentBuildError::MissingCreateKeyPairAlgorithm
    );

    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    let length_error = client
        .request()
        .create_key_pair()
        .algorithm(CryptographicAlgorithm::from_raw(RSA))
        .private_name(PRIVATE_SENTINEL)
        .build()
        .err()
        .expect("algorithm does not select a key length");
    assert_eq!(length_error, FluentBuildError::MissingCreateKeyPairLength);
    assert_ne!(algorithm_error, length_error);

    for error in [algorithm_error, length_error] {
        assert!(!format!("{error:?} {error}").contains(PRIVATE_SENTINEL));
    }
}

#[test]
fn key_pair_builder_places_semantic_attributes_in_their_exact_key_groups() {
    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    let request: CreateKeyPairRequest = client
        .request()
        .create_key_pair()
        .algorithm(CryptographicAlgorithm::from_raw(RSA))
        .length(2048)
        .private_usage_mask(CryptographicUsageMask::SIGN)
        .private_name(PRIVATE_NAME)
        .public_usage_mask(CryptographicUsageMask::VERIFY)
        .public_name(PUBLIC_NAME)
        .build()
        .expect("explicit semantic values form a typed Create Key Pair request");
    let payload = request
        .into_ttlv_payload()
        .expect("the existing typed request accepts the grouped attributes");
    let groups = payload_groups(&payload);
    assert_eq!(
        groups
            .iter()
            .map(|group| group.tag().raw())
            .collect::<Vec<_>>(),
        [
            COMMON_ATTRIBUTES,
            PRIVATE_KEY_ATTRIBUTES,
            PUBLIC_KEY_ATTRIBUTES
        ]
    );
    assert_eq!(
        attribute_tags(groups[0]),
        [
            typed::CryptographicAlgorithm::TAG,
            typed::CryptographicLength::TAG
        ]
    );
    let common = groups[0].as_structure().unwrap().children();
    assert_eq!(read_enumeration(&common[0]), Some(RSA));
    assert_eq!(read_integer(&common[1]), Some(2048));

    for (group, mask, name) in [
        (groups[1], CryptographicUsageMask::SIGN.raw(), PRIVATE_NAME),
        (groups[2], CryptographicUsageMask::VERIFY.raw(), PUBLIC_NAME),
    ] {
        assert_eq!(
            attribute_tags(group),
            [typed::CryptographicUsageMask::TAG, typed::Name::TAG]
        );
        let attributes = group.as_structure().unwrap().children();
        assert_eq!(read_integer(&attributes[0]), Some(mask));
        assert_name(&attributes[1], name);
    }
}

#[test]
fn key_pair_builder_does_not_synthesize_key_usage_or_curve_attributes() {
    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    let request = client
        .request()
        .create_key_pair()
        .algorithm(CryptographicAlgorithm::from_raw(RSA))
        .length(3072)
        .build()
        .expect("usage masks and names remain optional caller choices");
    assert!(!request.has_private_key_attributes());
    assert!(!request.has_public_key_attributes());
    let payload = request.into_ttlv_payload().unwrap();
    let groups = payload_groups(&payload);
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].tag().raw(), COMMON_ATTRIBUTES);
    assert_eq!(
        attribute_tags(groups[0]),
        [
            typed::CryptographicAlgorithm::TAG,
            typed::CryptographicLength::TAG
        ]
    );
}

#[test]
fn conflicting_table_191_key_algorithms_are_rejected_before_transport() {
    let (mut client, transport, _) = client_for(ExchangeScript::FailBeforeWrite);
    let error = client
        .request()
        .create_key_pair()
        .algorithm(CryptographicAlgorithm::from_raw(RSA))
        .length(2048)
        .private_attribute(Attribute::<typed::CryptographicAlgorithm>::new(RSA).unwrap())
        .public_attribute(Attribute::<typed::CryptographicAlgorithm>::new(DSA).unwrap())
        .private_name(PRIVATE_SENTINEL)
        .execute(&CodecLimits::defaults())
        .expect_err("Table 191 rejects distinct effective algorithms for the pair");

    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(transport.borrow().exchange_count(), 0);
    assert!(!format!("{error:?} {error}").contains(PRIVATE_SENTINEL));
}
